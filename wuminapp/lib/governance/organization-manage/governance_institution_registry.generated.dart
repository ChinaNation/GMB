part of 'institution_registry.dart';

// 本文件由 tools/generate_wuminapp_governance_registry.mjs 自动生成。
// 中文注释：治理机构名称、sfid_number 和制度账户地址来自 runtime primitives；管理员必须动态读取链上 AdminsChange::AdminAccounts。

/// 国储会（1 个）。
const List<InstitutionInfo> kNationalCouncil = [
  InstitutionInfo(
    name: '国家公民储备委员会',
    sfidNumber: 'LN001-GCB05-944805165-2026',
    orgType: OrgType.nrc,
    accounts: InstitutionAccounts(
      mainAddress: '39936ebd8564c61f315662ff859d8fb5470ac3f1b4bfbf86746aff391d14db3d',
      feeAddress: '66d1de031e332facb67bd20ae428e187ae4bbf3caa0a1421bd0023c49fb228d3',
      safetyFundAddress: 'c878e700bde52b5c9c2a94bcf5296c4f6a75ca61b8e920a4e53a01c6da433e52',
      heFundAddress: 'ce19b7f0df3e9ba6c88b02364aa97cd1994df25aaa86c36e790ee85eea009f76',
    ),
  ),
];

/// 省储会（43 个）。
const List<InstitutionInfo> kProvincialCouncils = [
  InstitutionInfo(
    name: '中枢省公民储备委员会',
    sfidNumber: 'ZS001-GCB0R-016974075-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '1a2853434d5b7bb336670dab136b2479a029fdbbb447f49482f09be80660024a',
      feeAddress: '5bc1f22ef6e4147e61ac745f50e77f17656c0d6d789d600a1ffe014e5d44ab58',
    ),
  ),
  InstitutionInfo(
    name: '岭南省公民储备委员会',
    sfidNumber: 'LN001-GCB0I-773405642-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '9c30e747b1112ee82b8ad553ae19746328fcb7107d2ef67a4332e85071d0e197',
      feeAddress: '7ad835b5f7e6e72da144f5011c04b03a06cc0dec3bbf425300f5146acb09cf97',
    ),
  ),
  InstitutionInfo(
    name: '广东省公民储备委员会',
    sfidNumber: 'GD001-GCB08-067440774-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'e126c45b313e182a52a89dc9573fef09c34e08043ddeeaafe8524aa8132d0f2e',
      feeAddress: '5a8629fe12292877a16be390b77a1224cca65d011f828c514a2d1fc30a404a34',
    ),
  ),
  InstitutionInfo(
    name: '广西省公民储备委员会',
    sfidNumber: 'GX001-GCB0P-663454043-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '3494bb8aa47bbf4c4f8a7a4e102267709fe191257a238821b5cec3081d2408e7',
      feeAddress: '9f54f42e73c04c04fc5e8387c54e2a668f49fe1ceef0d6718933b1caa00de960',
    ),
  ),
  InstitutionInfo(
    name: '福建省公民储备委员会',
    sfidNumber: 'FJ001-GCB0V-389570546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'e2477f00c3529e3e703c9be2b659f68228ddacbbe7643f8772bb38c1ea7b1c43',
      feeAddress: '51d4ee733dd933dd920f3e14d9d71bb47e5218d68a7a312743945e6f01f2a5bb',
    ),
  ),
  InstitutionInfo(
    name: '海南省公民储备委员会',
    sfidNumber: 'HN001-GCB05-545676096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'c988fa2303032471ce2303e53952ffd3d36d0e0fa3222e628c484f41eeb864dd',
      feeAddress: '2fd7d9549ea1214deb2c32ad9b06971eda967aa3f76969bdfab0bc076bb4ca3c',
    ),
  ),
  InstitutionInfo(
    name: '云南省公民储备委员会',
    sfidNumber: 'YN001-GCB09-145427171-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '60888a2563a4ea470053b9da52377870011b1308d3947181eb4dd267b33abbec',
      feeAddress: '73cc53484e1f1d2ee9008510c925c1eea7d8cdf86598b48c1b8ae98d28b3efdd',
    ),
  ),
  InstitutionInfo(
    name: '贵州省公民储备委员会',
    sfidNumber: 'GZ001-GCB0F-969970096-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '28731a234ad27f5cca1ea23fa237278027341e6c305136262348c2566a96b243',
      feeAddress: 'c658f015dcb92160b14972e9eb4e031abfceaa4f63ed0bcf107ee72ec07be30a',
    ),
  ),
  InstitutionInfo(
    name: '湖南省公民储备委员会',
    sfidNumber: 'HU001-GCB02-400319700-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '0fc718e73994724ad6d3106df5bf301234e12395999999571702c3243f3b71e1',
      feeAddress: '6e6d0810a422c696909b59e48c8684cddfe8fc4b8474611b5d6c9c95e75c8588',
    ),
  ),
  InstitutionInfo(
    name: '江西省公民储备委员会',
    sfidNumber: 'JX001-GCB0W-458681566-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '329317ae1c34170c6bd84f7c82f63454b6771a8eefbb74678b47f50da21fe63b',
      feeAddress: 'd0c01c8a5d044a14bf6bea0421f026e87d2c3baee93d47754f64d234d0d8d584',
    ),
  ),
  InstitutionInfo(
    name: '浙江省公民储备委员会',
    sfidNumber: 'ZJ001-GCB0L-471270801-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'd23c3717cf286427e8f64dfe2ab2fffc497b2c877fcceda7bdb428274a709069',
      feeAddress: '60e899456f4ce442106861fe4bf25c05325ed3ccb89d4b4f638851546b3a5b97',
    ),
  ),
  InstitutionInfo(
    name: '江苏省公民储备委员会',
    sfidNumber: 'JS001-GCB01-358467174-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'fb3e341388899cd56990744505f62a57eee830febe59845757d957cc2a824559',
      feeAddress: '96050787ea606f1fec767fe7a128f544922fc25b42ba8088be27fe4094d50ae2',
    ),
  ),
  InstitutionInfo(
    name: '山东省公民储备委员会',
    sfidNumber: 'SD001-GCB0K-027328848-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'cdf192f50b63c3c039c9fc52c5699b2faa9b54216c4f392a3f3cb72c70271bb5',
      feeAddress: '11e10219d26dd19aaa99d7edf4d6067f06af6278edbdfbfb0e5c7fa9f375bb62',
    ),
  ),
  InstitutionInfo(
    name: '山西省公民储备委员会',
    sfidNumber: 'SX001-GCB01-104465679-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'a19117c80c894af3b644384d9aab951ae7f3f4b2315a225cbe0741f4f4d8ff50',
      feeAddress: '23504f08462e75df40322cb9b3711637bcd8a5359e55bd90d6d577ec92001bcc',
    ),
  ),
  InstitutionInfo(
    name: '河南省公民储备委员会',
    sfidNumber: 'HE001-GCB05-849245626-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '1077dedeae824a1e7a74fa6bee267cf32ccfc6c36137603a4f1978e5bff3bcb4',
      feeAddress: '787d12fac8c0bb711141c0fe5461c13148db0e5411f35a49756a030d399d51aa',
    ),
  ),
  InstitutionInfo(
    name: '河北省公民储备委员会',
    sfidNumber: 'HB001-GCB09-499533387-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '895d3722a1ba33819bdb871869fb4607976b8457857757968836d454920dffcd',
      feeAddress: '1dd3c397c680a5499cf1c45c1fb64420bd8da10d5a949e38c92d90bc092a95c8',
    ),
  ),
  InstitutionInfo(
    name: '湖北省公民储备委员会',
    sfidNumber: 'HI001-GCB0Q-659443961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'af605b27f80119dbc413519086d499930baf44b0ad403e2f6a410372a24f8ad2',
      feeAddress: '1527a93825adb1c68d5745a27e85ac047e04b0800cdaeb2954eae37e3d34c8ec',
    ),
  ),
  InstitutionInfo(
    name: '陕西省公民储备委员会',
    sfidNumber: 'SI001-GCB06-711309909-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '8710c1a5c5021fb374401b67ddaab7b3ced0e8c5dddd54dd3583d520d1e2e180',
      feeAddress: 'bc1a93347c9099e82afce1e0baf71cf4d69f1ba70d4bbc9557ab9fb730a495b5',
    ),
  ),
  InstitutionInfo(
    name: '重庆省公民储备委员会',
    sfidNumber: 'CQ001-GCB0J-478472058-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '5dadca1cf98697186d3cd2afafcf9652245d8639d297fbd642e42f73d6135e17',
      feeAddress: '8417860a0364f3e5038fa108d884d55210c7071315c242f277be4f7ece051c4f',
    ),
  ),
  InstitutionInfo(
    name: '四川省公民储备委员会',
    sfidNumber: 'SC001-GCB0B-935659021-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'b9b062b16f40ba8769d0fb2271b4e6430ed3a66b1cf9f4453f1985196c481c18',
      feeAddress: 'cbf48b589d4ceca43d90b1c1d8c3551931a3a6ab809f948aa19f9a0589edd8e6',
    ),
  ),
  InstitutionInfo(
    name: '甘肃省公民储备委员会',
    sfidNumber: 'GS001-GCB0Y-679051155-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '5482543d25b68467851621d8b4b33a947d924f4fa347415ce2c0ab61e8b079d8',
      feeAddress: 'c050ddd4e1f5e2cf4e366152f8d91df845901aeba3dbea50a4cadaa37998c275',
    ),
  ),
  InstitutionInfo(
    name: '北平省公民储备委员会',
    sfidNumber: 'BP001-GCB04-189323546-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'b318c753c28a570cbd81df49850592c30801afc48cafe41bb625d6e3a723cfdd',
      feeAddress: 'bc300e1f3a839b1645da45a9804ed8fccbf7399720fb1d1fd7fc5eef0c2496b2',
    ),
  ),
  InstitutionInfo(
    name: '海滨省公民储备委员会',
    sfidNumber: 'HA001-GCB0B-214178517-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '52fb0396a94447838aca50f8b539ed2a38997b337a7ec3981d2442ce009ff855',
      feeAddress: 'a98bffa5a69ff4a280f4035678972b2fd3e757c8884d419343f8dec60963d671',
    ),
  ),
  InstitutionInfo(
    name: '松江省公民储备委员会',
    sfidNumber: 'SJ001-GCB0M-044490898-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'f6dc51119d922808f58200ac090d412b2991db2885616efd3a04c4fe755d3216',
      feeAddress: 'a1408b8701b3ee02cf097d2ff27b6cd2d21bdb64dda21229bd9e7a33b004be36',
    ),
  ),
  InstitutionInfo(
    name: '龙江省公民储备委员会',
    sfidNumber: 'LJ001-GCB0L-279890045-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'c03742e08fc5f1dac94997fec3fce2eeaf1b445a606768ec90c2392410cce813',
      feeAddress: '622afe058889e3e176fb4b36860ef9cd6114cde638da7a38ba7f079f59779565',
    ),
  ),
  InstitutionInfo(
    name: '吉林省公民储备委员会',
    sfidNumber: 'JL001-GCB0I-850461124-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '29d7801c4d7bc1ef9bc7d7f8a9cfc9beb0c17f987ff38cc8b9e7d1bf5efe6449',
      feeAddress: '3739247dbc14c4ee98a745c0a2057b82c100398f73dd0e484a4a69b517ddebf7',
    ),
  ),
  InstitutionInfo(
    name: '辽宁省公民储备委员会',
    sfidNumber: 'LI001-GCB06-978545133-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '37c142883a379b8d56d47218408189e5c0b484370ff60243f777893d09a09efd',
      feeAddress: '74696b21f7cb01605901fe98e4565f13eba7e8dec791defdbbda1ecaf0a99622',
    ),
  ),
  InstitutionInfo(
    name: '宁夏省公民储备委员会',
    sfidNumber: 'NX001-GCB0W-389752794-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '5b0cef9dee73d5055921ab0f078a250ebc3bd7e2c44fa432cf53f9ffe22c285e',
      feeAddress: '4087d6ddcb2edaf6f5f1992864f344f4a8602c816d9e212b535e80389e287b57',
    ),
  ),
  InstitutionInfo(
    name: '青海省公民储备委员会',
    sfidNumber: 'QH001-GCB0P-882026762-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'ca0ffc8e3ec26cd679b1aa9f904f4342ab0793f87ad3b10de8326409d0517315',
      feeAddress: '6a52994daf82140cddbfcd965255c8f45ba75f258eef45741ac05695faebea01',
    ),
  ),
  InstitutionInfo(
    name: '安徽省公民储备委员会',
    sfidNumber: 'AH001-GCB0D-589856828-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'acecd778d6cfb5f2c53926cf7e56c61096b329509dd38ddd361ba2b5c12c6798',
      feeAddress: '03057c7f4e95fe220dfecc53f3246ed2f880969bfaf7af8500b04d76eff2bd0e',
    ),
  ),
  InstitutionInfo(
    name: '台湾省公民储备委员会',
    sfidNumber: 'TW001-GCB0K-265218823-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'eee2b2aff955485bda54faca13ab4ddb870c6a1110e27aea86f282de5f984553',
      feeAddress: '21784ee42ec77f61c144f85dc76a6eb9cf5dc12e29144ff08ff9f1df3d602ad5',
    ),
  ),
  InstitutionInfo(
    name: '西藏省公民储备委员会',
    sfidNumber: 'XZ001-GCB0F-435616961-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '9ecdeb733773a8a94e3ea429969154c64809786eb46259bbec28ecf221be7693',
      feeAddress: '4b6f895705f3339398f4ef2e94689f1db5bfb4a1440e2d81f9f7514003bb21fd',
    ),
  ),
  InstitutionInfo(
    name: '新疆省公民储备委员会',
    sfidNumber: 'XJ001-GCB0F-671044381-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'f20190cd9836cf8c17e50f39fb4723c50e99891dab252c4a54c54970115308c4',
      feeAddress: 'd5bad25c3e5e71064bd284797eb14c55ee1716db3a6bdc4ecc3475d6de71371b',
    ),
  ),
  InstitutionInfo(
    name: '西康省公民储备委员会',
    sfidNumber: 'XK001-GCB02-695945392-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'dc28bbd6b0cd88d1581cc0cc9d48fa7a11cb93f903b55da23b5ce2f73c2f2770',
      feeAddress: '0a9b0cacc6ed47d4810cdcdc42a1fc460834c35f60ae4393a9ca97b5c82fdee0',
    ),
  ),
  InstitutionInfo(
    name: '阿里省公民储备委员会',
    sfidNumber: 'AL001-GCB0Q-487847725-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'b853d97d7507b7eb35be86f4ed8d2ad4d9e0e472865a361c0d8d6f84ca02de23',
      feeAddress: '62c7a82421b6e734ca5531eb8246276229bb02bcef98f406da303139847ecba8',
    ),
  ),
  InstitutionInfo(
    name: '葱岭省公民储备委员会',
    sfidNumber: 'CL001-GCB0W-771698743-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'dc8477ea88ff303d553e7d546cb179a590ad64a4f39c5dd55ff6b351fa66af0e',
      feeAddress: '9e4543c79af9d748b246cbcb2218d7eb65fd0cc0d4c61325da17349431ab62b7',
    ),
  ),
  InstitutionInfo(
    name: '伊犁省公民储备委员会',
    sfidNumber: 'YL001-GCB0C-293160581-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '68eca56b1083c9432655fcac74c6c4468953024205d78f3a05c4744baf5be541',
      feeAddress: '4386d3bcd92aaec4b48215a4e5c7cf502b161937fc1b6958277e20d5dab54a35',
    ),
  ),
  InstitutionInfo(
    name: '河西省公民储备委员会',
    sfidNumber: 'HX001-GCB0Q-475713213-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '01d6e4602f4a959b4615c69355dacf9855a0bfe3f83fdc833c2b29c181b8a3ac',
      feeAddress: 'b6feeb6f734dbbc5348f35d9928942f11e2136924000380d365d4910677db349',
    ),
  ),
  InstitutionInfo(
    name: '昆仑省公民储备委员会',
    sfidNumber: 'KL001-GCB01-091969119-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '898c07c20f4a5a63d04ec7978800520fefd8c1128a0bd05e1b2248e8a605630b',
      feeAddress: 'de3e80497ff74120a15c51e9c20662f5fcc16376fa52e7b2213acf7adda34746',
    ),
  ),
  InstitutionInfo(
    name: '河套省公民储备委员会',
    sfidNumber: 'HT001-GCB0D-481172908-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'ab06c990c8adc427db3ee78c6e66c0a4c3677dc4deef5d3456c118e66d2c7048',
      feeAddress: '6b53378f00158813218465eb2000841934d4fe233df624cefba5dca4470e3dcc',
    ),
  ),
  InstitutionInfo(
    name: '热河省公民储备委员会',
    sfidNumber: 'RH001-GCB0S-697831866-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '3fdf0f790bf1d1dd64cd19b3eda163bc2deb89587fd96d4cb4d9f4ff2a15a54c',
      feeAddress: '4d4d52d030353a3901cdf38f59c7fcd0172fe510955fe5af963b95c6c95fd841',
    ),
  ),
  InstitutionInfo(
    name: '兴安省公民储备委员会',
    sfidNumber: 'XA001-GCB0U-384161601-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: '7a160d892cc77d03c4ea89ab53f953b7de101443ba2288b284d4273979fa95f8',
      feeAddress: 'baa48e7a6228b3b9caa2e6418d0cc6409457088c361c49d0e92f8a7bc7841e64',
    ),
  ),
  InstitutionInfo(
    name: '合江省公民储备委员会',
    sfidNumber: 'HJ001-GCB08-963948997-2026',
    orgType: OrgType.prc,
    accounts: InstitutionAccounts(
      mainAddress: 'f0b6a54689d5c9c32977130b1b47db66c4b56728a84108dba5f6ae05317ad51d',
      feeAddress: '0b87b228498811433d50bf7abd723a18bef47d9c9879033eb1365a02cbb1005c',
    ),
  ),
];

/// 省储行（43 个）。
const List<InstitutionInfo> kProvincialBanks = [
  InstitutionInfo(
    name: '中枢省公民储备银行',
    sfidNumber: 'ZS001-SCH1E-233384677-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'f8849691c497ed6c294ad61a2c29e8ace130e0ddcc5b0b7287a5a96c4870ed50',
      feeAddress: '50a039748269d7687aa48e4c73ffe9d116232d8e2ffe272af95b18496bbabd80',
      stakeAddress: 'e9a0f626c640cadb967e798a29f4fc1d3f8f72a02e1595c930ee7a6e7a0eb128',
    ),
  ),
  InstitutionInfo(
    name: '岭南省公民储备银行',
    sfidNumber: 'LN001-SCH1Q-703127075-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '97050936abfe52a30359bdd3286c17c6df4bd2b5b3786e914a7b11c0db224e21',
      feeAddress: '49091bc468cfd85f2ebe053f8fd6efc75df07ae9e3c4a3088d75c476297fb617',
      stakeAddress: '4bbc302666d71b426478d140445e0f963f5a86a8a25daef58783111aa0836e2e',
    ),
  ),
  InstitutionInfo(
    name: '广东省公民储备银行',
    sfidNumber: 'GD001-SCH1Z-239565809-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'd9bb0b5768bdc207ed788b4038100b3c167ac83ee1aaadde5769c1fda4488095',
      feeAddress: '0844e9170e6f7d1c4b3991ab685f5ca73db2af09fc5c55702ff91b799f0c598e',
      stakeAddress: '1ade5c85b38807b457c54c035268b01c09565209f938ae97ef8bbf610f5cbf39',
    ),
  ),
  InstitutionInfo(
    name: '广西省公民储备银行',
    sfidNumber: 'GX001-SCH17-025559630-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'b1ce2f090c528533874528f3f8ed68b5762c930004b776dabb7a03d2bd69a4f0',
      feeAddress: '168f3228f9a7d1129d21ffcb8c2e64b67ec7366ce949fdcf861d70a297487379',
      stakeAddress: 'd6f6b4e574c14875755d1f3a66bf4f68b54e9f2b137effd35ab00bb49ce77883',
    ),
  ),
  InstitutionInfo(
    name: '福建省公民储备银行',
    sfidNumber: 'FJ001-SCH11-504679612-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'a996934060a63311c2a9235888e36368d1db19ea9e085a36e32385eacc849e80',
      feeAddress: '3d44f89c16d2fdcececfc361a32a1a20c2e62bd6e81b9e60569095e057ab5cd2',
      stakeAddress: '188422119d3139d5946fec0bb656b38243833fe5fd68a443145e642064d237b8',
    ),
  ),
  InstitutionInfo(
    name: '海南省公民储备银行',
    sfidNumber: 'HN001-SCH1V-723623074-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'cfd4fc4cfab6b1aed99296e8e6e2036548a2b8a1f61d21a1381b18eeaccb5a81',
      feeAddress: '9059bc2f7cd45fa20e31a65283dfe41c2df30f6de9f08c264e87dd14af02010c',
      stakeAddress: 'caac495aa3ec6441978337ad4769ed0a4daee8ebf216d50c94313572e321a775',
    ),
  ),
  InstitutionInfo(
    name: '云南省公民储备银行',
    sfidNumber: 'YN001-SCH1E-692525950-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '9b5e69c3934c1f334b04cf0b6f13aad477b12b16c0f7e801ba01d21651ce8d60',
      feeAddress: '544413ba1a80de93f2452206bcb0a32fc827cadbe6bbdab026b677d1b9a6a648',
      stakeAddress: 'a3ee7ee59ccc65ca3fa3345c5c569d165cac5fde1f374bd8e57f83f63a5887b2',
    ),
  ),
  InstitutionInfo(
    name: '贵州省公民储备银行',
    sfidNumber: 'GZ001-SCH16-490015860-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '7684b573df3b772a35eb455e618b6a21c6d7b5e5a54af70ab7c8480a920d9a6e',
      feeAddress: 'a4f88bb6e978e06c3499e4030fc859b76941e45956828d8ec1dbe9510ef09afb',
      stakeAddress: 'c74e7ae830cde13f3af2b7d7893c455117a18656d635b5c93c81ead24fc43b72',
    ),
  ),
  InstitutionInfo(
    name: '湖南省公民储备银行',
    sfidNumber: 'HU001-SCH1L-084835673-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'c2747ca4ceec3f51d95cffeb5db3091d76e0dcc2f1757370c33a3f488b16373c',
      feeAddress: '75b1d9ffa61559df9cad189e8b8b25a1ce261f9154025caacabe72765ec9779a',
      stakeAddress: 'fab5a4e811ee8f217ad70f9f4ea07b1f7f829089ce2d7493c9518d44db516982',
    ),
  ),
  InstitutionInfo(
    name: '江西省公民储备银行',
    sfidNumber: 'JX001-SCH1F-243765987-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'b4874ab93df83815c1d4a48c722978afbf644d1433428c397b911e639f022a07',
      feeAddress: '65ea7bc5635e5516d56acf71fd9a185798db1f94af9c65a90080e3e6b48ffe6b',
      stakeAddress: '07f3a9c32b72c6945d943bb5c36cc1a85162ea881d3093754b51b129de6683d1',
    ),
  ),
  InstitutionInfo(
    name: '浙江省公民储备银行',
    sfidNumber: 'ZJ001-SCH1X-296232973-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '73a64e68988972eca2b41f6d85e0d95c5474e4ac76b0feaf0d223333b2f0e37c',
      feeAddress: '359fc9148334b7ad122edd18b98ec604f36f74c9a91d15d1d90092577bb78ff1',
      stakeAddress: 'e33930cac9a853965370b56d2b6f8c90170d5bba196cdf20fc9e8621a0ce7c45',
    ),
  ),
  InstitutionInfo(
    name: '江苏省公民储备银行',
    sfidNumber: 'JS001-SCH17-890774605-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '55b92434254f11ae30587a85f24a194ba389dd0aea844205780341bcca51de61',
      feeAddress: 'a8fde60007174359f3eec91e786ac619387f475e9aee71b3ef9d21435dcfcfcc',
      stakeAddress: '85d4f2e7689f292718ff91dbacd162c2f70886a66a17ca52a75044379186b0b1',
    ),
  ),
  InstitutionInfo(
    name: '山东省公民储备银行',
    sfidNumber: 'SD001-SCH1M-114256751-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '36fc82123bbc9313a2dd68a0ef7c1966a2f7cef51f6d3f9753032b5bc36dfd8f',
      feeAddress: 'c8df44a3c673c6561d44962c90af0aa29ed2b9506d9e830e5c71cb27fd056c5a',
      stakeAddress: 'a236b79ce5d79230d3d9c96fb23ba32291db1fe4be0b6eacbe27bf6af3337d36',
    ),
  ),
  InstitutionInfo(
    name: '山西省公民储备银行',
    sfidNumber: 'SX001-SCH1Q-520132196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'd2401d4463ce83397a4d6ac6a6049c5911065932fa27ff7a834fdca5093977f4',
      feeAddress: '959fc2049e0dc049ffbf6cf43f9407bee22f33d78b78da5eefd14c4821b27958',
      stakeAddress: '3d5f78b7afcb17aa13874971dd782470948fc09da4f1b19c804a27fbf1cfb552',
    ),
  ),
  InstitutionInfo(
    name: '河南省公民储备银行',
    sfidNumber: 'HE001-SCH19-158889343-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '36544450f6ed4aed9f97ec064a9dbbd706382fa1223fdb6187cd66bf12e60b7b',
      feeAddress: 'e8a0b874febb29f50e88a529f4f0589f9a5980bd9881d0566e6539a99315a768',
      stakeAddress: '64bf92a71f9e0afd0b531f1e7d8e2fd5e0ac5885b8c87b838b22644bff931455',
    ),
  ),
  InstitutionInfo(
    name: '河北省公民储备银行',
    sfidNumber: 'HB001-SCH15-484022741-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '7dbf135536581411a7f25ec9a46f3e17d0b138ae55797cbe4eb40c9475e9aeaf',
      feeAddress: 'bf40d7f470591c4d12a3c28d384016b479b567207917ca34da378cb65d31aa49',
      stakeAddress: '2f856da0e1057e14adaa20a91e26404baea9cc7ff02556f2303b9f2c41bced38',
    ),
  ),
  InstitutionInfo(
    name: '湖北省公民储备银行',
    sfidNumber: 'HI001-SCH11-514948302-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '24e51c132e6213dc8538289903f66d1fd1dab3c26355372af8ef74b99a2fc0db',
      feeAddress: 'e86174e5b5fbb0d84aef0387e945b1f4a9560bf88395112904b889b85bae8c64',
      stakeAddress: '7a8c88589966ebfbcedd4f068a9913808525bfefc146c33cc0de93b7482da444',
    ),
  ),
  InstitutionInfo(
    name: '陕西省公民储备银行',
    sfidNumber: 'SI001-SCH1T-245618374-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'a96a8b6edc42fd7db69441df4433b78552cae03e660191026b8dc59d38aae275',
      feeAddress: 'd02b42a9954267e18113243ff6974188479a6da1c78f6bf11114dfae4aebaea0',
      stakeAddress: 'a191af21765f937ab0b2f590e0c7176e1d574e33148c325ad9cf08582b6a6c34',
    ),
  ),
  InstitutionInfo(
    name: '重庆省公民储备银行',
    sfidNumber: 'CQ001-SCH1I-694162045-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'df51f1214bf79685dad2b0950d61bdbbea31c4f00787eaf968733f3adc0a02ab',
      feeAddress: '8cd711c0ec96975dfbac9c7a1c40b6d2c9d45e4597f2c8b23528b4c139e0ae22',
      stakeAddress: 'bf9c07f46e3200597b502eb79b54b121d5dbf409d82a4442de2c9dc9656fd2c7',
    ),
  ),
  InstitutionInfo(
    name: '四川省公民储备银行',
    sfidNumber: 'SC001-SCH1W-764253139-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '8dd554c6b1751227e1d6311e9e6dc5b2e542e8b21b1aa1f51a43b0d0f27e67ac',
      feeAddress: 'c80c0d9ae4fa2c87909dedd8b525df90d1f0ab47337a828cd04fd05e93125bcb',
      stakeAddress: 'a3612a88b871c57dce8cc1f4bc2958e76b3089845e2e2b9d8554d7f57f33ad31',
    ),
  ),
  InstitutionInfo(
    name: '甘肃省公民储备银行',
    sfidNumber: 'GS001-SCH1E-005784877-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '1c368b84eb015714752e01c816ea1c07bca95dc6747323326f0d1383abd3b623',
      feeAddress: '7df2c91adb9d56b64a2ce51acf6dcd4183edac75049a18bef591eb1311baab3c',
      stakeAddress: '42f5a1038c056f34b0906cf87c50237321d1f2eaace2b6f2973136c61b8d6016',
    ),
  ),
  InstitutionInfo(
    name: '北平省公民储备银行',
    sfidNumber: 'BP001-SCH1W-434307982-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '9e93a45e5a8a3cb59222b93a44e04e0929df47635be2bd44a5bf09cd5fac4f1b',
      feeAddress: '7f6cb359b8abb77b7f30ae34bac21d4ceabff90d8673d0b9dd6e5bbb4fba3a2e',
      stakeAddress: '38a16891215c7a75ca47092d8717da4920c626c8d059f462c3fcb31ecf6ac355',
    ),
  ),
  InstitutionInfo(
    name: '海滨省公民储备银行',
    sfidNumber: 'HA001-SCH1E-969179618-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'af305d6bd94d08f4536d0a573077804ee0704c25e085c8f47fef9a862f19b848',
      feeAddress: '28f8e719d05979fe4fef68e8e34fbbe5939290de197aa1a39bd78a328e4cfcff',
      stakeAddress: '20aa094f7a2fe90bbf5a96ff23fad1b1dc5ba71a392daae726d00fba627c8677',
    ),
  ),
  InstitutionInfo(
    name: '松江省公民储备银行',
    sfidNumber: 'SJ001-SCH19-644104544-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'fe7e52efbd4a81c0c1d18c231e3a9321a132a5da7406d44034bf6bb1a8627870',
      feeAddress: '87cb6eb6612ede3e8ae56d9e74086b09cda263d0389fe3ad653c9ebc2f0e3a78',
      stakeAddress: '0742f1db10c6bd856c39ec9903b994d47ab07f7486eec3be9c3ed1651e10499f',
    ),
  ),
  InstitutionInfo(
    name: '龙江省公民储备银行',
    sfidNumber: 'LJ001-SCH1Z-280510636-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '069aaa597bdb2d2d682ee7c44615e035dee75f1df0fb9eb098f3571c2d045bd8',
      feeAddress: '08d796def30e246b6fac0d245e76746e3a4167cb638b0badb6fc915be65f1f87',
      stakeAddress: '70e230d35264a726988b762c5ad26b72d4f95cda0fa7a0491b4d5df38b0b1224',
    ),
  ),
  InstitutionInfo(
    name: '吉林省公民储备银行',
    sfidNumber: 'JL001-SCH1D-129935340-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'dd3c7135c61368556911362f5c57da40c7772d71e027f2b1fcd499a7e18082f0',
      feeAddress: '274ffb6b95feeeff5683a54014b7d2e9926fd09f359cc401ca8dbd7b20fe25c5',
      stakeAddress: 'c7b82022aa1a5e210e6c63b51c77390e85e4607bf606a6130475849b06c5e56b',
    ),
  ),
  InstitutionInfo(
    name: '辽宁省公民储备银行',
    sfidNumber: 'LI001-SCH1P-249814963-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '2304d0242072d00d9b116224c4f09549e9be59e411b78c601542b606a16b3289',
      feeAddress: '3b468ce35bd63c997d91efcef179ba9dce5dc7506e89b839ebbbeca0e33e714b',
      stakeAddress: 'fcf5eb3cd66e5bb48748e0556583ece6c2ed851fc093844725f90a6ba845428a',
    ),
  ),
  InstitutionInfo(
    name: '宁夏省公民储备银行',
    sfidNumber: 'NX001-SCH1L-292327153-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '782864a3277c09bc2135d7529c34f7eb88f5d14b5d446db0cc12f68710803038',
      feeAddress: '6d656f51e9070406c7041caebcda93a8b87c80280764b282c98617e495898d4c',
      stakeAddress: '2881243de1f9ebe6e7021586efca46de4793eaf03d562d409c62c1c801adf5d8',
    ),
  ),
  InstitutionInfo(
    name: '青海省公民储备银行',
    sfidNumber: 'QH001-SCH11-075657014-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '608a1e04f7de9de454cb9d49641aa999eb644999d0c75ae17d8514bf16e1b1d6',
      feeAddress: '425565245d2298adb3789d9f207478cf3747b339eea6f031ea4fb3a8d001be68',
      stakeAddress: '7537eb5c0fb1d9b84a8a10329cbc919c5f31f787cab11da3efb24a6cd922620f',
    ),
  ),
  InstitutionInfo(
    name: '安徽省公民储备银行',
    sfidNumber: 'AH001-SCH1S-388477914-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '2030c382c5bad0fb3a6159e2a92b9502550951e1d62129c740087e6d50292ef0',
      feeAddress: '2131af1bca9075de2cecd8015bcb8a542b4a12ea71af7d9023680dd1985cf898',
      stakeAddress: '751e2ef0028880714823262de9a56f11fd89538ad903b939f9268065d8362ce3',
    ),
  ),
  InstitutionInfo(
    name: '台湾省公民储备银行',
    sfidNumber: 'TW001-SCH1Y-266238196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'bc95ab3ca2c658b5269d73bcd3e68a95ed4e41af7133947353d3681d1f84b72b',
      feeAddress: '85986826cb86e8e221bbb3114fe7faebccd4fad983aeeda32d8d2614abd0a585',
      stakeAddress: 'c54c7fff799abcf4cf493500d815ce883ce9497b330a387295fcc8faa60b9ff4',
    ),
  ),
  InstitutionInfo(
    name: '西藏省公民储备银行',
    sfidNumber: 'XZ001-SCH1C-210788637-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '9fb5edba282ba9cee81a8faa7e018f0ab173c52690b44509c65d7a8a0e6a427a',
      feeAddress: 'b847299b4a6018e765093801c73cbb4cff342f43aa54f4d3ac5002eecf3aed86',
      stakeAddress: '50915414c7f0235c23c051473a46f065762d5523b872f5db555eb5e57f78e502',
    ),
  ),
  InstitutionInfo(
    name: '新疆省公民储备银行',
    sfidNumber: 'XJ001-SCH11-233325633-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'a64f282dd3c24df09a6512cdd72fcd448b480a8f59dc460960a253839ca9332f',
      feeAddress: '6721c2a98f87c3a2f084bf5533c980e38c1bb65dc378a6a767f1470c14ba672e',
      stakeAddress: 'c35ac7e9d5b737e87d3a44ab6d2bef0f58b2b19788cdc61deb0434571d7a88cf',
    ),
  ),
  InstitutionInfo(
    name: '西康省公民储备银行',
    sfidNumber: 'XK001-SCH1W-300401625-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '99fa20a38fa94a3ce1830d49a7090a32069cd9b8688da9349633259e3f941ffa',
      feeAddress: 'a86a7d1adf7d73770c2c9832a72f1112f49988e8fb7d94315ed9488e8e754cc7',
      stakeAddress: 'd6cfa334492abfa002ab62131ad758e87724e5e7655defa70163140abce0f842',
    ),
  ),
  InstitutionInfo(
    name: '阿里省公民储备银行',
    sfidNumber: 'AL001-SCH1Y-527686065-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '68efded8ffe41856a1336773b22b57b70c22ea55b4754fa63de0b8f098a7cff0',
      feeAddress: 'a2d83c5492bc011d3a5f2f7c322139e7156c3b9ec90853b76dc97ed6b784e8af',
      stakeAddress: 'd47bb6901f37f8852572a1e58c9395e9e5ecc9aaed6ae4709554c1fb3460949d',
    ),
  ),
  InstitutionInfo(
    name: '葱岭省公民储备银行',
    sfidNumber: 'CL001-SCH1W-951267669-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'fc926117c924b15e85973d40a5c62f4989325c8e6000a217a5424bdaf4a50c02',
      feeAddress: '9b73e74d4b4bdf7b90b9314a522484e0eeabafd17613d0184423468101b52e2b',
      stakeAddress: '92f93b885902c91363a7d787c0e613ac58d0eec92f1a82039c6535c6d9e53bd9',
    ),
  ),
  InstitutionInfo(
    name: '伊犁省公民储备银行',
    sfidNumber: 'YL001-SCH1P-142800261-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '38f18eef0f0d2d51dc379b965ab22a7a6516d58313279553f9b04af1d188070b',
      feeAddress: '83575511e71fde28993976074a923b2f5f0f4a92a1e743771e4c76990b058814',
      stakeAddress: '9d2de234353a73b7149bb1db9fb7d272f87c9ed27cdba57979cdf048317b07a5',
    ),
  ),
  InstitutionInfo(
    name: '河西省公民储备银行',
    sfidNumber: 'HX001-SCH1L-215310265-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '59fe2f5bfcae83abf98853f063db01e3efceae46dee32e14b8aa9d9c74fef8de',
      feeAddress: '176e20195e7074351855893f304a38b4a70c83d9987b64c6cc7d5bb55a4609b8',
      stakeAddress: 'df06db195678afb328e33e013c832986e649e147c5cb622a5911409331fa1f97',
    ),
  ),
  InstitutionInfo(
    name: '昆仑省公民储备银行',
    sfidNumber: 'KL001-SCH1E-682838027-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '5216828b35be36f538a7ff0e69a8346a7b635d1ae34229e6206b2c76a5f35455',
      feeAddress: 'aedd82e0b7cd96fee49bb306a267f9c61d4bcbbb996c452888a4c0a7d5622b04',
      stakeAddress: 'fe13ea29e17c914af06753cd357c1bf4b6de431c5aa8df35c49a2bf54c83733f',
    ),
  ),
  InstitutionInfo(
    name: '河套省公民储备银行',
    sfidNumber: 'HT001-SCH1R-210616196-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '6445e4baab20cdbf9389fc8fdfe1a86fc30c22c80207078f7353f1e8acfc1418',
      feeAddress: 'fde86223c1945616182ce78304cba4c00c99799a990b528c4e1c6350a31dd908',
      stakeAddress: '411833c18dec2a0de42936c834c5f0ee79c3c756dbe0a329af54e03f4a18c212',
    ),
  ),
  InstitutionInfo(
    name: '热河省公民储备银行',
    sfidNumber: 'RH001-SCH1I-380830938-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '4c41c13662e52d3393eed706c5d714cfb2b3dc7851c01a6dc2cbb56990129a80',
      feeAddress: '014f9eb856f901b635680c53cb7d266ca67a98db8bf89ea27244e9fc7809d25d',
      stakeAddress: '092e70ab0742be7c478381d2b055e3d653d27b3329890eadd154cd6a7aed2853',
    ),
  ),
  InstitutionInfo(
    name: '兴安省公民储备银行',
    sfidNumber: 'XA001-SCH1W-928028839-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: '9781163b146fd7ba10720bc3a7b3505f44b1368824282c237b3a1cfffb548977',
      feeAddress: '88f4861da2db2982a187efc88eb7288a7c5cd57eeb98128ce3852bd81929f4bf',
      stakeAddress: '740fbb1ff1f1cf77d27ee50836983d82aa6079746b8d32786548e6c295ab9cd3',
    ),
  ),
  InstitutionInfo(
    name: '合江省公民储备银行',
    sfidNumber: 'HJ001-SCH1O-089279108-2026',
    orgType: OrgType.prb,
    accounts: InstitutionAccounts(
      mainAddress: 'b8486f8e93b974617597e1e681c4b39cfc329702bcd2b93ff7419656766a35f3',
      feeAddress: '6ade1894079bad5408e421eee261b21b4d507801aecae0ff9d02203ff0572997',
      stakeAddress: 'aba2fc9d4b85dd51f9adafe5a71e9114481673d52fb59c0a4c5f5f6dddee559c',
    ),
  ),
];
