import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

/// 管理员激活记录。
class ActivatedAdmin {
  const ActivatedAdmin({
    required this.pubkeyHex,
    required this.identityKey,
    required this.subjectIdHex,
    required this.org,
    required this.kind,
    required this.activatedAtMs,
  });

  /// 管理员公钥 hex（不含 0x，小写）。
  final String pubkeyHex;

  /// 管理员主体业务身份 key，不参与链上编码。
  final String identityKey;

  /// admins-change 链上主体 SubjectId hex（不含 0x，小写）。
  final String subjectIdHex;

  /// 链上 org 编码。
  final int org;

  /// 链上 AdminSubjectKind 编码。
  final int kind;

  /// 激活时间（毫秒时间戳）。
  final int activatedAtMs;

  Map<String, dynamic> toJson() => {
        'pubkeyHex': pubkeyHex,
        'identityKey': identityKey,
        'subjectIdHex': subjectIdHex,
        'org': org,
        'kind': kind,
        'activatedAtMs': activatedAtMs,
      };

  factory ActivatedAdmin.fromJson(Map<String, dynamic> json) => ActivatedAdmin(
        pubkeyHex: json['pubkeyHex'] as String,
        identityKey: json['identityKey'] as String,
        subjectIdHex: json['subjectIdHex'] as String,
        org: json['org'] as int,
        kind: json['kind'] as int,
        activatedAtMs: json['activatedAtMs'] as int,
      );
}

/// 管理员激活服务（QR 扫码签名激活模式）。
///
/// 用户在管理员列表页点击"激活"→ 展示签名请求 QR →
/// 持有私钥的外部设备扫码签名 → QrSignSessionPage 校验签名回执 →
/// 本服务复核链上管理员主体和签名公钥 → 写入本地存储。
class ActivationService {
  ActivationService({
    InstitutionAdminService? adminService,
  }) : _adminService = adminService ?? InstitutionAdminService();

  final InstitutionAdminService _adminService;

  /// v3 只保存 subject 语义；旧 v1/v2 激活记录不读取、不迁移。
  static const _storageKey = 'activated_admins_v3';

  /// subject 级管理员激活 payload 前缀。
  static final _activatePrefix = Uint8List.fromList(
    'GMB_ACTIVATE_SUBJECT_V1'.codeUnits,
  );

  // ---------------------------------------------------------------------------
  // 读取
  // ---------------------------------------------------------------------------

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

  /// 获取指定管理员主体的已激活管理员，并与链上管理员列表交叉校验。
  Future<List<ActivatedAdmin>> getActivatedAdmins(
      AdminSubjectIdentity identity) async {
    var all = await loadAll();
    final subjectId = _normalize(identity.subjectIdHex);
    final subjectRecords =
        all.where((a) => _normalize(a.subjectIdHex) == subjectId).toList();
    if (subjectRecords.isEmpty) return [];

    // 链上交叉校验
    try {
      final chainAdmins = await _adminService.fetchAdmins(identity);
      final validPubkeys = chainAdmins.toSet();
      final before = all.length;
      all.removeWhere(
        (a) =>
            _normalize(a.subjectIdHex) == subjectId &&
            !validPubkeys.contains(a.pubkeyHex),
      );
      if (all.length != before) {
        await _saveAll(all);
      }
      return all.where((a) => _normalize(a.subjectIdHex) == subjectId).toList();
    } catch (_) {
      // RPC 查询失败时不清除本地记录
      return subjectRecords;
    }
  }

  /// 检查指定公钥是否已激活。
  Future<bool> isActivated(
      String pubkeyHex, AdminSubjectIdentity identity) async {
    final pk = _normalize(pubkeyHex);
    final subjectId = _normalize(identity.subjectIdHex);
    final all = await loadAll();
    return all.any(
        (a) => a.pubkeyHex == pk && _normalize(a.subjectIdHex) == subjectId);
  }

  // ---------------------------------------------------------------------------
  // QR 激活流程
  // ---------------------------------------------------------------------------

  /// 构建激活签名请求（用于展示 QR 码）。
  ///
  /// 返回 (SignRequestEnvelope, requestJson),直接传给 QrSignSessionPage。
  ({SignRequestEnvelope request, String json}) buildActivationRequest({
    required String pubkeyHex,
    required AdminSubjectIdentity identity,
  }) {
    final pk = _normalize(pubkeyHex);

    final pkBytes = _hexToBytes(pk);
    final account = Keyring().encodeAddress(pkBytes, 2027);

    final payload = _buildActivatePayload(identity, pk);
    // feedback_pubkey_format_rule 铁律: 内部统一 0x 小写 hex。
    // wumin SignRequestBody.fromJson 严格要求 pubkey / payload_hex
    // 以 0x 开头,缺前缀会抛 "签名请求解析失败"(2026-04-22 修复)。
    final payloadHex = '0x${_bytesToHex(payload)}';

    final signer = QrSigner();
    final requestId = QrSigner.generateRequestId(prefix: 'act-');
    final request = signer.buildRequest(
      requestId: requestId,
      address: account,
      pubkey: '0x$pk',
      payloadHex: payloadHex,
      display: SignDisplay(
        action: 'activate_admin_subject',
        summary: '激活${identity.orgLabel}管理员',
        fields: [
          SignDisplayField(key: 'org', label: '组织类型', value: identity.orgLabel),
          SignDisplayField(
              key: 'subject',
              label: '管理员主体',
              value: '0x${identity.subjectIdHex}'),
          SignDisplayField(key: 'pubkey', label: '管理员公钥', value: '0x$pk'),
        ],
      ),
    );
    final json = signer.encodeRequest(request);

    return (request: request, json: json);
  }

  /// 通过 QR 签名回执完成激活。
  ///
  /// [pubkeyHex] 管理员公钥。
  /// [identity] 管理员主体。
  /// [response] 从 QrSignSessionPage 获取的签名回执。
  Future<ActivatedAdmin> activateViaQr({
    required String pubkeyHex,
    required AdminSubjectIdentity identity,
    required SignResponseEnvelope response,
  }) async {
    final pk = _normalize(pubkeyHex);

    // 验证签名者与目标管理员一致
    final responsePk = _normalize(response.body.pubkey);
    if (responsePk != pk) {
      throw Exception('签名公钥与管理员公钥不一致');
    }

    // 验证是链上管理员
    final admins = await _adminService.fetchAdmins(identity);
    if (!admins.contains(pk)) {
      throw Exception('该公钥不在此管理员主体的链上管理员列表中');
    }

    // 写入本地存储
    final now = DateTime.now().millisecondsSinceEpoch;
    final activation = ActivatedAdmin(
      pubkeyHex: pk,
      identityKey: identity.identityKey,
      subjectIdHex: identity.subjectIdHex,
      org: identity.org,
      kind: identity.kind,
      activatedAtMs: now,
    );

    var all = await loadAll();
    // 去重
    final subjectId = _normalize(identity.subjectIdHex);
    all.removeWhere(
        (a) => a.pubkeyHex == pk && _normalize(a.subjectIdHex) == subjectId);
    all.add(activation);
    await _saveAll(all);

    return activation;
  }

  // ---------------------------------------------------------------------------
  // 取消激活
  // ---------------------------------------------------------------------------

  /// 取消激活。
  Future<void> deactivate(
      String pubkeyHex, AdminSubjectIdentity identity) async {
    final pk = _normalize(pubkeyHex);
    final subjectId = _normalize(identity.subjectIdHex);
    var all = await loadAll();
    all.removeWhere(
        (a) => a.pubkeyHex == pk && _normalize(a.subjectIdHex) == subjectId);
    await _saveAll(all);
  }

  // ---------------------------------------------------------------------------
  // 内部方法
  // ---------------------------------------------------------------------------

  Uint8List _buildActivatePayload(
      AdminSubjectIdentity identity, String pubkeyHex) {
    final subjectId = AdminSubjectIdCodec.fromHex(identity.subjectIdHex);
    final pubkey = _hexToBytes(pubkeyHex);
    final payload =
        Uint8List(_activatePrefix.length + 48 + 1 + 1 + 32 + 8 + 16);
    var offset = 0;
    payload.setAll(offset, _activatePrefix);
    offset += _activatePrefix.length;
    payload.setAll(offset, subjectId);
    offset += 48;
    payload[offset++] = identity.org;
    payload[offset++] = identity.kind;
    payload.setAll(offset, pubkey);
    offset += 32;
    final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final bd = ByteData(8)..setUint64(0, timestamp, Endian.little);
    payload.setAll(offset, bd.buffer.asUint8List());
    // 中文注释：保留 nonce 字段位置；签名验证绑定 subject/pubkey/timestamp。
    return payload;
  }

  Future<void> _saveAll(List<ActivatedAdmin> all) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = jsonEncode(all.map((a) => a.toJson()).toList());
    await prefs.setString(_storageKey, raw);
  }

  static String _normalize(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
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
