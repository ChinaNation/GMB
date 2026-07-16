/// 固定治理机构静态账户表 + 联合投票常量 + 反向查找入口。
///
///
/// - 通用类型 `InstitutionInfo` / `InstitutionAccounts` / `OrgType` + 身份编码工具
///   `institutionAccountId` 等在
///   `lib/citizen/shared/institution_info.dart`。
/// - 联合投票只使用国家储委会/省储委会/省储行三类储备治理机构。
/// - `kFixedGovernanceInstitutions` 保存不进入治理 tab 的其它固定治理机构账户。
library;

import 'package:citizenapp/citizen/shared/institution_info.dart';

part 'governance_registry.generated.dart';

/// 链上联合投票总票数。
int get jointVoteTotal => 19 + kPrcs.length + kProvincialBanks.length;

/// 链上联合投票立即通过阈值。
const int jointVotePassThreshold = 105;

/// 通过机构唯一 CID 查找内置治理机构。
InstitutionInfo? findInstitutionByCidNumber(String cidNumber) {
  for (final inst in [
    ...kNrc,
    ...kPrcs,
    ...kProvincialBanks,
    ...kFixedGovernanceInstitutions,
  ]) {
    if (inst.cidNumber == cidNumber) return inst;
  }
  return null;
}

/// 把明确属于个人多签的 32 字节执行账户包装为个人多签上下文。
///
/// 机构提案禁止调用本函数；机构只能按 `actor_cid_number` 查找，不能从
/// execution account、主账户或管理员钱包回落反推身份。
InstitutionInfo? personalMultisigFromAccountId(List<int> accountIdBytes) {
  if (accountIdBytes.length != 32) return null;
  final account = _hexEncode(accountIdBytes);
  final cidFullName = '个人多签 ${account.substring(0, 8)}';
  final cidFullNameEn = 'Personal Multisig ${account.substring(0, 8)}';
  return InstitutionInfo(
    cidFullName: cidFullName,
    cidShortName: cidFullName,
    cidFullNameEn: cidFullNameEn,
    cidShortNameEn: cidFullNameEn,
    cidNumber: 'personal-account:$account',
    orgType: OrgType.personalMultisig,
    personalAccountHex: account,
  );
}

String _hexEncode(List<int> bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
