# 任务卡：修复 Pending 主体 API 容易误用问题

- 任务编号：20260430-132721
- 状态：done
- 所属模块：admins-change / voting-engine / duoqian-manage / runtime-config
- 当前负责人：Codex
- 创建时间：2026-04-30 13:27:21

## 任务需求

按已确认方案修复 P2：`admins-change` 的管理员主体读取 API 不再把 Pending 主体暴露给普通业务授权；投票引擎提供 Pending 主体创建投票的专用入口，避免下游误把 Pending 主体当成 Active 主体。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/duoqian-manage/DUOQIAN_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有投票引擎契约
- Pending 主体只能用于创建/激活自身的投票快照
- 普通业务授权必须只接受 Active 主体
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 测试
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已在 `admins-change` 拆分 Active-only 公共业务 API 与 Pending 快照专用 API，普通业务读取不再返回 Pending 主体。
- 已在 `voting-engine` 新增 `create_pending_subject_internal_proposal`，普通 `create_internal_proposal` 只走 Active 主体，Pending 主体只能通过专用入口创建自身激活投票。
- 已新增 `InternalThresholdSnapshot`，内部提案创建时同时锁定管理员快照和阈值快照，投票期间不再实时读取主体状态。
- 已把 runtime provider 改为普通路径读取 Active API、Pending 创建路径读取 Pending 快照 API。
- 已把 `duoqian-manage` 的机构多签/个人多签创建路径切到 Pending 专用入口，关闭多签继续使用普通 Active 内部提案入口。
- 已修复 `duoqian-manage` benchmark 中 `register_sfid_institution` 旧参数残留。
- 已更新 admins-change、voting-engine、duoqian-manage 技术文档。
- 已执行残留扫描，代码和相关文档中未再发现旧 `is_subject_admin / subject_admins / subject_threshold / subject_admin_count` API 使用。

## 验证记录

- 通过：`cargo test -p voting-engine --lib`
- 通过：`cargo test -p voting-engine --lib --features runtime-benchmarks`
- 通过：`cargo test -p admins-change --lib`
- 通过：`cargo test -p admins-change --lib --features runtime-benchmarks`
- 通过：`cargo test -p duoqian-manage --lib`
- 通过：`cargo test -p duoqian-manage --lib --features runtime-benchmarks`
- 通过：`cargo test -p duoqian-transfer --lib`
- 通过：`cargo test -p resolution-destro --lib`
- 通过：`cargo test -p grandpakey-change --lib`
- 通过：`WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check -p citizenchain`
- 通过：`git diff --check`
