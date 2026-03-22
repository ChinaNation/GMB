/// QR 码协议常量与路由标识。
///
/// 所有 QR 码统一通过 JSON 的 `proto` 字段识别类型，
/// 本文件定义全部已知协议版本。
class QrProtocols {
  QrProtocols._();

  /// 登录挑战/回执协议。
  static const String login = 'WUMIN_LOGIN_V1.0.0';

  /// 收款码协议（转账预填）。
  static const String transfer = 'WUMINAPP_TRANSFER_V1';

  /// 用户码协议（通讯录交换）。
  static const String contact = 'WUMINAPP_CONTACT_V1';

  /// 交易签名协议（冷钱包签名中继）。
  static const String sign = 'WUMIN_SIGN_V1.0.0';

  /// 旧版用户码协议（向后兼容解析）。
  static const String legacyUserCard = 'WUMINAPP_USER_CARD_V1';

  /// 所有已知 proto 值。
  static const Set<String> all = {
    login,
    transfer,
    contact,
    sign,
    legacyUserCard,
  };
}
