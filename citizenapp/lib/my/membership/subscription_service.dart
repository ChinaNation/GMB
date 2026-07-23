import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/rpc/chain_rpc.dart' show TxPoolWatchCallback;
import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show hexToBytes;
import 'package:citizenapp/wallet/core/secure_seed_store.dart'
    show SecureSeedException;
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:shared_preferences/shared_preferences.dart';

class SubscriptionException implements Exception {
  const SubscriptionException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 平台会员订阅编排：在「我的 → 会员」页订阅 / 取消平台会员（自由/民主/薪火）。
///
/// 用户签名订阅、取消和换档；首次扣款、真实公历到期时间与后续自动扣款由 runtime
/// 根据共识时间戳完成。CitizenApp 不提交续费或周期确认。
class SubscriptionService {
  SubscriptionService({
    SubscriptionRpc? rpc,
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    SquareApiClient? api,
    SharedPreferences? preferences,
  })  : _rpc = rpc ?? SubscriptionRpc(),
        _wallet = walletManager ?? WalletManager(),
        _session = sessionProvider ?? SquareSessionProvider.instance,
        _api = api ?? SquareApiClient(),
        _preferences = preferences;

  final SubscriptionRpc _rpc;
  final WalletManager _wallet;
  final SquareSessionProvider _session;
  final SquareApiClient _api;
  final SharedPreferences? _preferences;

  /// 会员页只以 finalized 链状态和同区块共识时间戳决定当前档位与权益。
  Future<FinalizedSubscriptionSnapshot> fetchFinalizedState(
      String accountId) async {
    await _retryPendingMirrors(accountId);
    return _rpc.fetchSubscriptionSnapshot(subscriberAccountId: accountId);
  }

  /// 订阅平台会员某档（level=freedom/democracy/spark）。
  Future<void> subscribe(
    String level,
    int expectedPriceFen, {
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final wallet = await _requireHotWallet();
    try {
      final result = await _rpc.subscribePlatform(
        fromSs58Address: wallet.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(wallet.accountId)),
        level: level,
        expectedPriceFen: BigInt.from(expectedPriceFen),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        accountId: wallet.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        action: 'subscribe',
        membershipLevel: level,
      );
    } on SecureSeedException catch (e) {
      throw SubscriptionException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw SubscriptionException(e.message);
    } on Exception catch (e) {
      throw SubscriptionException('订阅失败：$e');
    }
  }

  /// 取消平台会员（撤销按月扣款授权）。
  Future<void> cancel({TxPoolWatchCallback? onWatchEvent}) async {
    final wallet = await _requireHotWallet();
    try {
      final result = await _rpc.cancelPlatform(
        fromSs58Address: wallet.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(wallet.accountId)),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        accountId: wallet.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        action: 'cancel',
      );
    } on SecureSeedException catch (e) {
      throw SubscriptionException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw SubscriptionException(e.message);
    } on Exception catch (e) {
      throw SubscriptionException('取消失败：$e');
    }
  }

  /// 更换平台会员档。当前已付周期内仅登记待切换档位，具体生效时间由 runtime 决定。
  Future<void> changePlan(
    String level,
    int expectedPriceFen, {
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final wallet = await _requireHotWallet();
    try {
      final result = await _rpc.changePlatformPlan(
        fromSs58Address: wallet.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(wallet.accountId)),
        level: level,
        expectedPriceFen: BigInt.from(expectedPriceFen),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        accountId: wallet.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        action: 'change',
        membershipLevel: level,
      );
    } on SecureSeedException catch (e) {
      throw SubscriptionException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw SubscriptionException(e.message);
    } on Exception catch (e) {
      throw SubscriptionException('更换订阅失败：$e');
    }
  }

  Future<WalletProfile> _requireHotWallet() async {
    final wallet = await _wallet.getDefaultWallet();
    if (wallet == null || !wallet.isHotWallet) {
      throw const SubscriptionException('请先在「我的 → 我的钱包」创建热钱包');
    }
    return wallet;
  }

  Future<SharedPreferences> get _prefs async {
    final preferences = _preferences;
    if (preferences != null) return preferences;
    return SharedPreferences.getInstance();
  }

  String _pendingKey(String accountId) =>
      'platform_subscription_mirror_pending:$accountId';

  /// finalized 回执先按钱包账户落本地，再提交 Cloudflare；HTTP 失败只重试证明，不再签名。
  Future<void> _confirm({
    required String accountId,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
    required String action,
    String? membershipLevel,
  }) async {
    final proof = <String, dynamic>{
      'tx_hash': txHash,
      'block_hash': blockHashHex,
      'signed_extrinsic_hex': signedExtrinsicHex,
      'action': action,
      if (membershipLevel != null) 'membership_level': membershipLevel,
    };
    try {
      await _storeLocalProof(accountId, proof);
    } on Exception {
      // 链上已 finalized；本地缓存异常不能让用户重新签名。
    }
    try {
      final session = await _session.ensureSession();
      if (session == null || session.accountId != accountId) return;
      await _api.confirmPlatformSubscription(
        session: session,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
        action: action,
        membershipLevel: membershipLevel,
      );
      await _removePendingProof(accountId, txHash);
    } on Exception {
      // 保留本地证明；下次打开会员页只重试 HTTP。
    }
  }

  Future<void> _retryPendingMirrors(String accountId) async {
    try {
      final session = await _session.ensureSession();
      if (session == null || session.accountId != accountId) return;
      final pending = await _readList(_pendingKey(accountId));
      for (final proof in List<Map<String, dynamic>>.from(pending)) {
        final txHash = proof['tx_hash'];
        final blockHashHex = proof['block_hash'];
        final signedExtrinsicHex = proof['signed_extrinsic_hex'];
        final action = proof['action'];
        if (txHash is! String ||
            blockHashHex is! String ||
            signedExtrinsicHex is! String ||
            action is! String) {
          continue;
        }
        await _api.confirmPlatformSubscription(
          session: session,
          txHash: txHash,
          blockHashHex: blockHashHex,
          signedExtrinsicHex: signedExtrinsicHex,
          action: action,
          membershipLevel: proof['membership_level'] as String?,
        );
        await _removePendingProof(accountId, txHash);
      }
    } on Exception {
      // 保留未完成证明；链上订阅与自动续费不依赖 Cloudflare。
    }
  }

  Future<void> _storeLocalProof(
      String accountId, Map<String, dynamic> proof) async {
    final pending = await _readList(_pendingKey(accountId));
    pending.removeWhere((item) => item['tx_hash'] == proof['tx_hash']);
    pending.add(proof);
    await (await _prefs).setString(_pendingKey(accountId), jsonEncode(pending));

    final historyKey = 'subscription_tx_history:$accountId';
    final history = await _readList(historyKey);
    history.removeWhere((item) => item['tx_hash'] == proof['tx_hash']);
    history.add(proof);
    if (history.length > 50) history.removeRange(0, history.length - 50);
    await (await _prefs).setString(historyKey, jsonEncode(history));
  }

  Future<void> _removePendingProof(String accountId, String txHash) async {
    final pending = await _readList(_pendingKey(accountId));
    pending.removeWhere((item) => item['tx_hash'] == txHash);
    final prefs = await _prefs;
    if (pending.isEmpty) {
      await prefs.remove(_pendingKey(accountId));
    } else {
      await prefs.setString(_pendingKey(accountId), jsonEncode(pending));
    }
  }

  Future<List<Map<String, dynamic>>> _readList(String key) async {
    final raw = (await _prefs).getString(key);
    if (raw == null) return <Map<String, dynamic>>[];
    final decoded = jsonDecode(raw);
    return decoded is List
        ? decoded.whereType<Map<String, dynamic>>().toList(growable: true)
        : <Map<String, dynamic>>[];
  }
}
