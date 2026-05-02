# resolution-issuance 技术文档

## 1. 模块定位

`resolution-issuance` 是 CitizenChain 的决议发行完整流程 pallet，统一承载：

- 创建决议发行联合投票提案
- 将业务数据写入 `voting-engine::ProposalData`
- 只接受 `voting-engine` 终态转换事务内的回调执行发行
- 记录永久防重放标记与短期审计标记
- 提供暂停开关与短期执行记录清理入口

本模块位于：

- 代码：`citizenchain/runtime/issuance/resolution-issuance/`
- Runtime pallet：`ResolutionIssuance`
- Runtime pallet index：`8`
- ProposalData 模块标签：`b"res-iss"`

## 2. 设计原则

- 决议发行是一个完整业务功能，提案、投票回调和发行执行必须在同一 pallet 内闭环。
- 发行执行不再作为独立外部模块暴露，只有 `voting-engine` 的 `JointVoteResultCallback::on_joint_vote_finalized` 回调路径可以触发实际铸币。
- `propose_resolution_issuance` 保持 call index `0`，降低冷钱包签名路径的变更范围。
- `finalize_joint_vote` 手工 extrinsic 已删除，call index `1` 保持空缺，避免 Root 或误配 origin 绕过投票引擎。
- 提案核心数据、owner、业务 data 和投票凭证清理由 `voting-engine` 终态清理队列统一处理，本模块不再调用已废弃的 `cleanup_joint_proposal`。
- 旧 index `7` 不再注册任何 pallet。
- 当前链处于开发期 fresh genesis 口径，合并不做历史 storage 迁移；`migration.rs` 只推进 `StorageVersion`，不再为 `AllowedRecipients` 保留运行期兜底写入。如果未来已有运行链数据，必须单独设计显式迁移。

## 3. 文件结构

```text
citizenchain/runtime/issuance/resolution-issuance/
  Cargo.toml
  src/
    lib.rs
    proposal.rs
    execution.rs
    validation.rs
    migration.rs
    benchmarks.rs
    weights.rs
    tests.rs
```

- `lib.rs`：FRAME pallet 壳，包含 `Config`、storage、event、error、genesis、hooks 和 extrinsics。
- `proposal.rs`：提案创建、ProposalData 编解码、`owns_proposal` 与联合投票结果处理。
- `execution.rs`：发行执行、防重放、累计发行、暂停与短期执行记录清理。
- `validation.rs`：收款名单、金额、分配明细和 CHINA_CB 地址校验。
- `migration.rs`：storage version 与开发期 fresh genesis 说明；当前只维护 storage version。
- `benchmarks.rs` / `weights.rs`：合并后的 benchmark 与权重，覆盖全部公开 call。当前 `weights.rs` 为保守 fallback，不伪装成正式 benchmark 产物；发布前仍需用带 Benchmark Runtime API 的 WASM 重新生成。
- `tests.rs`：提案、回调、执行、暂停、清理和事件来源回归测试。

## 4. Runtime 接口

公开 call：

| call index | extrinsic | 说明 |
|---:|---|---|
| 0 | `propose_resolution_issuance` | 创建决议发行联合投票提案 |
| 1 | 空缺 | 原手工 `finalize_joint_vote` 已删除，终结只能经 voting-engine 回调 |
| 2 | `set_allowed_recipients` | 更新合法收款账户集合 |
| 3 | `clear_executed` | 清理短期执行记录，不清理永久防重放标记 |
| 4 | `set_paused` | 设置暂停开关 |

## 5. Storage

| storage | 说明 |
|---|---|
| `AllowedRecipients` | 合法收款账户集合 |
| `VotingProposalCount` | 当前 Voting 状态的决议发行提案数量 |
| `Executed` | 短期执行记录，可由维护入口清理 |
| `EverExecuted` | 永久防重放标记，不允许清理 |
| `TotalIssued` | 决议发行累计执行量 |
| `Paused` | 紧急暂停开关 |

## 6. 核心流程

1. 管理员调用 `propose_resolution_issuance`。
2. 模块校验理由、总金额、分配明细和合法收款名单。
3. 模块通过 `JointVoteEngine::create_joint_proposal_with_data` 创建联合投票提案，并在同一事务中写入 owner/data/meta。
4. `ProposalData` 内容为 `MODULE_TAG + IssuanceProposalData`。
5. 投票引擎终结联合投票后，在自身状态转换事务内回调 `ResolutionIssuance`。
6. 如果投票通过，模块在同一事务内执行发行、记录防重放并递减计数；提案数据由 voting-engine 终态清理队列统一延迟清理。
7. 如果投票否决，模块只递减计数；提案数据由 voting-engine 终态清理队列统一延迟清理。
8. 如果投票通过且执行成功，模块发出执行事件，并返回 `ProposalExecutionOutcome::Executed`。
9. 如果投票通过但执行失败，模块发出失败事件，并返回 `ProposalExecutionOutcome::FatalFailed`。
10. allocation 结构性校验集中在 `validation.rs`：收款人集合、唯一性、单笔非零与总额匹配由共享校验统一负责。
11. `execution.rs` 在共享校验后只保留执行期专属检查：暂停、防重放、理由长度、Existential Deposit、单笔 cap、累计 cap 和实际入账结果。

## 7. 安全边界

- 分配明细的收款人集合必须与 `AllowedRecipients` 完全一致。
- 分配明细的结构性校验只维护一份共享实现，避免提案层与执行层出现校验漂移。
- `AllowedRecipients` 只能新增，不能移除已有账户。
- 存在 Voting 提案时禁止更新 `AllowedRecipients`。
- `EverExecuted` 是永久防重放标记，`clear_executed` 不得清理它。
- `Paused=true` 时拒绝新的发行执行。
- 发行执行使用 storage layer，任一收款失败都会整体回滚。
- `apply_joint_vote_result` 会校验 `CallbackExecutionScopes`、联合提案类型和 voting-engine 状态：
  - `approved=true` 时只接受 `STATUS_PASSED`。
  - `approved=false` 时只接受 `STATUS_REJECTED`。
  - 已进入 `STATUS_EXECUTED` / `STATUS_EXECUTION_FAILED` 等终态的提案不得二次回调。

## 8. 联动影响

- `sfid/backend/indexer/event_parser.rs` 需要按 `ResolutionIssuance / ResolutionIssuanceExecuted` 解析治理发行事件。
- `wumin/lib/signer/pallet_registry.dart` 需要把 spec_version 更新到当前 runtime 版本，并使用 `resolutionIssuancePallet = 8`。
- `wuminapp` 与节点桌面端继续可通过 `b"res-iss"` 识别联合提案类型。

## 9. 验证命令

```bash
cd /Users/rhett/GMB/citizenchain
cargo test -p resolution-issuance
WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain
WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain --features runtime-benchmarks
./target/release/citizenchain benchmark pallet --chain=citizenchain --pallet=resolution_issuance --extrinsic='*' --steps=50 --repeat=20 --output=runtime/issuance/resolution-issuance/src/weights.rs
```

## 10. 权重状态

- `benchmarks.rs` 已覆盖 `set_allowed_recipients`、`propose_resolution_issuance`、`clear_executed`、`set_paused` 四个公开入口。
- Cargo feature：`runtime-benchmarks` 会向 `pallet-balances` 与 `voting-engine` 传播；`primitives` 当前不暴露 benchmark feature，不在传播列表中。
- 当前本地尝试生成正式 `weights.rs` 时，普通 CI WASM 缺少 Benchmark Runtime API；`WASM_BUILD_FROM_SOURCE=1` 又被 `wasm32v1-none` 下 `serde_core` / `byte-slice-cast` 的 `std` feature 问题阻塞。
- 因此 `weights.rs` 暂时采用偏高保守 fallback，发布前必须准备 benchmark runtime WASM 后重新生成。
