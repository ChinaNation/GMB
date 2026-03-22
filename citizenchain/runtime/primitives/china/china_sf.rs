//!  联邦司法院机构常量=china_sf.rs

use hex_literal::hex;

/// 单个司法院机构常量结构。
pub struct ChinaSf {
    pub shenfen_id: &'static str,
    pub shenfen_name: &'static str,
    pub duoqian_address: [u8; 32],
    pub duoqian_admins: &'static [[u8; 32]],
}

/// 当前文件尚未补齐真实创世管理员公钥，先用零值占位接入模块树。
pub const EMPTY_DUOQIAN_ADMINS: &[[u8; 32]] = &[[0u8; 32]; 5];

pub const CHINA_SF: &[ChinaSf] = &[
    ChinaSf {
        shenfen_id: "GFR-ZS001-SF09-134090812-20260222",
        shenfen_name: "国家司法院",
        duoqian_address: hex!("7fdb2741778695a330ba4e755872a89605e2827d3fa4b331bedb2e2e24ea4f0a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-ZS001-SF0D-573135799-20260222",
        shenfen_name: "中枢省司法院",
        duoqian_address: hex!("bf80606b982dace8aeacc04d9921e494f2c971c6ba0fbbad8b87a1d4495796a6"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-LN001-SF0S-386487097-20260222",
        shenfen_name: "岭南省司法院",
        duoqian_address: hex!("80377f769c0118f3d87e2113d31f2d5524e56e879d395419e885209fee7f71f8"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-GD000-SF0H-636757945-20260222",
        shenfen_name: "广东省司法院",
        duoqian_address: hex!("8cbc65e1f95d488a5a52f4368e41edfdcd76584e46b3df9414722b80fb53d7f5"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-GX000-SF0B-577422807-20260222",
        shenfen_name: "广西省司法院",
        duoqian_address: hex!("17cd69c84a78b22102f8d046fc5f547e98b1f0ee4d7491417e4e39263c83af66"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-FJ000-SF0Z-050954873-20260222",
        shenfen_name: "福建省司法院",
        duoqian_address: hex!("eccfa7572815c06969341dbdb7444e88c64e30633902061798bb6d4315eeddee"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HN000-SF0W-719849621-20260222",
        shenfen_name: "海南省司法院",
        duoqian_address: hex!("ff04bd49e347b0c918f6b9d1d0677b21a688c029e11b1f291952b0ee3155a5ea"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-YN000-SF0E-419030790-20260222",
        shenfen_name: "云南省司法院",
        duoqian_address: hex!("009b2861a3a7a63b7f635db9bd04175a120a762647954d4d5ed28ea046eb40e8"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-GZ000-SF06-627760254-20260222",
        shenfen_name: "贵州省司法院",
        duoqian_address: hex!("1d09b6dc2992968960964c34150fc5c09b735e8624312ca1943a1518b2dcd4a1"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HU000-SF0P-517016256-20260222",
        shenfen_name: "湖南省司法院",
        duoqian_address: hex!("21e47d9f9928838bdad02fe2e798e6bd1e5894453d5c0c2f3c8686ac8b178fcc"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-JX000-SF0X-185259287-20260222",
        shenfen_name: "江西省司法院",
        duoqian_address: hex!("9c6ea897be94ddb77de7ce7486140fced072ecf1e65bf59727b8963062605247"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-ZJ000-SF02-202699032-20260222",
        shenfen_name: "浙江省司法院",
        duoqian_address: hex!("e9bf08e3a3dd8abac11d467a3646335c34f81ee1b767cc68b33fc6db78dee990"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-JS000-SF0Y-843964423-20260222",
        shenfen_name: "江苏省司法院",
        duoqian_address: hex!("f2eae4be5c55cef3110a5f1220d0341cc993910b05064983e2b140c8b4a201cd"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-SD000-SF09-958422262-20260222",
        shenfen_name: "山东省司法院",
        duoqian_address: hex!("f669598a8c55d63c3ca2861d5bd8e4c93f262eb88f41a7a81f711351a32c29bc"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-SX000-SF0B-162856734-20260222",
        shenfen_name: "山西省司法院",
        duoqian_address: hex!("11bc85588071f5f2896ab216aa947a4e438db51a9ebaa930052793f5824759d8"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HE000-SF0R-281881294-20260222",
        shenfen_name: "河南省司法院",
        duoqian_address: hex!("566c1bc778ddbe8de349e33b464bb7c0ec9181cb6eff0550f06ef64efdc60c8a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HB000-SF0S-926728777-20260222",
        shenfen_name: "河北省司法院",
        duoqian_address: hex!("a145576c8a00c5afba242bee2b668fee4828343818e7ba304805c25fb4eb9f31"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HI000-SF0S-919487435-20260222",
        shenfen_name: "湖北省司法院",
        duoqian_address: hex!("60b0caf4c25ebdadfc5133c645d8cad0c58d77ecedd21587f90643b642f59e33"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-SI000-SF0Q-547837769-20260222",
        shenfen_name: "陕西省司法院",
        duoqian_address: hex!("7ee471850cf3aa008f7239de5b72ff5ebe4e5af36285d4719cc73a13a52b0efe"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-CQ001-SF05-032749576-20260222",
        shenfen_name: "重庆省司法院",
        duoqian_address: hex!("24548aa1c5278ab75c7d2fa5b1f792c5e8e8eb6dd4dfe928c1d66f58b0027a77"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-SC000-SF0B-751618450-20260222",
        shenfen_name: "四川省司法院",
        duoqian_address: hex!("d48e9884dfa828ac3a5aa2e5a98f09b506c85655d0938ee000647fb57943d438"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-GS000-SF0I-197562170-20260222",
        shenfen_name: "甘肃省司法院",
        duoqian_address: hex!("9429f02a8007990be0be15088a248f5b5325534e1aaf549a8b1939ed5be4d296"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-BP001-SF0E-834279244-20260222",
        shenfen_name: "北平省司法院",
        duoqian_address: hex!("7a84b42d710131e3d05b4121b641d1b49f53af91a4028d9565d460c05488d86d"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HA000-SF0Q-587131891-20260222",
        shenfen_name: "海滨省司法院",
        duoqian_address: hex!("a935855099b954b1c4328901cb810284db419c714b96676fab793e341a61bf60"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-SJ000-SF08-631857147-20260222",
        shenfen_name: "松江省司法院",
        duoqian_address: hex!("d78942be8bac0cf20bf19d691d00524908f357b1d8678ab9aa5444e66c13d5a2"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-LJ000-SF0B-016343648-20260222",
        shenfen_name: "龙江省司法院",
        duoqian_address: hex!("177df7c37f425defa7a420fdc4b40c4c645d10ff73993c70719444071328af24"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-JL000-SF0S-993565077-20260222",
        shenfen_name: "吉林省司法院",
        duoqian_address: hex!("0b4f10c15f3db1a1b7af8adc2060227a5c71b13c1a6dce908281972379e44be4"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-LI000-SF0J-743563427-20260222",
        shenfen_name: "辽宁省司法院",
        duoqian_address: hex!("4b1e0fdb47c61012aa6f629963e88c9b04d3d882eecbbad4174a6df2dcf9f9f4"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-NX000-SF0I-627630284-20260222",
        shenfen_name: "宁夏省司法院",
        duoqian_address: hex!("bce2b411c2d35c0fc218643945864d07686273be9f82b54bc70a4ab40a773dcc"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-QH000-SF0O-492162264-20260222",
        shenfen_name: "青海省司法院",
        duoqian_address: hex!("ff834ef7c34bc9f53ac5c4cb121c0d869fe3428cc5f9bba89180e9a07a37b81d"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-AH000-SF0F-587460113-20260222",
        shenfen_name: "安徽省司法院",
        duoqian_address: hex!("7e3b428de48d44cf24f3a64d78b04e74f5ce2c7c3a9b999fbde99e743ee45a2c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-TW000-SF0J-161879778-20260222",
        shenfen_name: "台湾省司法院",
        duoqian_address: hex!("21047224b25eeb74279361e3d0d27eee52a3a91ff92b2815e29ae6881bfabca2"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-XZ000-SF0R-140596407-20260222",
        shenfen_name: "西藏省司法院",
        duoqian_address: hex!("f4037b38a4a55e0fe1fb5feee947388e13f45e9a9213f47b4dface09f8e15cdd"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-XJ000-SF0C-957175775-20260222",
        shenfen_name: "新疆省司法院",
        duoqian_address: hex!("948e0cdc27e1afc0da7e231d83f82e099a0f830007d60c9ed89f058e3679dde9"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-XK000-SF0C-987642255-20260222",
        shenfen_name: "西康省司法院",
        duoqian_address: hex!("b71af5e93d1b9aec3a7769b04a1c5c428bd79b864e5227c5fbc62884b4783684"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-AL000-SF0O-493141277-20260222",
        shenfen_name: "阿里省司法院",
        duoqian_address: hex!("4050f8d19998bf4975414aaa130c73d9be1867f68e607561e04663918ed3c59d"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-CL000-SF0T-958194313-20260222",
        shenfen_name: "葱岭省司法院",
        duoqian_address: hex!("0dca170ab6ba1a01171f0719641c094c45311161b53bfd9dd2b6bf11e720bf73"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-TS000-SF0V-916660314-20260222",
        shenfen_name: "天山省司法院",
        duoqian_address: hex!("3d9feed0dcf0e8b29ddc7d8348beb81f9344f05119931277b0744052e4872755"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HX000-SF0P-155050943-20260222",
        shenfen_name: "河西省司法院",
        duoqian_address: hex!("27f29c66ac3e7f1fb6edce39efd407155bce7366151787c02b7d21db60c3b52b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-KL000-SF0L-676785487-20260222",
        shenfen_name: "昆仑省司法院",
        duoqian_address: hex!("01c7c7fafb35c465e0cdb96a3660d8156cb423c7e1c0d6ca3fc5325d7fb6de49"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HT000-SF0V-152659255-20260222",
        shenfen_name: "河套省司法院",
        duoqian_address: hex!("95a4dd703f295def99a06c5ff701c29e200e723dc3445ee2bafa5321455c425c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-RH000-SF0O-129133184-20260222",
        shenfen_name: "热河省司法院",
        duoqian_address: hex!("76d2220b2ae4cc184dae55a123288784e6be99e2b14c039949984c9c43083ade"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-XA000-SF06-234674241-20260222",
        shenfen_name: "兴安省司法院",
        duoqian_address: hex!("0045ba90a3c1f99d1b08ca1a9a37c2d55f28fa87e8a564de00f23f17dd9dafa5"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaSf {
        shenfen_id: "GFR-HJ000-SF0F-384975396-20260222",
        shenfen_name: "合江省司法院",
        duoqian_address: hex!("ba15ba114421d8ad310ab0dbb5ee310cdf02480f045a7170f1bb5bee1b61fb4f"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
];
