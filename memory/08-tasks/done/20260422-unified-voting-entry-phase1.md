# 任务卡：投票引擎统一入口整改 · Phase 1

- 状态：done（2026-04-22，待 Phase 2/3 完工后整任务搬 done 并改命名为 `-complete`）
- 归属：Blockchain Agent（citizenchain runtime · voting-engine）

## 背景

Phase 0 分析（见 MEMORY 里的 `project_duoqian_finalize_transfer_step2.md`）发现：
- `VotingEngine::internal_vote` 的公开 call 被历史决策封禁（`Err(NoPermission)`），
  导致每个业务 pallet 被迫自己实现 `vote_X` 样板代码。
- Step 1/2 聚合签名（`finalize_create / finalize_transfer / finalize_safety_fund /
  finalize_sweep`）把问题进一步复杂化：两套投票路径共存、客户端要给每个业务造不同
  call_data、冷钱包签名语义失真。

Phase 1 目标：**把"内部投票"拉齐到"联合/公民投票"已有的架构标准**——投票引擎自己
承担"记票 + 阈值 + 终态回调"，业务模块只管业务。

## Phase 1 范围（本任务卡）

**只改** `citizenchain/runtime/governance/voting-engine/`。业务模块和客户端分别
在 Phase 2/3 改。

## 改动内容

### 1. 新增 `InternalVoteResultCallback` trait

`voting-engine/src/lib.rs`：新增 trait + 空实现 + 手写 tuple impl（覆盖 1~6
成员，Phase 2 预计注册 5 个业务 Executor）。命名与 `JointVoteResultCallback` 完全对称。

### 2. `Config` 关联类型

新增 `type InternalVoteResultCallback: InternalVoteResultCallback`。

### 3. `set_status_and_emit` 增加 internal 回调分支

在现有 joint 回调分支之后镜像添加 internal 回调（事务内调用，返回 Err 整体回滚）。
注释明确两条路径的对称性。

### 4. call_index 重排 + 解封 internal_vote

| call_index | 改前 | 改后 |
|---|---|---|
| 0 | `create_internal_proposal`(禁用) | **`internal_vote`** |
| 1 | `create_joint_proposal`(禁用) | `joint_vote` |
| 2 | `internal_vote`(禁用) | `citizen_vote` |
| 3 | `joint_vote` | `finalize_proposal` |
| 4 | `citizen_vote` | 已删除 |
| 5 | `finalize_proposal` | 已删除 |

**删除**两个禁用 call（`create_internal_proposal` / `create_joint_proposal`），提案创建
保留为 `InternalVoteEngine::create_internal_proposal` / `JointVoteEngine::create_joint_proposal`
trait 入口（业务模块内部调用）。

**解封** `internal_vote`：删 `Err(NoPermission)`，改调 `Self::do_internal_vote(who, ...)`。
`do_internal_vote` 在 `internal_vote.rs:179-245` 的完整实现（权限/防双投/记票/阈值/状态
推进）本来就在，解封后直接生效。

### 5. 彻底删除 `InternalVoteEngine::cast_internal_vote`

按"no 兼容/保留/过渡"铁律：
- trait `InternalVoteEngine::cast_internal_vote` 方法移除
- `impl InternalVoteEngine for ()` 里对应实现移除
- `impl InternalVoteEngine for pallet::Pallet<T>` 里对应实现移除

业务模块 Phase 2 改造时不再通过 trait 转发投票；所有管理员必须通过公开
`internal_vote(proposal_id, approve)` extrinsic 自己投票。

### 6. weights.rs + benchmarks.rs

- `WeightInfo` trait 新增 `fn internal_vote() -> Weight`
- `SubstrateWeight` / `()` 两处 impl 新增占位实现（Phase 2 后重跑 benchmark-cli）
- `benchmarks.rs` 新增 `fn internal_vote() -> Result<(), BenchmarkError>` benchmark

### 7. runtime 侧占位

`citizenchain/runtime/src/configs/mod.rs:1326` 新增：
```rust
type InternalVoteResultCallback = ();
```
Phase 2 业务模块改造完成后替换为 5 元 tuple。

### 8. 测试

- 新增 7 个单元测试（`lib.rs` 末尾），覆盖：
  - `internal_vote_public_call_casts_vote`
  - `internal_vote_rejects_non_admin`
  - `internal_vote_rejects_double_vote`
  - `internal_vote_passes_triggers_callback_approved_true`
  - `internal_vote_early_rejection_triggers_callback_approved_false`
  - `internal_vote_callback_not_called_before_threshold`
  - `internal_vote_callback_err_rolls_back_status`
  - `internal_vote_rejects_wrong_stage_joint_proposal`
- 新增 mock 桩：`TestInternalVoteResultCallback` + 两个 thread_local
  (`INTERNAL_CALLBACK_SHOULD_FAIL` / `INTERNAL_CALLBACK_LOG`)
- 辅助函数 `cast_internal_vote_via_extrinsic(who, id, approve)` 替代已删除的
  `InternalVoteEngine::cast_internal_vote`
- 清理 2 处引用已删除 `create_X_proposal` 公开 call 的旧测试片段

## 验收

- ✅ `cargo test -p voting-engine --lib` — **49/49 全部通过**
- ✅ `grep -n 'cast_internal_vote' voting-engine/` — 空
- ✅ `grep -n 'pub fn create_internal_proposal\|pub fn create_joint_proposal'
  voting-engine/src/lib.rs` 只匹配 trait 定义，不再有 `#[pallet::call]`

## 不在本 Phase 范围

- ⚠️ **整链 `cargo check -p node` 不通过**：业务模块（admins-change / resolution-destro /
  grandpakey-change / duoqian-manage-pow / duoqian-transfer-pow）当前仍在引用
  `InternalVoteEngine::cast_internal_vote`，Phase 2 会统一清理。
- Phase 2：业务模块改造（删 `vote_X` / `finalize_X`，新增 `InternalVoteExecutor`
  实现 `InternalVoteResultCallback`）
- Phase 3：客户端改造（signing.rs / 前端 / wumin / wuminapp 统一为 `pallet=9 call=0`
  的单一投票 call 格式）

## 相关

- 废弃/归档 MEMORY 条目（Phase 3 完工时统一处理）：
  - `project_duoqian_finalize_create_step1.md`
  - `project_duoqian_finalize_transfer_step2.md`
