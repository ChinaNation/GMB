//! 省储行节点的常量
//! 包含：省储行ID（pallet_id）、节点名称（node_name）、交易账户（pallet_address）、管理员列表（admins）；
//! 其中 admins 为该节点的创世管理员公钥数组，用于多签权限控制，可通过内部投票更换。

use sp_core::H256;

/// 单个省储行节点的常量结构
pub struct ShengBankNodeConst {
    pub pallet_id: &'static str,
    pub node_name: &'static str,
    pub citizens_number: u64,
    pub stake_amount: u128,
    pub pallet_address: [u8; 32],
    pub keyless_address: [u8; 32],
    pub admins: &'static [[u8; 32]],
}

/// 所有省储行节点数组
pub const SHENG_BANK_NODES: &[ShengBankNodeConst] = &[
    // ======================== 01 中枢省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbzss01",
        node_name: "中枢省公民储备银行权益节点",
        citizens_number:10_913_902,
        stake_amount:10_913_902_0000,
        pallet_address: hex!("6d6f646c7072627a737330310000000000000000000000000000000000000000"),
        keyless_address:hex!("21bc9e12d717e4d55666501fd21f8f3fdfbf98d513d6584424f34162397ac1be"),
        admins: &[
            hex!("7a24e290379c6e458f5372246629a739064b01de97ca85e4008a0c52124ffa2b"),hex!("84a26299f660c0baa0af46ca33169aefaa41b35be06f2b8a66b4a20b7038d60b"),hex!("ece6ca93610b323a5a20c96d52078da1d90d2c4057784157194365ba51c61f6f"),
            hex!("f47c3849f4eb25e8306e7d525ef8f15d699ebe2f420abc53927ca7c1b63ae90b"),hex!("8002ed809a2fa9767d0def1cd0dede8dd8fa424c8468211ff07a684dd3695555"),hex!("40dd8262f62d8411be3795d187d226047271b4080033d22da9511dfacd106935"),
            hex!("941704d523594e4f2a3e6621e4970b2b037fb8db8252850f9fa202a5d373a35c"),hex!("16361247bc1a40a999aeaeb0ebd782801e95e7d023264501484568078f092c71"),hex!("3016f8ed2807e473a17f9f3354e914b26c456050f19c3e9badcc4422e5114a1a"),
        ],
    },

    // ======================== 02 岭南省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prblns02",
        node_name: "岭南省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626c6e7330320000000000000000000000000000000000000000"),
        admins: &[
            hex!("026c25206f34749215e5dd6ca6ab806ff53c4a047b68a5779ae7c22ed4befc73"),hex!("8834a8025fd436da0b7901653b4c6dd7ed05b14553a3c471b0991e1c840b0338"),hex!("5ec76404392102f3ac8f02efae9282160a478643b830d6c76c6edefaf6724227"),
            hex!("aede996c49ebd9b232ce1f2526c395c92532d5e0208d7f29ec5268e829cdaa4e"),hex!("4a51bb74e05f3f6d6cff6fcb627ebf435a6468de17496648d76706c6f588f26e"),hex!("863ff38fa1c71678b11665a5bcac920731c1408054ee362e331e23a9495d442f"),
            hex!("a6e3c01c8f3da28fb043faa9d1fcecc89aa8094cc95f267454fad3a415900a6f"),hex!("64c0493965122d338e1b4101996f64150915b27b06c304c307614a5357f72f57"),hex!("3e03007b51ce0a2e656417676162d0086738ba4a37708548e704c380c1bdfb19"),
        ],
    },

    // ======================== 03 广东省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbgds03",
        node_name: "广东省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726267647330330000000000000000000000000000000000000000"),
        admins: &[
            hex!("0489dc54c1f161b86bef1c9c5e5ef04ebd0e80b79a165178739f76a8aec19f71"),hex!("ae07a1c9f0f52a7316d5fd8e961ed0725aeb424d24db73a79b526d4174f76e61"),hex!("70d474f19e6dd32e28d009dfe481062e918b03b7b7f4e6ae40b95a1e8b361b5e"),
            hex!("fe7f55bc39453972d91fea7e0d04a053abef4f5f012b56f862d4fae410899848"),hex!("f0224a2586b9d75b26711a7e6593b6a1bc557f748e592d3e4f7ad15594bef366"),hex!("243f01b9ff354ee50a7c2e61e7c28169688b840f7becaf85867dc129f4ef8c2d"),
            hex!("92785fc18eb65a2d3bf99ac882cd3c68ba94426c8b634e774a1b9d60d1c8d769"),hex!("58b898b0823d46193cf006560efa9524ba55259e93b826d3460d2260cf53d360"),hex!("b89bfc4fa5836b6d91cb43b5837eb6dc57d4a7da078e46fa00a50e14a8282a6d"),
        ],
    },

    // ======================== 04 广西省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbgxs04",
        node_name: "广西省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726267787330340000000000000000000000000000000000000000"),
        admins: &[
            hex!("9427e1d51223861d4af89d3327844ff454c67a6dd7cf1a061efe930ebc894201"),hex!("ba79a4f7103ea953b143f9d78fcf31de39d234d034d619231104a0a8d680f408"),hex!("4477a14b6e3057e5347474c959b706afd7950b82e2797332badc181cd9593f3c"),
            hex!("3c6cbd92e34558b35e559da1a01b8549f1ae9c878348e33de9b78fd8d8703f19"),hex!("86172cb85369b7cde30da53aedf23570f55917a8e43eec50de38d387a4413709"),hex!("404a4e27edf2b55df63655f6ac704081e789a75e6dfc60d8f2a4ee1667ec7345"),
            hex!("ca8dcdddada12a10d7dc0b119dfad443eeec8f72fa722d68439383b35953415f"),hex!("b4c82456351205f4589018277a8ee436bb07112f865e6930fbc597fe85e7c969"),hex!("38f523f767dc344bfdc0c8907349fd2e3b56c240bc4376c065cda277a33ea672"),
        ],
    },

    // ======================== 05 福建省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbfjs05",
        node_name: "福建省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262666a7330350000000000000000000000000000000000000000"),
        admins: &[
            hex!("aafafefde99d84cdaa347e3525cad15b83661b21e919acb81f07bef285241760"),hex!("d829889b40819c533dd7470c46591ae30391f5e976061cdeef39b5b2fed9591b"),hex!("d0616e6da5b3e5d99ff7ebe4dae095a6b31182d4c5d93e60b0803b4a8d5eab37"),
            hex!("14a772f6e6ef1df35349357dd8a4b75034bdd2dfc12f0186f448aaaf4455ec02"),hex!("5ac97dacd36454b351b0712b0cc6f04b4c10e8aeb6639cee38f277a95a47096f"),hex!("78a8a8ff6677222fb6815291577fd9c919498f20896fc0520efe15050dda9a64"),
            hex!("2800e57f037b51b00dc5e7f5490fb370a805bfc822b12f60e5fd36558451fa18"),hex!("2a5140a7f4d2f6b6303a3b886288f546e7370fb894e96caa3f39e5cd50f11003"),hex!("f0df823bd57a2a2bf08460677d2988111cde7744f5939dfa5c688775b2283f58"),
        ],
    },

    // ======================== 06 海南省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhns06",
        node_name: "海南省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262686e7330360000000000000000000000000000000000000000"),
        admins: &[
            hex!("28e54d11584f1ca20f574a25de76ab19b077cf9ae1708fe52a00385b854ff72d"),hex!("2ae625ae1cc657c8639d5ff70578bae607c3cbb77561709dcee75dcc1e16ae43"),hex!("bc34797c613cdca8271ea8d179e8f7baf021f1f167ba1092c3c20f9126916a2f"),
            hex!("d8ded8b77e9ba0887fb0a3011c7e8217ba5caa330c93c364b02116f55da7bb37"),hex!("4436c7b7a014160676377b22b9fa9ccf7130e3152141ca09e48c517976bb1706"),hex!("1ae12ff247807eb8b2d3b4667601548ae4261cfe7aa048899308ac79f8456a27"),
            hex!("b0c93755f06e2db5e459d11de9ad6798a913df4e98afdd8a1ce2a56c65f48237"),hex!("5c31fc0b3fab3e946623b1fc7d69bac37bf35dbcedd425ecf829233a4eca3234"),hex!("3a330e3a98701d3c323866f65457054b67570e58afa43378625be2afa9a12708"),
        ],
    },

    // ======================== 07 云南省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbyns07",
        node_name: "云南省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262796e7330370000000000000000000000000000000000000000"),
        admins: &[
            hex!("ae54c9e6bd19eaa35d2d344255f64a4258fad0e342ccee088f3817b6a61b152a"),hex!("86189f4190f2070778837d9bad9bc68b6f6c3ae4c712c5555b25553fb3ad2160"),hex!("16b7f4e63a9cf16385e086ecb693d0498166b6df914098450f4f5458a47f543f"),
            hex!("52a4c26874c4ccbc604fafc13717b6944190be7771f77bf6b2c0689699e0eb68"),hex!("9c3703a3cb52199cd68d4bfde877de7f5d29b4929e2603f1e1bdad190b22a713"),hex!("16a8f8e3a8fc2f4758ed18733a7cb552d81b4e962e32632974a891bb1e2c6a35"),
            hex!("04c5b04b8d34524ea294ae18c4d75d27730de78d7b669fc0457e93054d59e561"),hex!("1037b46f56ff1f7e471bf2b43b83b74301f33139b0b396766cfe0e2d8ca6dd1b"),hex!("0c271a93d8ed8d06cb707b6478d16ce2fa9806a73683a8f26ac06b6565077537"),
        ],
    },

    // ======================== 08 贵州省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbgzs08",
        node_name: "贵州省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262677a7330380000000000000000000000000000000000000000"),
        admins: &[
            hex!("e0dcf22a3dd7e1f14aa455c0e274d4b4dcb5c049f36433a7f7f4135f04211d2f"),hex!("12f8a76239200bd531cf343957ad851e5135f3b5144c395342f03c5ee65a7272"),hex!("76f4fa8168efa5eb0cb752a62b2d348c3d8c8aa459c7a014078be89054c1fc62"),
            hex!("de0d6720ea51ea86e0d6efa3e0cfe13bc5f063bc2c8cc375f53fe3f5f2d3a42b"),hex!("da92e15c03aa76f39172dd852ec502dc013dd808e0506dc4d61465f3475b0110"),hex!("f43e1cfa5aa8ec4acadd347ad175c147dcfc54422c0c8282eff44ad62e7dcf5a"),
            hex!("367ec774daf55f99e34c21267392b944ad8f914a102181ac234dd4d74982e23e"),hex!("b419ac37a0f5e34484605dee70784a5a8d611054a6e77bc84972442793db8f5a"),hex!("144c2ddc1b38d105501da808dea9b212d312512683ffe485775dc867ef89316e"),
        ],
    },

    // ======================== 09 湖南省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhns09",
        node_name: "湖南省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262686e7330390000000000000000000000000000000000000000"),
        admins: &[
            hex!("1a4bf08983c7ca48d46ac578334d808c7ae02fa1bcff39c7fad236fa5b89e246"),hex!("764c622263b771743c4493f6bf282138b05f83ce22ed2bdce1295abc45130f1a"),hex!("9e1382c1b4e598b3c11af101087b48490107fdee8a60982e203cb45197507c64"),
            hex!("d4e24bb7ecd749c7a4d91d872a66883cd03be53a2e126572f5ab8642d9fe8471"),hex!("de5e61c37180c7185e3b5aebd6c23ec09f1f7fd58ef735bf44e665c205550b0b"),hex!("d84df227043cbb9c20b4c32889515a5b691161f1d674dd1bcceeaa9190438250"),
            hex!("d2776f7c9136cf9d9b4562b75a0f3bb8510c07be3bf60736506e8691f4745f63"),hex!("fc53ef26a1a9eeb77200d7662ea8855207e91bd94307f7438c478d735d186d53"),hex!("c03c2fd43e5fded10e23ca3ff8f89d2fc2cd7727ffd71702fe1361a3de90bc23"),
        ],
    },

    // ======================== 10 江西省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbjxs10",
        node_name: "江西省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626a787331300000000000000000000000000000000000000000"),
        admins: &[
            hex!("d0eb85c18ae4ba3a56618122ee359b2c18b3af18de0a8d994d01502cd8779176"),hex!("c6095937cbd97ddea9713f6e566837203e1af3df322d4c36031631dc3a76c412"),hex!("feb24a49cc19e93afc71c8f571f41ed4f50a7a4bf67aeb40e7b9aff6f7b42b0e"),
            hex!("045cf829b8d04ed2e30875edcad3f79bfc28712409d2e5ae22ecb9412039de75"),hex!("d2b613d5378c9baf80a7db7c5c1d2c1eaa93efed5e8afe6f4aecd2a1f827db4a"),hex!("1ccfe7cadeb274581a5c0b7e7074e3c847b1fd51f1cc09f7c1331971f1001c17"),
            hex!("70606dcfc50a907877a5efb5cf2142fd8e3677ec96d0b662a3663d5f22523701"),hex!("f806d5e7d50818d15874000200f645154bdd6393ac7718aff6c3fe137dc98809"),hex!("2c67ccb7d26a16998cca8872805c7748bb39690825c0e2481b0b2f82e6741948"),
        ],
    },

    // ======================== 11 浙江省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbzjs11",
        node_name: "浙江省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072627a6a7331310000000000000000000000000000000000000000"),
        admins: &[
            hex!("d45f674ef2a84f320674374cd71e40fe29f4532d4329588f59165011d6a6a673"),hex!("ec98eca9117b694254f7ce8d1c96228cee5b2ac1229a8c3da16982a4637ff86d"),hex!("dec896043b2cf543245066ab46433f95594b97ae42a35efbb07e2061115a4b20"),
            hex!("78e4fcc4f43ab62203800f04ce708a36e005b41e511f488848f3d087d2fc2649"),hex!("ae1760f488b82818d5b71dbc4e017e28bf05bf37257ca0ba7ad69e0cb9a54503"),hex!("483e77429838ae97f793c87e04817e212816deb0219a6ba7788691e35fd42b7e"),
            hex!("1ab693f09bb211d5d6c56451bed9f5e857c7feb5602e5c84fbb3b981554d1b50"),hex!("92c16ee0968dd2c61e94674fdfa6f1108feddd675bb1dac605befeaf044f1563"),hex!("986a428fa9fb51849ee350dad173f42c8ede6a9a30653d53e63f3870f04e9d0b"),
        ],
    },

    // ======================== 12 江苏省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbjss12",
        node_name: "江苏省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626a737331320000000000000000000000000000000000000000"),
        admins: &[
            hex!("00b31ce3c5a0c72d11d14e44e9fc3f24143fd4e7b50d9d666ca0744030538a11"),hex!("6ef2d3a56d37d85f07bd12ad1e039d1ebc6016118baed9d6adf055d22556003d"),hex!("a0c2f02007638ba832a18d3dad102c6fb47be43316491f8c87683a13bf96db18"),
            hex!("4a238e6088fffb866f53774a7f399bc8a52bb4cacdf84f635cc091a7ae1a1960"),hex!("d409e41d414040b979c034b3bf8f4fca0ba839d4e5f7530d5020aff7af553a33"),hex!("bc8f91bf16d953981fb30b695a98197209c1198b98f330c38433dfbbb2f70c5b"),
            hex!("1609d015067a3188f7e9584ebc3db6212d82e547a1f3408485596d8964a49451"),hex!("320af3ed8bd987824165cf675a12a8aa12544eafc693bfcd83520800baab9e18"),hex!("a8adf22c1547fd72c782320b677aab4990a58cf99257836330a8bfaa2c76b749"),
        ],
    },

    // ======================== 13 山东省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbsds13",
        node_name: "山东省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726273647331330000000000000000000000000000000000000000"),
        admins: &[
            hex!("5a5c268c38e99e8f20b9e8e3d0a02697a92406e601b8aeb0420a39925211d64f"),hex!("f612de6628031f3f285de2c1a3d54583b285057ec28eef35c1e719fa8748eb7b"),hex!("5ef1de0f6e4e18a6219c21ddbf65f73504a7ce40e15e0bbf3929480539d0fb12"),
            hex!("861bf1382a88743d6e61299f9b900286f05baf89a37c6ff522e05a80ed272a53"),hex!("922f33a54139b05fcddb676a27378b086fff85bf0d96a6d2d9dcea37611dfb63"),hex!("eef9c2f11aad66b7ebe86ef5441b748505a746c64e2c484b83845d3a5c348527"),
            hex!("860bcb0c12b2bef1388e826deeaad6e6fed7f62c2228e01d1b3701d67d6b400e"),hex!("868b7872374a214555f187d0497987f5f8e1045251892198cdea92c971386071"),hex!("802b7623102a77be6c066507e48d7a48e79b1dfa0d5e3762b2cfd614d0eeb807"),
        ],
    },

    // ======================== 14 山西省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbsxs14",
        node_name: "山西省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726273787331340000000000000000000000000000000000000000"),
        admins: &[
            hex!("cedaa06d70cd93f39ddb56be76f08d0fa7ee4f8bb74dcb1d65d0831ef1ba7367"),hex!("2cea02949c178fd8161759f40b3446f6a14f1f485098c822fb3556a410de1d6a"),hex!("fc3ab1d451fa03ba214d736cc6dc1544605899f84607dae17f1fc3a424e03c62"),
            hex!("20eb59ba06ad88a7c112f49d35e8292fcb8401f1ae52935f53ae5e2c11002d14"),hex!("e85a6533113458c7cc022509bbf202df292e9f00e8c513294bcd7e7b42c3c60f"),hex!("985d6124a97469f3f7727e4705b71ce9adf67e97a5a3e5ef8f42f9abeec05c0c"),
            hex!("b66e76c9b3bf4b247657b8bb186930d6fe8eea449284465ce9b961829838ca1f"),hex!("2201cff1dcbf2451fb72fc1526c158ecd6fc75526684228dcea64117fd301a25"),hex!("885c5388adc5eaf1601325e47e705dc7f5f9615df1054a074b1d072f724c5329"),
        ],
    },

    // ======================== 15 河南省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhns15",
        node_name: "河南省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262686e7331350000000000000000000000000000000000000000"),
        admins: &[
            hex!("16977016c793b7f8cb5375bfb343e06511e8b6bff0a3ded1763b03e65b49dc75"),hex!("2e01b472f46fb11f0adabe426e7a76305d2f1a040213a5b5ca966b89dcc58d7c"),hex!("48d4eaf64c72a1c0a79f309df3dc0827f24a29ec203d257048e047062807a844"),
            hex!("be16a87d333ef8b28c5ca8ab060f1c0b3144f61f4c327ca3196850f8a48bd505"),hex!("3c1bcb055d647ce9b86cff3b4b4f5102d937de4aa69e930338f3e29b29730e1c"),hex!("c6b9e8361550d17370710be369b445830e94408795fde24bcd768e546928e97f"),
            hex!("f06b7c19c2d9494c8e5ec20fea3024bc2c3b147ef35dd9e4e53222d4cd58e258"),hex!("a275c80a3e61dc6345902e52da5a623366c6825c3139fc88767698a08393f901"),hex!("10f39cdfcfcdb1ba80b09ab9b6e2b59c0982fabc3961bdff5201f8bf95cb9d7a"),
        ],
    },

    // ======================== 16 河北省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhbs16",
        node_name: "河北省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726268627331360000000000000000000000000000000000000000"),
        admins: &[
            hex!("bc50baf7fbc79b72df63804d5d48275528142136d1914a2334347875016fa551"),hex!("26ccde1f7b15cb0c45ce3a798aee6c68945e524fded101546d1b2e289260ca76"),hex!("c2cd07f51c525ac6ed14b6414d178cb8eab731f4b951f8499ecb480ffedd6e31"),
            hex!("5220eadb7fb7d8f93fdef65da75dac227207e88861c03d0aae706af128c5402d"),hex!("5a0abc0dbffae68046165d3c933c1a89c0b176eb40b1810708894cfa64ddd833"),hex!("caf58f9e16b6129cdd9d4fe6aed5fb5f5364bc9b7b6880185d4ac74e7bb0e256"),
            hex!("7ee9a12bdf4c4c343202bb2d131f9cb168bb46f802d1910960379c1a1320912b"),hex!("f0dee259e129953e151fd2c1fcd330a5d90f2fdc5ad7bff563d3e25562c7f238"),hex!("6029df40a2c27cbaa1621e86ce5c819593c6bf5bc1035c392ca6168e4d5d305c"),
        ],
    },

    // ======================== 17 湖北省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhbs17",
        node_name: "湖北省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726268627331370000000000000000000000000000000000000000"),
        admins: &[
            hex!("d8234d16d61cba73ebbfa12d4e53ddf1a4f5d24f106c286799d0e14801d0953b"),hex!("625284c24b30db1417fa4f253751f7e53c0b7f82bfbfbebdfbf2fc8ebc246e54"),hex!("f89647d183ef9fa59b51e5389b1037187d10d49d337569a27e31196f4fb43e78"),
            hex!("660fea734215081aa69b7405a78480f389bf74fb093194fe5369f1517a1f9e6a"),hex!("845c1a0fdb32ce0c19a343f6fe03b1164032d141431838feaafaa90061048a5f"),hex!("784a92ff76b2c8d58a5647f7613b5eec294ded88bedb23f9b925c8fe5047f515"),
            hex!("c29052fa231e6fa89a5d8b8a1622f504b979422bed293c30140310d8a5e7d84c"),hex!("2ea4e49b7e0bc331b57394e95a9074f3040821f03ae39ca23c946b9361ba8a71"),hex!("3c1eeca78af39920bace4596d5133528a85d7e1f8bf35ad39e49d94714b9e412"),
        ],
    },

    // ======================== 18 陕西省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbsxs18",
        node_name: "陕西省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726273787331380000000000000000000000000000000000000000"),
        admins: &[
            hex!("d230315cd4220886014478c89fe74112f144ef42c167a51ec1cff7d1a394df41"),hex!("fc28e2bd3c0f096fb04888a25eded5aa405b3bc010a75d6db79484c8c4e08a57"),hex!("188d874eb7953a2c2592b3847a63df8c04806430dd8d33f4ea83170d9bc6c876"),
            hex!("82661c2010f24c4d6889a7a463f790fc26f4cde3a7725f989900b0cb5bd3a978"),hex!("7c9d7fb48d2b7193a90aab0bdb124bd7163e3bfd36513a6eef8d55b784b1c92f"),hex!("c8f1164a96df87f6195243716412572d260a6720694f38bb5f6ce4fb7120e103"),
            hex!("2c0ffa383c48ca457a93881f7376f34ee4898fcfbd037d288df3abfaff606b3d"),hex!("56f2ac98940fd058eb0bccee7d1548ceaedbc82a0de92e446393a01f548f8e0d"),hex!("a45f6059cedd46b76cdc5b9f3ac46ccc6f3d1503359a8db604b442ed56dbaa3e"),
        ],
    },

    // ======================== 19 重庆省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbcqs19",
        node_name: "重庆省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726263717331390000000000000000000000000000000000000000"),
        admins: &[
            hex!("9e06d0846eebfe509661911c909abd83ae39180b7c6f97099b548d8352ed6428"),hex!("fec0660b9e067379bbc9196804cc517fc1b0ef4fb2b38b3e05e7b95d97e6d96b"),hex!("90c012f3b1be1015ad2dce39e86da83cff4270a0ded957be3e49394137a79818"),
            hex!("1a4b6b0eaee1e29acb459766de0e76dec40b80c6aad529bf7688c6b5b515d16f"),hex!("888ee9676cac1a791e6b353a5fb0bc224c871cd462cb1f03dd9eaf8162757c0c"),hex!("44e268b1abe87149495a22ea4ef8ce5fe8fc7b19a303c0c8cb5e7d185b135e08"),
            hex!("429b2a87bb9b3970370089093a774157d959bda3b9d355e363e9ce580ae0377a"),hex!("52b9b8aaf0ae0e2e410f578bf796be1de503ebbd4bcf4a44ed15fd95dcbb8036"),hex!("b2b8fdec4af97547cf70037644517497aacc45361c628276ef940acfadedba06"),
        ],
    },

    // ======================== 20 四川省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbscs20",
        node_name: "四川省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726273637332300000000000000000000000000000000000000000"),
        admins: &[
            hex!("66942d040ad78ebd3dd5823702e65165bfd07ca6b72f4d6b6487ebfe5f710834"),hex!("fec1a2a37c3709426ed5c6dda3ec021a491eb26b489d26e3cae2c7928e1ada08"),hex!("ec141e5d7509eac37b6a65fdce970e3bc555f6c3ed20661783d92f361986ed4c"),
            hex!("4c61314b93df6d3de10d116b175e6b88e7270c3f8bc8d4d668dfa1b3891fee38"),hex!("6ae71a2a2cb96275a79e8a0db3ac6c0e068d96a39ce69cdd1dff1f37540c850b"),hex!("4273a0f33bd151b43c3a07c9c21e97a81e1250613794b5d8ab468c40698dc860"),
            hex!("da5609cf631b20b08afe73d4b9bef70489dd21dee81426e5499006d34a78b239"),hex!("166edde25e50048f5c2b8315bca7cbd4000677c98c1598097c299c18f2018e03"),hex!("64f1e8533f0289a56438e8a9589b79d48edcd7f7378939b5498db8925a5a8552"),
        ],
    },

    // ======================== 21 甘肃省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbgss21",
        node_name: "甘肃省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726267737332310000000000000000000000000000000000000000"),
        admins: &[
            hex!("f6a25ebd4796d80cff6b511baa479d0415a940747edd09b604f000969a40c073"),hex!("96ac1e9089440a286d9a5779464d50cbce7bd9acf3f13971c34e2bb500bf8a77"),hex!("14745b4b10aeb10aa4a1d5d49d9940f3c73736179c1700b6320574fdaad3094f"),
            hex!("e248ff11a7143aaaa5b38fecc0d7cec1c926ae7c8d4cea7c93070386828ddd6b"),hex!("c8210270e23ba1062781d24c2c3408a56d86684297a8e6d5fb7851d16a65b264"),hex!("662ed2a1d32714580069ffccbf85f5c6e1bcb11c5931b4945691dd69e078f463"),
            hex!("6a83c3b5b49bcf996ac63826bc5a6eb6d40bc1aaee8e7c218eb1c3e576696b1c"),hex!("5e1c246b056c1062b28c550b3c5522b62fe4d7adc66c4834b4fbee6dd572d873"),hex!("86d0a0e0227792f44583395b3afc07243b7867cdf7aed5f6fa5938cd7c9e4249"),
        ],
    },    

    // ======================== 22 北平省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbbps22",
        node_name: "北平省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726262707332320000000000000000000000000000000000000000"),
        admins: &[
            hex!("5c02963fb3bd05b5219120d83d37f1fa579a385fecb85eb4ec2e2673ebbb2717"),hex!("426a2d8b60bea0d1d449f07944bb12b12f1d6462df665d370db2af69b81c3d4e"),hex!("d60f7c8aadb750f545334c524d38966f35d721636b4031c1788410c92b2d0904"),
            hex!("c29fb55b9b87156ebff233a30125050bbc3fed7c795b489c90ab56863542c166"),hex!("f4d6292e322ac011e8585cf7600ed6f5dfff24e69fb539f1f16c78188a264379"),hex!("6aca3b5f513ceb350452575bf535e75d6acf184e453513a65edd4c425a7c5453"),
            hex!("b27faa115f6b029fd1ab3d20059de2415fbe60146a85c0ac940090a7637af525"),hex!("fc81d4bc6fecc3cb8a5d3a37184176d728deb943a269610a1c17aa846f85e437"),hex!("a89ce9c37ad9e86cce3d8be399a14b93acfe574e776b292d6756904efc6a2e49"),
        ],
    },

    // ======================== 23 滨海省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbbhs23",
        node_name: "滨海省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726262687332330000000000000000000000000000000000000000"),
        admins: &[
            hex!("c088c0c872442f38d757e784c813dcec8553865ead6dd893ac6c017af5d69f55"),hex!("42114435469d7725d16e905e2a88c19da99bbb6ae524b1bce5b8ab5723307b02"),hex!("9e75e35544fb971fca7ee78a080cccdbd669f173a8f91bcc31d4303ad896dc0c"),
            hex!("dca005f0aaa33cff95f473d2109ed5bf2d133c5280a51bf1cc582cdc2d3dff35"),hex!("d07d6ffa23a36064a335f734eb60fcfb9b0a34cab795c2c91985086aa627da7e"),hex!("4e156fcd63d2d1365e201c559203739ef89580be01e57adc7b7dc0423dbd2f23"),
            hex!("262b175d94f407c4b7d55ee686660cb5dc241b4104f95c5999ae4fc7a75c0824"),hex!("f6f0c7794edd466007f5203e191048bac40b41cb84ae285832f109ca63eb827d"),hex!("667a17cff2c4f61a346f23bcf25b89f30198b972baf68327fa5c131a32139f3b"),
        ],
    },

    // ======================== 24 松江省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbsjs24",
        node_name: "松江省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262736a7332340000000000000000000000000000000000000000"),
        admins: &[
            hex!("827e73de31b2709b35089493589aacee2ae5b21783b42e5485d8f9b0e68fda61"),hex!("607171f3f3ea76ffc060c5168343e0b7e37eaa6a895aa5f767c9d7bbeb36b67a"),hex!("7cf58cab2eb3d22b06752610e229b6f7f295324e2f344bfec5b637a4e4749c4e"),
            hex!("9a37ad3f20e9598e2ccf5187069dba0a48916e684988332813ccac420afd8401"),hex!("2e845710e3839afcd92c911a8892afdf24803caf95484935a97c98cb0ea1bc4b"),hex!("86f9a87da746fc338a7dff2ecbb3f468076c51f1dc8e2d6f9e9d14719e760a0b"),
            hex!("50981da48a00d3d8bfd4ced7aa14a4ae3173529a5dc583cde2133d3753773656"),hex!("f2ae2874357ef87dcd293578734e04470a1b2ea9c99b0afb69a8d61859013f41"),hex!("4ad0d34a9e509e10bb963cfb61b925d3304d52cdb39425c9f84ae73a69a13359"),
        ],
    },

    // ======================== 25 龙江省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbljs25",
        node_name: "龙江省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626c6a7332350000000000000000000000000000000000000000"),
        admins: &[
            hex!("ce923cfccca40f7f50dbcc73f3993a47564ce7a2430d9f60bac6b9212db8082e"),hex!("c2aca903b71942762be6d8892ab61ad44389193ec442ee8a31ea7ced8c2e1b19"),hex!("c4405135f3207e48f73df3626170dbdd8a7c26bea487d954bc59d38e1bfe2411"),
            hex!("ca33870b20b1576efd042be443fd839f95e57c7168b7aeed1a8c457121967d28"),hex!("0ecec0bbc1460776b61772e59448cfec8e49d1e6e1690b37df634a8e92e3361d"),hex!("0274738d792b3b0d27b089732867eee6825c4840a738cb98827e81f677897f33"),
            hex!("2210636cd721b36f61e6ffe6595626d87c683d0f46644556c06b2ff5ad31c10c"),hex!("bc6aff4b1ff341288f950af231d2255d865c321d9be28fc3cc133051a02d7d43"),hex!("f40517df1e8dd81e696c077bc75dee2cc24b0b5c2e4469d8bb6302cf9a1a5262"),
        ],
    },

    // ======================== 26 吉林省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbjls26",
        node_name: "吉林省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626a6c7332360000000000000000000000000000000000000000"),
        admins: &[
            hex!("ec059effa082803a1cb54f4be99123b435514be482b2a45de16f2663d7bec07e"),hex!("e2720a778010c978a0dabcc8fd376aa1c2986cea86353695d89e452fe68e331b"),hex!("e07a5ae091692aee99991b9008c1c56381c251afa0efc37eb17d7b9275d00b15"),
            hex!("723c9fc9ad4ab90ffa9ee4cf5253d390d39536c2c05dd672b51574ecaa186c3a"),hex!("846fd5836f1f6fa13a771d2a9df45cfa210ec0effaf508e2d0dc3022ff1c4406"),hex!("74928fbc93cf85190f3ff261dfa1f8a5e47cbe1f2225417336210328f4d87742"),
            hex!("12558ca0dae6398cbee7997dcedce28ac6ae56193c81534d5da351f54b713868"),hex!("b2824e5b01c3a220919c644ed3c8fb4107f324b022e89c22143eaa2338acf467"),hex!("a2efbd8e24f0d7b4fa67c56a867c6f2d0e61af592e60beceb89838061341453d"),
        ],
    },

    // ======================== 27 辽宁省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prblns27",
        node_name: "辽宁省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626c6e7332370000000000000000000000000000000000000000"),
        admins: &[
            hex!("240519b39d7ac94ae048845ff707b81255bfb52383ba61c47d65c9adad1e783f"),hex!("0e95f2e4c8cf8e4f2b39d98149d2b8846edfb80d009f6e508e3b4dc457783a15"),hex!("7e0df17eafbb1fadafe0449f709f084a28fac02e306279512298fc13b918f924"),
            hex!("a85864830fa6048d2314494752472c5b94314bb76575af4c4f3a47e62e542326"),hex!("28bc4f9bebecc8fa5284d476e850ee515d91bf65dc238a4fd4ee61cfda2ff307"),hex!("e2c79da1ca71de5d4a5d105af88d8521e4ea0b3b7805b44167164b49a78b4d76"),
            hex!("1292bb5391dfa9c11c0fce23ba2a223f9ab346fe6b24e2b4de9dd80ac0c0fe65"),hex!("c26e3bb430202b0ae55d5ae987a03d8da785246d0010aa3ee17f973c1bdda84a"),hex!("2e5208b5ec652b2ebc566291686197d6d881b63de55fdd1997a3d6e5392d8934"),
        ],
    },

    // ======================== 28 宁夏省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbnxs28",
        node_name: "宁夏省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626e787332380000000000000000000000000000000000000000"),
        admins: &[
            hex!("9e6c016183e311d06f3af7f8c868d23a63ebb02aed985506ba9cbac71b88e636"),hex!("70d199e9871d49ca54f65ef84b22eddaeb46d65fa51acffee5bd8d4ec5f68c7d"),hex!("d8a96a54d932bc5e09f7bf38a88b71c95769dbf7af5ff33dd34d41b46d89e203"),
            hex!("542a2a6c6619905dae645155031798dead285d8ac2a3f8bc8bc8dd9b22361a0a"),hex!("c2ae8aa505437883c5735aa59b193651dec95f6d02a20584f8e41a39499d866f"),hex!("2e8789eec562809e80ab2d27c88cce06fc8ccfb282e2ca6e05687910546b0f2e"),
            hex!("e01c3d42d0a0c8c1be297c3a43ace06c8f7265f5a008b5ed7b57073d434bce5c"),hex!("aa319fb9890c5ca4f93b4fe04d16b3bafec2a7d0670d61c357de01aa38d80e24"),hex!("260996f5dedd4700d05f37910dcab1f22bfc0d605c89551f612f27eda6302f57"),
        ],
    },

    // ======================== 29 青海省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbqhs29",
        node_name: "青海省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726271687332390000000000000000000000000000000000000000"),
        admins: &[
            hex!("d0abf7d5bf48879b31cf8335bf4eaa35284185444c3c340902791c8860d4d703"),hex!("02186e9ca519afc3d461d7433bf47c932b4da73c262eb599b1aa507a6ce6f812"),hex!("825f083b9d940748da8bb79c00ebd971bed1a63fc5241f3d72a02726ba236529"),
            hex!("0cc7b2633d3eadebea0ba392104230960649d4d30e43049eabd155fd73f67605"),hex!("580f6cbb5a89e784b67573468244c172a80fb45acaec647fe6c5fa821e256454"),hex!("ac007a98f00ff09f1b6ffe26c92727288c0a882c6db1352061ff941742a4557d"),
            hex!("8699a89121d0a48effb06c98ec5ba101b00d6ba2b725049e411fd7ddacb4917b"),hex!("baa510a19474b3cfee2f652d06591ce4a301cb4031f96935a85bf7e39d4f942f"),hex!("6c5b61a1ed394fa19c6fff22d2244e135bf1c7a3f22e7ba20a66b675ed9eb50b"),
        ],
    },

    // ======================== 30 安徽省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbahs30",
        node_name: "安徽省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726261687333300000000000000000000000000000000000000000"),
        admins: &[
            hex!("6a23c0f923ff3bcb7b787eff0e09cdc5ec3c30d5c9fbaa244b30ff14ec88d00c"),hex!("10936e219740f088667645ce8df23fd47b1cb246b3e2f66a7d1111cca2b1820d"),hex!("2a89d69c10104b963a3b41e4b553f694f33a7604149bca455b64edc31b5d8d48"),
            hex!("e014f9711e2b50e2614e5f12e9c1df6c43462f17349e50482a31de74dd743442"),hex!("3a2029f48a4759b5ebc1190d03750cfeba5d456eb0d2e22c7b1f7e2029892d4d"),hex!("6c1e2ff3d9e4760404aa7f6439561a026e7ecd8691ded778c5d7452c833a0e6f"),
            hex!("acdc237f31c7f5bea6e1ba0ad0ef6825fb4f82a53320f9b40e54b52789b2263c"),hex!("f417d30ca2bbd7507cc6d4ecb58090fff8f812246cbab00af40afad61ed4c74f"),hex!("fa968f3b80bff7530b7e9166f6f15b0c1b61ce51862bebb1f531d44d3baa4a4d"),
        ],
    },

    // ======================== 31 台湾省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbtws31",
        node_name: "台湾省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726274777333310000000000000000000000000000000000000000"),
        admins: &[
            hex!("c2406f8a720ce27969456d67468debc78e2933f1fb8ebe75799582e6eec73839"),hex!("04908273d2916625087590caf764aae7eca0f8fa52a2f9b80959e7f308550430"),hex!("fe4fd3d43864c7c8b24249c85736bf42942ef3529e40c995dc47366af3c6352d"),
            hex!("14bef877de027a0731f83c45b540e867636568a9970f25c6084b2d7cff222a51"),hex!("82ac430e7fb79b0bc3578f2f7933dc9b7fead6133b19037f12deb0c70f80da56"),hex!("66930332da7d1571ef501128e9705f9b0b32bf0d928f8c9101426880b203492c"),
            hex!("98b034fa0021e3768fe7bf780fbff92c9cabed17a7f47eb24a0f11df9ea7c417"),hex!("20bfc9a0c0c8af372422ec49e3e1a4f8f593540d7b14d0540ddcb5cbcc2e8458"),hex!("bee18747fffd52f7a7ae8f6803a8eaecef7c2ebc39e41f8f134c032a2002e96e"),
        ],
    },

    // ======================== 32 西藏省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbxzs32",
        node_name: "西藏省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262787a7333320000000000000000000000000000000000000000"),
        admins: &[
            hex!("82df0fbf16857baffbed3ceb167c17ee87b999797bdd3cc725c55ddf7d3c040a"),hex!("70fe5e941a9171c1884a6094793b91276a1871737c1a182539c453daa743fa6b"),hex!("5ed91f2dffb02e1d8555552221895a0f2bc62e465c3c4e7fe2c39bad15515651"),
            hex!("42cdeb594be0bcec9769568a6ef78dfac46b3c5917f0b672689b61266f1ac057"),hex!("b2ed708f628783ce1e0cc3a9aa6cc1ac311f743d5aa1ceeb29060b6c6e568667"),hex!("90f2050b6194d58863593c45fca12235fa8afc3a72a2f74718b39690ceaea10d"),
            hex!("34de8ffb952a698e6a780006ff59af02df46c94ac0ccdea48702308af3e5e331"),hex!("02df24dc3fe938abeb515541d17ba6c195f9a1848e7f52d9f93f190949c60a36"),hex!("40205d44be3e23b4640466187c7ece048ec1d9e5a46543f8e837ff5305b7024a"),
        ],
    },

    // ======================== 33 新疆省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbxjs33",
        node_name: "新疆省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262786a7333330000000000000000000000000000000000000000"),
        admins: &[
            hex!("9af72d3d46fe219a8f48fb6f73fdaccc4057f32dffe110531e606487c5abf73e"),hex!("6c5c177a796e35fc3a18e74d196b21d41931027bae0b852db3d1844c672b4e62"),hex!("a20044c26eda427df6856636abcf530dd69dd52a2adf641ee121a64a3e6fd962"),
            hex!("1004c1236add6021c6daa0610ddcfef539beecb2dd498dbc5e1d42b76e710a23"),hex!("b28b41fb390552cd0c7c546cbe8190e6a13f0bde1449a4a7a4aec725c5524472"),hex!("b0787508008653bea39959178eeabbbc511ec02611be0f504a2b1786f093b465"),
            hex!("d46eaa309fe5284c16ac707733bb881a89713416f20bdef04e0efd463901a360"),hex!("0aaf632646c690f965c88d6c54d41f3b52e8915c8738e7070142733c3bda4843"),hex!("b2e4cf837363653e58df3554cb2718c6de361f4e4be89fdde1956038706db553"),
        ],
    },

    // ======================== 34 西康省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbxks34",
        node_name: "西康省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262786b7333340000000000000000000000000000000000000000"),
        admins: &[
            hex!("b0c59a13a468e7c5a5d4fd32a7ca4d34e4bd41517a0bcc8327e2e40c838c2c08"),hex!("2c35bd9c95e7ca5660ca44f7bfe6f1033e240860be666b84244ad0fbb2469945"),hex!("a82c184bd133150afdc1579c59c61d89d6300159caf65f1bce3a73f8cbc2dc3a"),
            hex!("bceb6eefbcd7170e2c07df83e9ceca10c76de3ae08577f8e3f362ef70d5f4133"),hex!("386071d0f932fa9bcb8ab1cc549f5c6fcfb76861850c780c6f2839b1b2891f46"),hex!("4c0caac03ec707615ef9223534885d16c17e7f455bc57a3a02679c19baadb05b"),
            hex!("982b283b08614ac24403f95eae373454e3f220ec9b163d633dcecc92db279972"),hex!("60cfe55ad58ff3de9b17b109611159f525f329df78011aec48fdbde652941f1c"),hex!("8c97d5cc67c4624bfa6e857c6f69e42a4bba6277e9837430c772e111fb20d160"),
        ],
    },

    // ======================== 35 阿里省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbals35",
        node_name: "阿里省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262616c7333350000000000000000000000000000000000000000"),
        admins: &[
            hex!("5ad6075c5d29eb2b8e91083c406fe6f7ccaa41053d30e2cb0961d85bb26eca48"),hex!("c0abc0e68d1086d3f4b9de703dd4484b4949ef81c7d6a33372fb0cc9b3be990b"),hex!("be251b50338eace5d5c5c40403e7902d505901468f893e659195fcc2c11f6b0b"),
            hex!("26c41b0a38b9ab81c8887afb895bebcee3cd7c571d931679915155c287aedc15"),hex!("5007bd9acb6717822c71939d8f781cc6b572f7445030a7e6ce703ed394e48b60"),hex!("1c04cca6d88a5a590e4e49ed2ed50f204256f2114050524b15aae4154c724254"),
            hex!("ac154d10b46fb07020e76371051ded3d617a8ef9927ef32a2fe8b4ba5245cc01"),hex!("f66e8124660382fb32f290ddea5ec82a351427aabf18e59fc1c1a6de1fb99754"),hex!("1ad56f41d627053c6b643e729bbd7025126537970df5a7a8dd6d76db05f07d50"),
        ],
    },

    // ======================== 36 葱岭省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbcls36",
        node_name: "葱岭省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262636c7333360000000000000000000000000000000000000000"),
        admins: &[
            hex!("e8167b6b1319e6b529958c802060970eb11cad3653fe21a3efbd8359fd4dfb5c"),hex!("4e33b4bbc3c37ae3a61b8e52d2e901ef15d104e1fc9e27b4c40acf11db39634d"),hex!("a0be6c58f47b96d89d005d42d2f2e3ab4b4f85eb1879ff22f847c5369816304f"),
            hex!("4cd5be2e88205c7a9f1145239de5eec4f5728e547ef36cd770b2453d0bb98324"),hex!("6e79cb82c87e32357183d6c176ec15d5e712c5e81575f3da0cff8fd09be6a25d"),hex!("7606fea554dd53dc61626041790f44b8fc33562ea01107c7d4937265f601a120"),
            hex!("f4ccdb23715c80cfed13183456e00b2e2d0bf62a5f17853967bd10e55b071437"),hex!("c24120f48ba3c9a24fe308624757030c49741bbbfe9ab918c69f93747fc2e354"),hex!("b445c6d549a51e08d5be26834928b0e77bd560c66ec94ed26b12796055911c1e"),
        ],
    },

    // ======================== 37 天山省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbtss37",
        node_name: "天山省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726274737333370000000000000000000000000000000000000000"),
        admins: &[
            hex!("d0a892173117fe6d54d93e474395063b29ed4b3d96c280c27c7669817b611948"),hex!("24a69bfd777beb23434a5dc979fe1fdce4de09f08fa4a88b263eeece26cb6b2c"),hex!("2a30b5fa75be94882127923977133b5530a103cecbc1ee5d0807e40f1ea1aa35"),
            hex!("023f1870b50950c2c16b2f11402b78e5c129b6c4560d1f8bba80abbcda97465f"),hex!("d8b5349b64476b3832435a5b75fb94d26541f367382868b04a9e08fb87965a3d"),hex!("78c4f038316af9fffcd6289a7c1d36f9ca065fcc1b2dc5d338f13b51adc33623"),
            hex!("f28aa8e99df7e7eadfef8658d509dabe2818ea829f1bcbf36b0bcd260ada4b65"),hex!("dece90b8b82b3c8dea84b28071914893a3c9304b992f3f379ab0eecd7eb08449"),hex!("fc384e2a7af5ef3a725e0c399bac03ecb9c6e09157477494edb3414b4ea0373a"),
        ],
    },

    // ======================== 38 河西省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhxs38",
        node_name: "河西省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726268787333380000000000000000000000000000000000000000"),
        admins: &[
            hex!("0afc29913ab6807bd7f56c017f2d50b45131b2b77e0323ea63b6ae6002327812"),hex!("3e5616265ad161e34ad89cf30fc8069e17ce015604687497162344fd7bce0030"),hex!("d0f1b61336b82066e88897b8a7697acb13b6d3597a877eaa5be83a987a02b96c"),
            hex!("04a5fd4a9227eb3631213622b41c15089211db051721a3494c1a0a89f9d7ee0f"),hex!("ca6e48124f214842fd86f7acae10441fe54f15ab291e9caf286bb4bc932e0328"),hex!("e2466f65f3ccd1345caea32ca2f511900a02c5d5149d70c32870d93c3d44c622"),
            hex!("a210b74e49cdcb02bc39f3d6764f0be76539a54cfd118df04588646dd8c0902a"),hex!("621e91b56e3512b94959bdd67bf9649900f2437bd1df4c5e57dcfb9836c35623"),hex!("4854b26775c14491b376a504bf78e9e898ad799f32a96b3e31f888bc24353f19"),
        ],
    },

    // ======================== 39 昆仑省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbkls39",
        node_name: "昆仑省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c7072626b6c7333390000000000000000000000000000000000000000"),
        admins: &[
            hex!("a6050f81f2e028db2037070544210b7dd2097b42da9453950c540734bc321528"),hex!("fc700d1ecd4360d9fb5b47d93a721df4053ccaf5c1370d2a02f47cb8e9466b08"),hex!("c85b5193653a90a7713d98dfe68453fc76b52cb00a419d0b7752e8c65e43c950"),
            hex!("50f27e4385d90cf0028c04ace69d30ea3f162f296a7d40109096fc94d59dcd24"),hex!("d652d1fb5716e2d73225593376d79acff7c8b20e7e5fb0b9839c7d3759bce809"),hex!("86d9cf94c03fbf68fe3b79ae1ca7365109a1277424d5011ca3ecc67dca72e85a"),
            hex!("444184c8aac4472e3bfe8960613828ee893ced35830c35e82ef4abc4633b1c37"),hex!("ccd960e374abe779b3b20e603ee00105d32996a8509e3513715a53278042c432"),hex!("06a040e65a75cda8c4d79c50135dc5614fc1deede28da778c213088e8142666b"),
        ],
    },

    // ======================== 40 河套省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhts40",
        node_name: "河套省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726268747334300000000000000000000000000000000000000000"),
        admins: &[
            hex!("22e781c336d56df96887073aeb1bae59da5d827c8ad3b68ff1bed9e33767775b"),hex!("b65dd0354fb19ea5666c4c2caada227b708907a033452fad41e12eb2e2377210"),hex!("0c6b9a4a19a32a9d778a5d9aa77b16383e5dd1d52aace6eacf3815d7e066c051"),
            hex!("4a3e42221d8cc56d7fe8fec2191115ca65e0e14986ef814bd17ff1474f157a4d"),hex!("ae8693594ffcf953dff40820864d415bbe59404f15d50d1545cd156a7d922a6f"),hex!("104a2f063dcdea8623565e71c54a14ee7730bed79e019c5602a883efe9936c26"),
            hex!("22d243ebd6b7e4fa0778a7e379a12f33b8d56d3e21d23e52eeefb278cde8870e"),hex!("ee5ee2c8214f45650c0ffc855638144a540528d335bbd73a3256ef7489861200"),hex!("062b9b0e4031ed25f7181bb3becd940ecb7a1047f016826a345a980098228c7f"),
        ],
    },

    // ======================== 41 热河省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbrhs41",
        node_name: "热河省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726272687334310000000000000000000000000000000000000000"),
        admins: &[
            hex!("a0260cf1c59232c08ccf080783363dd6f276183c58ad3e5bc29b494f823af449"),hex!("8aa0feb0fb2850e7fd26e6ba53ef828a9d2aa0e4077aecc08142d901862b5f31"),hex!("dc7de36d15c5ba3638a9831cdbb9552445993e420d11a9f1a43a073fd1216f47"),
            hex!("aa42dec9d96fa114e464c1aa1e83ad240cfce91fe71993cf2b4977228af2684f"),hex!("dc0d2dfd484a348e027ae162597347cd7aab7d135d2998b9a15b342f471ca711"),hex!("de387045bc281a103261a9cc26ec0f99837a955c108dd65a29c258f8fd757324"),
            hex!("0ae77465dd298d98bb5187755ed1d77f3e1df60dc767cb4c467e4f0541be0c11"),hex!("70595d3d39cc108da95ef02cf4092b920b71151cba1c8cc4a6d559223e227c18"),hex!("f0f2ac36f6de1420b12c08b9c3a4b36037d9214c94f6512987ec5877fd9ee57c"),
        ],
    },

    // ======================== 42 兴安省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbxas42",
        node_name: "兴安省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c70726278617334320000000000000000000000000000000000000000"),
        admins: &[
            hex!("785c4af6c7af289e9deaf018d0593f6d2393626f8b4a756e026e62209a57bf26"),hex!("f283c9b82d4661efbd32710abcf9406f017e3dee11825885bb7b92e1a319e52b"),hex!("9e161898b9367f6aad76f5fbb177a14f45c645f40d894ef585849222c335d650"),
            hex!("2665721bc683f52099abdedf222090a6738813671ce86797a813ece272d5e960"),hex!("1830bbbf745e44efb2d77f38777b3d73c1ada1116f0891d5e76921e1c6c37a5e"),hex!("508eed5a2beb5fa55d0b7e764ffc05a1e189c5c9812f6f8156c5042e129be752"),
            hex!("d4e07860ef7b39ffa02a49a6b1f1982290fd56d7b12d8e94205fb82bf56c7e20"),hex!("4697cc28663df2de06fad673fa636d0a60b1dc7a00b6ffdf44ce5787fd48ee12"),hex!("0c1c9de449f727865058320bd95a18cf8efdcd3aad590e19221b8d34f5fa4c11"),
        ],
    },

    // ======================== 43 合江省储行 ========================
    ShengBankNodeConst {
        pallet_id: "prbhjs43",
        node_name: "合江省公民储备银行权益节点",
        pallet_address: hex!("6d6f646c707262686a7334330000000000000000000000000000000000000000"),
        admins: &[
            hex!("5a6b9de943c7a5125eecc39a18e565c174583d8aabaec8e540171e25d0251578"),hex!("0c892cbc1e85b3ea66dc26006429b8b568a35b2d533347248f43570496830d79"),hex!("e8beab3accbdc7987a611da0afa5d486e19a6ff392cf2a0c38a436cc7f970040"),
            hex!("5e0c9e359a9a599679eae0eda3eed9096cc77b72fde7319377ad7fefb5d8e239"),hex!("c6d9bc956ee1281bf4d5ba20b794c5a180932acf600136f55db0c46d8e81f73a"),hex!("ce7967b62d43f4e07f198b78ec3c06978e99f9d3702f400841458be021301d44"),
            hex!("dc959308cbfb16b1473eab50401142cc8368b9802ef9756a76d0bfdb187d1e13"),hex!("b23bcea6cc05cb5d253886c27c301a4b38333094b5e10cb51aa7d26f5631943b"),hex!("2436e13ae238143d672692543cc3e9f5aacbe682f8fc5f22f96966bb22c28445"),
        ],
    },
];