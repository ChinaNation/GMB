# 任务卡：修复 grandpakey-change P1-P5 问题

- 任务编号：20260501-103245
- 状态：done
- 所属模块：citizenchain/runtime/governance/grandpakey-change
- 当前负责人：Codex
- 创建时间：2026-05-01 10:32:45

## 任务需求

修复 grandpakey-change 审查问题，范围包含 P1/P2/P3/P4/P5/P7；明确排除 P6 的 propose 阶段同 new_key 活跃提案拦截优化。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md

## 默认改动范围

- `citizenchain/runtime/governance/grandpakey-change`
- `memory/05-modules/citizenchain/runtime/governance/grandpakey-change`
- `memory/08-tasks`

## 输出物

- Cargo feature 与依赖归类修复
- Runtime 迁移幂等与 try-runtime 校验
- FatalFailed 自动执行路径测试
- 死错误码清理
- 技术文档 call_index 修正
- 验证记录

## 实施记录

- 任务卡已创建
- 已修正 `GRANDPAKEYCHANGE_TECHNICAL.md` 5.2-5.4 的 extrinsic 说明与 `call_index`，明确投票统一走 `voting_engine::internal_vote`，执行入口为 `call_index = 1`，取消失败提案入口为 `call_index = 2`。
- 已补齐 `try-runtime` feature 向 `frame-support`、`frame-system`、`pallet-grandpa`、`sp-runtime`、`voting-engine` 的传递，并将仅测试使用的 `sp-io` 移入 `dev-dependencies`。
- 已将 v2 迁移调整为先清空 `GrandpaKeyOwnerByKey` 反向索引再按 `CurrentGrandpaKeys` 重建，避免重复迁移或脏数据导致静默覆盖残留。
- 已新增 `try-runtime` 的 `pre_upgrade` / `post_upgrade` 校验：升级前记录当前 GRANDPA key 数量并防御重复 key，升级后校验正向列表与反向索引 1:1 对齐、数量一致且 storage version 正确。
- 已删除当前无抛出点的错误码 `InstitutionOrgMismatch`、`UnsupportedOrg`、`PassedProposalCannotBeCancelled`，并清理相关不可达分支。
- 已新增两条执行回调路径测试，覆盖投票通过后 `OldAuthorityNotFound` 与 `NewKeyAlreadyUsed` 触发 `FatalFailed` 并推进到 `STATUS_EXECUTION_FAILED` 的场景。
- 按用户要求未实现 P6：propose 阶段扫描活跃提案拦截相同 `new_key` 的体验优化。

## 验证记录

- `cargo fmt --manifest-path citizenchain/Cargo.toml --all`
- `WASM_FILE=/private/tmp/dummy_wasm.wasm cargo test -p grandpakey-change`
  - 17 passed，0 failed
- `WASM_FILE=/private/tmp/dummy_wasm.wasm cargo check -p grandpakey-change --features try-runtime`
- `WASM_FILE=/private/tmp/dummy_wasm.wasm cargo check -p grandpakey-change --no-default-features`
- `WASM_FILE=/private/tmp/dummy_wasm.wasm cargo check -p citizenchain --features try-runtime`
- `cargo fmt --manifest-path citizenchain/Cargo.toml --all -- --check`
- 已扫描 `grandpakey-change`、`wumin`、`wuminapp`、`citizenchain/node` 相关目录，确认被删除错误码无残留引用。
- 已扫描技术文档，确认旧 `call_index = 4`、执行入口旧 `index = 2`、取消入口旧 `index = 4` 等失真内容已清理。
