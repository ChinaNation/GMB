//! 中国公民链技术有限公司正式创世常量。
//!
//! 本文件是该私权创世机构身份、协议账户、三名创世管理员和固定岗位的唯一常量源。
//! 创世播种与原生节点守卫必须共同读取这里的值，禁止在各模块重复手写。

use hex_literal::hex;

use crate::institution_constraints::{
    ROLE_CODE_LEGAL_REPRESENTATIVE, ROLE_NAME_LEGAL_REPRESENTATIVE,
};

/// 创世产品经理固定岗位代码。
pub const ROLE_CODE_GENESIS_PRODUCT_MANAGER: &[u8] = b"GENESIS_PRODUCT_MANAGER";
/// 创世程序员固定岗位代码。
pub const ROLE_CODE_GENESIS_PROGRAMMER: &[u8] = b"GENESIS_PROGRAMMER";
/// 创世产品经理固定岗位名称。
pub const ROLE_NAME_GENESIS_PRODUCT_MANAGER: &[u8] = "创世产品经理".as_bytes();
/// 创世程序员固定岗位名称。
pub const ROLE_NAME_GENESIS_PROGRAMMER: &[u8] = "创世程序员".as_bytes();

/// 法定代表人对应的公民 CID；注册局后续直接从链上读取该公民资料。
pub const LEGAL_REPRESENTATIVE_CITIZEN_CID_NUMBER: &str = "GZ000-CTZN6-198805200-2026";
/// 法定代表人链上展示姓名。
pub const LEGAL_REPRESENTATIVE_NAME: &str = "程伟";

/// 中国公民链技术有限公司内置身份。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChinaCitizenChain {
    pub cid_full_name: &'static str,
    pub cid_short_name: &'static str,
    pub cid_full_name_en: &'static str,
    pub cid_short_name_en: &'static str,
    pub cid_number: &'static str,
    pub main_account: [u8; 32],
    pub fee_account: [u8; 32],
}

/// 创世管理员人员记录及其唯一固定岗位。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CitizenChainGenesisAdmin {
    pub admin_account: [u8; 32],
    pub family_name: &'static str,
    pub given_name: &'static str,
    pub role_code: &'static [u8],
    pub role_name: &'static [u8],
}

/// 三个岗位的冻结规格；岗位码、岗位名、启用状态和一岗一席均受节点守卫保护。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CitizenChainFixedRole {
    pub role_code: &'static [u8],
    pub role_name: &'static [u8],
    pub seats: u32,
}

/// 中国公民链技术有限公司正式创世身份。
pub const CITIZENCHAIN_TECHNOLOGY: ChinaCitizenChain = ChinaCitizenChain {
    cid_full_name: "中国公民链技术有限公司",
    cid_short_name: "公民链技术",
    cid_full_name_en: "China CitizenChain Technology Co., Ltd.",
    cid_short_name_en: "CitizenChain Technology",
    cid_number: "GZ018-SFGQ1-201206100-2026",
    main_account: hex!("7a20b8b7b1147abfdb24615222e3c9d77f1ff9a85d2a509fcf51dc89a64d1712"),
    fee_account: hex!("4bc5b8dd3770b1230c79fb8e048f27ae4f4ccf6d6890de0399123a617ccf305f"),
};

/// 三名创世管理员；账户是授权字段，姓、名仅用于人员展示。
pub const CITIZENCHAIN_GENESIS_ADMINS: &[CitizenChainGenesisAdmin] = &[
    CitizenChainGenesisAdmin {
        admin_account: hex!("d6d73cfd7d6b7c5692749b7c46fd3fe398f16f84283910dbf15f74472e1e3938"),
        family_name: "程",
        given_name: "伟",
        role_code: ROLE_CODE_LEGAL_REPRESENTATIVE,
        role_name: ROLE_NAME_LEGAL_REPRESENTATIVE,
    },
    CitizenChainGenesisAdmin {
        admin_account: hex!("700f70581bf67776df95240a5e24078a2966f0a0505f66e0c28978a9ccea3b49"),
        family_name: "管理",
        given_name: "员",
        role_code: ROLE_CODE_GENESIS_PRODUCT_MANAGER,
        role_name: ROLE_NAME_GENESIS_PRODUCT_MANAGER,
    },
    CitizenChainGenesisAdmin {
        admin_account: hex!("94bc684636aa0ca9b2696d6c22acb2b8b7d32b8136ee34fe120ed631f64f500c"),
        family_name: "管理",
        given_name: "员",
        role_code: ROLE_CODE_GENESIS_PROGRAMMER,
        role_name: ROLE_NAME_GENESIS_PROGRAMMER,
    },
];

/// 三个固定岗位各一席；任职账户可以依法原子更换，岗位骨架本身永久不变。
pub const CITIZENCHAIN_FIXED_ROLES: &[CitizenChainFixedRole] = &[
    CitizenChainFixedRole {
        role_code: ROLE_CODE_LEGAL_REPRESENTATIVE,
        role_name: ROLE_NAME_LEGAL_REPRESENTATIVE,
        seats: 1,
    },
    CitizenChainFixedRole {
        role_code: ROLE_CODE_GENESIS_PRODUCT_MANAGER,
        role_name: ROLE_NAME_GENESIS_PRODUCT_MANAGER,
        seats: 1,
    },
    CitizenChainFixedRole {
        role_code: ROLE_CODE_GENESIS_PROGRAMMER,
        role_name: ROLE_NAME_GENESIS_PROGRAMMER,
        seats: 1,
    },
];

/// 该机构内部治理固定采用三人严格过半。
pub const CITIZENCHAIN_GOVERNANCE_THRESHOLD: u32 = 2;

/// 精确判断中国公民链技术有限公司创世身份，禁止只按通用 `SFGQ` 机构码扩大保护范围。
pub fn is_citizenchain_technology_identity(code: [u8; 4], cid_number: &[u8]) -> bool {
    code == *b"SFGQ" && cid_number == CITIZENCHAIN_TECHNOLOGY.cid_number.as_bytes()
}

/// 查询该公司受保护固定岗位；普通私权机构和自治岗位返回 `None`。
pub fn fixed_role(role_code: &[u8]) -> Option<CitizenChainFixedRole> {
    CITIZENCHAIN_FIXED_ROLES
        .iter()
        .copied()
        .find(|role| role.role_code == role_code)
}
