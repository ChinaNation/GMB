# 任务卡：修复 runtime 审计发现的 SFID 解绑、联合投票快照、终态清理和 50% 判定问题

- 任务编号：20260516-runtime-audit-fixes-vote-sfid
- 状态：done
- 所属模块：citizenchain-runtime
- 当前负责人：Codex
- 创建时间：2026-05-16

## 任务需求

修复 runtime 审计中已确认进入本轮处理的 4 个问题：

1. `unbind_sfid` 事件不得把被解绑账户伪记为管理员。
2. 联合投票人口快照必须表达“提案发起那一刻拥有投票权的公民数”，不得在提案创建后随人口变化刷新。
3. 投票引擎进入执行失败终态时不得静默吞掉业务清理错误。
4. 联合公投 50% 判定改为长期安全的 `u128` 比较。

## 明确不做

- 不删除开发期直升 runtime 能力。
- 不修改创世期 30 秒出块逻辑。
- 不实现未来进入运行期前的开发能力物理删除。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/

## 预计修改目录

- `citizenchain/runtime/otherpallet/sfid-system/`：修正 `unbind_sfid` 事件字段和测试。
- `citizenchain/runtime/votingengine/joint-vote/`：修正人口快照消费语义与 50% 判定。
- `citizenchain/runtime/votingengine/`：修正执行失败终态业务清理错误处理。
- `citizenchain/runtime/votingengine/internal-vote/`：补充联合投票和终态清理回归测试。
- `memory/05-modules/citizenchain/runtime/`：更新过期文档和残留说明。

## 输出物

- runtime 代码修复
- 测试更新
- 中文注释同步
- 文档更新与残留清理

## 实施记录

- 任务卡已创建
- `sfid-system`：`SfidUnbound` 移除不真实的 `admin` 字段，Root 解绑只记录 `who` 与 `binding_id`。
- `joint-vote`：联合提案只消费当前区块准备的人口快照，过期快照返回 `PopulationSnapshotNotCurrent` 并删除；联合公投 50% 判定改用 `u128` 中间值。
- `votingengine`：新增 `PendingTerminalCleanups`，执行失败终态通知业务模块失败时入队，后续 `on_initialize` 有界重试。
- 测试：已覆盖 SFID 解绑事件、过期人口快照、极端 `u64` 阈值、终态清理失败入队与重试。

## 验证记录

- `cargo test --manifest-path citizenchain/runtime/Cargo.toml -p sfid-system`
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml -p internal-vote`
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml -p joint-vote`
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml -p votingengine`
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`
- `git diff --check`
