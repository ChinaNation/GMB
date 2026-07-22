# resolution-issuance 技术文档

## 2026-07-21 机构岗位权限协议收口

- `propose_issuance` 的 SCALE 参数固定为 `actor_cid_number + proposer_role_code + reason + total_amount + allocations`；岗位码不得由业务模块根据机构或管理员身份代填。
- 业务模块使用签名账户、机构 CID 和显式岗位码构造完整 `RoleSubject`，校验 `res-iss/0 + Propose`。当前权限目录只授予 NRC 与 43 个 PRC 的 `COMMITTEE_MEMBER`，传入其他岗位码直接拒绝。
- 显式岗位码同步写入 `IssuanceProposalData`、`VotePlan.proposer_subject` 和 `ResolutionIssuanceProposed` 事件，形成可复核的提案发起岗位审计链。
- 当前链尚未创世，不保留无岗位码旧载荷、旧 ProposalData 解码或 migration；runtime 版本与 storage version 继续为 `0`。

## 2026-07-14 投票与执行绑定

- 决议发行继续只由联合投票引擎推进；业务模块不实现计票或公投。
- 仅 NRC 和 43 个 PRC 的 `COMMITTEE_MEMBER / 委员` 岗位有效任职账户可发起；联合选民由 NRC/PRC 委员与 43 PRB `DIRECTOR / 董事` 组成，PRB 只投票。
- 回调执行必须匹配本模块 `ProposalOwner`、联合 proposal kind、`STAGE_JOINT` 或 `STAGE_REFERENDUM`、通过状态和原始发行摘要。
- 联合机构直接一致通过与转入联合公投后通过都执行同一份绑定发行决议；公投阶段不得因 stage 不同被误判为异常执行失败。

## 1. 模块定位

`resolution-issuance` 是 CitizenChain 的决议发行完整流程 pallet，统一承载：

- 创建决议发行联合投票提案
- 将业务数据写入 `votingengine::ProposalData`
- 只接受 `votingengine` 终态转换事务内的回调执行发行
- 记录永久防重放标记与短期审计标记
- 提供暂停开关与短期执行记录清理入口

本模块位于：

- 代码：`citizenchain/runtime/issuance/resolution-issuance/`
- Runtime pallet：`ResolutionIssuance`
- Runtime pallet index：`8`
- ProposalData 模块标签：`b"res-iss"`

## 2. 设计原则

- 决议发行是一个完整业务功能，提案、投票回调和发行执行必须在同一 pallet 内闭环。
- 发行执行不再作为独立外部模块暴露，只有 `votingengine` 的 `JointVoteResultCallback::on_joint_vote_finalized` 回调路径可以触发实际铸币。
- `propose_issuance` 保持 call index `0`。
- `finalize_joint_vote` 手工 extrinsic 已删除，call index `1` 保持空缺，避免 Root 或误配 origin 绕过投票引擎。
- 提案核心数据、owner、业务 data 和投票凭证清理由 `votingengine` 终态清理队列统一处理，本模块不再持有独立清理入口。
- 本模块只负责决议发行业务提案和执行回调，不接收、不生成、不校验人口快照、联合签名、地区或签名管理员公钥；这些全部属于 `votingengine` 的联合投票流程。
- 旧 index `7` 不再注册任何 pallet。
- 当前正式链尚未创世，直接使用终态 storage 和 `StorageVersion = 0`；不保留 `migration.rs`、运行期兜底写入、旧 storage 双读或兼容分支。正式创世后的真实升级必须另行设计并审查迁移。

## 3. 文件结构

```text
citizenchain/runtime/issuance/resolution-issuance/
  Cargo.toml
  src/
    lib.rs
    proposal.rs
    execution.rs
    validation.rs
    benchmarks.rs
    weights.rs
    tests/
      mod.rs
      cases.rs
```

- `lib.rs`：FRAME pallet 壳，包含 `Config`、storage、event、error、genesis、hooks 和 extrinsics。
- `proposal.rs`：提案创建、ProposalData 编解码、`owns_proposal` 与联合投票结果处理。
- `execution.rs`：发行执行、防重放、累计发行、暂停与短期执行记录清理。
- `validation.rs`：收款名单、金额、分配明细和 CHINA_CB 地址校验。
- `benchmarks.rs` / `weights.rs`：覆盖全部公开 call；新增岗位码不增加 storage 读写，benchmark 调用同步最终参数，现有正式权重保持有效。
- `tests/`：提案、显式岗位授权、回调、执行、暂停、清理和事件来源回归测试。

## 4. Runtime 接口

公开 call：

| call index | extrinsic | 说明 |
|---:|---|---|
| 0 | `propose_issuance(actor_cid_number, proposer_role_code, reason, total_amount, allocations)` | 创建决议发行联合投票提案 |
| 1 | 空缺 | 原手工 `finalize_joint_vote` 已删除，终结只能经 votingengine 回调 |
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

1. 签名账户调用 `propose_issuance`，显式提交操作机构 CID、发起岗位码和决议发行业务数据。
2. 模块校验理由、总金额、分配明细和合法收款名单，并要求签名账户对 `RoleSubject(actor_cid_number, proposer_role_code)` 拥有决议发行 `Propose` 权限；当前只有 NRC/PRC 委员岗位具备该权限。
3. 模块构造固定联合 `VotePlan`：NRC + 43 PRC `COMMITTEE_MEMBER` 为可发起/可投票主体，43 PRB `DIRECTOR` 为只投票主体，`business_object_hash` 绑定完整发行业务数据摘要。
4. 模块通过 `JointVoteEngine::create_joint_proposal_with_data` 创建联合投票提案，并在同一事务中写入 plan、owner/data/meta 和岗位有效任职快照；人口快照准备和消费由投票引擎完成。
5. `ProposalData` 内容为 `MODULE_TAG + IssuanceProposalData`；业务数据显式保存 `actor_cid_number + proposer_role_code + proposer`，不得回退为硬编码委员岗位。
6. 投票引擎终结联合投票后，在自身状态转换事务内回调 `ResolutionIssuance`。
7. 如果投票通过，模块在同一事务内执行发行、记录防重放并递减计数；提案数据由 votingengine 终态清理队列统一延迟清理。
8. 如果投票否决，模块只递减计数；提案数据由 votingengine 终态清理队列统一延迟清理。
9. 如果投票通过且执行成功，模块发出执行事件，并返回 `ProposalExecutionOutcome::Executed`。
10. 如果投票通过但执行失败，模块发出失败事件，并返回 `ProposalExecutionOutcome::FatalFailed`。
11. allocation 结构性校验集中在 `validation.rs`：收款人集合、唯一性、单笔非零与总额匹配由共享校验统一负责。
12. `execution.rs` 在共享校验后只保留执行期专属检查：暂停、防重放、理由长度、Existential Deposit、单笔 cap、累计 cap 和实际入账结果。

## 7. 安全边界

- 分配明细的收款人集合必须与 `AllowedRecipients` 完全一致。
- 分配明细的结构性校验只维护一份共享实现，避免提案层与执行层出现校验漂移。
- `AllowedRecipients` 只能新增，不能移除已有账户。
- 存在 Voting 提案时禁止更新 `AllowedRecipients`。
- `EverExecuted` 是永久防重放标记，`clear_executed` 不得清理它。
- `Paused=true` 时拒绝新的发行执行。
- 发行执行使用 storage layer，任一收款失败都会整体回滚。
- `apply_joint_vote_result` 会校验 `CallbackExecutionScopes`、联合提案类型和 votingengine 状态：
  - `approved=true` 时只接受 `STATUS_PASSED`。
  - `approved=false` 时只接受 `STATUS_REJECTED`。
  - 已进入 `STATUS_EXECUTED` / `STATUS_EXECUTION_FAILED` 等终态的提案不得二次回调。
- 发起授权与投票资格分层：业务模块校验调用者提交的完整 `RoleSubject` 的 `Propose` 权限；投票引擎只根据已绑定 `VotePlan` 和岗位有效任职快照接受票据。

## 8. 联动影响

- `citizenchain/onchina/src/indexer/event_parser.rs` 需要按 `ResolutionIssuance / ResolutionIssuanceExecuted` 解析治理发行事件。
- `citizenwallet/lib/signer/payload_decoder.dart` 按 CID 后紧随岗位码的最终 SCALE 布局严格解码；无岗位码旧载荷直接拒绝。
- `citizenapp` 与节点桌面端继续可通过 `b"res-iss"` 识别联合提案类型。

## 9. 验证命令

```bash
cd /Users/rhett/GMB/citizenchain
cargo test -p resolution-issuance
WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain
WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain --features runtime-benchmarks
./target/release/citizenchain benchmark pallet --chain=citizenchain --pallet=resolution_issuance --extrinsic='*' --steps=50 --repeat=20 --output=runtime/issuance/resolution-issuance/src/weights.rs
```

## 10. 权重状态

- `benchmarks.rs` 已覆盖 `set_allowed_recipients`、`propose_issuance`、`clear_executed`、`set_paused` 四个公开入口；`propose_issuance` benchmark 只携带显式机构岗位主体和发行内容，不承载人口快照或投票流程。
- Cargo feature：`runtime-benchmarks` 会向 `pallet-balances` 与 `votingengine` 传播；`primitives` 当前不暴露 benchmark feature，不在传播列表中。
- benchmark 环境先构建真实创世机构，再写入 NRC/PRC 委员与 PRB 董事岗位、任职和固定权限，不再用 admins 伪装业务授权。
- `propose_issuance` 已用当前 benchmark runtime WASM、50 steps / 20 repeats 重算：368 reads / 280 writes，参考时间 1.977 s，真实计入 87 个岗位快照、87 个 CID 有效选民快照和 `ProposalVotePlans`。

## 11. 岗位授权回归

- 目标模块当前 20 项测试通过。
- 普通 staff 即使属于 NRC admins，只要没有 NRC 委员岗位有效任职和 `Propose` 权限，仍不得创建决议发行提案。
- 有权管理员显式传入其他岗位码同样被拒绝；无岗位码旧 SCALE 载荷不兼容。
- `VotePlan` 回归检查 44 个委员主体、43 个 PRB 董事主体、联合引擎和业务对象哈希。
