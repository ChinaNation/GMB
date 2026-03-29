# ADMINS_ORIGIN_GOV Technical Notes

## 0. 功能需求
### 0.1 模块职责
`admins-origin-gov` 负责把"同机构管理员替换"包装成一个受治理约束的链上流程，要求：
- 只允许做替换，不允许增删管理员人数。
- 只允许本机构管理员发起与投票。
- 内部投票由 `voting-engine-system` 统一承载。

### 0.2 提案创建需求
- `org` 必须与 `institution` 的真实归属严格一致。
- `old_admin` 必须存在于当前管理员列表中。
- `new_admin` 不能已在当前管理员列表中。

### 0.3 执行与失败恢复需求
- 提案投票通过后应自动尝试执行管理员替换。
- 自动执行失败不能回滚已通过的投票结果。
- 已通过但执行失败的提案可通过 `execute_admin_replacement` 手动重试。

---

## 1. 模块定位
`admins-origin-gov` 是"机构管理员替换治理"pallet，只负责管理员替换动作本身，不负责投票引擎实现。

核心职责：
- 发起"同机构内管理员替换"提案。
- 将投票委托给 `voting-engine-system` 的内部投票。
- 在投票通过后执行管理员替换（可自动尝试，也可手动触发）。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-origin-gov/src/lib.rs`

---

## 2. 上下游关系与运行时接线
上游常量来源：
- `/Users/rhett/GMB/citizenchain/runtime/primitives/src/count_const.rs`
  - 管理员数量：`NRC=19`, `PRC=9`, `PRB=9`
  - 内部投票阈值：`NRC=13`, `PRC=6`, `PRB=6`

投票引擎：
- `/Users/rhett/GMB/citizenchain/runtime/governance/voting-engine-system/src/lib.rs`
  - 状态：`STATUS_VOTING=0`, `STATUS_PASSED=1`, `STATUS_REJECTED=2`, `STATUS_EXECUTED=3`, `STATUS_EXECUTION_FAILED=4`
  - 状态设置：`Pallet::set_status_and_emit`（执行成功后标记 `STATUS_EXECUTED`）
- 本模块使用：
  - `InternalVoteEngine::create_internal_proposal`
  - `InternalVoteEngine::cast_internal_vote`
  - `Pallet::proposals`
  - `Pallet::store_proposal_data` / `get_proposal_data`
  - `Pallet::store_proposal_meta` / `set_proposal_passed`
  - `Pallet::set_status_and_emit`

Runtime 配置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `MaxAdminsPerInstitution = 32`
  - `type InternalVoteEngine = VotingEngineSystem`

---

## 3. 存储模型
1. `CurrentAdmins: Map<InstitutionPalletId, BoundedVec<AccountId, MaxAdminsPerInstitution>>`
   - hasher: `Blake2_128Concat`
   - 机构当前管理员名单（创世后仅信任链上状态）

2. 业务数据存储位置
   - 提案业务数据（`AdminReplacementAction`）编码后存入投票引擎的 `ProposalData`。
   - 提案元数据（创建时间、通过时间）存入投票引擎的 `ProposalMetadata`。
   - 提案状态（VOTING/PASSED/REJECTED/EXECUTED）存入投票引擎的 `Proposals`。
   - 本模块不维护额外的提案索引或映射。

版本信息：
- `STORAGE_VERSION = 1`

---

### 模块标识
- `MODULE_TAG = b"adm-rep"`：存入 ProposalData 的前缀，用于区分不同业务模块，防止跨模块误解码。

## 4. 核心数据结构
```rust
pub struct AdminReplacementAction<AccountId> {
    pub institution: InstitutionPalletId,
    pub old_admin: AccountId,
    pub new_admin: AccountId,
}
```
- 编码后存入 `ProposalData`，通过 `get_proposal_data` 读取并解码。

---

## 5. 外部接口（Calls）
### 5.1 `propose_admin_replacement`（call index = 0）
入参：`org`, `institution`, `old_admin`, `new_admin`

流程：
1. 校验 `institution → org` 映射一致。
2. 校验 `who` 为该机构管理员。
3. 校验 `old_admin` 在名单、`new_admin` 不在名单。
4. 调投票引擎创建内部提案，获得真实 `proposal_id`。
5. 将 `AdminReplacementAction` 编码存入 ProposalData。
6. 记录提案元数据（创建时间）。
7. 发 `AdminReplacementProposed` 事件。

### 5.2 `vote_admin_replacement`（call index = 1）
流程：
1. 从 ProposalData 解码 `AdminReplacementAction`，校验未执行。
2. 校验投票人是目标机构管理员。
3. 通过 `InternalVoteEngine::cast_internal_vote` 代理调用投票引擎计票。
4. 仅当 `approve=true` 且投票引擎状态达到 `STATUS_PASSED` 时尝试自动执行。
5. 自动执行失败不回滚投票；发出 `AdminReplacementExecutionFailed` 事件。
6. 首次达到 PASSED 时记录 `passed_at`。

### 5.3 `execute_admin_replacement`（call index = 2）
语义：
- 独立执行入口；任意签名账户可触发。
- 仅当投票状态 `STATUS_PASSED` 且动作合法时执行替换。
- 用途：解决"已通过但自动执行失败"的手动重试场景。

---

## 6. 执行逻辑（`try_execute_replacement_from_action`）
1. 校验投票引擎提案状态为 `STATUS_PASSED`。
2. 读取当前管理员名单，校验人数符合固定约束（19/9/9）。
3. 校验 `old_admin` 仍在名单、`new_admin` 不在名单。
4. 等长替换：`admins[old_pos] = new_admin`。
5. 写回 `CurrentAdmins`。
6. 发 `AdminReplaced` 事件。
7. 调用 `set_status_and_emit(STATUS_EXECUTED)` 标记为已执行终态。

重复执行防护：
- `set_status_and_emit(STATUS_EXECUTED)` 将提案标记为终态，后续再次执行时 `ensure!(proposal.status == STATUS_PASSED)` 拒绝。

---

## 7. 需求与安全约束
1. 人数恒定约束
   - 执行前校验管理员数量等于组织固定人数。
   - 替换是等长元素替换，BoundedVec 上限再次约束。

2. 权限约束
   - 发起与投票必须是目标机构管理员。
   - 执行是公开触发型接口（`ensure_signed`），不要求管理员身份，但只有 STATUS_PASSED 且条件合法时才能执行。

3. 回滚隔离
   - 投票成功后即记账；自动执行失败不回滚投票行为。

4. Panic 风险控制
   - `nrc_pallet_id_bytes` 返回 `Option`，避免 runtime panic。

---

## 8. 事件与错误
事件：
- `AdminReplacementProposed { proposal_id, org, institution, proposer, old_admin, new_admin }`
- `AdminReplacementVoteSubmitted { proposal_id, who, approve }`
- `AdminReplacementExecutionFailed { proposal_id }`
- `AdminReplaced { proposal_id, institution, old_admin, new_admin }`

错误：
- `InvalidInstitution`：无效机构
- `InstitutionOrgMismatch`：机构类型与 org 参数不匹配
- `InvalidAdminCount`：管理员数量不符合固定人数约束
- `UnauthorizedAdmin`：非该机构管理员
- `OldAdminNotFound`：旧管理员不在当前名单中
- `NewAdminAlreadyExists`：新管理员已在当前名单中
- `ProposalActionNotFound`：找不到与投票提案绑定的管理员更换动作
- `ProposalNotPassed`：投票尚未通过

---

## 9. Weight 策略
`WeightInfo` 由 benchmark 自动产出（`weights.rs` 由 `frame-benchmarking-cli` 生成）：
- `propose_admin_replacement()`
- `vote_admin_replacement()`
- `execute_admin_replacement()`

注意：当前 `weights.rs` 在旧代码上生成，包含已删除存储项（ProposalActions、ActiveProposalByInstitution、ProposalCreatedAt、ProposalPassedAt）的 proof 注释。权重数值为过估（安全），须在代码稳定后重跑 benchmark。

---

## 10. 完整性与创世约束
`integrity_test` 断言：
- `MaxAdminsPerInstitution >= max(NRC_ADMIN_COUNT, PRC_ADMIN_COUNT, PRB_ADMIN_COUNT)`

创世构建：
- 从 `CHINA_CB`（国储会+省储会）和 `CHINA_CH`（省储行）读取机构管理员写入 `CurrentAdmins`。
- 创世后仅信任链上 `CurrentAdmins` 状态。

---

## 11. 测试覆盖
运行命令：
```
cargo test -p admins-origin-gov
```

当前结果：14 passed

覆盖重点：
- NRC/PRC/PRB 三种组织达阈值自动执行成功
- 非本机构管理员不能发起/投票（NRC/PRC/PRB 各自交叉测试）
- 替换后新管理员可发起下一轮提案
- 自动执行失败不回滚投票
- 手动执行可恢复自动执行失败的提案
- org 与 institution 不匹配拒绝
- 否决投票不触发执行
- 未达阈值不触发执行
- old_admin 缺失 / new_admin 已存在拒绝
- 已执行提案不能再次执行
- 被拒绝提案不阻塞同机构新提案
- 无效机构拒绝

---

## 12. 推送层（Rust 内置，不改 pallet 本体）

定位：
- 推送层已内置到 `nodeui-desktop-shell`。
- 只做"链上管理员状态 → 登录端授权快照 JSON"的同步，不参与投票、不参与替换执行。
- 不修改 `admins-origin-gov/src/lib.rs` 的任何业务逻辑。

同步来源：
- 读取 `adminsOriginGov.currentAdmins` 链上存储（finalized 状态）。

同步目标：
- 运行时快照文件：`app_data/org-registry.snapshot.json`。
- `nodeui` 登录判权实时读取该快照。

运行方式：
- `nodeui` 启动时自动拉起内置同步线程，以 finalized 区块订阅驱动链上变更并写入快照（断线自动重连）。
