import 'package:citizenapp/qr/envelope.dart';

/// kind = user_transfer(临时码)
///
/// 收款方生成展示,付款方扫码后预填转账表单。
class UserTransferBody implements QrBody {
  const UserTransferBody({
    required this.ss58Address,
    required this.recipientName,
    required this.amount,
    required this.symbol,
    required this.memo,
    required this.bank,
  });

  final String ss58Address;
  final String recipientName;
  final String amount;
  final String symbol;
  final String memo;
  final String bank;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'ss58_address': ss58Address,
        'recipient_name': recipientName,
        'amount': amount,
        'symbol': symbol,
        'memo': memo,
        'bank': bank,
      };

  static UserTransferBody fromJson(Map<String, dynamic> data) {
    final ss58Address = data['ss58_address'];
    final recipientName = data['recipient_name'];
    final amount = data['amount'];
    final symbol = data['symbol'];
    final memo = data['memo'];
    final bank = data['bank'];
    if (ss58Address is! String || ss58Address.isEmpty) {
      throw const FormatException('user_transfer.ss58_address 必填');
    }
    if (recipientName is! String ||
        amount is! String ||
        symbol is! String ||
        memo is! String ||
        bank is! String) {
      throw const FormatException(
          'user_transfer 的 recipient_name/amount/symbol/memo/bank 必须为字符串');
    }
    return UserTransferBody(
      ss58Address: ss58Address,
      recipientName: recipientName,
      amount: amount,
      symbol: symbol,
      memo: memo,
      bank: bank,
    );
  }
}
