//!  公民教育委员会机构常量=china_jy.rs

use hex_literal::hex;

/// 公民教育委员会机构常量结构。
pub struct ChinaJy {
    pub shenfen_id: &'static str,
    pub shenfen_name: &'static str,
    pub duoqian_address: [u8; 32],
    pub duoqian_admins: &'static [[u8; 32]],
}

/// 当前文件尚未补齐真实创世管理员公钥，先用零值占位接入模块树。
pub const EMPTY_DUOQIAN_ADMINS: &[[u8; 32]] = &[[0u8; 32]; 5];

pub const CHINA_JY: &[ChinaJy] = &[
    ChinaJy {
        shenfen_id: "GFR-BP001-JY0E-413041075-20260221",
        shenfen_name: "公民教育委员会",
        duoqian_address: hex!("51bbd8bd50c4b0bb091e2a4c979f592afc1a11e9a0d2344ff9890be5061b68ce"),
        duoqian_admins: EMPTY_DUOQIAN_ADMINS,
    },
];
