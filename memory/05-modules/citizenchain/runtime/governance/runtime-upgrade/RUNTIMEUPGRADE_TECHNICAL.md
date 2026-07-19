# runtime-upgrade 技术文档

## 0. 功能需求
### 0.1 模块职责
`runtime-upgrade` 负责把"Runtime wasm 升级"包装成一个受治理约束的链上流程，核心要求是：
- 仅允许 NRC 和 43 个 PRC 的 `COMMITTEE_MEMBER / 委员` 岗位有效任职账户发起升级提案，仅属于 admins 不构成授权。
- 升级提案必须先经过 `votingengine` 的联合投票。
- 联合阶段由 `VotePlan` 固定绑定 NRC + 43 PRC 委员岗位和 43 PRB `DIRECTOR / 董事` 岗位；有效任职账户用个人钱包直接上链投票，链上按机构阈值形成机构结果。PRB 董事只有投票权，没有提案权。
- 联合投票通过后才允许执行 `set_code`。
- 开发期直升通道只允许国家储委会管理员使用，并且必须受 `DeveloperUpgradeEnabled` 开关约束。
- 投票结果、执行结果必须在链上可追踪。

### 0.2 提案创建需求
- 提案必须携带非空升级理由 `reason`。
- 提案必须携带非空 wasm `code`。
- 创建提案时同步在 `votingengine` 创建联合投票，使用投票引擎统一分配的 `proposal_id`（本模块不维护独立 ID）。
- 本模块不接收、不生成、不校验人口快照、联合签名、投票资格和计票数据；这些都属于 `votingengine`。

### 0.3 联合投票回调需求
- 联合投票拒绝时，投票引擎状态保持 `STATUS_REJECTED`。
- 联合投票通过时，模块必须尝试执行 runtime code。
- 联合投票结束后，投票引擎侧状态保持真实业务结果：执行成功写为 `STATUS_EXECUTED`，否决保持 `STATUS_REJECTED`，执行失败写为 `STATUS_EXECUTION_FAILED`。
- 回调直接使用投票引擎的 `proposal_id`，无需映射反查。
- 回调还必须校验 callback scope、`ProposalOwner`、联合 kind、`STAGE_JOINT/STAGE_REFERENDUM`，并复算 `ProposalObject` 中 runtime code 的哈希与提案摘要一致；任何一项不符都不得执行 `set_code`。

### 0.4 执行失败处理
- 若联合投票已通过但 `set_code` 执行失败，投票引擎状态进入 `STATUS_EXECUTION_FAILED`。
- runtime wasm 不再内嵌在摘要结构里，而是统一存入 `votingengine::ProposalObject`。
- 执行成功、拒绝、执行失败后，wasm 对象继续保留到投票引擎 90 天延迟清理统一删除，不由业务模块手工删除。

### 0.5 可审计与运维需求
- 需要区分以下事件：提案创建、联合投票终结、升级执行成功、升级执行失败。
- 投票引擎侧状态机：
  - `VOTING → PASSED → EXECUTED`（投票通过且执行成功）
  - `VOTING → REJECTED`（投票拒绝）
  - `VOTING → PASSED → EXECUTION_FAILED`（投票通过但执行失败）
- 所有终态均为不可逆（无重试、无取消）。

## 1. 模块定位
`runtime-upgrade` 是"协议升级治理编排模块"，负责：
- 接收 NRC/PRC 委员岗位有效任职账户提交的 wasm 升级提案；
- 调用 `votingengine` 创建联合投票；
- 在联合投票回调后执行 `set_code`；
- 摘要数据存储在 `votingengine` 的 `ProposalData`；
- 原始 wasm 对象存储在 `votingengine` 的 `ProposalObject`；
- 本模块零本地存储。

代码位置：
- `runtime/governance/runtime-upgrade/src/lib.rs`
- `node/src/governance/runtime_upgrade/`
- `node/frontend/governance/runtime-upgrade/`

命名说明：
- 2026-04-29 起，本模块统一使用 `runtime-upgrade` / `runtime_upgrade` / `RuntimeUpgrade`。
- 模块位于 `citizenchain/runtime/governance/runtime-upgrade/`。
- `pallet_index = 12`、call index 与 `MODULE_TAG = b"rt-upg"` 保持不变。

节点侧边界：
- node 后端 `runtime_upgrade` 只负责读取 wasm、构建协议升级 call data、生成签名请求、提交签名交易。
- node 前端 `runtime-upgrade` 只负责协议升级/开发升级页面交互和签名流程。
- node 的 `runtime_upgrade` 不获取人口快照、不接收联合签名上下文、不拥有投票引擎状态、不展示投票终态。
- 协议升级提案详情展示真实状态时必须以 `VotingEngine::Proposals.status` 为准，`runtime-upgrade` 摘要里不保存业务状态字段。

citizenapp / citizenwallet 边界：
- citizenapp 的 `governance/runtime-upgrade` 不发起协议升级提案，不选择 WASM，不获取人口快照，不提交 `propose_runtime_upgrade`。
- citizenapp 只展示协议升级介绍、协议升级提案详情，并保留现有提案详情页投票入口。
- citizenwallet 公民钱包不恢复 runtime-upgrade SCALE decoder；大 WASM 交易继续走哈希直签例外，由用户核对显示字段中的代码哈希。

## 2. 运行时接线
Runtime 配置位置：
- `runtime/src/configs/mod.rs`

当前接线：
- `ProposeOrigin = EnsureJointProposer`
- `InstitutionRoleAuthorization = PublicManage`
- `DeveloperUpgradeOrigin = EnsureNrcAdmin`
- `JointVoteEngine = VotingEngine`
- `RuntimeCodeExecutor = RuntimeSetCodeExecutor`
- `MaxReasonLen = RuntimeUpgradeMaxReasonLen`（1024）
- `MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize`（5 * 1024 * 1024）
- `VotingEngine::MaxProposalDataLen = 100 * 1024`
- `VotingEngine::MaxProposalObjectLen = 10 * 1024 * 1024`
- `WeightInfo = runtime_upgrade::weights::SubstrateWeight<Runtime>`

说明：
- `finalize_joint_vote` 手工 extrinsic 已删除，call index `1` 保持空缺。
- 正常生产路径只能由投票引擎通过 `JointVoteResultCallback` 自动回调本模块，避免 Root 手工回放形成第二条执行入口。

## 3. 核心数据结构
### 3.1 Proposal（摘要，序列化存入 votingengine ProposalData）
- `proposer: AccountId`：提案发起人（NRC 或 PRC 委员岗位的有效任职账户）
- `reason: BoundedVec<u8, MaxReasonLen>`：升级理由
- `code_hash: Hash`：升级 code 哈希，便于事件与链下审计对齐

说明：
- `Proposal` 只保存业务展示所需摘要，不保存投票状态。
- 协议升级真实状态只能读取 `votingengine::Proposals.status`。

### 3.2 对象层数据（统一存入 votingengine ProposalObject）
- `kind = 1`：表示 runtime wasm 对象
- `object_len`：wasm 字节长度
- `object_hash`：对象哈希
- `object bytes`：原始 wasm 字节（对象层上限 10MB，业务自身继续限制 5MB）

### 3.3 模块标识
- `MODULE_TAG = b"rt-upg"`：存入 ProposalData 的前缀，用于区分不同业务模块，防止跨模块误解码。

## 4. 存储模型
本模块只保留一项本地审计，其余提案数据、投票数据、元数据均存储在 `votingengine`：
- `LastRuntimeUpgradeAudit`：最近一次成功执行的 runtime 升级审计，记录执行路径、code hash、
  旧/新 PoW 参数 hash、执行高度和参数激活高度，供 NodeGuard 验证 `:code` 与 PoW 参数原子绑定。
- `ProposalData`：存放 `MODULE_TAG + Proposal<T>` 摘要的 SCALE 编码
- `ProposalObjectMeta`：存放 runtime wasm 的对象元数据（kind / len / hash）
- `ProposalObject`：存放 runtime wasm 原始字节
- `ProposalMeta`：存放提案创建时间
- `Proposals`：投票引擎核心提案表（状态、阶段、截止区块等）
- `ProposalVotePlans`：一次性绑定协议升级动作、提案主体、87 个投票岗位主体、联合引擎和 runtime WASM 对象哈希
- `VoterSnapshot` / `EffectiveVoterSnapshot`：分别保存岗位有效任职快照和按 CID 合并去重的有效选民快照

## 5. 外部接口
### 5.1 `propose_runtime_upgrade`（call index = 0）
流程：
1. 校验 `ProposeOrigin`（`EnsureJointProposer`），再用 `InstitutionRoleAuthorization` 校验签名账户对 `RoleSubject(actor_cid_number, COMMITTEE_MEMBER)` 拥有协议升级 `Propose` 权限。
2. 校验 `reason` 与 `code` 非空，并校验 `new_pow_params` 的参数/算法版本合法。
3. 计算 `code_hash`、当前 `ActiveParams` hash 与新 PoW 参数 hash，构造摘要 `Proposal`
   并加 `MODULE_TAG` 序列化。
4. 构造固定联合 `VotePlan`：NRC + 43 PRC `COMMITTEE_MEMBER` 为可发起/可投票主体，43 PRB `DIRECTOR` 为只投票主体，`business_object_hash` 绑定 runtime WASM 对象哈希。
5. 调用 `JointVoteEngine::create_joint_proposal_with_data_and_object` 创建联合投票，并在同一事务中写入 plan、owner/data/meta、岗位选民快照和 runtime wasm 对象。
6. 发出 `RuntimeUpgradeProposed` 事件。

边界：
- 该接口接收 `origin / reason / code / new_pow_params`。
- PoW 参数只能随 runtime code 一起表决；`CurrentDifficulty` 不进入提案参数，仍由算法推进。
- 人口快照、联合签名、投票资格、计票与终态推进均由投票引擎内部流程负责。

### 5.2 call index 1 空缺
原 `finalize_joint_vote` 手工入口已删除。该位置保持空缺，不再注册任何 extrinsic。

协议升级联合投票终结流程只允许从 `JointVoteResultCallback::on_joint_vote_finalized` 进入：
1. 从 `ProposalData` 加载提案摘要，并要求投票引擎 `Proposals` 必须存在。
2. 要求投票引擎状态与本次回调方向一致：通过为 `STATUS_PASSED`，否决为 `STATUS_REJECTED`。
3. 若 `approved=false`：
   - 不改写业务摘要
   - 返回 `ProposalExecutionOutcome::Executed`，投票引擎保持 `STATUS_REJECTED`
   - 发出 `JointVoteFinalized`
4. 若 `approved=true`：
   - 从 `ProposalObject` 加载 runtime wasm
   - 尝试执行 `RuntimeCodeExecutor::execute_runtime_code`
   - 成功：回调返回 `ProposalExecutionOutcome::Executed`
   - 失败：回调返回 `ProposalExecutionOutcome::FatalFailed`
   - 发出 `JointVoteFinalized` + 执行成功或失败事件
5. wasm 对象不由本模块手工删除，统一交由投票引擎 90 天延迟清理。

### 5.3 `developer_direct_upgrade`（call index = 2）
说明：
- 开发期快捷通道：仅国家储委会管理员直接 `set_code`，不走联合投票。
- 仅在 `genesis-pallet` 的 `DeveloperUpgradeEnabled` 为 `true` 时可用。
- 链进入运行期后此调用永久失效，升级必须走 `propose_runtime_upgrade` 联合投票。

流程：
1. 交易载荷显式携带 `actor_cid_number`，不得用主账户或本地登录态代替机构身份。
2. 校验 `DeveloperUpgradeOrigin` 后，要求 actor CID 的机构码为 NRC，且外层签名者属于 `AdminAccounts[actor_cid_number].admins`。
3. 校验 `DeveloperUpgradeCheck::is_enabled()`，关闭则拒绝（`DeveloperUpgradeDisabled`）。
4. 校验 `code` 非空。
5. 计算 `code_hash`，调用 `RuntimeCodeExecutor::execute_runtime_code`，同样原子暂存 PoW 参数并写审计。
6. 发出 `DeveloperDirectUpgradeExecuted` 事件。

费用：开发直升是国家储委会机构操作，由该 `actor_cid_number` 的唯一费用账户支付 0.1 元；管理员钱包只提供外层签名，不允许作为回落付款人。

权重：使用 `frame_system::set_code()` 的系统权重。

版本要求：
- `developer_direct_upgrade` 最终通过 `System.set_code` 写入新 runtime code，系统会拒绝 `spec_version` 小于或等于链上当前版本的 WASM，错误表现为 `System::SpecVersionNeedsToIncrease`
- WASM push 自动 CI 不查询链上版本；只有手动 `Run workflow` 才使用 `GMB_SSH_KEY` 登录服务器后访问本机 `127.0.0.1:9944` 读取链上 `state_getRuntimeVersion.specVersion`，源码版本不足时只在 CI 工作区临时提升到 `链上版本 + 1` 再编译 artifact
- CI 不把临时提升后的 `spec_version` 自动提交回仓库；源码中的版本号仍用于记录开发者认可的 runtime 版本基线

### 5.4 投票引擎状态协同

当前实现与 `votingengine` 的协作关系如下：

- 联合投票通过时，投票引擎先按通用路径把提案写成 `STATUS_PASSED`，再在同一事务中执行本模块回调
- 联合投票拒绝时，投票引擎保持 `STATUS_REJECTED`
- runtime code 执行成功时，本模块返回 `ProposalExecutionOutcome::Executed`，投票引擎写入执行成功终态
- runtime code 执行失败时，本模块返回 `ProposalExecutionOutcome::FatalFailed`，投票引擎写入执行失败终态

原因：本模块的执行逻辑运行在投票引擎 `set_status_and_emit` 的回调事务内。业务回调只返回统一执行结果，不回写任何业务状态字段；最终状态、`ProposalFinalized`、清理登记和互斥锁释放由投票引擎外层统一执行一次。

提案状态流转（投票引擎侧）：
- `VOTING → PASSED → EXECUTED`（联合投票通过且 runtime code 执行成功）
- `VOTING → REJECTED`（联合投票拒绝）
- `VOTING → PASSED → EXECUTION_FAILED`（联合投票通过，但 runtime code 执行失败）

说明：
- 节点 UI / RPC 查询层如果需要面向用户展示真实升级结果，应读取 `VotingEngine::Proposals.status`；`ProposalData` 只用于展示 proposer、reason、code_hash 等摘要信息。
  - `VotingEngine::STATUS_VOTING` / `STATUS_PASSED` → 投票中或执行待重试态
  - `VotingEngine::STATUS_REJECTED` → 已否决
  - `VotingEngine::STATUS_EXECUTED` → 已执行
  - `VotingEngine::STATUS_EXECUTION_FAILED` → 执行失败

## 6. 回调路径
`JointVoteResultCallback::on_joint_vote_finalized`：
1. 接收投票引擎统一的 `proposal_id`（无需映射反查）
2. 调用 `apply_joint_vote_result(proposal_id, approved)`
3. 返回 `ProposalExecutionOutcome`，由投票引擎统一推进状态

Runtime 层的 `RuntimeJointVoteResultCallback` 负责路由：先尝试 `resolution-issuance`，再尝试 `runtime-upgrade`。

## 7. 安全审查结论
### 7.1 已修复风险：执行失败误记为 Passed
旧实现中，联合投票通过后会先把提案写成 `Passed` 并清空 `code`，再尝试执行 `set_code`。如果执行失败：
- 链上状态仍显示 `Passed`
- 原始 code 已丢失

现已修复：
- 先执行，根据结果返回 `ProposalExecutionOutcome`
- 执行成功由投票引擎进入 `STATUS_EXECUTED`
- 执行失败由投票引擎进入 `STATUS_EXECUTION_FAILED`
- 业务摘要不再保存业务状态字段

### 7.2 已修复风险：大 wasm 直接塞入 ProposalData 导致提案创建失败
旧实现中整份 runtime wasm 会直接编码进 `ProposalData`，而投票引擎通用摘要存储无法承载 MB 级对象，runtime 升级提案会在创建阶段触发 `ProposalDataTooLarge`。

现已修复：
- `ProposalData` 只存摘要
- wasm 改为统一写入投票引擎对象层 `ProposalObject`
- 创建提案时通过 `create_joint_proposal_with_data_and_object` 一次性原子写入，后续不暴露对象覆写入口
- 投票引擎摘要上限提升到 `100KB`
- 投票引擎对象层上限提升到 `10MB`
- runtime 升级业务自身 `MaxRuntimeCodeSize` 继续保持 `5MB`

### 7.3 已修复风险：投票引擎状态与业务执行结果脱节
旧实现/旧文档把投票引擎终态过度抽象成统一的 `STATUS_EXECUTED`，无法准确表达“联合投票已通过，但 runtime code 执行失败”的差异，也容易让查询层误判真实业务结果。

现已修复：
- 联合投票通过且执行成功时写入 `STATUS_EXECUTED`
- 联合投票拒绝时保持 `STATUS_REJECTED`
- 联合投票通过但执行失败时写入 `STATUS_EXECUTION_FAILED`
- 查询层文档已明确：展示真实升级结果时以 votingengine 的 `Proposal.status` 为准，业务摘要只用于展示 proposer/reason/code_hash

### 7.4 已修复风险：benchmark 与实际逻辑不一致
旧版 benchmark 存在偏差。现已修复：
- `propose_runtime_upgrade` benchmark 改为真实 extrinsic
- `propose_runtime_upgrade` benchmark 已删除人口快照、联合签名、省份和签名管理员公钥参数。
- benchmark 环境先构建真实创世机构，再写入 NRC/PRC 委员与 PRB 董事岗位、任职和固定权限，不再用 admins 伪装业务授权。
- 权重已用当前 benchmark runtime WASM、50 steps / 20 repeats 重算：367 reads / 281 writes，参考时间 12.483 s，并真实计入 87 个岗位快照、87 个 CID 有效选民快照与 `ProposalVotePlans`。
- `finalize_joint_vote` benchmark 与权重项已删除，终结执行成本由 `votingengine` 的联合投票终态回调路径覆盖。

### 7.5 已收口入口
1. `finalize_joint_vote` 手工 Root 入口已删除，只保留 votingengine callback。

## 8. 中文注释覆盖重点
本模块当前已在以下关键位置补充注释：
- `RuntimeCodeExecutor` 职责边界
- `propose_runtime_upgrade` 与 votingengine 的职责边界
- 联合投票通过后的执行/失败分叉
- `ProposalExecutionOutcome::Executed / FatalFailed` 与投票引擎状态的映射原因
- `on_joint_vote_finalized` 回调入口

## 9. 测试覆盖
已覆盖（当前单测与框架完整性检查共 20 个测试）：
- NRC 和 PRC 委员岗位有效任职账户可发起提案，普通 staff 即使属于 admins 也被拒绝
- `VotePlan` 精确绑定 44 个委员主体、43 个董事主体、联合引擎与 runtime WASM 对象哈希
- 提案摘要与对象数据正确分别存入 votingengine
- 联合投票拒绝时保持 votingengine `STATUS_REJECTED`（含 wasm 对象保留到统一清理）
- 联合投票通过并成功执行进入 votingengine `STATUS_EXECUTED`
- 联合投票通过但执行失败进入 votingengine `STATUS_EXECUTION_FAILED`
- 联合投票通过成功时投票引擎状态进入 `STATUS_EXECUTED`
- `owns_proposal` 能正确识别本模块提案
- 已终结的提案不可重复终结（`ProposalNotVoting`）
- 不存在的提案终结失败（`ProposalNotFound`）
- 开发者直升：国家储委会管理员可直接升级
- 开发者直升：省储委会管理员拒绝（`BadOrigin`）
- 开发者直升：开关关闭时拒绝（`DeveloperUpgradeDisabled`）
- 开发者直升：非国家储委会管理员拒绝（`BadOrigin`）
- 开发者直升：空 code 拒绝（`EmptyRuntimeCode`）
- GenesisConfig 构建成功
- Runtime 完整性检查

Runtime 集成测试：
- 不存在的 proposal_id 回调返回错误
- 回调正确路由到本模块并执行拒绝流程

本地验证：
- 2026-05-10 `cargo test --manifest-path citizenchain/Cargo.toml -p runtime-upgrade --lib`：通过，17 passed。
- 2026-05-10 `cargo check --manifest-path citizenchain/Cargo.toml -p runtime-upgrade`：通过。
- 已执行格式整理与残留扫描。

## 10. 文件索引
- 模块代码：`src/lib.rs`
- Benchmark：`src/benchmarks.rs`
- 权重：`src/weights.rs`
- 技术文档：`RUNTIMEUPGRADE_TECHNICAL.md`
