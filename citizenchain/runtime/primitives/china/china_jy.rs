//!  公民教育委员会机构常量=china_jy.rs

use hex_literal::hex;

/// 公民教育委员会机构常量结构。
pub struct ChinaJy {
    pub shenfen_id: &'static str,
    pub shenfen_name: &'static str,
    pub main_address: [u8; 32],
    pub duoqian_admins: &'static [[u8; 32]],
}

pub const EMPTY_DUOQIAN_ADMINS: &[[u8; 32]] = &[[0u8; 32]; 5];

pub const CHINA_JY: &[ChinaJy] = &[ChinaJy {
    shenfen_id: "GFR-BP001-JY0E-413041075-20260221",
    shenfen_name: "公民教育委员会",
    main_address: hex!("bc2dbf091de15eb2fb9e6e1740d2bd3783848bb9fb5c794a8eb09548492f32bd"),
    duoqian_admins: &[
        hex!("54e0451ef8d23e2f79d28756567dcdbc3fd475e7fc090b8c655ad243655f8231"),
        hex!("c48d9b25e6bf3fee1c8002bb7e91112a54a44fb99317483bd5dc73416eb48904"),
        hex!("e0751a2fd67acb376168f7dc385c3db123efe7dba845e9d6792229ddec030032"),
        hex!("7a00b7bd1d1f9b0523e5faa20bfb94c59767eea947761d0a9368d19cdd7a177f"),
        hex!("84e41e3ed044442ced3c0418167c80a30ad70593ffad4a7c1cc2c83c169c5163"),
        hex!("5e3a5b78b32283a6049cf03f67c26321853742b1600f06453331d6d4c3738705"),
        hex!("564d5f9f00f8575246c06477d73d92afb3c8bce348bd38c27eef1a357aa97d5c"),
        hex!("3c0542b668553d0f574b17acca2bbe1db516860bce4973c2faa05a73d80d5a12"),
        hex!("102525e5bb8054175b1609d528c09b2289c44807896aaa56fdfda66487c7f247"),
    ],
}];
