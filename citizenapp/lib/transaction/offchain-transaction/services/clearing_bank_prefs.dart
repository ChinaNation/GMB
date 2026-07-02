import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

/// 当前钱包绑定清算行的本地快照。
///
///
/// - 链上权威仍然是 `UserBank[user]` 与 `ClearingBankNodes[cid_number]`。
/// - 本地只缓存 UI 和扫码付款所需的索引字段,每次关键操作前都要重新查链上
///   或清算行节点确认,不能把本快照当作信任根。
class ClearingBankBindingSnapshot {
  const ClearingBankBindingSnapshot({
    required this.cidNumber,
    required this.cidFullName,
    required this.cidShortName,
    required this.mainAccount,
    required this.feeAccount,
    required this.peerId,
    required this.rpcDomain,
    required this.rpcPort,
    required this.boundAtMs,
    required this.lastVerifiedAtMs,
  });

  final String cidNumber;
  final String cidFullName;
  final String cidShortName;
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

  String get displayTitle {
    final cidShort = cidShortName.trim();
    if (cidShort.isNotEmpty) return cidShort;
    final cidFull = cidFullName.trim();
    return cidFull.isEmpty ? cidNumber : cidFull;
  }

  Map<String, dynamic> toJson() => {
        'cid_number': cidNumber,
        'cid_full_name': cidFullName,
        'cid_short_name': cidShortName,
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
      cidNumber: (json['cid_number'] as String?) ?? '',
      cidFullName: (json['cid_full_name'] as String?) ?? '',
      cidShortName: (json['cid_short_name'] as String?) ?? '',
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
///
/// - 链上 `OffchainTransaction::UserBank[user]` 存的是**主账户** `AccountId32`
///   (32 字节),**不是** CID `cid_number` 字符串。CitizenApp 同时需要 cid_number、
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
      if (snapshot.cidNumber.isEmpty ||
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

  /// 只写入 `cid_number` 的轻量入口不作为业务真源使用。
  ///
  /// 该入口写入一个不可用于支付的最小快照；真实绑定页面必须调用 [saveSnapshot]。
  static Future<void> save(int walletIndex, String cidNumber) async {
    final trimmed = cidNumber.trim();
    if (trimmed.isEmpty) {
      await clear(walletIndex);
    } else {
      final now = DateTime.now().millisecondsSinceEpoch;
      await saveSnapshot(
        walletIndex,
        ClearingBankBindingSnapshot(
          cidNumber: trimmed,
          cidFullName: '',
          cidShortName: '',
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

  /// 读取 `cid_number`,未绑定/未写入返回 `null`。
  static Future<String?> load(int walletIndex) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString('$_keyPrefix$walletIndex');
    if (raw == null || raw.trim().isEmpty) return null;
    try {
      final json = jsonDecode(raw) as Map<String, dynamic>;
      final cidNumber = (json['cid_number'] as String?)?.trim();
      return (cidNumber == null || cidNumber.isEmpty) ? null : cidNumber;
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
