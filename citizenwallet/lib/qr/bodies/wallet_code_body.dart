import 'package:citizenwallet/qr/envelope.dart';

/// kind = wallet_code(**固定码**,envelope 顶层无 id / expires_at)
///
/// 钱包码只声明「这是哪个账户」,body 只有 `account_id` 一个字段。
///
/// 不得携带钱包账户名、公开昵称、CID、SS58 或任何时效字段:本机钱包标签用户可随意
/// 改写、无任何链上或服务端约束,一旦进入二维码就会被扫码端当成对方公开身份显示。
/// 公民钱包完全离线、无 NTP,更不得签发带绝对时间戳的凭证。
///
/// 与 citizenapp/lib/qr/bodies/wallet_code_body.dart 逐字节一致。
class WalletCodeBody implements QrBody {
  const WalletCodeBody({required this.accountId});

  static final RegExp _accountIdPattern = RegExp(r'^0x[0-9a-f]{64}$');

  /// 小写 `0x` 加 64 位十六进制,即 sr25519 公钥原字节。
  final String accountId;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'account_id': accountId,
      };

  static WalletCodeBody fromJson(Map<String, dynamic> data) {
    _requireExactKeys(data, const {'account_id'});
    final accountId = data['account_id'];
    if (accountId is! String || !_accountIdPattern.hasMatch(accountId)) {
      throw const FormatException(
        'wallet_code.account_id 必须是小写 0x 加 64 位十六进制',
      );
    }
    return WalletCodeBody(accountId: accountId);
  }

  static void _requireExactKeys(
    Map<String, dynamic> data,
    Set<String> expected,
  ) {
    final actual = data.keys.toSet();
    if (actual.length != expected.length ||
        !actual.containsAll(expected) ||
        !expected.containsAll(actual)) {
      throw const FormatException('wallet_code.b 字段必须严格为 account_id');
    }
  }
}
