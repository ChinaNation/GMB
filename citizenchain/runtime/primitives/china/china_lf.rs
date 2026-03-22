//!  联邦立法院机构常量=china_lf.rs

use hex_literal::hex;

/// 单个立法院机构常量结构。
pub struct ChinaLf {
    pub shenfen_id: &'static str,
    pub shenfen_name: &'static str,
    pub duoqian_address: [u8; 32],
    pub duoqian_admins: &'static [[u8; 32]],
}

/// 当前文件尚未补齐真实创世管理员公钥，先用零值占位接入模块树。
pub const EMPTY_DUOQIAN_ADMINS: &[[u8; 32]] = &[[0u8; 32]; 5];

pub const CHINA_LF: &[ChinaLf] = &[
    ChinaLf {
        shenfen_id: "GFR-ZS001-LF04-322011991-20260222",
        shenfen_name: "国家立法院",
        duoqian_address: hex!("89240894859dd039d23e394047a2f6b58a0d18f9103719c961bee9b673b5e5cb"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-ZS001-LF0D-567216072-20260222",
        shenfen_name: "中枢省立法院",
        duoqian_address: hex!("96f098a34dbc5bf8eba42d3650368cc2cb394a25ad10e8c25ed77cada93e45e2"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-LN001-LF0S-687129656-20260222",
        shenfen_name: "岭南省立法院",
        duoqian_address: hex!("e6cf9ab11d326d3ac67363ed66fef489198f8ba92ddd82039df0fba5e7daf539"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-GD000-LF0C-052333305-20260222",
        shenfen_name: "广东省立法院",
        duoqian_address: hex!("80f0153934d5ab817e46d94619d8fd168adcda162ee1165701ec9df77e81a6df"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-GX000-LF0U-550329623-20260222",
        shenfen_name: "广西省立法院",
        duoqian_address: hex!("f5dcebbf3c4b0e61f074809ea9761a171b52a59952ac3e9e18fae73f6c5452fe"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-FJ000-LF0D-211971255-20260222",
        shenfen_name: "福建省立法院",
        duoqian_address: hex!("df6949bb3b27ba8f39513a60abe70afa127611c55ff99405dda381e1f079a15b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HN000-LF0M-994247755-20260222",
        shenfen_name: "海南省立法院",
        duoqian_address: hex!("599f74aa85d816015f640e175603cabd44a6847225220fd183c8f467b20e3fa4"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-YN000-LF0W-753818460-20260222",
        shenfen_name: "云南省立法院",
        duoqian_address: hex!("e42a86ee2dddc15ed0732884665b742db9d5e641082e03a567ff153f9a86fe8a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-GZ000-LF0E-032025014-20260222",
        shenfen_name: "贵州省立法院",
        duoqian_address: hex!("5ff2e1e9bb211712874c90a05bc8a6257dcd5af2dba8c089e686d3a677e196cb"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HU000-LF0R-750335780-20260222",
        shenfen_name: "湖南省立法院",
        duoqian_address: hex!("e2f36a5af5ea73c2d13ae96588822bca49b2bf5b5aa057b982ed2faf6130f767"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-JX000-LF0C-050464661-20260222",
        shenfen_name: "江西省立法院",
        duoqian_address: hex!("8229372c9c8e0f0839d46a84140981a0df1d6cc79ab6727654c143bb6ebaa24c"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-ZJ000-LF0M-045881965-20260222",
        shenfen_name: "浙江省立法院",
        duoqian_address: hex!("a6f12465fd22264ac672219e70e120013bd0122cde0c604b76cc6870f38e6d45"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-JS000-LF0P-990674786-20260222",
        shenfen_name: "江苏省立法院",
        duoqian_address: hex!("f5dbf679c26053c11201886b088694763947a6141bb0550199a60d9fcc726624"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-SD000-LF0E-425528544-20260222",
        shenfen_name: "山东省立法院",
        duoqian_address: hex!("60c2e8e48e43a01888f1775fa6a3b30043b5f269b65b4bd6cc639020ffc0ff4b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-SX000-LF0F-024836586-20260222",
        shenfen_name: "山西省立法院",
        duoqian_address: hex!("44ebb9d2bfb83960ca9d21ab1bd70afd8826b480e1324692152008ede1bf39f6"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HE000-LF0Y-331451307-20260222",
        shenfen_name: "河南省立法院",
        duoqian_address: hex!("075ef7938da91a7c792ca979571ecf3926b14dea29f12e8baadc9b8ce7d2b882"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HB000-LF08-541657945-20260222",
        shenfen_name: "河北省立法院",
        duoqian_address: hex!("cbe94d9b6d297d89407e6e6f79f05bd119eca26c1e348dff5ea11dd509b6597a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HI000-LF0M-301101620-20260222",
        shenfen_name: "湖北省立法院",
        duoqian_address: hex!("36e71628af995b540c82a8aa1438d66c03d32ef916036a92914daaf8f2ab6ca2"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-SI000-LF0B-236529547-20260222",
        shenfen_name: "陕西省立法院",
        duoqian_address: hex!("23de9f44147820f6fce84c1a8519c49b4d88f9aa7dc997169a3fb46830912d43"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-CQ001-LF0T-076401371-20260222",
        shenfen_name: "重庆省立法院",
        duoqian_address: hex!("5b90be3819c4006f4e358fa172fd997fe2bdaa1714447084161b897b8d8f77e3"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-SC000-LF08-652961696-20260222",
        shenfen_name: "四川省立法院",
        duoqian_address: hex!("281aff89be524dbbca015820d481ca218d641b3a7f6e9c10adbbc1c748cb548a"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-GS000-LF00-041478229-20260222",
        shenfen_name: "甘肃省立法院",
        duoqian_address: hex!("64c262972b5ab6ed310aa72ce0fe4de9265d412b7b00935f3bc1c7ba04f8bf3b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-BP001-LF0W-420000038-20260222",
        shenfen_name: "北平省立法院",
        duoqian_address: hex!("5399e0494aa091a1b3f9e6a113d974c09587e8798a53a38a3deb4d424e0f0c35"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HA000-LF03-155146715-20260222",
        shenfen_name: "海滨省立法院",
        duoqian_address: hex!("0b7e89ec65ad6ca9015520e225191f867b0f5833b7dbbcd6705a448c60a32438"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-SJ000-LF0N-112914008-20260222",
        shenfen_name: "松江省立法院",
        duoqian_address: hex!("f6bfba487da59b2174dc252a0a2f5910d464e447ec3d6c9bd019c6ef58ae9003"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-LJ000-LF0V-069130158-20260222",
        shenfen_name: "龙江省立法院",
        duoqian_address: hex!("666d30bdb700a9494ce4033ce2b37acce81c552693a39e12658dd27d39779766"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-JL000-LF06-244906085-20260222",
        shenfen_name: "吉林省立法院",
        duoqian_address: hex!("b08d1767186315c7f25d263de295b1b63ad06eea1c4cea7095cd6d8b85ee10fe"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-LI000-LF0Z-497176796-20260222",
        shenfen_name: "辽宁省立法院",
        duoqian_address: hex!("7798a475e5363d5617107144bd2f023a55ec61643ba42fb1fedb59681fb3bfaf"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-NX000-LF0Q-508119872-20260222",
        shenfen_name: "宁夏省立法院",
        duoqian_address: hex!("faf5791c330e9e800d38246c28115f5323a98906c94179ceaf83503f9bf94c07"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-QH000-LF0A-504085800-20260222",
        shenfen_name: "青海省立法院",
        duoqian_address: hex!("dea646822e85785c133f35716ed6c3d6d026da9fa4bbd1f06b7552802a9283fc"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-AH000-LF0M-648014035-20260222",
        shenfen_name: "安徽省立法院",
        duoqian_address: hex!("99d9d46efee5695bb930ef8c21aa23f1bcebbbd3a282c621ac47432d0f68f1bf"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-TW000-LF09-852624022-20260222",
        shenfen_name: "台湾省立法院",
        duoqian_address: hex!("68336bbf44d60e6bafbd0b49725ef043d024c1cdede568523e969aafb0eab151"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-XZ000-LF0Z-869677815-20260222",
        shenfen_name: "西藏省立法院",
        duoqian_address: hex!("7171ac27b52651ca2cc704f32e6dcb0f0a37c5cd21059b02f7330f7f1fb49f91"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-XJ000-LF0N-319077388-20260222",
        shenfen_name: "新疆省立法院",
        duoqian_address: hex!("0106255b8f972408f64838f7bf00cfe95e33688c8ded0d15306cd8eb43abf236"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-XK000-LF0A-700045037-20260222",
        shenfen_name: "西康省立法院",
        duoqian_address: hex!("e71a79a38c4494a59b1b96735802a570fb50f9898c6e2b6fe051d536614e69d5"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-AL000-LF02-381626250-20260222",
        shenfen_name: "阿里省立法院",
        duoqian_address: hex!("6d63617bcb51a16fa02bde8170cb9dbe8be9e11dea2ada377ee9b8d80794ac53"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-CL000-LF0M-941193753-20260222",
        shenfen_name: "葱岭省立法院",
        duoqian_address: hex!("22ffa7686f611c409afcb3ce1ef5f0cb73499d88ba63fc9fbf9982422fa71da9"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-TS000-LF0H-016224869-20260222",
        shenfen_name: "天山省立法院",
        duoqian_address: hex!("66d7fe372ac150d4293d8667e054d92cc6b80155fd178c46bf35b3dc75aaa199"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HX000-LF0Z-408465994-20260222",
        shenfen_name: "河西省立法院",
        duoqian_address: hex!("603e094df9834524c30ec5adfc7d96c8e15454e900146e2e1b84f90f849436d0"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-KL000-LF0B-670586781-20260222",
        shenfen_name: "昆仑省立法院",
        duoqian_address: hex!("c32ea7b121ae75b847eee9faec8519cb9be606e2720eceefce70c6fe45d50255"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HT000-LF0X-832682996-20260222",
        shenfen_name: "河套省立法院",
        duoqian_address: hex!("0529fce585e2da57c1828fbdd09545c012c7602ae314fdea8d55b34ad9a8d561"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-RH000-LF0B-911066915-20260222",
        shenfen_name: "热河省立法院",
        duoqian_address: hex!("4e6cf40ef7ad2f68fdbe4d77791f07ef42abb2025f38ae40c12d6263a0ac2c5b"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-XA000-LF03-382214972-20260222",
        shenfen_name: "兴安省立法院",
        duoqian_address: hex!("65b676a8d8f291ab18fad3f53d6444fc915e8c49e49afa02052061664e8472e1"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
    ChinaLf {
        shenfen_id: "GFR-HJ000-LF03-398476598-20260222",
        shenfen_name: "合江省立法院",
        duoqian_address: hex!("ea012bcc6eff48138290b421261462997657fae9559c95760c6425f8fdedaaba"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
];
