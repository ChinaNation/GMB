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
  })  : _rpc = rpc ?? SubscriptionRpc(),
        _wallet = walletManager ?? WalletManager(),
        _session = sessionProvider ?? SquareSessionProvider.instance,
        _api = api ?? SquareApiClient();

  final SubscriptionRpc _rpc;
  final WalletManager _wallet;
  final SquareSessionProvider _session;
  final SquareApiClient _api;

  /// 会员页只以 finalized 链状态和同区块共识时间戳决定当前档位与权益。
  Future<FinalizedSubscriptionSnapshot> fetchFinalizedState(
          String ownerAccount) =>
      _rpc.fetchSubscriptionSnapshot(subscriberAddress: ownerAccount);

  /// 订阅平台会员某档（level=freedom/democracy/spark）。
  Future<void> subscribe(
    String level,
    int expectedPriceFen, {
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final wallet = await _requireHotWallet();
    try {
      final result = await _rpc.subscribePlatform(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(hexToBytes(wallet.pubkeyHex)),
        level: level,
        expectedPriceFen: BigInt.from(expectedPriceFen),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(txHash: result.txHash, level: level);
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
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(hexToBytes(wallet.pubkeyHex)),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(txHash: result.txHash);
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
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(hexToBytes(wallet.pubkeyHex)),
        level: level,
        expectedPriceFen: BigInt.from(expectedPriceFen),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(txHash: result.txHash, level: level);
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

  /// best-effort 回执：链上已是真源，失败不阻塞（下次进页刷新会再对齐）。
  /// 带 [level]=订阅确认；缺 [level]=取消确认。
  Future<void> _confirm({required String txHash, String? level}) async {
    try {
      final session = await _session.ensureSession();
      if (session == null) return;
      await _api.confirmPlatformSubscription(
        session: session,
        txHash: txHash,
        level: level,
      );
    } on Exception {
      // 链上已成功；镜像回执失败仅忽略，进页刷新会再对齐。
    }
  }
}
