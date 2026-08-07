import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';

/// k = 4 收款码(**临时码**,envelope 顶层必带 id / expires_at)。
///
/// 语义 = 「一笔收款请求」。收款方生成展示,付款方扫码后预填转账表单。
///
/// body 键全部单字母(单字母全局注册表见 `QrKind` 文档注释):
///
/// - `n` = `account_id` 收款账户(`0x` 小写 64 位 hex)
/// - `v` = 金额
/// - `t` = 币种
/// - `m` = 备注
/// - `l` = 收款方清算行 CID(离线支付提交 `offchain_submitPayment` 必需)
///
/// **不得携带收款人姓名**:本机可随意填写,付款方看到的"张三"完全由出码方控制,
/// 是冒名风险;真实身份应由付款方按链上/服务端数据自行核对。
///
/// **不得携带 SS58**:SS58 只是给人看的展示形态,机器一律用 `account_id`。
final RegExp _cidPattern = RegExp(r'^[A-Za-z0-9-]{1,32}$');

class UserTransferBody implements QrBody {
  const UserTransferBody({
    required this.accountId,
    required this.amount,
    required this.symbol,
    required this.memo,
    required this.bank,
  });

  /// 小写 `0x` 加 64 位十六进制,即 sr25519 公钥原字节。
  final String accountId;
  final String amount;
  final String symbol;
  final String memo;

  /// 收款方绑定的清算行 `cid_number`。
  final String bank;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'n': accountId,
        'v': amount,
        't': symbol,
        'm': memo,
        'l': bank,
      };

  static UserTransferBody fromJson(Map<String, dynamic> data) {
    _requireExactKeys(data, const {'n', 'v', 't', 'm', 'l'});
    final accountId = data['n'];
    final amount = data['v'];
    final symbol = data['t'];
    final memo = data['m'];
    final bank = data['l'];
    // 账户标识格式走全仓单源校验器,禁止各处另写正则。
    if (accountId is! String || !isAccountIdText(accountId)) {
      throw const FormatException(
        'user_transfer.n 必须是小写 0x 加 64 位十六进制',
      );
    }
    if (amount is! String || amount.isEmpty) {
      throw const FormatException('user_transfer.v 必填');
    }
    if (symbol is! String || symbol.isEmpty) {
      throw const FormatException('user_transfer.t 必填');
    }
    if (memo is! String) {
      throw const FormatException('user_transfer.m 必须是字符串');
    }
    // 清算行 CID 与用户 CID 同规:显式字符集白名单,挡零宽字符钓鱼。
    if (bank is! String || _cidPattern.hasMatch(bank) == false) {
      throw const FormatException(
        'user_transfer.l 必须为 1 到 32 位字母数字与连字符',
      );
    }
    return UserTransferBody(
      accountId: accountId,
      amount: amount,
      symbol: symbol,
      memo: memo,
      bank: bank,
    );
  }

  static void _requireExactKeys(
    Map<String, dynamic> data,
    Set<String> expected,
  ) {
    final actual = data.keys.toSet();
    if (actual.length != expected.length ||
        !actual.containsAll(expected) ||
        !expected.containsAll(actual)) {
      throw const FormatException('user_transfer.b 字段必须严格为 n/v/t/m/l');
    }
  }
}
