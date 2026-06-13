import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 多签转账管理员钱包余额检查。
///
/// 中文注释：runtime 的交易支付扩展会向签名管理员钱包扣费，不能只检查
/// 多签资金账户余额；管理员钱包没钱时，交易会被交易池拒绝或无法出块。
class DuoqianTransferBalanceGuard {
  const DuoqianTransferBalanceGuard._();

  /// InternalVote::cast 固定按 1 元计费。
  static const double voteFeeYuan = 1.0;

  /// 检查管理员钱包是否足以支付本次交易费。
  static Future<String?> checkAdminWalletBalance({
    required WalletProfile wallet,
    required double requiredFeeYuan,
    required String actionLabel,
    ChainRpc? chainRpc,
  }) async {
    final rpc = chainRpc ?? ChainRpc();
    // 中文注释(ADR-018 卡⑤)：转账前余额守卫必须读最新 finalized 余额,旁路缓存。
    final balanceYuan =
        await rpc.fetchFinalizedBalance(wallet.pubkeyHex, forceFresh: true);
    if (balanceYuan >= requiredFeeYuan) {
      return null;
    }
    return '管理员钱包余额不足，不能$actionLabel。当前余额 '
        '${AmountFormat.format(balanceYuan, symbol: '')} 元，至少需要 '
        '${AmountFormat.format(requiredFeeYuan, symbol: '')} 元。';
  }
}
