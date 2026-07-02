// 链上机构名称注册表（公民钱包签名校验用）。
//
// 本文件由 scripts/generate_citizenapp_governance_registry.mjs 自动生成。
// 唯一事实源是 citizenchain/runtime/primitives/cid/china/china_{cb,ch}.rs。
// 冷钱包用同一套映射把 cid_number 还原成中英全称/简称，保证交易摘要与解码结果一致。

/// 机构分类（与服务端 OrgType 对齐）。
enum InstitutionType {
  /// 国家公民储备委员会。
  nrc,

  /// 省级公民储备委员会。
  prc,

  /// 省级公民储备银行。
  prb,

  /// 其它固定治理机构。
  fixedGovernance,
}

class Institution {
  const Institution({
    required this.cidNumber,
    required this.cidFullName,
    required this.cidShortName,
    required this.cidFullNameEn,
    required this.cidShortNameEn,
    required this.type,
  });

  final String cidNumber;
  final String cidFullName;
  final String cidShortName;
  final String cidFullNameEn;
  final String cidShortNameEn;
  final InstitutionType type;
}

/// 国储会（1）。
const List<Institution> kNationalCouncils = [
  Institution(
    cidNumber: 'LN001-NRC0G-944805165-2026',
    cidFullName: '国家公民储备委员会',
    cidShortName: '国储会',
    cidFullNameEn: 'National Citizen Reserve Committee',
    cidShortNameEn: 'National Reserve Committee',
    type: InstitutionType.nrc,
  ),
];

/// 省储会（43）。
const List<Institution> kProvincialCouncils = [
  Institution(
    cidNumber: 'ZS001-PRC0E-016974075-2026',
    cidFullName: '中枢省公民储备委员会',
    cidShortName: '中枢省储会',
    cidFullNameEn: 'Zhongshu Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Zhongshu Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'LN001-PRC05-773405642-2026',
    cidFullName: '岭南省公民储备委员会',
    cidShortName: '岭南省储会',
    cidFullNameEn: 'Lingnan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Lingnan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'GD001-PRC0V-067440774-2026',
    cidFullName: '广东省公民储备委员会',
    cidShortName: '广东省储会',
    cidFullNameEn: 'Guangdong Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Guangdong Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'GX001-PRC0C-663454043-2026',
    cidFullName: '广西省公民储备委员会',
    cidShortName: '广西省储会',
    cidFullNameEn: 'Guangxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Guangxi Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'FJ001-PRC0I-389570546-2026',
    cidFullName: '福建省公民储备委员会',
    cidShortName: '福建省储会',
    cidFullNameEn: 'Fujian Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Fujian Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HN001-PRC0S-545676096-2026',
    cidFullName: '海南省公民储备委员会',
    cidShortName: '海南省储会',
    cidFullNameEn: 'Hainan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hainan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'YN001-PRC0W-145427171-2026',
    cidFullName: '云南省公民储备委员会',
    cidShortName: '云南省储会',
    cidFullNameEn: 'Yunnan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Yunnan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'GZ001-PRC02-969970096-2026',
    cidFullName: '贵州省公民储备委员会',
    cidShortName: '贵州省储会',
    cidFullNameEn: 'Guizhou Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Guizhou Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HU001-PRC0P-400319700-2026',
    cidFullName: '湖南省公民储备委员会',
    cidShortName: '湖南省储会',
    cidFullNameEn: 'Hunan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hunan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'JX001-PRC0J-458681566-2026',
    cidFullName: '江西省公民储备委员会',
    cidShortName: '江西省储会',
    cidFullNameEn: 'Jiangxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Jiangxi Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'ZJ001-PRC08-471270801-2026',
    cidFullName: '浙江省公民储备委员会',
    cidShortName: '浙江省储会',
    cidFullNameEn: 'Zhejiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Zhejiang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'JS001-PRC0O-358467174-2026',
    cidFullName: '江苏省公民储备委员会',
    cidShortName: '江苏省储会',
    cidFullNameEn: 'Jiangsu Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Jiangsu Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'SD001-PRC07-027328848-2026',
    cidFullName: '山东省公民储备委员会',
    cidShortName: '山东省储会',
    cidFullNameEn: 'Shandong Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Shandong Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'SX001-PRC0O-104465679-2026',
    cidFullName: '山西省公民储备委员会',
    cidShortName: '山西省储会',
    cidFullNameEn: 'Shanxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Shanxi Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HE001-PRC0S-849245626-2026',
    cidFullName: '河南省公民储备委员会',
    cidShortName: '河南省储会',
    cidFullNameEn: 'Henan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Henan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HB001-PRC0W-499533387-2026',
    cidFullName: '河北省公民储备委员会',
    cidShortName: '河北省储会',
    cidFullNameEn: 'Hebei Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hebei Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HI001-PRC0D-659443961-2026',
    cidFullName: '湖北省公民储备委员会',
    cidShortName: '湖北省储会',
    cidFullNameEn: 'Hubei Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hubei Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'SI001-PRC0T-711309909-2026',
    cidFullName: '陕西省公民储备委员会',
    cidShortName: '陕西省储会',
    cidFullNameEn: 'Shaanxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Shaanxi Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'CQ001-PRC06-478472058-2026',
    cidFullName: '重庆省公民储备委员会',
    cidShortName: '重庆省储会',
    cidFullNameEn: 'Chongqing Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Chongqing Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'SC001-PRC0Y-935659021-2026',
    cidFullName: '四川省公民储备委员会',
    cidShortName: '四川省储会',
    cidFullNameEn: 'Sichuan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Sichuan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'GS001-PRC0L-679051155-2026',
    cidFullName: '甘肃省公民储备委员会',
    cidShortName: '甘肃省储会',
    cidFullNameEn: 'Gansu Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Gansu Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'BP001-PRC0R-189323546-2026',
    cidFullName: '北平省公民储备委员会',
    cidShortName: '北平省储会',
    cidFullNameEn: 'Beiping Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Beiping Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HA001-PRC0Y-214178517-2026',
    cidFullName: '海滨省公民储备委员会',
    cidShortName: '海滨省储会',
    cidFullNameEn: 'Haibin Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Haibin Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'SJ001-PRC09-044490898-2026',
    cidFullName: '松江省公民储备委员会',
    cidShortName: '松江省储会',
    cidFullNameEn: 'Songjiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Songjiang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'LJ001-PRC08-279890045-2026',
    cidFullName: '龙江省公民储备委员会',
    cidShortName: '龙江省储会',
    cidFullNameEn: 'Longjiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Longjiang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'JL001-PRC05-850461124-2026',
    cidFullName: '吉林省公民储备委员会',
    cidShortName: '吉林省储会',
    cidFullNameEn: 'Jilin Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Jilin Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'LI001-PRC0T-978545133-2026',
    cidFullName: '辽宁省公民储备委员会',
    cidShortName: '辽宁省储会',
    cidFullNameEn: 'Liaoning Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Liaoning Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'NX001-PRC0J-389752794-2026',
    cidFullName: '宁夏省公民储备委员会',
    cidShortName: '宁夏省储会',
    cidFullNameEn: 'Ningxia Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Ningxia Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'QH001-PRC0C-882026762-2026',
    cidFullName: '青海省公民储备委员会',
    cidShortName: '青海省储会',
    cidFullNameEn: 'Qinghai Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Qinghai Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'AH001-PRC00-589856828-2026',
    cidFullName: '安徽省公民储备委员会',
    cidShortName: '安徽省储会',
    cidFullNameEn: 'Anhui Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Anhui Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'TW001-PRC07-265218823-2026',
    cidFullName: '台湾省公民储备委员会',
    cidShortName: '台湾省储会',
    cidFullNameEn: 'Taiwan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Taiwan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'XZ001-PRC02-435616961-2026',
    cidFullName: '西藏省公民储备委员会',
    cidShortName: '西藏省储会',
    cidFullNameEn: 'Xizang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xizang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'XJ001-PRC02-671044381-2026',
    cidFullName: '新疆省公民储备委员会',
    cidShortName: '新疆省储会',
    cidFullNameEn: 'Xinjiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xinjiang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'XK001-PRC0P-695945392-2026',
    cidFullName: '西康省公民储备委员会',
    cidShortName: '西康省储会',
    cidFullNameEn: 'Xikang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xikang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'AL001-PRC0D-487847725-2026',
    cidFullName: '阿里省公民储备委员会',
    cidShortName: '阿里省储会',
    cidFullNameEn: 'Ali Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Ali Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'CL001-PRC0J-771698743-2026',
    cidFullName: '葱岭省公民储备委员会',
    cidShortName: '葱岭省储会',
    cidFullNameEn: 'Congling Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Congling Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'YL001-PRC0Q-293160581-2026',
    cidFullName: '伊犁省公民储备委员会',
    cidShortName: '伊犁省储会',
    cidFullNameEn: 'Yili Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Yili Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HX001-PRC0D-475713213-2026',
    cidFullName: '河西省公民储备委员会',
    cidShortName: '河西省储会',
    cidFullNameEn: 'Hexi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hexi Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'KL001-PRC0O-091969119-2026',
    cidFullName: '昆仑省公民储备委员会',
    cidShortName: '昆仑省储会',
    cidFullNameEn: 'Kunlun Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Kunlun Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HT001-PRC00-481172908-2026',
    cidFullName: '河套省公民储备委员会',
    cidShortName: '河套省储会',
    cidFullNameEn: 'Hetao Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hetao Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'RH001-PRC0F-697831866-2026',
    cidFullName: '热河省公民储备委员会',
    cidShortName: '热河省储会',
    cidFullNameEn: 'Rehe Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Rehe Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'XA001-PRC0H-384161601-2026',
    cidFullName: '兴安省公民储备委员会',
    cidShortName: '兴安省储会',
    cidFullNameEn: 'Xingan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xingan Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
  Institution(
    cidNumber: 'HJ001-PRC0V-963948997-2026',
    cidFullName: '合江省公民储备委员会',
    cidShortName: '合江省储会',
    cidFullNameEn: 'Hejiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hejiang Provincial Reserve Committee',
    type: InstitutionType.prc,
  ),
];

/// 省储行（43）。
const List<Institution> kProvincialBanks = [
  Institution(
    cidNumber: 'ZS001-PRB08-233384677-2026',
    cidFullName: '中枢省公民储备银行',
    cidShortName: '中枢省储行',
    cidFullNameEn: 'Zhongshu Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Zhongshu Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'LN001-PRB0K-703127075-2026',
    cidFullName: '岭南省公民储备银行',
    cidShortName: '岭南省储行',
    cidFullNameEn: 'Lingnan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Lingnan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'GD001-PRB0T-239565809-2026',
    cidFullName: '广东省公民储备银行',
    cidShortName: '广东省储行',
    cidFullNameEn: 'Guangdong Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Guangdong Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'GX001-PRB01-025559630-2026',
    cidFullName: '广西省公民储备银行',
    cidShortName: '广西省储行',
    cidFullNameEn: 'Guangxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Guangxi Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'FJ001-PRB0V-504679612-2026',
    cidFullName: '福建省公民储备银行',
    cidShortName: '福建省储行',
    cidFullNameEn: 'Fujian Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Fujian Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HN001-PRB0P-723623074-2026',
    cidFullName: '海南省公民储备银行',
    cidShortName: '海南省储行',
    cidFullNameEn: 'Hainan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hainan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'YN001-PRB08-692525950-2026',
    cidFullName: '云南省公民储备银行',
    cidShortName: '云南省储行',
    cidFullNameEn: 'Yunnan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Yunnan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'GZ001-PRB00-490015860-2026',
    cidFullName: '贵州省公民储备银行',
    cidShortName: '贵州省储行',
    cidFullNameEn: 'Guizhou Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Guizhou Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HU001-PRB0F-084835673-2026',
    cidFullName: '湖南省公民储备银行',
    cidShortName: '湖南省储行',
    cidFullNameEn: 'Hunan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hunan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'JX001-PRB09-243765987-2026',
    cidFullName: '江西省公民储备银行',
    cidShortName: '江西省储行',
    cidFullNameEn: 'Jiangxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Jiangxi Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'ZJ001-PRB0R-296232973-2026',
    cidFullName: '浙江省公民储备银行',
    cidShortName: '浙江省储行',
    cidFullNameEn: 'Zhejiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Zhejiang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'JS001-PRB01-890774605-2026',
    cidFullName: '江苏省公民储备银行',
    cidShortName: '江苏省储行',
    cidFullNameEn: 'Jiangsu Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Jiangsu Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'SD001-PRB0G-114256751-2026',
    cidFullName: '山东省公民储备银行',
    cidShortName: '山东省储行',
    cidFullNameEn: 'Shandong Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Shandong Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'SX001-PRB0K-520132196-2026',
    cidFullName: '山西省公民储备银行',
    cidShortName: '山西省储行',
    cidFullNameEn: 'Shanxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Shanxi Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HE001-PRB03-158889343-2026',
    cidFullName: '河南省公民储备银行',
    cidShortName: '河南省储行',
    cidFullNameEn: 'Henan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Henan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HB001-PRB0Z-484022741-2026',
    cidFullName: '河北省公民储备银行',
    cidShortName: '河北省储行',
    cidFullNameEn: 'Hebei Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hebei Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HI001-PRB0V-514948302-2026',
    cidFullName: '湖北省公民储备银行',
    cidShortName: '湖北省储行',
    cidFullNameEn: 'Hubei Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hubei Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'SI001-PRB0N-245618374-2026',
    cidFullName: '陕西省公民储备银行',
    cidShortName: '陕西省储行',
    cidFullNameEn: 'Shaanxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Shaanxi Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'CQ001-PRB0C-694162045-2026',
    cidFullName: '重庆省公民储备银行',
    cidShortName: '重庆省储行',
    cidFullNameEn: 'Chongqing Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Chongqing Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'SC001-PRB0Q-764253139-2026',
    cidFullName: '四川省公民储备银行',
    cidShortName: '四川省储行',
    cidFullNameEn: 'Sichuan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Sichuan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'GS001-PRB08-005784877-2026',
    cidFullName: '甘肃省公民储备银行',
    cidShortName: '甘肃省储行',
    cidFullNameEn: 'Gansu Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Gansu Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'BP001-PRB0Q-434307982-2026',
    cidFullName: '北平省公民储备银行',
    cidShortName: '北平省储行',
    cidFullNameEn: 'Beiping Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Beiping Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HA001-PRB08-969179618-2026',
    cidFullName: '海滨省公民储备银行',
    cidShortName: '海滨省储行',
    cidFullNameEn: 'Haibin Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Haibin Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'SJ001-PRB03-644104544-2026',
    cidFullName: '松江省公民储备银行',
    cidShortName: '松江省储行',
    cidFullNameEn: 'Songjiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Songjiang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'LJ001-PRB0T-280510636-2026',
    cidFullName: '龙江省公民储备银行',
    cidShortName: '龙江省储行',
    cidFullNameEn: 'Longjiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Longjiang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'JL001-PRB07-129935340-2026',
    cidFullName: '吉林省公民储备银行',
    cidShortName: '吉林省储行',
    cidFullNameEn: 'Jilin Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Jilin Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'LI001-PRB0J-249814963-2026',
    cidFullName: '辽宁省公民储备银行',
    cidShortName: '辽宁省储行',
    cidFullNameEn: 'Liaoning Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Liaoning Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'NX001-PRB0F-292327153-2026',
    cidFullName: '宁夏省公民储备银行',
    cidShortName: '宁夏省储行',
    cidFullNameEn: 'Ningxia Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Ningxia Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'QH001-PRB0V-075657014-2026',
    cidFullName: '青海省公民储备银行',
    cidShortName: '青海省储行',
    cidFullNameEn: 'Qinghai Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Qinghai Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'AH001-PRB0M-388477914-2026',
    cidFullName: '安徽省公民储备银行',
    cidShortName: '安徽省储行',
    cidFullNameEn: 'Anhui Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Anhui Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'TW001-PRB0S-266238196-2026',
    cidFullName: '台湾省公民储备银行',
    cidShortName: '台湾省储行',
    cidFullNameEn: 'Taiwan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Taiwan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'XZ001-PRB06-210788637-2026',
    cidFullName: '西藏省公民储备银行',
    cidShortName: '西藏省储行',
    cidFullNameEn: 'Xizang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xizang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'XJ001-PRB0V-233325633-2026',
    cidFullName: '新疆省公民储备银行',
    cidShortName: '新疆省储行',
    cidFullNameEn: 'Xinjiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xinjiang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'XK001-PRB0Q-300401625-2026',
    cidFullName: '西康省公民储备银行',
    cidShortName: '西康省储行',
    cidFullNameEn: 'Xikang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xikang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'AL001-PRB0S-527686065-2026',
    cidFullName: '阿里省公民储备银行',
    cidShortName: '阿里省储行',
    cidFullNameEn: 'Ali Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Ali Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'CL001-PRB0Q-951267669-2026',
    cidFullName: '葱岭省公民储备银行',
    cidShortName: '葱岭省储行',
    cidFullNameEn: 'Congling Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Congling Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'YL001-PRB0A-142800261-2026',
    cidFullName: '伊犁省公民储备银行',
    cidShortName: '伊犁省储行',
    cidFullNameEn: 'Yili Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Yili Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HX001-PRB0F-215310265-2026',
    cidFullName: '河西省公民储备银行',
    cidShortName: '河西省储行',
    cidFullNameEn: 'Hexi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hexi Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'KL001-PRB08-682838027-2026',
    cidFullName: '昆仑省公民储备银行',
    cidShortName: '昆仑省储行',
    cidFullNameEn: 'Kunlun Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Kunlun Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HT001-PRB0L-210616196-2026',
    cidFullName: '河套省公民储备银行',
    cidShortName: '河套省储行',
    cidFullNameEn: 'Hetao Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hetao Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'RH001-PRB0C-380830938-2026',
    cidFullName: '热河省公民储备银行',
    cidShortName: '热河省储行',
    cidFullNameEn: 'Rehe Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Rehe Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'XA001-PRB0Q-928028839-2026',
    cidFullName: '兴安省公民储备银行',
    cidShortName: '兴安省储行',
    cidFullNameEn: 'Xingan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xingan Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
  Institution(
    cidNumber: 'HJ001-PRB0I-089279108-2026',
    cidFullName: '合江省公民储备银行',
    cidShortName: '合江省储行',
    cidFullNameEn: 'Hejiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hejiang Provincial Reserve Bank',
    type: InstitutionType.prb,
  ),
];

/// 其它固定治理机构（2）。
const List<Institution> kFixedGovernanceInstitutions = [
  Institution(
    cidNumber: 'ZS001-FRG07-249474503-2026',
    cidFullName: '总统府联邦注册局',
    cidShortName: '联邦注册局',
    cidFullNameEn: 'Federal Registry Bureau of the Presidential Office',
    cidShortNameEn: 'Federal Registry Bureau',
    type: InstitutionType.fixedGovernance,
  ),
  Institution(
    cidNumber: 'ZS001-NJD0T-052283563-2026',
    cidFullName: '中华民族联邦共和国国家司法院',
    cidShortName: '国家司法院',
    cidFullNameEn:
        'National Judicial Yuan of the Federal Republic of the China Nation',
    cidShortNameEn: 'National Judicial Yuan',
    type: InstitutionType.fixedGovernance,
  ),
];

/// 所有机构（89）。按服务端 find_entry 的查找顺序：NRC → PRC → PRB → 固定治理。
final List<Institution> kAllInstitutions = List.unmodifiable([
  ...kNationalCouncils,
  ...kProvincialCouncils,
  ...kProvincialBanks,
  ...kFixedGovernanceInstitutions,
]);

/// 根据 cid_number 查找机构中文全称。
///
/// 返回 null 表示链上交易含未知机构。若遇到此情况，说明链端常量与公民钱包
/// 机构注册表未对齐，应重新运行生成器。
String? cidFullName(String cidNumber) {
  for (final inst in kAllInstitutions) {
    if (inst.cidNumber == cidNumber) return inst.cidFullName;
  }
  return null;
}

String? cidShortName(String cidNumber) {
  for (final inst in kAllInstitutions) {
    if (inst.cidNumber == cidNumber) return inst.cidShortName;
  }
  return null;
}

String? cidFullNameEn(String cidNumber) {
  for (final inst in kAllInstitutions) {
    if (inst.cidNumber == cidNumber) return inst.cidFullNameEn;
  }
  return null;
}

String? cidShortNameEn(String cidNumber) {
  for (final inst in kAllInstitutions) {
    if (inst.cidNumber == cidNumber) return inst.cidShortNameEn;
  }
  return null;
}
