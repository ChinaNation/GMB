# 任务卡：投票引擎统一入口整改 · Phase 2

- 状态：done(2026-04-22),等 Phase 3 客户端改造完工后整 Phase 1+2+3 一起移 done
- 归属：Blockchain Agent(5 个业务 pallet + primitives/core_const + runtime configs)
- 承接：`20260422-unified-voting-entry-phase1.md`

## Phase 2 目标

把 Phase 1 已打开的投票引擎公开 `internal_vote` call 对接到全部 5 个业务模块:
业务模块只负责"创建提案 + 业务执行回调",所有投票动作统一走投票引擎。

## 改动清单

### 5 个业务 pallet

| pallet | 删除 | 新增/保留 |
|---|---|---|
| `admins_change` (12) | `vote_admin_replacement` call | `propose_admin_replacement` + `execute_admin_replacement`(call_index 2→1) + `InternalVoteExecutor` |
| `resolution_destro` (14) | `vote_destroy` call | `propose_destroy` + `execute_destroy`(call_index 2→1) + `InternalVoteExecutor` |
| `grandpakey_change` (16) | `vote_replace_grandpa_key` call | `propose_replace_grandpa_key` + `execute_replace_grandpa_key`(2→1) + `cancel_failed_replace_grandpa_key`(4→2) + `InternalVoteExecutor` |
| `duoqian_manage` (17) | `finalize_create` + `vote_close` + `CreateVoteIntent` + `signing_hash` + 相关 use | `propose_create`/`propose_close`/`register_sfid_institution`/`propose_create_personal`(5→3)/`cleanup_rejected_proposal`(6→4) + `InternalVoteExecutor`(按 ACTION_CREATE / ACTION_CLOSE 分派) |
| `duoqian_transfer` (19) | `finalize_transfer`/`finalize_safety_fund_transfer`/`finalize_sweep_to_main` + `TransferVoteIntent` + `verify_and_cast_votes` + 相关 use | `propose_X ×3`(0/1/2) + **新增** `execute_X ×3`(3/4/5) 作为任意人重试通道 + `InternalVoteExecutor`(三路分派:transfer 走 MODULE_TAG、safety_fund/sweep 走独立存储键) |

call_index 全部按"业务分组"重新连续排列,不保留占位(遵循"无兼容/无残留"铁律)。

### primitives/core_const

- 删除 `OP_SIGN_CREATE / OP_SIGN_TRANSFER / OP_SIGN_SAFETY_FUND / OP_SIGN_SWEEP`
  4 个 op_tag 常量(0x14~0x17)
- 保留 `DUOQIAN_DOMAIN`(地址派生仍在用)+ `OP_SIGN_BIND / VOTE / POP / INST`(SFID
  认证 + 公民投票 + 人口快照仍在用)
- 留注释说明 0x14-0x17 为已删占位,新业务从 0x18 起

### runtime configs/mod.rs

```rust
type InternalVoteResultCallback = (
    duoqian_transfer::InternalVoteExecutor<Runtime>,
    duoqian_manage::InternalVoteExecutor<Runtime>,
    admins_change::InternalVoteExecutor<Runtime>,
    resolution_destro::InternalVoteExecutor<Runtime>,
    grandpakey_change::InternalVoteExecutor<Runtime>,
);
```

另:
- 简化 `OnchainTxAmountExtractor` 中 `VotingEngine::Call` 的匹配(create_X/finalize_X
  已删,只剩 internal_vote/finalize_proposal 免费,其他付费 1 元)
- 删除 `DuoqianManage::Call::finalize_create` / `vote_close` 两条匹配
- 修复 runtime lib.rs 的旧 `ResolutionDestro::vote_destroy` 测试 → `internal_vote`

### Phase 1 的关键追加修复

**callback 触发条件修正**:`set_status_and_emit` 在提案终态转换时回调业务模块,
原 Phase 1 实现在 `PASSED → EXECUTED` / `PASSED → EXECUTION_FAILED` 二次转换时
也会触发回调 + `approved = (status == PASSED) = false` 被错误解读为"否决"。

修复:callback 只在首次进入 `STATUS_PASSED / STATUS_REJECTED` 时触发,不覆盖
`EXECUTED / EXECUTION_FAILED` 的业务结果转换。

此修复对 Phase 1 测试无影响(已验证 49/49 pass),是 Phase 2 业务 Executor
能正常工作的前提。

## Executor 统一模板

```rust
pub struct InternalVoteExecutor<T>(PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(proposal_id: u64, approved: bool) -> DispatchResult {
        // 1. 认领:通过 MODULE_TAG 前缀 + 独立存储键判断是否本模块提案
        let raw = match voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(()),  // 非本模块提案,skip
        };

        if approved {
            // 2a. 通过:解码 action → 调 try_execute_X
            //     失败发 ExecutionFailed 事件,保留 PASSED 状态供 execute_X 重试
        } else {
            // 2b. 否决:清理业务独立存储(如 SafetyFundProposalActions)
        }
        Ok(())
    }
}
```

## 执行失败语义

**铁律:Executor 绝不 return Err**(除非数据层异常如解码失败)。业务执行
失败 → 发 `ExecutionFailed` 事件 + 保留 PASSED 状态 → 任意人通过 `execute_X`
重试。若 return Err,Phase 1 的 `set_status_and_emit` 会回滚整个事务,导致
票数消失,与"一人一票 + 结果不可回滚"的治理铁律冲突。

## MODULE_TAG 全局唯一性

在 `runtime/src/lib.rs` tests 模块加回归测试
`governance_module_tags_are_globally_unique`,覆盖 7 个业务 pallet 的 tag:

| pallet | MODULE_TAG |
|---|---|
| `admins_change` | `b"adm-rep"` |
| `grandpakey_change` | `b"gra-key"` |
| `resolution_destro` | `b"res-dst"` |
| `resolution_issuance_gov` | `b"res-iss"` |
| `runtime_upgrade` | `b"rt-upg"` |
| `duoqian_manage` | `b"dq-mgmt"` |
| `duoqian_transfer` | `b"dq-xfer"` |

## 验收

- ✅ `cargo check -p node`(需 WASM_FILE + 占位 frontend/dist)— **全链编译通过**
- ✅ 6 个 pallet 单测:
  - `voting-engine`: 49/49
  - `admins-change`: 20/20
  - `resolution-destro`: 14/14
  - `grandpakey-change`: 15/15
  - `duoqian-manage`: 17/17
  - `duoqian-transfer`: 20/20
  - **合计 135/135 通过**
- ✅ 全仓 `grep 'finalize_create\|finalize_transfer\|finalize_safety_fund\|finalize_sweep\|vote_admin_replacement\|vote_destroy\|vote_replace_grandpa_key\|vote_close' citizenchain/runtime/` → 空。后续任务已删除 `resolution-issuance` 与 `runtime-upgrade` 的手工 `finalize_joint_vote` extrinsic，仅保留 voting-engine 回调路径。
- ✅ 全仓 `grep 'TransferVoteIntent\|CreateVoteIntent\|OP_SIGN_(CREATE|TRANSFER|SAFETY_FUND|SWEEP)\|verify_and_cast_votes' citizenchain/` → 空
- ✅ 全仓 `grep 'cast_internal_vote' citizenchain/runtime/` → 空

## 清理残留

删除的聚合签名特化测试(Phase 2.4 + 2.5):

**duoqian-manage**:
- `finalize_rejects_non_admin_signature`
- `finalize_create_insufficient_sigs_rejected`
- `finalize_create_duplicate_sig_rejected`
- `finalize_create_tampered_sig_rejected`
- `finalize_create_malformed_sig_rejected`
- `finalize_create_second_call_is_replay_protected` → 改名 `passed_create_proposal_rejects_replay`

**duoqian-transfer**:
- `finalize_transfer_malformed_sig_rejected`
- `finalize_safety_fund_end_to_end`
- `finalize_sweep_end_to_end`
- `cross_op_signature_rejected_transfer_to_safety_fund`
- `cross_op_signature_rejected_sweep_to_transfer`
- `duplicate_signature_is_rejected`
- `finalize_transfer_insufficient_sigs_rejected`
- `finalize_transfer_tampered_amount_rejected`
- `non_admin_cannot_propose_or_finalize`

这些测试专门验证聚合签名的"签名非法/阈值不足/outsider 签名/跨业务隔离"等错误
语义,在新架构下统一由投票引擎的 `NoPermission` / `AlreadyVoted` 覆盖,原测试
意图失效,整体删除。

测试辅助函数迁移:
- `finalize_with` / `finalize_transfer_n`(duoqian-manage / duoqian-transfer)
  参数签名保留,内部改为循环调 `VotingEngine::internal_vote(...)`。
- 删除 `CreateVoteIntent::signing_hash` / `sign_create_intent` / `make_transfer_sigs`
  等聚合签名构造 helper。

## 不在本 Phase 范围

Phase 3:客户端改造(signing.rs / 前端 / wumin / wuminapp)统一为
`pallet=9 call=0` 的 internal_vote 格式。Phase 3 完成前客户端不能上线。

## 相关

- `20260422-unified-voting-entry-phase1.md`(Phase 1 投票引擎整改)
- `project_duoqian_finalize_create_step1.md` / `project_duoqian_finalize_transfer_step2.md`
  auto-memory 条目在 Phase 3 完工时一起标记 obsolete
