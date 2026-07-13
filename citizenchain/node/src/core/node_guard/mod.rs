//! 节点守卫统一 `BlockImport` 包装器。
//!
//! 公民宪法是整条链最高规则，继续由独立的 `ConstitutionGuard` 在本包装器外层先行检查。
//! 本模块只收口**除宪法外**的节点永久规则：统一预执行正常区块、统一提取后置 storage delta，
//! 再把同一份检查上下文交给内部策略。当前已注册固定治理骨架、三类固定发行、GenesisPallet
//! 五字段与 CID 生命周期；后续非宪法永久规则仍必须加在本包装器内部，不得新增平行包装器。

mod cid_lifecycle;
mod citizen_issuance;
mod fullnode_issuance;
mod genesis_pallet;
mod governance_skeleton;
mod pow_difficulty;
mod provincialbank_interest;

use std::collections::BTreeMap;
use std::sync::Arc;

use codec::Decode;
use sc_client_api::backend::{Backend as _, TrieCacheContext};
use sc_client_api::StorageProvider;
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ImportResult, StateAction, StorageChanges,
};
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::HeaderBackend;
use sp_consensus::Error as ConsensusError;
use sp_core::hashing::blake2_128;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_storage::StorageKey;

use citizenchain::opaque::Block;

use crate::core::service::{FullBackend, FullClient};

/// 除宪法外的节点永久规则统一导入包装器。
pub struct NodeGuard<I> {
    inner: I,
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    cid_lifecycle: cid_lifecycle::GenesisReference,
}

/// 所有合法 finalize 原生发行按账户汇总后，由 `NodeGuard` 统一核对余额和总发行量。
#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct FinalizeIssuancePlan {
    accounts: BTreeMap<[u8; 32], u128>,
    total: u128,
}

impl FinalizeIssuancePlan {
    pub(super) fn add(&mut self, account: [u8; 32], amount: u128) -> Result<(), ()> {
        let next = self
            .accounts
            .get(&account)
            .copied()
            .unwrap_or_default()
            .checked_add(amount)
            .ok_or(())?;
        self.total = self.total.checked_add(amount).ok_or(())?;
        self.accounts.insert(account, next);
        Ok(())
    }
}

/// 两层守卫共用的最终导入闸门：只有全部原生规则明确验证成功才允许调用内层导入器。
///
/// 把委派动作单独收口后，测试可以直接证明 `Err` 路径返回 `KnownBad` 且内层调用次数为零，
/// 避免某个包装器以后在日志分支中误调用内层导入器。
pub(super) async fn import_if_verified<I>(
    inner: &I,
    params: BlockImportParams<Block>,
    verdict: Result<(), String>,
) -> Result<ImportResult, ConsensusError>
where
    I: BlockImport<Block, Error = ConsensusError> + Send + Sync,
{
    match verdict {
        Ok(()) => inner.import_block(params).await,
        Err(_) => Ok(ImportResult::KnownBad),
    }
}

/// 节点镜像的 `frame_system::AccountInfo<u32, pallet_balances::AccountData<u128>>`。
#[derive(Debug, Decode, Eq, PartialEq)]
pub(super) struct MAccountInfo {
    nonce: u32,
    consumers: u32,
    providers: u32,
    sufficients: u32,
    data: MAccountData,
}

#[derive(Debug, Decode, Eq, PartialEq)]
pub(super) struct MAccountData {
    free: u128,
    reserved: u128,
    frozen: u128,
    flags: u128,
}

fn decode_exact<T: Decode>(raw: &[u8], label: &str) -> Result<T, String> {
    let mut input = raw;
    let value = T::decode(&mut input).map_err(|_| format!("{label} SCALE 解码失败"))?;
    if !input.is_empty() {
        return Err(format!("{label} 存在尾随字节"));
    }
    Ok(value)
}

fn signed_delta(before: u128, after: u128) -> i128 {
    if after >= before {
        i128::try_from(after - before).unwrap_or(i128::MAX)
    } else {
        -i128::try_from(before - after).unwrap_or(i128::MAX)
    }
}

/// timestamp inherent 之外必须至少有一笔用户交易；在执行 runtime 前判断，避免空提案触发 panic。
fn has_user_transaction(extrinsic_count: usize) -> bool {
    extrinsic_count > 1
}

fn parse_system_account_key(key: &[u8], prefix: &[u8]) -> Result<[u8; 32], String> {
    if !key.starts_with(prefix) || key.len() != prefix.len() + 48 {
        return Err("System::Account RAW key 形态错误".into());
    }
    let hash_at = prefix.len();
    let account_at = hash_at + 16;
    let account: [u8; 32] = key[account_at..]
        .try_into()
        .map_err(|_| "System::Account 账户长度错误")?;
    if blake2_128(&account) != key[hash_at..account_at] {
        return Err("System::Account Blake2_128Concat 校验失败".into());
    }
    Ok(account)
}

fn account_info<F>(read: &F, account: &[u8; 32]) -> Result<Option<MAccountInfo>, String>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    read(&fullnode_issuance::storage_key::system_account(account))
        .map(|raw| decode_exact(&raw, "System::Account"))
        .transpose()
}

#[derive(Default)]
struct ImportedPolicyState {
    governance: BTreeMap<Vec<u8>, Vec<u8>>,
    fullnode_issuance: BTreeMap<Vec<u8>, Vec<u8>>,
    citizen_issuance: BTreeMap<Vec<u8>, Vec<u8>>,
    genesis_pallet: BTreeMap<Vec<u8>, Vec<u8>>,
    pow_difficulty: BTreeMap<Vec<u8>, Vec<u8>>,
    provincialbank_interest: BTreeMap<Vec<u8>, Vec<u8>>,
    cid: BTreeMap<Vec<u8>, Vec<u8>>,
    scanned: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImportedPolicyStats {
    scanned: usize,
    governance: usize,
    fullnode_issuance: usize,
    citizen_issuance: usize,
    genesis_pallet: usize,
    pow_difficulty: usize,
    provincialbank_interest: usize,
    cid: usize,
}

impl ImportedPolicyState {
    fn stats(&self) -> ImportedPolicyStats {
        ImportedPolicyStats {
            scanned: self.scanned,
            governance: self.governance.len(),
            fullnode_issuance: self.fullnode_issuance.len(),
            citizen_issuance: self.citizen_issuance.len(),
            genesis_pallet: self.genesis_pallet.len(),
            pow_difficulty: self.pow_difficulty.len(),
            provincialbank_interest: self.provincialbank_interest.len(),
            cid: self.cid.len(),
        }
    }
}

/// 对完整下载态只遍历一次，并把所有内部策略需要的 RAW 状态分流到共享快照。
fn partition_imported_state<'a, I>(pairs: I) -> ImportedPolicyState
where
    I: IntoIterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    let governance_prefix = governance_skeleton::storage_key::pallet_prefix();
    let fullnode_prefix = fullnode_issuance::storage_key::pallet_prefix();
    let system_prefix = fullnode_issuance::storage_key::system_prefix();
    let total_issuance_key = fullnode_issuance::storage_key::total_issuance();
    let citizen_issuance_prefix = citizen_issuance::storage_key::pallet_prefix();
    let genesis_pallet_prefix = genesis_pallet::storage_key::pallet_prefix();
    let pow_difficulty_prefix = pow_difficulty::storage_key::pallet_prefix();
    let provincialbank_prefix = provincialbank_interest::storage_key::pallet_prefix();
    let provincialbank_keys = provincialbank_interest::relevant_import_keys();
    let cid_prefixes = cid_lifecycle::storage_key::relevant_prefixes();
    let mut state = ImportedPolicyState::default();
    for (key, value) in pairs {
        state.scanned = state.scanned.saturating_add(1);
        if key.starts_with(&governance_prefix) {
            state.governance.insert(key.clone(), value.clone());
        }
        if key.starts_with(&fullnode_prefix)
            || key.starts_with(&system_prefix)
            || key == &total_issuance_key
        {
            state.fullnode_issuance.insert(key.clone(), value.clone());
        }
        if cid_lifecycle::matches_relevant_prefixes(key, &cid_prefixes) {
            state.cid.insert(key.clone(), value.clone());
        }
        if key.starts_with(&citizen_issuance_prefix) {
            state.citizen_issuance.insert(key.clone(), value.clone());
        }
        if key.starts_with(&genesis_pallet_prefix) {
            state.genesis_pallet.insert(key.clone(), value.clone());
        }
        if key.starts_with(&pow_difficulty_prefix) {
            state.pow_difficulty.insert(key.clone(), value.clone());
        }
        if key.starts_with(&provincialbank_prefix) || provincialbank_keys.contains(key) {
            state
                .provincialbank_interest
                .insert(key.clone(), value.clone());
        }
    }
    state
}

/// 校验完整下载态中的全部 NodeGuard 策略，并返回单遍扫描统计供日志和回归测试核对。
fn verify_imported_policy_state<'a, I>(
    block: u32,
    pairs: I,
    cid_reference: &cid_lifecycle::GenesisReference,
) -> Result<ImportedPolicyStats, String>
where
    I: IntoIterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    cid_lifecycle::check_state_import_height(block).map_err(|e| format!("CID 生命周期:{e:?}"))?;
    let state = partition_imported_state(pairs);
    governance_skeleton::check_skeleton_invariants(|key| state.governance.get(key).cloned())
        .map_err(|e| format!("固定治理骨架:{e:?}"))?;
    fullnode_issuance::check_imported_state_key_values(block, state.fullnode_issuance.iter())
        .map_err(|e| format!("全节点发行:{e:?}"))?;
    citizen_issuance::check_genesis_key_values(state.citizen_issuance.iter())
        .map_err(|e| format!("公民认证发行:{e:?}"))?;
    genesis_pallet::check_imported_state(state.genesis_pallet.iter())
        .map_err(|e| format!("创世模块:{e:?}"))?;
    pow_difficulty::check_imported_genesis(state.pow_difficulty.iter())
        .map_err(|e| format!("PoW 动态难度:{e:?}"))?;
    provincialbank_interest::check_imported_genesis(state.provincialbank_interest.iter())
        .map_err(|e| format!("省储行固定发行:{e:?}"))?;
    cid_lifecycle::check_imported_genesis(state.cid.iter(), cid_reference)
        .map_err(|e| format!("CID 生命周期:{e:?}"))?;
    Ok(state.stats())
}

/// finalize 阶段只允许已注册发行计划改变账户和 `TotalIssuance`，其余变化一律 fail-closed。
fn verify_finalize_issuance<FPre, FPost>(
    pre_delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    post_delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    pre: &FPre,
    post: &FPost,
    plan: &FinalizeIssuancePlan,
) -> Result<(), String>
where
    FPre: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let total_key = fullnode_issuance::storage_key::total_issuance();
    let total_before: u128 = decode_exact(
        &pre(&total_key).ok_or("finalize 前缺少 Balances::TotalIssuance")?,
        "Balances::TotalIssuance",
    )?;
    let total_after: u128 = decode_exact(
        &post(&total_key).ok_or("finalize 后缺少 Balances::TotalIssuance")?,
        "Balances::TotalIssuance",
    )?;
    let total_delta = signed_delta(total_before, total_after);
    let expected_total = i128::try_from(plan.total).unwrap_or(i128::MAX);
    if total_delta != expected_total {
        return Err(format!(
            "finalize 总发行差额错误:期望 {},实际 {total_delta}",
            plan.total
        ));
    }

    let account_prefix = fullnode_issuance::storage_key::system_account_prefix();
    let mut changed_accounts = BTreeMap::<[u8; 32], ()>::new();
    for key in pre_delta.keys().chain(post_delta.keys()) {
        if key.starts_with(&account_prefix) && pre(key) != post(key) {
            changed_accounts.insert(parse_system_account_key(key, &account_prefix)?, ());
        }
    }
    for account in changed_accounts.keys() {
        if !plan.accounts.contains_key(account) {
            return Err(format!(
                "finalize 出现未登记账户变化:0x{}",
                hex::encode(account)
            ));
        }
    }

    const NEW_BALANCES_FLAGS: u128 = 0x80000000_00000000_00000000_00000000;
    for (account, expected) in &plan.accounts {
        let before = account_info(pre, account)?;
        let after = account_info(post, account)?
            .ok_or_else(|| format!("finalize 收款账户缺失:0x{}", hex::encode(account)))?;
        let before_free = before
            .as_ref()
            .map(|info| info.data.free)
            .unwrap_or_default();
        let balance_delta = signed_delta(before_free, after.data.free);
        let expected_delta = i128::try_from(*expected).unwrap_or(i128::MAX);
        if balance_delta != expected_delta {
            return Err(format!(
                "finalize 收款差额错误:账户 0x{},期望 {expected},实际 {balance_delta}",
                hex::encode(account)
            ));
        }
        if let Some(before) = before {
            if before.nonce != after.nonce
                || before.consumers != after.consumers
                || before.providers != after.providers
                || before.sufficients != after.sufficients
                || before.data.reserved != after.data.reserved
                || before.data.frozen != after.data.frozen
                || before.data.flags != after.data.flags
            {
                return Err(format!(
                    "finalize 非 free 账户字段被改写:0x{}",
                    hex::encode(account)
                ));
            }
        } else if after.nonce != 0
            || after.consumers != 0
            || after.providers != 1
            || after.sufficients != 0
            || after.data.reserved != 0
            || after.data.frozen != 0
            || after.data.flags != NEW_BALANCES_FLAGS
        {
            return Err(format!(
                "finalize 新建收款账户形态错误:0x{}",
                hex::encode(account)
            ));
        }
    }
    Ok(())
}

impl<I> NodeGuard<I> {
    /// 装配节点守卫，并用 block#0 状态校验当前所有已注册永久策略的创世基准。
    pub fn new(
        inner: I,
        client: Arc<FullClient>,
        backend: Arc<FullBackend>,
    ) -> Result<Self, String> {
        let genesis_hash = client.info().genesis_hash;
        governance_skeleton::check_skeleton_invariants(|key| {
            client
                .storage(genesis_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        })
        .map_err(|e| format!("节点守卫:创世固定治理骨架校验失败:{e:?}"))?;
        fullnode_issuance::check_genesis(|key| {
            client
                .storage(genesis_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        })
        .map_err(|e| format!("节点守卫:创世全节点发行审计状态校验失败:{e:?}"))?;
        let mut provincialbank_state = Self::pallet_state(
            &client,
            genesis_hash,
            provincialbank_interest::storage_key::pallet_prefix(),
        )?;
        for key in provincialbank_interest::relevant_import_keys() {
            if key.starts_with(&provincialbank_interest::storage_key::pallet_prefix()) {
                continue;
            }
            if let Some(value) = client
                .storage(genesis_hash, &StorageKey(key.clone()))
                .map_err(|e| format!("读取创世省储行质押本金失败:{e}"))?
            {
                provincialbank_state.insert(key, value.0);
            }
        }
        provincialbank_interest::check_imported_genesis(provincialbank_state.iter())
            .map_err(|e| format!("节点守卫:创世省储行固定发行校验失败:{e:?}"))?;
        let citizen_issuance_state = Self::pallet_state(
            &client,
            genesis_hash,
            citizen_issuance::storage_key::pallet_prefix(),
        )?;
        citizen_issuance::check_genesis_key_values(citizen_issuance_state.iter())
            .map_err(|e| format!("节点守卫:创世公民认证发行状态校验失败:{e:?}"))?;
        let genesis_pallet_state = Self::pallet_state(
            &client,
            genesis_hash,
            genesis_pallet::storage_key::pallet_prefix(),
        )?;
        genesis_pallet::check_genesis(|key| genesis_pallet_state.get(key).cloned())
            .map_err(|e| format!("节点守卫:GenesisPallet 创世事实校验失败:{e:?}"))?;
        let pow_difficulty_state = Self::pallet_state(
            &client,
            genesis_hash,
            pow_difficulty::storage_key::pallet_prefix(),
        )?;
        pow_difficulty::check_genesis(|key| pow_difficulty_state.get(key).cloned())
            .map_err(|e| format!("节点守卫:PoW 动态难度创世状态校验失败:{e:?}"))?;
        let cid_keys = Self::cid_state_keys(&client, genesis_hash)?;
        let cid_lifecycle = cid_lifecycle::GenesisReference::from_genesis(&cid_keys, |key| {
            client
                .storage(genesis_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        })
        .map_err(|e| format!("节点守卫:创世 CID 生命周期基准校验失败:{e:?}"))?;

        Ok(Self {
            inner,
            client,
            backend,
            cid_lifecycle,
        })
    }

    /// 枚举节点原生 CID 策略承认的全部规范 RAW 表；只用于启动与 runtime 升级全检。
    fn cid_state_keys(
        client: &Arc<FullClient>,
        at: <Block as BlockT>::Hash,
    ) -> Result<Vec<Vec<u8>>, String> {
        let mut keys = BTreeMap::<Vec<u8>, ()>::new();
        for prefix in cid_lifecycle::storage_key::enumerated_prefixes() {
            let prefix = StorageKey(prefix);
            let iter = client
                .storage_keys(at, Some(&prefix), None)
                .map_err(|e| format!("枚举 CID 规范表失败:{e}"))?;
            for key in iter {
                keys.insert(key.0, ());
            }
        }
        Ok(keys.into_keys().collect())
    }

    /// 枚举指定 pallet 的全部 RAW key/value；只用于启动与 block#0 完整导入基准。
    fn pallet_state(
        client: &Arc<FullClient>,
        at: <Block as BlockT>::Hash,
        pallet_prefix: [u8; 16],
    ) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, String> {
        let prefix = StorageKey(pallet_prefix.to_vec());
        let keys = client
            .storage_keys(at, Some(&prefix), None)
            .map_err(|e| format!("枚举节点永久策略 pallet 失败:{e}"))?;
        let mut state = BTreeMap::new();
        for key in keys {
            let value = client
                .storage(at, &key)
                .map_err(|e| format!("读取节点永久策略 RAW 状态失败:{e}"))?
                .ok_or_else(|| "枚举到的节点永久策略 RAW key 缺少值".to_string())?;
            state.insert(key.0, value.0);
        }
        Ok(state)
    }

    /// 提交前校验 warp/状态导入携带的完整下载态；无法抽取或不满足策略时一律拒绝导入。
    fn verify_imported_state(&self, params: &BlockImportParams<Block>) -> Result<(), String> {
        let imported = match &params.state_action {
            StateAction::ApplyChanges(StorageChanges::Import(imported)) => imported,
            _ => return Err("warp 状态非 ApplyChanges(Import) 形态,无法提交前校验".into()),
        };
        // 所有策略复用同一遍 imported state 扫描，禁止为单项永久规则再新增包装器。
        let stats = verify_imported_policy_state(
            *params.header.number(),
            imported
                .state
                .0
                .iter()
                .flat_map(|level| level.key_values.iter())
                .map(|(key, value)| (key, value)),
            &self.cid_lifecycle,
        )?;
        log::debug!(
            target: "node-guard",
            "完整状态单遍扫描:总键 {},治理 {},全节点发行/账户 {},公民发行 {},创世模块 {},PoW 动态难度 {},省储行固定发行 {},CID {}",
            stats.scanned,
            stats.governance,
            stats.fullnode_issuance,
            stats.citizen_issuance,
            stats.genesis_pallet,
            stats.pow_difficulty,
            stats.provincialbank_interest,
            stats.cid,
        );
        Ok(())
    }

    /// 对正常执行型区块统一预执行一次，并检查当前已注册的全部节点永久策略。
    fn detect_violation(&self, params: &BlockImportParams<Block>) -> Result<bool, String> {
        let body = params
            .body
            .clone()
            .ok_or_else(|| "普通区块缺少 body,无法复算 finalize 前后发行状态".to_string())?;
        let extrinsic_count = body.len();
        if !has_user_transaction(extrinsic_count) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):空块不允许上链",
                params.header.number(),
                params.post_hash(),
            );
            return Ok(true);
        }

        let parent_hash = *params.header.parent_hash();
        let parent_state = self
            .backend
            .state_at(parent_hash, TrieCacheContext::Untrusted)
            .map_err(|e| format!("取父状态失败:{e}"))?;

        // 第一阶段只执行 initialize + extrinsics，不运行 finalize，用来隔离 runtime on_finalize 的净变化。
        let pre_api = self.client.runtime_api();
        pre_api
            .initialize_block(parent_hash, &params.header)
            .map_err(|e| format!("初始化区块预执行失败:{e}"))?;
        for extrinsic in body.iter().cloned() {
            match pre_api.apply_extrinsic(parent_hash, extrinsic) {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => return Err(format!("预执行交易有效性失败:{e:?}")),
                Err(e) => return Err(format!("预执行交易调用失败:{e}")),
            }
        }
        let pre_changes = pre_api
            .into_storage_changes(&parent_state, parent_hash)
            .map_err(|e| format!("提取 finalize 前存储变更失败:{e}"))?;
        let pre_delta: BTreeMap<Vec<u8>, Option<Vec<u8>>> =
            pre_changes.main_storage_changes.into_iter().collect();

        // 第二阶段完整执行同一区块，得到 finalize 后状态；两阶段都只在 overlay 中执行，不提交后端。
        let block = Block::new(params.header.clone(), body);
        let api = self.client.runtime_api();
        api.execute_block(parent_hash, block.into())
            .map_err(|e| format!("只读执行区块失败:{e}"))?;
        let changes = api
            .into_storage_changes(&parent_state, parent_hash)
            .map_err(|e| format!("提取存储变更失败:{e}"))?;
        let post_delta: BTreeMap<Vec<u8>, Option<Vec<u8>>> =
            changes.main_storage_changes.into_iter().collect();

        let read_parent = |key: &[u8]| -> Option<Vec<u8>> {
            self.client
                .storage(parent_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        };
        let read_pre = |key: &[u8]| -> Option<Vec<u8>> {
            match pre_delta.get(key) {
                Some(value) => value.clone(),
                None => read_parent(key),
            }
        };
        let read_post = |key: &[u8]| -> Option<Vec<u8>> {
            match post_delta.get(key) {
                Some(value) => value.clone(),
                None => read_parent(key),
            }
        };

        let mut issuance_plan = FinalizeIssuancePlan::default();
        if let Err(reason) = pow_difficulty::check_transition(
            *params.header.number(),
            &post_delta,
            &read_parent,
            &read_post,
        ) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):PoW 动态难度永久规则被破坏 —— {:?}",
                params.header.number(),
                params.post_hash(),
                reason,
            );
            return Ok(true);
        }

        if let Err(reason) = genesis_pallet::check_transition(&post_delta, &read_parent, &read_post)
        {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):创世模块永久规则被破坏 —— {:?}",
                params.header.number(),
                params.post_hash(),
                reason,
            );
            return Ok(true);
        }

        if let Err(reason) = fullnode_issuance::check_transition(
            *params.header.number(),
            fullnode_issuance::author_from_header(&params.header),
            &read_parent,
            &read_pre,
            &read_post,
            &mut issuance_plan,
        ) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):全节点发行永久规则被破坏 —— {:?}",
                params.header.number(),
                params.post_hash(),
                reason,
            );
            return Ok(true);
        }

        if let Err(reason) = citizen_issuance::check_transition(
            extrinsic_count,
            &pre_delta,
            &post_delta,
            &read_parent,
            &read_pre,
            &read_post,
            &mut issuance_plan,
        ) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):公民认证发行永久规则被破坏 —— {:?}",
                params.header.number(),
                params.post_hash(),
                reason,
            );
            return Ok(true);
        }

        if let Err(reason) = provincialbank_interest::check_transition(
            *params.header.number(),
            &pre_delta,
            &post_delta,
            &read_parent,
            &read_pre,
            &read_post,
            &mut issuance_plan,
        ) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):省储行固定发行永久规则被破坏 —— {:?}",
                params.header.number(),
                params.post_hash(),
                reason,
            );
            return Ok(true);
        }

        if let Err(reason) = verify_finalize_issuance(
            &pre_delta,
            &post_delta,
            &read_pre,
            &read_post,
            &issuance_plan,
        ) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):finalize 统一发行核算失败 —— {reason}",
                params.header.number(),
                params.post_hash(),
            );
            return Ok(true);
        }

        if let Err(reason) = cid_lifecycle::check_transition(
            *params.header.number(),
            &post_delta,
            &read_parent,
            &read_post,
            &self.cid_lifecycle,
        ) {
            log::error!(
                target: "node-guard",
                "拒绝区块 #{} ({:?}):CID 生命周期永久规则被破坏 —— {:?}",
                params.header.number(),
                params.post_hash(),
                reason,
            );
            return Ok(true);
        }

        if cid_lifecycle::needs_full_check(&post_delta) {
            let mut keys: BTreeMap<Vec<u8>, ()> = Self::cid_state_keys(&self.client, parent_hash)?
                .into_iter()
                .map(|key| (key, ()))
                .collect();
            for key in post_delta
                .keys()
                .filter(|key| cid_lifecycle::is_relevant_key(key))
            {
                keys.insert(key.clone(), ());
            }
            let keys: Vec<Vec<u8>> = keys.into_keys().collect();
            if let Err(reason) =
                cid_lifecycle::check_full_state(&keys, &read_post, &self.cid_lifecycle)
            {
                log::error!(
                    target: "node-guard",
                    "拒绝区块 #{} ({:?}):runtime 升级后的 CID 规范表全检失败 —— {:?}",
                    params.header.number(),
                    params.post_hash(),
                    reason,
                );
                return Ok(true);
            }
        }

        if post_delta.contains_key(sp_storage::well_known_keys::CODE) {
            if let Err(reason) = genesis_pallet::check_full_state(&read_post) {
                log::error!(
                    target: "node-guard",
                    "拒绝区块 #{} ({:?}):runtime 升级后的创世模块全检失败 —— {:?}",
                    params.header.number(),
                    params.post_hash(),
                    reason,
                );
                return Ok(true);
            }
            if let Err(reason) =
                provincialbank_interest::check_full_state(*params.header.number(), &read_post)
            {
                log::error!(
                    target: "node-guard",
                    "拒绝区块 #{} ({:?}):runtime 升级后的省储行固定发行全检失败 —— {:?}",
                    params.header.number(),
                    params.post_hash(),
                    reason,
                );
                return Ok(true);
            }
        }

        // 治理骨架只在相关 storage 或 `:code` 变化时全量复核，避免每块重复解码全部管理员集合。
        if governance_skeleton::needs_full_check(&post_delta) {
            if let Err(reason) = governance_skeleton::check_skeleton_invariants(read_post) {
                log::error!(
                    target: "node-guard",
                    "拒绝区块 #{} ({:?}):固定治理骨架不变式被破坏 —— {:?}",
                    params.header.number(),
                    params.post_hash(),
                    reason,
                );
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[async_trait::async_trait]
impl<I> BlockImport<Block> for NodeGuard<I>
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
        if params.with_state() {
            let verdict = self.verify_imported_state(&params);
            if let Err(reason) = &verdict {
                log::error!(
                    target: "node-guard",
                    "拒绝 warp/状态导入 ({:?}):节点永久规则校验未通过 —— {reason}",
                    params.post_hash(),
                );
            }
            return import_if_verified(&self.inner, params, verdict).await;
        }

        let verdict = match self.detect_violation(&params) {
            Ok(true) => Err("节点永久规则明确判定为违规".to_string()),
            Ok(false) => Ok(()),
            // 沿用现有 fail-closed 口径：无法完成节点规则检查时不导入未经验证的区块。
            Err(reason) => {
                log::error!(
                    target: "node-guard",
                    "节点守卫判定失败,fail-closed 拒块 ({:?}):{reason}",
                    params.post_hash(),
                );
                Err(reason)
            }
        };
        import_if_verified(&self.inner, params, verdict).await
    }
}

#[cfg(test)]
mod finalize_issuance_tests {
    use super::*;

    fn full_runtime_genesis_storage() -> sp_runtime::Storage {
        use sp_runtime::BuildStorage;

        let config: citizenchain::RuntimeGenesisConfig =
            serde_json::from_value(citizenchain::genesis::genesis_config())
                .expect("decode complete runtime genesis config");
        config
            .build_storage()
            .expect("build complete runtime genesis storage")
    }
    use codec::Encode;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn block_requires_timestamp_and_at_least_one_user_transaction() {
        assert!(!has_user_transaction(0));
        assert!(!has_user_transaction(1));
        assert!(has_user_transaction(2));
    }

    #[derive(Default)]
    struct CountingImport {
        imports: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl BlockImport<Block> for CountingImport {
        type Error = ConsensusError;

        async fn check_block(
            &self,
            _block: BlockCheckParams<Block>,
        ) -> Result<ImportResult, Self::Error> {
            Ok(ImportResult::AlreadyInChain)
        }

        async fn import_block(
            &self,
            _block: BlockImportParams<Block>,
        ) -> Result<ImportResult, Self::Error> {
            self.imports.fetch_add(1, Ordering::SeqCst);
            Ok(ImportResult::AlreadyInChain)
        }
    }

    fn import_params(number: u32) -> BlockImportParams<Block> {
        use sp_consensus::BlockOrigin;
        use sp_core::H256;
        use sp_runtime::Digest;

        let header = citizenchain::opaque::Header::new(
            number,
            H256::repeat_byte(1),
            H256::repeat_byte(2),
            H256::repeat_byte(3),
            Digest::default(),
        );
        BlockImportParams::new(BlockOrigin::NetworkInitialSync, header)
    }

    const NEW_BALANCES_FLAGS: u128 = 0x80000000_00000000_00000000_00000000;

    fn account(free: u128, providers: u32, flags: u128) -> Vec<u8> {
        (0u32, 0u32, providers, 0u32, (free, 0u128, 0u128, flags)).encode()
    }

    #[test]
    fn combined_rewards_for_same_account_are_checked_once() {
        let recipient = [9u8; 32];
        let total_key = fullnode_issuance::storage_key::total_issuance();
        let account_key = fullnode_issuance::storage_key::system_account(&recipient);
        let mut pre = BTreeMap::new();
        let mut post = BTreeMap::new();
        pre.insert(total_key.clone(), 1_000_000u128.encode());
        post.insert(total_key.clone(), 2_999_800u128.encode());
        pre.insert(account_key.clone(), account(100, 1, NEW_BALANCES_FLAGS));
        post.insert(
            account_key.clone(),
            account(1_999_900, 1, NEW_BALANCES_FLAGS),
        );
        let pre_delta = BTreeMap::new();
        let post_delta = BTreeMap::from([
            (total_key, Some(2_999_800u128.encode())),
            (account_key, Some(account(1_999_900, 1, NEW_BALANCES_FLAGS))),
        ]);
        let mut plan = FinalizeIssuancePlan::default();
        plan.add(recipient, 999_900).expect("first reward");
        plan.add(recipient, 999_900).expect("second reward");
        assert_eq!(
            verify_finalize_issuance(
                &pre_delta,
                &post_delta,
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &plan,
            ),
            Ok(())
        );
    }

    #[test]
    fn unexpected_finalize_account_change_is_rejected() {
        let recipient = [1u8; 32];
        let attacker = [2u8; 32];
        let total_key = fullnode_issuance::storage_key::total_issuance();
        let recipient_key = fullnode_issuance::storage_key::system_account(&recipient);
        let attacker_key = fullnode_issuance::storage_key::system_account(&attacker);
        let mut pre = BTreeMap::new();
        let mut post = BTreeMap::new();
        pre.insert(total_key.clone(), 1_000u128.encode());
        post.insert(total_key.clone(), 1_100u128.encode());
        pre.insert(recipient_key.clone(), account(0, 1, NEW_BALANCES_FLAGS));
        post.insert(recipient_key.clone(), account(100, 1, NEW_BALANCES_FLAGS));
        pre.insert(attacker_key.clone(), account(50, 1, NEW_BALANCES_FLAGS));
        post.insert(attacker_key.clone(), account(51, 1, NEW_BALANCES_FLAGS));
        let post_delta = BTreeMap::from([
            (total_key, Some(1_100u128.encode())),
            (recipient_key, Some(account(100, 1, NEW_BALANCES_FLAGS))),
            (attacker_key, Some(account(51, 1, NEW_BALANCES_FLAGS))),
        ]);
        let mut plan = FinalizeIssuancePlan::default();
        plan.add(recipient, 100).expect("reward");
        let error = verify_finalize_issuance(
            &BTreeMap::new(),
            &post_delta,
            &|key| pre.get(key).cloned(),
            &|key| post.get(key).cloned(),
            &plan,
        )
        .expect_err("attacker change must fail");
        assert!(error.contains("未登记账户变化"));
    }

    #[test]
    fn new_reward_account_must_match_balances_default_shape() {
        let recipient = [3u8; 32];
        let total_key = fullnode_issuance::storage_key::total_issuance();
        let account_key = fullnode_issuance::storage_key::system_account(&recipient);
        let pre = BTreeMap::from([(total_key.clone(), 1_000u128.encode())]);
        let mut post = BTreeMap::from([(total_key.clone(), 1_100u128.encode())]);
        post.insert(account_key.clone(), account(100, 1, NEW_BALANCES_FLAGS));
        let post_delta = BTreeMap::from([
            (total_key, Some(1_100u128.encode())),
            (account_key, Some(account(100, 1, NEW_BALANCES_FLAGS))),
        ]);
        let mut plan = FinalizeIssuancePlan::default();
        plan.add(recipient, 100).expect("reward");
        assert_eq!(
            verify_finalize_issuance(
                &BTreeMap::new(),
                &post_delta,
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &plan,
            ),
            Ok(())
        );
    }

    #[test]
    fn guarded_import_delegates_only_after_explicit_success() {
        let inner = CountingImport::default();
        let accepted =
            futures::executor::block_on(import_if_verified(&inner, import_params(1), Ok(())))
                .expect("verified import result");
        assert_eq!(accepted, ImportResult::AlreadyInChain);
        assert_eq!(inner.imports.load(Ordering::SeqCst), 1);

        let rejected = futures::executor::block_on(import_if_verified(
            &inner,
            import_params(2),
            Err("malicious state".into()),
        ))
        .expect("known bad result");
        assert_eq!(rejected, ImportResult::KnownBad);
        assert_eq!(inner.imports.load(Ordering::SeqCst), 1);

        let rejected_again = futures::executor::block_on(import_if_verified(
            &inner,
            import_params(3),
            Err("another malicious state".into()),
        ))
        .expect("second known bad result");
        assert_eq!(rejected_again, ImportResult::KnownBad);
        assert_eq!(inner.imports.load(Ordering::SeqCst), 1);

        // 连续拒绝不保存污染状态；随后合法区块仍只委派一次。
        let accepted_after_rejection =
            futures::executor::block_on(import_if_verified(&inner, import_params(4), Ok(())))
                .expect("verified import after rejection");
        assert_eq!(accepted_after_rejection, ImportResult::AlreadyInChain);
        assert_eq!(inner.imports.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn finalize_plan_rejects_overflow_wrong_total_and_non_free_mutation() {
        let mut overflow = FinalizeIssuancePlan::default();
        overflow.add([1u8; 32], u128::MAX).expect("first amount");
        assert_eq!(overflow.add([2u8; 32], 1), Err(()));

        let recipient = [4u8; 32];
        let total_key = fullnode_issuance::storage_key::total_issuance();
        let account_key = fullnode_issuance::storage_key::system_account(&recipient);
        let pre = BTreeMap::from([
            (total_key.clone(), 1_000u128.encode()),
            (account_key.clone(), account(10, 1, NEW_BALANCES_FLAGS)),
        ]);
        let mut post = BTreeMap::from([
            (total_key.clone(), 1_099u128.encode()),
            (account_key.clone(), account(110, 1, NEW_BALANCES_FLAGS)),
        ]);
        let mut post_delta = post
            .iter()
            .map(|(key, value)| (key.clone(), Some(value.clone())))
            .collect::<BTreeMap<_, _>>();
        let mut plan = FinalizeIssuancePlan::default();
        plan.add(recipient, 100).expect("reward");
        let wrong_total = verify_finalize_issuance(
            &BTreeMap::new(),
            &post_delta,
            &|key| pre.get(key).cloned(),
            &|key| post.get(key).cloned(),
            &plan,
        )
        .expect_err("wrong total must fail");
        assert!(wrong_total.contains("总发行差额错误"));

        post.insert(total_key.clone(), 1_100u128.encode());
        let mutated = (
            1u32,
            0u32,
            1u32,
            0u32,
            (110u128, 0u128, 0u128, NEW_BALANCES_FLAGS),
        )
            .encode();
        post.insert(account_key.clone(), mutated.clone());
        post_delta.insert(total_key, Some(1_100u128.encode()));
        post_delta.insert(account_key, Some(mutated));
        let non_free = verify_finalize_issuance(
            &BTreeMap::new(),
            &post_delta,
            &|key| pre.get(key).cloned(),
            &|key| post.get(key).cloned(),
            &plan,
        )
        .expect_err("nonce mutation must fail");
        assert!(non_free.contains("非 free 账户字段被改写"));
    }

    #[test]
    fn imported_state_is_partitioned_in_one_shared_pass() {
        let governance = governance_skeleton::storage_key::admin_account(&[1u8; 32]);
        let fullnode = fullnode_issuance::storage_key::rewarded_block_count();
        let citizen = citizen_issuance::storage_key::rewarded_count();
        let genesis = genesis_pallet::storage_key::citizen_max();
        let provincialbank = provincialbank_interest::storage_key::last_settled_year();
        let cid = cid_lifecycle::storage_key::citizen_registry_prefix();
        let unrelated = b"unrelated".to_vec();
        let pairs = BTreeMap::from([
            (governance.clone(), vec![1]),
            (fullnode.clone(), vec![2]),
            (citizen.clone(), vec![3]),
            (genesis.clone(), vec![4]),
            (provincialbank.clone(), vec![4]),
            (cid.clone(), vec![4]),
            (unrelated, vec![5]),
        ]);
        let state = partition_imported_state(pairs.iter());
        assert_eq!(state.scanned, 7);
        assert_eq!(
            state.governance.keys().cloned().collect::<Vec<_>>(),
            [governance]
        );
        assert_eq!(
            state.fullnode_issuance.keys().cloned().collect::<Vec<_>>(),
            [fullnode]
        );
        assert_eq!(
            state.citizen_issuance.keys().cloned().collect::<Vec<_>>(),
            [citizen]
        );
        assert_eq!(
            state.genesis_pallet.keys().cloned().collect::<Vec<_>>(),
            [genesis]
        );
        assert_eq!(
            state
                .provincialbank_interest
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            [provincialbank]
        );
        assert_eq!(state.cid.keys().cloned().collect::<Vec<_>>(), [cid]);
    }

    #[test]
    fn real_genesis_complete_state_passes_all_policies_in_one_scan() {
        let storage = full_runtime_genesis_storage();
        let top = storage.top;
        let cid_keys: Vec<Vec<u8>> = top
            .keys()
            .filter(|key| cid_lifecycle::is_relevant_key(key))
            .cloned()
            .collect();
        let reference =
            cid_lifecycle::GenesisReference::from_genesis(&cid_keys, |key| top.get(key).cloned())
                .expect("build CID genesis reference");
        let stats = verify_imported_policy_state(0, top.iter(), &reference)
            .expect("real block zero state must pass every policy");
        assert_eq!(stats.scanned, top.len());
        assert!(stats.governance > 0);
        assert!(stats.fullnode_issuance > 0);
        assert!(stats.genesis_pallet > 0);
        assert!(stats.provincialbank_interest > 0);
        assert!(stats.cid > 0);
    }

    #[test]
    fn complete_state_rejects_each_policy_before_inner_import() {
        use codec::Encode;

        let storage = full_runtime_genesis_storage();
        let top = storage.top;
        let cid_keys: Vec<Vec<u8>> = top
            .keys()
            .filter(|key| cid_lifecycle::is_relevant_key(key))
            .cloned()
            .collect();
        let reference =
            cid_lifecycle::GenesisReference::from_genesis(&cid_keys, |key| top.get(key).cloned())
                .expect("build CID genesis reference");

        let fixed = primitives::governance_skeleton::fixed_institutions()[0];
        let mut missing_governance = top.clone();
        missing_governance.remove(&governance_skeleton::storage_key::admin_account(
            &fixed.main_account,
        ));
        assert!(
            verify_imported_policy_state(0, missing_governance.iter(), &reference)
                .expect_err("missing governance must fail")
                .starts_with("固定治理骨架:")
        );

        let mut bad_fullnode = top.clone();
        bad_fullnode.insert(
            fullnode_issuance::storage_key::rewarded_block_count(),
            1u32.encode(),
        );
        assert!(
            verify_imported_policy_state(0, bad_fullnode.iter(), &reference)
                .expect_err("non-zero genesis issuance must fail")
                .starts_with("全节点发行:")
        );

        let mut bad_citizen = top.clone();
        let mut unknown = citizen_issuance::storage_key::pallet_prefix().to_vec();
        unknown.extend_from_slice(b"UnknownGuardState");
        bad_citizen.insert(unknown, vec![1]);
        assert!(
            verify_imported_policy_state(0, bad_citizen.iter(), &reference)
                .expect_err("unknown citizen issuance key must fail")
                .starts_with("公民认证发行:")
        );

        let mut bad_genesis = top.clone();
        bad_genesis.insert(
            genesis_pallet::storage_key::citizen_max(),
            1_443_497_379u64.encode(),
        );
        assert!(
            verify_imported_policy_state(0, bad_genesis.iter(), &reference)
                .expect_err("changed genesis population must fail")
                .starts_with("创世模块:")
        );

        let first_bank = &primitives::cid::china::china_ch::CHINA_CH[0];
        let mut bad_provincialbank = top.clone();
        bad_provincialbank.remove(&provincialbank_interest::storage_key::system_account(
            &first_bank.stake_account,
        ));
        assert!(
            verify_imported_policy_state(0, bad_provincialbank.iter(), &reference)
                .expect_err("missing provincial bank principal must fail")
                .starts_with("省储行固定发行:")
        );

        let mut unknown_provincialbank = top.clone();
        let mut unknown_interest_key =
            provincialbank_interest::storage_key::pallet_prefix().to_vec();
        unknown_interest_key.extend_from_slice(&sp_core::hashing::twox_128(b"ShadowInterest"));
        unknown_provincialbank.insert(unknown_interest_key, 1u32.encode());
        assert!(
            verify_imported_policy_state(0, unknown_provincialbank.iter(), &reference)
                .expect_err("unknown provincial bank storage must fail")
                .starts_with("省储行固定发行:")
        );

        let mut bad_cid = top.clone();
        let protected_prefix = cid_lifecycle::storage_key::protected_prefix();
        let protected_key = bad_cid
            .keys()
            .find(|key| key.starts_with(&protected_prefix))
            .cloned()
            .expect("protected genesis key");
        bad_cid.remove(&protected_key);
        assert!(verify_imported_policy_state(0, bad_cid.iter(), &reference)
            .expect_err("missing protected account must fail")
            .starts_with("CID 生命周期:"));
    }

    #[test]
    fn non_genesis_complete_state_is_always_rejected_by_cid_policy() {
        let empty = BTreeMap::<Vec<u8>, Vec<u8>>::new();
        assert!(verify_imported_policy_state(
            1,
            empty.iter(),
            &cid_lifecycle::GenesisReference::default(),
        )
        .expect_err("non-genesis snapshot cannot prove CID monotonicity")
        .contains("NonGenesisStateImportForbidden"));
    }
}
