import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/signer/signing.dart';

/// 管理员激活记录。
class ActivatedAdmin {
  const ActivatedAdmin({
    required this.accountId,
    required this.cidNumber,
    required this.institutionCode,
    required this.kind,
    required this.activatedAtMs,
  });

  /// 管理员账户 ID（小写 `0x` 加 64 位十六进制）。
  final String accountId;

  /// 机构唯一主键；管理员激活不再绑定任一机构账户。
  final String cidNumber;

  /// 链上 institution_code（4 字节机构码字符串，如 "NRC"/"PMUL"/"CGOV"）。
  final String institutionCode;

  /// 链上 AdminAccountKind 编码。
  final int kind;

  /// 激活时间（毫秒时间戳）。
  final int activatedAtMs;

  Map<String, dynamic> toJson() => {
        'account_id': accountId,
        'cid_number': cidNumber,
        'institution_code': institutionCode,
        'kind': kind,
        'activated_at_ms': activatedAtMs,
      };

  factory ActivatedAdmin.fromJson(Map<String, dynamic> json) => ActivatedAdmin(
        accountId: json['account_id'] as String,
        cidNumber: json['cid_number'] as String,
        institutionCode: json['institution_code'] as String,
        kind: json['kind'] as int,
        activatedAtMs: json['activated_at_ms'] as int,
      );
}

/// 管理员激活服务（QR 扫码签名激活模式）。
///
/// 用户在管理员列表页点击"激活"→ 展示签名请求 QR →
/// 持有私钥的外部设备扫码签名 → QrSignSessionPage 校验签名响应 →
/// 本服务复核链上管理员账户和签名账户 ID → 写入本地存储。
class ActivationService {
  ActivationService({
    InstitutionAdminService? adminService,
  }) : _adminService = adminService ?? InstitutionAdminService();

  final InstitutionAdminService _adminService;

  /// 只保存 CID 语义的当前激活记录；不读取旧账户主键记录。
  static const _storageKey = 'activated_institution_admins';

  /// AccountId 级管理员激活 payload 4 字节二进制前缀 = GMB || 0x18。
  ///
  /// 二进制前缀域:此前缀**内嵌在被签 payload 字节里**
  /// (冷钱包对整段 payloadHex 直接 sr25519 签名,node 按字节偏移解析),不经
  /// signingMessage 做 blake2 hash。
  /// 四方逐字节锁步:node activation.rs(build/decode)、冷钱包
  /// payload_decoder.dart、本服务。前缀单源对齐 primitives::sign::
  /// binary_domain_prefix,金标见 test/signer/fixtures/
  /// binary_prefix_domain_vectors.json。
  // 读取
  /// 加载所有激活记录。
  Future<List<ActivatedAdmin>> loadAll() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_storageKey);
    if (raw == null || raw.isEmpty) return [];
    try {
      final list = jsonDecode(raw) as List<dynamic>;
      return list
          .map((e) => ActivatedAdmin.fromJson(e as Map<String, dynamic>))
          .toList();
    } catch (_) {
      return [];
    }
  }

  /// 获取指定管理员账户的已激活管理员，并与链上管理员列表交叉校验。
  Future<List<ActivatedAdmin>> getActivatedAdmins(
      AdminAccountIdentity identity) async {
    final cidNumber = _requireInstitutionCid(identity);
    var all = await loadAll();
    final institutionRecords =
        all.where((item) => item.cidNumber == cidNumber).toList();
    if (institutionRecords.isEmpty) return [];

    // 链上交叉校验
    try {
      final chainAdmins = await _adminService.fetchAdmins(identity);
      final validAccountIds =
          chainAdmins.map((admin) => admin.account_id).toSet();
      final before = all.length;
      all.removeWhere(
        (a) =>
            a.cidNumber == cidNumber && !validAccountIds.contains(a.accountId),
      );
      if (all.length != before) {
        await _saveAll(all);
      }
      return all.where((a) => a.cidNumber == cidNumber).toList();
    } catch (_) {
      // RPC 查询失败时不清除本地记录
      return institutionRecords;
    }
  }

  /// 检查指定账户 ID 是否已激活。
  Future<bool> isActivated(
      String accountId, AdminAccountIdentity identity) async {
    final normalizedAccountId = _normalize(accountId);
    final cidNumber = _requireInstitutionCid(identity);
    final all = await loadAll();
    return all.any(
      (a) => a.accountId == normalizedAccountId && a.cidNumber == cidNumber,
    );
  }

  // QR 激活流程
  /// 构建激活签名请求（用于展示 QR 码）。
  ///
  /// 返回 (SignRequestEnvelope, requestJson),直接传给 QrSignSessionPage。
  ({SignRequestEnvelope request, String json}) buildActivationRequest({
    required String accountId,
    required AdminAccountIdentity identity,
  }) {
    _requireInstitutionCid(identity);
    final normalizedAccountId = _normalize(accountId);

    final payload = _buildActivatePayload(identity, normalizedAccountId);
    final payloadHex = '0x${_bytesToHex(payload)}';

    final signer = QrSigner();
    final requestId = QrSigner.generateRequestId(prefix: 'act-');
    final request = signer.buildRequest(
      requestId: requestId,
      signerPublicKey: normalizedAccountId,
      payloadHex: payloadHex,
      action: QrActions.activateAdmin,
    );
    final json = signer.encodeRequest(request);

    return (request: request, json: json);
  }

  /// 通过 QR 签名响应完成激活。
  ///
  /// [accountId] 管理员账户 ID。
  /// [identity] 管理员账户。
  /// [response] 从 QrSignSessionPage 获取的签名响应。
  Future<ActivatedAdmin> activateViaQr({
    required String accountId,
    required AdminAccountIdentity identity,
    required SignResponseEnvelope response,
  }) async {
    final cidNumber = _requireInstitutionCid(identity);
    final normalizedAccountId = _normalize(accountId);

    // 验证签名者与目标管理员一致
    final responsePk = _normalize(response.body.signerPublicKeyHex);
    if (responsePk != normalizedAccountId) {
      throw Exception('签名账户 ID 与管理员账户 ID 不一致');
    }

    // 验证是链上管理员
    final admins = await _adminService.fetchAdmins(identity);
    if (!admins.any((admin) => admin.account_id == normalizedAccountId)) {
      throw Exception('该账户 ID 不在此管理员账户的链上管理员列表中');
    }

    // 写入本地存储
    final now = DateTime.now().millisecondsSinceEpoch;
    final activation = ActivatedAdmin(
      accountId: normalizedAccountId,
      cidNumber: cidNumber,
      institutionCode: identity.institutionCode,
      kind: identity.kind,
      activatedAtMs: now,
    );

    var all = await loadAll();
    // 去重
    all.removeWhere(
      (a) => a.accountId == normalizedAccountId && a.cidNumber == cidNumber,
    );
    all.add(activation);
    await _saveAll(all);

    return activation;
  }

  // 取消激活
  /// 取消激活。
  Future<void> deactivate(
      String accountId, AdminAccountIdentity identity) async {
    final normalizedAccountId = _normalize(accountId);
    final cidNumber = _requireInstitutionCid(identity);
    var all = await loadAll();
    all.removeWhere(
      (a) => a.accountId == normalizedAccountId && a.cidNumber == cidNumber,
    );
    await _saveAll(all);
  }

  // 内部方法
  Uint8List _buildActivatePayload(
      AdminAccountIdentity identity, String accountId) {
    final signerPublicKey = _hexToBytes(accountId);
    final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final random = Random.secure();
    return activateAdminPayload(
      cidNumber: _requireInstitutionCid(identity),
      institutionCode: InstitutionCodeLabel.codeBytes(identity.institutionCode),
      kind: identity.kind,
      signerPublicKey: signerPublicKey,
      timestamp: timestamp,
      nonce: List<int>.generate(
        kAdminNonceLength,
        (_) => random.nextInt(256),
        growable: false,
      ),
    );
  }

  static String _requireInstitutionCid(AdminAccountIdentity identity) {
    if (identity.type != AdminAccountIdentityType.institution) {
      throw ArgumentError('管理员激活只适用于机构 CID；个人多签不使用机构激活协议');
    }
    return identity.cidNumber!;
  }

  Future<void> _saveAll(List<ActivatedAdmin> all) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = jsonEncode(all.map((a) => a.toJson()).toList());
    await prefs.setString(_storageKey, raw);
  }

  static String _normalize(String accountId) {
    if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId)) {
      throw const FormatException('account_id 必须为小写 0x + 64 位十六进制');
    }
    return accountId;
  }

  static Uint8List _hexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  static String _bytesToHex(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
