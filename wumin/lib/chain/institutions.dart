// 链上机构中文名注册表（冷钱包签名校验用）。
//
// 唯一事实源：citizenchain/node/src/ui/governance/mod.rs 中的三个 static 数组
//   - NATIONAL_COUNCILS   （国储会 Nrc，1 个）
//   - PROVINCIAL_COUNCILS （省储会 Prc，43 个）
//   - PROVINCIAL_BANKS    （省储行 Prb，43 个）
// 合计 87 个机构。服务端 `find_entry(sfid_number)` 会在这三个数组中依次查找，
// 冷钱包解码 payload 时用同一套映射把 `sfid_number` 还原成中文名，
// 保证 `display.fields.institution` 与 `decoded.fields.institution` 逐字节一致。
//
// 任何机构名/身份号变更必须同时更新此文件与服务端 mod.rs。
// 以后改为代码生成后可删除手抄。

/// 机构分类（与服务端 OrgType 对齐）。
enum InstitutionType {
  /// 国家储备委员会。
  nrc,

  /// 省级储备委员会。
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
    sfidNumber: 'GFR-LN001-CB0X-944805165-2026',
    name: '国家储备委员会',
    type: InstitutionType.nrc,
  ),
];

/// 省储会（43）。
const List<Institution> kProvincialCouncils = [
  Institution(sfidNumber: 'GFR-ZS001-CB0Y-016974075-2026', name: '中枢省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-LN001-CB02-773405642-2026', name: '岭南省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-GD001-CB0L-067440774-2026', name: '广东省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-GX001-CB0I-663454043-2026', name: '广西省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-FJ001-CB03-389570546-2026', name: '福建省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HN001-CB0X-545676096-2026', name: '海南省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-YN001-CB0K-145427171-2026', name: '云南省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-GZ001-CB0I-969970096-2026', name: '贵州省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HU001-CB03-400319700-2026', name: '湖南省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-JX001-CB0Q-458681566-2026', name: '江西省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-ZJ001-CB0J-471270801-2026', name: '浙江省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-JS001-CB08-358467174-2026', name: '江苏省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-SD001-CB03-027328848-2026', name: '山东省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-SX001-CB08-104465679-2026', name: '山西省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HE001-CB02-849245626-2026', name: '河南省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HB001-CB07-499533387-2026', name: '河北省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HI001-CB01-659443961-2026', name: '湖北省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-SI001-CB0Y-711309909-2026', name: '陕西省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-CQ001-CB0Z-478472058-2026', name: '重庆省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-SC001-CB0N-935659021-2026', name: '四川省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-GS001-CB0K-679051155-2026', name: '甘肃省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-BP001-CB06-189323546-2026', name: '北平省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HA001-CB0C-214178517-2026', name: '海滨省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-SJ001-CB0V-044490898-2026', name: '松江省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-LJ001-CB05-279890045-2026', name: '龙江省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-JL001-CB0C-850461124-2026', name: '吉林省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-LI001-CB0P-978545133-2026', name: '辽宁省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-NX001-CB0C-389752794-2026', name: '宁夏省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-QH001-CB0C-882026762-2026', name: '青海省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-AH001-CB0O-589856828-2026', name: '安徽省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-TW001-CB0H-265218823-2026', name: '台湾省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-XZ001-CB05-435616961-2026', name: '西藏省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-XJ001-CB0F-671044381-2026', name: '新疆省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-XK001-CB05-695945392-2026', name: '西康省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-AL001-CB0Z-487847725-2026', name: '阿里省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-CL001-CB0B-771698743-2026', name: '葱岭省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-TS001-CB0T-293160581-2026', name: '天山省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HX001-CB0I-475713213-2026', name: '河西省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-KL001-CB0Q-091969119-2026', name: '昆仑省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HT001-CB07-481172908-2026', name: '河套省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-RH001-CB08-697831866-2026', name: '热河省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-XA001-CB0V-384161601-2026', name: '兴安省储备委员会', type: InstitutionType.prc),
  Institution(sfidNumber: 'GFR-HJ001-CB0K-963948997-2026', name: '合江省储备委员会', type: InstitutionType.prc),
];

/// 省储行（43）。
const List<Institution> kProvincialBanks = [
  Institution(sfidNumber: 'SFR-ZS001-CH1J-233384677-2026', name: '中枢省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-LN001-CH1O-703127075-2026', name: '岭南省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-GD001-CH1I-239565809-2026', name: '广东省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-GX001-CH1Q-025559630-2026', name: '广西省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-FJ001-CH1L-504679612-2026', name: '福建省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HN001-CH1L-723623074-2026', name: '海南省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-YN001-CH11-692525950-2026', name: '云南省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-GZ001-CH1R-490015860-2026', name: '贵州省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HU001-CH1G-084835673-2026', name: '湖南省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-JX001-CH13-243765987-2026', name: '江西省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-ZJ001-CH1B-296232973-2026', name: '浙江省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-JS001-CH16-890774605-2026', name: '江苏省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-SD001-CH1B-114256751-2026', name: '山东省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-SX001-CH1X-520132196-2026', name: '山西省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HE001-CH12-158889343-2026', name: '河南省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HB001-CH1R-484022741-2026', name: '河北省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HI001-CH1G-514948302-2026', name: '湖北省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-SI001-CH1D-245618374-2026', name: '陕西省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-CQ001-CH18-694162045-2026', name: '重庆省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-SC001-CH1Y-764253139-2026', name: '四川省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-GS001-CH14-005784877-2026', name: '甘肃省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-BP001-CH1M-434307982-2026', name: '北平省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HA001-CH19-969179618-2026', name: '海滨省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-SJ001-CH1G-644104544-2026', name: '松江省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-LJ001-CH1J-280510636-2026', name: '龙江省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-JL001-CH17-129935340-2026', name: '吉林省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-LI001-CH10-249814963-2026', name: '辽宁省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-NX001-CH1N-292327153-2026', name: '宁夏省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-QH001-CH12-075657014-2026', name: '青海省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-AH001-CH1D-388477914-2026', name: '安徽省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-TW001-CH1X-266238196-2026', name: '台湾省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-XZ001-CH1U-210788637-2026', name: '西藏省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-XJ001-CH1J-233325633-2026', name: '新疆省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-XK001-CH1Z-300401625-2026', name: '西康省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-AL001-CH1J-527686065-2026', name: '阿里省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-CL001-CH1Z-951267669-2026', name: '葱岭省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-TS001-CH1A-142800261-2026', name: '天山省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HX001-CH1N-215310265-2026', name: '河西省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-KL001-CH1R-682838027-2026', name: '昆仑省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HT001-CH1V-210616196-2026', name: '河套省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-RH001-CH10-380830938-2026', name: '热河省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-XA001-CH1P-928028839-2026', name: '兴安省公民储备银行', type: InstitutionType.prb),
  Institution(sfidNumber: 'SFR-HJ001-CH1M-089279108-2026', name: '合江省公民储备银行', type: InstitutionType.prb),
];

/// 所有机构（87）。按服务端 find_entry 的查找顺序：NRC → PRC → PRB。
final List<Institution> kAllInstitutions = List.unmodifiable([
  ...kNationalCouncils,
  ...kProvincialCouncils,
  ...kProvincialBanks,
]);

/// 根据 sfid_number 查找机构中文名（任意类型：国储会 / 省储会 / 省储行）。
///
/// 返回 null 表示链上交易含未知机构 —— 解码器会 fallback 成原始 sfid_number 字符串，
/// 但与服务端 `display` 字段对不上会触发"交易内容与摘要不符"。
/// 若遇到此情况，说明两端数据未对齐，应补齐本文件。
String? institutionName(String sfidNumber) {
  for (final inst in kAllInstitutions) {
    if (inst.sfidNumber == sfidNumber) return inst.name;
  }
  return null;
}
