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
const String kReservedNameAnquan = '安全基金';

/// 两和基金（制度专属，禁止自定义）。
const String kReservedNameHe = '两和基金';

/// 全部 5 个受限保留名。
const List<String> kReservedAccountNames = <String>[
  kReservedNameMain,
  kReservedNameFee,
  kReservedNameStake,
  kReservedNameAnquan,
  kReservedNameHe,
];

/// 自定义账户名是否命中受限保留名（命中即拒）。
bool isForbiddenAccountName(String name) {
  return kReservedAccountNames.contains(name.trim());
}
