import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareSession;
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/my/creator/models/creator_overview.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show hexToBytes;
import 'package:citizenapp/wallet/core/secure_seed_store.dart'
    show SecureSeedException;
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 创作者页展示态：无可用热钱包会话 / 已开通（含计划与概览）。

class CreatorPageData {
  const CreatorPageData._({required this.gated, this.plan, this.overview});

  /// true = 无会话或没有当前有效的平台会员权益。
  final bool gated;
  final CreatorPlan? plan;
  final CreatorOverview? overview;

  factory CreatorPageData.gated() => const CreatorPageData._(gated: true);

  factory CreatorPageData.active({
    required CreatorPlan plan,
    required CreatorOverview overview,
  }) =>
      CreatorPageData._(gated: false, plan: plan, overview: overview);
}

class CreatorException implements Exception {
  const CreatorException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 创作者管理编排：当前有效的平台会员可创建档位、读取概览并收取订阅款。
///
/// - 保存档位只签一次 `set_creator_plans` 链上交易；finalized 后 Cloudflare 只保存展示字段。
class CreatorService {
  CreatorService({
    CreatorApi? api,
    SubscriptionRpc? subscriptionRpc,
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    SharedPreferences? preferences,
  })  : _api = api ?? CreatorApiHttp(),
        _subscriptionRpc = subscriptionRpc ?? SubscriptionRpc(),
        _wallet = walletManager ?? WalletManager(),
        _session = sessionProvider ?? SquareSessionProvider.instance,
        _preferences = preferences;

  final CreatorApi _api;
  final SubscriptionRpc _subscriptionRpc;
  final WalletManager _wallet;
  final SquareSessionProvider _session;
  final SharedPreferences? _preferences;

  /// 首屏加载：先按 finalized 平台订阅真态门禁，再合并链上价格与 Cloudflare 展示名。
  Future<CreatorPageData> load() async {
    final session = await _session.ensureSession();
    if (session == null) return CreatorPageData.gated();

    final membership = await _subscriptionRpc.fetchSubscriptionSnapshot(
      subscriberAccountId: session.accountId,
    );
    if (membership.state?.isEffectiveAt(membership.chainNowMs) != true) {
      return CreatorPageData.gated();
    }

    // 上一次链上已 finalized、但 Cloudflare 瞬时失败时只重试展示镜像，不再签名。
    await _retryPendingMirror(session);

    final results = await Future.wait([
      // Cloudflare 只补展示名与统计；瞬时不可用时仍允许按链上真态进入页面。
      _api.fetchMyPlan(session).catchError((_) => null),
      _api.fetchOverview(session).catchError((_) => CreatorOverview.zero),
      _subscriptionRpc.fetchCreatorPlans(session.accountId),
    ]);
    final displayPlan = results[0] as CreatorPlan?;
    final overview = results[1] as CreatorOverview;
    final chainTiers = results[2] as List<ChainCreatorTier>;
    return CreatorPageData.active(
      plan: mergeCreatorPlanWithChain(
        creatorAccountId: session.accountId,
        displayPlan: displayPlan,
        chainTiers: chainTiers,
      ),
      overview: overview,
    );
  }

  /// 覆盖式保存档位：一次链上签名 → finalized → Cloudflare 保存展示字段。
  Future<CreatorPlan> saveTiers(List<CreatorTier> tiers) async {
    if (tiers.length > CreatorPlan.maxTiers) {
      throw const CreatorException('最多 ${CreatorPlan.maxTiers} 个会员档');
    }
    final wallet = await _wallet.getDefaultWallet();
    if (wallet == null || !wallet.isHotWallet) {
      throw const CreatorException('请先在「我的 → 我的钱包」创建热钱包');
    }
    final session = await _session.ensureSession();
    if (session == null) {
      throw const CreatorException('会话不可用，请稍后重试');
    }
    if (session.accountId != wallet.accountId) {
      throw const CreatorException('当前会话与默认热钱包不一致，请重新登录');
    }
    try {
      final membership = await _subscriptionRpc.fetchSubscriptionSnapshot(
        subscriberAccountId: wallet.accountId,
      );
      if (membership.state?.isEffectiveAt(membership.chainNowMs) != true) {
        throw const CreatorException('需要当前有效的平台会员才能设置创作者会员档');
      }
      final result = await _subscriptionRpc.setCreatorPlans(
        fromSs58Address: wallet.ss58Address,
        signerPublicKey: Uint8List.fromList(hexToBytes(wallet.accountId)),
        tiers: tiers
            .map(
              (tier) => CreatorTierInput(
                tierId: tier.tierId,
                pricesFen: tier.pricesFen.entries
                    .map(
                      (entry) => CreatorPeriodPriceInput(
                        billingPeriod: entry.key.key,
                        priceFen: BigInt.from(entry.value),
                      ),
                    )
                    .toList(growable: false),
              ),
            )
            .toList(growable: false),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
      );
      return _completeFinalizedSave(
        session: session,
        accountId: wallet.accountId,
        txHash: result.txHash,
        blockHashHex: result.blockHashHex,
        signedExtrinsicHex: result.signedExtrinsicHex,
        tiers: tiers,
      );
    } on SecureSeedException catch (e) {
      // 生物识别取消 / 无锁屏等：单源文案，杜绝静默失败。
      throw CreatorException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw CreatorException(e.message);
    } on CreatorApiException catch (e) {
      throw CreatorException(e.message);
    } on Exception catch (e) {
      throw CreatorException('保存失败：$e');
    }
  }

  String _pendingMirrorKey(String accountId) =>
      'creator_plan_mirror_pending:$accountId';

  Future<SharedPreferences> get _prefs async {
    final preferences = _preferences;
    if (preferences != null) return preferences;
    return SharedPreferences.getInstance();
  }

  Future<void> _writePendingMirror({
    required String accountId,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
    required List<CreatorTier> tiers,
  }) async {
    final prefs = await _prefs;
    final saved = await prefs.setString(
      _pendingMirrorKey(accountId),
      jsonEncode({
        'tx_hash': txHash,
        'block_hash': blockHashHex,
        'signed_extrinsic_hex': signedExtrinsicHex,
        'tiers': tiers.map((tier) => tier.toJson()).toList(growable: false),
      }),
    );
    if (!saved) throw StateError('无法保存创作者展示镜像重试记录');
  }

  Future<void> _clearPendingMirror(String accountId) async {
    final prefs = await _prefs;
    await prefs.remove(_pendingMirrorKey(accountId));
  }

  Future<void> _retryPendingMirror(SquareSession session) async {
    try {
      final prefs = await _prefs;
      final raw = prefs.getString(_pendingMirrorKey(session.accountId));
      if (raw == null) return;
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return;
      final txHash = decoded['tx_hash'];
      final blockHashHex = decoded['block_hash'];
      final signedExtrinsicHex = decoded['signed_extrinsic_hex'];
      final rawTiers = decoded['tiers'];
      if (txHash is! String ||
          blockHashHex is! String ||
          signedExtrinsicHex is! String ||
          rawTiers is! List) {
        return;
      }
      final tiers = rawTiers
          .whereType<Map<String, dynamic>>()
          .map(CreatorTier.fromJson)
          .toList(growable: false);
      await _api.saveMyPlan(
        session: session,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
        tiers: tiers,
      );
      await prefs.remove(_pendingMirrorKey(session.accountId));
    } on Exception {
      // 保留待同步记录；页面仍以 finalized 链上价格为真源，不阻断创作者功能。
    }
  }

  /// 进入本方法时链上业务已经 finalized；之后任何边缘或本地缓存失败都不得要求用户重签。
  Future<CreatorPlan> _completeFinalizedSave({
    required SquareSession session,
    required String accountId,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
    required List<CreatorTier> tiers,
  }) async {
    final localPlan = CreatorPlan(
      creatorAccountId: accountId,
      tiers: tiers,
      updatedAt: 0,
    );
    try {
      await _appendLocalTransaction(
        accountId: accountId,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
      );
      await _writePendingMirror(
        accountId: accountId,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
        tiers: tiers,
      );
    } on Exception {
      // 继续立即提交 Cloudflare；链上已成功，禁止把本地缓存异常变成第二次签名。
    }

    var displayPlan = localPlan;
    try {
      displayPlan = await _api.saveMyPlan(
        session: session,
        txHash: txHash,
        blockHashHex: blockHashHex,
        signedExtrinsicHex: signedExtrinsicHex,
        tiers: tiers,
      );
      await _clearPendingMirror(accountId);
    } on Exception {
      // 保留待同步记录；下次进入创作者页只重试 HTTP。
    }

    try {
      final chainTiers = await _subscriptionRpc.fetchCreatorPlans(accountId);
      return mergeCreatorPlanWithChain(
        creatorAccountId: accountId,
        displayPlan: displayPlan,
        chainTiers: chainTiers,
      );
    } on Exception {
      return localPlan;
    }
  }

  /// 本地按钱包账户保留有限条 finalized 交易证明；Cloudflare 成功后也不删除链上交易记录。
  Future<void> _appendLocalTransaction({
    required String accountId,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
  }) async {
    final prefs = await _prefs;
    final key = 'subscription_tx_history:$accountId';
    final raw = prefs.getString(key);
    final history = <Map<String, dynamic>>[];
    if (raw != null) {
      final decoded = jsonDecode(raw);
      if (decoded is List) {
        history.addAll(decoded.whereType<Map<String, dynamic>>());
      }
    }
    history.removeWhere((item) => item['tx_hash'] == txHash);
    history.add({
      'action': 'set_creator_plans',
      'tx_hash': txHash,
      'block_hash': blockHashHex,
      'signed_extrinsic_hex': signedExtrinsicHex,
    });
    if (history.length > 50) history.removeRange(0, history.length - 50);
    await prefs.setString(key, jsonEncode(history));
  }
}
