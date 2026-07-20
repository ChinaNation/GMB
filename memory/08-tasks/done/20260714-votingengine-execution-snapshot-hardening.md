# VotingEngine 执行重试与资格快照收口

状态：已完成并通过最终验收。

## 任务目标

- 修复异步业务执行外层事务错误不递增次数、每块无限重排的问题。
- 将联合公投、立法公投和 Popular 选举统一改为 citizen-identity 提供的创建时资格快照。
- Popular 选举取消完整选民列表，解除 4,096 名选民上限；Mutual 继续使用机构 admins 快照。
- 将 joint-vote 状态机测试迁回 joint-vote crate，并直接覆盖公开 extrinsic。

## 安全边界

- 投票引擎只保存并消费人口快照引用，不自行维护公民资格真源。
- citizen-identity 是人口数量、创建时资格和资格历史的唯一真源。
- 机构互选只使用管理员模块提供的 admins 快照。
- 自动执行失败必须有次数、退避和 dead-letter；确定性错误不得每块无限重试。
- 不保留旧接口、旧存储语义、迁移兼容或双轨流程。
- 本任务按重新创世口径实施，runtime spec 与受影响 StorageVersion 保持 1。

## 实施范围

1. 重构 VotingEngine 自动执行失败路径，统一回调错误、Ignored、Track 后处理错误和孤儿队列处理。
2. 扩展 citizen-identity 人口快照，使 snapshot_id 同时冻结分母和成员资格。
3. joint-vote 与 legislation-vote 按 snapshot_id 校验公投资格。
4. election-vote Popular 模式改用 snapshot_id，Mutual 模式保留有界 admins 快照。
5. 迁移并补齐 joint-vote crate 自有状态机和公开 extrinsic 测试。
6. 重新生成相关 benchmark/weights，执行 workspace、no_std、runtime-benchmarks、try-runtime、release WASM 与 fresh genesis 验收。
7. 更新技术文档、ADR、中文注释并清理旧口径残留。

## 计划新增文件

- `citizenchain/runtime/votingengine/joint-vote/src/tests/mod.rs`：joint-vote 自有 mock runtime 和状态机测试。

## 验收标准

- 外层事务错误按指数退避并在上限后进入 EXECUTION_FAILED/dead-letter，不存在固定每块重排路径。
- 联合公投和立法公投的人口分母、成员资格均来自同一个 citizen-identity snapshot_id。
- Popular 创建不接收或保存完整选民列表，不受互选管理员名单上限限制。
- Mutual 选举仍严格校验完整 admins 快照。
- joint-vote crate 直接覆盖 cast_admin、cast_referendum、105 票全票、机构否决和超时转公投。
- 相关测试、benchmark、release WASM 和全新创世真实启动全部通过。
- 文档、注释、版本号和残留清理完成。

## 实施结果

- VotingEngine 将回调错误、`Ignored`、结果应用错误和 Track 终态错误统一纳入有界重试；达到上限后写入 `STATUS_EXECUTION_FAILED` 和 dead-letter，终态副作用由独立队列处理，不再重复业务执行。
- `citizen-identity` 新增资格 revision、账户不可变资格历史和四级人口数据；联合公投、立法公投和 Popular 选举由投票引擎读取人口真源后，统一生成并保存 `ProposalPopulationSnapshots[proposal_id]`。
- Popular 选举删除完整选民列表和人数上限；Mutual 选举只接受管理员模块提供并核验的完整 `admins` 快照。
- `joint-vote` 直属测试增至 10 项，直接覆盖 `cast_admin`、`cast_referendum`、105 票全票、机构否决和 30 天超时转公投。
- 相关 benchmark 以 `steps=50`、`repeat=20`、WASM compiled 重新生成；runtime 升级执行预算显式叠加 `set_code` 权重。
- 旧 `ReferendumScopes`、`referendum_scope`、`MaxElectionVoters`、`ElectionVoters` 和旧权重类型路径已从当前代码与现行文档清除。
- 所有触达 pallet 均使用最终创世布局；`citizen-identity` 与五个投票 pallet 显式 `StorageVersion = 1`，未增加迁移或兼容分支。

## 最终验收

- 专项测试：`citizen-identity` 23、`internal-vote` 96、`joint-vote` 10、`legislation-vote` 33、`election-vote` 13、runtime 40 项全部通过。
- `cargo test --workspace` 全量通过；六个相关 crate 的 `no_std` 检查通过。
- `cargo check -p citizenchain --features runtime-benchmarks,try-runtime` 通过。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --release --bin citizenchain` 通过。
- 当前源码以 `citizenchain-fresh --tmp` 真实启动成功；`system_health.isSyncing=false`，genesis hash 为 `0xd81962210c603a4a0f078b2cc022bac3daab344cd7dce8c6fc3501973d1552ab`，metadata RPC 响应 418,806 字节，`specVersion=1`、`systemVersion=1`、`stateVersion=1`；验收节点已停止。
- `cargo fmt --check`、`git diff --check`、冲突标记和当前口径残留扫描均通过；未推送、未部署、未修改冻结 chainspec。
