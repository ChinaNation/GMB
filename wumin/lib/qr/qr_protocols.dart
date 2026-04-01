/// QR 码协议常量。
///
/// 所有 QR 码统一通过 JSON 的 `proto` 字段识别类型。
class QrProtocols {
  QrProtocols._();

  /// 登录挑战/回执协议。
  static const String login = 'WUMIN_LOGIN_V1.0.0';

  /// 交易签名协议（冷钱包签名中继）。
  static const String sign = 'WUMIN_SIGN_V1.0.0';

  /// 用户名片协议（展示地址/公钥，供扫码转账或管理员绑定）。
  static const String user = 'WUMIN_USER_V1.0.0';
}
