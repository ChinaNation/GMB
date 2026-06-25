// 机构 → 子 tab 分组(ADR-028 决策 3)——按 `institution_code` 把统一机构目录
// 切成立法/治理/公权 三个机构视图的过滤谓词。单一源。
//
// 中文注释:
// - 机构码的底层分类(固定治理 / 公权法人 / 机构账户 / 个人多签)由
//   `governance/shared/institution_code_label.dart` 提供,本文件只在其上叠加
//   五子 tab 的「业务分组」,绝不复制码表。
// - 五子 tab 中:立法/治理/公权 是机构视图(本文件过滤);广场/选举 是活动视图
//   (非机构子集,见 ADR-028 决策 5/7),不在本文件。
// - 公权 = 全集(超集),不需过滤——任何已知机构都属于公权。

import 'package:citizenapp/governance/shared/institution_code_label.dart';

/// 机构所属的子 tab 业务分组(仅机构视图三类;公权为全集不单列)。
enum InstitutionTabGroup {
  /// 立法机构:立法律条文(国家/省立法院含参众议会、市立法会、国家公民教育委员会)。
  legislation,

  /// 治理机构:区块链/货币治理(国储会/省储会/省储行)。≠ 宪法「自治政府」(ADR-028 决策 9)。
  governance,

  /// 其它公权机构(行政/司法/监察/公安…),只在公权 tab 出现。
  other,
}

/// 立法 tab 机构码集合(ADR-028 决策 3):国家/省立法院(含参众议会)、市立法会、
/// 国家公民教育委员会(起草教育类法案,宪法第十/四十四条 → 立法职能)。
const Set<String> kLegislationCodes = <String>{
  'NLG', // 国家立法院
  'NSN', // 国家参议会
  'NRP', // 国家众议会
  'PLG', // 省立法院
  'PSN', // 省参议会
  'PRP', // 省众议会
  'CLEG', // 市立法会(市公民立法委员会)
  'NED', // 国家公民教育委员会
};

/// 治理 tab 机构码集合:固定治理档(与 `InstitutionCodeLabel.isFixedGovernance` 同源)。
const Set<String> kGovernanceCodes = <String>{'NRC', 'PRC', 'PRB'};

/// 机构分类工具(纯函数,单一源)。
class InstitutionClassification {
  const InstitutionClassification._();

  /// 是否立法机构(立法 tab 过滤谓词)。
  static bool isLegislation(String institutionCode) =>
      kLegislationCodes.contains(institutionCode);

  /// 是否治理机构(治理 tab 过滤谓词)。
  static bool isGovernance(String institutionCode) =>
      kGovernanceCodes.contains(institutionCode);

  /// 机构所属业务分组(立法/治理/其它)。公权 tab 不依赖本分组(全集)。
  static InstitutionTabGroup groupOf(String institutionCode) {
    if (isLegislation(institutionCode)) return InstitutionTabGroup.legislation;
    if (isGovernance(institutionCode)) return InstitutionTabGroup.governance;
    return InstitutionTabGroup.other;
  }

  /// 机构码人机展示标签(复用单一源:固定治理/个人多签特化中文,其余返回码本身)。
  static String label(String institutionCode) =>
      InstitutionCodeLabel.codeLabel(institutionCode);
}
