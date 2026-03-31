/// QR 码协议常量与路由标识。
///
/// 所有 QR 码统一通过 JSON 的 `proto` 字段识别类型，
/// 本文件定义全部已知协议版本。
///
/// 三种协议：
/// - 登录协议：管理员/用户扫码登录、绑定签名验证
/// - 签名协议：冷钱包离线交易签名
/// - 用户协议：用户联系人交换、付款码、用户信息传输
class QrProtocols {
  QrProtocols._();

  /// 登录协议（登录、绑定签名验证）。
  static const String login = 'WUMIN_LOGIN_V1.0.0';

  /// 交易签名协议（冷钱包签名中继）。
  static const String sign = 'WUMIN_SIGN_V1.0.0';

  /// 用户协议（联系人、付款、用户信息传输）。
  static const String user = 'WUMIN_USER_V1.0.0';

  /// 所有已知 proto 值。
  static const Set<String> all = {
    login,
    sign,
    user,
  };
}
