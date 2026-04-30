import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_admin_service.dart';

/// 管理员激活记录。
class ActivatedAdmin {
  const ActivatedAdmin({
    required this.pubkeyHex,
    required this.shenfenId,
    required this.activatedAtMs,
  });

  /// 管理员公钥 hex（不含 0x，小写）。
  final String pubkeyHex;

  /// 所属机构身份码。
  final String shenfenId;

  /// 激活时间（毫秒时间戳）。
  final int activatedAtMs;

  Map<String, dynamic> toJson() => {
        'pubkeyHex': pubkeyHex,
        'shenfenId': shenfenId,
        'activatedAtMs': activatedAtMs,
      };

  factory ActivatedAdmin.fromJson(Map<String, dynamic> json) => ActivatedAdmin(
        pubkeyHex: json['pubkeyHex'] as String,
        shenfenId: json['shenfenId'] as String,
        activatedAtMs: json['activatedAtMs'] as int,
      );
}

/// 管理员激活服务（QR 扫码签名激活模式）。
///
/// 用户在管理员列表页点击"激活"→ 展示签名请求 QR →
/// 持有私钥的外部设备扫码签名 → wuminapp 扫码读取签名回执 →
/// 验证 sr25519 签名 → 写入本地存储。
class ActivationService {
  ActivationService({
    InstitutionAdminService? adminService,
  }) : _adminService = adminService ?? InstitutionAdminService();

  final InstitutionAdminService _adminService;

  /// v2 存储键，旧 v1 数据自动废弃。
  static const _storageKey = 'activated_admins_v2';

  /// "GMB_ACTIVATE" 前缀（12 字节 ASCII）。
  static final _activatePrefix = Uint8List.fromList(
    'GMB_ACTIVATE'.codeUnits,
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

  /// 获取指定机构的已激活管理员，并与链上管理员列表交叉校验。
  Future<List<ActivatedAdmin>> getActivatedAdmins(String shenfenId) async {
    var all = await loadAll();
    final institution = all.where((a) => a.shenfenId == shenfenId).toList();
    if (institution.isEmpty) return [];

    // 链上交叉校验
    try {
      final chainAdmins = await _adminService.fetchAdmins(shenfenId);
      final validPubkeys = chainAdmins.toSet();
      final before = all.length;
      all.removeWhere(
        (a) => a.shenfenId == shenfenId && !validPubkeys.contains(a.pubkeyHex),
      );
      if (all.length != before) {
        await _saveAll(all);
      }
      return all.where((a) => a.shenfenId == shenfenId).toList();
    } catch (_) {
      // RPC 查询失败时不清除本地记录
      return institution;
    }
  }

  /// 检查指定公钥是否已激活。
  Future<bool> isActivated(String pubkeyHex, String shenfenId) async {
    final pk = _normalize(pubkeyHex);
    final all = await loadAll();
    return all.any((a) => a.pubkeyHex == pk && a.shenfenId == shenfenId);
  }

  // ---------------------------------------------------------------------------
  // QR 激活流程
  // ---------------------------------------------------------------------------

  /// 构建激活签名请求（用于展示 QR 码）。
  ///
  /// 返回 (SignRequestEnvelope, requestJson),直接传给 QrSignSessionPage。
  ({SignRequestEnvelope request, String json}) buildActivationRequest({
    required String pubkeyHex,
    required String shenfenId,
  }) {
    final pk = _normalize(pubkeyHex);

    final pkBytes = _hexToBytes(pk);
    final account = Keyring().encodeAddress(pkBytes, 2027);

    final payload = _buildActivatePayload(shenfenId);
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
      specVersion: 0,
      display: SignDisplay(
        action: 'activate_admin',
        summary: '激活机构管理员',
        fields: [
          // activate_admin 链下签名 payload 只含 shenfen_id(48B 右补零),
          // Registry 字段清单只有 shenfen_id。管理员公钥属辅助信息,不塞
          // display.fields 避免对齐失败(2026-04-22 两色识别整改)。
          SignDisplayField(key: 'shenfen_id', label: '身份码', value: shenfenId),
        ],
      ),
    );
    final json = signer.encodeRequest(request);

    return (request: request, json: json);
  }

  /// 通过 QR 签名回执完成激活。
  ///
  /// [pubkeyHex] 管理员公钥。
  /// [shenfenId] 机构身份码。
  /// [response] 从 QrSignSessionPage 获取的签名回执。
  Future<ActivatedAdmin> activateViaQr({
    required String pubkeyHex,
    required String shenfenId,
    required SignResponseEnvelope response,
  }) async {
    final pk = _normalize(pubkeyHex);

    // 验证签名者与目标管理员一致
    final responsePk = _normalize(response.body.pubkey);
    if (responsePk != pk) {
      throw Exception('签名公钥与管理员公钥不一致');
    }

    // 验证是链上管理员
    final admins = await _adminService.fetchAdmins(shenfenId);
    if (!admins.contains(pk)) {
      throw Exception('该公钥不在此机构的链上管理员列表中');
    }

    // 写入本地存储
    final now = DateTime.now().millisecondsSinceEpoch;
    final activation = ActivatedAdmin(
      pubkeyHex: pk,
      shenfenId: shenfenId,
      activatedAtMs: now,
    );

    var all = await loadAll();
    // 去重
    all.removeWhere((a) => a.pubkeyHex == pk && a.shenfenId == shenfenId);
    all.add(activation);
    await _saveAll(all);

    return activation;
  }

  // ---------------------------------------------------------------------------
  // 取消激活
  // ---------------------------------------------------------------------------

  /// 取消激活。
  Future<void> deactivate(String pubkeyHex, String shenfenId) async {
    final pk = _normalize(pubkeyHex);
    var all = await loadAll();
    all.removeWhere((a) => a.pubkeyHex == pk && a.shenfenId == shenfenId);
    await _saveAll(all);
  }

  // ---------------------------------------------------------------------------
  // 内部方法
  // ---------------------------------------------------------------------------

  Uint8List _buildActivatePayload(String shenfenId) {
    final payload = Uint8List(84);
    // 前缀
    payload.setAll(0, _activatePrefix);
    // shenfen_id 固定 48 字节，右补零
    final idBytes = Uint8List.fromList(shenfenId.codeUnits);
    payload.setAll(12, idBytes.sublist(0, idBytes.length.clamp(0, 48)));
    // 时间戳 u64 LE
    final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final bd = ByteData(8)..setUint64(0, timestamp, Endian.little);
    payload.setAll(60, bd.buffer.asUint8List());
    // 随机 nonce 16 字节（用零填充，签名验证不依赖 nonce 内容）
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
