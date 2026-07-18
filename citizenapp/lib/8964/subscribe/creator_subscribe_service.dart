import 'dart:typed_data';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/rpc/chain_rpc.dart' show TxPoolWatchCallback;
import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show hexToBytes;
import 'package:citizenapp/wallet/core/secure_seed_store.dart'
    show SecureSeedException;
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class CreatorSubscribeException implements Exception {
  const CreatorSubscribeException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 订阅者侧编排：在他人主页订阅 / 取消订阅创作者会员。
///
/// 订阅、取消**都是上链热签 + 生物识别**：订阅签=授权按月自动扣款，取消签=撤销授权；
/// 按月续扣由 keeper 依此授权 `charge_due` 拉取，不逐月再签。confirm 仅刷新 Cloudflare 镜像。
class CreatorSubscribeService {
  CreatorSubscribeService({
    SubscriptionRpc? rpc,
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    CreatorApi? api,
  })  : _rpc = rpc ?? SubscriptionRpc(),
        _wallet = walletManager ?? WalletManager(),
        _session = sessionProvider ?? SquareSessionProvider.instance,
        _api = api ?? CreatorApiHttp();

  final SubscriptionRpc _rpc;
  final WalletManager _wallet;
  final SquareSessionProvider _session;
  final CreatorApi _api;

  /// 订阅创作者某档某周期（priceFen=该档该周期价，分）。
  Future<void> subscribe({
    required String creatorAddress,
    required String tierId,
    required String period,
    required int priceFen,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final wallet = await _requireHotWallet();
    if (wallet.address == creatorAddress) {
      throw const CreatorSubscribeException('不能订阅自己');
    }
    try {
      final result = await _rpc.subscribeCreator(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(hexToBytes(wallet.pubkeyHex)),
        creatorAddress: creatorAddress,
        priceFen: BigInt.from(priceFen),
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(
        txHash: result.txHash,
        creatorAddress: creatorAddress,
        tierId: tierId,
        period: period,
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
    required String creatorAddress,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final wallet = await _requireHotWallet();
    try {
      final result = await _rpc.cancelCreator(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(hexToBytes(wallet.pubkeyHex)),
        creatorAddress: creatorAddress,
        sign: (payload) => _wallet.signWithWallet(wallet.walletIndex, payload),
        onWatchEvent: onWatchEvent,
      );
      await _confirm(txHash: result.txHash, creatorAddress: creatorAddress);
    } on SecureSeedException catch (e) {
      throw CreatorSubscribeException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw CreatorSubscribeException(e.message);
    } on Exception catch (e) {
      throw CreatorSubscribeException('取消失败：$e');
    }
  }

  Future<WalletProfile> _requireHotWallet() async {
    final wallet = await _wallet.getDefaultWallet();
    if (wallet == null || !wallet.isHotWallet) {
      throw const CreatorSubscribeException('请先在「我的 → 我的钱包」创建热钱包');
    }
    return wallet;
  }

  /// best-effort 回执：链上已是真源，失败不阻塞（下次进页刷新会再对齐）。
  Future<void> _confirm({
    required String txHash,
    required String creatorAddress,
    String? tierId,
    String? period,
  }) async {
    try {
      final session = await _session.ensureSession();
      if (session == null) return;
      await _api.confirmCreatorSubscription(
        session: session,
        txHash: txHash,
        creatorAccount: creatorAddress,
        tierId: tierId,
        period: period,
      );
    } on Exception {
      // 链上已成功；镜像回执失败仅忽略，进页刷新会再对齐。
    }
  }
}
