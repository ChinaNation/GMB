# votingengine 提级与目录拆分

- **日期**: 2026-05-04
- **模块**: Blockchain Agent — `citizenchain/runtime/voting-engine` 提级到 `citizenchain/runtime/votingengine`
- **优先级**: 高
- **依赖**: 无(本卡是前置;PR-Y 双层 ID 与 PR-Z 客户端同步在本卡完成后串行启动)

## 背景

`citizenchain/runtime/governance/voting-engine/` 当前位置存在两个问题:

1. **架构错位** — 依赖图上 7 个 pallet(duoqian-manage / duoqian-transfer / offchain-transaction / resolution-issuance / admins-change / resolution-destro / grandpakey-change)都依赖投票引擎,但投票引擎被放在 `governance/` 子目录,看起来像是 governance 的 sibling pallet。事实是 governance 反过来依赖它。
2. **`lib.rs` 6135 行过大** — 单文件混杂 types / traits / `#[pallet]` 宏模块 / 引擎 trait impl / mock runtime / 189 个测试函数,新人改动定位困难。

总行数清单(改造前):

| 文件 | 行数 |
|---|---|
| `src/lib.rs` | 6135 |
| `src/internal_vote.rs` | 512 |
| `src/joint_vote.rs` | 458 |
| `src/citizen_vote.rs` | 191 |
| `src/proposal_cleanup.rs` | 118 |
| `src/active_proposal_limit.rs` | 42 |
| `src/weights.rs` | 275 |
| `src/benchmarks.rs` | 258 |
| **合计** | **7989** |

## 决策(已与 user 确认)

1. **目录提级**:`runtime/governance/voting-engine/` → `runtime/votingengine/`
2. **crate 改名**:`voting-engine` → `votingengine`(单词风格,与 `otherpallet` 一致)
3. **`construct_runtime!` alias 保持 `VotingEngine` 不动** ← 铁律
4. **链上 storage prefix `'VotingEngine'` 不动** ← 铁律,11+ 处客户端字面量 0 改动
5. **`lib.rs` 拆分**:按功能轴切到 19 个生产文件 + 11 个测试文件
6. **L5 投票模式 选垂直切片** — 每种投票模式自成一个文件,内含 create + vote + finalize + 引擎 trait impl
7. **mock runtime 独立到 `tests/mock.rs`**(与 `tests/fixtures.rs` 分开)
8. **`engine_impl.rs` 不单独存在**,`impl InternalVoteEngine for Pallet<T>` 合并进 `internal_vote.rs` 末尾;`impl JointVoteEngine for Pallet<T>` 合并进 `joint_vote.rs` 末尾

## 目录结构(锁定)

```
runtime/votingengine/
├── Cargo.toml
├── benches/
│   └── benchmarks.rs           # 由 src/benchmarks.rs 提级
└── src/
    ├── lib.rs                  L0  #[pallet] 宏本体,目标 ≤ 1500 行
    │                              含 storage / events / errors / calls 签名 / hooks 声明
    │
    ├── types.rs                L1  Proposal<BN> / VoteCountU32 / VoteCountU64 /
    │                              ProposalMetadata / ProposalObjectMetadata /
    │                              PendingCleanupStage / InternalProposalMutexState /
    │                              InternalProposalMutexBinding / ExecutionRetryState /
    │                              ProposalExecutionOutcome / ProposalCancelDecision /
    │                              InternalProposalMutexKind;
    │                              常量 PROPOSAL_KIND_* / STAGE_* / STATUS_*
    │
    ├── traits.rs               L1  对外 trait 定义:
    │                              JointVoteEngine / InternalVoteEngine /
    │                              PopulationSnapshotVerifier /
    │                              JointVoteResultCallback / InternalVoteResultCallback(含 tuple impl) /
    │                              InternalAdminProvider / InternalAdminCountProvider /
    │                              InternalThresholdProvider;
    │                              全部默认 () impl
    │
    ├── id.rs                   L1  提案 ID 体系:
    │                              allocate_proposal_id() / current_year() /
    │                              unix_seconds_to_year() / YearProposalCounter 维护
    │                              (本卡只搬代码,双层 ID 主键改造在 PR-Y 接入)
    │
    ├── data.rs                 L2  ProposalData / Owner / Meta:
    │                              register_proposal_data() / is_proposal_owner() /
    │                              get_proposal_data() / get_proposal_meta() /
    │                              get_proposal_object() / get_proposal_object_meta() /
    │                              bounded_module_tag()
    │
    ├── index.rs                L2  反向索引接口(本卡建空 stub,PR-Y 写实现)
    │
    ├── snapshot.rs             L3  AdminSnapshot + InternalThresholdSnapshot
    │                              snapshot_institution_admins() / is_admin_in_snapshot() /
    │                              snapshot_admin_count()
    │
    ├── mutex.rs                L3  InternalProposalMutex 互斥锁:
    │                              acquire_internal_proposal_mutex() /
    │                              release_internal_proposal_mutexes() /
    │                              ensure_admin_set_mutation_lock_owner()
    │
    ├── limit.rs                L3  ActiveProposalsByInstitution 容量上限
    │                              (即原 active_proposal_limit.rs 内容)
    │
    ├── internal_vote.rs        L5  do_create_internal_proposal* 系列 5 个变体 +
    │                              internal_vote extrinsic 主逻辑 + 计票 + 阶段切换 +
    │                              on_internal_vote_finalized 回调分发 +
    │                              impl<T> InternalVoteEngine for Pallet<T>
    │
    ├── joint_vote.rs           L5  do_create_joint_proposal* 系列 3 个变体 +
    │                              joint_vote extrinsic + 管理员阶段→公民阶段切换 +
    │                              on_joint_vote_finalized 回调分发 +
    │                              impl<T> JointVoteEngine for Pallet<T>
    │
    ├── citizen_vote.rs         L5  citizen_vote extrinsic + PopulationSnapshotVerifier 调用 +
    │                              UsedPopulationSnapshotNonce 防重放 + finalize_citizen
    │
    ├── execution.rs            L4  retry_passed_proposal / cancel_passed_proposal +
    │                              ProposalExecutionRetryStates / ExecutionRetryDeadlines
    │
    ├── expiry.rs               L4  schedule_proposal_expiry() +
    │                              process_pending_expiry(weight_budget)(on_initialize 调用) +
    │                              ProposalsByExpiry 维护
    │
    ├── cleanup.rs              L4  90 天保留期级联清理:
    │                              process_pending_cleanup(weight_budget) +
    │                              PendingExpiryBucket / PendingCleanupStage 状态机
    │                              (即原 proposal_cleanup.rs 内容)
    │
    ├── hooks.rs                L4  on_initialize 主入口:weight 预算切分给
    │                              expiry / retry / cleanup 三家 +
    │                              on_runtime_upgrade 调度 migrations
    │
    ├── migrations/
    │   └── mod.rs              本卡建空 mod 占位,PR-Y 在此写 v1
    │
    ├── weights.rs              现有,保留
    │
    └── tests/
        ├── mod.rs              测试入口 + mod 声明(mock / fixtures / 各场景)
        ├── mock.rs             mock runtime(独立文件)
        ├── fixtures.rs         公共 fixtures:
        │                       nrc_pid / prc_pid / prb_pid / nrc_admin / prc_admin /
        │                       prb_admin / institution_admins / institution_threshold 等
        ├── id.rs               ID 分配 / 跨年 / counter cap
        ├── internal.rs         内部投票场景(NRC/PRC/PRB/duoqian)
        ├── joint.rs            联合投票场景(管理员阶段 + 公民阶段)
        ├── citizen.rs          公民投票场景
        ├── mutex.rs            admin_set_mutation 互斥
        ├── limit.rs            ActiveProposalsByInstitution 上限
        ├── expiry.rs           到期 + 90 天清理
        ├── execution.rs        retry / cancel
        └── reverse_index.rs    本卡建空 stub,PR-Y 写实现
```

**目标体量**:生产代码 ~6500 行散到 19 文件(平均 ~340 行/文件,lib.rs ≤ 1500),测试 ~3100 行散到 11 文件(平均 ~280 行/文件)。

## 实现步骤

### Step 1 — 目录搬迁与 Cargo 改名

1. `git mv citizenchain/runtime/governance/voting-engine citizenchain/runtime/votingengine`
2. 改 `runtime/votingengine/Cargo.toml` 的 `name = "voting-engine"` → `name = "votingengine"`
3. 改 7 个依赖 pallet 的 Cargo.toml `path` 字段(`../voting-engine` / `../../governance/voting-engine` → 新路径)
4. 改 workspace `runtime/Cargo.toml`:`path = "governance/voting-engine"` → `path = "votingengine"`
5. 改 `runtime/src/lib.rs` `construct_runtime!`:`pub type VotingEngine = voting_engine;` → `pub type VotingEngine = votingengine;`(**alias 保持 `VotingEngine`**,只换 crate 名)
6. 全局机械替换:`use voting_engine::` / `voting_engine::` → `use votingengine::` / `votingengine::`(424 处)
7. `cargo check --workspace` 确认编译通过

### Step 2 — `lib.rs` 拆分

按目录结构表把代码搬到对应文件。**原则:零行为变化**,只搬代码 + 改 mod 路径 + 改 import,不改任何业务逻辑。

注意:
- 公开 API(extrinsic 签名 / 引擎 trait / 默认 () impl)所有 `pub` 不变
- 所有 helper 通过 `impl<T: Config> Pallet<T> { ... }` 块在子模块挂载
- 测试用 `mod tests; mod mock; mod fixtures;` 在 `tests/mod.rs` 顶级声明
- 投票模式 trait impl(`impl<T> InternalVoteEngine for Pallet<T>` 等)合并进对应模式文件末尾

### Step 3 — 测试拆分

把 lib.rs 第 3059-6135 行的测试段(189 个测试)按场景分到 11 个文件。**fixtures 与 mock 独立**,被各场景文件 `use super::mock::*; use super::fixtures::*;` 引入。

### Step 4 — 编译 + 测试 + 链上检查

- `cargo check -p votingengine`
- `cargo test -p votingengine`(189 个测试全过)
- `cargo build --release -p citizenchain-runtime`
- `cargo test --workspace`
- `cargo build --release --features runtime-benchmarks`
- `cargo build --release --features try-runtime`

## 验收标准(零行为变化)

- [ ] 现有 189 个 voting-engine 单元测试全部通过
- [ ] 依赖 voting-engine 的 7 个 pallet 单元测试全部通过(duoqian-manage / duoqian-transfer / offchain-transaction / resolution-issuance / admins-change / resolution-destro / grandpakey-change)
- [ ] runtime 整体 build 通过,三套 flag(默认 / `--features runtime-benchmarks` / `--features try-runtime`)全过
- [ ] `runtime/src/lib.rs` 的 `construct_runtime!` 里 `VotingEngine` alias 保留
- [ ] `spec_version` 不动(仍为 0)
- [ ] 工作树 grep `voting-engine` 仅剩 README / 任务卡 / 历史回溯文档里的字面量,生产代码 0 残留
- [ ] 工作树 grep `voting_engine::` 0 残留(全替换成 `votingengine::`)
- [ ] 工作树 grep `'VotingEngine'`(带引号字面量)在客户端代码里 0 改动 — 链上 storage prefix 铁律

## 不做的事(范围外,留给后续 PR)

- ❌ 双层 ID 改造(主键 u64 单调 / `ProposalDisplayId` / `YearProposalCounter` 去 cap)→ **PR-Y**
- ❌ 反向索引实现(`ProposalsByOrg / ByInstitution / ByOwner / ByYear` 写入与清理逻辑)→ **PR-Y**;本卡只建空 `index.rs` 文件占位
- ❌ `on_runtime_upgrade` 迁移代码 → **PR-Y**;本卡只建空 `migrations/mod.rs`
- ❌ wuminapp / wumin / Tauri 节点前端 / sfid 客户端任何改动 → **PR-Z**;本卡因 Rust crate 名变化与外部 Dart/TS 客户端无关,链上 storage prefix `'VotingEngine'` 不动,客户端 storage key 计算照旧
- ❌ 修任何已知 bug(如 wuminapp `_hasMore` 翻页、`fetchProposalPage` 事后过滤等)→ 单独 PR

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 误改链上 alias 导致 storage prefix 漂移 | 验收标准强制 grep `'VotingEngine'` 字面量在客户端代码里 0 改动 |
| 测试 mock runtime 切到独立文件后,内部 type 路径解析失败 | Step 3 优先把 `mock.rs` 编译过,再搬测试 |
| 已 merge 但未发现的 PR 在 base 切过来后大量 conflict | PR 启动前先 rebase base 锁定 commit;PR 期间禁止其他人改 voting-engine |
| 7 个依赖 pallet 的 import path 改写漏处 | 依赖 `cargo check --workspace`,任何遗漏立即编译失败 |
| 测试拆分后 mod 可见性问题(`pub(crate)` vs `pub(super)`) | Step 3 测试文件统一用 `use super::*;` 模式 |

## 涉及文件清单

### 整体搬迁
- `citizenchain/runtime/governance/voting-engine/` → `citizenchain/runtime/votingengine/`(目录消失,新建)

### Cargo 文件
- `citizenchain/runtime/Cargo.toml`(workspace 成员路径 + 包名引用)
- `citizenchain/runtime/votingengine/Cargo.toml`(包名)
- `citizenchain/runtime/transaction/duoqian-manage/Cargo.toml`
- `citizenchain/runtime/transaction/duoqian-transfer/Cargo.toml`
- `citizenchain/runtime/transaction/offchain-transaction/Cargo.toml`
- `citizenchain/runtime/issuance/resolution-issuance/Cargo.toml`
- `citizenchain/runtime/governance/admins-change/Cargo.toml`
- `citizenchain/runtime/governance/resolution-destro/Cargo.toml`
- `citizenchain/runtime/governance/grandpakey-change/Cargo.toml`
- `citizenchain/runtime/governance/runtime-upgrade/Cargo.toml`(若依赖)

### 顶层入口
- `citizenchain/runtime/src/lib.rs`(`construct_runtime!` 块)

### 全局 Rust import
- 424 处 `voting_engine::` → `votingengine::`(机械替换,跨 ~12 个 .rs 文件)

## 后续 PR(本卡完成后启动)

| PR | 内容 | spec_version |
|---|---|---|
| **PR-Y 双层 ID + 反向索引 + 迁移** | 主键 u64 单调 / `ProposalDisplayId` / counter 去 cap / 四张反向索引 / `on_runtime_upgrade` v1 迁移 | 0 → 1 |
| **PR-Z 客户端同步** | wuminapp + Tauri 节点前端 + node 后端接入新 RPC,删 `id ~/ 1_000_000` 硬编码,顺手修 `_hasMore` 翻页 bug | 跟 PR-Y 升级版本同步发布 |

## 上下文加载

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/01-architecture/citizenchain-target-structure.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md
