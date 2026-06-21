/// 机构账户受限注册保留名（Dart 端单一权威源）。
///
/// 取值逐字对齐链端 primitives::core_const（CANON 决策2 第 5 类）：
///   RESERVED_NAME_MAIN  = "主账户"
///   RESERVED_NAME_FEE   = "费用账户"
///   RESERVED_NAME_STAKE = "永久质押"
///   RESERVED_NAME_ANQUAN= "安全基金"
///   RESERVED_NAME_HE    = "两和基金"
///
/// - 主账户 / 费用账户：每个机构强制生成的默认账户，创建时强制路由
///   OP_MAIN / OP_FEE，不得作为自定义命名账户（"强制"而非"禁止"）。
/// - 永久质押 / 安全基金 / 两和基金：制度专属账户，普通 CID 机构禁止注册，
///   account_name 命中即拒绝（链端 ReservedAccountName）。
class ReservedAccountNames {
  const ReservedAccountNames._();

  /// 主账户：机构强制默认账户，强制路由 OP_MAIN。
  static const String main = '主账户';

  /// 费用账户：机构强制默认账户，强制路由 OP_FEE。
  static const String fee = '费用账户';

  /// 永久质押：制度专属账户，禁止作为自定义账户名注册。
  static const String stake = '永久质押';

  /// 安全基金：制度专属账户，禁止作为自定义账户名注册。
  static const String anquan = '安全基金';

  /// 两和基金：制度专属账户，禁止作为自定义账户名注册。
  static const String he = '两和基金';

  /// 全部 5 个受限保留名，供遍历校验。
  static const List<String> all = <String>[main, fee, stake, anquan, he];

  /// account_name 是否为"禁止注册"的制度专属保留名（永久质押/安全基金/两和基金）。
  ///
  /// 与链端 core_const::is_forbidden_account_name 语义逐字一致：
  /// 主账户/费用账户不在此列（走强制默认路由，是"强制"而非"禁止"）。
  static bool isForbidden(String name) {
    return name == stake || name == anquan || name == he;
  }
}
