/// 省储行常量列表。
///
/// 数据来源：citizenchain/runtime/primitives/china/china_ch.rs 中 CHINA_CH 数组。
/// shenfen_id 用于链上标识（转为 InstitutionPalletId），shenfen_name 用于用户界面显示。
class ClearingBank {
  const ClearingBank({required this.shenfenId, required this.shenfenName, required this.wssUrl, required this.enabled});

  /// 省储行身份标识（链上唯一标识，对应 china_ch.rs 中 shenfen_id）。
  final String shenfenId;

  /// 省储行中文名称（用户界面显示）。
  final String shenfenName;

  /// 省储行链下清算 WSS RPC 地址。
  final String wssUrl;

  /// 是否已开通链下清算服务（false 表示未开通，前端禁止选择）。
  final bool enabled;
}

/// 43 个省储行完整列表。
const List<ClearingBank> clearingBanks = [
  ClearingBank(shenfenId: 'SFR-ZS001-CH1Z-572590896-20260222', shenfenName: '中枢省公民储备银行', wssUrl: 'wss://prbzss.crcfrcn.com', enabled: true),
  ClearingBank(shenfenId: 'SFR-LN001-CH1D-067241191-20260222', shenfenName: '岭南省公民储备银行', wssUrl: 'wss://prblns.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-GD000-CH1S-539766913-20260222', shenfenName: '广东省公民储备银行', wssUrl: 'wss://prbgds.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-GX000-CH17-770836097-20260222', shenfenName: '广西省公民储备银行', wssUrl: 'wss://prbgxs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-FJ000-CH1Y-285514007-20260222', shenfenName: '福建省公民储备银行', wssUrl: 'wss://prbfjs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HN000-CH1W-701494632-20260222', shenfenName: '海南省公民储备银行', wssUrl: 'wss://prbhns.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-YN000-CH1M-088552001-20260222', shenfenName: '云南省公民储备银行', wssUrl: 'wss://prbyns.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-GZ000-CH17-073795499-20260222', shenfenName: '贵州省公民储备银行', wssUrl: 'wss://prbgzs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HU000-CH1P-721228492-20260222', shenfenName: '湖南省公民储备银行', wssUrl: 'wss://prbhus.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-JX000-CH1T-532829662-20260222', shenfenName: '江西省公民储备银行', wssUrl: 'wss://prbjxs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-ZJ000-CH19-249528657-20260222', shenfenName: '浙江省公民储备银行', wssUrl: 'wss://prbzjs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-JS000-CH1C-191178842-20260222', shenfenName: '江苏省公民储备银行', wssUrl: 'wss://prbjss.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-SD000-CH1V-887886640-20260222', shenfenName: '山东省公民储备银行', wssUrl: 'wss://prbsds.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-SX000-CH1F-755750488-20260222', shenfenName: '山西省公民储备银行', wssUrl: 'wss://prbsxs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HE000-CH1T-357503840-20260222', shenfenName: '河南省公民储备银行', wssUrl: 'wss://prbhes.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HB000-CH12-172598053-20260222', shenfenName: '河北省公民储备银行', wssUrl: 'wss://prbhbs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HI000-CH1W-584177104-20260222', shenfenName: '湖北省公民储备银行', wssUrl: 'wss://prbhis.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-SI000-CH1G-814942227-20260222', shenfenName: '陕西省公民储备银行', wssUrl: 'wss://prbsis.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-CQ001-CH1A-811483361-20260222', shenfenName: '重庆省公民储备银行', wssUrl: 'wss://prbcqs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-SC000-CH19-320507619-20260222', shenfenName: '四川省公民储备银行', wssUrl: 'wss://prbscs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-GS000-CH1U-319639307-20260222', shenfenName: '甘肃省公民储备银行', wssUrl: 'wss://prbgss.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-BP001-CH19-330141933-20260222', shenfenName: '北平省公民储备银行', wssUrl: 'wss://prbbps.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HA000-CH1N-832919801-20260222', shenfenName: '滨海省公民储备银行', wssUrl: 'wss://prbhas.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-SJ000-CH17-991726244-20260222', shenfenName: '松江省公民储备银行', wssUrl: 'wss://prbsjs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-LJ000-CH1U-321069400-20260222', shenfenName: '龙江省公民储备银行', wssUrl: 'wss://prbljs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-JL000-CH1Z-114671562-20260222', shenfenName: '吉林省公民储备银行', wssUrl: 'wss://prbjls.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-LI000-CH1O-060821950-20260222', shenfenName: '辽宁省公民储备银行', wssUrl: 'wss://prblis.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-NX000-CH1W-927112322-20260222', shenfenName: '宁夏省公民储备银行', wssUrl: 'wss://prbnxs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-QH000-CH15-480036803-20260222', shenfenName: '青海省公民储备银行', wssUrl: 'wss://prbqhs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-AH000-CH14-243470490-20260222', shenfenName: '安徽省公民储备银行', wssUrl: 'wss://prbahs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-TW000-CH1O-339827620-20260222', shenfenName: '台湾省公民储备银行', wssUrl: 'wss://prbtws.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-XZ000-CH1A-076183922-20260222', shenfenName: '西藏省公民储备银行', wssUrl: 'wss://prbxzs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-XJ000-CH1T-624864385-20260222', shenfenName: '新疆省公民储备银行', wssUrl: 'wss://prbxjs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-XK000-CH19-727906387-20260222', shenfenName: '西康省公民储备银行', wssUrl: 'wss://prbxks.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-AL000-CH1Z-823361903-20260222', shenfenName: '阿里省公民储备银行', wssUrl: 'wss://prbals.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-CL000-CH1I-930688147-20260222', shenfenName: '葱岭省公民储备银行', wssUrl: 'wss://prbcls.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-TS000-CH1S-351739678-20260222', shenfenName: '天山省公民储备银行', wssUrl: 'wss://prbtss.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HX000-CH1X-115163356-20260222', shenfenName: '河西省公民储备银行', wssUrl: 'wss://prbhxs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-KL000-CH1F-853206078-20260222', shenfenName: '昆仑省公民储备银行', wssUrl: 'wss://prbkls.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HT000-CH1H-294801127-20260222', shenfenName: '河套省公民储备银行', wssUrl: 'wss://prbhts.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-RH000-CH14-762808938-20260222', shenfenName: '热河省公民储备银行', wssUrl: 'wss://prbrhs.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-XA000-CH1P-285320269-20260222', shenfenName: '兴安省公民储备银行', wssUrl: 'wss://prbxas.crcfrcn.com', enabled: false),
  ClearingBank(shenfenId: 'SFR-HJ000-CH1C-538936570-20260222', shenfenName: '合江省公民储备银行', wssUrl: 'wss://prbhjs.crcfrcn.com', enabled: false),
];

/// 根据 shenfen_id 查找省储行中文名称。
String? clearingBankName(String shenfenId) {
  for (final bank in clearingBanks) {
    if (bank.shenfenId == shenfenId) return bank.shenfenName;
  }
  return null;
}

/// 根据 shenfen_id 查找省储行。
ClearingBank? findClearingBank(String shenfenId) {
  for (final bank in clearingBanks) {
    if (bank.shenfenId == shenfenId) return bank;
  }
  return null;
}
