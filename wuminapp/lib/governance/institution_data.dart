/// 机构数据模型与静态注册表。
///
/// shenfen_id 与链上 `AdminsOriginGov.CurrentAdmins` 存储的 key 一一对应，
/// 来源于 `primitives/china/china_cb.rs`（国储会+省储会）和
/// `primitives/china/china_ch.rs`（省储行）。
library;

/// 机构类型枚举，数值与链上 `org` 编码一致。
class OrgType {
  OrgType._();

  /// 国储会 National Reserve Committee
  static const int nrc = 0;

  /// 省储会 Provincial Reserve Committee
  static const int prc = 1;

  /// 省储行 Provincial Reserve Bank
  static const int prb = 2;

  static String label(int orgType) {
    switch (orgType) {
      case nrc:
        return '国储会';
      case prc:
        return '省储会';
      case prb:
        return '省储行';
      default:
        return '未知';
    }
  }
}

/// 单个机构的结构化信息。
class InstitutionInfo {
  const InstitutionInfo({
    required this.name,
    required this.shenfenId,
    required this.orgType,
  });

  /// 显示名称。
  final String name;

  /// 链上身份标识（与 Rust 常量 `shenfen_id` 完全一致）。
  /// 在查询链上存储时会右补零到 48 字节作为 `InstitutionPalletId`。
  final String shenfenId;

  /// 机构类型：0=NRC, 1=PRC, 2=PRB。
  final int orgType;

  /// 内部投票通过阈值。
  int get internalThreshold {
    switch (orgType) {
      case OrgType.nrc:
        return 13;
      case OrgType.prc:
        return 6;
      case OrgType.prb:
        return 6;
      default:
        return 0;
    }
  }
}

/// 国储会（1 个）。
const List<InstitutionInfo> kNationalCouncil = [
  InstitutionInfo(
    name: '国家储备委员会',
    shenfenId: 'GFR-LN001-CB0C-617776487-20260222',
    orgType: OrgType.nrc,
  ),
];

/// 省储会（43 个）。
const List<InstitutionInfo> kProvincialCouncils = [
  InstitutionInfo(name: '中枢省储备委员会', shenfenId: 'GFR-ZS001-CB0X-464088047-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '岭南省储备委员会', shenfenId: 'GFR-LN002-CB0Q-850177236-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '广东省储备委员会', shenfenId: 'GFR-GD000-CB0O-261883838-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '广西省储备委员会', shenfenId: 'GFR-GX000-CB0X-936039238-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '福建省储备委员会', shenfenId: 'GFR-FJ000-CB0I-232415560-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '海南省储备委员会', shenfenId: 'GFR-HN000-CB04-832186703-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '云南省储备委员会', shenfenId: 'GFR-YN000-CB0G-574048259-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '贵州省储备委员会', shenfenId: 'GFR-GZ000-CB03-700488596-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '湖南省储备委员会', shenfenId: 'GFR-HU000-CB0V-865805553-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '江西省储备委员会', shenfenId: 'GFR-JX000-CB09-183645800-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '浙江省储备委员会', shenfenId: 'GFR-ZJ000-CB0Y-452554562-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '江苏省储备委员会', shenfenId: 'GFR-JS000-CB0T-266669398-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '山东省储备委员会', shenfenId: 'GFR-SD000-CB0A-354794960-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '山西省储备委员会', shenfenId: 'GFR-SX000-CB0T-700141630-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '河南省储备委员会', shenfenId: 'GFR-HE000-CB0R-527771281-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '河北省储备委员会', shenfenId: 'GFR-HB000-CB04-025532397-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '湖北省储备委员会', shenfenId: 'GFR-HI000-CB0M-247491104-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '陕西省储备委员会', shenfenId: 'GFR-SI000-CB0Q-626717092-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '重庆省储备委员会', shenfenId: 'GFR-CQ001-CB00-452250444-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '四川省储备委员会', shenfenId: 'GFR-SC000-CB0N-676087668-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '甘肃省储备委员会', shenfenId: 'GFR-GS000-CB02-451145443-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '北平省储备委员会', shenfenId: 'GFR-BP001-CB0C-164347900-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '海滨省储备委员会', shenfenId: 'GFR-HA000-CB02-156526094-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '松江省储备委员会', shenfenId: 'GFR-SJ000-CB0A-005282342-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '龙江省储备委员会', shenfenId: 'GFR-LJ000-CB0A-105584375-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '吉林省储备委员会', shenfenId: 'GFR-JL000-CB0T-855212821-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '辽宁省储备委员会', shenfenId: 'GFR-LI000-CB03-221473214-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '宁夏省储备委员会', shenfenId: 'GFR-NX000-CB0A-240866560-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '青海省储备委员会', shenfenId: 'GFR-QH000-CB0N-229555853-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '安徽省储备委员会', shenfenId: 'GFR-AH000-CB0Q-714959233-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '台湾省储备委员会', shenfenId: 'GFR-TW000-CB0U-188063480-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '西藏省储备委员会', shenfenId: 'GFR-XZ000-CB0R-085197231-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '新疆省储备委员会', shenfenId: 'GFR-XJ000-CB0I-803866647-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '西康省储备委员会', shenfenId: 'GFR-XK000-CB0B-810391358-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '阿里省储备委员会', shenfenId: 'GFR-AL000-CB08-769336671-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '葱岭省储备委员会', shenfenId: 'GFR-CL000-CB0Z-914234080-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '天山省储备委员会', shenfenId: 'GFR-TS000-CB0O-063508625-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '河西省储备委员会', shenfenId: 'GFR-HX000-CB0J-238307168-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '昆仑省储备委员会', shenfenId: 'GFR-KL000-CB00-453003140-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '河套省储备委员会', shenfenId: 'GFR-HT000-CB0F-763975330-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '热河省储备委员会', shenfenId: 'GFR-RH000-CB0T-258553387-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '兴安省储备委员会', shenfenId: 'GFR-XA000-CB0D-997757073-20260222', orgType: OrgType.prc),
  InstitutionInfo(name: '合江省储备委员会', shenfenId: 'GFR-HJ000-CB0C-544834501-20260222', orgType: OrgType.prc),
];

/// 省储行（43 个）。
const List<InstitutionInfo> kProvincialBanks = [
  InstitutionInfo(name: '中枢省公民储备银行', shenfenId: 'SFR-ZS001-CH1Z-572590896-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '岭南省公民储备银行', shenfenId: 'SFR-LN001-CH1D-067241191-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '广东省公民储备银行', shenfenId: 'SFR-GD000-CH1S-539766913-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '广西省公民储备银行', shenfenId: 'SFR-GX000-CH17-770836097-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '福建省公民储备银行', shenfenId: 'SFR-FJ000-CH1Y-285514007-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '海南省公民储备银行', shenfenId: 'SFR-HN000-CH1W-701494632-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '云南省公民储备银行', shenfenId: 'SFR-YN000-CH1M-088552001-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '贵州省公民储备银行', shenfenId: 'SFR-GZ000-CH17-073795499-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '湖南省公民储备银行', shenfenId: 'SFR-HU000-CH1P-721228492-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '江西省公民储备银行', shenfenId: 'SFR-JX000-CH1T-532829662-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '浙江省公民储备银行', shenfenId: 'SFR-ZJ000-CH19-249528657-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '江苏省公民储备银行', shenfenId: 'SFR-JS000-CH1C-191178842-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '山东省公民储备银行', shenfenId: 'SFR-SD000-CH1V-887886640-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '山西省公民储备银行', shenfenId: 'SFR-SX000-CH1F-755750488-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '河南省公民储备银行', shenfenId: 'SFR-HE000-CH1T-357503840-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '河北省公民储备银行', shenfenId: 'SFR-HB000-CH12-172598053-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '湖北省公民储备银行', shenfenId: 'SFR-HI000-CH1W-584177104-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '陕西省公民储备银行', shenfenId: 'SFR-SI000-CH1G-814942227-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '重庆省公民储备银行', shenfenId: 'SFR-CQ001-CH1A-811483361-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '四川省公民储备银行', shenfenId: 'SFR-SC000-CH19-320507619-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '甘肃省公民储备银行', shenfenId: 'SFR-GS000-CH1U-319639307-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '北平省公民储备银行', shenfenId: 'SFR-BP001-CH19-330141933-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '滨海省公民储备银行', shenfenId: 'SFR-HA000-CH1N-832919801-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '松江省公民储备银行', shenfenId: 'SFR-SJ000-CH17-991726244-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '龙江省公民储备银行', shenfenId: 'SFR-LJ000-CH1U-321069400-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '吉林省公民储备银行', shenfenId: 'SFR-JL000-CH1Z-114671562-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '辽宁省公民储备银行', shenfenId: 'SFR-LI000-CH1O-060821950-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '宁夏省公民储备银行', shenfenId: 'SFR-NX000-CH1W-927112322-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '青海省公民储备银行', shenfenId: 'SFR-QH000-CH15-480036803-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '安徽省公民储备银行', shenfenId: 'SFR-AH000-CH14-243470490-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '台湾省公民储备银行', shenfenId: 'SFR-TW000-CH1O-339827620-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '西藏省公民储备银行', shenfenId: 'SFR-XZ000-CH1A-076183922-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '新疆省公民储备银行', shenfenId: 'SFR-XJ000-CH1T-624864385-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '西康省公民储备银行', shenfenId: 'SFR-XK000-CH19-727906387-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '阿里省公民储备银行', shenfenId: 'SFR-AL000-CH1Z-823361903-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '葱岭省公民储备银行', shenfenId: 'SFR-CL000-CH1I-930688147-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '天山省公民储备银行', shenfenId: 'SFR-TS000-CH1S-351739678-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '河西省公民储备银行', shenfenId: 'SFR-HX000-CH1X-115163356-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '昆仑省公民储备银行', shenfenId: 'SFR-KL000-CH1F-853206078-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '河套省公民储备银行', shenfenId: 'SFR-HT000-CH1H-294801127-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '热河省公民储备银行', shenfenId: 'SFR-RH000-CH14-762808938-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '兴安省公民储备银行', shenfenId: 'SFR-XA000-CH1P-285320269-20260222', orgType: OrgType.prb),
  InstitutionInfo(name: '合江省公民储备银行', shenfenId: 'SFR-HJ000-CH1C-538936570-20260222', orgType: OrgType.prb),
];
