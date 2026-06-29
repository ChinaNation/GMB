part of 'governance_registry.dart';

// 本文件由 scripts/generate_citizenapp_governance_registry.mjs 自动生成。
// 中文注释：治理机构中英全称/简称、cid_number 和制度账户来自 runtime primitives；管理员必须动态读取链上 AdminsChange::AdminAccounts。

/// 国储会（1 个）。
const List<InstitutionInfo> kNationalCouncil = [
  InstitutionInfo(
    cidFullName: '国家公民储备委员会',
    cidShortName: '国储会',
    cidFullNameEn: 'National Citizen Reserve Committee',
    cidShortNameEn: 'National Reserve Committee',
    cidNumber: 'LN001-NRC0G-944805165-2026',
    orgType: OrgType.nrc,
    accounts: InstitutionAccounts(
      mainAccount:
          'b38e86de933984b3a6b4190fc9d4b020ff44b38471a8a65bbf95b440e05c5153',
      feeAccount:
          '7c0c099ee4df10c5bd3f618ddf132b6d15390fa27d2c1369f70aeb6b5f3907e5',
      safetyFundAccount:
          'd78abac2e0a7772e72ba663313718e97288377d9ca2ca1467c710058f8b5effa',
      heAccount:
          '4ac779852c175087c445c35efecfef3ce6e0232702152ea2283f0b5ec3952e53',
    ),
  ),
];

/// 省储会（43 个）。
const List<InstitutionInfo> kProvincialCouncils = [
  InstitutionInfo(
    cidFullName: '中枢省公民储备委员会',
    cidShortName: '中枢省储会',
    cidFullNameEn: 'Zhongshu Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Zhongshu Provincial Reserve Committee',
    cidNumber: 'ZS001-PRC0E-016974075-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '65c057a38041753f31f1d891f4d1ce79326291cb4d340a125dd7dc33710783dd',
      feeAccount:
          '54bad80b12cedbf7a1569fb96d18d90c4793949a356eb16c6304841af81001dd',
    ),
  ),
  InstitutionInfo(
    cidFullName: '岭南省公民储备委员会',
    cidShortName: '岭南省储会',
    cidFullNameEn: 'Lingnan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Lingnan Provincial Reserve Committee',
    cidNumber: 'LN001-PRC05-773405642-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'eabaa8213fc5431b9496ca5986b640ff327ce3cee0200877263ff338829380c0',
      feeAccount:
          'd9ef67d0e814fa3a927b9e6a0ebb7dbf6c15d669fc3067600adcab0016ea1cdb',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广东省公民储备委员会',
    cidShortName: '广东省储会',
    cidFullNameEn: 'Guangdong Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Guangdong Provincial Reserve Committee',
    cidNumber: 'GD001-PRC0V-067440774-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '28cc977b04b6d883084cb7569297c081759cccbffa6b9f85752989a3c137d220',
      feeAccount:
          '9d19ff83985b2bd8d30b2845d9d4c7ab90083f98cfe127a0e2297cb30cbc0085',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广西省公民储备委员会',
    cidShortName: '广西省储会',
    cidFullNameEn: 'Guangxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Guangxi Provincial Reserve Committee',
    cidNumber: 'GX001-PRC0C-663454043-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'cc9cf90ab2016f3a261b362cd2d29f795c2e8edf96bd8a62bc3b099e24ee8939',
      feeAccount:
          'f6d5db67d791d8a7f2b35d00cf2e1f72ce1277d18bcb1addf04c8e97a9bb452b',
    ),
  ),
  InstitutionInfo(
    cidFullName: '福建省公民储备委员会',
    cidShortName: '福建省储会',
    cidFullNameEn: 'Fujian Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Fujian Provincial Reserve Committee',
    cidNumber: 'FJ001-PRC0I-389570546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '39c82e31525167083bbb1a0476378e41df59ccb7687a3770dd95a9c3b971fc4d',
      feeAccount:
          '1431de61e3e80e5c7830d475cb6587cb7dde89c050c90aa1b40672a8e39b0c36',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海南省公民储备委员会',
    cidShortName: '海南省储会',
    cidFullNameEn: 'Hainan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hainan Provincial Reserve Committee',
    cidNumber: 'HN001-PRC0S-545676096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '0f1bd1f235b0943805a17013e664ac435d8f47862280a18c3cb532299e236887',
      feeAccount:
          'cd5f16df4e71f6bf5cf9080d0576730f8f3a17f7a7e4a291214f6ad3ac656941',
    ),
  ),
  InstitutionInfo(
    cidFullName: '云南省公民储备委员会',
    cidShortName: '云南省储会',
    cidFullNameEn: 'Yunnan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Yunnan Provincial Reserve Committee',
    cidNumber: 'YN001-PRC0W-145427171-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '209b4aca30dbe12799159187306172ac624585a7f2a59eea4580c8f9701687bc',
      feeAccount:
          '96b5f59842dffd1ca7f27cf004cbaff3be5afd02cf1935fe52670d0c858f0d57',
    ),
  ),
  InstitutionInfo(
    cidFullName: '贵州省公民储备委员会',
    cidShortName: '贵州省储会',
    cidFullNameEn: 'Guizhou Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Guizhou Provincial Reserve Committee',
    cidNumber: 'GZ001-PRC02-969970096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'cb89f5f626b4f9064075cb31c540c02424150f37c23d9b965e83e97596047915',
      feeAccount:
          'c25d8cf44c827852777ee62f5ea0ecccb539b2cac04628c2277aa0ab17101002',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖南省公民储备委员会',
    cidShortName: '湖南省储会',
    cidFullNameEn: 'Hunan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hunan Provincial Reserve Committee',
    cidNumber: 'HU001-PRC0P-400319700-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '36068adc23dbd140a9ddadf3da1e6d4ac11e543613b54a2783d74551d043163a',
      feeAccount:
          '0de24adffa51d31085f41f30c96ef92aa979181e6abd22ea53567a26048e4d40',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江西省公民储备委员会',
    cidShortName: '江西省储会',
    cidFullNameEn: 'Jiangxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Jiangxi Provincial Reserve Committee',
    cidNumber: 'JX001-PRC0J-458681566-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '0bcda61fd2ae9471520a29eb5fed6740cc35a775ce966094e110509d2084bc33',
      feeAccount:
          '26ce33ca747e1e51d102c1c70708a6e73b9ab57d1edfa2889e923eff06575592',
    ),
  ),
  InstitutionInfo(
    cidFullName: '浙江省公民储备委员会',
    cidShortName: '浙江省储会',
    cidFullNameEn: 'Zhejiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Zhejiang Provincial Reserve Committee',
    cidNumber: 'ZJ001-PRC08-471270801-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '51b0c01287085ea84a21f1e3b5eecf3c013c9381ee168ca33aff2d8546326a83',
      feeAccount:
          'b8eb8e8b5df07eb15c9bb5f7b0bb371683ef263646f8a5cd4f6637ee38b4fea3',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江苏省公民储备委员会',
    cidShortName: '江苏省储会',
    cidFullNameEn: 'Jiangsu Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Jiangsu Provincial Reserve Committee',
    cidNumber: 'JS001-PRC0O-358467174-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '00b73850de0f0b89a8d182a8d00dfdf5e255b2a7ef03f34399e6c5c85c2f0b5a',
      feeAccount:
          '1de52f16c1be56a716209b1e3f6c79deeb00652b4afe16d0cb24725b9b274b4d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山东省公民储备委员会',
    cidShortName: '山东省储会',
    cidFullNameEn: 'Shandong Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Shandong Provincial Reserve Committee',
    cidNumber: 'SD001-PRC07-027328848-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'c5300b9510f68294315db7121b03792ffbf0ea29602bbb89a83b25089aa1c71a',
      feeAccount:
          'b83a2219f9b5ca85685cfb7392ecda03ac443592d73d6d56241f719baf3b561f',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山西省公民储备委员会',
    cidShortName: '山西省储会',
    cidFullNameEn: 'Shanxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Shanxi Provincial Reserve Committee',
    cidNumber: 'SX001-PRC0O-104465679-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a8d0ffd86a860a00315937c7400d2e14180d46377c9ccabd78dbc884027db80c',
      feeAccount:
          'b30c88f3752ed770d120acc2481f0357f9b4bb052058fea819cdd3eff36e5526',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河南省公民储备委员会',
    cidShortName: '河南省储会',
    cidFullNameEn: 'Henan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Henan Provincial Reserve Committee',
    cidNumber: 'HE001-PRC0S-849245626-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a4b2276375e4c2fe44cb920f5173bf225a82ae35f80378305e434d8859e1e387',
      feeAccount:
          '0df1ae8648f0617c96f42a108803622a319ce3e150efecc50c8e5d57d560e22a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河北省公民储备委员会',
    cidShortName: '河北省储会',
    cidFullNameEn: 'Hebei Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hebei Provincial Reserve Committee',
    cidNumber: 'HB001-PRC0W-499533387-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '831e79e20fe67bca7c2e5bbd3f26586814b080287049f8fd7bc656a379ce972e',
      feeAccount:
          'be5e60c9c96cdc7e65a570172a7f2a9467e0b85e2790fdc411af12c89d71aebf',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖北省公民储备委员会',
    cidShortName: '湖北省储会',
    cidFullNameEn: 'Hubei Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hubei Provincial Reserve Committee',
    cidNumber: 'HI001-PRC0D-659443961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a4b1711a1bf598dd7952a9c44bb7e321fac66b50e3fc618eef6b9b10f82499ea',
      feeAccount:
          '7ff4742a43efa3fd02d94006d4ce838c7eb380a10423eee040b84531603b9614',
    ),
  ),
  InstitutionInfo(
    cidFullName: '陕西省公民储备委员会',
    cidShortName: '陕西省储会',
    cidFullNameEn: 'Shaanxi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Shaanxi Provincial Reserve Committee',
    cidNumber: 'SI001-PRC0T-711309909-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'd38bb19ed5a279325ba65271d177c67271b2374c46b12fe36ccb369533f0a62e',
      feeAccount:
          '00d09d3a723fb14c435ff3c474fbe333ca247efea81b84f765f45558c950063c',
    ),
  ),
  InstitutionInfo(
    cidFullName: '重庆省公民储备委员会',
    cidShortName: '重庆省储会',
    cidFullNameEn: 'Chongqing Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Chongqing Provincial Reserve Committee',
    cidNumber: 'CQ001-PRC06-478472058-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '19dfc75f5e87bffc9ab3010841798eb7a00164630379cafc3aa96ed70f4140d7',
      feeAccount:
          '9c1a3f35ec63f61cc350e431895c307c3c4b251abffc511d5524916816963e72',
    ),
  ),
  InstitutionInfo(
    cidFullName: '四川省公民储备委员会',
    cidShortName: '四川省储会',
    cidFullNameEn: 'Sichuan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Sichuan Provincial Reserve Committee',
    cidNumber: 'SC001-PRC0Y-935659021-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '8ba74cb8ee744c79d97e2a3f989fb2c5812dccbc596b881a7a8adfd4b44fd235',
      feeAccount:
          '0267991ba2e7345703c28470cf721522ca1f38fe83e6452303efaa137517c71e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '甘肃省公民储备委员会',
    cidShortName: '甘肃省储会',
    cidFullNameEn: 'Gansu Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Gansu Provincial Reserve Committee',
    cidNumber: 'GS001-PRC0L-679051155-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '9f99c515ce0c9a4bb7a62f3c86e0de70961d28546218d6150418470ccf5af672',
      feeAccount:
          '49e3f7a807f2ff0cababe59d1c31279e4c50687b81485c025b78550f00ff83b4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '北平省公民储备委员会',
    cidShortName: '北平省储会',
    cidFullNameEn: 'Beiping Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Beiping Provincial Reserve Committee',
    cidNumber: 'BP001-PRC0R-189323546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'c56011ce29bf37ead2a4fa84b4387506745221b4756a960b1c08f35b117bbb6e',
      feeAccount:
          'f43653d988585331dbe53e654c0b12e00ba67561f858945e99f27fbfb5c961a3',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海滨省公民储备委员会',
    cidShortName: '海滨省储会',
    cidFullNameEn: 'Haibin Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Haibin Provincial Reserve Committee',
    cidNumber: 'HA001-PRC0Y-214178517-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'bf03d6c18cc1727780c1c09c5f76c290528a1f2b7ca4901f75b553c29e743ec0',
      feeAccount:
          '4e78cc42842740c058a589b75ad9c5710d4026524371eb786514f50f50d45339',
    ),
  ),
  InstitutionInfo(
    cidFullName: '松江省公民储备委员会',
    cidShortName: '松江省储会',
    cidFullNameEn: 'Songjiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Songjiang Provincial Reserve Committee',
    cidNumber: 'SJ001-PRC09-044490898-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'b93798c43667a0d297da05e05490369758a82dbf3d748f99f1a5321fb1724663',
      feeAccount:
          '679104bd5edef4032d11cab381c94ac9ad89df4f365e8d309a965b30daba368d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '龙江省公民储备委员会',
    cidShortName: '龙江省储会',
    cidFullNameEn: 'Longjiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Longjiang Provincial Reserve Committee',
    cidNumber: 'LJ001-PRC08-279890045-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '607b16ae5b5ae4ac5feedb59b63603e17ef75f92500049fcbd62f4939939c711',
      feeAccount:
          '849442bed3bc17e958af1612cb560d4594755324df3e56b81245bf65f4076558',
    ),
  ),
  InstitutionInfo(
    cidFullName: '吉林省公民储备委员会',
    cidShortName: '吉林省储会',
    cidFullNameEn: 'Jilin Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Jilin Provincial Reserve Committee',
    cidNumber: 'JL001-PRC05-850461124-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '097d5df07098d2a4f56f2608a79d5112292ab0afb32ccbf5cedd6fbe2c3b970c',
      feeAccount:
          '91822638bcd0a5028d470bac69c8d7a9f3a10d995b291b79ef2febac76ce81a6',
    ),
  ),
  InstitutionInfo(
    cidFullName: '辽宁省公民储备委员会',
    cidShortName: '辽宁省储会',
    cidFullNameEn: 'Liaoning Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Liaoning Provincial Reserve Committee',
    cidNumber: 'LI001-PRC0T-978545133-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '35492f6d685491158c3eb212e91989b8bdb2a53c560c66ca2e14abd970c89a5e',
      feeAccount:
          'e9e22ea860eef168eaa319245d40fa255d5a3a722ae34823e66590d0bdc92684',
    ),
  ),
  InstitutionInfo(
    cidFullName: '宁夏省公民储备委员会',
    cidShortName: '宁夏省储会',
    cidFullNameEn: 'Ningxia Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Ningxia Provincial Reserve Committee',
    cidNumber: 'NX001-PRC0J-389752794-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '4fa384cc8b9b0852eb0a0e57aba444d047ce3b73fe1545f4617e9248b8fdf075',
      feeAccount:
          'a4095a3a388044d7a6ff154d4bcfb35cec384eefe8382d9e1d59710f1b16b347',
    ),
  ),
  InstitutionInfo(
    cidFullName: '青海省公民储备委员会',
    cidShortName: '青海省储会',
    cidFullNameEn: 'Qinghai Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Qinghai Provincial Reserve Committee',
    cidNumber: 'QH001-PRC0C-882026762-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '097fc5639a9ae0a26e59437934d876f6c6310b22d8a0d8b7dd5bc8088bbd30b5',
      feeAccount:
          '972fd76274a5aa2a5781ebc1f78996b89716f2a43806469641888cc8bab628fb',
    ),
  ),
  InstitutionInfo(
    cidFullName: '安徽省公民储备委员会',
    cidShortName: '安徽省储会',
    cidFullNameEn: 'Anhui Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Anhui Provincial Reserve Committee',
    cidNumber: 'AH001-PRC00-589856828-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '41d57600ee0d0d95a6c60a2e5899dc224df000448f841e06004cc4a820ddb2ac',
      feeAccount:
          '6ed62e6ed8d66b93ce7de191b31d46a4645314830ec254498c9d8e72d03a8b17',
    ),
  ),
  InstitutionInfo(
    cidFullName: '台湾省公民储备委员会',
    cidShortName: '台湾省储会',
    cidFullNameEn: 'Taiwan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Taiwan Provincial Reserve Committee',
    cidNumber: 'TW001-PRC07-265218823-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'd51c44903eeddd0de9d8ab6e2aac3ec490f08fc6210c4d899e6e30a4346b0567',
      feeAccount:
          'ee3adae7d737c734a0c167fdd553d085451f5391cafeb2baed289f11be422076',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西藏省公民储备委员会',
    cidShortName: '西藏省储会',
    cidFullNameEn: 'Xizang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xizang Provincial Reserve Committee',
    cidNumber: 'XZ001-PRC02-435616961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'e30fdd8e8a7d40d75d1dd6546062863ce50f55d2e6610910403cbfa40d2fec94',
      feeAccount:
          '7e57913cbe8fb3f5e7731a041c72a62caf210124823dd368323f0c2927070fae',
    ),
  ),
  InstitutionInfo(
    cidFullName: '新疆省公民储备委员会',
    cidShortName: '新疆省储会',
    cidFullNameEn: 'Xinjiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xinjiang Provincial Reserve Committee',
    cidNumber: 'XJ001-PRC02-671044381-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'd067dffc462f314b7aa31757aa9b155d7ed125b04e5fd666df5f2cccff5f25dc',
      feeAccount:
          '603fd39b08aa180ce23131902b6617937c1e0458f2ddd8d9fede9b91f9d5c955',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西康省公民储备委员会',
    cidShortName: '西康省储会',
    cidFullNameEn: 'Xikang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xikang Provincial Reserve Committee',
    cidNumber: 'XK001-PRC0P-695945392-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'b9901372d3ce3b2d5bcaded7025396913f6ea91858db81404b132d3bd0beb654',
      feeAccount:
          '937e6a8d5e08e283141ea99c3ee4864922fc153ed1d60a6c959a9139abba32ad',
    ),
  ),
  InstitutionInfo(
    cidFullName: '阿里省公民储备委员会',
    cidShortName: '阿里省储会',
    cidFullNameEn: 'Ali Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Ali Provincial Reserve Committee',
    cidNumber: 'AL001-PRC0D-487847725-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '2665082bb00acb14948e02b12a0d41e27af67618397d0026d8a0b4f1b4c10213',
      feeAccount:
          '9179901438d3fa33d76f7fe8578c054754954eeba2a8257df37db761265a129c',
    ),
  ),
  InstitutionInfo(
    cidFullName: '葱岭省公民储备委员会',
    cidShortName: '葱岭省储会',
    cidFullNameEn: 'Congling Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Congling Provincial Reserve Committee',
    cidNumber: 'CL001-PRC0J-771698743-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'f0fe0e639228e644e52550ef9ab7d5df73935397950a5382eaa916f921312b15',
      feeAccount:
          '45939c77d7c4a02b91777a7d5577d3934a12ac8803f66cbc2ebd4dcb83147a5a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '伊犁省公民储备委员会',
    cidShortName: '伊犁省储会',
    cidFullNameEn: 'Yili Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Yili Provincial Reserve Committee',
    cidNumber: 'YL001-PRC0Q-293160581-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '532d8ab5e781a0acb73c87269affdb673ff82c3bb1fa36a4d08d2a0339fe5001',
      feeAccount:
          '19e8cac822cc520209a1f02789732c4320eb55e319bb9ef1852d0ff3bd0306ad',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河西省公民储备委员会',
    cidShortName: '河西省储会',
    cidFullNameEn: 'Hexi Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hexi Provincial Reserve Committee',
    cidNumber: 'HX001-PRC0D-475713213-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '44d4b86c61e838f3a26321d177acb45053ad885bedf1499c2ee14e1491e3c735',
      feeAccount:
          'a31f30cbb854362a6de74a6e67b2602caa019e71cb661715edd348a2e30078a3',
    ),
  ),
  InstitutionInfo(
    cidFullName: '昆仑省公民储备委员会',
    cidShortName: '昆仑省储会',
    cidFullNameEn: 'Kunlun Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Kunlun Provincial Reserve Committee',
    cidNumber: 'KL001-PRC0O-091969119-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '79530e3f1342f793fc471f066e56e39f56cbf2a0ab995065e21d9817d718c50e',
      feeAccount:
          'c97a3d8e211609b7aa55c7f75c9e1d44c161287c69609840f3b7f296bd82d8b4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河套省公民储备委员会',
    cidShortName: '河套省储会',
    cidFullNameEn: 'Hetao Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hetao Provincial Reserve Committee',
    cidNumber: 'HT001-PRC00-481172908-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'e4d2cc7938458fb0e4e76e46aea04f217034eb3353e50b5c3d92278c4c98a943',
      feeAccount:
          'eb4cbfe90cbc0ec9f46b5a77c697e0462bde29b546d65b17e0e6fff2541c20c2',
    ),
  ),
  InstitutionInfo(
    cidFullName: '热河省公民储备委员会',
    cidShortName: '热河省储会',
    cidFullNameEn: 'Rehe Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Rehe Provincial Reserve Committee',
    cidNumber: 'RH001-PRC0F-697831866-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'b016aaf327e094d714b9478f34caaeef329c9300b2c33028885df864e6a9e6ac',
      feeAccount:
          'a68904edd0892ddb585b31bdb13711f9919f1e1bdb8a2b243ed8462295ff0787',
    ),
  ),
  InstitutionInfo(
    cidFullName: '兴安省公民储备委员会',
    cidShortName: '兴安省储会',
    cidFullNameEn: 'Xingan Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Xingan Provincial Reserve Committee',
    cidNumber: 'XA001-PRC0H-384161601-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '249c53e455d02582f64fee07d2e785958ee40652a87ea97fc443a976553b306c',
      feeAccount:
          '38b95239084a4d72f46899781fb9fa619e901e39e1f3d225fd37c3165d4a9409',
    ),
  ),
  InstitutionInfo(
    cidFullName: '合江省公民储备委员会',
    cidShortName: '合江省储会',
    cidFullNameEn: 'Hejiang Provincial Citizen Reserve Committee',
    cidShortNameEn: 'Hejiang Provincial Reserve Committee',
    cidNumber: 'HJ001-PRC0V-963948997-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'f91218a2c829417c1853b45d96604cafb0b6f5ad588f9c8d69a456f8ddd5e165',
      feeAccount:
          '13695b601d837128635740439d81e8ffb955abd2edde9efc34b870d5745f1f64',
    ),
  ),
];

/// 省储行（43 个）。
const List<InstitutionInfo> kProvincialBanks = [
  InstitutionInfo(
    cidFullName: '中枢省公民储备银行',
    cidShortName: '中枢省储行',
    cidFullNameEn: 'Zhongshu Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Zhongshu Provincial Reserve Bank',
    cidNumber: 'ZS001-PRB08-233384677-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'affa67900b4c9da9bdc52bff834c1870ee6f50507ae9d782e1c3cb1261788c11',
      feeAccount:
          'af881a7feb5ea6653fcfc1e801308b4cc0aeedf85bd0674c992830a4e2ccfc46',
      stakeAccount:
          '6a0c89dfe0fbd9d475226c9163e936f400f960bf63743f46463e75a6c159bcdb',
    ),
  ),
  InstitutionInfo(
    cidFullName: '岭南省公民储备银行',
    cidShortName: '岭南省储行',
    cidFullNameEn: 'Lingnan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Lingnan Provincial Reserve Bank',
    cidNumber: 'LN001-PRB0K-703127075-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '29a3df898a8949c4ad89f596fcafe98a5530df7ed3c75ab263d76ab9485efb74',
      feeAccount:
          '86f1e88c787286ab371cef9ca4ada192f6d98e950cba5586de3280a0ad748c33',
      stakeAccount:
          '75a2285a594c249db42fb12228c3cfc4f825e090d19373f29b9ef23741162693',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广东省公民储备银行',
    cidShortName: '广东省储行',
    cidFullNameEn: 'Guangdong Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Guangdong Provincial Reserve Bank',
    cidNumber: 'GD001-PRB0T-239565809-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '88226f81d658fb59c69023ecff02f09ce18f32bb92b1eab751a1db909f6109c0',
      feeAccount:
          '46fb2dfc71054c9d5187772dd69f4097636454a679a14c73ff20409c000de1c8',
      stakeAccount:
          'a084bfe5ddce3d77b3e947e5c08de796d38ecc6c29ef35180fec241e184b608b',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广西省公民储备银行',
    cidShortName: '广西省储行',
    cidFullNameEn: 'Guangxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Guangxi Provincial Reserve Bank',
    cidNumber: 'GX001-PRB01-025559630-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '7eba6634bdfd349d91ae3cd0e9d14a6a575fa4b583d7137ab218c7021ce59ec9',
      feeAccount:
          'aece89d4edcbc584933fc22f306dd855de67fbeb9bd13223bc7c9c4d8be490f8',
      stakeAccount:
          'a58dd0ee88d3eb2a14130c1bdf2c3a56072a3d8bcc9a05f789d396a587463486',
    ),
  ),
  InstitutionInfo(
    cidFullName: '福建省公民储备银行',
    cidShortName: '福建省储行',
    cidFullNameEn: 'Fujian Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Fujian Provincial Reserve Bank',
    cidNumber: 'FJ001-PRB0V-504679612-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'b57d82564a7e4a76168fca2a7b033c12fd72b976278540f2d6820c3f23981892',
      feeAccount:
          '329e53e43c1a68a6a31aa88705c576963e4ff230abbcea9a09be369f60af599e',
      stakeAccount:
          '589fd173a787ae33d1baca04fa120be4724835ddd3fd0d9fe56114d41b17da87',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海南省公民储备银行',
    cidShortName: '海南省储行',
    cidFullNameEn: 'Hainan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hainan Provincial Reserve Bank',
    cidNumber: 'HN001-PRB0P-723623074-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '11bb1fb6ca4ee8cfed09849bf39ac6e5493c3befb6623b11ff4eaec3140af06b',
      feeAccount:
          'e5fbc89f5366aeb575380066fc4df33f3938d1376087108787f2549cbec4a779',
      stakeAccount:
          'cc4ad3d70c61312b445c5e652d70bc8ffd35c0d8627e0f62375b26b1326ac87e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '云南省公民储备银行',
    cidShortName: '云南省储行',
    cidFullNameEn: 'Yunnan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Yunnan Provincial Reserve Bank',
    cidNumber: 'YN001-PRB08-692525950-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'bfbee97e294301823bc7e602f70cf183fed9e9fb99f75926a1931f6a6f83de6e',
      feeAccount:
          '5b2b6206e34cd3e0540b90fe44818c52ed4361e3425186889c29f186574b55aa',
      stakeAccount:
          '09d2dc993adc04d3b666db10a42f11568b843ee05105eca8984fed258d678b6b',
    ),
  ),
  InstitutionInfo(
    cidFullName: '贵州省公民储备银行',
    cidShortName: '贵州省储行',
    cidFullNameEn: 'Guizhou Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Guizhou Provincial Reserve Bank',
    cidNumber: 'GZ001-PRB00-490015860-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '0dd61056639eab106d2fc77bad61e424df5bf1ad69e39de9f757dd6db8593da0',
      feeAccount:
          '155dcbfa1fe28f6a085db3373c5a30e2bfb796713feb8e624d4e63d229f8e079',
      stakeAccount:
          '933b8fd387a7072b7bdc99ed37d110908a4e0f55e3ecf9993a12f5a78c978573',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖南省公民储备银行',
    cidShortName: '湖南省储行',
    cidFullNameEn: 'Hunan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hunan Provincial Reserve Bank',
    cidNumber: 'HU001-PRB0F-084835673-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '98e68934cbc777ebe48e4b98bb92972aa71b101764979eb56e6ea082f81e7983',
      feeAccount:
          'd6b3115e4b1aae55f1c3601a7b0612c304b8d9699cc7a9993fd182382e77ad0c',
      stakeAccount:
          'f9c1c2d5a242707780a1f431cabe22e828f12782d2757702de552d255fa21d29',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江西省公民储备银行',
    cidShortName: '江西省储行',
    cidFullNameEn: 'Jiangxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Jiangxi Provincial Reserve Bank',
    cidNumber: 'JX001-PRB09-243765987-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '5bd42ede1f285789efc25fa1379ed5c03f729d411e08f1ec8760a8d5bdb93af3',
      feeAccount:
          'de3584a03bd971117b22a746b5ce1dcea73828d76e079882bd6e0b54db126bca',
      stakeAccount:
          '7033fc1f98c21021d4650e430ed068cf46d4a93a7f1d8f14c8f35b68da5839d8',
    ),
  ),
  InstitutionInfo(
    cidFullName: '浙江省公民储备银行',
    cidShortName: '浙江省储行',
    cidFullNameEn: 'Zhejiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Zhejiang Provincial Reserve Bank',
    cidNumber: 'ZJ001-PRB0R-296232973-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'ad031950c02d7cbb6824bd00912fb53ccb45d1bfc51ecb9f26d942dee1983484',
      feeAccount:
          '4b05f4fd4e07fe55084d26a64ca7fa787681e5130dd02344851d5f1ea93e95e0',
      stakeAccount:
          'f57da45b2648193199c09269945555ce9b8f8e355339a429a23d799992a3d333',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江苏省公民储备银行',
    cidShortName: '江苏省储行',
    cidFullNameEn: 'Jiangsu Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Jiangsu Provincial Reserve Bank',
    cidNumber: 'JS001-PRB01-890774605-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '1cdaa5245d1736c2b36f35c00c4fa0b052956890ad1fd24d0027e6cdaac6ab68',
      feeAccount:
          '3f75ec35139fe076e006ebc8c0352dfbba3d1a7d484a0ff33a3984426932dd4e',
      stakeAccount:
          'ae66d540fe1dca4092f5c4df7e8c605c32152fdf9ca7af3dc861c58a9d77cf07',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山东省公民储备银行',
    cidShortName: '山东省储行',
    cidFullNameEn: 'Shandong Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Shandong Provincial Reserve Bank',
    cidNumber: 'SD001-PRB0G-114256751-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '82c4d82a8ae12e1695ae9a6fdf5939660b66ae699d07cf75a01cfed21e3604c4',
      feeAccount:
          '85f850faf1f4ace6c3169a61fbe129e2486878dae1d6380a698c2a0fcd70a564',
      stakeAccount:
          '9ab73917fca226990eddae1e6eba1b6b8b05cac60357cf1037ce65727e5db467',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山西省公民储备银行',
    cidShortName: '山西省储行',
    cidFullNameEn: 'Shanxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Shanxi Provincial Reserve Bank',
    cidNumber: 'SX001-PRB0K-520132196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '1d3bfef890601e3de0d2a541855b409ac63450e8e0681d6def5dc394a4785fdc',
      feeAccount:
          '7cb015e5de047a799f66b33bef38f2ec28505d87ffa083e741dd498677cdf5d2',
      stakeAccount:
          '71c92875a01d78a54f9e0032524435b73bc949e368e5495174eff4530f7a1c1a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河南省公民储备银行',
    cidShortName: '河南省储行',
    cidFullNameEn: 'Henan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Henan Provincial Reserve Bank',
    cidNumber: 'HE001-PRB03-158889343-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '5aa0f2525cb46a033a5e19195b5c6f42fb8b1b2c856c4fc594b119e5d5ffa669',
      feeAccount:
          '6efbdece0985faa711ad64f07bcce5e706bac2d127aedf413f8c8f577441d397',
      stakeAccount:
          '7c3a4bbff9ea5923b137e6f404c30794347869264e4d150c563083ca13c2a26b',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河北省公民储备银行',
    cidShortName: '河北省储行',
    cidFullNameEn: 'Hebei Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hebei Provincial Reserve Bank',
    cidNumber: 'HB001-PRB0Z-484022741-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '3dd7aee598e825e93cfbc8fa1312d056d2a5f6babd2b6dd94289ca9ccb3bcd4f',
      feeAccount:
          '8794b9837ea3ea84b68af195ff5815bf260fc8f085dd8eec0646d3dd17a3eac4',
      stakeAccount:
          'a1cf47deaed6648501ddcbc35d06e5d16d8a9ddbf93f69a83f30e41cf6f6f8b9',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖北省公民储备银行',
    cidShortName: '湖北省储行',
    cidFullNameEn: 'Hubei Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hubei Provincial Reserve Bank',
    cidNumber: 'HI001-PRB0V-514948302-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '544bdd28da30fe51f95f577ea24ec29ca460891f9f410126d9df35969ce2328b',
      feeAccount:
          '0405200fc6a67d2b4a5b1602ccc55009bfa0148c7f4d7e4552a7ec6b8c561f96',
      stakeAccount:
          '06ac835853610374c4ff60f6508502313ca555fc5379bddbb7f871b730e12369',
    ),
  ),
  InstitutionInfo(
    cidFullName: '陕西省公民储备银行',
    cidShortName: '陕西省储行',
    cidFullNameEn: 'Shaanxi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Shaanxi Provincial Reserve Bank',
    cidNumber: 'SI001-PRB0N-245618374-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '36bdc592735caef328dd4dcf5305b1c70ae10c9e249953ccfa7c0ff13fe45df8',
      feeAccount:
          'd3d8e94d9dc2eb0457ebb871d551e46ba749cdfaae97e295ca20356add922fda',
      stakeAccount:
          'eae9574351984389de27c3ebdef6a3a374621385b470f6a052a7cd9cff107911',
    ),
  ),
  InstitutionInfo(
    cidFullName: '重庆省公民储备银行',
    cidShortName: '重庆省储行',
    cidFullNameEn: 'Chongqing Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Chongqing Provincial Reserve Bank',
    cidNumber: 'CQ001-PRB0C-694162045-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '3f84adf51c2470eacbd97f8271e2757cfab239c2c92af90d5d50c0403489070e',
      feeAccount:
          '8e7397e21e3c7d345b68fbeb3e479936bec8f697cb83cf90b81382d6ac5f7cab',
      stakeAccount:
          '68887536644c8c402c303d930c44af6e68c505f7dfd10f2a6df97c50a99195ab',
    ),
  ),
  InstitutionInfo(
    cidFullName: '四川省公民储备银行',
    cidShortName: '四川省储行',
    cidFullNameEn: 'Sichuan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Sichuan Provincial Reserve Bank',
    cidNumber: 'SC001-PRB0Q-764253139-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'ce8454ee381e9812b4bbee291ec8d4c27e79a1b61f3ba1e01ed49909dcccca07',
      feeAccount:
          '445832e057237b6c0ca00d9ed474e9fc01d28e3f6f4c7d8cc7c4a3d61785ae69',
      stakeAccount:
          '35c3366db061dd2645a8d609ad2339865eab41fc9f847ecb3fef7fac0a6b2a1c',
    ),
  ),
  InstitutionInfo(
    cidFullName: '甘肃省公民储备银行',
    cidShortName: '甘肃省储行',
    cidFullNameEn: 'Gansu Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Gansu Provincial Reserve Bank',
    cidNumber: 'GS001-PRB08-005784877-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'c1aaf912e0b7a9d01b2d170de641f0bd632dc32e17bc0350da088e07eb49f457',
      feeAccount:
          'd36f42f0b356092a5bee8d0df2d8c44fef5e4130b543aeedb4b0a4171a0ebe10',
      stakeAccount:
          'f00c243942dbdaf80833d8500416692a9223ad5f7d89a866e7c4e38d8574932d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '北平省公民储备银行',
    cidShortName: '北平省储行',
    cidFullNameEn: 'Beiping Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Beiping Provincial Reserve Bank',
    cidNumber: 'BP001-PRB0Q-434307982-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '376487c820b127f311a1d54f8e7df5c4b58b566c006bf1c52d176c05f9af5dab',
      feeAccount:
          '5ecefffbb5c631d39618f08c952ad6bc34e0365c80ff3f73221e28bd4d6cdddf',
      stakeAccount:
          'c842a048d42b3b5eb4ac844afce9afaf2d93ef454d64cf57a4d836dcd5655070',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海滨省公民储备银行',
    cidShortName: '海滨省储行',
    cidFullNameEn: 'Haibin Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Haibin Provincial Reserve Bank',
    cidNumber: 'HA001-PRB08-969179618-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '4271f2e0bb144f2482d7c5b362f9a1a3e6f8ffa7f40333dffa8213991b8b94ee',
      feeAccount:
          '21723c4ffbdedfe3dc03f8f99f2be69d38974cefb8baaa5ae0dccb2aa19a2894',
      stakeAccount:
          'ce51ee3dfe8bdd27ea58290bbf873bbe3f1cdaf2ae9d83b5bda546ebeed2c221',
    ),
  ),
  InstitutionInfo(
    cidFullName: '松江省公民储备银行',
    cidShortName: '松江省储行',
    cidFullNameEn: 'Songjiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Songjiang Provincial Reserve Bank',
    cidNumber: 'SJ001-PRB03-644104544-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'f21546e45cf21d7b0144228eb6935f6effc7f09ee900456b675baa297e785d18',
      feeAccount:
          '3e32836b01771c6e78ca0352f272a1c9a6f94a48c2fe43eb404ed12a28b0556f',
      stakeAccount:
          'c9d1ee88617e82d574446ca791246b46b76fe13da37a503af6dc4b34725eba3e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '龙江省公民储备银行',
    cidShortName: '龙江省储行',
    cidFullNameEn: 'Longjiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Longjiang Provincial Reserve Bank',
    cidNumber: 'LJ001-PRB0T-280510636-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '383589c013a555712077984b6ee5890f50facab6360be9bfc95e77e1e3d7a227',
      feeAccount:
          '880a95a01195ecab49863ac2941da1b3e24d1498b975d8bf17fb658e27f7d355',
      stakeAccount:
          'fbbfc3cd2fb99b1939539b02cdc3ef145b82d60f35fb6a0a963cf93dcc6b2f2e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '吉林省公民储备银行',
    cidShortName: '吉林省储行',
    cidFullNameEn: 'Jilin Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Jilin Provincial Reserve Bank',
    cidNumber: 'JL001-PRB07-129935340-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '466f3a89d6e21aa32fcf63839b7453b1ebee2966653b46b6d80ad3f80801375f',
      feeAccount:
          'cfd73254a825eda01cf08614205b4175d682613604e795982af1507a40b768af',
      stakeAccount:
          '9f4cd452e0a136f55dca8a4016c122f7981b4a356a41c894c98e762cae814adb',
    ),
  ),
  InstitutionInfo(
    cidFullName: '辽宁省公民储备银行',
    cidShortName: '辽宁省储行',
    cidFullNameEn: 'Liaoning Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Liaoning Provincial Reserve Bank',
    cidNumber: 'LI001-PRB0J-249814963-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '03f6cbd31b9a5e3d71e71df902f9d79155c5c4b87699f6a330bbadeee669ec0d',
      feeAccount:
          'fe9782b1cb4961ee45717105db51d7f9dca16d87c3399125778a9140ee71b9b1',
      stakeAccount:
          'ea877c4fb6a99b92a1d670de59b6a44bec2e85270a6d945f0343fbc78c8adfbf',
    ),
  ),
  InstitutionInfo(
    cidFullName: '宁夏省公民储备银行',
    cidShortName: '宁夏省储行',
    cidFullNameEn: 'Ningxia Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Ningxia Provincial Reserve Bank',
    cidNumber: 'NX001-PRB0F-292327153-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'c9186f551d669111a28fc3208657699f7d9cd643ae954cc8e6f7041a1cfea6b8',
      feeAccount:
          '7e802a3dc6982cacc75bfa02e46bc8f1d074eb55f92acba831e5fbfdfa656627',
      stakeAccount:
          'edcb2ee9ad255d650f5e4854032822787ed7de7bc91f5b9b9e28abd6ce6bc071',
    ),
  ),
  InstitutionInfo(
    cidFullName: '青海省公民储备银行',
    cidShortName: '青海省储行',
    cidFullNameEn: 'Qinghai Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Qinghai Provincial Reserve Bank',
    cidNumber: 'QH001-PRB0V-075657014-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'bc23a48edb423a423d12d344b6f99f368c73105780b7edd7edc798e1a0cd5ee9',
      feeAccount:
          'e52d727901162c6e4cf6e03f2066e057399ad9292001820d436dabdec2ed0679',
      stakeAccount:
          'cba5c8145bd6f14b1792e91489068655df18bee79fa751c029dbdaf0539712af',
    ),
  ),
  InstitutionInfo(
    cidFullName: '安徽省公民储备银行',
    cidShortName: '安徽省储行',
    cidFullNameEn: 'Anhui Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Anhui Provincial Reserve Bank',
    cidNumber: 'AH001-PRB0M-388477914-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '2d31757a3f8e01cdf0eb620c8b8c752884b52ab7d585b58ccc48fa32eb33f410',
      feeAccount:
          '0003751d234a7a8afd21b2f5ec07f7bfc8232b20e38a7cd3f7b2c261bf164a7f',
      stakeAccount:
          'b21f3f0ac6b5ffbda92677932db40cc4901a52d7e112314b97971d7db24772a2',
    ),
  ),
  InstitutionInfo(
    cidFullName: '台湾省公民储备银行',
    cidShortName: '台湾省储行',
    cidFullNameEn: 'Taiwan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Taiwan Provincial Reserve Bank',
    cidNumber: 'TW001-PRB0S-266238196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '7116d14df4481430641f47a994ddf73fcc48adb85713772a48354c5391344aa0',
      feeAccount:
          '6e52e744891abe902a42b039276cbf0593989d4ed4806fda54d19c69a73cda14',
      stakeAccount:
          '1aab1f7f10d33e3721e011f290c725453afa732a281d5ebfcd5bdf8b86c88566',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西藏省公民储备银行',
    cidShortName: '西藏省储行',
    cidFullNameEn: 'Xizang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xizang Provincial Reserve Bank',
    cidNumber: 'XZ001-PRB06-210788637-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '7df0ae830fd51ebfdf5e37d635f967d62af07d69c4f02adcdee5ba4908ad7b0b',
      feeAccount:
          'e84daa85a070ae64eb1b7f18f75223158039349276df7e697737ee19086fe34b',
      stakeAccount:
          '885c1ee80f7cb124fa4b02c81bf8134fa86f192cb22859041e874039f132c613',
    ),
  ),
  InstitutionInfo(
    cidFullName: '新疆省公民储备银行',
    cidShortName: '新疆省储行',
    cidFullNameEn: 'Xinjiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xinjiang Provincial Reserve Bank',
    cidNumber: 'XJ001-PRB0V-233325633-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '798a2d662e9e1a5511b5b3c21bcc176b1dbb7bdfcf150608959c8bc11473e6cc',
      feeAccount:
          'e3e528582c58cb319c0941bef1354b8303d2c1a9865b18f73b107e2354cee5e6',
      stakeAccount:
          'd8dd32e8f0ef1b843b8e1088d3a3a81e617d760768e57cd5cb889b59542b9444',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西康省公民储备银行',
    cidShortName: '西康省储行',
    cidFullNameEn: 'Xikang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xikang Provincial Reserve Bank',
    cidNumber: 'XK001-PRB0Q-300401625-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '21b41b5fed2316c5067e5efe732b47dd1640590cc1f946b259a4cb0222b2fe8e',
      feeAccount:
          '0016695ce634d377fc5348f6e31bac12f89c275112bbf0683783a026e7d6e1bb',
      stakeAccount:
          'f8b0ce8c6dd9e6291063e9d1fa272d60e46248a53156d4da7309db23e3a66fe0',
    ),
  ),
  InstitutionInfo(
    cidFullName: '阿里省公民储备银行',
    cidShortName: '阿里省储行',
    cidFullNameEn: 'Ali Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Ali Provincial Reserve Bank',
    cidNumber: 'AL001-PRB0S-527686065-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'aa7e791c257bb50e85e757ced57e0378640553eea2aa2318b839383cfbfeeec7',
      feeAccount:
          'cd15753c666a0604b68cf9a2af1626e17f0a259da8db94c3c54d4a092fbeac9c',
      stakeAccount:
          'c7764f932bef9b979daad3bb8e31c38aa35cab068a13a78ab1773dff21589114',
    ),
  ),
  InstitutionInfo(
    cidFullName: '葱岭省公民储备银行',
    cidShortName: '葱岭省储行',
    cidFullNameEn: 'Congling Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Congling Provincial Reserve Bank',
    cidNumber: 'CL001-PRB0Q-951267669-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '5ff062c60c72af10aa2de0e5f3428807b03c1243df2009445f327fb337c2fa40',
      feeAccount:
          '9f5bd94ff37a8d3f04c8e90560782f5c6f20477bf813059e4d8bb3c17ae2bcc6',
      stakeAccount:
          'd971cc69417f5d1d777358c249295928ed9583cac22fdf9ca8e5f7a30a4e5f98',
    ),
  ),
  InstitutionInfo(
    cidFullName: '伊犁省公民储备银行',
    cidShortName: '伊犁省储行',
    cidFullNameEn: 'Yili Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Yili Provincial Reserve Bank',
    cidNumber: 'YL001-PRB0A-142800261-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'd14743839d5569fe06b7c6e157883a7fe6dda2d49f1127e4e5f13ec369c85e4c',
      feeAccount:
          '0da105fd6917c7703b6f357eeb7e05c445c63200eaab80efa93a66b1b286f9b6',
      stakeAccount:
          '5e44f4d61ba255aece31e1431772fe581202d9d94228de2b001da7e6b878aca4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河西省公民储备银行',
    cidShortName: '河西省储行',
    cidFullNameEn: 'Hexi Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hexi Provincial Reserve Bank',
    cidNumber: 'HX001-PRB0F-215310265-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'bffb43fa61a2ffe1aa844b281be42f2e15ddf049c130c1863f0fba6bfa4b78bd',
      feeAccount:
          '219a983e60a6e9f684632e0783374f4fec3325f46d5c7de3cb1d05dde7599193',
      stakeAccount:
          '9c32f71166acabb5c83a3f996da1799850ee81085238ab3f77769c1b9be17fd2',
    ),
  ),
  InstitutionInfo(
    cidFullName: '昆仑省公民储备银行',
    cidShortName: '昆仑省储行',
    cidFullNameEn: 'Kunlun Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Kunlun Provincial Reserve Bank',
    cidNumber: 'KL001-PRB08-682838027-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '7f89b1c5e71028b13fa06b7d9314eb85d35200a90085c6ead6a734dd28bdbc78',
      feeAccount:
          'd2f1ca163d87fcc96546ebd5891c2487f21b6fd1702de8a1182da79651cc89a7',
      stakeAccount:
          '286f18c7b374885dcd2a8c5e742180c73370cdfca2477936fd9f780c83d2fe8f',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河套省公民储备银行',
    cidShortName: '河套省储行',
    cidFullNameEn: 'Hetao Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hetao Provincial Reserve Bank',
    cidNumber: 'HT001-PRB0L-210616196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '2e42158b741722659d561874f0d27b57f456c44acf676c79fa6f10eccbed7747',
      feeAccount:
          '6c59fa9059358d7bf95f83b0c4eaaf6f2e4837a7151174e128ffd2acc8f23f39',
      stakeAccount:
          '778ba42edb6e07a03b2e23d5f4ba297d1afa5e2ed3c52c05dbba9646ddc52646',
    ),
  ),
  InstitutionInfo(
    cidFullName: '热河省公民储备银行',
    cidShortName: '热河省储行',
    cidFullNameEn: 'Rehe Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Rehe Provincial Reserve Bank',
    cidNumber: 'RH001-PRB0C-380830938-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'ad5fa77655372c35e1cf42daa8eeb44c3266b08dd0191a44924869329c1f704c',
      feeAccount:
          '8d59d4a31ab0cb729ce80b42d1d82f50d87a8557b1008a960a30892772a7d296',
      stakeAccount:
          'aece412409c5309ffbe83f67ee32d1ebb8b6192d767411d4af57d11c5122e2ab',
    ),
  ),
  InstitutionInfo(
    cidFullName: '兴安省公民储备银行',
    cidShortName: '兴安省储行',
    cidFullNameEn: 'Xingan Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Xingan Provincial Reserve Bank',
    cidNumber: 'XA001-PRB0Q-928028839-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'e6d8e636607809421495d448bc447c4ea1147ad3a97626ae8d520b1f3a162baf',
      feeAccount:
          '4123d536f6277e041f1b983274fa77ecb004e698e8195bf81263277a3e92a750',
      stakeAccount:
          'eed8acdd92edf647e50c814d184dcb945d3144ec501f71755d511e03fd25b1bc',
    ),
  ),
  InstitutionInfo(
    cidFullName: '合江省公民储备银行',
    cidShortName: '合江省储行',
    cidFullNameEn: 'Hejiang Provincial Citizen Reserve Bank',
    cidShortNameEn: 'Hejiang Provincial Reserve Bank',
    cidNumber: 'HJ001-PRB0I-089279108-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '63c2f05ed58d35c0206fd5a4451ca84a239e8855febc818e85929212cbe602d0',
      feeAccount:
          'a2f41e35019070029a4fc5a56b379973d51c48a5dab940a010bdd8e28c4ec434',
      stakeAccount:
          '0ef2de0db2ef2fbd6b077f92eff51fce641809d5bc9017cb31df2f47842daf5e',
    ),
  ),
];
