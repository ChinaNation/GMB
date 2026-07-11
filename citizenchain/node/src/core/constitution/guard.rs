//! 公民宪法独立 `BlockImport` 包装器。
//!
//! 本包装器始终位于 `NodeGuard` 外层，先执行整条链最高优先级的宪法检查，再委派给其他
//! 节点永久规则和 PoW 导入。它只编排导入形态、只读执行和 fail-closed；纯不变式留在父模块。

use super::*;

/// 在区块进入规范链之前校验宪法不可修改条款和结构不变式。
pub struct ConstitutionGuard<I> {
    inner: I,
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    reference: ImmutableReference,
}

impl<I> ConstitutionGuard<I> {
    /// 从 block#0 派生不可修改基准并执行完整创世校验；任何异常都拒绝启动。
    pub fn new(
        inner: I,
        client: Arc<FullClient>,
        backend: Arc<FullBackend>,
    ) -> Result<Self, String> {
        let genesis_hash = client.info().genesis_hash;
        let read_genesis = |key: &[u8]| {
            client
                .storage(genesis_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        };
        let reference = ImmutableReference::from_raw_reader(&read_genesis)
            .map_err(|e| format!("护宪守卫:创世不可修改条款基准派生失败:{e:?}"))?;
        verify_manifest_from_reader(&read_genesis, &reference)
            .map_err(|e| format!("护宪守卫:启动 manifest 交叉校验失败:{e}"))?;
        check_immutable_articles(&read_genesis, &reference)
            .map_err(|e| format!("护宪守卫:创世完整不变式校验失败:{e:?}"))?;
        let law_bytes = read_genesis(&storage_key::law(CONSTITUTION_LAW_ID))
            .ok_or_else(|| "护宪守卫:创世缺宪法 Law(0)".to_string())?;
        let law =
            decode_law_head(&law_bytes).map_err(|e| format!("护宪守卫:创世 Law 解码失败:{e:?}"))?;
        let versions_prefix = StorageKey(storage_key::constitution_versions_prefix());
        let version_keys: Vec<Vec<u8>> = client
            .storage_keys(genesis_hash, Some(&versions_prefix), None)
            .map_err(|e| format!("护宪守卫:枚举创世宪法版本失败:{e}"))?
            .map(|key| key.0)
            .collect();
        let versions = declared_constitution_versions(version_keys.iter(), law.latest_version)
            .map_err(|e| format!("护宪守卫:创世宪法版本集合非法:{e:?}"))?;
        for version in versions {
            check_immutable_version(&read_genesis, &reference, version)
                .map_err(|e| format!("护宪守卫:创世历史版本 {version} 非法:{e:?}"))?;
        }

        Ok(Self {
            inner,
            client,
            backend,
            reference,
        })
    }

    /// 提交前校验 warp/状态导入携带的完整下载态。
    fn verify_imported_state(&self, params: &BlockImportParams<Block>) -> Result<(), String> {
        let imported = match &params.state_action {
            StateAction::ApplyChanges(StorageChanges::Import(imported)) => imported,
            _ => return Err("warp 状态非 ApplyChanges(Import) 形态,无法提交前校验".into()),
        };
        check_imported_state_immutable(imported, &self.reference)
    }

    /// 读取父状态并把 delta 覆盖为目标后置状态，再执行全部宪法不变式。
    fn check_delta(
        &self,
        parent_hash: <Block as BlockT>::Hash,
        delta: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    ) -> Result<bool, String> {
        if !needs_full_invariant_check(&delta) {
            return Ok(false);
        }
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
        let law_bytes = read_post(&storage_key::law(CONSTITUTION_LAW_ID))
            .ok_or_else(|| "目标状态缺宪法 Law(0)".to_string())?;
        let law = decode_law_head(&law_bytes).map_err(|e| format!("宪法 Law 解码失败:{e:?}"))?;
        check_version_key_range(
            delta
                .iter()
                .filter_map(|(key, value)| value.as_ref().map(|_| key)),
            law.latest_version,
        )
        .map_err(|e| format!("宪法版本键范围非法:{e:?}"))?;
        let parent_law_bytes = self
            .client
            .storage(
                parent_hash,
                &StorageKey(storage_key::law(CONSTITUTION_LAW_ID)),
            )
            .map_err(|e| format!("读取父状态宪法 Law 失败:{e}"))?
            .ok_or_else(|| "父状态缺宪法 Law(0)".to_string())?;
        let parent_law = decode_law_head(&parent_law_bytes.0)
            .map_err(|e| format!("父状态宪法 Law 解码失败:{e:?}"))?;
        if law.latest_version < parent_law.latest_version {
            return Err(format!(
                "宪法 latest_version 从 {} 回退到 {}",
                parent_law.latest_version, law.latest_version
            ));
        }
        check_immutable_articles(&read_post, &self.reference)
            .map(|()| false)
            .map_err(|reason| format!("宪法不变式被破坏:{reason:?}"))?;
        // 任一历史版本 RAW key 被新增、修改或删除时，都按目标后置状态单独复核该版本。
        for key in delta.keys() {
            if let Some(version) = storage_key::constitution_version_from_key(key)
                .or_else(|| storage_key::constitution_proof_version_from_key(key))
            {
                check_immutable_version(&read_post, &self.reference, version)
                    .map_err(|reason| format!("宪法历史版本 {version} 非法:{reason:?}"))?;
            }
        }
        Ok(false)
    }

    /// 对普通导入形态取得可验证的后置 storage delta；无法证明时 fail-closed。
    fn detect_violation(&self, params: &BlockImportParams<Block>) -> Result<bool, String> {
        let parent_hash = *params.header.parent_hash();
        if let Some(body) = &params.body {
            let block = Block::new(params.header.clone(), body.clone());
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
            let delta = changes.main_storage_changes.into_iter().collect();
            return self.check_delta(parent_hash, delta);
        }

        match &params.state_action {
            // 预计算状态变化即使没有 body 也必须经过宪法校验，不能走旧快路径绕过。
            StateAction::ApplyChanges(StorageChanges::Changes(changes)) => {
                let delta = changes.main_storage_changes.iter().cloned().collect();
                self.check_delta(parent_hash, delta)
            }
            // Skip 明确定义为不执行且不导入状态，因此不可能在本次导入中改写宪法状态。
            StateAction::Skip => Ok(false),
            StateAction::Execute | StateAction::ExecuteIfPossible => {
                Err("执行型区块缺少 body,无法独立证明宪法后置状态".into())
            }
            StateAction::ApplyChanges(StorageChanges::Import(_)) => {
                Err("完整状态导入必须走 verify_imported_state".into())
            }
        }
    }
}

#[async_trait::async_trait]
impl<I> BlockImport<Block> for ConstitutionGuard<I>
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
                    target: "constitution-guard",
                    "拒绝 warp/状态导入 ({:?}):宪法不变式校验未通过 —— {reason}",
                    params.post_hash(),
                );
            }
            return crate::core::node_guard::import_if_verified(&self.inner, params, verdict).await;
        }

        let verdict = match self.detect_violation(&params) {
            Ok(true) => Err("宪法永久规则明确判定为违规".to_string()),
            Ok(false) => Ok(()),
            Err(reason) => {
                log::error!(
                    target: "constitution-guard",
                    "守卫判定失败,fail-closed 拒块 ({:?}):{reason}",
                    params.post_hash(),
                );
                Err(reason)
            }
        };
        crate::core::node_guard::import_if_verified(&self.inner, params, verdict).await
    }
}
