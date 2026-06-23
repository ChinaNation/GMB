part of 'institution_registry.dart';

// 本文件由 scripts/generate_citizenapp_governance_registry.mjs 自动生成。
// 中文注释：治理机构全称/简称、cid_number 和制度账户来自 runtime primitives；管理员必须动态读取链上 AdminsChange::AdminAccounts。

/// 国储会（1 个）。
const List<InstitutionInfo> kNationalCouncil = [
  InstitutionInfo(
    cidFullName: '国家公民储备委员会',
    cidShortName: '国储会',
    cidNumber: 'LN001-NRC0G-944805165-2026',
    orgType: OrgType.nrc,
    accounts: InstitutionAccounts(
      mainAccount:
          'e97b09bf079f00a243de874e1129ca04cca12de75bd0943b25519e8c1153b005',
      feeAccount:
          '34435d3c37634342f6b7044fe556c9be25dbe7cf237fe9a71c29e64ddf11e0ad',
      anquanAccount:
          '24b62e1756a27dd00946f33507004ce7973be28b0728cb4dc786231c40348a92',
      heAccount:
          '37994b77b6ae316d1916e343a14e3d18c02ebe74686e4c2652202f55ae0e66fc',
    ),
  ),
];

/// 省储会（43 个）。
const List<InstitutionInfo> kProvincialCouncils = [
  InstitutionInfo(
    cidFullName: '中枢省公民储备委员会',
    cidShortName: '中枢省储会',
    cidNumber: 'ZS001-PRC0E-016974075-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '26d09d54d2284662bb35c23591ba7ef970038a22a0d484c0a5835a9984432fdf',
      feeAccount:
          'f38aaa5b3688ba54b9c916055403399b86a6f62e004a4eac2b18decff63647c4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '岭南省公民储备委员会',
    cidShortName: '岭南省储会',
    cidNumber: 'LN001-PRC05-773405642-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'b19e5915f588844fd7d4fce0fc9840b0237bd9ccf167a00eb591ed6d66c288d2',
      feeAccount:
          'c73e7d5f50ead9e6986c077d0ee31b9b9c9177f1b1eb0707c21762c354167c1c',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广东省公民储备委员会',
    cidShortName: '广东省储会',
    cidNumber: 'GD001-PRC0V-067440774-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'b1d8c292e8c9d80a8859cbc74e949fb915c30a52c69a99d2a29fefca9e8fc5dd',
      feeAccount:
          'e773f4ee2aa2fad4f85d6ca55fd11eab7fda40d1a6f36504dc9b6071288f84d2',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广西省公民储备委员会',
    cidShortName: '广西省储会',
    cidNumber: 'GX001-PRC0C-663454043-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '4b8cc2064fa68c762a68c9e065f318f8422d90fc7309815dfb44302d516a54a3',
      feeAccount:
          '3033e01652e5f51c26f8c71dd787160a1150147e9339ae112b97e74b7ae7b49e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '福建省公民储备委员会',
    cidShortName: '福建省储会',
    cidNumber: 'FJ001-PRC0I-389570546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'bf8c477d995bf67c2851b9c64125cbf13830afc50021ca3ffaa0e619cd27f898',
      feeAccount:
          '856ebbe38cf48b0e6ee4ef0f6a9c83905740c2d67474fe155013e2a8baa3427d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海南省公民储备委员会',
    cidShortName: '海南省储会',
    cidNumber: 'HN001-PRC0S-545676096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '79097820f9ed276d615a503224a7b9b3fecebebb66f05b5b728851c180a0a3f1',
      feeAccount:
          'ac16123e738704125a27477cec635f9ba68cad7560d724a19d403d06e68701d3',
    ),
  ),
  InstitutionInfo(
    cidFullName: '云南省公民储备委员会',
    cidShortName: '云南省储会',
    cidNumber: 'YN001-PRC0W-145427171-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'd701c9230195b5c35ca329b62c9d3dfdd9924b2eb000ddfe67d8414ef821e0a6',
      feeAccount:
          'a3c43e0f6a2451e24f0adac98cec23a81a10a66f0f828aff4c29680b83c4ea84',
    ),
  ),
  InstitutionInfo(
    cidFullName: '贵州省公民储备委员会',
    cidShortName: '贵州省储会',
    cidNumber: 'GZ001-PRC02-969970096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '7e818624c9f3c46f523612fcdb0c9459ce71191d29bac1a38cb9bcae3d8dbc84',
      feeAccount:
          '91123be54e191aa2f900d55875c14dcaecbf3bdcb8e6cfbab80745df630b4d76',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖南省公民储备委员会',
    cidShortName: '湖南省储会',
    cidNumber: 'HU001-PRC0P-400319700-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'fa74c11655fe1a950fd1064d9b342805d15b06ff379e36d2f1b5df0be0cb4a59',
      feeAccount:
          'd0b4e10c273b2923776435f272a201ada40f5a3673967b459125478b0fd439df',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江西省公民储备委员会',
    cidShortName: '江西省储会',
    cidNumber: 'JX001-PRC0J-458681566-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '3a31b63db36c21f8880889b7b9450a8db31d5e6ca689e4ef20d9c378bf8e9275',
      feeAccount:
          '55729a5417725684fd8285aba320521c2fbeb82da421eccf2c8f7e8e6fea3793',
    ),
  ),
  InstitutionInfo(
    cidFullName: '浙江省公民储备委员会',
    cidShortName: '浙江省储会',
    cidNumber: 'ZJ001-PRC08-471270801-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '1486ac02464ddd913bcbb93f93bafc8de69b79fd7fe9fc011c76332e415e07b5',
      feeAccount:
          'a2a83bc042df7d02ed8c2431fceb0512c14c9066c2761aab88b5696fd377fda0',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江苏省公民储备委员会',
    cidShortName: '江苏省储会',
    cidNumber: 'JS001-PRC0O-358467174-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '022757a4129d93d71270ab97a7bfe5b8adf8846bb4936d4fdaa05b029e088bd4',
      feeAccount:
          '865edf8b99bfa40100273f2d32e52292546adb431a3f88e9e348b4ff9dc5bcb0',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山东省公民储备委员会',
    cidShortName: '山东省储会',
    cidNumber: 'SD001-PRC07-027328848-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '3b4aa5d4c2d25ae694cd0ac18ba48f61c6c40d8a6433703aff5d40cddba08a38',
      feeAccount:
          'f0942f6f88e26921d964113dd42dbe14a86c0fdb82d2c37b0c63a0ba28b8abe8',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山西省公民储备委员会',
    cidShortName: '山西省储会',
    cidNumber: 'SX001-PRC0O-104465679-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a6910ef8bd8297ec15f970a27e57f0b6e9c6ffb5d59f5bdb2b89e38e6aea5da4',
      feeAccount:
          '89811950cd3ee67a9f3738d4df2b0542a3976b4c7b093ffeb4ea869a2f59e02d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河南省公民储备委员会',
    cidShortName: '河南省储会',
    cidNumber: 'HE001-PRC0S-849245626-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'd74ffd4b37c7aa43c24fe6f6ed63ebbdb846744164c755521044f138229d557e',
      feeAccount:
          'a501821113e4036ca8468abb5f7a2aeee6e270ecfdd2bb5e0c4a939b102053a4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河北省公民储备委员会',
    cidShortName: '河北省储会',
    cidNumber: 'HB001-PRC0W-499533387-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '332cb4fe167ed56c4559acded147cebc9554571ed1b857df4ee88b7a2a2144c9',
      feeAccount:
          'd730f66b03d00047ad7edde249fb1e542860074cbe73cc5f928716e99136587b',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖北省公民储备委员会',
    cidShortName: '湖北省储会',
    cidNumber: 'HI001-PRC0D-659443961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'eb21e0f3088320c9ff2139ad1d635485afb7a72c3de745c8410dfc5d126b1fcd',
      feeAccount:
          'c871114150b71c4902de7401ade1598c066f8a74d395edccb3289c755508f9d4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '陕西省公民储备委员会',
    cidShortName: '陕西省储会',
    cidNumber: 'SI001-PRC0T-711309909-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '3632eb7489038a0d73c8ef884840a6b3ba8e7b3a90e4fad1412f7dcd38fdff5a',
      feeAccount:
          '563522fe4c097f94260854021d9c187b01da73662d4bb17d507eb77ab0352482',
    ),
  ),
  InstitutionInfo(
    cidFullName: '重庆省公民储备委员会',
    cidShortName: '重庆省储会',
    cidNumber: 'CQ001-PRC06-478472058-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'ea5376d58108e950f0164b18d020fdc688cf56e79d3e79f4ab8719c55cc8d1e8',
      feeAccount:
          'c96e1ebb718aa02cd7eb0fecb76d6efa79142aa708768e3816b66781524031f9',
    ),
  ),
  InstitutionInfo(
    cidFullName: '四川省公民储备委员会',
    cidShortName: '四川省储会',
    cidNumber: 'SC001-PRC0Y-935659021-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '9abb47bf5f889cb54c9dd14e6dc02d5ae1a607d8125f1878e574cfa90969f4ff',
      feeAccount:
          'a78062f28462cbda4510c617faacaa3d5bbe4e32a6138a968a60beca9181b5d6',
    ),
  ),
  InstitutionInfo(
    cidFullName: '甘肃省公民储备委员会',
    cidShortName: '甘肃省储会',
    cidNumber: 'GS001-PRC0L-679051155-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a311932c50ca748f0fd24b65cd4f1dec3b3b115659ad07f33aa7b9d86f55ee10',
      feeAccount:
          '98c5afc5dd5a0e9d1306e3669554153a94a2136c15888892ba6ea542a4bb2d9b',
    ),
  ),
  InstitutionInfo(
    cidFullName: '北平省公民储备委员会',
    cidShortName: '北平省储会',
    cidNumber: 'BP001-PRC0R-189323546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '52b778d81a6cad379b2179992d35ac972fd8623759e8d69a931154fbccd674a2',
      feeAccount:
          'c99c4f381c37e986d623e36d2f8d33788e2d3de1e361ac062092b58d5d9f7013',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海滨省公民储备委员会',
    cidShortName: '海滨省储会',
    cidNumber: 'HA001-PRC0Y-214178517-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '8fd51d6d81d0bdd8bbedac4122823a57e4c953cc707c996b9cd7f32f72be43fc',
      feeAccount:
          'f167b19cda5bc7698902cf565a846c40b65e959e9ab1caa7975dbaa0a6f68ef2',
    ),
  ),
  InstitutionInfo(
    cidFullName: '松江省公民储备委员会',
    cidShortName: '松江省储会',
    cidNumber: 'SJ001-PRC09-044490898-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '515f2e19341b916e15b651efde76c1ee7b99f0fd2322bb3a06edfefcc0f4e8ef',
      feeAccount:
          'c5490cdb2919509dae64a7e5372c7aaacb4b27822c66c71f31a81ccac0adefa3',
    ),
  ),
  InstitutionInfo(
    cidFullName: '龙江省公民储备委员会',
    cidShortName: '龙江省储会',
    cidNumber: 'LJ001-PRC08-279890045-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '440f31d90586ba47e23fd4ac3f888ef5c83b87dfe08cdad113fdf07915abc985',
      feeAccount:
          '94ca5265a83623176d279115974b708973d7d6429b8cf80be1c7fc5b53e8de00',
    ),
  ),
  InstitutionInfo(
    cidFullName: '吉林省公民储备委员会',
    cidShortName: '吉林省储会',
    cidNumber: 'JL001-PRC05-850461124-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '10e7420d12ef291bb599bd32dc78d9684d839ff17384ed81f52c753606d2a28e',
      feeAccount:
          '9a9c0219ae2dde6b0334f78a22351d293df2bdb87b8d7ddaae0085303618a2eb',
    ),
  ),
  InstitutionInfo(
    cidFullName: '辽宁省公民储备委员会',
    cidShortName: '辽宁省储会',
    cidNumber: 'LI001-PRC0T-978545133-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '37f3817972e1abcce557e32a9c0b1c19afcae2e9baa3d8bf4aacf214b400c980',
      feeAccount:
          'bb0f6e2e864d31682419575981cafb39883748262ac4b35d93f49fcca5675b84',
    ),
  ),
  InstitutionInfo(
    cidFullName: '宁夏省公民储备委员会',
    cidShortName: '宁夏省储会',
    cidNumber: 'NX001-PRC0J-389752794-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'bf2fc01d64a26e95844f5572a06d66a86076db40d7cb0dedeb2d8bb32e3a197b',
      feeAccount:
          'c35312ece6c32f759156e4518149dc16430971fb050f15f5347984a572b8b72f',
    ),
  ),
  InstitutionInfo(
    cidFullName: '青海省公民储备委员会',
    cidShortName: '青海省储会',
    cidNumber: 'QH001-PRC0C-882026762-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a50d1007599d8a21693dcd074a26998ea0f2e40e4d99f1187abd9e1ac36b8c68',
      feeAccount:
          '1a520b9c4a52d65528d9cd12dbad23101b76dcfbe45d9e834573213a012d2c88',
    ),
  ),
  InstitutionInfo(
    cidFullName: '安徽省公民储备委员会',
    cidShortName: '安徽省储会',
    cidNumber: 'AH001-PRC00-589856828-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '64684bedb642195fb12d1b34e5519ed343e33dac934e7fb0a52c01b38139771c',
      feeAccount:
          '6ff371ae229dc712a1b0bcf2a1b4e20103f8e9bb72b28f89c069b3116ea23541',
    ),
  ),
  InstitutionInfo(
    cidFullName: '台湾省公民储备委员会',
    cidShortName: '台湾省储会',
    cidNumber: 'TW001-PRC07-265218823-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'a06e5338b4b24a93d6a4b39b8eb8ff48c105e76eefedf65515c564b0c83b9456',
      feeAccount:
          '0b32fe2697cecce360eea5b5da2518af8c4090d791faa74c4bd92399a6239d26',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西藏省公民储备委员会',
    cidShortName: '西藏省储会',
    cidNumber: 'XZ001-PRC02-435616961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'bd1bffac79a29f9b60f75e9b540ad77e99ac4d7a75e88e2efa5ed9903fc8c5df',
      feeAccount:
          'b99b6ca8055b1f96c3cc6a3d69e9cba26b565290b638170ea5f4060e2bc515d3',
    ),
  ),
  InstitutionInfo(
    cidFullName: '新疆省公民储备委员会',
    cidShortName: '新疆省储会',
    cidNumber: 'XJ001-PRC02-671044381-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '641352be82f3e5aa0e20ad6c2ef2031be08952a49443b7ad3b6253ae2fc50256',
      feeAccount:
          'b8b2398a8fa33560660fc32f8ed9efd4a18f3bb2f4adea3ca925af8e34edbc46',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西康省公民储备委员会',
    cidShortName: '西康省储会',
    cidNumber: 'XK001-PRC0P-695945392-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '30a8934b142898221901f1427d2ce5641f999f49d205856247b84b9de99ccaa8',
      feeAccount:
          'adac69e2efc0df73832edb17fcb5bc5a9e4c70fef792ef8dcdd2016dd85cc734',
    ),
  ),
  InstitutionInfo(
    cidFullName: '阿里省公民储备委员会',
    cidShortName: '阿里省储会',
    cidNumber: 'AL001-PRC0D-487847725-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'cb09539b0ffe54bb3c6f23c2b0bc73973dfd5a4dc540dafa739f21f7d4db2fb5',
      feeAccount:
          'e5b04cea0258e77835c3ff571393261725cf7c44e49f7c02ec1f727661fbd26f',
    ),
  ),
  InstitutionInfo(
    cidFullName: '葱岭省公民储备委员会',
    cidShortName: '葱岭省储会',
    cidNumber: 'CL001-PRC0J-771698743-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'ccd6e383073dd63480b2f2934d2f094e84a8725d0241fe4d510d8c7bf63e1467',
      feeAccount:
          '38d21944472e77fe0136fdc1399e12e88b6f015e41d0c7eb480585cf9d50359e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '伊犁省公民储备委员会',
    cidShortName: '伊犁省储会',
    cidNumber: 'YL001-PRC0Q-293160581-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          'c245039fa1a1f5849fb8a0daa514e99858a2c80419c83fa0056b5c0a281ea234',
      feeAccount:
          'dc663f25f95ee97fa18b1ca3b57ab9aaa8e771a34c19429cfdc4f590304871f9',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河西省公民储备委员会',
    cidShortName: '河西省储会',
    cidNumber: 'HX001-PRC0D-475713213-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '75bf0a42e5a1b5a0ba4ec5bd162cefe3df3ef2b0b46afa06a7d4e1d9072bc278',
      feeAccount:
          '36373c60b3be38bd2b9409976c85ea8231480f648f088e99dc7b1aa01857a928',
    ),
  ),
  InstitutionInfo(
    cidFullName: '昆仑省公民储备委员会',
    cidShortName: '昆仑省储会',
    cidNumber: 'KL001-PRC0O-091969119-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '4f81995eed779cc1a74e50e4cc1bde47b319d14b8929f90113bbac715ab86900',
      feeAccount:
          '96cdfa942d22f0e5b5003cd71c6f78a10a190236e63ae539af3fe7e0f3c40ab4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河套省公民储备委员会',
    cidShortName: '河套省储会',
    cidNumber: 'HT001-PRC00-481172908-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '3d69f0ef94d5fbc6be4d644e4232e30ccc98744b95a5512816fce98079356904',
      feeAccount:
          'b5d40b55fb67180b171f50909e0518673449afa8dfe906fdfee13eb46c9587ce',
    ),
  ),
  InstitutionInfo(
    cidFullName: '热河省公民储备委员会',
    cidShortName: '热河省储会',
    cidNumber: 'RH001-PRC0F-697831866-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '4a66873273f160a0c5140b698848549bdfb0ad8b457c6e296f213df6abb34850',
      feeAccount:
          '2627e52b7e6fd4e8f38b2c062f61598d7dadf067371eec919c0523b3c2de7dd7',
    ),
  ),
  InstitutionInfo(
    cidFullName: '兴安省公民储备委员会',
    cidShortName: '兴安省储会',
    cidNumber: 'XA001-PRC0H-384161601-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '3972b00978cb3b98fa95406ea657e3191e6f975c40b47156c9bbaca856e7d4c3',
      feeAccount:
          'ddaccca95b4300af85384f8beb2d5c3e91f2a83469eacf74cdca8081bfd432ee',
    ),
  ),
  InstitutionInfo(
    cidFullName: '合江省公民储备委员会',
    cidShortName: '合江省储会',
    cidNumber: 'HJ001-PRC0V-963948997-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAccount:
          '2a217cc580f1072581e4155324769dcab7beecbf95cf968ea54730c2e0a302c6',
      feeAccount:
          '7123ed2a52eeb9ad5d0ff109b1ad2b6ee0f8e636bf789807e631186ad115247d',
    ),
  ),
];

/// 省储行（43 个）。
const List<InstitutionInfo> kProvincialBanks = [
  InstitutionInfo(
    cidFullName: '中枢省公民储备银行',
    cidShortName: '中枢省储行',
    cidNumber: 'ZS001-PRB08-233384677-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'd596f0bdfb799af5e0b0aae1ebfc80357477eed0d83543c596377c5f235f4e51',
      feeAccount:
          '6f71263082e7a191df3e4a7a60785b3fc29ae53a72610a00e23a0786a3c99c2a',
      stakeAccount:
          'b43cfd7ef89206908fcf0e215e588c3757c2633cad0958d98dd438ef0ce07e92',
    ),
  ),
  InstitutionInfo(
    cidFullName: '岭南省公民储备银行',
    cidShortName: '岭南省储行',
    cidNumber: 'LN001-PRB0K-703127075-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'b43425e8d0ca5ade4f483c08a6bcc63bbc4843535c6c31f467973ac761dc2958',
      feeAccount:
          '35edaf95caf0906847e8cea0b4edadfb9b0de533c3a3ac88d5fdecffc3fb7ec3',
      stakeAccount:
          'f7c4d58ce71cda36e033fae776223186dfc7ae4637030961bd2f6a632a2c9701',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广东省公民储备银行',
    cidShortName: '广东省储行',
    cidNumber: 'GD001-PRB0T-239565809-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '1ea03625ab717b62b933c78647910347703d009ec3bdb45fce070c50cac25a8e',
      feeAccount:
          '975f9fe25143661147bad8ca476a8a7fa0139c2ffbfa071c6dc8462d49b7bc80',
      stakeAccount:
          'aa05a44ba302a84210cd0b67ba88517e8faa37a6945f5913fe726e851bbdd86d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '广西省公民储备银行',
    cidShortName: '广西省储行',
    cidNumber: 'GX001-PRB01-025559630-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'ac12eec509065b84ce2dcf73a0f53f297e0a90c1e9bbcfc68695b646650d28e1',
      feeAccount:
          'bff888d5935f7b2dc37064f890a0ff70d79d06919d1befdef74a20a659bac6de',
      stakeAccount:
          'd69b05dcf85a97a3f13fe5be6fe5c0445d1241f54d19f54e58c89bc1c7da3dad',
    ),
  ),
  InstitutionInfo(
    cidFullName: '福建省公民储备银行',
    cidShortName: '福建省储行',
    cidNumber: 'FJ001-PRB0V-504679612-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'cadf197b8da1664c76c9510c770a3dcd9e93efd9b2d0f551ad99b37307843e5f',
      feeAccount:
          '4950f8b99b36ae61b5cc4623868cfd8c506217c069620e5e101dd4f12ef777df',
      stakeAccount:
          'acf054e3ab6b4c0f000af1e0c6dd9212492bd8b145227cbf5509cfadc85e5495',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海南省公民储备银行',
    cidShortName: '海南省储行',
    cidNumber: 'HN001-PRB0P-723623074-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'dcd362d994821148bdbdf19202a3dfb8d03ad009e9b489c6f90aab7633d5ed79',
      feeAccount:
          'c2abbd24f1f766437ef92671c28c14c404c16ee476d18498d5f21f10b81bc2c9',
      stakeAccount:
          '8eadcc71b7b374554c7d3274d1b7319d383553b33992492db8ae2ead3be5bb10',
    ),
  ),
  InstitutionInfo(
    cidFullName: '云南省公民储备银行',
    cidShortName: '云南省储行',
    cidNumber: 'YN001-PRB08-692525950-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '959f12cfc94503e51c29a3b9c28df8391c3bb75f6bdf96984b4c3190d54b9abc',
      feeAccount:
          'a946fef0347da6e98913c96ff8b2d509ff5cd0650165b5209f3d0250db956a74',
      stakeAccount:
          '143e2343f8ce35e8764833a81882353d4ad8f17ea5fa39e6f5d94081b7cece93',
    ),
  ),
  InstitutionInfo(
    cidFullName: '贵州省公民储备银行',
    cidShortName: '贵州省储行',
    cidNumber: 'GZ001-PRB00-490015860-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'ead0df65ce9714644ba1ff34d85d3e8fc2c9f5009bd70229c067afb23b78ccb2',
      feeAccount:
          'f109494a140a92e8aea15aa84be3823bd0ac8ef12144af52bdb1d53ddcb63481',
      stakeAccount:
          '2a6b48b8fdb832d1cfc4f84db11951932e28eee024ff4d21f16df285dc717fe8',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖南省公民储备银行',
    cidShortName: '湖南省储行',
    cidNumber: 'HU001-PRB0F-084835673-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '497bee06c2b354a834f895844d761f9a913e1d6da5d4d45005e5c798cc025582',
      feeAccount:
          '544276f73276868fc9b62f20f0a0a908346927dd83abcbec2c176f0da3e81e41',
      stakeAccount:
          'f21baf39f86f0a9d574e0c57ccbe41e89b34d9e435981529c2c0ab8b1b5108c2',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江西省公民储备银行',
    cidShortName: '江西省储行',
    cidNumber: 'JX001-PRB09-243765987-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '707a0eda0aa3b9c4c2bd3da0e32d2ca03087371729e600b41f81d5c354deea2a',
      feeAccount:
          'f3816f72585c1d78b8a9f969b391779a84d4ace250532b1f06f7cf072bbb55d6',
      stakeAccount:
          'd011d2dbbab723ea01c4b66ce0077f08003cb619ffbfe1cfbf7326bcd32eb849',
    ),
  ),
  InstitutionInfo(
    cidFullName: '浙江省公民储备银行',
    cidShortName: '浙江省储行',
    cidNumber: 'ZJ001-PRB0R-296232973-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'cc792d64f59954256013e56704edeeac07c61f152411b24eff5d0feae26a378a',
      feeAccount:
          '54e8e7e5b8a5040b75b24d9727bfa66fe05892dedf7f3b359240e94867135666',
      stakeAccount:
          'c55ee028f582fa3b6bcdc7a0a0c4c19846312598e327453b68ef26d74f44c8d6',
    ),
  ),
  InstitutionInfo(
    cidFullName: '江苏省公民储备银行',
    cidShortName: '江苏省储行',
    cidNumber: 'JS001-PRB01-890774605-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '8b85832e3dd7d081c2212bc9f47a98d071a7d52757b611d0a8097ed5f8cbc030',
      feeAccount:
          '0fdac38f6cbc78b6f08d8bc3572e9225aae4fc826895a40485d733227892b48b',
      stakeAccount:
          '3da67736cd97c33102914c8d9d6dfd5ebf1949b82e83606c9c7c13f4c14b4779',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山东省公民储备银行',
    cidShortName: '山东省储行',
    cidNumber: 'SD001-PRB0G-114256751-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '9ad4231360fc3fbb0b66870a729c87a0b6736f1210d129f13e6c523904b73648',
      feeAccount:
          '354907ea36964a345e8c83a26fb7db0836bc4e40a86fce03474fa50ba147750a',
      stakeAccount:
          '7e0a6b1fee9c9b99a6cd37eac6a697800d40982ba55c0efc4a32d69d31986557',
    ),
  ),
  InstitutionInfo(
    cidFullName: '山西省公民储备银行',
    cidShortName: '山西省储行',
    cidNumber: 'SX001-PRB0K-520132196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '9dc27af87541e84fd0fd4f7a6a33929f18c81521c652e68097b17c2fefe382d9',
      feeAccount:
          '8eeabdeaca341513e7f2948603326861d1a719a6ab2bf8cd013e4698d76d1196',
      stakeAccount:
          '5f078dea08e5a72161a20e65222cd5ebd5464ffb27daee4f6165ccb486e5c393',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河南省公民储备银行',
    cidShortName: '河南省储行',
    cidNumber: 'HE001-PRB03-158889343-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'af6daa62333b5314c0433d826b899d50ac8887f4ed1a3211599232ba0dbb2017',
      feeAccount:
          'ec39b561a192bdc1d96e569408d805c3cebec2a6728f1d8dd5e2814739f59b61',
      stakeAccount:
          'f3bc751c654ab2f16b3815e61a4bb191ddc20fff6a639f3b27901a4ff12e953a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河北省公民储备银行',
    cidShortName: '河北省储行',
    cidNumber: 'HB001-PRB0Z-484022741-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'e4a0ee503ec962b51f3b8fd25dc546c57bda0485fac532e05589cac7059d23ff',
      feeAccount:
          '8fb11b8dcabc23f843737275546b55bb299808ee8bef6839d282d5ae5a606aba',
      stakeAccount:
          '5adf87cc2e0fc54e6bba0679c20f22b2bf0e37b2923fcf040508d648d968e333',
    ),
  ),
  InstitutionInfo(
    cidFullName: '湖北省公民储备银行',
    cidShortName: '湖北省储行',
    cidNumber: 'HI001-PRB0V-514948302-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '43e53e6189c46484a565ed518ca1353848dbc487034a891647dc74ec274c5457',
      feeAccount:
          'cc727a248cc3c6a46dc227a79091530a34c730d1ef562cab11667ef7e5d031dd',
      stakeAccount:
          '9c94380f23b1796ba47b534847a342874e5fb2567b2d775d03c40a54b0c26147',
    ),
  ),
  InstitutionInfo(
    cidFullName: '陕西省公民储备银行',
    cidShortName: '陕西省储行',
    cidNumber: 'SI001-PRB0N-245618374-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '2e36b35236e5d4fb7b33fe9bb6942d39cf1ddd3eae9127cbb923749b81b906b9',
      feeAccount:
          '5ade99aa8398f95e963b9eabbfd800bdd5cfdbf67e62be508af90f50bdbdc7c0',
      stakeAccount:
          '306341d085a49e2ca799f53d130d8eabf0441673d9d3176b1070373740c053ab',
    ),
  ),
  InstitutionInfo(
    cidFullName: '重庆省公民储备银行',
    cidShortName: '重庆省储行',
    cidNumber: 'CQ001-PRB0C-694162045-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '525a38c2ac3ae8a368cfe72116ef8994fa7ba28bbfb551c7754e6df1e67b9ae3',
      feeAccount:
          'dee2cad3716b68e724ff24c02d62d222ed0a1ca76aa2e22774ad473db30652b2',
      stakeAccount:
          '07f26d1ba26ae71afe6f2e34c36cfff48c9b58641f5bdd386b561cb5481e119d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '四川省公民储备银行',
    cidShortName: '四川省储行',
    cidNumber: 'SC001-PRB0Q-764253139-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'b6c2766ecb62eef4bc39aa8965dbdc642ca069af72d00466334c8fe111b1bbc0',
      feeAccount:
          '4fa37a9da649627776f898d42a3b06689215f40861c7a110d1b747729b4f1cbf',
      stakeAccount:
          '34557516eb459576cc5ac279745f9b76663909d8dfbd82d4c98c910c80406c3c',
    ),
  ),
  InstitutionInfo(
    cidFullName: '甘肃省公民储备银行',
    cidShortName: '甘肃省储行',
    cidNumber: 'GS001-PRB08-005784877-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '802f9eab320d0d2cd9f94aa7ab8ea8904f68cbf85b2c47e49560d1c63cc3115c',
      feeAccount:
          '353e7af1334691742cd1f37af2ef7cdb81c0a93ead4e562c2fe38425fca4b99c',
      stakeAccount:
          'd888b224bd45ffcb5b622e126e20ec0f813e65a913f60bd280bc670386de7d2d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '北平省公民储备银行',
    cidShortName: '北平省储行',
    cidNumber: 'BP001-PRB0Q-434307982-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'ec4326f895c157b1a12e0dcf919be7297a72a135197b39d877fb66720f0399d8',
      feeAccount:
          'f5c91376eb21901b97397512df25f5382e882cda861d332bb18194965f26b386',
      stakeAccount:
          'e5e0b667b3e93c01403a1c7ac55f8f5e739514a49f40d0722508e99139b6491a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '海滨省公民储备银行',
    cidShortName: '海滨省储行',
    cidNumber: 'HA001-PRB08-969179618-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '6b475b0a5b087f0c9c9eb06a99b456b57703884e2e7ef7b446e41d4e55625c78',
      feeAccount:
          '57ee79b3098e51316bdba23d03c2248728e6c57412805d77301ba18d1f986f04',
      stakeAccount:
          '0d6e02844616a73508da71bfe716e0ab9a4d2bcd605bb32b60cfb64148a1f244',
    ),
  ),
  InstitutionInfo(
    cidFullName: '松江省公民储备银行',
    cidShortName: '松江省储行',
    cidNumber: 'SJ001-PRB03-644104544-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '34f9ffc323303decc0af7c974310c143d076409fc6a78fad147a90fe3f5da761',
      feeAccount:
          '929b57e34eadb5dae91b2da9905de7427e5641fceeb7ee411af3a1f0bf1df949',
      stakeAccount:
          'ac0975acdf9979c1a34808493ec9928fab13d06408c4d35e80b0de60945c8763',
    ),
  ),
  InstitutionInfo(
    cidFullName: '龙江省公民储备银行',
    cidShortName: '龙江省储行',
    cidNumber: 'LJ001-PRB0T-280510636-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'a59b83df90281e1f8a347a863877122b8da36c2ee9e40ad66362b7238fa27caa',
      feeAccount:
          '96e5d1f75ab62be1017c118543f9707668a56bd0372fd6ed5b0ce9ddb1bdc688',
      stakeAccount:
          '2c57eb655e64bf3cd698375d5f207e7a0b0e5a5b54242b8ef32b588d1ad3a718',
    ),
  ),
  InstitutionInfo(
    cidFullName: '吉林省公民储备银行',
    cidShortName: '吉林省储行',
    cidNumber: 'JL001-PRB07-129935340-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'eea80d6d9f000b1e10158e3ebdbce3e4a4ac6edc86307003c13d81588e4bcda5',
      feeAccount:
          '704f407c2932118b70addab892d5f17e11e381a6daedb436d5686dea64bffda3',
      stakeAccount:
          '8602c912f127ddcf1818b026ed01460df1549d78fc2c6632113d9a44a2092706',
    ),
  ),
  InstitutionInfo(
    cidFullName: '辽宁省公民储备银行',
    cidShortName: '辽宁省储行',
    cidNumber: 'LI001-PRB0J-249814963-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '075fa9b4a0d8e567d70eab10bc50ed47d6b24d97b7c55ad7f9081be836e57e82',
      feeAccount:
          'd425f90230501f1db635ca5505ef4f3cee3cd751b06efbfd88496826a589f0d1',
      stakeAccount:
          '5c81e551b0840897b226f61fa4d36e88fa0aaaeb5957056be9deff612eb91800',
    ),
  ),
  InstitutionInfo(
    cidFullName: '宁夏省公民储备银行',
    cidShortName: '宁夏省储行',
    cidNumber: 'NX001-PRB0F-292327153-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'c8a805234ac76921c09c347a0700e1c64ec0e221cba0a37cc65e7604da27858a',
      feeAccount:
          '3e30c95641b6cf9410d00e39af5e32a6a9e9795ad8a0e891bab007a3451cfbea',
      stakeAccount:
          'adb3c2462f349d146d12594c1b71155b401f059ba0a96c8216decca0eb301da6',
    ),
  ),
  InstitutionInfo(
    cidFullName: '青海省公民储备银行',
    cidShortName: '青海省储行',
    cidNumber: 'QH001-PRB0V-075657014-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '0324a2d6e3672b5dc00b61271680e315f51a7b03ca4869b185b5edac48f0440c',
      feeAccount:
          '45e7f97a07796c2133ce96b54fd5579eb5b7ef9af750c3461d23e9b346d38495',
      stakeAccount:
          '4775b2a94fccb4a46d806d5165bb2c5d7346bba4da3689753492694f64686a84',
    ),
  ),
  InstitutionInfo(
    cidFullName: '安徽省公民储备银行',
    cidShortName: '安徽省储行',
    cidNumber: 'AH001-PRB0M-388477914-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '2f0255af996f4cc7c40687a60cb7a06de1f6922f2c6b2321f27d3f41c8555f40',
      feeAccount:
          'ffd0aecab04b1bff21ee86038caddc0b0af6a5901b1954d3683da140de0432a8',
      stakeAccount:
          'd6c4886dd6751c7ae2182ee0565d5a3b43750904dbf2309be396245dba79549e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '台湾省公民储备银行',
    cidShortName: '台湾省储行',
    cidNumber: 'TW001-PRB0S-266238196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '1f7f0e7babee820420ab765dabc6c1e098bb63894812b60cb6a51c11c1279a34',
      feeAccount:
          '46637aa4e6d9fb11ec53f07c2cd7ccd068e2bb28eecb00e8af0f9bcb3597b8af',
      stakeAccount:
          'd1579d8699f7ff50260bb4760409cad73a94ccdbd4009697f8229990adb85958',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西藏省公民储备银行',
    cidShortName: '西藏省储行',
    cidNumber: 'XZ001-PRB06-210788637-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '684bc4280b514bc0cad53e57d8f3658aeca21b90c499402e94d7a29ba612df51',
      feeAccount:
          'bd7f7f1d9b15e4712b0282ab6e782c208cc22384f05b4e7777432f6765ad70f1',
      stakeAccount:
          'a9095af00d6de39fbfce292b5ea3401c918b215f87aef636036b9974b06e9d2a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '新疆省公民储备银行',
    cidShortName: '新疆省储行',
    cidNumber: 'XJ001-PRB0V-233325633-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'b8e3769f585536aaf8894cee61ca62166af304f95e95489d71ff843b62e84c5d',
      feeAccount:
          'd12b8d9bb958ebe724241b319405b1d58b3ffa18211a2e3780ed124e87d52a57',
      stakeAccount:
          '3686027822290ec5fa11c78ecaac941e465859a8e4220cab48cbabb77c478c69',
    ),
  ),
  InstitutionInfo(
    cidFullName: '西康省公民储备银行',
    cidShortName: '西康省储行',
    cidNumber: 'XK001-PRB0Q-300401625-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '53a65be534139ba72d1e70224397fe850c72b7d6c3132b43c33b79dee41a6f58',
      feeAccount:
          'e146dc5ec4b6f7ae9ab727413420a56a2cbf66d2ca5b42de8e497535a4ebee30',
      stakeAccount:
          '8c984de177ccc400468e33ed2bcec0d96682f3e42cc7473a727700fd52351de0',
    ),
  ),
  InstitutionInfo(
    cidFullName: '阿里省公民储备银行',
    cidShortName: '阿里省储行',
    cidNumber: 'AL001-PRB0S-527686065-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'de5d0a88082f96ac4a635881937c78c535ec6e4d421a5df5a42c2c5305f38fc6',
      feeAccount:
          '7a1fe349b145beac2c5f9a49cdbc5f64e973c08483d5305e74faa35a48b2fa95',
      stakeAccount:
          'a4cad3c5a0b2787aef19c89389fe47d3940a90066489785b0853bb98c1745a8d',
    ),
  ),
  InstitutionInfo(
    cidFullName: '葱岭省公民储备银行',
    cidShortName: '葱岭省储行',
    cidNumber: 'CL001-PRB0Q-951267669-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '4d1f0b595642be7ba90398ad06940e933bb43fd309cb8426c69ded81eb54a796',
      feeAccount:
          '1bfb2d8da0f56339340003755d1671beeca0252e2fd18fa65f9c898037e7cb70',
      stakeAccount:
          '354756b6fc07d59f36c1fd3efbecec86eb3db2ec07fab6051af00b3395bd5e8e',
    ),
  ),
  InstitutionInfo(
    cidFullName: '伊犁省公民储备银行',
    cidShortName: '伊犁省储行',
    cidNumber: 'YL001-PRB0A-142800261-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'f621bf0f107d2501ba66c1d7df8b845330f7cd6584934e2981b5677d243fb2d7',
      feeAccount:
          '73a1328ff9ffb8dc8fccb912cc5e16677ed84b6b3fc51d2989aa2835926296e1',
      stakeAccount:
          'a8e35aeb25d0d293cca0ced98986896bbac8b2531912dac5d8c00cbdabe38c8a',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河西省公民储备银行',
    cidShortName: '河西省储行',
    cidNumber: 'HX001-PRB0F-215310265-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'cbc4c87576decd1eb50fd6310af6b8b4825d9553580f526a785d95698c6f0f45',
      feeAccount:
          '07cb607ec40886f34d00c70fc94f23ff4f3bb75e940cf1e290f25de6adc0fe9d',
      stakeAccount:
          'fe12d17cd39a4a36899cf8c9b3288f4753b8323abd742201a626b974a260dd3c',
    ),
  ),
  InstitutionInfo(
    cidFullName: '昆仑省公民储备银行',
    cidShortName: '昆仑省储行',
    cidNumber: 'KL001-PRB08-682838027-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '9dd41c065f299d5c057187e9c63a6efe759be99135c7d9d3d0295a06ed200508',
      feeAccount:
          'c6c49b0bed029580d810082966b578cf4c988fc9f936a19a7b6b51fc396a64cb',
      stakeAccount:
          'c7f850dc201c252d02fcb6cb5df5fee1ebd0a6533b4dbf9700fe64285ab28324',
    ),
  ),
  InstitutionInfo(
    cidFullName: '河套省公民储备银行',
    cidShortName: '河套省储行',
    cidNumber: 'HT001-PRB0L-210616196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '6bec3644a6636968f6a1680e398e787bb857f366879ae8e664282d6f4a89a5f2',
      feeAccount:
          'e0d5ba0a16e3569ab9b44241cf85e47ce80f5b31fb39ae4641b00a61d8f387cd',
      stakeAccount:
          '0c05da2c289fae6c5694b6e9c439fc8e7ea9a2d21a58c0c7262e67527351bbd4',
    ),
  ),
  InstitutionInfo(
    cidFullName: '热河省公民储备银行',
    cidShortName: '热河省储行',
    cidNumber: 'RH001-PRB0C-380830938-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          '751b054a53dfa53770ed0a06e9e51d9e8b77e09be8c19ec4864dca46b92a0828',
      feeAccount:
          'b62e186d54f7c04007633524bd2be96ad04165d9ae67e51968613929e86c975f',
      stakeAccount:
          '34efc01b44ff46e4d100d9f714cc563b497b9617302efb8575df3484363d7f09',
    ),
  ),
  InstitutionInfo(
    cidFullName: '兴安省公民储备银行',
    cidShortName: '兴安省储行',
    cidNumber: 'XA001-PRB0Q-928028839-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'df24be9645bd4ff2a87d5bda8dc42718e9a52634658863bdd21582642ad29579',
      feeAccount:
          '371d773fd5c6030232410693004122069fe0567ff6ba4771c9990f2fe6550d97',
      stakeAccount:
          'd742b21ecb0c8636c6bad0df482045204d62975e120b54b1b00dda98f0dde7ce',
    ),
  ),
  InstitutionInfo(
    cidFullName: '合江省公民储备银行',
    cidShortName: '合江省储行',
    cidNumber: 'HJ001-PRB0I-089279108-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAccount:
          'b51ed3b63cff1cad9d4cef6dda4ce231b78425baab3a443d06e151cfabaec49d',
      feeAccount:
          'c5cd54ad6cee8f3b230fefb2505b9c55c6951d2df22b08093c40db5b191bace3',
      stakeAccount:
          '6e72020687933242d0633720688e7afee5da3bcd948312dbedb888e0749dc148',
    ),
  ),
];
