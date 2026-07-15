# ADR-035 VotingEngine Track 与维护预算最终架构

## 标题

VotingEngine 使用 Track handler、异步业务执行和公平维护预算，并以 v1 最终布局重新创世。

## 背景

VotingEngine 已统一内部、联合、立法和选举投票，但核心仍按具体 stage 维护多份派发分支；生产文件超过 800 行，清理状态机固定读取 `StorageMap::iter().next()`，真实 FRAME benchmark 也尚未覆盖五个投票 crate。项目将在改造完成后重新创世，因此继续保留开发期旧布局迁移会制造无用复杂度和错误入口。

## 决策

- 先做不改变行为和 metadata 的物理拆分，再改存储和调度，最后基于稳定代码生成 benchmark。
- 核心引擎通过 runtime 注册的递归 Track tuple 派发 timeout、mode 数据清理和 mode 终态副作用，不再匹配每个具体 stage。
- 投票判定只写状态并登记业务执行队列；业务执行、自动终结和清理分别使用条数与 weight 双重预算。
- 清理使用两级 FIFO：固定保留期的延迟 FIFO 到期后进入就绪 FIFO；每次只删除一个有界 chunk，未完成项重新入队，避免大型公投阻塞其他提案。
- 公民人口与创建时资格只由 `citizen-identity` 的 snapshot_id 提供，联合公投、立法公投和 Popular 选举只绑定引用；机构互选管理员快照只由 admins provider 提供，Track 不复制资格真源。
- 身份模块以全局资格 revision 和每账户不可变版本历史支持 `can_vote_at(snapshot_id)`；提案创建后的身份变化不得改变既有提案。
- 自动业务执行的回调错误、Ignored、结果应用错误和 Track 后处理错误统一递增 attempts、指数退避并在上限后 dead-letter；终态副作用使用独立队列，禁止重新执行业务动作。
- 宪法阈值继续引用 `primitives::constitution`，Track 只表达流程和派发，不复制宪法数学。
- 相关 pallet 直接以最终存储布局和 `StorageVersion = 1` 重新创世；删除旧 storage alias、升级翻译和兼容测试，不保留双读或影子流程。
- 生产权重必须由 FRAME benchmark 生成；runtime-upgrade 业务执行仍显式叠加 `SystemWeightInfo::set_code()` 的最重成本。

当前 Runtime 将自动终结、异步执行、历史清理的独立预算分别配置为最大区块权重的 `1/4`、`1/4`、`1/8`。正式 benchmark 后复核总占比为 62.5%，每条管线在 60 秒最大计算区块内均能容纳至少一个最重任务，因此保持该预算。

## 影响

- 新增普通投票模式时，只需在 VotingEngine 边界增加 Track 实现并在 runtime 注册，不需要修改核心 timeout/cleanup 分支，也不需要创建新的 pallet。
- 特殊投票仍可由现有 sub-pallet 承载自己的快照、票据和计票，但必须实现统一 Track handler。
- Popular 选举不再把完整选区人口塞入 `BoundedVec`；Mutual 互选仍固化完整 admins 快照，因此大选区规模与机构管理员规模不再混用同一个上限。
- 清理吞吐按实际 weight 控制，proposal 之间公平推进；任何单项错误或大规模票据都不能饿死其他维护管线。
- 重新创世会生成新的 genesis hash；正式冻结后升级仍必须走链上 `setCode`，本 ADR 不改变创世冻结规则。

## 备选方案

- 保留 stage 巨型 `match`：新增模式仍需修改核心多处分支，拒绝。
- 把所有投票强行统一成同一计票算法：会破坏选举排名、立法会签和联合公投的业务边界，拒绝。
- 继续用 `StorageMap::iter().next()` 清理：大型提案可以长期头部阻塞，拒绝。
- 为开发期旧链保留 v2/v3/v4 migration：正式链尚未创世，违反目标态一次性改造，拒绝。

## 落地与验收

1. 四个超限生产文件已物理拆分，生产文件均不超过 800 行，空模块已删除或承载真实职责。
2. 公平清理队列、递归 Track handler、异步执行和三条独立 weight 预算已落地。
3. 五个投票 crate 共 19 个 FRAME benchmark 已按 `steps=50/repeat=20/WASM compiled` 生成生产权重；Track timeout 与 cleanup 改为动态计账。
4. 原生 LLVM 可执行业务代码行覆盖率为 81.80%，workspace 全量测试、`no_std`、`runtime-benchmarks`、`try-runtime` 和 release WASM 构建通过。
5. 最终源码的 `citizenchain-fresh` 全新创世真实启动通过，genesis hash 为 `0xd81962210c603a4a0f078b2cc022bac3daab344cd7dce8c6fc3501973d1552ab`，metadata RPC 响应 418,806 字节，runtime spec、system、state 与受影响 StorageVersion 均为 1。
6. 正式创世发布仍须统一烘焙冻结 chainspec、替换 bootnode 所在网络的 genesis，并按发布门禁验收客户端签名版本；本任务未推送或部署。
