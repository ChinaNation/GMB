//!  公民教育委员会机构常量=china_jy.rs

use hex_literal::hex;

/// 公民教育委员会机构常量结构。
pub struct ChinaJy {
    pub sfid_name: &'static str,
    pub sfid_number: &'static str,
    pub main_address: [u8; 32],
    pub fee_address: [u8; 32],
    pub duoqian_admins: &'static [[u8; 32]],
}

pub const EMPTY_DUOQIAN_ADMINS: &[[u8; 32]] = &[[0u8; 32]; 5];

pub const CHINA_JY: &[ChinaJy] = &[ChinaJy {
    sfid_name: "中华民族联邦共和国公民教育委员会",
    sfid_number: "BP001-GJY0Y-689724263-2026",
    main_address: hex!("a333dd55f9e2d7730249a42753f8ddae0486edf851f24a8e84e4afac34069ecd"),
    fee_address: hex!("670917e9e43ab5195745e334a063b0a03fdf275ee51777292627dfd92a09d377"),
    duoqian_admins: &[
        hex!("54e0451ef8d23e2f79d28756567dcdbc3fd475e7fc090b8c655ad243655f8231"),
        hex!("c48d9b25e6bf3fee1c8002bb7e91112a54a44fb99317483bd5dc73416eb48904"),
        hex!("e0751a2fd67acb376168f7dc385c3db123efe7dba845e9d6792229ddec030032"),
        hex!("7a00b7bd1d1f9b0523e5faa20bfb94c59767eea947761d0a9368d19cdd7a177f"),
        hex!("84e41e3ed044442ced3c0418167c80a30ad70593ffad4a7c1cc2c83c169c5163"),
    ],
}];
