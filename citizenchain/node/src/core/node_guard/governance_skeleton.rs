//! 固定治理骨架节点策略(档 A)。
//!
//! 本模块只负责 RAW storage key、链上结构镜像和 I1..I7 纯不变式判定；区块预执行、warp
//! 提交前检查和 `BlockImport` 委派统一由上层 [`super::NodeGuard`] 编排。这样后续新增发行、
//! CID 等节点永久规则时可以共用一次区块预执行，不再为每条规则叠加独立包装器。
//!
//! 逐块断言的不变式(I1..I7):对每个固定治理机构(NRC/PRC/PRB/NJD)与 43 个 FRG 省组——
//!   I1 `AdminAccounts[主账户]`(FRG:`FederalRegistryProvinceGroups[省码]`)恒存在;
//!   I2 `institution_code` 不变;I3 `kind==PublicInstitution`;I4 `status==Active`;
//!   I5 固定名额不变(NRC=19/PRC=9/PRB=9/NJD=15/FRG 组=5);
//!   I6 NJD `role_name==护宪大法官` 计数恒 7(补 `ConstitutionGuardVoteProof` 的 4/7 里没锚的「7」)。
//!
//! **只冻结构,不冻成员**:普选/互选等长换人(名额/护宪席位数不变)照常放行；稀释、灌水、
//! 删机构、改码和关闭才拒绝。

use std::collections::BTreeMap;

use codec::{Decode, Encode};

use primitives::cid::code::FRG;
use primitives::governance_skeleton::{
    fixed_institutions, frg_province_groups, KIND_PUBLIC_INSTITUTION, STATUS_ACTIVE,
};

/// public-admins pallet 在 `construct_runtime` 中的名字(twox128 前缀据此推导)。
/// 硬编码,绝不读链上 metadata —— metadata 属可升级 runtime,会被恶意升级伪造。
const PALLET_NAME: &[u8] = b"PublicAdmins";

/// 守卫推导出的管理员存储 RAW key,硬编码 hasher 与链端一致:
/// `AdminAccounts: StorageMap<Blake2_128Concat, AccountId, ..>`、
/// `FederalRegistryProvinceGroups: StorageMap<Blake2_128Concat, ProvinceCode, ..>`。
pub mod storage_key {
    use super::PALLET_NAME;
    use sp_core::hashing::twox_128;

    // `crate::shared::storage_keys` 单源的薄委托(pallet 固定为 PALLET_NAME)。
    fn map_prefix(storage: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::prefix(PALLET_NAME, storage)
    }

    fn blake2_128_concat(encoded: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_128_concat(encoded)
    }

    /// `PublicAdmins::AdminAccounts[account]` 的完整存储 key。
    /// `AccountId = AccountId32`,SCALE 编码即 32 裸字节(无长度前缀)。
    pub fn admin_account(account: &[u8; 32]) -> Vec<u8> {
        let mut k = map_prefix(b"AdminAccounts");
        k.extend_from_slice(&blake2_128_concat(account));
        k
    }

    /// `PublicAdmins::FederalRegistryProvinceGroups[province]` 的完整存储 key。
    /// `ProvinceCode = [u8; 2]`,SCALE 编码即 2 裸字节。
    pub fn frg_group(province: &[u8; 2]) -> Vec<u8> {
        let mut k = map_prefix(b"FederalRegistryProvinceGroups");
        k.extend_from_slice(&blake2_128_concat(province));
        k
    }

    /// public-admins 存储的公共前缀(twox128(pallet)),用于快速判断区块是否动过管理员存储。
    pub fn pallet_prefix() -> [u8; 16] {
        twox_128(PALLET_NAME)
    }
}

// ───────── 链上结构镜像 ─────────
// 字段序必须与 `admin-primitives::{AdminAccount, AdminProfile}` 严格一致(SCALE 按声明序解码)。
// kind/status 的 u8 判别值由 admin-primitives 测试 `scale_discriminants_match_governance_skeleton`
// 交叉钉死;护宪 role 字面量单源 `primitives::governance_skeleton::ROLE_CONSTITUTION_GUARD`。
// Encode 仅供单测构造字节。守卫只用 institution_code/kind/admins(role_name)/status,其余字段占位保序。

#[derive(Decode, Encode)]
#[allow(dead_code)]
struct MAdminProfile {
    admin_account: [u8; 32],
    admin_cid_number: Vec<u8>,
    admin_name: Vec<u8>,
    role_code: Vec<u8>,
    role_name: Vec<u8>,
    term_start: u32,
    term_end: u32,
    admin_source: u8,
    admin_source_ref: Vec<u8>,
}

#[derive(Decode, Encode)]
#[allow(dead_code)]
struct MAdminAccount {
    cid_number: Vec<u8>,
    institution_code: [u8; 4],
    kind: u8,
    admins: Vec<MAdminProfile>,
    creator: [u8; 32],
    created_at: u32,
    updated_at: u32,
    status: u8,
}

/// 固定治理骨架守卫的判定失败原因(全部一律拒块/拒启,fail-safe 方向恒为「拒绝」)。
#[derive(Debug, PartialEq)]
pub enum GuardError {
    /// 某固定治理机构 `AdminAccounts[主账户]` 在目标状态缺失(被删/改键)。
    FixedInstitutionMissing([u8; 4]),
    /// 某固定治理机构 `AdminAccount` 解码失败。
    AdminAccountDecodeFailed([u8; 4]),
    /// 机构码被改(不再是规格机构码)。
    InstitutionCodeChanged([u8; 4]),
    /// 机构类型被改(不再是 PublicInstitution)。
    KindChanged([u8; 4]),
    /// 机构被置为非 Active(违反固定机构永不可关闭/挂起)。
    NotActive([u8; 4]),
    /// 固定名额被改(仅允许等长换人)。
    AdminsLenChanged {
        code: [u8; 4],
        expected: u32,
        found: u32,
    },
    /// NJD 护宪大法官席位数被改(违反第21条 4/7 的「7」)。
    CourtSizeChanged {
        code: [u8; 4],
        expected: u32,
        found: u32,
    },
    /// 某 FRG 省组 `FederalRegistryProvinceGroups[省码]` 缺失。
    FrgGroupMissing([u8; 2]),
    /// 某 FRG 省组解码失败。
    FrgGroupDecodeFailed([u8; 2]),
    /// 某 FRG 省组结构不符(码≠FRG / 非 Active / 人数≠5)。
    FrgGroupInvalid([u8; 2]),
}

fn decode_admin_account(raw: &[u8]) -> Result<MAdminAccount, ()> {
    let mut input = raw;
    let account = MAdminAccount::decode(&mut input).map_err(|_| ())?;
    if !input.is_empty() {
        return Err(());
    }
    Ok(account)
}

/// 纯判定:给定一个指向**目标状态**的 RAW 读取闭包,校验固定治理骨架全部不变式(I1..I7)。
/// 规格来自 `primitives::governance_skeleton`(编译常量,不读链)。任一缺失/解码失败/不符 → `Err`。
pub fn check_skeleton_invariants<F>(read_raw: F) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    // ── 固定治理机构(NRC/PRC/PRB/NJD)──
    for inst in fixed_institutions() {
        let raw = read_raw(&storage_key::admin_account(&inst.main_account))
            .ok_or(GuardError::FixedInstitutionMissing(inst.code))?;
        let account = decode_admin_account(&raw)
            .map_err(|_| GuardError::AdminAccountDecodeFailed(inst.code))?;

        if account.institution_code != inst.code {
            return Err(GuardError::InstitutionCodeChanged(inst.code)); // I2
        }
        if account.kind != KIND_PUBLIC_INSTITUTION {
            return Err(GuardError::KindChanged(inst.code)); // I3
        }
        if account.status != STATUS_ACTIVE {
            return Err(GuardError::NotActive(inst.code)); // I4
        }
        let found_len = account.admins.len() as u32;
        if found_len != inst.expected_len {
            return Err(GuardError::AdminsLenChanged {
                code: inst.code,
                expected: inst.expected_len,
                found: found_len,
            }); // I5
        }
        if let Some(court) = inst.court {
            let found = account
                .admins
                .iter()
                .filter(|p| p.role_name.as_slice() == court.role_name)
                .count() as u32;
            if found != court.exact_count {
                return Err(GuardError::CourtSizeChanged {
                    code: inst.code,
                    expected: court.exact_count,
                    found,
                }); // I6
            }
        }
    }

    // ── FRG 43 省行政区组(I7)──
    for (province, expected_len) in frg_province_groups() {
        let raw = read_raw(&storage_key::frg_group(&province))
            .ok_or(GuardError::FrgGroupMissing(province))?;
        let account =
            decode_admin_account(&raw).map_err(|_| GuardError::FrgGroupDecodeFailed(province))?;
        if account.institution_code != FRG
            || account.kind != KIND_PUBLIC_INSTITUTION
            || account.status != STATUS_ACTIVE
            || account.admins.len() as u32 != expected_len
        {
            return Err(GuardError::FrgGroupInvalid(province));
        }
    }

    Ok(())
}

/// 是否必须跑完整骨架校验。普通块只要触及 public-admins 存储或 `:code` runtime 升级,
/// 就不能走快路径;其余块按归纳假设跳过。
pub(super) fn needs_full_check(delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> bool {
    let prefix = storage_key::pallet_prefix();
    delta.keys().any(|k| k.starts_with(&prefix))
        || delta.contains_key(sp_storage::well_known_keys::CODE)
}

#[cfg(test)]
mod tests {
    // 测试代码沿用 expect() 断言(工作区 expect_used=warn 面向生产码;测试内 expect 是惯用法)。
    #![allow(clippy::expect_used)]
    use super::*;
    use primitives::governance_skeleton::ROLE_CONSTITUTION_GUARD;

    const STATUS_PENDING: u8 = 0;

    fn profile(role_name: &[u8]) -> MAdminProfile {
        MAdminProfile {
            admin_account: [0u8; 32],
            admin_cid_number: Vec::new(),
            admin_name: Vec::new(),
            role_code: Vec::new(),
            role_name: role_name.to_vec(),
            term_start: 0,
            term_end: 0,
            admin_source: 0,
            admin_source_ref: Vec::new(),
        }
    }

    fn account_bytes(code: [u8; 4], kind: u8, status: u8, admins: Vec<MAdminProfile>) -> Vec<u8> {
        MAdminAccount {
            cid_number: Vec::new(),
            institution_code: code,
            kind,
            admins,
            creator: [0u8; 32],
            created_at: 0,
            updated_at: 0,
            status,
        }
        .encode()
    }

    /// 为某固定机构造一份合法管理员集:总数达标;NJD 前 7 名为护宪,其余大法官。
    fn valid_admins_for(
        expected_len: u32,
        court: Option<primitives::governance_skeleton::CourtSpec>,
    ) -> Vec<MAdminProfile> {
        let guard_seats = court.map(|c| c.exact_count).unwrap_or(0);
        (0..expected_len)
            .map(|i| {
                if i < guard_seats {
                    profile(ROLE_CONSTITUTION_GUARD)
                } else {
                    profile("大法官".as_bytes())
                }
            })
            .collect()
    }

    /// 一份完整合法当前态:全部固定机构 + 43 FRG 省组均达标。
    fn valid_state() -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut m = BTreeMap::new();
        for inst in fixed_institutions() {
            m.insert(
                storage_key::admin_account(&inst.main_account),
                account_bytes(
                    inst.code,
                    KIND_PUBLIC_INSTITUTION,
                    STATUS_ACTIVE,
                    valid_admins_for(inst.expected_len, inst.court),
                ),
            );
        }
        for (province, expected_len) in frg_province_groups() {
            m.insert(
                storage_key::frg_group(&province),
                account_bytes(
                    FRG,
                    KIND_PUBLIC_INSTITUTION,
                    STATUS_ACTIVE,
                    valid_admins_for(expected_len, None),
                ),
            );
        }
        m
    }

    fn reader(map: BTreeMap<Vec<u8>, Vec<u8>>) -> impl Fn(&[u8]) -> Option<Vec<u8>> {
        move |k: &[u8]| map.get(k).cloned()
    }

    #[test]
    fn valid_state_passes() {
        assert_eq!(check_skeleton_invariants(reader(valid_state())), Ok(()));
    }

    #[test]
    fn missing_fixed_institution_is_rejected() {
        let njd = fixed_institutions()
            .into_iter()
            .find(|f| f.court.is_some())
            .expect("NJD 在册");
        let mut m = valid_state();
        m.remove(&storage_key::admin_account(&njd.main_account));
        assert_eq!(
            check_skeleton_invariants(reader(m)),
            Err(GuardError::FixedInstitutionMissing(njd.code))
        );
    }

    #[test]
    fn njd_court_dilution_is_rejected() {
        // NJD 保持 15 人但护宪从 7 降为 6(1 名改判大法官)→ CourtSizeChanged。
        let njd = fixed_institutions()
            .into_iter()
            .find(|f| f.court.is_some())
            .expect("NJD 在册");
        let mut admins = valid_admins_for(njd.expected_len, njd.court);
        admins[0] = profile("大法官".as_bytes()); // 抽掉 1 名护宪
        let mut m = valid_state();
        m.insert(
            storage_key::admin_account(&njd.main_account),
            account_bytes(njd.code, KIND_PUBLIC_INSTITUTION, STATUS_ACTIVE, admins),
        );
        assert_eq!(
            check_skeleton_invariants(reader(m)),
            Err(GuardError::CourtSizeChanged {
                code: njd.code,
                expected: 7,
                found: 6,
            })
        );
    }

    #[test]
    fn njd_equal_length_reshuffle_keeping_seven_guards_passes() {
        // 等长换人:仍 15 人、仍 7 护宪,只是角色分布不同 → 放行(不冻成员)。
        let njd = fixed_institutions()
            .into_iter()
            .find(|f| f.court.is_some())
            .expect("NJD 在册");
        let mut admins = valid_admins_for(njd.expected_len, njd.court);
        // 打乱:把最后一名大法官也设为护宪、同时把一名护宪设为大法官 → 护宪数仍 7。
        let n = admins.len();
        admins[n - 1] = profile(ROLE_CONSTITUTION_GUARD);
        admins[0] = profile("大法官".as_bytes());
        let mut m = valid_state();
        m.insert(
            storage_key::admin_account(&njd.main_account),
            account_bytes(njd.code, KIND_PUBLIC_INSTITUTION, STATUS_ACTIVE, admins),
        );
        assert_eq!(check_skeleton_invariants(reader(m)), Ok(()));
    }

    #[test]
    fn admins_len_change_is_rejected() {
        let njd = fixed_institutions()
            .into_iter()
            .find(|f| f.court.is_some())
            .expect("NJD 在册");
        let mut admins = valid_admins_for(njd.expected_len, njd.court);
        admins.pop(); // 14 人
        let mut m = valid_state();
        m.insert(
            storage_key::admin_account(&njd.main_account),
            account_bytes(njd.code, KIND_PUBLIC_INSTITUTION, STATUS_ACTIVE, admins),
        );
        assert_eq!(
            check_skeleton_invariants(reader(m)),
            Err(GuardError::AdminsLenChanged {
                code: njd.code,
                expected: njd.expected_len,
                found: njd.expected_len - 1,
            })
        );
    }

    #[test]
    fn non_active_status_is_rejected() {
        let njd = fixed_institutions()
            .into_iter()
            .find(|f| f.court.is_some())
            .expect("NJD 在册");
        let mut m = valid_state();
        m.insert(
            storage_key::admin_account(&njd.main_account),
            account_bytes(
                njd.code,
                KIND_PUBLIC_INSTITUTION,
                STATUS_PENDING,
                valid_admins_for(njd.expected_len, njd.court),
            ),
        );
        assert_eq!(
            check_skeleton_invariants(reader(m)),
            Err(GuardError::NotActive(njd.code))
        );
    }

    #[test]
    fn kind_change_is_rejected() {
        let njd = fixed_institutions()
            .into_iter()
            .find(|f| f.court.is_some())
            .expect("NJD 在册");
        let mut m = valid_state();
        m.insert(
            storage_key::admin_account(&njd.main_account),
            account_bytes(
                njd.code,
                1, // PrivateInstitution
                STATUS_ACTIVE,
                valid_admins_for(njd.expected_len, njd.court),
            ),
        );
        assert_eq!(
            check_skeleton_invariants(reader(m)),
            Err(GuardError::KindChanged(njd.code))
        );
    }

    #[test]
    fn frg_group_undersized_is_rejected() {
        let (province, expected_len) = frg_province_groups()[0];
        let mut admins = valid_admins_for(expected_len, None);
        admins.pop(); // 4 人
        let mut m = valid_state();
        m.insert(
            storage_key::frg_group(&province),
            account_bytes(FRG, KIND_PUBLIC_INSTITUTION, STATUS_ACTIVE, admins),
        );
        assert_eq!(
            check_skeleton_invariants(reader(m)),
            Err(GuardError::FrgGroupInvalid(province))
        );
    }

    #[test]
    fn key_derivation_is_stable_and_prefixed() {
        let acc = [7u8; 32];
        assert_eq!(
            storage_key::admin_account(&acc),
            storage_key::admin_account(&acc)
        );
        assert!(storage_key::admin_account(&acc).starts_with(&storage_key::pallet_prefix()));
        assert!(storage_key::frg_group(&[1, 2]).starts_with(&storage_key::pallet_prefix()));
        assert_ne!(
            storage_key::admin_account(&acc),
            storage_key::frg_group(&[0, 0])
        );
    }

    #[test]
    fn fast_path_only_triggers_on_public_admins_or_code() {
        // 触及 PublicAdmins 前缀 → 需全量校验。
        let mut touched: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();
        touched.insert(storage_key::admin_account(&[9u8; 32]), Some(vec![1]));
        assert!(needs_full_check(&touched));

        // 升级 `:code` → 需全量校验(高危块)。
        let mut code: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();
        code.insert(sp_storage::well_known_keys::CODE.to_vec(), Some(vec![0]));
        assert!(needs_full_check(&code));

        // 无关键 → 快路径跳过。
        let mut other: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();
        other.insert(b"unrelated-key".to_vec(), Some(vec![0]));
        assert!(!needs_full_check(&other));
    }

    /// 真链创世双锚:用 runtime 创世构建器生成含 `genesis::institution::build` 播种的
    /// 固定治理机构管理员集 + FRG 省组的真 state,再用节点镜像 + 存储键 + 规格逐条校验。
    /// 这是 `MAdminAccount` 字段序 / 存储键推导 / 规格计数与真链的最强交叉钉死;守卫 `new()`
    /// 启动期跑的正是同一套校验,故本测试等价于"创世能过启动双锚"的离线确认。
    #[test]
    fn real_runtime_genesis_satisfies_skeleton_invariants() {
        use sp_runtime::BuildStorage;
        let storage = citizenchain::RuntimeGenesisConfig::default()
            .build_storage()
            .expect("build runtime genesis storage");
        let top = storage.top;
        assert_eq!(check_skeleton_invariants(|k| top.get(k).cloned()), Ok(()));
    }

    #[test]
    fn trailing_bytes_in_fixed_or_frg_records_are_rejected() {
        let first = fixed_institutions()[0];
        let fixed_key = storage_key::admin_account(&first.main_account);
        let mut fixed_state = valid_state();
        fixed_state
            .get_mut(&fixed_key)
            .expect("fixed institution")
            .push(0xff);
        assert_eq!(
            check_skeleton_invariants(reader(fixed_state)),
            Err(GuardError::AdminAccountDecodeFailed(first.code))
        );

        let province = frg_province_groups()[0].0;
        let frg_key = storage_key::frg_group(&province);
        let mut frg_state = valid_state();
        frg_state.get_mut(&frg_key).expect("FRG group").push(0xff);
        assert_eq!(
            check_skeleton_invariants(reader(frg_state)),
            Err(GuardError::FrgGroupDecodeFailed(province))
        );
    }
}
