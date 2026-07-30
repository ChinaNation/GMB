import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareSession;
import 'package:citizenapp/my/creator/creator_api.dart';
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

class CreatorSubscribeException implements Exception {
  const CreatorSubscribeException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 订阅者侧编排：在他人主页订阅 / 取消订阅创作者会员。
///
/// 用户只为订阅、取消和换档签名；首次扣款、真实公历到期时间与后续自动扣款由
/// runtime 根据共识时间戳完成，CitizenApp 不提交续费或周期确认。
class CreatorSubscribeService {
  CreatorSubscribeService({
    SubscriptionRpc? rpc,
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    CreatorApi? api,
    SharedPreferences? preferences,
  })  : _rpc = rpc ?? SubscriptionRpc(),
        _wallet = walletManager ?? WalletManager(),
        _session = sessionProvider ?? SquareSessionProvider.instance,
        _api = api ?? CreatorApiHttp(),
        _preferences = preferences;

  final SubscriptionRpc _rpc;
  final WalletManager _wallet;
  final SquareSessionProvider _session;
  final CreatorApi _api;
  final SharedPreferences? _preferences;

  Future<FinalizedSubscriptionSnapshot> fetchFinalizedState({
    required String subscriberCidNumber,
    required String creatorCidNumber,
  }) async {
    await _retryPendingMirrors();
    return _rpc.fetchSubscriptionSnapshot(
      subscriberCidNumber: subscriberCidNumber,
      creatorCidNumber: creatorCidNumber,
    );
  }

  Future<List<ChainCreatorTier>> fetchCreatorPlans(String creatorCidNumber) =>
      _rpc.fetchCreatorPlans(creatorCidNumber);

  /// 读某 CID 的平台会员 finalized 快照（不传创作者主体即平台 IssuerKey）。
  /// 供他人主页订阅按钮判定被查看创作者本人平台会员是否仍有效（订阅按钮门禁）。
  Future<FinalizedSubscriptionSnapshot> fetchPlatformSnapshot(
          String cidNumber) =>
      _rpc.fetchSubscriptionSnapshot(subscriberCidNumber: cidNumber);

  /// 订阅创作者某档某周期（priceFen=该档该周期价，分）。
  Future<void> subscribe({
    required String creatorCidNumber,
    required String tierId,
    required String period,
    required int priceFen,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    await _requireHotWallet();
    final identity = await _requireIdentity();
    final session = await _requireCurrentSession(identity.accountId);
    if (session.cidNumber == creatorCidNumber) {
      throw const CreatorSubscribeException('不能订阅自己');
    }
    try {
      final result = await _rpc.subscribeCreator(
        fromSs58Address: identity.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(identity.accountId)),
        creatorCidNumber: creatorCidNumber,
        tierId: tierId,
        billingPeriod: period,
        expectedPriceFen: BigInt.from(priceFen),
        sign: (payload) =>
            _wallet.signForAccountId(identity.accountId, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        subscriberCidNumber: session.cidNumber,
        signerAccountId: identity.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        action: 'subscribe',
        creatorCidNumber: creatorCidNumber,
        tierId: tierId,
        billingPeriod: period,
      );
    } on SecureSeedException catch (e) {
      throw CreatorSubscribeException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw CreatorSubscribeException(e.message);
    } on Exception catch (e) {
      throw CreatorSubscribeException('订阅失败：$e');
    }
  }

  /// 取消对某创作者的订阅（撤销按月扣款授权）。
  Future<void> cancel({
    required String creatorCidNumber,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    await _requireHotWallet();
    final identity = await _requireIdentity();
    final session = await _requireCurrentSession(identity.accountId);
    try {
      final result = await _rpc.cancelCreator(
        fromSs58Address: identity.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(identity.accountId)),
        creatorCidNumber: creatorCidNumber,
        sign: (payload) =>
            _wallet.signForAccountId(identity.accountId, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        subscriberCidNumber: session.cidNumber,
        signerAccountId: identity.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        action: 'cancel',
        creatorCidNumber: creatorCidNumber,
      );
    } on SecureSeedException catch (e) {
      throw CreatorSubscribeException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw CreatorSubscribeException(e.message);
    } on Exception catch (e) {
      throw CreatorSubscribeException('取消失败：$e');
    }
  }

  /// 更换创作者档位或周期；同一换档业务只提交这一笔账户签名交易。
  Future<void> changePlan({
    required String creatorCidNumber,
    required String tierId,
    required String period,
    required int priceFen,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    await _requireHotWallet();
    final identity = await _requireIdentity();
    final session = await _requireCurrentSession(identity.accountId);
    if (session.cidNumber == creatorCidNumber) {
      throw const CreatorSubscribeException('不能订阅自己');
    }
    try {
      final result = await _rpc.changeCreatorPlan(
        fromSs58Address: identity.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(identity.accountId)),
        creatorCidNumber: creatorCidNumber,
        tierId: tierId,
        billingPeriod: period,
        expectedPriceFen: BigInt.from(priceFen),
        sign: (payload) =>
            _wallet.signForAccountId(identity.accountId, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        subscriberCidNumber: session.cidNumber,
        signerAccountId: identity.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        action: 'change',
        creatorCidNumber: creatorCidNumber,
        tierId: tierId,
        billingPeriod: period,
      );
    } on SecureSeedException catch (e) {
      throw CreatorSubscribeException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw CreatorSubscribeException(e.message);
    } on Exception catch (e) {
      throw CreatorSubscribeException('更换订阅失败：$e');
    }
  }

  Future<WalletProfile> _requireHotWallet() async {
    final wallet = await _wallet.getDefaultWallet();
    if (wallet == null || !wallet.isHotWallet) {
      throw const CreatorSubscribeException('请先在「我的 → 我的钱包」创建热钱包');
    }
    return wallet;
  }

  /// 身份账户（CID 绑定账户，单源 [IdentityAccountCache]）：创作者订阅交易的唯一签名者。
  /// 待提交证明归属永久 CID，账户只记录当时的签名与付款事实；自订阅拦截也按
  /// 不可换绑的 CID 比对。
  Future<ResolvedIdentity> _requireIdentity() async {
    final identity = await IdentityAccountCache.instance.resolve();
    if (identity == null || !identity.isRegistered) {
      throw const CreatorSubscribeException('请先注册并绑定公民 CID');
    }
    return identity;
  }

  Future<SquareSession> _requireCurrentSession(String accountId) async {
    final session = await _session.ensureSession();
    if (session == null || session.accountId != accountId) {
      throw const CreatorSubscribeException('当前会话与默认热钱包不一致，请重新登录');
    }
    return session;
  }

  Future<SharedPreferences> get _prefs async {
    final preferences = _preferences;
    if (preferences != null) return preferences;
    return SharedPreferences.getInstance();
  }

  String _pendingKey(String subscriberCidNumber) =>
      'creator_subscription_mirror_pending_by_cid:$subscriberCidNumber';

  /// finalized 回执按订阅者永久 CID 持久化；签名账户只作为交易事实保留。
  /// HTTP 失败只重放同一交易证明，不要求第二次签名。
  Future<void> _confirm({
    required String subscriberCidNumber,
    required String signerAccountId,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
    required String action,
    required String creatorCidNumber,
    String? tierId,
    String? billingPeriod,
  }) async {
    final proof = <String, dynamic>{
      'tx_hash': txHash,
      'block_hash': blockHashHex,
      'signed_extrinsic_hex': signedExtrinsicHex,
      'action': action,
      'creator_cid_number': creatorCidNumber,
      'signer_account_id': signerAccountId,
      if (tierId != null) 'tier_id': tierId,
      if (billingPeriod != null) 'billing_period': billingPeriod,
    };
    try {
      await _storeLocalProof(subscriberCidNumber, proof);
    } on Exception {
      // 链上已 finalized；本地缓存异常不得转化为重新签名。
    }
    try {
      final session = await _session.ensureSession();
      if (session == null || session.cidNumber != subscriberCidNumber) return;
      await _api.confirmCreatorSubscription(
        session: session,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
        action: action,
        creatorCidNumber: creatorCidNumber,
        tierId: tierId,
        billingPeriod: billingPeriod,
      );
      await _removePendingProof(subscriberCidNumber, txHash);
    } on Exception {
      // 保留证明，下次进入创作者订阅页仅重试 HTTP。
    }
  }

  Future<void> _retryPendingMirrors() async {
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
        final creatorCidNumber = proof['creator_cid_number'];
        if (txHash is! String ||
            blockHashHex is! String ||
            signedExtrinsicHex is! String ||
            action is! String ||
            creatorCidNumber is! String) {
          continue;
        }
        await _api.confirmCreatorSubscription(
          session: session,
          txHash: txHash,
          blockHashHex: blockHashHex,
          signedExtrinsicHex: signedExtrinsicHex,
          action: action,
          creatorCidNumber: creatorCidNumber,
          tierId: proof['tier_id'] as String?,
          billingPeriod: proof['billing_period'] as String?,
        );
        await _removePendingProof(subscriberCidNumber, txHash);
      }
    } on Exception {
      // Cloudflare 不可用不影响链上自动续费，证明继续保留。
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
