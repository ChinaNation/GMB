import 'package:citizenapp/my/util/amount_format.dart';

/// 多签创建金额规则。
///
/// 本类的创建金额函数只服务个人多签：签名者账户承担初始资金和
/// 创建手续费并保留 ED。机构交易的本金账户和费用账户必须分开校验，
/// 不得复用此合并公式。
class MultisigCreateAmountRules {
  MultisigCreateAmountRules._();

  static final BigInt existentialDepositFen = BigInt.from(111);
  static final BigInt minOnchainFeeFen = BigInt.from(10);

  static const int _perbillParts = 1000000;
  static const int _perbillDenominator = 1000000000;

  /// 创建多签复用链上 onchain_transaction 手续费公式：
  /// `max(amount_fen * 0.1%, 10 fen)`，half-up 到分。
  static BigInt calculateOnchainFeeFen(BigInt amountFen) {
    final byRate = (amountFen * BigInt.from(_perbillParts) +
            BigInt.from(_perbillDenominator ~/ 2)) ~/
        BigInt.from(_perbillDenominator);
    return byRate < minOnchainFeeFen ? minOnchainFeeFen : byRate;
  }

  static BigInt requiredBalanceFen(BigInt initialAmountFen) {
    return initialAmountFen +
        calculateOnchainFeeFen(initialAmountFen) +
        existentialDepositFen;
  }

  static BigInt yuanToFen(double yuan) {
    return BigInt.from((yuan * 100).round());
  }

  static double fenToYuan(BigInt fen) => fen.toDouble() / 100.0;

  static String insufficientBalanceMessage({
    required String actionLabel,
    required double balanceYuan,
    required BigInt initialAmountFen,
  }) {
    final feeFen = calculateOnchainFeeFen(initialAmountFen);
    final requiredFen = initialAmountFen + feeFen + existentialDepositFen;
    return '发起钱包余额不足，不能$actionLabel。当前余额 '
        '${AmountFormat.format(balanceYuan, symbol: '')} 元，至少需要 '
        '${AmountFormat.format(fenToYuan(requiredFen), symbol: '')} 元'
        '（初始资金 ${AmountFormat.format(fenToYuan(initialAmountFen), symbol: '')} 元'
        ' + 创建手续费 ${AmountFormat.format(fenToYuan(feeFen), symbol: '')} 元'
        ' + ED ${AmountFormat.format(fenToYuan(existentialDepositFen), symbol: '')} 元）。';
  }
}
