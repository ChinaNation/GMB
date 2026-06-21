/// 跨模块共用：机构/多签账户的数据载体类型 + 身份编码工具。
///
/// 中文注释：
/// - 此文件由 `lib/institution/institution_data.dart` 拆分而来（2026-05-09 模块边界整改）。
/// - 内置治理机构静态注册表（`kNationalCouncil`/`kProvincialCouncils`/`kProvincialBanks`）+
///   `findInstitutionByAccountId()`/`jointVoteTotal`/`jointVotePassThreshold` 已迁至
///   `lib/organization-manage/institution_registry.dart`。
/// - 治理主体统一为机构多签 AccountId；sfid_number 只用于查找机构资料。
library;

import 'package:citizenapp/governance/shared/proposal/proposal_models.dart';

/// 提案展示号格式化(双层 ID v1):`2026000123` 风格。
///
/// 主键 `proposal_id` 是全局单调 u64,与展示无关。展示号由链上
/// `ProposalDisplayId[id] = ProposalDisplayMeta { year, seq_in_year }`
/// 反查得到,本函数把它拼成纯数字字符串(年份 + 6 位补零序号)。
///
/// 当 `seq_in_year` 突破 6 位(>=1_000_000)时自动扩展到 7、8 位等,
/// 不会截断。
String formatProposalId(ProposalDisplayMeta? meta) {
  if (meta == null) return '—';
  final seq = meta.seqInYear.toString().padLeft(6, '0');
  return '${meta.year}$seq';
}

class OrgType {
  OrgType._();

  /// 国储会 National Reserve Committee
  static const int nrc = 0;

  /// 省储会 Provincial Reserve Committee
  static const int prc = 1;

  /// 省储行 Provincial Reserve Bank
  static const int prb = 2;

  /// 多签账户。具体是个人多签还是机构账户，由 admins-change 的 account identity 区分。
  static const int duoqian = 3;

  static String label(int orgType) {
    switch (orgType) {
      case nrc:
        return '国储会';
      case prc:
        return '省储会';
      case prb:
        return '省储行';
      case duoqian:
        return '多签账户';
      default:
        return '未知';
    }
  }
}

/// 治理机构及多签账户的制度账户集合。
///
/// 中文注释：内置治理机构没有笼统的 `duoqianAccount`；链端按主账户、费用账户、
/// 国储会安全基金账户、省储行永久质押账户分别建模。个人多签/机构账户只使用主账户。
class InstitutionAccounts {
  const InstitutionAccounts({
    required this.mainAccount,
    this.feeAccount,
    this.anquanAccount,
    this.heAccount,
    this.stakeAccount,
  });

  /// 主账户 AccountId hex（32 字节，不含 0x）。
  final String mainAccount;

  /// 费用账户 AccountId hex；内置治理机构必有，个人/注册多签账户可为空。
  final String? feeAccount;

  /// 安全基金账户 AccountId hex；仅国储会存在。
  final String? anquanAccount;

  /// 两和基金账户地址 hex（Reconciliation Fund，链端 NRC_HE_ACCOUNT）；仅国储会存在。
  final String? heAccount;

  /// 永久质押账户 AccountId hex；仅省储行存在。
  final String? stakeAccount;
}

/// 单个机构或多签账户的结构化信息。
class InstitutionInfo {
  const InstitutionInfo({
    required this.name,
    required this.sfidNumber,
    required this.orgType,
    this.accounts,
    String? duoqianAccount,
    this.adminAccountOrg,
    this.internalThresholdOverride,
  })  : assert(accounts != null || duoqianAccount != null),
        _singleMainAccount = duoqianAccount;

  /// 显示名称。
  final String name;

  /// 链上身份标识（与 Rust 常量 `sfid_number` 完全一致）。
  /// 查询治理 storage 时使用 `mainAccount` 这个 AccountId，不再从 sfid_number 派生主体。
  final String sfidNumber;

  /// 机构类型：0=NRC, 1=PRC, 2=PRB。
  final int orgType;

  /// 注册机构账户管理员更换使用的 org：4=公权机构账户，5=其他机构账户。
  final int? adminAccountOrg;

  /// 制度账户集合。
  ///
  /// 中文注释：治理机构使用生成的完整账户集合；个人多签/机构账户使用
  /// 主账户 AccountId作为多签账户地址。
  final InstitutionAccounts? accounts;

  final String? _singleMainAccount;

  /// 主账户 AccountId hex（32 字节，不含 0x）。
  String get mainAccount => accounts?.mainAccount ?? _singleMainAccount!;

  /// 个人多签/注册机构账户的多签账户；内置治理机构不得使用这个语义。
  String get duoqianAccount => mainAccount;

  /// 机构账户的动态阈值覆盖。
  final int? internalThresholdOverride;

  /// 是否为链上注册的机构账户。
  bool get isRegisteredDuoqian =>
      orgType == OrgType.duoqian && isRegisteredDuoqianIdentity(sfidNumber);

  /// 内部投票通过阈值。
  int get internalThreshold {
    if (internalThresholdOverride != null) return internalThresholdOverride!;
    switch (orgType) {
      case OrgType.nrc:
        return 13;
      case OrgType.prc:
        return 6;
      case OrgType.prb:
        return 6;
      case OrgType.duoqian:
        return 0;
      default:
        return 0;
    }
  }

  /// 联合投票中的机构权重。
  int get jointVoteWeight {
    switch (orgType) {
      case OrgType.nrc:
        return 19;
      case OrgType.prc:
      case OrgType.prb:
        return 1;
      default:
        return 0;
    }
  }

  InstitutionInfo copyWith({
    String? name,
    String? sfidNumber,
    int? orgType,
    InstitutionAccounts? accounts,
    String? duoqianAccount,
    int? adminAccountOrg,
    int? internalThresholdOverride,
  }) {
    return InstitutionInfo(
      name: name ?? this.name,
      sfidNumber: sfidNumber ?? this.sfidNumber,
      orgType: orgType ?? this.orgType,
      accounts: accounts ?? this.accounts,
      duoqianAccount: duoqianAccount ?? _singleMainAccount,
      adminAccountOrg: adminAccountOrg ?? this.adminAccountOrg,
      internalThresholdOverride:
          internalThresholdOverride ?? this.internalThresholdOverride,
    );
  }
}

const String _registeredDuoqianPrefix = 'duoqian:';
const String _personalDuoqianPrefix = 'personal:';

bool isRegisteredDuoqianIdentity(String institutionIdentity) {
  return institutionIdentity.startsWith(_registeredDuoqianPrefix);
}

String registeredDuoqianIdentity(String duoqianAccount) {
  return '$_registeredDuoqianPrefix${_normalizeHex(duoqianAccount)}';
}

String? registeredDuoqianAddressFromIdentity(String institutionIdentity) {
  if (!isRegisteredDuoqianIdentity(institutionIdentity)) return null;
  final hex = _normalizeHex(
    institutionIdentity.substring(_registeredDuoqianPrefix.length),
  );
  if (hex.length != 64) return null;
  return hex;
}

bool isPersonalDuoqianIdentity(String institutionIdentity) {
  return institutionIdentity.startsWith(_personalDuoqianPrefix);
}

String? personalDuoqianAddressFromIdentity(String institutionIdentity) {
  if (!isPersonalDuoqianIdentity(institutionIdentity)) return null;
  final hex = _normalizeHex(
    institutionIdentity.substring(_personalDuoqianPrefix.length),
  );
  if (hex.length != 64) return null;
  return hex;
}

List<int> institutionIdentityToAccountId(
  String institutionIdentity, {
  String? mainAccount,
}) {
  final duoqianAccount =
      registeredDuoqianAddressFromIdentity(institutionIdentity);
  if (duoqianAccount != null) {
    return _accountHexToBytes(duoqianAccount);
  }
  final personalAddress =
      personalDuoqianAddressFromIdentity(institutionIdentity);
  if (personalAddress != null) {
    return _accountHexToBytes(personalAddress);
  }
  if (mainAccount == null) {
    throw ArgumentError('内置治理机构必须提供 mainAccount 作为治理 AccountId');
  }
  return _accountHexToBytes(mainAccount);
}

List<int> _accountHexToBytes(String accountHex) {
  final account = _hexDecode(accountHex);
  if (account.length != 32) {
    throw ArgumentError('account hex 必须为 32 字节');
  }
  return account;
}

List<int> _hexDecode(String hex) {
  final clean = _normalizeHex(hex);
  return List<int>.generate(
    clean.length ~/ 2,
    (index) => int.parse(clean.substring(index * 2, index * 2 + 2), radix: 16),
    growable: false,
  );
}

String _normalizeHex(String hex) {
  final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
  return clean.toLowerCase();
}
