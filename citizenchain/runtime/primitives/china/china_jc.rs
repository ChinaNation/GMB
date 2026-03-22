//!  联邦监察院机构常量=china_jc.rs

use hex_literal::hex;

/// 单个监察院机构常量结构。
pub struct ChinaJc {
    pub shenfen_id: &'static str,
    pub shenfen_name: &'static str,
    pub duoqian_address: [u8; 32],
    pub duoqian_admins: &'static [[u8; 32]],
}

/// 当前文件尚未补齐真实创世管理员公钥，先用零值占位接入模块树。
pub const EMPTY_DUOQIAN_ADMINS: &[[u8; 32]] = &[[0u8; 32]; 5];

pub const CHINA_JC: &[ChinaJc] = &[
    ChinaJc {
        shenfen_id: "GFR-ZS001-JC06-904016805-20260222",
        shenfen_name: "国家监察院",
        duoqian_address: hex!("6eef79aecbc812a8879a86443515fe4092f691137a5bf87e1ded96b240f940d8"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-ZS001-JC0R-307171518-20260222",
        shenfen_name: "联邦廉政署",
        duoqian_address: hex!("6935f511e92b01b5bd30bb405c26f7bbc649b619e166ddfabd0c60bc6a849544"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-ZS001-JC0H-113480944-20260222",
        shenfen_name: "联邦审计署",
        duoqian_address: hex!("3258ef54f26c9924c8d9e5f5d070ff08b83ff58d41d28ec374cc9ab3bbb08ea1"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-ZS001-JC0D-710326231-20260222",
        shenfen_name: "联邦调查署",
        duoqian_address: hex!("94613d33667c12d65b059b9d82185e0b5cc765e5bfaac824a83fbf0cd25a00c6"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-ZS001-JC0N-516635657-20260222",
        shenfen_name: "中枢省监察院",
        duoqian_address: hex!("be07c39d3e4022052c19340450bbce9d9858544b9c551c0a2ab422ea374d515c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-LN001-JC0Q-582006790-20260222",
        shenfen_name: "岭南省监察院",
        duoqian_address: hex!("08bacb2cbd582c81534a7aa22e3ab78801e0b20ee4dd590d4b54a64c7c2265ce"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-GD000-JC0I-657317663-20260222",
        shenfen_name: "广东省监察院",
        duoqian_address: hex!("dd9acba82f86e95484d457ce66fde3bb5cd3b6d2919d70c64cd1e68352fc5b4c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-GX000-JC0V-676330909-20260222",
        shenfen_name: "广西省监察院",
        duoqian_address: hex!("723106acde77ae8f4a0d0bf8cf3ed041441898ec44e40dd5fe4018c7866d4510"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-FJ000-JC0J-039938480-20260222",
        shenfen_name: "福建省监察院",
        duoqian_address: hex!("e6afd147a03a055b9d260e4985c92b64a7e1c96d7310dda9aae12e51241f96d8"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HN000-JC0W-469213448-20260222",
        shenfen_name: "海南省监察院",
        duoqian_address: hex!("5c2585c5b398dac7d41b1b5191993d9bcac6f71b90a14b24848f5265e6dfe4ce"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-YN000-JC0N-497134869-20260222",
        shenfen_name: "云南省监察院",
        duoqian_address: hex!("ac790f9c11ebfe92a73782c0a212b7fb4577a0d47676240619b28bbc2377077c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-GZ000-JC0Z-706225319-20260222",
        shenfen_name: "贵州省监察院",
        duoqian_address: hex!("166ddccf9208155a83242f2244804729b8eac69cff64340114b37d0af2ec782b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HU000-JC0H-687454356-20260222",
        shenfen_name: "湖南省监察院",
        duoqian_address: hex!("3a383d73c3b0a592cac1d098b76f7deb2d1640427cea1be373c59474769b3e2d"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-JX000-JC0D-097600650-20260222",
        shenfen_name: "江西省监察院",
        duoqian_address: hex!("181bc995ffdd4582c29ad96cf58a6a60c20ca02e003e4ae767a04446281d51ad"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-ZJ000-JC0Y-532437557-20260222",
        shenfen_name: "浙江省监察院",
        duoqian_address: hex!("4701ea72f5d037556d5e8ec608f85ea443853704e27a27215ab8bcfd064b12d0"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-JS000-JC0E-299917474-20260222",
        shenfen_name: "江苏省监察院",
        duoqian_address: hex!("f1cd05c0483addb433b4cead7990c3b2a3473cec20f6ac500376400db50061af"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-SD000-JC01-059502842-20260222",
        shenfen_name: "山东省监察院",
        duoqian_address: hex!("80c2461203d3d76fe03d652f0c10c20d0d7f14df835d3db07ae2c1f464cbfffa"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-SX000-JC0N-188614480-20260222",
        shenfen_name: "山西省监察院",
        duoqian_address: hex!("443d954d020013e841de8306ed25cb3a153a36adc6e668ce510f3e34cd34722d"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HE000-JC0P-903663767-20260222",
        shenfen_name: "河南省监察院",
        duoqian_address: hex!("76cc68fbf6208cb3d4be38905913a68c53acfd091c741bf62521ed1b549c3737"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HB000-JC05-207494242-20260222",
        shenfen_name: "河北省监察院",
        duoqian_address: hex!("9c1f75fb8d7d4b9820383f3687271922800ea5567cd5bb313d89eb7392cea9b8"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HI000-JC09-899576935-20260222",
        shenfen_name: "湖北省监察院",
        duoqian_address: hex!("b82b16d49f04fc7458545ec6dca19a9173f7f1de3021f8cdf2dccc26650644db"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-SI000-JC06-712712961-20260222",
        shenfen_name: "陕西省监察院",
        duoqian_address: hex!("58a85d7456f2c8c26d0b90371d96e086002e53d0d4ae464b242982ce255cd81f"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-CQ001-JC0B-174457894-20260222",
        shenfen_name: "重庆省监察院",
        duoqian_address: hex!("49d12b1ed58ee0f6e205ef8bab27e266e313df69440f2650c0c81d6f1875b730"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-SC000-JC00-989142678-20260222",
        shenfen_name: "四川省监察院",
        duoqian_address: hex!("9027410430dd13470c7d83af874fbf6781ed7f0116cfd294eb8ee725618cd09b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-GS000-JC0Q-004906235-20260222",
        shenfen_name: "甘肃省监察院",
        duoqian_address: hex!("ab818aab18344a1143082ba582e67f4f26b0795f129655ec205340adc0dfcd5e"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-BP001-JC0Z-125929902-20260222",
        shenfen_name: "北平省监察院",
        duoqian_address: hex!("8bc2453622439f9d4f674522b96376563f51cd75153a1cefa5aa5bc2a1e2d6dd"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HA000-JC0K-960336031-20260222",
        shenfen_name: "海滨省监察院",
        duoqian_address: hex!("1ab95db13ea7df0ffdfbb9f064c188d9c1102ef9667d4058504d522f398cf737"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-SJ000-JC0Q-976572936-20260222",
        shenfen_name: "松江省监察院",
        duoqian_address: hex!("0e64138ac2961f8ad47aff071f136510d78a2baaa25cb106160d1dc24a33d285"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-LJ000-JC0X-110963692-20260222",
        shenfen_name: "龙江省监察院",
        duoqian_address: hex!("7d5a677765ad11b8e6477963382ef20fa3742d61df0f8d12cb38f2cb60a0641a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-JL000-JC0O-725401946-20260222",
        shenfen_name: "吉林省监察院",
        duoqian_address: hex!("a2ccaf22be0eb2d1bc2ee5e68b6e7e74168a171e9a238a0ec9d775426e1663c3"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-LI000-JC08-177199877-20260222",
        shenfen_name: "辽宁省监察院",
        duoqian_address: hex!("8d2d4d6e839eeb20cc95760c2e395eba8798f648e3fe69cc322b9d9bced0c24e"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-NX000-JC02-339994359-20260222",
        shenfen_name: "宁夏省监察院",
        duoqian_address: hex!("90df3aa4623014e71afa862799e8d23689c9015a5eb67970cc82429307aa9482"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-QH000-JC04-500038410-20260222",
        shenfen_name: "青海省监察院",
        duoqian_address: hex!("47dcab547274fa88311bad4e620ee9c0f2bf222b150b24a8990a12239dedbcc0"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-AH000-JC0A-609660358-20260222",
        shenfen_name: "安徽省监察院",
        duoqian_address: hex!("2eb05292ff7435685d90bb657056fd9a611552b4858f1ac2445ea00badedd6cd"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-TW000-JC01-092384858-20260222",
        shenfen_name: "台湾省监察院",
        duoqian_address: hex!("49eed6e90d9e42d4323ac74f3bd30090a2a66f7762da601afd5bc84783b7dcd0"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-XZ000-JC0M-218249895-20260222",
        shenfen_name: "西藏省监察院",
        duoqian_address: hex!("bceb0b181b4f0ee6d11578d42a0ab3be751f6e4827be6e21b561b5f9582206cf"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-XJ000-JC0H-406882249-20260222",
        shenfen_name: "新疆省监察院",
        duoqian_address: hex!("12b0a180ce9a0e3035d1f73f788d6815457708c02fbd32570dc66cd0f5057f28"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-XK000-JC0A-777213166-20260222",
        shenfen_name: "西康省监察院",
        duoqian_address: hex!("df37b8b340a78b3eead06fbcc669482eefea422ce05740d7621dc78d15129c38"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-AL000-JC01-966364359-20260222",
        shenfen_name: "阿里省监察院",
        duoqian_address: hex!("d5edd641f6204d470c9981d8ac139d7af17fdca54490ca4368d5c30f40ddd39d"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-CL000-JC0E-115150334-20260222",
        shenfen_name: "葱岭省监察院",
        duoqian_address: hex!("4a5abfef1aaeb7a85c20847b4f408bb580477620da21df57ee05f360dc9728ee"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-TS000-JC0Z-122030512-20260222",
        shenfen_name: "天山省监察院",
        duoqian_address: hex!("234254433cfa1588aee29cf7d1c90d53758d1c4cd0551b1c7a363bae5edad73b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HX000-JC0O-965675320-20260222",
        shenfen_name: "河西省监察院",
        duoqian_address: hex!("6901be577c98a8231c253ea7076bc48d742394c149c6b1fd87a9f82330871ca0"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-KL000-JC04-785662559-20260222",
        shenfen_name: "昆仑省监察院",
        duoqian_address: hex!("f04b02faf717608a2951301e8dc21e1ef6419bba9976d6a58528c2e515eaed8a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HT000-JC0D-044789622-20260222",
        shenfen_name: "河套省监察院",
        duoqian_address: hex!("a9c98abd08c27157f25021281b652e0562a4a9c37a13d57b2c9424ca184b1198"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-RH000-JC0J-615523474-20260222",
        shenfen_name: "热河省监察院",
        duoqian_address: hex!("30be48fdb2210cbe932feae4e301cde4e301c520cc938213e6bc669da69285e7"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-XA000-JC05-275211978-20260222",
        shenfen_name: "兴安省监察院",
        duoqian_address: hex!("52e2437f31762fe80b2e02fbb3702e885f04cae0da1f37d5a8cde6e73c3b147c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaJc {
        shenfen_id: "GFR-HJ000-JC03-294493127-20260222",
        shenfen_name: "合江省监察院",
        duoqian_address: hex!("8c21466f3949c278d0299cb9a97dd73b9fb5ee5016ae4bbcfddd6371458d8ba4"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
];
