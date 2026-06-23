/// 受限保留账户名（与 citizenchain `primitives::core_const` 单一权威源逐字对齐）。
///
/// 死规则：自定义账户名命中以下任意一项即拒绝（主/费维持默认账户语义，
/// 不可作为自定义名）。中文取值固定，禁止本地另写别名。
library;

/// 主账户（强制默认账户）。
const String kReservedNameMain = '主账户';

/// 费用账户（强制默认账户）。
const String kReservedNameFee = '费用账户';

/// 永久质押（制度专属，禁止自定义）。
const String kReservedNameStake = '永久质押';

/// 安全基金（制度专属，禁止自定义）。
const String kReservedNameSafetyFund = '安全基金';

/// 两和基金（制度专属，禁止自定义）。
const String kReservedNameHe = '两和基金';

/// 全部 5 个受限保留名。
const List<String> kReservedAccountNames = <String>[
  kReservedNameMain,
  kReservedNameFee,
  kReservedNameStake,
  kReservedNameSafetyFund,
  kReservedNameHe,
];

/// 制度专属「禁止注册」名（质押/安全/两和）。主/费不在此列（强制默认账户语义）。
///
/// 与链端 `account_derive::is_forbidden_account_name` 字节对齐：只判 3 名，
/// **不 trim**（trim 仅允许在 UI 输入层，绝不进派生/校验）。
bool isForbiddenAccountName(String name) {
  return name == kReservedNameStake ||
      name == kReservedNameSafetyFund ||
      name == kReservedNameHe;
}

/// 注册策略（非派生）：自定义名是否可注册。空/主/费/制度专属 一律拒绝。
///
/// 对齐链端 `account_derive::is_registrable_custom_name`。不 trim。
bool isRegistrableCustomName(String name) {
  return name.isNotEmpty &&
      name != kReservedNameMain &&
      name != kReservedNameFee &&
      !isForbiddenAccountName(name);
}
