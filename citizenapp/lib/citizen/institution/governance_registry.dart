/// 固定治理机构静态账户表 + 联合投票常量 + 反向查找入口。
///
///
/// - 通用类型 `InstitutionInfo` / `InstitutionAccounts` / `OrgType` + 身份编码工具
///   `institutionIdentityToAccountId` / `registeredAccountIdentity` 等在
///   `lib/citizen/shared/institution_info.dart`。
/// - 联合投票只使用国家储委会/省储委会/省储行三类储备治理机构。
/// - `kFixedGovernanceInstitutions` 保存不进入治理 tab 的其它固定治理机构账户。
library;

import 'package:citizenapp/citizen/shared/institution_info.dart';

part 'governance_registry.generated.dart';

/// 链上联合投票总票数。
int get jointVoteTotal =>
    19 + kProvincialCouncils.length + kProvincialBanks.length;

/// 链上联合投票立即通过阈值。
const int jointVotePassThreshold = 105;

/// 通过 32 字节治理 AccountId 反查机构信息。
InstitutionInfo? findInstitutionByAccountId(List<int> accountIdBytes,
    {String? adminAccountCode}) {
  if (accountIdBytes.length != 32) return null;
  for (final inst in [
    ...kNationalCouncil,
    ...kProvincialCouncils,
    ...kProvincialBanks,
    ...kFixedGovernanceInstitutions,
  ]) {
    final encoded = institutionIdentityToAccountId(
      inst.cidNumber,
      mainAccount: inst.mainAccount,
    );
    if (_bytesEqual(encoded, accountIdBytes)) return inst;
  }

  final account = _hexEncode(accountIdBytes);
  if (adminAccountCode != null && adminAccountCode.isNotEmpty) {
    final cidFullName = '机构账户 ${account.substring(0, 8)}';
    final cidFullNameEn = 'Institution Account ${account.substring(0, 8)}';
    return InstitutionInfo(
      cidFullName: cidFullName,
      cidShortName: cidFullName,
      cidFullNameEn: cidFullNameEn,
      cidShortNameEn: cidFullNameEn,
      cidNumber: registeredAccountIdentity(account),
      orgType: OrgType.account,
      account: account,
      adminAccountCode: adminAccountCode,
    );
  }
  final cidFullName = '个人多签 ${account.substring(0, 8)}';
  final cidFullNameEn = 'Personal Multisig ${account.substring(0, 8)}';
  return InstitutionInfo(
    cidFullName: cidFullName,
    cidShortName: cidFullName,
    cidFullNameEn: cidFullNameEn,
    cidShortNameEn: cidFullNameEn,
    cidNumber: 'personal-account:$account',
    orgType: OrgType.account,
    account: account,
  );
}

bool _bytesEqual(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

String _hexEncode(List<int> bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
