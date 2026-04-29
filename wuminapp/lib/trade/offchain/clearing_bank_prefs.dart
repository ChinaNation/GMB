import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

/// 当前钱包绑定清算行的本地快照。
///
/// 中文注释:
/// - 链上权威仍然是 `UserBank[user]` 与 `ClearingBankNodes[sfid_id]`。
/// - 本地只缓存 UI 和扫码付款所需的索引字段,每次关键操作前都要重新查链上
///   或清算行节点确认,不能把本快照当作信任根。
class ClearingBankBindingSnapshot {
  const ClearingBankBindingSnapshot({
    required this.sfidId,
    required this.institutionName,
    required this.mainAccount,
    required this.feeAccount,
    required this.peerId,
    required this.rpcDomain,
    required this.rpcPort,
    required this.boundAtMs,
    required this.lastVerifiedAtMs,
  });

  final String sfidId;
  final String institutionName;
  final String mainAccount;
  final String? feeAccount;
  final String peerId;
  final String rpcDomain;
  final int rpcPort;
  final int boundAtMs;
  final int lastVerifiedAtMs;

  String get wssUrl {
    final isLocal = rpcDomain == '127.0.0.1' || rpcDomain == 'localhost';
    final scheme = isLocal ? 'ws' : 'wss';
    return '$scheme://$rpcDomain:$rpcPort';
  }

  Map<String, dynamic> toJson() => {
        'sfid_id': sfidId,
        'institution_name': institutionName,
        'main_account': mainAccount,
        'fee_account': feeAccount,
        'peer_id': peerId,
        'rpc_domain': rpcDomain,
        'rpc_port': rpcPort,
        'bound_at_ms': boundAtMs,
        'last_verified_at_ms': lastVerifiedAtMs,
      };

  factory ClearingBankBindingSnapshot.fromJson(Map<String, dynamic> json) {
    return ClearingBankBindingSnapshot(
      sfidId: (json['sfid_id'] as String?) ?? '',
      institutionName: (json['institution_name'] as String?) ?? '',
      mainAccount: (json['main_account'] as String?) ?? '',
      feeAccount: json['fee_account'] as String?,
      peerId: (json['peer_id'] as String?) ?? '',
      rpcDomain: (json['rpc_domain'] as String?) ?? '',
      rpcPort: (json['rpc_port'] as num?)?.toInt() ?? 0,
      boundAtMs: (json['bound_at_ms'] as num?)?.toInt() ?? 0,
      lastVerifiedAtMs: (json['last_verified_at_ms'] as num?)?.toInt() ?? 0,
    );
  }
}

/// 扫码支付 Step 3:**用户绑定清算行本地快照缓存**。
///
/// 中文注释:
/// - 链上 `OffchainTransaction::UserBank[user]` 存的是**主账户** `AccountId32`
///   (32 字节),**不是** SFID `shenfen_id` 字符串。wuminapp 同时需要 sfid_id、
///   主账户和链上 `ClearingBankNodes` 端点,所以本地缓存升级为 JSON 快照。
/// - 快照仅是用户体验缓存;绑定、支付、充值、提现前仍要查链上或清算行节点。
/// - 缓存按 `walletIndex` 隔离,同 App 多钱包互不干扰。
class ClearingBankPrefs {
  ClearingBankPrefs._();

  static const String _keyPrefix = 'clearing_bank_binding_';

  /// 写入完整绑定快照。
  static Future<void> saveSnapshot(
    int walletIndex,
    ClearingBankBindingSnapshot snapshot,
  ) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
      '$_keyPrefix$walletIndex',
      jsonEncode(snapshot.toJson()),
    );
  }

  /// 读取完整绑定快照。
  static Future<ClearingBankBindingSnapshot?> loadSnapshot(
    int walletIndex,
  ) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString('$_keyPrefix$walletIndex');
    if (raw == null || raw.trim().isEmpty) return null;
    try {
      final json = jsonDecode(raw) as Map<String, dynamic>;
      final snapshot = ClearingBankBindingSnapshot.fromJson(json);
      if (snapshot.sfidId.isEmpty ||
          snapshot.mainAccount.isEmpty ||
          snapshot.rpcDomain.isEmpty ||
          snapshot.rpcPort <= 0) {
        return null;
      }
      return snapshot;
    } catch (_) {
      return null;
    }
  }

  /// 只写入 `shenfen_id` 的旧便捷入口不再作为业务真源使用。
  ///
  /// 这里保留给少量测试和过渡调用,会写入一个不可用于支付的最小快照;真实绑定
  /// 页面必须调用 [saveSnapshot]。
  static Future<void> save(int walletIndex, String shenfenId) async {
    final trimmed = shenfenId.trim();
    if (trimmed.isEmpty) {
      await clear(walletIndex);
    } else {
      final now = DateTime.now().millisecondsSinceEpoch;
      await saveSnapshot(
        walletIndex,
        ClearingBankBindingSnapshot(
          sfidId: trimmed,
          institutionName: '',
          mainAccount: '',
          feeAccount: null,
          peerId: '',
          rpcDomain: '',
          rpcPort: 0,
          boundAtMs: now,
          lastVerifiedAtMs: now,
        ),
      );
    }
  }

  /// 读取 `sfid_id`,未绑定/未写入返回 `null`。
  static Future<String?> load(int walletIndex) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString('$_keyPrefix$walletIndex');
    if (raw == null || raw.trim().isEmpty) return null;
    try {
      final json = jsonDecode(raw) as Map<String, dynamic>;
      final sfidId = (json['sfid_id'] as String?)?.trim();
      return (sfidId == null || sfidId.isEmpty) ? null : sfidId;
    } catch (_) {
      return null;
    }
  }

  /// 清除(切换清算行后由 bind 页主动覆盖,或用户手动解绑时调)。
  static Future<void> clear(int walletIndex) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('$_keyPrefix$walletIndex');
  }
}
