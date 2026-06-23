/// 内置治理机构静态注册表 + 联合投票常量 + 反向查找入口。
///
/// 中文注释：
/// - 此文件由 `lib/institution/institution_data.dart` 拆分而来（2026-05-09 模块边界整改）。
/// - 通用类型 `InstitutionInfo` / `InstitutionAccounts` / `OrgType` + 身份编码工具
///   `institutionIdentityToAccountId` / `registeredAccountIdentity` 等已迁至
///   `lib/governance/shared/institution_info.dart`。
/// - 静态注册表仅包含国储会/省储会/省储行三类内置治理机构，机构账户与个人多签
///   不在此表中（动态从链上读取 `AdminsChange::AdminAccounts`）。
library;

import 'package:citizenapp/governance/shared/institution_info.dart';

part 'governance_institution_registry.generated.dart';

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
    ...kProvincialBanks
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
    return InstitutionInfo(
      cidFullName: cidFullName,
      cidShortName: cidFullName,
      cidNumber: registeredAccountIdentity(account),
      orgType: OrgType.account,
      account: account,
      adminAccountCode: adminAccountCode,
    );
  }
  final cidFullName = '个人多签 ${account.substring(0, 8)}';
  return InstitutionInfo(
    cidFullName: cidFullName,
    cidShortName: cidFullName,
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
