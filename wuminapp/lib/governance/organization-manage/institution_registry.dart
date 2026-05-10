/// 内置治理机构静态注册表 + 联合投票常量 + 反向查找入口。
///
/// 中文注释：
/// - 此文件由 `lib/institution/institution_data.dart` 拆分而来（2026-05-09 模块边界整改）。
/// - 通用类型 `InstitutionInfo` / `InstitutionAccounts` / `OrgType` + 身份编码工具
///   `institutionIdentityToPalletId` / `registeredDuoqianIdentity` 等已迁至
///   `lib/common/institution_info.dart`。
/// - 静态注册表仅包含国储会/省储会/省储行三类内置治理机构，机构账户与个人多签
///   不在此表中（动态从链上读取 `AdminsChange::Subjects`）。
library;

import 'package:wuminapp_mobile/common/institution_info.dart';

part 'governance_institution_registry.generated.dart';

/// 链上联合投票总票数。
int get jointVoteTotal =>
    19 + kProvincialCouncils.length + kProvincialBanks.length;

/// 链上联合投票立即通过阈值。
const int jointVotePassThreshold = 105;

/// 通过 48 字节 `SubjectId`(D 协议)反查机构信息。
/// sfidNumber 按 SubjectKind=0x01 Builtin 派生(byte[0]=0x01 + payload UTF-8 + 右填零)后与 palletIdBytes 比较。
///
/// 内置机构无匹配时尝试按"机构账户(0x05 InstitutionAccount)"/"个人多签(0x03
/// PersonalDuoqian)"的 SubjectId 协议从字节自构 `InstitutionInfo`。
InstitutionInfo? findInstitutionByPalletId(List<int> palletIdBytes,
    {int? adminSubjectOrg}) {
  if (palletIdBytes.length != 48) return null;
  for (final inst in [
    ...kNationalCouncil,
    ...kProvincialCouncils,
    ...kProvincialBanks
  ]) {
    final encoded = institutionIdentityToPalletId(inst.sfidNumber);
    if (_bytesEqual(encoded, palletIdBytes)) return inst;
  }

  if (_looksLikeAccountSubjectId(
      palletIdBytes, _subjectKindInstitutionAccount)) {
    final duoqianAddress = _hexEncode(palletIdBytes.sublist(1, 33));
    return InstitutionInfo(
      name: '机构账户 ${duoqianAddress.substring(0, 8)}',
      sfidNumber: registeredDuoqianIdentity(duoqianAddress),
      orgType: OrgType.duoqian,
      duoqianAddress: duoqianAddress,
      adminSubjectOrg: adminSubjectOrg,
    );
  }

  if (_looksLikeAccountSubjectId(palletIdBytes, _subjectKindPersonalDuoqian)) {
    final duoqianAddress = _hexEncode(palletIdBytes.sublist(1, 33));
    return InstitutionInfo(
      name: '个人多签 ${duoqianAddress.substring(0, 8)}',
      sfidNumber: 'personal:$duoqianAddress',
      orgType: OrgType.duoqian,
      duoqianAddress: duoqianAddress,
    );
  }

  return null;
}

const int _subjectKindPersonalDuoqian = 0x03;
const int _subjectKindInstitutionAccount = 0x05;

bool _bytesEqual(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

bool _looksLikeAccountSubjectId(List<int> palletIdBytes, int kind) {
  if (palletIdBytes.length != 48) return false;
  if (palletIdBytes[0] != kind) return false;
  for (var i = 33; i < 48; i++) {
    if (palletIdBytes[i] != 0) return false;
  }
  return true;
}

String _hexEncode(List<int> bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
