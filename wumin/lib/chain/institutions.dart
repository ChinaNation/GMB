// 链上机构中文名注册表（公民钱包签名校验用）。
//
// 本文件由 tools/generate_wuminapp_governance_registry.mjs 自动生成。
// 中文注释：唯一事实源是 citizenchain/runtime/primitives/china/china_{cb,ch}.rs。
// 冷钱包用同一套映射把 sfid_number 还原成中文名，保证交易摘要与解码结果一致。

/// 机构分类（与服务端 OrgType 对齐）。
enum InstitutionType {
  /// 国家公民储备委员会。
  nrc,

  /// 省级公民储备委员会。
  prc,

  /// 省级公民储备银行。
  prb,
}

class Institution {
  const Institution({
    required this.sfidNumber,
    required this.name,
    required this.type,
  });

  final String sfidNumber;
  final String name;
  final InstitutionType type;
}

/// 国储会（1）。
const List<Institution> kNationalCouncils = [
  Institution(
    sfidNumber: 'LN001-GCB05-944805165-2026',
    name: '中华民族联邦共和国国家公民储备委员会',
    type: InstitutionType.nrc,
  ),
];

/// 省储会（43）。
const List<Institution> kProvincialCouncils = [
  Institution(
    sfidNumber: 'ZS001-GCB0R-016974075-2026',
    name: '中枢省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'LN001-GCB0I-773405642-2026',
    name: '岭南省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'GD001-GCB08-067440774-2026',
    name: '广东省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'GX001-GCB0P-663454043-2026',
    name: '广西省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'FJ001-GCB0V-389570546-2026',
    name: '福建省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HN001-GCB05-545676096-2026',
    name: '海南省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'YN001-GCB09-145427171-2026',
    name: '云南省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'GZ001-GCB0F-969970096-2026',
    name: '贵州省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HU001-GCB02-400319700-2026',
    name: '湖南省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'JX001-GCB0W-458681566-2026',
    name: '江西省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'ZJ001-GCB0L-471270801-2026',
    name: '浙江省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'JS001-GCB01-358467174-2026',
    name: '江苏省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'SD001-GCB0K-027328848-2026',
    name: '山东省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'SX001-GCB01-104465679-2026',
    name: '山西省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HE001-GCB05-849245626-2026',
    name: '河南省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HB001-GCB09-499533387-2026',
    name: '河北省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HI001-GCB0Q-659443961-2026',
    name: '湖北省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'SI001-GCB06-711309909-2026',
    name: '陕西省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'CQ001-GCB0J-478472058-2026',
    name: '重庆省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'SC001-GCB0B-935659021-2026',
    name: '四川省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'GS001-GCB0Y-679051155-2026',
    name: '甘肃省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'BP001-GCB04-189323546-2026',
    name: '北平省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HA001-GCB0B-214178517-2026',
    name: '海滨省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'SJ001-GCB0M-044490898-2026',
    name: '松江省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'LJ001-GCB0L-279890045-2026',
    name: '龙江省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'JL001-GCB0I-850461124-2026',
    name: '吉林省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'LI001-GCB06-978545133-2026',
    name: '辽宁省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'NX001-GCB0W-389752794-2026',
    name: '宁夏省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'QH001-GCB0P-882026762-2026',
    name: '青海省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'AH001-GCB0D-589856828-2026',
    name: '安徽省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'TW001-GCB0K-265218823-2026',
    name: '台湾省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'XZ001-GCB0F-435616961-2026',
    name: '西藏省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'XJ001-GCB0F-671044381-2026',
    name: '新疆省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'XK001-GCB02-695945392-2026',
    name: '西康省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'AL001-GCB0Q-487847725-2026',
    name: '阿里省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'CL001-GCB0W-771698743-2026',
    name: '葱岭省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'YL001-GCB0C-293160581-2026',
    name: '伊犁省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HX001-GCB0Q-475713213-2026',
    name: '河西省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'KL001-GCB01-091969119-2026',
    name: '昆仑省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HT001-GCB0D-481172908-2026',
    name: '河套省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'RH001-GCB0S-697831866-2026',
    name: '热河省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'XA001-GCB0U-384161601-2026',
    name: '兴安省公民储备委员会',
    type: InstitutionType.prc,
  ),
  Institution(
    sfidNumber: 'HJ001-GCB08-963948997-2026',
    name: '合江省公民储备委员会',
    type: InstitutionType.prc,
  ),
];

/// 省储行（43）。
const List<Institution> kProvincialBanks = [
  Institution(
    sfidNumber: 'ZS001-SCH1E-233384677-2026',
    name: '中枢省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'LN001-SCH1Q-703127075-2026',
    name: '岭南省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'GD001-SCH1Z-239565809-2026',
    name: '广东省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'GX001-SCH17-025559630-2026',
    name: '广西省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'FJ001-SCH11-504679612-2026',
    name: '福建省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HN001-SCH1V-723623074-2026',
    name: '海南省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'YN001-SCH1E-692525950-2026',
    name: '云南省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'GZ001-SCH16-490015860-2026',
    name: '贵州省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HU001-SCH1L-084835673-2026',
    name: '湖南省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'JX001-SCH1F-243765987-2026',
    name: '江西省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'ZJ001-SCH1X-296232973-2026',
    name: '浙江省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'JS001-SCH17-890774605-2026',
    name: '江苏省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'SD001-SCH1M-114256751-2026',
    name: '山东省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'SX001-SCH1Q-520132196-2026',
    name: '山西省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HE001-SCH19-158889343-2026',
    name: '河南省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HB001-SCH15-484022741-2026',
    name: '河北省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HI001-SCH11-514948302-2026',
    name: '湖北省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'SI001-SCH1T-245618374-2026',
    name: '陕西省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'CQ001-SCH1I-694162045-2026',
    name: '重庆省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'SC001-SCH1W-764253139-2026',
    name: '四川省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'GS001-SCH1E-005784877-2026',
    name: '甘肃省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'BP001-SCH1W-434307982-2026',
    name: '北平省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HA001-SCH1E-969179618-2026',
    name: '海滨省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'SJ001-SCH19-644104544-2026',
    name: '松江省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'LJ001-SCH1Z-280510636-2026',
    name: '龙江省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'JL001-SCH1D-129935340-2026',
    name: '吉林省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'LI001-SCH1P-249814963-2026',
    name: '辽宁省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'NX001-SCH1L-292327153-2026',
    name: '宁夏省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'QH001-SCH11-075657014-2026',
    name: '青海省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'AH001-SCH1S-388477914-2026',
    name: '安徽省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'TW001-SCH1Y-266238196-2026',
    name: '台湾省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'XZ001-SCH1C-210788637-2026',
    name: '西藏省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'XJ001-SCH11-233325633-2026',
    name: '新疆省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'XK001-SCH1W-300401625-2026',
    name: '西康省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'AL001-SCH1Y-527686065-2026',
    name: '阿里省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'CL001-SCH1W-951267669-2026',
    name: '葱岭省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'YL001-SCH1P-142800261-2026',
    name: '伊犁省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HX001-SCH1L-215310265-2026',
    name: '河西省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'KL001-SCH1E-682838027-2026',
    name: '昆仑省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HT001-SCH1R-210616196-2026',
    name: '河套省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'RH001-SCH1I-380830938-2026',
    name: '热河省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'XA001-SCH1W-928028839-2026',
    name: '兴安省公民储备银行',
    type: InstitutionType.prb,
  ),
  Institution(
    sfidNumber: 'HJ001-SCH1O-089279108-2026',
    name: '合江省公民储备银行',
    type: InstitutionType.prb,
  ),
];

/// 所有机构（87）。按服务端 find_entry 的查找顺序：NRC → PRC → PRB。
final List<Institution> kAllInstitutions = List.unmodifiable([
  ...kNationalCouncils,
  ...kProvincialCouncils,
  ...kProvincialBanks,
]);

/// 根据 sfid_number 查找机构中文名（任意类型：国储会 / 省储会 / 省储行）。
///
/// 返回 null 表示链上交易含未知机构。若遇到此情况，说明链端常量与公民钱包
/// 机构注册表未对齐，应重新运行生成器。
String? sfidFullName(String sfidNumber) {
  for (final inst in kAllInstitutions) {
    if (inst.sfidNumber == sfidNumber) return inst.name;
  }
  return null;
}
