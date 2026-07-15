# 20260714 VotingEngine 结构、Track 与 Benchmark 收口

状态：三步全部完成，等待用户验收。

## 目标

- 把 VotingEngine 生产文件物理拆分到单文件不超过 800 行，删除空壳和重复实现。
- 把到期、业务执行和清理维护管线改为独立有界预算，清理队列按 proposal 公平轮转。
- 用 Track handler 统一 timeout 与 cleanup 派发，删除核心引擎中的 mode-specific 巨型分支。
- 为核心、内部、联合、立法和选举投票补齐真实 FRAME benchmark，替换生产占位权重。
- 面向正式重新创世直接形成最终存储布局，不保留旧链迁移、兼容或双轨代码。

## 三步执行边界

1. 纯结构拆分：移动实现、完善中文注释、清理残留，不改变存储布局、call index、事件、错误和业务行为。
2. 公平清理与 Track 参数化：按最终创世布局改存储和维护调度，所有相关 pallet 的 `StorageVersion` 统一为 1。
3. 真实 benchmark 与创世验收：生成生产权重，运行覆盖率、四类构建和 fresh genesis 真实节点验收。

每一步完成并回写验收后，先输出下一步完整技术方案，得到用户确认和 runtime 二次确认后再执行。

## 创世与版本口径

- `votingengine`、`internal-vote`、`joint-vote`、`legislation-vote`、`election-vote` 的最终 `StorageVersion` 均为 1。
- 删除历史 storage alias、`on_runtime_upgrade` 翻译、迁移专用类型和旧布局迁移测试。
- runtime `spec_version` 当前已经为 1，本任务不机械修改 `transaction_version`、`authoring_version`、`impl_version` 或 Cargo 包版本。
- 本次 fresh genesis 收口同时把受改造影响的 `public-admins`、`public-manage`、`private-manage` 恢复为 `StorageVersion = 1`；runtime `spec_version = 1`，全仓 runtime 已不存在高于 1 的 storage version。

## 硬约束

- 投票流程只能存在于 VotingEngine 体系，业务 pallet 不能实现或绕过投票状态机。
- 公民资格和人口由 `citizen-identity` 提供；机构互选资格和快照由 admins provider 提供。
- 宪法公投阈值只调用 `primitives::constitution`，不得复制数学实现。
- 不保留旧存储、旧阶段路由、迁移兼容、空模块、占位权重或过期文档。
- 不修改本任务范围外的用户工作区改动，不执行 GitHub 推送、PR、远端 workflow 或生产部署。

## 第 1 步验收清单

- [x] `votingengine/src/lib.rs` 为 785 行，不超过 800 行。
- [x] `votingengine/src/traits.rs` 为 16 行门面文件，五个职责模块均不超过 505 行。
- [x] `internal-vote/src/lib.rs` 为 496 行。
- [x] `legislation-vote/src/lib.rs` 为 572 行。
- [x] 所有新增模块都有真实职责和中文模块注释，不存在纯注释残桩。
- [x] pallet 存储、call index、Event/Error 声明未移动或改序；只移动普通实现和 trait 定义。
- [x] 五个投票 crate、runtime 40 项测试和五 crate `no_std` 构建通过。

### 第 1 步运行态记录

- 五个投票 crate：internal 93、joint 3、legislation 33、election 3，全部通过。
- runtime：40/40 通过。
- runtime 普通、`runtime-benchmarks`、`try-runtime` 构建和五 crate `no_std` 构建通过；`runtime-benchmarks` 仅有任务范围外 `resolution-issuance` 的既有 unused import 警告。
- 当前源码重新构建 node 后，以 `citizenchain-fresh --tmp` 启动真实隔离节点；NodeGuard 与创世装载通过，`chain_getBlockHash(0)=0x15b19408800b8ab685b49e8076f861ed76b4713abea54a216a7be2dc0cee41ea`，`system_health.isSyncing=false`，metadata RPC 响应 415,224 字节；验收节点已正常停止。
- 本步没有修改存储布局、版本号和业务规则；创世 v1、迁移残留清理、公平清理与 Track 参数化属于第 2 步。

## 后续验收

- [x] 公平清理与 Track 参数化完成，所有 VotingEngine pallet 以 v1 最终布局创世。
- [x] 五个 pallet 共 19 个 benchmark 在目标配置真实生成，生产不再使用占位权重。
- [x] 投票体系可执行业务代码行覆盖率达到 81.80%，fresh genesis 节点真实运行通过。
- [x] 第 2 步技术文档、ADR、任务卡和中文注释已更新，旧版本与迁移描述清零。

## 第 2 步实现记录

- 新增 `votingengine/src/tracks.rs`，以递归 tuple 统一派发四类 Track 的超时、模式账本清理、执行成功和终态副作用。
- Runtime 删除八个 mode-specific finalizer/cleanup 关联类型，改为单一 `TrackHandlers` 注册。
- 延迟清理由区块桶改为 `ScheduledCleanups` FIFO；到期后进入 `PendingCleanupQueue` 公平 FIFO，未完成提案每轮排回队尾。
- 清理阶段由跨四类模式逐项空扫改为 `AdminSnapshots → TrackData → ProposalObject → FinalCleanup`，`TrackData` 只访问所属模式。
- 自动终结、异步执行、历史清理分别设置独立 weight 预算，并继续叠加各自条数上限。
- 五个 VotingEngine pallet 的 `StorageVersion` 已统一为 1，旧 storage alias、`on_runtime_upgrade`、迁移类型和迁移测试已删除。
- 新增公平轮转与 Track 隔离测试；内部投票测试由 93 项增至 94 项并全部通过，joint 3、legislation 33、election 3、runtime 40 项全部通过。

### 第 2 步验收记录

- `cargo check --workspace --tests` 通过；检查过程清理了 `personal-manage` 仍按旧双键读取 pending 阈值、仍假设投票通过后同步执行的测试残留。
- `personal-manage` 23、internal 94、joint 3、legislation 33、election 3、runtime 40 项测试全部通过。
- 五个投票 crate `no_std` 构建通过；runtime 普通、`runtime-benchmarks`、`try-runtime` 构建通过。benchmark 构建仅保留任务范围外 `resolution-issuance` 的既有 unused import 警告。
- 当前源码以 `WASM_BUILD_FROM_SOURCE=1` 重建 node 后，`citizenchain-fresh --tmp --pool-type single-state --mining-threads 0` 真实启动成功；NodeGuard 与创世装载通过，`chain_getBlockHash(0)=0xf20b42ad98756fa464678ab2473abc6f0be089dceae290c587cea80c1ead9ab1`，`system_health.isSyncing=false`，metadata RPC 响应 415,442 字节；隔离节点已正常停止。
- fresh 链连接到冻结旧 bootnode 时按预期报告 genesis mismatch；本步未改冻结 chainspec、bootnode 或部署状态，正式重新创世由发布流程统一更新。

## 第 3 步实现记录

- Runtime benchmark registry 已注册 `votingengine`、`internal-vote`、`joint-vote`、`legislation-vote`、`election-vote`，共生成 19 条正式样本。
- benchmark 环境：Apple M5 Pro / arm64、Rust 1.94.0、FRAME Benchmark CLI 53.0.0、WASM compiled、`steps=50`、`repeat=20`。
- 核心执行权重：公开终结保守包络 35 ms、手动重试 24 ms、取消 10 ms、异步执行 22 ms；公开终结另叠加实际 Track 权重，异步执行继续显式叠加 `SystemWeightInfo::set_code()`。
- 内部投票：写票 29 ms、超时终结 32 ms；联合投票五条为 12/25/22/13/20 ms；立法六条为 12/31/22/38/45/35 ms。
- 选举最后一票按候选人数线性计费：普选 `38,212,644 + 1,524,772*c` ps，互选 `36,834,244 + 1,534,883*c` ps；proof 与读次数同步按 `c` 增长。
- Track handler 新增 timeout、chunk cleanup、terminal cleanup 动态权重接口；公开终结、自动终结和清理维护均按实际 Track 返回值计账，不再只按核心固定权重或条数估算。
- 自动终结、异步执行、清理预算继续采用最大区块权重的 `1/4 + 1/4 + 1/8 = 62.5%`。60 秒最大计算区块下，每条管线均能容纳至少一项最重任务，无需放大预算。
- election-vote 在既有测试入口补齐完整 mock runtime，覆盖普选/互选快照、资格、写票、超时、结果回调与分块清理；异步执行改造触达的管理员、entity、GRANDPA 密钥和多签测试辅助均已按真实维护时序收口。
- 生产残留检查未发现旧 benchmark 占位说明、旧 finalizer/cleanup 关联类型、空模块、迁移入口或高于 1 的 runtime StorageVersion。

### 第 3 步验收记录

- 原生 LLVM coverage：排除测试、`benchmarks.rs`、`weights.rs` 以及纯声明 `traits/types/data` 后，可执行业务代码 4,324 行、命中 3,537 行，行覆盖率 81.80%；若把纯接口和类型声明也计入，五个投票 crate 全源码为 71.60%，两种口径均留档，不混淆口径。
- `cargo test --workspace --quiet` 全部通过；五个投票 crate 专项为 internal 94、joint 3、legislation 33、election 13。
- 五 crate `no_std`、runtime `runtime-benchmarks`、runtime `try-runtime` 均通过；benchmark 构建仍只有任务范围外 `resolution-issuance` 的既有 unused import 警告。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --release -p node` 通过。当前源码以 `citizenchain-fresh` 和全新临时 base path 启动成功，NodeGuard 与创世装载通过。
- fresh genesis hash 为 `0x8d3fc4c4567796d8056e61a8dbf431f04230126a1023a49ffecde7b5bff25390`，state root 为 `0x51ef488b720c9f049c501367f31e3779dd7a3711c295ce8cc79ddbe7688413ca`，runtime `specVersion=1`，`system_health.isSyncing=false`，metadata 响应 415,442 字节；节点已正常停止。
- fresh 链无交易且无同 genesis peer 时按本链规则不生产空块，因此验收链头保持 block 0；这符合“空块不提交 + 离线不挖矿”共识门禁。冻结 chainspec 与 bootnode 尚未更新，正式创世发布流程需统一烘焙。
