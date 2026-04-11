import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = user_transfer(临时码)
///
/// 收款方生成展示,付款方扫码后预填转账表单。
class UserTransferBody implements QrBody {
  const UserTransferBody({
    required this.address,
    required this.name,
    required this.amount,
    required this.symbol,
    required this.memo,
    required this.bank,
  });

  final String address;
  final String name;
  final String amount;
  final String symbol;
  final String memo;
  final String bank;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'address': address,
        'name': name,
        'amount': amount,
        'symbol': symbol,
        'memo': memo,
        'bank': bank,
      };

  static UserTransferBody fromJson(Map<String, dynamic> data) {
    final address = data['address'];
    final name = data['name'];
    final amount = data['amount'];
    final symbol = data['symbol'];
    final memo = data['memo'];
    final bank = data['bank'];
    if (address is! String || address.isEmpty) {
      throw const FormatException('user_transfer.address 必填');
    }
    if (name is! String ||
        amount is! String ||
        symbol is! String ||
        memo is! String ||
        bank is! String) {
      throw const FormatException(
          'user_transfer 的 name/amount/symbol/memo/bank 必须为字符串');
    }
    return UserTransferBody(
      address: address,
      name: name,
      amount: amount,
      symbol: symbol,
      memo: memo,
      bank: bank,
    );
  }
}
