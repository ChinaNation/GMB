//!  公民教育委员会机构常量=china_jy.rs

use hex_literal::hex;

/// 公民教育委员会机构常量结构。
pub struct ChinaJy {
    pub cid_full_name: &'static str,
    pub cid_short_name: &'static str,
    pub cid_full_name_en: &'static str,
    pub cid_short_name_en: &'static str,
    pub cid_number: &'static str,
    pub main_account: [u8; 32],
    pub fee_account: [u8; 32],
    pub admins: &'static [[u8; 32]],
}

pub const CHINA_JY: &[ChinaJy] = &[ChinaJy {
    cid_full_name: "中华民族联邦共和国国家公民教育委员会",
    cid_short_name: "国家教委会",
    cid_full_name_en:
        "National Citizen Education Committee of the Federal Republic of the China Nation",
    cid_short_name_en: "National Education Committee",
    cid_number: "BP001-NED0H-689724263-2026",
    main_account: hex!("3161c992b304566b3f912a0a7d298a84447982ab96ee6fc06f85ae90204d4e84"),
    fee_account: hex!("506ac991dcbcaa7e09bece1696b425e0c4e73e0e0610b270a4343f8a99dd1bc1"),
    admins: &[
        hex!("54e0451ef8d23e2f79d28756567dcdbc3fd475e7fc090b8c655ad243655f8231"),
        hex!("c48d9b25e6bf3fee1c8002bb7e91112a54a44fb99317483bd5dc73416eb48904"),
        hex!("e0751a2fd67acb376168f7dc385c3db123efe7dba845e9d6792229ddec030032"),
        hex!("7a00b7bd1d1f9b0523e5faa20bfb94c59767eea947761d0a9368d19cdd7a177f"),
        hex!("84e41e3ed044442ced3c0418167c80a30ad70593ffad4a7c1cc2c83c169c5163"),
    ],
}];
