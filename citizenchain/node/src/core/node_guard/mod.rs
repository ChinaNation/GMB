//! 节点守卫统一 `BlockImport` 包装器。
//!
//! 公民宪法是整条链最高规则，继续由独立的 `ConstitutionGuard` 在本包装器外层先行检查。
//! 本模块只收口**除宪法外**的节点永久规则：统一预执行正常区块、统一提取后置 storage delta，
//! 再把同一份检查上下文交给内部策略。当前已注册固定治理骨架、全节点 PoW 发行、公民认证发行
//! 与 CID 生命周期；后续非宪法永久规则仍必须加在本包装器内部，不得再新增平行 `BlockImport` 包装器。

mod cid_lifecycle;
mod citizen_issuance;
mod fullnode_issuance;
mod governance_skeleton;

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

/// 节点镜像的 `frame_system::AccountInfo<u32, pallet_balances::AccountData<u128>>`。
#[derive(Debug, Decode, Eq, PartialEq)]
struct NativeAccountInfo {
    nonce: u32,
    consumers: u32,
    providers: u32,
    sufficients: u32,
    data: NativeAccountData,
}

#[derive(Debug, Decode, Eq, PartialEq)]
struct NativeAccountData {
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

fn account_info<F>(read: &F, account: &[u8; 32]) -> Result<Option<NativeAccountInfo>, String>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    read(&fullnode_issuance::storage_key::system_account(account))
        .map(|raw| decode_exact(&raw, "System::Account"))
        .transpose()
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
        let citizen_issuance_state = Self::pallet_state(
            &client,
            genesis_hash,
            citizen_issuance::storage_key::pallet_prefix(),
        )?;
        citizen_issuance::check_genesis_key_values(citizen_issuance_state.iter())
            .map_err(|e| format!("节点守卫:创世公民认证发行状态校验失败:{e:?}"))?;
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
        cid_lifecycle::check_state_import_height(*params.header.number())
            .map_err(|e| format!("CID 生命周期:{e:?}"))?;

        // 所有策略复用同一遍 imported state 扫描，禁止为单项永久规则再新增包装器。
        let governance_prefix = governance_skeleton::storage_key::pallet_prefix();
        let fullnode_prefix = fullnode_issuance::storage_key::pallet_prefix();
        let system_prefix = fullnode_issuance::storage_key::system_prefix();
        let total_issuance_key = fullnode_issuance::storage_key::total_issuance();
        let citizen_issuance_prefix = citizen_issuance::storage_key::pallet_prefix();
        let cid_prefixes = cid_lifecycle::storage_key::relevant_prefixes();
        let mut governance_state: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let mut issuance_state: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let mut cid_state: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let mut citizen_issuance_state: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        for (key, value) in imported
            .state
            .0
            .iter()
            .flat_map(|level| level.key_values.iter())
        {
            if key.starts_with(&governance_prefix) {
                governance_state.insert(key.clone(), value.clone());
            }
            if key.starts_with(&fullnode_prefix)
                || key.starts_with(&system_prefix)
                || key == &total_issuance_key
            {
                issuance_state.insert(key.clone(), value.clone());
            }
            if cid_lifecycle::matches_relevant_prefixes(key, &cid_prefixes) {
                cid_state.insert(key.clone(), value.clone());
            }
            if key.starts_with(&citizen_issuance_prefix) {
                citizen_issuance_state.insert(key.clone(), value.clone());
            }
        }
        governance_skeleton::check_skeleton_invariants(|key| governance_state.get(key).cloned())
            .map_err(|e| format!("固定治理骨架:{e:?}"))?;
        fullnode_issuance::check_imported_state_key_values(
            *params.header.number(),
            issuance_state.iter(),
        )
        .map_err(|e| format!("全节点发行:{e:?}"))?;
        citizen_issuance::check_genesis_key_values(citizen_issuance_state.iter())
            .map_err(|e| format!("公民认证发行:{e:?}"))?;
        cid_lifecycle::check_imported_genesis(cid_state.iter(), &self.cid_lifecycle)
            .map_err(|e| format!("CID 生命周期:{e:?}"))
    }

    /// 对正常执行型区块统一预执行一次，并检查当前已注册的全部节点永久策略。
    fn detect_violation(&self, params: &BlockImportParams<Block>) -> Result<bool, String> {
        let body = params
            .body
            .clone()
            .ok_or_else(|| "普通区块缺少 body,无法复算 finalize 前后发行状态".to_string())?;
        let extrinsic_count = body.len();

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
            return match self.verify_imported_state(&params) {
                Ok(()) => self.inner.import_block(params).await,
                Err(reason) => {
                    log::error!(
                        target: "node-guard",
                        "拒绝 warp/状态导入 ({:?}):节点永久规则校验未通过 —— {reason}",
                        params.post_hash(),
                    );
                    Ok(ImportResult::KnownBad)
                }
            };
        }

        match self.detect_violation(&params) {
            Ok(true) => Ok(ImportResult::KnownBad),
            Ok(false) => self.inner.import_block(params).await,
            // 沿用现有 fail-closed 口径：无法完成节点规则检查时不导入未经验证的区块。
            Err(reason) => {
                log::error!(
                    target: "node-guard",
                    "节点守卫判定失败,fail-closed 拒块 ({:?}):{reason}",
                    params.post_hash(),
                );
                Ok(ImportResult::KnownBad)
            }
        }
    }
}

#[cfg(test)]
mod finalize_issuance_tests {
    use super::*;
    use codec::Encode;

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
}
