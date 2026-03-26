# 任务卡：收敛 GRANDPA 节点启动条件，并停止 `concluded_rounds` 追加持久化

- 任务编号：20260326-011500
- 状态：in-progress
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-03-26 01:15:00

## 任务需求

1. 非 GRANDPA 节点不启动 `grandpa-voter`。
2. GRANDPA 节点保留必要的覆盖写持久化状态。
3. 停止 `concluded_rounds` 这类按轮次追加的持久化写入，观察节点数据增长是否明显收敛。

## 必读上下文

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/agent-rules.md`
- `memory/07-ai/context-loading-order.md`
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`

## 模块边界

- 允许改动：
  - `citizenchain/node`
  - `citizenchain/Cargo.toml`
  - `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- 如需改动上游 `sc-consensus-grandpa`，必须把覆盖方案落在仓库内，不能只改本机 `.cargo` 缓存。

## 风险说明

- `grandpa-voter` 启动恢复依赖 `set_state` 覆盖写状态，不能直接全部关闭。
- `best_justification` 会被 finality proof / warp proof 读取，不能删除。
- `concluded_rounds` 当前定位为生产路径未读取、仅追加写的候选冗余数据；若验证发现有隐藏依赖，需要立即回退该部分策略。

## 实施记录

- 已确认当前 `service.rs` 会在 `enable_grandpa` 时无条件启动 `grandpa-voter`，即使本地没有匹配的 GRANDPA authority 私钥。
- 已确认 GRANDPA 生产恢复路径读取 `SET_STATE_KEY`，而 `CONCLUDED_ROUNDS` 当前仅发现写入路径和测试读取路径。
- 已在 `citizenchain/node/src/service.rs` 增加启动门控：仅 `has_local_grandpa_authority == true` 时启动 `grandpa-voter`。
- 已将 `sc-consensus-grandpa` vendoring 到 `citizenchain/vendor/sc-consensus-grandpa`，并把工作区依赖切到本地路径，避免只改本机 `.cargo` 缓存。
- 已在本地 vendored `sc-consensus-grandpa/src/aux_schema.rs` 中停掉 `write_concluded_round` 的 AUX 追加写，保留 `set_state` / `best_justification` 等必要覆盖写状态。
- 已补 vendored 测试资源 `citizenchain/vendor/chain-spec/res/chain_spec.json`，并修正测试资源相对路径。
- 验证结果：
  - `cargo check -p node` 通过
  - `cargo test -p sc-consensus-grandpa concluded_rounds_are_not_persisted_anymore --lib` 通过
