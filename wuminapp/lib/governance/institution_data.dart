/// 机构数据模型与静态注册表。
///
/// shenfen_id 与链上 `AdminsOriginGov.CurrentAdmins` 存储的 key 一一对应，
/// 来源于 `primitives/china/china_cb.rs`（国储会+省储会）和
/// `primitives/china/china_ch.rs`（省储行）。
library;

/// 机构类型枚举，数值与链上 `org` 编码一致。
/// 提案 ID 显示格式化：2026000001 → "2026#1"
String formatProposalId(int proposalId) {
  final year = proposalId ~/ 1000000;
  final counter = proposalId % 1000000;
  return '$year#$counter';
}

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
    required this.duoqianAddress,
  });

  /// 显示名称。
  final String name;

  /// 链上身份标识（与 Rust 常量 `shenfen_id` 完全一致）。
  /// 在查询链上存储时会右补零到 48 字节作为 `InstitutionPalletId`。
  final String shenfenId;

  /// 机构类型：0=NRC, 1=PRC, 2=PRB。
  final int orgType;

  /// 机构多签名地址公钥（32 字节 hex，不含 0x 前缀）。
  /// 来源于 primitives 中的 `duoqian_address` 字段。
  final String duoqianAddress;

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

  /// 联合投票中的机构权重。
  int get jointVoteWeight {
    switch (orgType) {
      case OrgType.nrc:
        return 19;
      case OrgType.prc:
      case OrgType.prb:
        return 1;
      default:
        return 0;
    }
  }
}

/// 链上联合投票总票数。
int get jointVoteTotal =>
    19 + kProvincialCouncils.length + kProvincialBanks.length;

/// 链上联合投票立即通过阈值。
const int jointVotePassThreshold = 105;

/// 通过 48 字节 InstitutionPalletId 反查机构信息。
/// shenfenId UTF-8 右补零到 48 字节后与 palletIdBytes 比较。
InstitutionInfo? findInstitutionByPalletId(List<int> palletIdBytes) {
  if (palletIdBytes.length != 48) return null;
  for (final inst in [
    ...kNationalCouncil,
    ...kProvincialCouncils,
    ...kProvincialBanks
  ]) {
    final encoded = _shenfenIdToFixed48(inst.shenfenId);
    if (_bytesEqual(encoded, palletIdBytes)) return inst;
  }
  return null;
}

List<int> _shenfenIdToFixed48(String shenfenId) {
  final utf8Bytes = shenfenId.codeUnits;
  final result = List<int>.filled(48, 0);
  for (var i = 0; i < utf8Bytes.length && i < 48; i++) {
    result[i] = utf8Bytes[i];
  }
  return result;
}

bool _bytesEqual(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

/// 国储会（1 个）。
const List<InstitutionInfo> kNationalCouncil = [
  InstitutionInfo(
    name: '国家储备委员会',
    shenfenId: 'GFR-LN001-CB0C-617776487-20260222',
    orgType: OrgType.nrc,
    duoqianAddress:
        'a4dcfcee4629dbd67ebcb271aadf2d79b3b0b72c133156c57f136426b819216e',
  ),
];

/// 省储会（43 个）。
const List<InstitutionInfo> kProvincialCouncils = [
  InstitutionInfo(
    name: '中枢省储备委员会',
    shenfenId: 'GFR-ZS001-CB0X-464088047-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '005860c65dfa43d1efd730560d35fdab296841cfce863039614a690ddd456860',
  ),
  InstitutionInfo(
    name: '岭南省储备委员会',
    shenfenId: 'GFR-LN002-CB0Q-850177236-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '979ddbbac4c3df93e37b410999ff614265d8c5295faa705e795525405b10b8ea',
  ),
  InstitutionInfo(
    name: '广东省储备委员会',
    shenfenId: 'GFR-GD000-CB0O-261883838-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '58438c61071a1a52a24b01f414bd5f30c2d01b749f0fc0d7dee628d8a734bf3b',
  ),
  InstitutionInfo(
    name: '广西省储备委员会',
    shenfenId: 'GFR-GX000-CB0X-936039238-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'bf2d2a5bcfdf09556a8c8bce39831f466a7538372231505bd6426a92a1a6e9b6',
  ),
  InstitutionInfo(
    name: '福建省储备委员会',
    shenfenId: 'GFR-FJ000-CB0I-232415560-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '27e246c446b60d8503e393e1e49ec554cd48bc3ec68df74a20c0b776a04c8cea',
  ),
  InstitutionInfo(
    name: '海南省储备委员会',
    shenfenId: 'GFR-HN000-CB04-832186703-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '72142867d115388200dbd0f8d6279b6c96bf6399d7bf09a691d513e49a104689',
  ),
  InstitutionInfo(
    name: '云南省储备委员会',
    shenfenId: 'GFR-YN000-CB0G-574048259-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'ca96f91555a850e99e0f1f62ec4937d69ef52ebf88dd4b501f9d4298e9104dc6',
  ),
  InstitutionInfo(
    name: '贵州省储备委员会',
    shenfenId: 'GFR-GZ000-CB03-700488596-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '35b4b1bee060112b348478f77e4075be5ec2d969e313ebfd9b26cf519390d05a',
  ),
  InstitutionInfo(
    name: '湖南省储备委员会',
    shenfenId: 'GFR-HU000-CB0V-865805553-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'b49be3e53ffc0086f74aa4080d49600a6de3a43229d00811e1ce513624ac96f5',
  ),
  InstitutionInfo(
    name: '江西省储备委员会',
    shenfenId: 'GFR-JX000-CB09-183645800-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '0950cef8244e929f363946110a75d91e00671cb14e5e67b145d42d4826e0be9b',
  ),
  InstitutionInfo(
    name: '浙江省储备委员会',
    shenfenId: 'GFR-ZJ000-CB0Y-452554562-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '45b50263c9438e8642932bc23c1c5d86ec72dd42adcb1dea95e8204e6922dde4',
  ),
  InstitutionInfo(
    name: '江苏省储备委员会',
    shenfenId: 'GFR-JS000-CB0T-266669398-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'fcd48c7f4357b0bc6419cf3be4adbe83f9e2bd59003367ecfa7ae171e422e930',
  ),
  InstitutionInfo(
    name: '山东省储备委员会',
    shenfenId: 'GFR-SD000-CB0A-354794960-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '979570fa62d1963802150c9ed4c75ebde4f223db00420e624f11a08403a3a6cd',
  ),
  InstitutionInfo(
    name: '山西省储备委员会',
    shenfenId: 'GFR-SX000-CB0T-700141630-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '0f2a278947e933750b3cc14c9613299c7670b95dfd8ef719f9de56d290495122',
  ),
  InstitutionInfo(
    name: '河南省储备委员会',
    shenfenId: 'GFR-HE000-CB0R-527771281-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'b0f272d9ac4caeb41f463549732bbeddce3e0bf422450f5ab2627b684cb2e24b',
  ),
  InstitutionInfo(
    name: '河北省储备委员会',
    shenfenId: 'GFR-HB000-CB04-025532397-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '216ad2c3fd9715de1ae1854fd4216b3fe6f9245767575fbd855b80c87060c664',
  ),
  InstitutionInfo(
    name: '湖北省储备委员会',
    shenfenId: 'GFR-HI000-CB0M-247491104-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'd43bde789ab9b4fa011ac54fcec77047928de324e812b12be7c3d611f107c637',
  ),
  InstitutionInfo(
    name: '陕西省储备委员会',
    shenfenId: 'GFR-SI000-CB0Q-626717092-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '037afa9fa24097b480ef7d35c142e874f3ac78139cd9edfd20fca3ab0e483986',
  ),
  InstitutionInfo(
    name: '重庆省储备委员会',
    shenfenId: 'GFR-CQ001-CB00-452250444-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'fca5f44d8fe158205bb9adb859adf60f4683ec0ac0c122677517914ed220b753',
  ),
  InstitutionInfo(
    name: '四川省储备委员会',
    shenfenId: 'GFR-SC000-CB0N-676087668-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '7b0e36626b4906b36fe60cbc22376deae4b2b6b25f1dc48447cb1339a63be972',
  ),
  InstitutionInfo(
    name: '甘肃省储备委员会',
    shenfenId: 'GFR-GS000-CB02-451145443-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '86afdddf3d531f775fd46b5b6aca115bc281d06b16434b188f44a5b6e758796c',
  ),
  InstitutionInfo(
    name: '北平省储备委员会',
    shenfenId: 'GFR-BP001-CB0C-164347900-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'db80eef695bef0ef0268059a027b4d0641a4d59a11d562f1a53cd2c3587aca59',
  ),
  InstitutionInfo(
    name: '海滨省储备委员会',
    shenfenId: 'GFR-HA000-CB02-156526094-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'e58770d249bd55f63eb052e93b54557e4d565feebc284f6bb8398b238af30529',
  ),
  InstitutionInfo(
    name: '松江省储备委员会',
    shenfenId: 'GFR-SJ000-CB0A-005282342-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'd8c2177ef57b4ca651460f233cc39f7af405a5442937026a697cc4852e56e2d8',
  ),
  InstitutionInfo(
    name: '龙江省储备委员会',
    shenfenId: 'GFR-LJ000-CB0A-105584375-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '4bcca0d178ed251c23391f34d9c72214af1656c5431dbbbf8e191785a9b0d0a0',
  ),
  InstitutionInfo(
    name: '吉林省储备委员会',
    shenfenId: 'GFR-JL000-CB0T-855212821-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '9c52a4de06b27c9cca3fb4b8f2a1794f2dfdc0ee09a8a0286041218075e9be00',
  ),
  InstitutionInfo(
    name: '辽宁省储备委员会',
    shenfenId: 'GFR-LI000-CB03-221473214-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '69f2eb3f9f161ef9f469010acec759e40a9e8974fbf43249149472ed68bf43c4',
  ),
  InstitutionInfo(
    name: '宁夏省储备委员会',
    shenfenId: 'GFR-NX000-CB0A-240866560-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'e8f661615592fe19d33a8424d61b647ccdd7c4244349484d651e4851680caf27',
  ),
  InstitutionInfo(
    name: '青海省储备委员会',
    shenfenId: 'GFR-QH000-CB0N-229555853-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '5fbca2c6f277e9382747bdbbdfc170c3f83d563d3acd1a4fec3aa7ff81aca71b',
  ),
  InstitutionInfo(
    name: '安徽省储备委员会',
    shenfenId: 'GFR-AH000-CB0Q-714959233-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '53aa5754796f98f8f6fb74f0302ea381936b6b06d48c17e455bc64725e8af35b',
  ),
  InstitutionInfo(
    name: '台湾省储备委员会',
    shenfenId: 'GFR-TW000-CB0U-188063480-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'fe6d7dcc07faaae8face0c0fdd66de2933dea83d9bd0df25bd571979bdd55859',
  ),
  InstitutionInfo(
    name: '西藏省储备委员会',
    shenfenId: 'GFR-XZ000-CB0R-085197231-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'f3e4b26435892b5e0330028690498f309dc5eaec1ba91942cc0902d13c71a4df',
  ),
  InstitutionInfo(
    name: '新疆省储备委员会',
    shenfenId: 'GFR-XJ000-CB0I-803866647-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'a809b8e77ad103708a77b3be1d2277555eedbf0d433f436f9901d46bdb217c79',
  ),
  InstitutionInfo(
    name: '西康省储备委员会',
    shenfenId: 'GFR-XK000-CB0B-810391358-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        'f4937d7a2c61c57cdf5079d25e0d9ff8e189b668a98b0489ab946e065a6c1c63',
  ),
  InstitutionInfo(
    name: '阿里省储备委员会',
    shenfenId: 'GFR-AL000-CB08-769336671-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '969316fc4c788f7c9e1b96cd6a33ade8f40acd759b353502f64b3a3427e569c1',
  ),
  InstitutionInfo(
    name: '葱岭省储备委员会',
    shenfenId: 'GFR-CL000-CB0Z-914234080-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '6e08fcbf5a5c3429b5c408da8b8bc558feb9581ab50b758cd5c89fd7c1db3263',
  ),
  InstitutionInfo(
    name: '天山省储备委员会',
    shenfenId: 'GFR-TS000-CB0O-063508625-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '6ce2b03f2b129a204f332da81a61b1248f53efbf08848a77a6fa39ddd3c2b8b2',
  ),
  InstitutionInfo(
    name: '河西省储备委员会',
    shenfenId: 'GFR-HX000-CB0J-238307168-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '584dc4763c2a9998f137b96e55a9984e3ccb4436aefed3667b5ee33ae4f7b9d1',
  ),
  InstitutionInfo(
    name: '昆仑省储备委员会',
    shenfenId: 'GFR-KL000-CB00-453003140-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '51041527a777faa5df81ea521fd19b1981712c9bff15056fa44fd0de2696c20e',
  ),
  InstitutionInfo(
    name: '河套省储备委员会',
    shenfenId: 'GFR-HT000-CB0F-763975330-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '44a0d06f571743e1a513d28dad6e6609445451f23c6929387372f0dc9bd761d3',
  ),
  InstitutionInfo(
    name: '热河省储备委员会',
    shenfenId: 'GFR-RH000-CB0T-258553387-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '7a2703df0624d7d7afab04a169dd04ef9a89991ee76f059c586aaf376437e653',
  ),
  InstitutionInfo(
    name: '兴安省储备委员会',
    shenfenId: 'GFR-XA000-CB0D-997757073-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '3a4d16f29220b431fd778bba9ff0d0b1e1ee8958e3b36fb22512160d6b4eca0f',
  ),
  InstitutionInfo(
    name: '合江省储备委员会',
    shenfenId: 'GFR-HJ000-CB0C-544834501-20260222',
    orgType: OrgType.prc,
    duoqianAddress:
        '8ce152ac8c86e441ebcba60f515d5530492b42d9eb3335d99b526471a76d3495',
  ),
];

/// 省储行（43 个）。
const List<InstitutionInfo> kProvincialBanks = [
  InstitutionInfo(
    name: '中枢省公民储备银行',
    shenfenId: 'SFR-ZS001-CH1Z-572590896-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'fe45d3e78fd7dce6e13715a3e30ffc52ee80551d5f40e68ef4c501c3c2985ab1',
  ),
  InstitutionInfo(
    name: '岭南省公民储备银行',
    shenfenId: 'SFR-LN001-CH1D-067241191-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '6f26889bc70faa896c2fc464c0c4a4da1cd3f3df1f4347c0d56edf9e3883dc71',
  ),
  InstitutionInfo(
    name: '广东省公民储备银行',
    shenfenId: 'SFR-GD000-CH1S-539766913-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'cffd5c331e9323b1fd5b3724a3b35804bba9492e60b63a2353c857c585e2fd63',
  ),
  InstitutionInfo(
    name: '广西省公民储备银行',
    shenfenId: 'SFR-GX000-CH17-770836097-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'df01f593daed649ebaaa8b658dd127c792c02b41df515b18df05cccb483787ee',
  ),
  InstitutionInfo(
    name: '福建省公民储备银行',
    shenfenId: 'SFR-FJ000-CH1Y-285514007-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'bec1ed0746ea6e6e24db89750fb44a76a289556ca65c84e425c0b448205e18e8',
  ),
  InstitutionInfo(
    name: '海南省公民储备银行',
    shenfenId: 'SFR-HN000-CH1W-701494632-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'da92404c22e9f2d52253e737ced41bd1cdbe83c18df0ffaed5408fd1221cae53',
  ),
  InstitutionInfo(
    name: '云南省公民储备银行',
    shenfenId: 'SFR-YN000-CH1M-088552001-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '2dbe1db434c63c032aac0772681f457506c1c022e8f43ab0d656a5f0d9e611d2',
  ),
  InstitutionInfo(
    name: '贵州省公民储备银行',
    shenfenId: 'SFR-GZ000-CH17-073795499-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'e743674d50fd8cac955958b9dd1f46b0fd92bf18be5f709de6e75c9c9b13b681',
  ),
  InstitutionInfo(
    name: '湖南省公民储备银行',
    shenfenId: 'SFR-HU000-CH1P-721228492-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '54e7e17e7b493ba360e8035f86976a5e7deef2833738fd41ba955b8794022c73',
  ),
  InstitutionInfo(
    name: '江西省公民储备银行',
    shenfenId: 'SFR-JX000-CH1T-532829662-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'c5f77d6ecc1bc1e2bfe144754355ae24b7f5b0909f15705914de87d7e6382e6b',
  ),
  InstitutionInfo(
    name: '浙江省公民储备银行',
    shenfenId: 'SFR-ZJ000-CH19-249528657-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'a97dfa62d5eca6d2f1bded65fa6528c6372e2bf34f740181beb7b5c8e5e4cc77',
  ),
  InstitutionInfo(
    name: '江苏省公民储备银行',
    shenfenId: 'SFR-JS000-CH1C-191178842-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'c7fc95907a57f04c07869d4e181d17f46393e7c1224f6d2ebf16ddfec348310d',
  ),
  InstitutionInfo(
    name: '山东省公民储备银行',
    shenfenId: 'SFR-SD000-CH1V-887886640-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '98d016ea45313719d30d171932500168ed9e3de37fa07ee9f9f6f977fdba0f79',
  ),
  InstitutionInfo(
    name: '山西省公民储备银行',
    shenfenId: 'SFR-SX000-CH1F-755750488-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '735599f633072eff9cc2074520a5db9e5aa4afdfda5d0ec2dd925b0c0c14b2a1',
  ),
  InstitutionInfo(
    name: '河南省公民储备银行',
    shenfenId: 'SFR-HE000-CH1T-357503840-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '736b13ab5bd7242d880e95507a2068d05a5ae6cd78dc72bc5d44c3f474e724d6',
  ),
  InstitutionInfo(
    name: '河北省公民储备银行',
    shenfenId: 'SFR-HB000-CH12-172598053-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'e08397c483d8962e6aea1d2ebf18ae39f7291f8918fd918eba32de54ad50c394',
  ),
  InstitutionInfo(
    name: '湖北省公民储备银行',
    shenfenId: 'SFR-HI000-CH1W-584177104-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '98d151fde59630b63b99ba5c9aa56389247ece26689b432d9ebe7baddd7d8191',
  ),
  InstitutionInfo(
    name: '陕西省公民储备银行',
    shenfenId: 'SFR-SI000-CH1G-814942227-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '58c0b0ea8fb4fa430de47c4d70030645ac3a4f464728ab1e7ab304669403a732',
  ),
  InstitutionInfo(
    name: '重庆省公民储备银行',
    shenfenId: 'SFR-CQ001-CH1A-811483361-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '072abcf96cb315ab1c654a482172429314f9f15b126c1f51d2bf1ef233e03d1f',
  ),
  InstitutionInfo(
    name: '四川省公民储备银行',
    shenfenId: 'SFR-SC000-CH19-320507619-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'e104ec87a747420fc31702551d8153f0edf0a7ac2a77a5bfe8910adc3f8b0ae9',
  ),
  InstitutionInfo(
    name: '甘肃省公民储备银行',
    shenfenId: 'SFR-GS000-CH1U-319639307-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'ea360306c0190de49513faede894fc44f827960fa8f45b33be9093800d104791',
  ),
  InstitutionInfo(
    name: '北平省公民储备银行',
    shenfenId: 'SFR-BP001-CH19-330141933-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '5b9005b8abfb70803e2b0fdbd31e494044f09e8f3bd369abbafdeb481c0e148a',
  ),
  InstitutionInfo(
    name: '滨海省公民储备银行',
    shenfenId: 'SFR-HA000-CH1N-832919801-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '3670a84c5f8a3d0e710e881d59113df7f3d8694532be797c9415e0bdd5d25a3a',
  ),
  InstitutionInfo(
    name: '松江省公民储备银行',
    shenfenId: 'SFR-SJ000-CH17-991726244-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '6842bab4d4c88d0508255d1f6e768262c1dffe5b6f31470757bebbaab37990bb',
  ),
  InstitutionInfo(
    name: '龙江省公民储备银行',
    shenfenId: 'SFR-LJ000-CH1U-321069400-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'b30f35b0013af60c12cda5c17b997957412a090e2b49987cfba291778774bd92',
  ),
  InstitutionInfo(
    name: '吉林省公民储备银行',
    shenfenId: 'SFR-JL000-CH1Z-114671562-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '9ee95711f6dc002676e3da8dc1cb9bf88669b9e12e5349636e0f2700560c0c21',
  ),
  InstitutionInfo(
    name: '辽宁省公民储备银行',
    shenfenId: 'SFR-LI000-CH1O-060821950-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'b53e53d962f192c2f081f88bbce75267ff5dd344ed94ea8220b1dfd6e4467882',
  ),
  InstitutionInfo(
    name: '宁夏省公民储备银行',
    shenfenId: 'SFR-NX000-CH1W-927112322-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '5fae794dac4b836be2dd0827f47a84f11531b9447cf6f9ffd1ac770abfda9243',
  ),
  InstitutionInfo(
    name: '青海省公民储备银行',
    shenfenId: 'SFR-QH000-CH15-480036803-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'b656dbc26f915cc6d5b872f57aaa9a6a4cb80fb899bdbe8e2e60d1a3e18a3f21',
  ),
  InstitutionInfo(
    name: '安徽省公民储备银行',
    shenfenId: 'SFR-AH000-CH14-243470490-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'efc6292e20288623f6cfe838abde86b9fe132018393717e5af3ad2f46b17b895',
  ),
  InstitutionInfo(
    name: '台湾省公民储备银行',
    shenfenId: 'SFR-TW000-CH1O-339827620-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'b2c055b85357313c990832ef61ac3d9fd1b52476a8671c23a14ca7edd6302e1b',
  ),
  InstitutionInfo(
    name: '西藏省公民储备银行',
    shenfenId: 'SFR-XZ000-CH1A-076183922-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '83d7ecc0558c66037fb4bf0b32e03ac152c44ad19c40f22f2271bcb5c5b441db',
  ),
  InstitutionInfo(
    name: '新疆省公民储备银行',
    shenfenId: 'SFR-XJ000-CH1T-624864385-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '1024ef3049018c3e045d7025bcb1db301b50dee6c3c6a42259191351b988cb3a',
  ),
  InstitutionInfo(
    name: '西康省公民储备银行',
    shenfenId: 'SFR-XK000-CH19-727906387-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '817ecdb4588004991fb7ee6cdc27c212641b278fc0b2b47cccd0ce47ba0c12ca',
  ),
  InstitutionInfo(
    name: '阿里省公民储备银行',
    shenfenId: 'SFR-AL000-CH1Z-823361903-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '30990485af39af3e37e3d802319e929091adf4c75b5848dea5de19fee495393e',
  ),
  InstitutionInfo(
    name: '葱岭省公民储备银行',
    shenfenId: 'SFR-CL000-CH1I-930688147-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '00363eb57b4ed7e22ae0f1b11f58f7d8eb17d5d9899967b160682133ca88af1c',
  ),
  InstitutionInfo(
    name: '天山省公民储备银行',
    shenfenId: 'SFR-TS000-CH1S-351739678-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '72df5a28d36d27568996779bc43b043d4fc91c31d9000e8affa2b15475aa0448',
  ),
  InstitutionInfo(
    name: '河西省公民储备银行',
    shenfenId: 'SFR-HX000-CH1X-115163356-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '043cfa9fabcd16c21b55bedd2dd88fae917f25224ba12f4ea0837fae1e4407d4',
  ),
  InstitutionInfo(
    name: '昆仑省公民储备银行',
    shenfenId: 'SFR-KL000-CH1F-853206078-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'bec71276f83ca65b5fe38748f93540e3b8c935b4c0c219813f40b4a524e87380',
  ),
  InstitutionInfo(
    name: '河套省公民储备银行',
    shenfenId: 'SFR-HT000-CH1H-294801127-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '3f9f61de83c84bdd9cdc723c878135316fbd88eb9b9a98d91ff389ddb887c4b0',
  ),
  InstitutionInfo(
    name: '热河省公民储备银行',
    shenfenId: 'SFR-RH000-CH14-762808938-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'e8fc4c4266531ac8e056f16458edcccf56e74a5e766e068a23ed95a60a832af8',
  ),
  InstitutionInfo(
    name: '兴安省公民储备银行',
    shenfenId: 'SFR-XA000-CH1P-285320269-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        'e0a70ce7e5ae81e8f95f1510ebfa72da10d73116ed49249ea1cc6c96b4773e3c',
  ),
  InstitutionInfo(
    name: '合江省公民储备银行',
    shenfenId: 'SFR-HJ000-CH1C-538936570-20260222',
    orgType: OrgType.prb,
    duoqianAddress:
        '8907191cf2c30e055072de592c2d29ee5539d13260e23f41f0081c50f845464d',
  ),
];
