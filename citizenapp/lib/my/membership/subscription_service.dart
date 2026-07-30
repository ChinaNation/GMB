import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart'
    show ResolvedIdentity;
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

/// 会员页持久化展示快照：只缓存低频变化的 finalized 订阅态与三档链上价格。
///
/// 套餐名称和权益不进入缓存，始终使用 App 内置静态定义；订阅或支付动作仍在提交前
/// 使用链上真值校验，展示缓存不构成授权真源。
class MembershipDisplaySnapshot {
  const MembershipDisplaySnapshot({
    required this.state,
    required this.prices,
    required this.subscriptionFetchedAtMs,
    required this.pricesFetchedAtMs,
  });

  final SquareMembershipState state;
  final Map<String, int> prices;
  final int subscriptionFetchedAtMs;
  final int pricesFetchedAtMs;

  bool subscriptionIsFresh(int nowMs, Duration ttl) =>
      subscriptionFetchedAtMs > 0 &&
      nowMs >= subscriptionFetchedAtMs &&
      nowMs - subscriptionFetchedAtMs <= ttl.inMilliseconds;

  bool pricesAreFresh(int nowMs, Duration ttl) =>
      pricesFetchedAtMs > 0 &&
      nowMs >= pricesFetchedAtMs &&
      nowMs - pricesFetchedAtMs <= ttl.inMilliseconds;
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
      String cidNumber) async {
    // Cloudflare 只是 finalized 回执镜像；历史回执重试不得阻塞会员页链上真态读取。
    unawaited(_retryPendingMirrorsForCurrentSession());
    return _rpc.fetchSubscriptionSnapshot(subscriberCidNumber: cidNumber);
  }

  String _displaySnapshotKey(String cidNumber) =>
      'platform_membership_display_snapshot:$cidNumber';

  /// 读取当前账户上一次成功同步的展示快照；损坏缓存直接丢弃，绝不阻塞静态卡片。
  Future<MembershipDisplaySnapshot?> readDisplaySnapshot(
      String cidNumber) async {
    final preferences = await _prefs;
    final key = _displaySnapshotKey(cidNumber);
    final raw = preferences.getString(key);
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final pricesRaw = decoded['prices'];
      final prices = <String, int>{};
      if (pricesRaw is Map<String, dynamic>) {
        for (final level in const ['freedom', 'democracy', 'spark']) {
          final value = pricesRaw[level];
          if (value is int && value >= 0) prices[level] = value;
        }
      }
      final membershipLevel = decoded['membership_level'];
      final subscriptionStatus = decoded['subscription_status'];
      return MembershipDisplaySnapshot(
        state: SquareMembershipState(
          active: decoded['active'] == true,
          paidUntil:
              decoded['paid_until'] is int ? decoded['paid_until'] as int : 0,
          membershipLevel: membershipLevel is String ? membershipLevel : null,
          subscriptionStatus:
              subscriptionStatus is String ? subscriptionStatus : null,
          subscriptionActive: decoded['subscription_active'] == true,
          lastChargedAt: decoded['last_charged_at'] is int
              ? decoded['last_charged_at'] as int
              : 0,
        ),
        prices: prices,
        subscriptionFetchedAtMs: decoded['subscription_fetched_at_ms'] is int
            ? decoded['subscription_fetched_at_ms'] as int
            : 0,
        pricesFetchedAtMs: decoded['prices_fetched_at_ms'] is int
            ? decoded['prices_fetched_at_ms'] as int
            : 0,
      );
    } on FormatException {
      await preferences.remove(key);
      return null;
    }
  }

  /// 原子覆盖当前账户展示快照；调用方只在对应链读成功后推进该部分时间戳。
  Future<void> writeDisplaySnapshot(
    String cidNumber,
    MembershipDisplaySnapshot snapshot,
  ) async {
    await (await _prefs).setString(
      _displaySnapshotKey(cidNumber),
      jsonEncode({
        'active': snapshot.state.active,
        'paid_until': snapshot.state.paidUntil,
        'membership_level': snapshot.state.membershipLevel,
        'subscription_status': snapshot.state.subscriptionStatus,
        'subscription_active': snapshot.state.subscriptionActive,
        'last_charged_at': snapshot.state.lastChargedAt,
        'prices': snapshot.prices,
        'subscription_fetched_at_ms': snapshot.subscriptionFetchedAtMs,
        'prices_fetched_at_ms': snapshot.pricesFetchedAtMs,
      }),
    );
  }

  /// 订阅平台会员某档（level=freedom/democracy/spark）。
  Future<void> subscribe(
    String level,
    int expectedPriceFen, {
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    await _requireHotWallet();
    final identity = await _requireIdentity();
    final cidNumber = identity.snapshot!.cidNumber;
    try {
      final result = await _rpc.subscribePlatform(
        fromSs58Address: identity.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(identity.accountId)),
        level: level,
        expectedPriceFen: BigInt.from(expectedPriceFen),
        sign: (payload) =>
            _wallet.signForAccountId(identity.accountId, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        subscriberCidNumber: cidNumber,
        signerAccountId: identity.accountId,
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
    await _requireHotWallet();
    final identity = await _requireIdentity();
    final cidNumber = identity.snapshot!.cidNumber;
    try {
      final result = await _rpc.cancelPlatform(
        fromSs58Address: identity.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(identity.accountId)),
        sign: (payload) =>
            _wallet.signForAccountId(identity.accountId, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        subscriberCidNumber: cidNumber,
        signerAccountId: identity.accountId,
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
    await _requireHotWallet();
    final identity = await _requireIdentity();
    final cidNumber = identity.snapshot!.cidNumber;
    try {
      final result = await _rpc.changePlatformPlan(
        fromSs58Address: identity.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(identity.accountId)),
        level: level,
        expectedPriceFen: BigInt.from(expectedPriceFen),
        sign: (payload) =>
            _wallet.signForAccountId(identity.accountId, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        subscriberCidNumber: cidNumber,
        signerAccountId: identity.accountId,
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

  /// 身份账户（CID 绑定账户，单源 [IdentityAccountCache]）：链上订阅交易的唯一签名者。
  /// 本地待提交证明归属永久 CID；账户只记录当时的签名与付款事实。
  Future<ResolvedIdentity> _requireIdentity() async {
    final identity = await IdentityAccountCache.instance.resolve();
    if (identity == null || !identity.isRegistered) {
      throw const SubscriptionException('请先注册并绑定公民 CID');
    }
    return identity;
  }

  Future<SharedPreferences> get _prefs async {
    final preferences = _preferences;
    if (preferences != null) return preferences;
    return SharedPreferences.getInstance();
  }

  String _pendingKey(String subscriberCidNumber) =>
      'platform_subscription_mirror_pending_by_cid:$subscriberCidNumber';

  /// finalized 回执按永久 CID 落本地，再提交 Cloudflare；当时的签名账户只作为
  /// 交易事实写入证明。HTTP 失败只重试证明，不再签名。
  Future<void> _confirm({
    required String subscriberCidNumber,
    required String signerAccountId,
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
      'signer_account_id': signerAccountId,
      if (membershipLevel != null) 'membership_level': membershipLevel,
    };
    try {
      await _storeLocalProof(subscriberCidNumber, proof);
    } on Exception {
      // 链上已 finalized；本地缓存异常不能让用户重新签名。
    }
    try {
      final session = await _session.ensureSession();
      if (session == null || session.cidNumber != subscriberCidNumber) return;
      await _api.confirmPlatformSubscription(
        session: session,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
        action: action,
        membershipLevel: membershipLevel,
      );
      await _removePendingProof(subscriberCidNumber, txHash);
    } on Exception {
      // 保留本地证明；下次打开会员页只重试 HTTP。
    }
  }

  Future<void> _retryPendingMirrorsForCurrentSession() async {
    try {
      final session = await _session.ensureSession();
      if (session == null) return;
      final subscriberCidNumber = session.cidNumber;
      final pending = await _readList(_pendingKey(subscriberCidNumber));
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
        await _removePendingProof(subscriberCidNumber, txHash);
      }
    } on Exception {
      // 保留未完成证明；链上订阅与自动续费不依赖 Cloudflare。
    }
  }

  Future<void> _storeLocalProof(
      String subscriberCidNumber, Map<String, dynamic> proof) async {
    final pending = await _readList(_pendingKey(subscriberCidNumber));
    pending.removeWhere((item) => item['tx_hash'] == proof['tx_hash']);
    pending.add(proof);
    await (await _prefs)
        .setString(_pendingKey(subscriberCidNumber), jsonEncode(pending));

    final historyKey = 'subscription_tx_history_by_cid:$subscriberCidNumber';
    final history = await _readList(historyKey);
    history.removeWhere((item) => item['tx_hash'] == proof['tx_hash']);
    history.add(proof);
    if (history.length > 50) history.removeRange(0, history.length - 50);
    await (await _prefs).setString(historyKey, jsonEncode(history));
  }

  Future<void> _removePendingProof(
    String subscriberCidNumber,
    String txHash,
  ) async {
    final pending = await _readList(_pendingKey(subscriberCidNumber));
    pending.removeWhere((item) => item['tx_hash'] == txHash);
    final prefs = await _prefs;
    if (pending.isEmpty) {
      await prefs.remove(_pendingKey(subscriberCidNumber));
    } else {
      await prefs.setString(
        _pendingKey(subscriberCidNumber),
        jsonEncode(pending),
      );
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
