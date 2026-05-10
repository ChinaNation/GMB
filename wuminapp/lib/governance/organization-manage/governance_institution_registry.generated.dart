part of 'institution_registry.dart';

// 本文件由 tools/generate_wuminapp_governance_registry.mjs 自动生成。
// 中文注释：治理机构名称、身份 ID 和制度账户地址来自 runtime primitives；管理员必须动态读取链上 AdminsChange::Subjects。

/// 国储会（1 个）。
const List<InstitutionInfo> kNationalCouncil = [
  InstitutionInfo(
    name: '国家储备委员会',
    sfidNumber: 'GFR-LN001-CB0X-944805165-2026',
    orgType: OrgType.nrc,
    accounts: InstitutionAccounts(
      mainAddress:
          'c17a81b8659f872add43cac07593ab9cd09cf5db19de543e8dc88b5712b8254d',
      feeAddress:
          '1742656aa3214a4bd8fab7670564413d597033c9844c63ea8485917aa9963dfc',
      safetyFundAddress:
          'b8a5c135280278916442137418ab6423eda038bb4662a5c02e70f8d528903529',
    ),
  ),
];

/// 省储会（43 个）。
const List<InstitutionInfo> kProvincialCouncils = [
  InstitutionInfo(
    name: '中枢省储备委员会',
    sfidNumber: 'GFR-ZS001-CB0Y-016974075-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '35b260da16e2a404fe4fd971b4f3410bf3910785b552ef94d223ca5b1dbd7add',
      feeAddress:
          'c74fb9bac6690819b506bb66a343dc4b16f608bb70b9ca572581512b47bd0b2a',
    ),
  ),
  InstitutionInfo(
    name: '岭南省储备委员会',
    sfidNumber: 'GFR-LN001-CB02-773405642-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'b1062a24fb68e7f4df9d2dcd2dd60268b7a97935f0772320b58b742d1270bafd',
      feeAddress:
          'd03e93fb3db2f0318725c6f7ec639185fba3b8346af2ccc12595e2e4c73b6038',
    ),
  ),
  InstitutionInfo(
    name: '广东省储备委员会',
    sfidNumber: 'GFR-GD001-CB0L-067440774-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'ef15f47852f19deb71ac2837147881c81e4eaab7e116bcd656a135273477677a',
      feeAddress:
          '855f76a4a42a4e6cce401f7236b3fd1f7de4b3bd11e0fa7b549666fc1ac83524',
    ),
  ),
  InstitutionInfo(
    name: '广西省储备委员会',
    sfidNumber: 'GFR-GX001-CB0I-663454043-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '214a2802c35b7cb5b1677bf2d9917dc00ad9cfcd2a4fe7e5c89443a8551b0a1d',
      feeAddress:
          'a1529da5a782a6528062baba77c812d990ff5df4068de1316874d214a3dc025d',
    ),
  ),
  InstitutionInfo(
    name: '福建省储备委员会',
    sfidNumber: 'GFR-FJ001-CB03-389570546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '5eb17686a6ebb5c9041e32856fabb36f3600588430664fb868a81c982658f9db',
      feeAddress:
          '7c47e02c34c7c81d7fbc766efc9ed2db8e49a9ba4832c8467d5243f6679c3390',
    ),
  ),
  InstitutionInfo(
    name: '海南省储备委员会',
    sfidNumber: 'GFR-HN001-CB0X-545676096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '515b2bff520b8f95162b83d63a8b147fd1a23fefdc89f44f8e9ea29296d2fce9',
      feeAddress:
          '9f6b62cc5a7912b8896a077280510b841f7cb690527804bd83c1e756c37f7db5',
    ),
  ),
  InstitutionInfo(
    name: '云南省储备委员会',
    sfidNumber: 'GFR-YN001-CB0K-145427171-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'd2f204c6e0a3d406ebe7754075e7fdfe4e08842d05cd99de043c18e2037f4291',
      feeAddress:
          '00bdcb16ef47909f9343ba747199383c72066f9a7bacdaf02840685a9705f563',
    ),
  ),
  InstitutionInfo(
    name: '贵州省储备委员会',
    sfidNumber: 'GFR-GZ001-CB0I-969970096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '1838966ec86b3633447ff84577d830b242b8926cd44b653a7ba6a75219bfcb40',
      feeAddress:
          'b9719a583337e46c72a09a4e909f378f4e123dd72e4a13781e6f9e810310caeb',
    ),
  ),
  InstitutionInfo(
    name: '湖南省储备委员会',
    sfidNumber: 'GFR-HU001-CB03-400319700-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '524bae134492c1c0164a7882123e61f83aece451046f9df100241a2c0d779231',
      feeAddress:
          'b1a2e95a95dfb675945e2f4613b2bf120580406c10f795626abd696fc1f4bcec',
    ),
  ),
  InstitutionInfo(
    name: '江西省储备委员会',
    sfidNumber: 'GFR-JX001-CB0Q-458681566-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '4c0bdcc3b57fc2eea8aa63cd5f68dfeb225eba75010c9d860d67a51488f34fb3',
      feeAddress:
          'd3db673aaf927bab62585b98d74f48227a8b7bbd34b983bce81c7e1d7c978e27',
    ),
  ),
  InstitutionInfo(
    name: '浙江省储备委员会',
    sfidNumber: 'GFR-ZJ001-CB0J-471270801-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'd4a44d51a769234aff61c6e44522b3087a250be7ed2bb32e234da7a0eb807ee2',
      feeAddress:
          'f304dc6035783afb4f6daa7dced71afddcf3885a20fb0acd6e2994d05d7efe5e',
    ),
  ),
  InstitutionInfo(
    name: '江苏省储备委员会',
    sfidNumber: 'GFR-JS001-CB08-358467174-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '5fddcd22f1517423117d17f41ed95e5a8ae61daf42baedbd2adb793c5098176c',
      feeAddress:
          'b9c7fd454ce093997a9acbc982f5b36e457acbddd80a5ca95a20bcfe21b6cfdc',
    ),
  ),
  InstitutionInfo(
    name: '山东省储备委员会',
    sfidNumber: 'GFR-SD001-CB03-027328848-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '720c2d2c2539818f3cdad00620e6247e3baa469d289e257d78f0d891ae6c095d',
      feeAddress:
          '3ef1ca7ced2d9eeeb71a0c8cc58b93d3ed274e8b6146137c508132056acdf6a3',
    ),
  ),
  InstitutionInfo(
    name: '山西省储备委员会',
    sfidNumber: 'GFR-SX001-CB08-104465679-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '4b5732a8d83d1fcb34d90c23153c0467105b9e55f2b27e6f87780659d7303016',
      feeAddress:
          '91d95e3449fb9f035813bcbc5344d71a15d447a152000ea16f5b96570dcaf96b',
    ),
  ),
  InstitutionInfo(
    name: '河南省储备委员会',
    sfidNumber: 'GFR-HE001-CB02-849245626-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '10a63700102f43b8ba2468146ded4a3732254eca9d2ecb60a05d3a51654ccfe9',
      feeAddress:
          'ad82e189c5a322c403e05ba53522aac086a583fc30854e198c1300eb1ea089c7',
    ),
  ),
  InstitutionInfo(
    name: '河北省储备委员会',
    sfidNumber: 'GFR-HB001-CB07-499533387-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '368dc07f781b6ca2dab8ed973ceee27b096d8222a5b5de7df22769c38a978db2',
      feeAddress:
          '870a0110ba608537dd647a5bdc82f3e2a3979dab8be3051baa928af9dc357940',
    ),
  ),
  InstitutionInfo(
    name: '湖北省储备委员会',
    sfidNumber: 'GFR-HI001-CB01-659443961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'a47221068aecaed3a6a52f2763a87b4ad8f3351324a5b1bf919a9e8d3f000b7a',
      feeAddress:
          '1d04f3c4afb40ac5dcf1bc2cdfd5abfe2e2cd1d5c9f87d70eba011e902c253e6',
    ),
  ),
  InstitutionInfo(
    name: '陕西省储备委员会',
    sfidNumber: 'GFR-SI001-CB0Y-711309909-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'deb9cbe0c6b9f9321396077ac9ee539b3a5b90ac0e3f3393f93d4d944eb13b97',
      feeAddress:
          'fb84a24aa6c5c8268439eaa96a11118547ca043a1a33d88b53c188178af518ab',
    ),
  ),
  InstitutionInfo(
    name: '重庆省储备委员会',
    sfidNumber: 'GFR-CQ001-CB0Z-478472058-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '2d6efd3cb52cda3781bd266aa6b83cea52e7cee6fcd225165a324d0cb045c402',
      feeAddress:
          '13886e7186a005c10174c08f08dc14605daafa25178b700c0c983be79cf98583',
    ),
  ),
  InstitutionInfo(
    name: '四川省储备委员会',
    sfidNumber: 'GFR-SC001-CB0N-935659021-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '5831d7f28215d3bffbaf32392277445d607c463fc8d4c06b5dae98963096d124',
      feeAddress:
          '808b84d6fe9b754dd730fa01cd49970c0c744aac7a76d219b720c76a7df913cd',
    ),
  ),
  InstitutionInfo(
    name: '甘肃省储备委员会',
    sfidNumber: 'GFR-GS001-CB0K-679051155-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '5bb1f737a39a74480097f9976a76694986e0baa6604cd49116dfef8ef5157d87',
      feeAddress:
          '46e1123d56115da600720120d9195b7c8e754553147c6647096aac856f9639bc',
    ),
  ),
  InstitutionInfo(
    name: '北平省储备委员会',
    sfidNumber: 'GFR-BP001-CB06-189323546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'ebac3f3618b2b1d88eaaa9bcb75c9bfb556ee8b423f5f5299d795e90dbc2e2be',
      feeAddress:
          '6321d225caf7484d30cd5e2592411ee91608ac29f82ae284819b07490077c7da',
    ),
  ),
  InstitutionInfo(
    name: '海滨省储备委员会',
    sfidNumber: 'GFR-HA001-CB0C-214178517-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '6f7b25bb78faba8860ea10544f9e144b87ed4bca0fb80fe8884c52b4fb8550aa',
      feeAddress:
          '033a29e901fe60c38d543a6c62a107a56680cecceb225d25856cea0716abbe42',
    ),
  ),
  InstitutionInfo(
    name: '松江省储备委员会',
    sfidNumber: 'GFR-SJ001-CB0V-044490898-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '42dc268532b8a3a9fdf0b631caea82998cacb674720cfa9b129d5a9147b9d154',
      feeAddress:
          '2f6ad9eb6d6e1c450e76f1309f1916c56500c06c4c941e4fbfd1b0fd526ea4e8',
    ),
  ),
  InstitutionInfo(
    name: '龙江省储备委员会',
    sfidNumber: 'GFR-LJ001-CB05-279890045-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '4225308656249d8cbea7ddd608e14f78d408ff772577d51c9f06c3f34aa04fa5',
      feeAddress:
          '8638089cebc367edde7df248a02a77be06c7495da7af4a92b25f1fb3cacfffc6',
    ),
  ),
  InstitutionInfo(
    name: '吉林省储备委员会',
    sfidNumber: 'GFR-JL001-CB0C-850461124-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'f196c92448273df5b1c5c241f921aa649dd4b01dd966c692697801d99ab668b4',
      feeAddress:
          '64deeab7a35e4a928d7d7354bbe5d550770e0b045df790f048b078934e700944',
    ),
  ),
  InstitutionInfo(
    name: '辽宁省储备委员会',
    sfidNumber: 'GFR-LI001-CB0P-978545133-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'a2590d7e00706a709476f385a029b66f5e03ff321e66d3cc0a4cb5355bae3f6a',
      feeAddress:
          'c41240bfbee955bee4aca0d712ea78b37c6ccf72388dbb7285b2cb607cb16fb9',
    ),
  ),
  InstitutionInfo(
    name: '宁夏省储备委员会',
    sfidNumber: 'GFR-NX001-CB0C-389752794-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'd68e13c986abafc8b6cbf8db4ff2ff8be9ac888839cd1c3b005e80d043267f7c',
      feeAddress:
          '6b38e7f00f4c3b397c5ea7c4b04ec5f09c58504e4ec46ccbaea3ff06fda79b6f',
    ),
  ),
  InstitutionInfo(
    name: '青海省储备委员会',
    sfidNumber: 'GFR-QH001-CB0C-882026762-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '5c5dba4150fe345fd9fbe7cf583d50d7794c415136d3c062a4f99a54ef572031',
      feeAddress:
          '591ade331d4c5e3a1bb62360ba2e23bd450b07605b6026360f02303c427a4331',
    ),
  ),
  InstitutionInfo(
    name: '安徽省储备委员会',
    sfidNumber: 'GFR-AH001-CB0O-589856828-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'e1e084176b88e0ced1b18cfa1fde4f0ec4e8c8ae2204c98a106f708ec272db6a',
      feeAddress:
          '8a10ec1911ac6edaf4d62ed565176ca88b9d3fdf6aa024d881e163bdce7ff384',
    ),
  ),
  InstitutionInfo(
    name: '台湾省储备委员会',
    sfidNumber: 'GFR-TW001-CB0H-265218823-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '39070ab9ec17b3acec0cb6c9d3e59d136d26f0c5e73508ece1c8eee28b157c77',
      feeAddress:
          '26d42c4403a5fe1d5e6d3a0e93f5028c7124f49d7751564ad79584d1ef4c138b',
    ),
  ),
  InstitutionInfo(
    name: '西藏省储备委员会',
    sfidNumber: 'GFR-XZ001-CB05-435616961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '352df132a06d0f10deb176026137ddf6b03837f8b879009d89a41acbeab4256b',
      feeAddress:
          '58f322de2d510d33418c5f8502bcb9c09863975090c62f40e9dea3873b6554a0',
    ),
  ),
  InstitutionInfo(
    name: '新疆省储备委员会',
    sfidNumber: 'GFR-XJ001-CB0F-671044381-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '0da0c46dcfaae51f07018ca62dee18c498ffca78971a6617e9d83fa8b5a894ff',
      feeAddress:
          'aab30467c107d558c252efec6cdb92b4ea789a18148afdfbd0d76be2d8e51a3e',
    ),
  ),
  InstitutionInfo(
    name: '西康省储备委员会',
    sfidNumber: 'GFR-XK001-CB05-695945392-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '7032837e5f3e403cd424154ac64beb46fe58e2cc86f9f8fcb54fb5d2443cf43c',
      feeAddress:
          '6e27d91a4fd231032d497c6b333be9914d84b720f1913ae2c5261f13628ec303',
    ),
  ),
  InstitutionInfo(
    name: '阿里省储备委员会',
    sfidNumber: 'GFR-AL001-CB0Z-487847725-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '3d64628da4a73f480032de48f1c848caedc0e055aa8be355659061132ec444a3',
      feeAddress:
          '3b2b604dab24a02a01a389a5b98774a2848afed39216d5f7a78717fa098c7dcf',
    ),
  ),
  InstitutionInfo(
    name: '葱岭省储备委员会',
    sfidNumber: 'GFR-CL001-CB0B-771698743-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '8da886c3376fafb490d6b1f841848d06d69d6c8efa5f5cad100780add632fa67',
      feeAddress:
          'ce3c9691796e19b385aa20901fb591fe1c6ea309325b48bf7be2da231a779bad',
    ),
  ),
  InstitutionInfo(
    name: '天山省储备委员会',
    sfidNumber: 'GFR-TS001-CB0T-293160581-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '90c1d59424df1c3e395580f7cd99f5464e9aa837630d793d6fbc14d017022e32',
      feeAddress:
          '97506497fd7c848273d75868d13d627b21ce68031649bdbdd543cc301dae088c',
    ),
  ),
  InstitutionInfo(
    name: '河西省储备委员会',
    sfidNumber: 'GFR-HX001-CB0I-475713213-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'de54b98d0555079f61b693c4c6ea1e854b3f55d49ad4b57c886388c9b8e77139',
      feeAddress:
          'f2dabdbc96de4e2086db2fd285da3f7ecb536d1f65796e5a14019b93330d568a',
    ),
  ),
  InstitutionInfo(
    name: '昆仑省储备委员会',
    sfidNumber: 'GFR-KL001-CB0Q-091969119-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'fe04e3332dc1be1bf01b2525b02a8794c36ad64dc1b212f0df27a7a6640514c9',
      feeAddress:
          'e589adcc31b50b02cb2f43e5b593ff26f25a9dc8bb4e43ff43ff5c9a2bd2a4ec',
    ),
  ),
  InstitutionInfo(
    name: '河套省储备委员会',
    sfidNumber: 'GFR-HT001-CB07-481172908-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '6526d5e20bd6d0209512a77dc8f53d2a42746bf88c23e28f7b5f49f2a70c1748',
      feeAddress:
          '85b36acc3e28790041e1cd15c1a55f570fdf658cb598c2127186459c4c403b1b',
    ),
  ),
  InstitutionInfo(
    name: '热河省储备委员会',
    sfidNumber: 'GFR-RH001-CB08-697831866-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          'a15e4c7d37e8d2ce51a253200bbdf7beb82b5f060e673f141699b0ae134ece2a',
      feeAddress:
          'b921be02cc6c1c5968958f48799b35f806006e4f033e152d75d927e1a12b5acd',
    ),
  ),
  InstitutionInfo(
    name: '兴安省储备委员会',
    sfidNumber: 'GFR-XA001-CB0V-384161601-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '08cf73227c0b584297d611a95f8da227a844d154bcde508f26468880be7b7eaf',
      feeAddress:
          '64c55623b1716b61605e0c820beebc117a9d15b7aa95e21271ca2ee8c5576b2d',
    ),
  ),
  InstitutionInfo(
    name: '合江省储备委员会',
    sfidNumber: 'GFR-HJ001-CB0K-963948997-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress:
          '429bc73c92fff4dc6fd6e8a85fbbc9675b101985b4728b6bd22c6f2573a9dc82',
      feeAddress:
          '12df81a2994dd0437f377b459313736696d113cf57c3f9106d2cd621489952bb',
    ),
  ),
];

/// 省储行（43 个）。
const List<InstitutionInfo> kProvincialBanks = [
  InstitutionInfo(
    name: '中枢省公民储备银行',
    sfidNumber: 'SFR-ZS001-CH1J-233384677-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'c066e6652040f49c6d9865112203977b23bf616659d3c2c960ff904b9bd6bd63',
      feeAddress:
          '63ea64072a868a7f5f78d073a2c1b4dc9b28674edbb2f915c210548b4a3a8706',
      stakeAddress:
          'ae4ed52d57db615a93f5df925b1f4f4a2eb20f5708d6b423064b1249d4304d85',
    ),
  ),
  InstitutionInfo(
    name: '岭南省公民储备银行',
    sfidNumber: 'SFR-LN001-CH1O-703127075-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '9187a38f9a82ca7c9494d82f7cfe5472c724657c0b6392510dadf9dba25790dd',
      feeAddress:
          'aa70a2971cfc72434dfa9248d0a90a75678fbeb58afd36696314f5396595ccbc',
      stakeAddress:
          'f663f92bac6aae83628e49213ce81136f0967b5e034f2a4ef155590325655f29',
    ),
  ),
  InstitutionInfo(
    name: '广东省公民储备银行',
    sfidNumber: 'SFR-GD001-CH1I-239565809-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '07086282474acd9888107c0d63b2bc5e0900b08357eb339b0d4cc7d19f1188ba',
      feeAddress:
          '064660d90bf169856deaef954ca690a7374594fc7127c7561ae54a18e1f8e1f0',
      stakeAddress:
          '26e18cac5a882e0489ed46d5edbb81ccfab36fb7f66122299d8d5a9d7cef1e32',
    ),
  ),
  InstitutionInfo(
    name: '广西省公民储备银行',
    sfidNumber: 'SFR-GX001-CH1Q-025559630-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '0daea8edbdaab3fa7431f1c826a8e9352afc9e5caafcdb61992d734c7df524bc',
      feeAddress:
          'bb6204cd1ac9544924fc251dc642a9b62811011190152803e14633b340f93e2e',
      stakeAddress:
          '07fb4574efa8531f86c32b244f9c2e87a4f50dd8c8ac85e7f2016f2b0ea2f544',
    ),
  ),
  InstitutionInfo(
    name: '福建省公民储备银行',
    sfidNumber: 'SFR-FJ001-CH1L-504679612-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'c02ac802d28ea69edf6c4da12df5025d1813e640ab05b50435bfe6b32d9b3c3f',
      feeAddress:
          '4132b35eb79dee4dd466430ac32ee3de9d017f7154a0480e5af3b903cbbb1dd2',
      stakeAddress:
          'aa7265816feb1764a8cd4d31baaf6c00e7d00059426479cccd56b129d2f461d8',
    ),
  ),
  InstitutionInfo(
    name: '海南省公民储备银行',
    sfidNumber: 'SFR-HN001-CH1L-723623074-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '0c4c6caee18ada96ff171be9b434eb7afc1ffca7b4ee51ea041fb4f1dbab4d75',
      feeAddress:
          '7d3585d09ce4cdbff6c72aad80f085fd0a9b5852bf5615c46384ec0554512418',
      stakeAddress:
          'f315bda9054e07b5c2ec0a313b74dd237a1723d8f1602bce8e03a1d9b0776184',
    ),
  ),
  InstitutionInfo(
    name: '云南省公民储备银行',
    sfidNumber: 'SFR-YN001-CH11-692525950-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '82a23332e3b7d1d961fd51b7a7676ee5686284f4f6f807c4611735b4f1cdaf86',
      feeAddress:
          '88b87ae92b85d333d9e1ade878fddb162d4b7c226db36fa985eda507898c1a5f',
      stakeAddress:
          '8524ccfd3c567451f63f55c11ace4fcf59a857dc652d6cc8adbb384d56a1b8b2',
    ),
  ),
  InstitutionInfo(
    name: '贵州省公民储备银行',
    sfidNumber: 'SFR-GZ001-CH1R-490015860-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'fdd065561c3bfd66453ecc8310f610c25e17f7e785be34f508ae80090c195822',
      feeAddress:
          '9017744a29c962f19d51441d8bc7d779e6dfc3270c8d52ac314238c5677be194',
      stakeAddress:
          'f58b0cd1d169baedbafddd366fce7ec97d1477ad931a64c26c46f25b35820d3a',
    ),
  ),
  InstitutionInfo(
    name: '湖南省公民储备银行',
    sfidNumber: 'SFR-HU001-CH1G-084835673-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'db3f8a4b6e220216364ddcde165c215dffd16915834d679015b2068d79d45376',
      feeAddress:
          '0307e4b7241131d0841db14d090689bfd5c3e7a3cd7420e8fc67eda4df088c4f',
      stakeAddress:
          '1e14d0b8fbad8734073b1f1089b33bfad77dacf102f5e842da48f7a3a5920710',
    ),
  ),
  InstitutionInfo(
    name: '江西省公民储备银行',
    sfidNumber: 'SFR-JX001-CH13-243765987-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '95d7d76c4796cc2d8bd0851c83d71bdd01fd5726cd20e5ba9ae157218e0de1fb',
      feeAddress:
          'b2346e19fbac17adbd29e4fb4ca9c37dadc043edb00c6907174c13a55697c883',
      stakeAddress:
          'ec0e9a1bb89072ef38f161a6f08186aceba1decb1395139f565b1e62a3e26f5f',
    ),
  ),
  InstitutionInfo(
    name: '浙江省公民储备银行',
    sfidNumber: 'SFR-ZJ001-CH1B-296232973-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'ec78ef7b3135fc8b7aae292fc22f8f9e80acfe3eb17d69fb4c73a0b6214642ea',
      feeAddress:
          '35d8c6af39466ffb8c2b41e669ee7102b484eaecec85377b83b04eb6cf91165d',
      stakeAddress:
          'd3413cb063bafb89c77c6e55cd3c3a8f4d9b65157136c10e0d3f18fd50b9d2a2',
    ),
  ),
  InstitutionInfo(
    name: '江苏省公民储备银行',
    sfidNumber: 'SFR-JS001-CH16-890774605-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'f25c54b2ca04a5056fc01108d2d9f9b76c74112de8dba0818b8e8da3c5c01c6e',
      feeAddress:
          '0dd4e61bd6abc8903750ae303ea344a8bae6c248f608b6605613e5e8b5e2c993',
      stakeAddress:
          '2ad18a8d778bb218d5593fd561ce438a61e1161cb0cd87c98ac0eee3855cab74',
    ),
  ),
  InstitutionInfo(
    name: '山东省公民储备银行',
    sfidNumber: 'SFR-SD001-CH1B-114256751-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '484f35db47c56aae30af9fd11aae23249054c0f755659e0454a3d19ddbb6fea6',
      feeAddress:
          '737b1f50b20bf7db32eb1ce3208ddaa87313162a0c3657b03e173f81cb7378e8',
      stakeAddress:
          '6c37caceb8ead83a7eadfdf252c9d94dcec368aea97dfd80976969f47e6f88d8',
    ),
  ),
  InstitutionInfo(
    name: '山西省公民储备银行',
    sfidNumber: 'SFR-SX001-CH1X-520132196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '7d1a1d96dfdc1bf8af028168bd6b647dcaab70696a88feefb89059ec3d6e41c8',
      feeAddress:
          '33fc70cf4d4ce054ed8a4630ed8cfafa71e69d15e177ec2abbc05d562fe5174b',
      stakeAddress:
          'd976d0d414fe5718f3053ee3477abdd4423689d5a0c62bd883c775bcf7eb7a82',
    ),
  ),
  InstitutionInfo(
    name: '河南省公民储备银行',
    sfidNumber: 'SFR-HE001-CH12-158889343-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '6e3af0e062bccec09599e30cadee61679f74c1788b0a6c6fa4b6ea8540c71685',
      feeAddress:
          'e237db6886260f4732000d3af8b10e228d99ec384a5d682c80d88449f8aa15a8',
      stakeAddress:
          'b77510391b2fe4f533225d9cc108cd78a8c9dbaa62fa9257a827505f1ce666f3',
    ),
  ),
  InstitutionInfo(
    name: '河北省公民储备银行',
    sfidNumber: 'SFR-HB001-CH1R-484022741-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'fdff9c2f318b13c7eb34241eecbb26863e909d92d2a5f39022edbb200291dd73',
      feeAddress:
          '8e21388e83ad39c4dc32c4c9532f67110be042c33caa1c34fbb98fbdf8e38ff1',
      stakeAddress:
          'e6446dbf0c8c5524cce842282dac603bc3b336423d528c15dbddc60c0a3de446',
    ),
  ),
  InstitutionInfo(
    name: '湖北省公民储备银行',
    sfidNumber: 'SFR-HI001-CH1G-514948302-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '531d6274bb23fc3c1774802a951b5f694e43bbc596c788df289346bd9114fbad',
      feeAddress:
          'fc05f87fc20c6c7ab83e361caa671165c44ce3a7f2c1cbf80a7a76f8693ce5a8',
      stakeAddress:
          '5dc07587276348048a300cabe6b6778152201c93f275925153c9e70337de06bd',
    ),
  ),
  InstitutionInfo(
    name: '陕西省公民储备银行',
    sfidNumber: 'SFR-SI001-CH1D-245618374-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '55600791d80bc6d2c13b90d876a1d3a2022a9ef9be8e8b058e7e9b77028ec031',
      feeAddress:
          'a1033e826fe9d01070c395be7d7153db2f417b0fe7e11a712dc38a1c286c8969',
      stakeAddress:
          'e2a814c60b6a26c605fcfc5c473d2db023ba9ae150a651b2eb4caf115c96e2a1',
    ),
  ),
  InstitutionInfo(
    name: '重庆省公民储备银行',
    sfidNumber: 'SFR-CQ001-CH18-694162045-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'bf9823a0ed5e86c3a1c1285f85b99a074bbac70891ad7a534366bbe79162c622',
      feeAddress:
          '63e5e8359973380ff95325532c20b7c4dacaf35a724766b8dc3e277fd563d8ba',
      stakeAddress:
          '6e237576a81138b3888dbea9fa6745f2d557922e44f96d4450fa1fa233ec4b08',
    ),
  ),
  InstitutionInfo(
    name: '四川省公民储备银行',
    sfidNumber: 'SFR-SC001-CH1Y-764253139-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '9b21bc43a34fa9a749e5bfb6cddafef8301b64cec5c25bead43c8975f466a11f',
      feeAddress:
          'e4af9254c728d875362973e3a2c6ff709b9917a61e8473b641c3097589cde8de',
      stakeAddress:
          '388e208cc1936c779e554d9682498263542847c70fb11eda6ee75a37dd5b1252',
    ),
  ),
  InstitutionInfo(
    name: '甘肃省公民储备银行',
    sfidNumber: 'SFR-GS001-CH14-005784877-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '9f053cd910af2e6fed90a86cd9a903cc8051360f58c548b0612537843119ec66',
      feeAddress:
          '6daddba49b688803f6b9b35a7845914438ffa9abad3a84da9b884d6ce500d877',
      stakeAddress:
          'c99fda9ce9d94dc5741b9929e52fafc6d5c70feeddfec885f85ded2ed18cabd5',
    ),
  ),
  InstitutionInfo(
    name: '北平省公民储备银行',
    sfidNumber: 'SFR-BP001-CH1M-434307982-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'c40fa2fcd9c0d161edaf31857d97b483b04370a1712cba2e9a95a4e703dbf98b',
      feeAddress:
          '752c0d672248ebb5a25fc0f42f768c4374ce46e231153464346695c948d7a832',
      stakeAddress:
          '293523f4797b8ef238cd450dbeb6685ac70fe33952826323c8d1c867ed26701c',
    ),
  ),
  InstitutionInfo(
    name: '海滨省公民储备银行',
    sfidNumber: 'SFR-HA001-CH19-969179618-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '44f16cae73f18f7fbd6bbea6095625401633a25e5c4b7f97d21c51817950a061',
      feeAddress:
          '88c7ea7595bf30b9768988c6d0cd6a70c7e56dc648ba63c8adda23166aa9cbce',
      stakeAddress:
          'bbdebd7bd38742432ae558daff2350ca500d2601938290e7ef975538bc92c303',
    ),
  ),
  InstitutionInfo(
    name: '松江省公民储备银行',
    sfidNumber: 'SFR-SJ001-CH1G-644104544-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '9183d611128cf8879cf87b60dc1773756067a30d48b6c187dfdaa4abdc27fa94',
      feeAddress:
          '4aecfa8eca6ad369546c3c8997d1c6b3575a702e49f35a5e6f82ac6d355bfb70',
      stakeAddress:
          '3a7432067dc91184e20b6d9ef491b3ae5fd1a68b0caa9f9c1420fe9b5986ca04',
    ),
  ),
  InstitutionInfo(
    name: '龙江省公民储备银行',
    sfidNumber: 'SFR-LJ001-CH1J-280510636-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '4483ae1dede4aa9a7e7ec224337a6cec2fb744b2ba9b107af0ed0c67ffd3ed08',
      feeAddress:
          '89f64d011fd7bb75267faecd01e46ccdd4735e0582e76b6e7d010bdb9b4282c4',
      stakeAddress:
          'd53fa0a8d694b92489c48a18968ba28cc798cb3637d3853e2cec155cafa6c6b4',
    ),
  ),
  InstitutionInfo(
    name: '吉林省公民储备银行',
    sfidNumber: 'SFR-JL001-CH17-129935340-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'd53c03fe7263d6a88e7f4b0984854b665280e571b9e0f30dd8954acc868da1c2',
      feeAddress:
          '4cb06d54c65fc87d5521bc892687ac36666fd48a044ee2183a89428599fc9041',
      stakeAddress:
          '37fd9a2a607af9317ae3a5af3a4f4b9b6f40554d58484d1dc10059b1f70cccbb',
    ),
  ),
  InstitutionInfo(
    name: '辽宁省公民储备银行',
    sfidNumber: 'SFR-LI001-CH10-249814963-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'd0b8f1388aa00194902fffdc278da17c11374bd3abcc24715ab7d18c28734aa6',
      feeAddress:
          '14f9e16ef32ccc25b3d12d680a59676be0c1a5839a2baa633b16f27f54c6711a',
      stakeAddress:
          '28cbfa291ab4f84fb61bf8a028d1d2adaf043d513f2417fb88c537c728ca4869',
    ),
  ),
  InstitutionInfo(
    name: '宁夏省公民储备银行',
    sfidNumber: 'SFR-NX001-CH1N-292327153-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'c0922791a0d52ade7bc528ba686424061075878dccb6b1424c95762f277da910',
      feeAddress:
          'aad1f1e862baf2f8be2a05ccda284bead3b232882056b5e115b5f3fffc9eb600',
      stakeAddress:
          '462a3227dec2369278a7c2ae310b5800b308d927051a51d8eaf2fe4f663d7471',
    ),
  ),
  InstitutionInfo(
    name: '青海省公民储备银行',
    sfidNumber: 'SFR-QH001-CH12-075657014-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'edabbc35805ada30a2e8b9940445b1ab977f2e23ae3af7bb53dfcd3ea1455e69',
      feeAddress:
          '23b3fc8cd1668743bca12a9ac45ace19f867039d2e1e73e098d6fa10335f7ed4',
      stakeAddress:
          '8ac1e0f5aaf7a1875caa592a75dd6ee703133741e50697e219d5a94fac7bcbae',
    ),
  ),
  InstitutionInfo(
    name: '安徽省公民储备银行',
    sfidNumber: 'SFR-AH001-CH1D-388477914-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '26d44dbfe74a3c7c7972479899f5d02a6e26e30d883acbfa1e9a173b2f163768',
      feeAddress:
          '8a1121245458dd74dc329313e54c5f8b249640c1703618a05c5738bdac1e5fe0',
      stakeAddress:
          'd038281290f7ffbb5c3db5346cbcdec38c763d5aa6972b5b8a233d93b56ed145',
    ),
  ),
  InstitutionInfo(
    name: '台湾省公民储备银行',
    sfidNumber: 'SFR-TW001-CH1X-266238196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '2f041118fb18c823b706cb77eb3af5ad701e80abd91a95ba9c100c184da0c53c',
      feeAddress:
          '9528e3a1b2984b2efdfc33188cb7823f81ed9359636fb7c41b4fde964c105c18',
      stakeAddress:
          'bc222ce4d09d8733402472cc51d2ec0714e2e8fec31d3e2cd07c98ba5f32633e',
    ),
  ),
  InstitutionInfo(
    name: '西藏省公民储备银行',
    sfidNumber: 'SFR-XZ001-CH1U-210788637-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'f4c04bb7cd32f486bac50b6d2f57147a278c79d6be8460e8f8f9333e380a4496',
      feeAddress:
          'efc3a0b41445498d8cfe2330c220e1564531b94d0bfdd5847cf07a1c77349299',
      stakeAddress:
          '104fda79b083fba75fea375f964884d3217e207c5cb0098d8a0fe1dc71999704',
    ),
  ),
  InstitutionInfo(
    name: '新疆省公民储备银行',
    sfidNumber: 'SFR-XJ001-CH1J-233325633-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'e53005053a1b8aae68abaa5f4602924330cd7027af3228fa5c4c2bd9962080ef',
      feeAddress:
          '8399093f6cf21174084c7ac4dcbf708b6c481160ff40159afbbf9a9c5c109b45',
      stakeAddress:
          '4706b64a5ebcb9fb1a65fce4e483e24ce6401d0ed92aeb0296094fb8584a103a',
    ),
  ),
  InstitutionInfo(
    name: '西康省公民储备银行',
    sfidNumber: 'SFR-XK001-CH1Z-300401625-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'be34f27eb2c71a72afc17872acd3c34dc1a04427b54b13fa7410275cacf60769',
      feeAddress:
          '5382bd2d0053d13a701e287fd62d2430ae0200b29ae66a299e3cd186cdc582d9',
      stakeAddress:
          '808ec0f9804456f9222a40b4429a10b0b6431525a4fd3cee87425dd551f969d6',
    ),
  ),
  InstitutionInfo(
    name: '阿里省公民储备银行',
    sfidNumber: 'SFR-AL001-CH1J-527686065-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'b531c162a6cc380d076b56ce7c099325b93b447d789bf39183cd73a956bb9018',
      feeAddress:
          '18d2734d04bf921f04514dab96162ea55cc938c16c60cba584a1d4c01d51d87c',
      stakeAddress:
          '7566a1de856e8429ffbe16c4407739537ffdef5e562f3d9d177b0546e61516bf',
    ),
  ),
  InstitutionInfo(
    name: '葱岭省公民储备银行',
    sfidNumber: 'SFR-CL001-CH1Z-951267669-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '555523b2c5b90af752e24da0b63445db598eecb1f730fa662f04d0ab25bae585',
      feeAddress:
          'd3c1ad93ddb59bbc45bf2220cc9cee1b345942bcc91e6f05a389b1ce567667b2',
      stakeAddress:
          '02601bdedcd01cab7c283c01ee75edf7c8c17823f9035bebfbb0600846be086c',
    ),
  ),
  InstitutionInfo(
    name: '天山省公民储备银行',
    sfidNumber: 'SFR-TS001-CH1A-142800261-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '66759c25290539461128a8df6cd4b410b941fbc58134f9b2c6faf6f6d4eb8d4c',
      feeAddress:
          'c286f84149d187f4519228c12464dcc1d7706b88b281125d45702a6caf7e33a9',
      stakeAddress:
          'e19de339f42f26eb2027bb2b973024b4e1ad91c4969c534b647d9051a3e41292',
    ),
  ),
  InstitutionInfo(
    name: '河西省公民储备银行',
    sfidNumber: 'SFR-HX001-CH1N-215310265-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '6d32c47def72193b2ffdfae638f260c2277f3ad6676d5a8d383b22605b97a62f',
      feeAddress:
          '3a14e02620a534aa00321378d803be1e3a8c03c61d37f99cbc266c23c80e74fb',
      stakeAddress:
          '358f7e8e1747e9ab86d05a79b9001cf833116ea682f30a91a2ea96d86b3790b1',
    ),
  ),
  InstitutionInfo(
    name: '昆仑省公民储备银行',
    sfidNumber: 'SFR-KL001-CH1R-682838027-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'ea8330083abc599178d00f0276c4029f27a509f9f26f68407409ba65db112727',
      feeAddress:
          'eebd7db15ddbab0410ff7ec33c517728451ce9e5774aac4c57fec6d148ad7392',
      stakeAddress:
          '79af8d78edafd34dafeee47ee97444a25a7997592e75044756f279f06f5fd172',
    ),
  ),
  InstitutionInfo(
    name: '河套省公民储备银行',
    sfidNumber: 'SFR-HT001-CH1V-210616196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '9796b613f9a284266b116ca3aaee536995b19c8c0831ef67a92fe0833eeaeb62',
      feeAddress:
          '14a9e00066263ee4f004fd81e53166018b3c08668d77a5b0bcd43c1a10240956',
      stakeAddress:
          '2b9aae8c4ee545ea43918d42841b804fb8ace3d14190b937e1373a4c840b3ee5',
    ),
  ),
  InstitutionInfo(
    name: '热河省公民储备银行',
    sfidNumber: 'SFR-RH001-CH10-380830938-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '020cacb8a1395f95f51fd8b1ffeae3ce0095555b3715316b877189e0f6b8948d',
      feeAddress:
          'a1887841ef3b504c9ba3ece9d3d710cbc8963a204a4ee11be0fa795eb73ba0a9',
      stakeAddress:
          '88cbdd69cbbbcb287dda98cbc75b472bc3f69f1ce09b1f9608eaf6ced095d253',
    ),
  ),
  InstitutionInfo(
    name: '兴安省公民储备银行',
    sfidNumber: 'SFR-XA001-CH1P-928028839-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          '187298ff4b7ca783074c227061a9cdb3421f2cb69a9e9a7431c5ddd5039e2424',
      feeAddress:
          'cc0fada91d04730cd71e801561737156f44d192108db730ea0e0c60e56dc948b',
      stakeAddress:
          '5262f96b92c85ea41d76d6f4bcb18076889e8fdf9eff05c95e5ed4e9a777ee25',
    ),
  ),
  InstitutionInfo(
    name: '合江省公民储备银行',
    sfidNumber: 'SFR-HJ001-CH1M-089279108-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress:
          'e80e37a9e27fff257257acd4517fe13efc58ddf73f18838fe261f314d59cbc73',
      feeAddress:
          'fdc705e3fdb5e89150b08c055296df642d44b2bb98d079a7e1e1389751baa152',
      stakeAddress:
          '4bea80006c939414d77e06200f3e867cf1e664399911e5631b6c9a0508d6fc76',
    ),
  ),
];
