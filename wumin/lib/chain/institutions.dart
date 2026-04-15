// 链上机构中文名注册表（冷钱包签名校验用）。
//
// 唯一事实源：citizenchain/node/src/ui/governance/mod.rs 中的三个 static 数组
//   - NATIONAL_COUNCILS   （国储会 Nrc，1 个）
//   - PROVINCIAL_COUNCILS （省储会 Prc，43 个）
//   - PROVINCIAL_BANKS    （省储行 Prb，43 个）
// 合计 87 个机构。服务端 `find_entry(shenfen_id)` 会在这三个数组中依次查找，
// 冷钱包解码 payload 时用同一套映射把 `shenfen_id` 还原成中文名，
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
    required this.shenfenId,
    required this.name,
    required this.type,
  });

  final String shenfenId;
  final String name;
  final InstitutionType type;
}

/// 国储会（1）。
const List<Institution> kNationalCouncils = [
  Institution(
    shenfenId: 'GFR-LN001-CB0C-617776487-20260222',
    name: '国家储备委员会',
    type: InstitutionType.nrc,
  ),
];

/// 省储会（43）。
const List<Institution> kProvincialCouncils = [
  Institution(shenfenId: 'GFR-ZS001-CB0X-464088047-20260222', name: '中枢省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-LN002-CB0Q-850177236-20260222', name: '岭南省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-GD000-CB0O-261883838-20260222', name: '广东省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-GX000-CB0X-936039238-20260222', name: '广西省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-FJ000-CB0I-232415560-20260222', name: '福建省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HN000-CB04-832186703-20260222', name: '海南省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-YN000-CB0G-574048259-20260222', name: '云南省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-GZ000-CB03-700488596-20260222', name: '贵州省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HU000-CB0V-865805553-20260222', name: '湖南省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-JX000-CB09-183645800-20260222', name: '江西省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-ZJ000-CB0Y-452554562-20260222', name: '浙江省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-JS000-CB0T-266669398-20260222', name: '江苏省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-SD000-CB0A-354794960-20260222', name: '山东省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-SX000-CB0T-700141630-20260222', name: '山西省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HE000-CB0R-527771281-20260222', name: '河南省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HB000-CB04-025532397-20260222', name: '河北省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HI000-CB0M-247491104-20260222', name: '湖北省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-SI000-CB0Q-626717092-20260222', name: '陕西省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-CQ001-CB00-452250444-20260222', name: '重庆省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-SC000-CB0N-676087668-20260222', name: '四川省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-GS000-CB02-451145443-20260222', name: '甘肃省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-BP001-CB0C-164347900-20260222', name: '北平省储备委员会', type: InstitutionType.prc),
  // 注：HA000 省储会名为"海滨"，省储行名为"滨海"（服务端权威源原样如此，不自行统一）。
  Institution(shenfenId: 'GFR-HA000-CB02-156526094-20260222', name: '海滨省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-SJ000-CB0A-005282342-20260222', name: '松江省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-LJ000-CB0A-105584375-20260222', name: '龙江省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-JL000-CB0T-855212821-20260222', name: '吉林省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-LI000-CB03-221473214-20260222', name: '辽宁省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-NX000-CB0A-240866560-20260222', name: '宁夏省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-QH000-CB0N-229555853-20260222', name: '青海省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-AH000-CB0Q-714959233-20260222', name: '安徽省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-TW000-CB0U-188063480-20260222', name: '台湾省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-XZ000-CB0R-085197231-20260222', name: '西藏省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-XJ000-CB0I-803866647-20260222', name: '新疆省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-XK000-CB0B-810391358-20260222', name: '西康省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-AL000-CB08-769336671-20260222', name: '阿里省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-CL000-CB0Z-914234080-20260222', name: '葱岭省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-TS000-CB0O-063508625-20260222', name: '天山省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HX000-CB0J-238307168-20260222', name: '河西省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-KL000-CB00-453003140-20260222', name: '昆仑省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HT000-CB0F-763975330-20260222', name: '河套省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-RH000-CB0T-258553387-20260222', name: '热河省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-XA000-CB0D-997757073-20260222', name: '兴安省储备委员会', type: InstitutionType.prc),
  Institution(shenfenId: 'GFR-HJ000-CB0C-544834501-20260222', name: '合江省储备委员会', type: InstitutionType.prc),
];

/// 省储行（43）。
const List<Institution> kProvincialBanks = [
  Institution(shenfenId: 'SFR-ZS001-CH1Z-572590896-20260222', name: '中枢省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-LN001-CH1D-067241191-20260222', name: '岭南省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-GD000-CH1S-539766913-20260222', name: '广东省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-GX000-CH17-770836097-20260222', name: '广西省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-FJ000-CH1Y-285514007-20260222', name: '福建省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HN000-CH1W-701494632-20260222', name: '海南省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-YN000-CH1M-088552001-20260222', name: '云南省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-GZ000-CH17-073795499-20260222', name: '贵州省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HU000-CH1P-721228492-20260222', name: '湖南省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-JX000-CH1T-532829662-20260222', name: '江西省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-ZJ000-CH19-249528657-20260222', name: '浙江省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-JS000-CH1C-191178842-20260222', name: '江苏省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-SD000-CH1V-887886640-20260222', name: '山东省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-SX000-CH1F-755750488-20260222', name: '山西省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HE000-CH1T-357503840-20260222', name: '河南省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HB000-CH12-172598053-20260222', name: '河北省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HI000-CH1W-584177104-20260222', name: '湖北省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-SI000-CH1G-814942227-20260222', name: '陕西省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-CQ001-CH1A-811483361-20260222', name: '重庆省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-SC000-CH19-320507619-20260222', name: '四川省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-GS000-CH1U-319639307-20260222', name: '甘肃省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-BP001-CH19-330141933-20260222', name: '北平省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HA000-CH1N-832919801-20260222', name: '滨海省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-SJ000-CH17-991726244-20260222', name: '松江省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-LJ000-CH1U-321069400-20260222', name: '龙江省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-JL000-CH1Z-114671562-20260222', name: '吉林省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-LI000-CH1O-060821950-20260222', name: '辽宁省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-NX000-CH1W-927112322-20260222', name: '宁夏省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-QH000-CH15-480036803-20260222', name: '青海省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-AH000-CH14-243470490-20260222', name: '安徽省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-TW000-CH1O-339827620-20260222', name: '台湾省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-XZ000-CH1A-076183922-20260222', name: '西藏省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-XJ000-CH1T-624864385-20260222', name: '新疆省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-XK000-CH19-727906387-20260222', name: '西康省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-AL000-CH1Z-823361903-20260222', name: '阿里省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-CL000-CH1I-930688147-20260222', name: '葱岭省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-TS000-CH1S-351739678-20260222', name: '天山省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HX000-CH1X-115163356-20260222', name: '河西省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-KL000-CH1F-853206078-20260222', name: '昆仑省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HT000-CH1H-294801127-20260222', name: '河套省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-RH000-CH14-762808938-20260222', name: '热河省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-XA000-CH1P-285320269-20260222', name: '兴安省公民储备银行', type: InstitutionType.prb),
  Institution(shenfenId: 'SFR-HJ000-CH1C-538936570-20260222', name: '合江省公民储备银行', type: InstitutionType.prb),
];

/// 所有机构（87）。按服务端 find_entry 的查找顺序：NRC → PRC → PRB。
final List<Institution> kAllInstitutions = List.unmodifiable([
  ...kNationalCouncils,
  ...kProvincialCouncils,
  ...kProvincialBanks,
]);

/// 根据 shenfen_id 查找机构中文名（任意类型：国储会 / 省储会 / 省储行）。
///
/// 返回 null 表示链上交易含未知机构 —— 解码器会 fallback 成原始 shenfen_id 字符串，
/// 但与服务端 `display` 字段对不上会触发"交易内容与摘要不符"。
/// 若遇到此情况，说明两端数据未对齐，应补齐本文件。
String? institutionName(String shenfenId) {
  for (final inst in kAllInstitutions) {
    if (inst.shenfenId == shenfenId) return inst.name;
  }
  return null;
}
