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
/// 订阅、取消**都是上链热签 + 生物识别**：订阅签=授权按月自动扣公民币，取消签=撤销授权；
/// 按月续扣由 keeper 依此授权 `charge_due` 拉取，不逐月再签。平台档价格链上单源，
/// 客户端不传价。confirm 仅刷新 Cloudflare 镜像（best-effort，链上已是真源）。
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

  /// 订阅平台会员某档（level=freedom/democracy/spark）。
  Future<void> subscribe(
    String level, {
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final wallet = await _requireHotWallet();
    try {
      final result = await _rpc.subscribePlatform(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(hexToBytes(wallet.pubkeyHex)),
        level: level,
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
