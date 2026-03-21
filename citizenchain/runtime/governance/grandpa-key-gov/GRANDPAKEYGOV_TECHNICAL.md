# GRANDPA_KEY_GOV Technical Notes

## 0. 功能需求
### 0.1 模块职责
`grandpa-key-gov` 负责把“机构 GRANDPA 公钥替换”包装成受治理约束的链上流程，要求：
- 仅支持国储会（NRC）与省储会（PRC）发起 GRANDPA 密钥替换。
- 仅允许目标机构内部管理员发起、投票、执行和清理提案。
- 治理投票由 `voting-engine-system` 的内部投票统一承载。
- 投票通过后由模块自动调度 `pallet-grandpa::schedule_change`。

### 0.2 提案创建需求
- `institution` 必须真实属于 NRC 或 PRC。
- 发起人必须是该机构当前内部管理员。
- `new_key` 不能为零值，必须是有效且非 weak/small-order 的 ed25519 公钥。
- `new_key` 不能等于该机构当前 GRANDPA 公钥，也不能被其他机构当前占用。
- 同一把 `new_key` 同一时间只能被一个活跃提案占用。
- 同一机构同一时间只允许一个活跃提案。

### 0.3 执行与失败恢复需求
- 提案达到通过阈值后，应自动尝试执行 GRANDPA 密钥替换。
- 自动执行失败不能回滚已通过的投票结果。
- 若失败原因是暂时性的（如 `GrandpaChangePending`），应允许机构管理员后续手动重试执行。
- 若提案已通过但已确定不可执行，应允许机构管理员手动取消失败提案，解除机构阻塞。

### 0.4 生命周期与清理需求
- 被拒绝、缺失或超过 stale 窗口的未通过提案，不应长期阻塞机构后续发起新提案。
- `cancel_stale_replace_grandpa_key` 只能清理未通过的 stale 提案，不能取消已通过提案。
- 下一次 `propose_replace_grandpa_key` 时，应能自动清理 rejected/stale 的旧提案索引与并发占用索引。

## 1. 模块定位
`grandpa-key-gov` 是“GRANDPA 密钥治理模块”，职责是：
- 仅允许国储会（NRC）与省储会（PRC）发起 GRANDPA 密钥替换提案。
- 仅允许目标机构内部管理员参与提案/投票/执行/清理。
- 借助 `voting-engine-system` 内部投票达成通过后，调用 `pallet-grandpa::schedule_change` 变更 authority set。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/governance/grandpa-key-gov/src/lib.rs`

## 2. 运行时接线
Runtime 配置位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

关键接线：
- `impl grandpa_key_gov::Config for Runtime`
  - `InternalVoteEngine = VotingEngineSystem`
  - `StaleProposalLifetime = GrandpaKeyStaleProposalLifetime`
  - `GrandpaChangeDelay = GrandpaAuthoritySetChangeDelay`
- `GrandpaAuthoritySetChangeDelay = 30`（预留运维注入新 gran 私钥窗口）
- `MaxSetIdSessionEntries = 128`（为后续等值投票追溯能力预留）

## 3. 存储模型
1. `ProposalActions: Map<u64, GrandpaKeyReplacementAction>`
- 绑定提案与动作：`institution`, `old_key`, `new_key`

2. `CurrentGrandpaKeys: Map<InstitutionPalletId, [u8; 32]>`
- 机构当前 GRANDPA 公钥

3. `GrandpaKeyOwnerByKey: Map<[u8; 32], InstitutionPalletId>`
- 公钥反向索引，避免 O(n) 扫描

4. `ProposalCreatedAt: Map<u64, BlockNumber>`
- 提案创建高度（用于 stale 清理）

5. `ActiveProposalByInstitution: Map<InstitutionPalletId, u64>`
- 单机构活跃提案索引（同机构同一时间只允许 1 个）

6. `PendingProposalByNewKey: Map<[u8; 32], u64>`
- `new_key -> proposal_id` 的并发占用索引
- 防止两家机构同时把同一把新 key 放进不同活跃提案

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
- `new_key` 必须是有效且非 weak/small-order 的 ed25519 公钥
- `new_key != old_key`
- `new_key` 不能被其他机构占用（反向索引 O(1)）
- 同机构不能有活跃提案
- 若旧提案已经 rejected、缺失或超过 stale 窗口，会在本次 propose 成功创建新提案后自动清理

### 5.2 `vote_replace_grandpa_key`（index = 1）
约束：
- 投票人必须是该机构内部管理员
- 委托 `voting-engine-system` 内部投票
- 一旦提案 `STATUS_PASSED`，无条件自动尝试执行替换
- 自动执行失败仅记录 `GrandpaKeyExecutionFailed`，不回滚投票

### 5.3 `execute_replace_grandpa_key`（index = 2）
约束：
- 仅该机构内部管理员可手动执行
- 用于“已通过但自动执行失败”的重试

### 5.4 `cancel_stale_replace_grandpa_key`（index = 3）
约束：
- 仅该机构内部管理员可清理
- 仅可清理“超时且未通过”的提案
- 不允许取消 `STATUS_PASSED` 提案

### 5.5 `cancel_failed_replace_grandpa_key`（index = 4）
约束：
- 仅该机构内部管理员可清理
- 仅可清理“已通过但当前确定不可执行”的提案
- 用于解除机构被 ActiveProposal 长期锁死的问题

## 6. 执行路径与 GRANDPA 交互
`try_execute_from_action` 关键步骤：
1. 读取当前 GRANDPA authorities
2. 用 `old_key -> new_key` 替换目标条目
3. 校验旧 key 存在（否则 `OldAuthorityNotFound`）
4. 校验替换后无重复 key（否则 `NewKeyAlreadyUsed`）
5. 若已有 pending change，直接报 `GrandpaChangePending`
6. 调用 `pallet_grandpa::schedule_change(next_authorities, delay, None)`
7. 成功后调用 `set_status_and_emit(STATUS_EXECUTED)` 标记为已执行终态，防止重复执行
8. 同步更新 `CurrentGrandpaKeys` 与 `GrandpaKeyOwnerByKey`

提案状态流转：`VOTING → PASSED → EXECUTED`（执行成功）/ `VOTING → REJECTED`（否决）。
注：`cancel_failed_replace_grandpa_key` 会将已通过但不可执行的提案设置为 `STATUS_REJECTED` 后清理。

## 7. 关键错误与语义
- `GrandpaChangePending`：当前已有待生效 authority set 变更
- `OldAuthorityNotFound`：提案绑定的旧 key 已不在当前 authority set
- `ProposalStillExecutable`：不允许误清理仍可执行的通过提案
- `UnauthorizedAdmin`：非该机构管理员

## 8. 风险控制与并发策略
- 并发冲突：两个提案若同时执行，后者会因 `GrandpaChangePending` 失败，等待下次重试或人工清理失败提案。
- 立即切换风险：通过 `GrandpaAuthoritySetChangeDelay=30` 降低。
- 长期卡死风险：通过 `cancel_failed_replace_grandpa_key` 消除。
- 已修复风险：过去只校验 `new_key` “能解压为曲线点”，未拒绝 small-order 弱公钥；现在已显式拒绝 weak key。
- 已修复风险：过去 rejected/stale 的旧提案可能继续占着机构活跃索引；现在会在下一次成功 `propose` 时自动清理。

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
- small-order 弱 ed25519 公钥会被拒绝。
- rejected 的旧提案会在下一次 `propose` 时自动清理。
- 超过 stale 窗口但未终结的旧提案会在下一次 `propose` 时自动清理。
