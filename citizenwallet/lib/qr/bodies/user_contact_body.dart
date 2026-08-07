import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/util/account_id_text.dart';

/// k = 3 用户码(**固定码**,envelope 顶层无 id / expires_at)。
///
/// 语义 = 「这是**谁**」。全 App 唯一能写入通讯录的码,只有已注册 CID 的人才有。
///
/// body 键全部单字母(与 k=1/k=2 同风格,单字母全局注册表见 `QrKind` 文档注释):
///
/// - `c` = `cid_number` 身份主键
/// - `n` = `account_id` 账户标识(`0x` 小写 64 位 hex)
///
/// **不得携带昵称**:本机可随意改写、无任何链上或服务端约束,一旦进码就会被扫码端
/// 当成对方公开身份显示,是冒名风险。扫码端应按 `c` 从服务端拉真实公开资料。
///
/// **不得携带 SS58**:SS58 只是给人看的展示形态,机器一律用 `account_id`;
/// 展示用 SS58 由扫码端自行派生。
///
/// 与 `citizenapp/lib/qr/bodies/user_contact_body.dart` 逐字节一致。
/// CID 字符集白名单:仅 ASCII 字母数字与连字符。
///
/// 只查「非空 + 无首尾空格 + 字节长度」挡不住零宽字符:`"\u200BCN...\u200B".trim()`
/// 与原串完全相同(Dart 的 trim 不把 U+200B 当空白),纯零宽串也能通过全部检查。
/// `account_id` 因为有锚定正则天然免疫,CID 必须显式加白名单。
final RegExp _cidPattern = RegExp(r'^[A-Za-z0-9-]{1,32}$');

class UserContactBody implements QrBody {
  const UserContactBody({
    required this.cidNumber,
    required this.accountId,
  });

  /// 身份主键 CID 号(永久,换绑不变)。
  final String cidNumber;

  /// 小写 `0x` 加 64 位十六进制,即 sr25519 公钥原字节。
  final String accountId;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'c': cidNumber,
        'n': accountId,
      };

  static UserContactBody fromJson(Map<String, dynamic> data) {
    _requireExactKeys(data, const {'c', 'n'});
    final cidNumber = data['c'];
    final accountId = data['n'];
    if (cidNumber is! String || !_cidPattern.hasMatch(cidNumber)) {
      throw const FormatException(
        'user_contact.c 必须为 1 到 32 位字母数字与连字符',
      );
    }
    // 账户标识格式走全仓单源校验器,禁止各处另写正则。
    if (accountId is! String || !isAccountIdText(accountId)) {
      throw const FormatException(
        'user_contact.n 必须是小写 0x 加 64 位十六进制',
      );
    }
    return UserContactBody(cidNumber: cidNumber, accountId: accountId);
  }

  static void _requireExactKeys(
    Map<String, dynamic> data,
    Set<String> expected,
  ) {
    final actual = data.keys.toSet();
    if (actual.length != expected.length ||
        !actual.containsAll(expected) ||
        !expected.containsAll(actual)) {
      throw const FormatException('user_contact.b 字段必须严格为 c/n');
    }
  }
}
