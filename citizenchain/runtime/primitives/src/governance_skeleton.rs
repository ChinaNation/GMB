//! 固定治理骨架冻结规格(档 A 单源)。
//!
//! `admins-change`(`public-admins::AdminAccounts` + `FederalRegistryProvinceGroups`)是全部
//! 机构管理员角色的唯一真源,但节点层零守卫,一次 setCode/恶意 runtime 可任意改写。本模块把
//! **永不合法变更的结构骨架**收敛为编译常量单源,供三端共读、逐字节防漂移:
//!   1. 创世播种(`runtime/genesis`):写入固定机构管理员集与护宪席位;
//!   2. runtime 校验(`public-admins`):NJD 管理员集变更强制护宪恰 7 席(I6);
//!   3. 节点守卫(`node/src/core/governance_skeleton.rs`):逐块背书 I1..I7,setCode 改不动。
//!
//! **只冻结构,不冻成员**:固定机构的存在性/机构码/类型/名额/护宪席位数是永不合法变更的
//! 结构量;而"座位上坐的是谁"由普选/互选/阈值票合法轮换,不在本规格内(等长换座即过)。
//! **不含阈值**:固定治理阈值是 `fixed_governance_pass_threshold` 计票逻辑、不落 state,守卫
//! 锚不到,故不在本规格。成员劫持(保持席位数、整体换攻击者密钥)属档 B(创世根验签链),不在此。

extern crate alloc;

use alloc::vec::Vec;

use crate::cid::china::china_cb::CHINA_CB;
use crate::cid::china::china_ch::CHINA_CH;
use crate::cid::china::china_sf::CHINA_SF;
use crate::cid::code::{
    institution_code_from_cid_number, InstitutionCode, ProvinceCode, NJD, NRC, PRB, PRC,
    PROVINCE_CODE_INFOS,
};
use crate::count_const::{
    FRG_PROVINCE_GROUP_ADMIN_COUNT, NJD_ADMIN_COUNT, NRC_ADMIN_COUNT, PRB_ADMIN_COUNT,
    PRC_ADMIN_COUNT,
};

/// 护宪大法官职务字面量单源。`admin-primitives::ADMIN_ROLE_CONSTITUTION_GUARD` re-export 本常量,
/// 创世 role-by-index 与节点守卫 I6 逐字节共用,禁止各处手写字符串。
pub const ROLE_CONSTITUTION_GUARD: &[u8] = "护宪大法官".as_bytes();

/// NJD 护宪大法官法庭固定席位数 = 公民宪法第 21 条 4/7 终审的「7」。
///
/// 创世 `national_judicial_yuan_admin_role` 的 role-by-index 0..=6 落 7 名护宪,本常量即其规格单源;
/// 节点骨架守卫 I6 逐块断言护宪计数恒等于本值,补上宪法守卫 `guard_review_passed(approve>=4)`
/// 里从未被锚定的「7」(见 ADR-027 §6.3)。
pub const NJD_CONSTITUTION_GUARD_SEATS: u32 = 7;

/// `AdminAccountKind::PublicInstitution` 的 SCALE 判别值(声明序第 0 位)。
/// 由 `admin-primitives` 测试 `scale_discriminants_match_governance_skeleton` 交叉钉死。
pub const KIND_PUBLIC_INSTITUTION: u8 = 0;

/// `AdminAccountStatus::Active` 的 SCALE 判别值(声明序 Pending=0/Active=1/Closed=2)。
/// 由 `admin-primitives` 测试交叉钉死。
pub const STATUS_ACTIVE: u8 = 1;

/// 某固定治理机构的法庭角色构成约束(当前仅 NJD 有护宪席位约束)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CourtSpec {
    /// 受约束的职务名(逐字节比对 `AdminProfile.role_name`)。
    pub role_name: &'static [u8],
    /// 该职务的固定席位数。
    pub exact_count: u32,
}

/// 单个固定治理机构的冻结规格。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FixedInstitution {
    /// 机构码(NRC/PRC/PRB/NJD)。
    pub code: InstitutionCode,
    /// 机构主账户 = `AdminAccounts` 键(来自 `CHINA_*` 创世常量,编译进二进制)。
    pub main_account: [u8; 32],
    /// 固定管理员总数(仅允许等长换人)。
    pub expected_len: u32,
    /// 法庭角色约束(仅 NJD 为 `Some`)。
    pub court: Option<CourtSpec>,
}

/// 枚举全部固定治理机构冻结规格(纯编译常量,不读链)。
///
/// NRC/PRC 来自 `CHINA_CB`,PRB 来自 `CHINA_CH`,NJD 来自 `CHINA_SF`;三者创世即
/// `insert_fixed_admins` 写入 `AdminAccounts`,故 block#0 state 与本规格双锚。
pub fn fixed_institutions() -> Vec<FixedInstitution> {
    let mut out = Vec::new();
    for node in CHINA_CB.iter() {
        let Some(code) = institution_code_from_cid_number(node.cid_number) else {
            continue;
        };
        if code == NRC {
            out.push(FixedInstitution {
                code: NRC,
                main_account: node.main_account,
                expected_len: NRC_ADMIN_COUNT,
                court: None,
            });
        } else if code == PRC {
            out.push(FixedInstitution {
                code: PRC,
                main_account: node.main_account,
                expected_len: PRC_ADMIN_COUNT,
                court: None,
            });
        }
    }
    for node in CHINA_CH.iter() {
        if institution_code_from_cid_number(node.cid_number) == Some(PRB) {
            out.push(FixedInstitution {
                code: PRB,
                main_account: node.main_account,
                expected_len: PRB_ADMIN_COUNT,
                court: None,
            });
        }
    }
    for node in CHINA_SF.iter() {
        if institution_code_from_cid_number(node.cid_number) == Some(NJD) {
            out.push(FixedInstitution {
                code: NJD,
                main_account: node.main_account,
                expected_len: NJD_ADMIN_COUNT,
                court: Some(CourtSpec {
                    role_name: ROLE_CONSTITUTION_GUARD,
                    exact_count: NJD_CONSTITUTION_GUARD_SEATS,
                }),
            });
        }
    }
    out
}

/// FRG 省行政区治理组冻结规格:每省一条 `(省码, 固定人数=5)`。
///
/// 键 = `FederalRegistryProvinceGroups[省码]`(不是聚合账户);守卫据省码直接读该组。
pub fn frg_province_groups() -> Vec<(ProvinceCode, u32)> {
    PROVINCE_CODE_INFOS
        .iter()
        .map(|p| (p.province_code, FRG_PROVINCE_GROUP_ADMIN_COUNT))
        .collect()
}

#[cfg(test)]
mod tests {
    // 测试代码沿用 expect() 断言(工作区 expect_used=warn 面向生产码;测试内 expect 是惯用法)。
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn fixed_institutions_cover_fixed_codes_with_expected_counts() {
        let list = fixed_institutions();

        let nrc = list.iter().find(|f| f.code == NRC).expect("NRC 必须在册");
        assert_eq!(nrc.expected_len, NRC_ADMIN_COUNT);
        assert!(nrc.court.is_none());

        let njd = list.iter().find(|f| f.code == NJD).expect("NJD 必须在册");
        assert_eq!(njd.expected_len, NJD_ADMIN_COUNT);
        let court = njd.court.expect("NJD 必须有护宪法庭约束");
        assert_eq!(court.role_name, ROLE_CONSTITUTION_GUARD);
        assert_eq!(court.exact_count, NJD_CONSTITUTION_GUARD_SEATS);

        assert!(list
            .iter()
            .any(|f| f.code == PRC && f.expected_len == PRC_ADMIN_COUNT));
        assert!(list
            .iter()
            .any(|f| f.code == PRB && f.expected_len == PRB_ADMIN_COUNT));
    }

    #[test]
    fn njd_guard_seats_is_seven() {
        // 4/7 终审的「7」;与创世 role-by-index 0..=6 一致。
        assert_eq!(NJD_CONSTITUTION_GUARD_SEATS, 7);
    }

    #[test]
    fn frg_groups_cover_all_provinces_with_five_seats() {
        let groups = frg_province_groups();
        assert_eq!(groups.len(), PROVINCE_CODE_INFOS.len());
        assert!(groups
            .iter()
            .all(|(_, n)| *n == FRG_PROVINCE_GROUP_ADMIN_COUNT));
    }

    #[test]
    fn role_literal_is_stable() {
        assert_eq!(ROLE_CONSTITUTION_GUARD, "护宪大法官".as_bytes());
    }
}
