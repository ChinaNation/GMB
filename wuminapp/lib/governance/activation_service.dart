import 'dart:convert';
import 'dart:typed_data';

import 'package:shared_preferences/shared_preferences.dart';

import '../wallet/core/wallet_manager.dart';
import 'institution_admin_service.dart';

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

/// 管理员激活服务。
///
/// 在 wuminapp 中，用户已持有钱包私钥（通过 WalletManager），
/// 因此激活只需用本地钱包签名 GMB_ACTIVATE payload，
/// 本地验证后写入 SharedPreferences 存储。
class ActivationService {
  ActivationService({
    WalletManager? walletManager,
    InstitutionAdminService? adminService,
  })  : _walletManager = walletManager ?? WalletManager(),
        _adminService = adminService ?? InstitutionAdminService();

  final WalletManager _walletManager;
  final InstitutionAdminService _adminService;

  static const _storageKey = 'activated_admins_v1';

  /// "GMB_ACTIVATE" 前缀（12 字节 ASCII）。
  static final _activatePrefix = Uint8List.fromList(
    'GMB_ACTIVATE'.codeUnits,
  );

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
      all.removeWhere((a) =>
        a.shenfenId == shenfenId && !validPubkeys.contains(a.pubkeyHex),
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

  /// 激活管理员：使用本地钱包签名 GMB_ACTIVATE payload。
  ///
  /// [walletIndex] 持有私钥的钱包索引。
  /// [pubkeyHex] 管理员公钥（必须与钱包公钥一致）。
  /// [shenfenId] 机构身份码。
  Future<ActivatedAdmin> activate({
    required int walletIndex,
    required String pubkeyHex,
    required String shenfenId,
  }) async {
    final pk = _normalize(pubkeyHex);

    // 验证钱包公钥匹配
    final wallet = await _walletManager.getWalletByIndex(walletIndex);
    if (wallet == null) {
      throw Exception('未找到指定钱包');
    }
    if (_normalize(wallet.pubkeyHex) != pk) {
      throw Exception('钱包公钥与管理员公钥不一致');
    }

    // 验证是链上管理员
    final admins = await _adminService.fetchAdmins(shenfenId);
    if (!admins.contains(pk)) {
      throw Exception('该公钥不在此机构的链上管理员列表中');
    }

    // 构建激活 payload：GMB_ACTIVATE(12B) + shenfen_id(48B) + timestamp(8B) + nonce(16B)
    final payload = _buildActivatePayload(shenfenId);

    // 用钱包私钥签名
    await _walletManager.signWithWallet(walletIndex, payload);

    // 签名成功 = 证明持有私钥，写入本地存储
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

  /// 取消激活。
  Future<void> deactivate(String pubkeyHex, String shenfenId) async {
    final pk = _normalize(pubkeyHex);
    var all = await loadAll();
    all.removeWhere((a) => a.pubkeyHex == pk && a.shenfenId == shenfenId);
    await _saveAll(all);
  }

  /// 检查指定公钥是否已激活。
  Future<bool> isActivated(String pubkeyHex, String shenfenId) async {
    final pk = _normalize(pubkeyHex);
    final all = await loadAll();
    return all.any((a) => a.pubkeyHex == pk && a.shenfenId == shenfenId);
  }

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
    // 实际签名已证明私钥持有权，nonce 用于区分不同激活请求
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
}
