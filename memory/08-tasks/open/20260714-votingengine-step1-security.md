# 20260714 VotingEngine 第一步安全修复

状态：三步功能改造与运行态验收完成；主网上线前 benchmark 与大文件结构治理继续保持显式门禁。

## 任务目标

修复投票引擎第 1 步确认范围：立法超时提前终结、到期桶毒丸阻塞、公投判定重复实现、内部投票动态阈值并发覆盖。

## 范围

- 为立法签署、三人会签、护宪终审超时函数补充过期校验。
- 为自动终结失败提案增加有限重试与退避隔离，避免维护管线永久早退。
- 统一立法公投判定到 constitution 单源。
- 按 proposal_id 隔离 PendingDynamicThresholds。
- 更新中文注释、测试、模块文档并清理旧实现残留。

## 约束

- 本任务涉及 `citizenchain/runtime/`，执行前已获得 runtime 修改二次确认。
- 不执行远端推送、PR 或 CI/CD 触发操作。

## 验收记录

- 已完成立法三个超时入口的 `VoteNotExpired` 校验。
- 已完成自动到期失败提案移出原到期桶的止血处理，并在第三步补齐计数、指数退避和 dead-letter。
- 已完成立法公投判定切换到 `primitives::constitution::referendum_passed`，并清理重复规则导出和测试引用。
- 已完成 `PendingDynamicThresholds` 从 `(institution_code, account)` 改为 `proposal_id` 键控。
- `cargo test -p legislation-vote -p internal-vote -p votingengine`：通过；最终 `internal-vote` 93、`legislation-vote` 33、`votingengine` 0。
- `cargo test -p citizenchain`：通过，runtime 40 项测试全部通过。
- `cargo fmt -p votingengine -p legislation-vote -p internal-vote` 与 `git diff --check`：通过。
- 第 2 步已完成：普选按 `citizen-identity` 作用域校验，互选按 admins provider 校验，并将普选作用域写入 `ElectionMeta`。
- 第 2 步已完成：多席位并列计票改为“并列组完整落入剩余席位时共同当选”。
- 第 2 步基础验收：`cargo test -p election-vote` 3 项、`cargo test -p citizenchain -p joint-vote -p legislation-vote` 全部通过。

## 第 3 步完成记录

- 到期桶分批处理不再提前跳过其余维护管线；finalizer 错误使用独立计数、指数退避和 dead-letter，不再无限回插同一桶。
- 投票判定与业务执行已通过 `PendingProposalExecutions` 解耦；`on_initialize` 按四分之一区块 weight 与固定条数双重预算执行，回调错误有界退避，达到上限进入执行失败终态。
- 生产执行预算保守包含 `SystemWeightInfo::set_code()`；最后一票和公开 `finalize_proposal` 不再同步承担重业务回调。
- `internal-vote` storage v2 已实现旧动态阈值键迁移并增加并发提案迁移测试；`election-vote` storage v2 已实现旧元数据翻译；`votingengine` storage v3 登记新增队列状态。
- `joint-vote` 增加 3 项本 crate 边界测试；完整联合/公投状态机由 `internal-vote` 集成测试覆盖。
- legislation signing/guard/referendum/result/cleanup 空壳已改为实际规则与清理辅助模块。
- 普选按 `citizen-identity` 人口作用域校验完整选民集合和候选资格；互选选民集合必须与 admins provider 返回的完整管理员快照一致。
- 公投累计票数达到创建时人口分母后拒绝继续写票，分子不得超过快照分母。

## 保持 fail-closed 的边界

- `election-campaign` 的职位、发起机构、任期和选举法规则仍无仓库内权威真源，因此真实创建和任职写入继续保持 fail-closed，未猜测开放。
- 仓库尚无本次目标硬件和链配置生成的 benchmark CLI 产物；本次已完成 `runtime-benchmarks` 特性编译与 `set_code` 最重上界止血，但不能把手工权重宣称为真实 benchmark。主网上线门禁必须生成并审核实测权重。
- 后续任务卡 `20260714-votingengine-structure-tracks-benchmark` 已完成四个超限生产文件的物理拆分；核心 `lib.rs` 785 行、traits 门面 16 行、internal `lib.rs` 496 行、legislation `lib.rs` 572 行。剩余门禁为创世 v1 与迁移残留清理、公平清理、Track 参数化和真实 benchmark。
- 未修改或覆盖工作区内 citizenapp、deploy 与其它任务卡的既有用户改动；未执行 git push、PR 或远端 CI/CD。

## 最终验收

- 五个投票 crate：`internal-vote` 93、`joint-vote` 3、`legislation-vote` 33、`election-vote` 3、`votingengine` 0，全部测试与 doc-tests 通过。
- runtime：`cargo test -p citizenchain` 40/40 通过。
- 构建：runtime 普通、`--no-default-features`、`runtime-benchmarks`、`try-runtime` 四种检查通过；benchmark 特性仅有本任务范围外的 `resolution-issuance` 既有 unused import 警告。
- 格式与残留：目标 packages 已格式化，`git diff --check` 通过；旧异步原型撤回说明、同步最后一票注释和重复公投规则已清理。
- 真实运行态：用当前源码 WASM 启动隔离 `citizenchain-fresh` 节点，NodeGuard/创世装载通过，RPC `system_health.isSyncing=false`，block#0 为 `0x29c687fd920baf0c2e7461ac58c887860384877f3edfde014e5a82f4dcb793e3`，metadata 207,593 bytes；验收节点已正常停止。
