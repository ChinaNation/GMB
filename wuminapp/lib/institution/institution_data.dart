/// 机构数据模型与静态注册表。
///
/// sfid_number 与链上 `AdminsChange::Subjects` 存储的 subject key 一一对应，
/// 来源于 `primitives/china/china_cb.rs`（国储会+省储会）和
/// `primitives/china/china_ch.rs`（省储行）。
library;

import 'package:wuminapp_mobile/proposal/shared/proposal_models.dart';

part 'governance_institution_registry.generated.dart';

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

  /// 注册型多签机构
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
        return '注册多签机构';
      default:
        return '未知';
    }
  }
}

/// 治理机构及多签账户的制度账户集合。
///
/// 中文注释：内置治理机构没有笼统的 `duoqianAddress`；链端按主账户、费用账户、
/// 国储会安全基金账户、省储行质押账户分别建模。个人多签/注册机构账户只使用主账户。
class InstitutionAccounts {
  const InstitutionAccounts({
    required this.mainAddress,
    this.feeAddress,
    this.safetyFundAddress,
    this.stakeAddress,
  });

  /// 主账户地址 hex（32 字节，不含 0x）。
  final String mainAddress;

  /// 费用账户地址 hex；内置治理机构必有，个人/注册多签账户可为空。
  final String? feeAddress;

  /// 安全基金账户地址 hex；仅国储会存在。
  final String? safetyFundAddress;

  /// 质押账户地址 hex；仅省储行存在。
  final String? stakeAddress;
}

/// 单个机构或多签账户的结构化信息。
class InstitutionInfo {
  const InstitutionInfo({
    required this.name,
    required this.sfidNumber,
    required this.orgType,
    this.accounts,
    String? duoqianAddress,
    this.internalThresholdOverride,
  })  : assert(accounts != null || duoqianAddress != null),
        _legacyMainAddress = duoqianAddress;

  /// 显示名称。
  final String name;

  /// 链上身份标识（与 Rust 常量 `sfid_number` 完全一致）。
  /// 在查询链上存储时按 D 协议派生为 48 字节 `SubjectId`(byte[0]=0x01 Builtin + payload)。
  final String sfidNumber;

  /// 机构类型：0=NRC, 1=PRC, 2=PRB。
  final int orgType;

  /// 制度账户集合。
  ///
  /// 中文注释：治理机构使用生成的完整账户集合；个人/注册多签旧入口传入
  /// `duoqianAddress` 时会被视为 `mainAddress`。
  final InstitutionAccounts? accounts;

  final String? _legacyMainAddress;

  /// 主账户地址 hex（32 字节，不含 0x）。
  String get mainAddress => accounts?.mainAddress ?? _legacyMainAddress!;

  /// 兼容个人多签/注册机构旧调用；治理机构新代码不得再使用这个语义。
  String get duoqianAddress => mainAddress;

  /// 注册型机构的动态阈值覆盖。
  final int? internalThresholdOverride;

  /// 是否为注册型多签机构。
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
    String? duoqianAddress,
    int? internalThresholdOverride,
  }) {
    return InstitutionInfo(
      name: name ?? this.name,
      sfidNumber: sfidNumber ?? this.sfidNumber,
      orgType: orgType ?? this.orgType,
      accounts: accounts ?? this.accounts,
      duoqianAddress: duoqianAddress ?? _legacyMainAddress,
      internalThresholdOverride:
          internalThresholdOverride ?? this.internalThresholdOverride,
    );
  }
}

/// 链上联合投票总票数。
int get jointVoteTotal =>
    19 + kProvincialCouncils.length + kProvincialBanks.length;

/// 链上联合投票立即通过阈值。
const int jointVotePassThreshold = 105;

const String _registeredDuoqianPrefix = 'duoqian:';
const String _personalDuoqianPrefix = 'personal:';
const int _subjectKindBuiltin = 0x01;
const int _subjectKindPersonalDuoqian = 0x03;
const int _subjectKindInstitutionAccount = 0x05;

bool isRegisteredDuoqianIdentity(String institutionIdentity) {
  return institutionIdentity.startsWith(_registeredDuoqianPrefix);
}

String registeredDuoqianIdentity(String duoqianAddress) {
  return '$_registeredDuoqianPrefix${_normalizeHex(duoqianAddress)}';
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

List<int> institutionIdentityToPalletId(String institutionIdentity) {
  final duoqianAddress =
      registeredDuoqianAddressFromIdentity(institutionIdentity);
  if (duoqianAddress != null) {
    return _accountToSubjectId(_subjectKindInstitutionAccount, duoqianAddress);
  }
  final personalAddress =
      personalDuoqianAddressFromIdentity(institutionIdentity);
  if (personalAddress != null) {
    return _accountToSubjectId(_subjectKindPersonalDuoqian, personalAddress);
  }
  return _sfidNumberToFixed48(institutionIdentity);
}

/// 通过 48 字节 `SubjectId`(D 协议)反查机构信息。
/// sfidNumber 按 SubjectKind=0x01 Builtin 派生(byte[0]=0x01 + payload UTF-8 + 右填零)后与 palletIdBytes 比较。
InstitutionInfo? findInstitutionByPalletId(List<int> palletIdBytes) {
  if (palletIdBytes.length != 48) return null;
  for (final inst in [
    ...kNationalCouncil,
    ...kProvincialCouncils,
    ...kProvincialBanks
  ]) {
    final encoded = institutionIdentityToPalletId(inst.sfidNumber);
    if (_bytesEqual(encoded, palletIdBytes)) return inst;
  }

  if (_looksLikeRegisteredInstitutionId(palletIdBytes)) {
    final duoqianAddress = _hexEncode(palletIdBytes.sublist(1, 33));
    return InstitutionInfo(
      name: '注册多签机构 ${duoqianAddress.substring(0, 8)}',
      sfidNumber: registeredDuoqianIdentity(duoqianAddress),
      orgType: OrgType.duoqian,
      duoqianAddress: duoqianAddress,
    );
  }

  if (_looksLikePersonalDuoqianId(palletIdBytes)) {
    final duoqianAddress = _hexEncode(palletIdBytes.sublist(1, 33));
    return InstitutionInfo(
      name: '个人多签 ${duoqianAddress.substring(0, 8)}',
      sfidNumber: '$_personalDuoqianPrefix$duoqianAddress',
      orgType: OrgType.duoqian,
      duoqianAddress: duoqianAddress,
    );
  }

  return null;
}

List<int> _sfidNumberToFixed48(String sfidNumber) {
  final utf8Bytes = sfidNumber.codeUnits;
  final result = List<int>.filled(48, 0);
  result[0] = _subjectKindBuiltin;
  for (var i = 0; i < utf8Bytes.length && i < 47; i++) {
    result[i + 1] = utf8Bytes[i];
  }
  return result;
}

bool _bytesEqual(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

bool _looksLikeRegisteredInstitutionId(List<int> palletIdBytes) {
  return _looksLikeAccountSubjectId(
      palletIdBytes, _subjectKindInstitutionAccount);
}

bool _looksLikePersonalDuoqianId(List<int> palletIdBytes) {
  return _looksLikeAccountSubjectId(palletIdBytes, _subjectKindPersonalDuoqian);
}

bool _looksLikeAccountSubjectId(List<int> palletIdBytes, int kind) {
  if (palletIdBytes.length != 48) return false;
  if (palletIdBytes[0] != kind) return false;
  for (var i = 33; i < 48; i++) {
    if (palletIdBytes[i] != 0) return false;
  }
  return true;
}

List<int> _accountToSubjectId(int kind, String accountHex) {
  final account = _hexDecode(accountHex);
  if (account.length != 32) {
    throw ArgumentError('account hex 必须为 32 字节');
  }
  final result = List<int>.filled(48, 0);
  result[0] = kind;
  result.setAll(1, account);
  return result;
}

List<int> _hexDecode(String hex) {
  final clean = _normalizeHex(hex);
  return List<int>.generate(
    clean.length ~/ 2,
    (index) => int.parse(clean.substring(index * 2, index * 2 + 2), radix: 16),
    growable: false,
  );
}

String _hexEncode(List<int> bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}

String _normalizeHex(String hex) {
  final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
  return clean.toLowerCase();
}

// 中文注释：内置治理机构静态账户列表由 governance_institution_registry.generated.dart 生成；
// 管理员和阈值不得写入静态表，必须动态读取链上 AdminsChange::Subjects。
