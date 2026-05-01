# GRANDPA_KEY_GOV Technical Notes

## 0. 功能需求
### 0.1 模块职责
`grandpakey-change` 负责把“机构 GRANDPA 公钥替换”包装成受治理约束的链上流程，要求：
- 仅支持国储会（NRC）与省储会（PRC）发起 GRANDPA 密钥替换。
- 仅允许目标机构内部管理员发起、投票、执行和清理提案。
- 治理投票由 `voting-engine` 的内部投票统一承载。
- 投票通过后由模块自动调度 `pallet-grandpa::schedule_change`。

### 0.2 提案创建需求
- `institution` 必须真实属于 NRC 或 PRC。
- 发起人必须是该机构当前内部管理员。
- `new_key` 不能为零值，必须是有效且非 weak/small-order 的 ed25519 公钥。
- `new_key` 不能等于该机构当前 GRANDPA 公钥，也不能被其他机构当前占用。
- 并发控制由 `voting-engine` 的 `ActiveProposalsByInstitution` 统一管控（每机构上限 10 个活跃提案），本模块不另设单机构单提案限制。
- 同一把 `new_key` 若被多个活跃提案占用，第一个执行成功后后续执行会因 `NewKeyAlreadyUsed` 失败，可通过 `cancel_failed_replace_grandpa_key` 清理。

### 0.3 执行与失败恢复需求
- 提案达到通过阈值后，应自动尝试执行 GRANDPA 密钥替换。
- 自动执行失败不能回滚已通过的投票结果。
- 若失败原因是暂时性的（如 `GrandpaChangePending`），应允许机构管理员后续手动重试执行。
- 若提案已通过但已确定不可执行，应允许机构管理员手动取消失败提案，解除机构阻塞。

### 0.4 生命周期与清理需求
- 被拒绝的提案由 `voting-engine` 的过期/清理机制处理。
- 已通过但确定不可执行的提案，通过 `cancel_failed_replace_grandpa_key` 手动清理。
- 注意：旧版的 `cancel_stale_replace_grandpa_key`（call_index=3）已移除，stale 清理由投票引擎统一承载。

## 1. 模块定位
`grandpakey-change` 是“GRANDPA 密钥治理模块”，职责是：
- 仅允许国储会（NRC）与省储会（PRC）发起 GRANDPA 密钥替换提案。
- 仅允许目标机构内部管理员参与提案/投票/执行/清理。
- 借助 `voting-engine` 内部投票达成通过后，调用 `pallet-grandpa::schedule_change` 变更 authority set。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/governance/grandpakey-change/src/lib.rs`

## 2. 运行时接线
Runtime 配置位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

关键接线：
- `impl grandpakey_change::Config for Runtime`
  - `InternalVoteEngine = VotingEngine`
  - `GrandpaChangeDelay = GrandpaAuthoritySetChangeDelay`
- `GrandpaAuthoritySetChangeDelay = 30`（预留运维注入新 gran 私钥窗口）
- `MaxSetIdSessionEntries = 128`（为后续等值投票追溯能力预留）
- 注意：旧版的 `StaleProposalLifetime` 配置项已移除。

## 3. 存储模型
本模块仅维护 2 个存储项，提案数据由 `voting-engine` 统一管理。

1. `CurrentGrandpaKeys: Map<InstitutionPalletId, [u8; 32]>`
- 机构当前治理认可的 GRANDPA 公钥

2. `GrandpaKeyOwnerByKey: Map<[u8; 32], InstitutionPalletId>`
- 公钥到机构的反向索引，O(1) 判断 new_key 是否已被占用

提案数据存储在 `voting-engine` 中：
- 提案动作（`GrandpaKeyReplacementAction`）通过 `create_internal_proposal_with_data` 在创建提案时原子写入，并同步绑定 `ProposalOwner`
- 机构活跃提案列表由 `ActiveProposalsByInstitution`（上限 10 个）管控

历史存储项（已移除）：
- `ActiveProposalByInstitution`：已由投票引擎统一管控
- `PendingProposalByNewKey`：已移除，冲突在执行时检测
- `ProposalActions`：已迁移到投票引擎
- `ProposalCreatedAt`：已迁移到投票引擎

## 4. 创世初始化
`GenesisBuild` 会从 `CHINA_CB` 初始化：
- `CurrentGrandpaKeys[institution] = node.grandpa_key`
- `GrandpaKeyOwnerByKey[node.grandpa_key] = institution`

并在构建时断言初始 key 不重复。

## 5. 外部接口（Calls）
### 5.1 `propose_replace_grandpa_key`（index = 0）
约束：
- 仅 `ORG_NRC | ORG_PRC`
- 发起人必须是该机构内部管理员
- `new_key` 不能为零值
- `new_key` 必须是有效且非 weak/small-order 的 ed25519 公钥（`CompressedEdwardsY` + `is_small_order`）
- `new_key != old_key`
- `new_key` 不能被其他机构当前占用（反向索引 O(1)）
- 机构活跃提案数由 `voting-engine` 的 `ActiveProposalsByInstitution`（上限 10 个）管控

### 5.2 `vote_replace_grandpa_key`（index = 1）
约束：
- 投票人必须是该机构内部管理员
- 委托 `voting-engine` 内部投票
- 一旦提案 `STATUS_PASSED`，无条件自动尝试执行替换
- 自动执行暂时失败仅记录 `GrandpaKeyExecutionFailed`，返回 `RetryableFailed`，不回滚投票

### 5.3 `execute_replace_grandpa_key`（index = 2）
约束：
- 兼容入口：委托 `VotingEngine::retry_passed_proposal_for`
- 仅提案快照管理员可手动执行，重试次数、deadline 与状态推进由投票引擎统一校验
- 用于“已通过但自动执行暂时失败”的重试

### 5.4 `cancel_failed_replace_grandpa_key`（index = 4）
注意：旧版 `cancel_stale_replace_grandpa_key`（index = 3）已移除，stale 清理由投票引擎统一承载。

当前 `cancel_failed_replace_grandpa_key`（index = 4）：
约束：
- 兼容入口：委托 `VotingEngine::cancel_passed_proposal_for`
- 仅提案快照管理员可清理
- 仅可清理“已通过但当前确定不可执行”的提案
- 清理时由投票引擎将状态从 `STATUS_PASSED` 推进到 `STATUS_EXECUTION_FAILED`

## 6. 执行路径与 GRANDPA 交互
`try_execute_from_action` 关键步骤：
1. 读取当前 GRANDPA authorities
2. 用 `old_key -> new_key` 替换目标条目
3. 校验旧 key 存在（否则 `OldAuthorityNotFound`）
4. 校验替换后无重复 key（否则 `NewKeyAlreadyUsed`）
5. 若已有 pending change，直接报 `GrandpaChangePending`
6. 调用 `pallet_grandpa::schedule_change(next_authorities, delay, None)`
7. 同步更新 `CurrentGrandpaKeys` 与 `GrandpaKeyOwnerByKey`
8. 返回 `ProposalExecutionOutcome::Executed`，由投票引擎统一标记 `STATUS_EXECUTED`

提案状态流转：`VOTING → PASSED → EXECUTED`（执行成功）/ `VOTING → REJECTED`（否决）/ `VOTING → PASSED → EXECUTION_FAILED`（已通过但确认不可执行）。
注：`cancel_failed_replace_grandpa_key` 会将已通过但不可执行的提案设置为 `STATUS_EXECUTION_FAILED` 后清理。

## 7. 关键错误与语义
- `GrandpaChangePending`：当前已有待生效 authority set 变更
- `OldAuthorityNotFound`：提案绑定的旧 key 已不在当前 authority set
- `ProposalStillExecutable`：不允许误清理仍可执行的通过提案
- `UnauthorizedAdmin`：非该机构管理员

## 8. 风险控制与并发策略
- 并发冲突：两个提案若同时执行，后者会因 `GrandpaChangePending` 返回 `RetryableFailed`，等待下次重试；确定不可执行时再人工取消失败提案。
- 立即切换风险：通过 `GrandpaAuthoritySetChangeDelay=30` 降低。
- 长期卡死风险：通过 `cancel_failed_replace_grandpa_key` 消除。
- 已修复风险：过去只校验 `new_key` “能解压为曲线点”，未拒绝 small-order 弱公钥；现在已显式拒绝 weak key。
- 并发 new_key 冲突：当前设计不在提案创建时拦截（旧版 `PendingProposalByNewKey` 已移除），而是在执行时通过 `validate_action` 的 `BTreeSet` 唯一性检查拒绝。冲突的提案可通过 `cancel_failed_replace_grandpa_key` 清理。

## 9. 创世公钥严格校验（按“严格要求”）
位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/genesis_config_presets.rs`

已实现测试：
1. `grandpa_authority_keys_are_unique_valid_hex_and_32_bytes`
- 校验长度、hex、唯一性，并用 `ed25519-dalek::VerifyingKey::from_bytes` 强制校验 ed25519 曲线点有效性。

2. `grandpa_keys_match_china_cb_grandpa_keys`
- 强制校验 `GRANDPA_AUTHORITY_KEYS_HEX` 与 `CHINA_CB.grandpa_key` 一一一致。

3. `china_cb_grandpa_keys_are_valid_ed25519_pubkeys`
- 强制校验 `CHINA_CB.grandpa_key` 全量都是有效 ed25519 公钥。

说明：
- 当前若数据不是合法 ed25519 点，测试会失败。这是设计要求，不做兼容放宽。

## 10. 运维要点
1. 新 key 上线顺序：先注入本机 keystore，再等待治理通过并生效。
2. 换钥提案通过后若执行失败：
- 先查是否 `GrandpaChangePending`，等待 pending change 落地后重试执行。
- 若已确定不可执行，可用 `cancel_failed_replace_grandpa_key` 清理。
3. 生产节点应监控：
- `GrandpaKeyExecutionFailed`
- `GrandpaKeyReplaced`
- GRANDPA pending change 状态

## 11. 与当前版本边界
- 本模块负责“换钥治理编排 + authority set 调度”，不负责私钥托管。
- 等值投票惩罚链路（offences/session historical）仍未启用；当前仅为后续接入保留历史 set 映射能力。

## 12. 测试覆盖
`cargo test -p grandpakey-change` 覆盖（15 个用例）：

正向路径：
- 投票通过后自动执行密钥替换并更新 authority set
- 自动执行失败后手动重试成功
- 已通过但不可执行的提案可被取消

错误路径：
- small-order 弱 ed25519 公钥拒绝
- 零值 new_key 拒绝（`NewKeyIsZero`）
- new_key == old_key 拒绝（`NewKeyUnchanged`）
- new_key 被他机构占用拒绝（`NewKeyAlreadyUsed`）
- 非管理员提案/投票拒绝（`UnauthorizedAdmin`）
- 无效机构拒绝（`InvalidInstitution`）
- 执行非通过提案拒绝（`ProposalNotPassed`）
- 取消仍可执行提案拒绝（`ProposalStillExecutable`）
- 取消暂时阻塞提案拒绝（`GrandpaChangePending`）
