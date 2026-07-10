//! 固定治理骨架守卫(档 A,L2 共识层)。
//!
//! `admins-change` 的真源 `public-admins::{AdminAccounts, FederalRegistryProvinceGroups}` 是全部
//! 机构管理员角色的唯一真源;护宪大法官 4/7 终审等治理其可信度封顶在该真源完整性上,而它是纯
//! runtime state,一次 setCode/恶意 runtime 可任意改写。本守卫把**永不合法变更的结构骨架**冻到
//! 节点二进制 + 创世(block#0),在区块导入时逐块背书,违者拒块——执法在 runtime 之外,setCode
//! 改不动。规格单源 = `primitives::governance_skeleton`(与创世播种、runtime 校验三端共读)。
//!
//! 逐块断言的不变式(I1..I7):对每个固定治理机构(NRC/PRC/PRB/NJD)与 43 个 FRG 省组——
//!   I1 `AdminAccounts[主账户]`(FRG:`FederalRegistryProvinceGroups[省码]`)恒存在;
//!   I2 `institution_code` 不变;I3 `kind==PublicInstitution`;I4 `status==Active`;
//!   I5 固定名额不变(NRC=19/PRC=9/PRB=9/NJD=15/FRG 组=5);
//!   I6 NJD `role_name==护宪大法官` 计数恒 7(补 `ConstitutionGuardVoteProof` 的 4/7 里没锚的「7」)。
//!
//! **只冻结构,不冻成员**:普选/互选等长换人(名额/护宪席位数不变)照常放行;稀释/灌水/删机构/
//! 改码/关闭才拒块。**天花板**:保持席位数、整体换攻击者密钥的成员劫持不在本守卫范围(节点无独立
//! 预言机判合法当选),留档 B(创世根验签链)。判定路径完全复刻 `constitution.rs`。

use std::collections::BTreeMap;
use std::sync::Arc;

use codec::{Decode, Encode};
use sc_client_api::backend::{Backend as _, TrieCacheContext};
use sc_client_api::StorageProvider;
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ImportResult, StateAction, StorageChanges,
};
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_storage::StorageKey;

use primitives::cid::code::FRG;
use primitives::governance_skeleton::{
    fixed_institutions, frg_province_groups, KIND_PUBLIC_INSTITUTION, STATUS_ACTIVE,
};

use citizenchain::opaque::Block;

use crate::core::service::{FullBackend, FullClient};

/// public-admins pallet 在 `construct_runtime` 中的名字(twox128 前缀据此推导)。
/// 硬编码,绝不读链上 metadata —— metadata 属可升级 runtime,会被恶意升级伪造。
const PALLET_NAME: &[u8] = b"PublicAdmins";

/// 守卫推导出的管理员存储 RAW key,硬编码 hasher 与链端一致:
/// `AdminAccounts: StorageMap<Blake2_128Concat, AccountId, ..>`、
/// `FederalRegistryProvinceGroups: StorageMap<Blake2_128Concat, ProvinceCode, ..>`。
pub mod storage_key {
    use super::PALLET_NAME;
    use sp_core::hashing::{blake2_128, twox_128};

    fn map_prefix(storage: &[u8]) -> Vec<u8> {
        let mut k = Vec::with_capacity(32);
        k.extend_from_slice(&twox_128(PALLET_NAME));
        k.extend_from_slice(&twox_128(storage));
        k
    }

    fn blake2_128_concat(encoded: &[u8]) -> Vec<u8> {
        let mut out = blake2_128(encoded).to_vec();
        out.extend_from_slice(encoded);
        out
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
        let account = MAdminAccount::decode(&mut &raw[..])
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
        let account = MAdminAccount::decode(&mut &raw[..])
            .map_err(|_| GuardError::FrgGroupDecodeFailed(province))?;
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
fn needs_full_check(delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> bool {
    let prefix = storage_key::pallet_prefix();
    delta.keys().any(|k| k.starts_with(&prefix))
        || delta.contains_key(sp_storage::well_known_keys::CODE)
}

/// 区块导入守卫:包住内层 `BlockImport`,在区块进入规范链之前校验固定治理骨架不变式。
///
/// 与 `ConstitutionGuard` 并列(互相独立)串在导入栈里。判定路径:对携带 body 的普通块,先用
/// runtime API 在**父状态**上只读执行取后置存储变更,仅当变更触及 public-admins 存储或 `:code`
/// 时据「变更 ∪ 父状态」比对规格;命中违规 → `Ok(KnownBad)`;通过 → 委派内层正常导入。
pub struct GovernanceSkeletonGuard<I> {
    inner: I,
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
}

impl<I> GovernanceSkeletonGuard<I> {
    /// 装配守卫:启动即做创世双锚——从 block#0 state 读固定骨架,必须已满足编译规格,否则拒绝启动
    /// (fail-closed;创世不合法即换链/改二进制)。
    pub fn new(
        inner: I,
        client: Arc<FullClient>,
        backend: Arc<FullBackend>,
    ) -> Result<Self, String> {
        let genesis_hash = client.info().genesis_hash;
        check_skeleton_invariants(|key| {
            client
                .storage(genesis_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        })
        .map_err(|e| format!("骨架守卫:创世固定治理骨架基准校验失败:{e:?}"))?;

        Ok(Self {
            inner,
            client,
            backend,
        })
    }

    /// **提交前**校验 warp/状态导入块携带的下载态骨架不变式(vendored GRANDPA 在 `inner.import_block`
    /// 内即落库,post-import 拒块无法回滚,故必须在调用 inner **之前**校验)。抽 public-admins 前缀键
    /// (仅几十 KB)跑全套不变式。`Err` = 违规或无法抽取(拒绝,fail-closed)。
    fn verify_imported_state(&self, params: &BlockImportParams<Block>) -> Result<(), String> {
        let imported = match &params.state_action {
            StateAction::ApplyChanges(StorageChanges::Import(imported)) => imported,
            _ => return Err("warp 状态非 ApplyChanges(Import) 形态,拒绝(无法提交前校验)".into()),
        };
        let prefix = storage_key::pallet_prefix();
        let mut map: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        for (key, value) in imported
            .state
            .0
            .iter()
            .flat_map(|level| level.key_values.iter())
        {
            if key.starts_with(&prefix) {
                map.insert(key.clone(), value.clone());
            }
        }
        check_skeleton_invariants(|key| map.get(key).cloned()).map_err(|e| format!("{e:?}"))
    }

    /// 计算普通(执行型)区块后置状态是否违反骨架不变式。
    /// `Ok(true)` = 确认违规(拒块);`Ok(false)` = 合规;`Err` = 无法判定(`import_block` fail-closed 拒块)。
    fn detect_violation(&self, params: &BlockImportParams<Block>) -> Result<bool, String> {
        let body = match &params.body {
            Some(b) => b.clone(),
            None => return Ok(false), // 无 body 且非状态导入,不经执行改 state,跳过
        };

        let parent_hash = *params.header.parent_hash();
        let block = Block::new(params.header.clone(), body);

        // 在父状态上只读执行该区块(不提交),取后置存储变更。
        let api = self.client.runtime_api();
        api.execute_block(parent_hash, block.into())
            .map_err(|e| format!("只读执行区块失败:{e}"))?;
        let parent_state = self
            .backend
            .state_at(parent_hash, TrieCacheContext::Untrusted)
            .map_err(|e| format!("取父状态失败:{e}"))?;
        let changes = api
            .into_storage_changes(&parent_state, parent_hash)
            .map_err(|e| format!("提取存储变更失败:{e}"))?;

        // 快路径:本块既未动 public-admins 存储、也未升级 runtime(`:code`)→ 归纳骨架不变,合规。
        let delta: BTreeMap<Vec<u8>, Option<Vec<u8>>> =
            changes.main_storage_changes.into_iter().collect();
        if !needs_full_check(&delta) {
            return Ok(false);
        }

        // 后置状态读取器:命中变更取变更值(Some=改、None=删),否则回落父状态(已提交)。
        let read_post = |key: &[u8]| -> Option<Vec<u8>> {
            match delta.get(key) {
                Some(value) => value.clone(),
                None => self
                    .client
                    .storage(parent_hash, &StorageKey(key.to_vec()))
                    .ok()
                    .flatten()
                    .map(|data| data.0),
            }
        };

        match check_skeleton_invariants(read_post) {
            Ok(()) => Ok(false),
            Err(reason) => {
                log::error!(
                    target: "governance-skeleton-guard",
                    "拒绝区块 #{} ({:?}):固定治理骨架不变式被破坏 —— {:?}",
                    params.header.number(),
                    params.post_hash(),
                    reason,
                );
                Ok(true)
            }
        }
    }
}

#[async_trait::async_trait]
impl<I> BlockImport<Block> for GovernanceSkeletonGuard<I>
where
    I: BlockImport<Block, Error = ConsensusError> + Send + Sync,
{
    type Error = ConsensusError;

    async fn check_block(
        &self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(
        &self,
        params: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        // warp/状态同步块:vendored GRANDPA 在 inner 内即落库,无法事后回滚,故提交前校验。
        if params.with_state() {
            return match self.verify_imported_state(&params) {
                Ok(()) => self.inner.import_block(params).await,
                Err(reason) => {
                    log::error!(
                        target: "governance-skeleton-guard",
                        "拒绝 warp/状态导入 ({:?}):固定治理骨架校验未通过 —— {reason}",
                        params.post_hash(),
                    );
                    Ok(ImportResult::KnownBad)
                }
            };
        }

        // 普通(执行型)块:执行前判定,违规 KnownBad(内层永不被调用)。
        match self.detect_violation(&params) {
            Ok(true) => Ok(ImportResult::KnownBad),
            Ok(false) => self.inner.import_block(params).await,
            // fail-closed:守卫自身取数/执行/解码失败 → 拒块,不放行未经校验的块。
            Err(why) => {
                log::error!(
                    target: "governance-skeleton-guard",
                    "守卫判定失败,fail-closed 拒块 ({:?}):{why}",
                    params.post_hash(),
                );
                Ok(ImportResult::KnownBad)
            }
        }
    }
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
}
