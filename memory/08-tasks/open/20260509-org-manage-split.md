# 任务卡：将机构多签管理统一收敛为 organization-manage 独立模块：runtime 中 organization-manage 作为机构多签管理模块；citizenchain/node 前后端在 governance 目录下创建 organization-manage 目录；citizenapp/lib 创建 organization-manage 目录并删除旧 duoqian 目录；代码改动后同步更新文档、补中文注释并清理残留。

- 任务编号：20260509-151149
- 状态：open
- 所属模块：citizenchain-citizenapp
- 当前负责人：Codex
- 创建时间：2026-05-09 15:11:49

## 任务需求

将机构多签管理统一收敛为 organization-manage 独立模块：runtime 中 organization-manage 作为机构多签管理模块；citizenchain/node 前后端在 governance 目录下创建 organization-manage 目录；citizenapp/lib 创建 organization-manage 目录并删除旧 duoqian 目录；代码改动后同步更新文档、补中文注释并清理残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/workflow.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/07-ai/unified-naming.md
- memory/07-ai/unified-protocols.md
- memory/05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md
- memory/05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md
- memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md
- memory/05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md
- memory/05-modules/citizenapp/onchain/ONCHAIN_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已将 node 后端机构多签目录从 `citizenchain/node/src/offchain/organization_manage/` 迁入 `citizenchain/node/src/governance/organization-manage/`。
- 已将 node 前端机构多签目录从 `citizenchain/node/frontend/offchain/organization-manage/` 迁入 `citizenchain/node/frontend/governance/organization-manage/`。
- 已将 citizenapp 机构多签目录从 `citizenapp/lib/duoqian/` 迁入 `citizenapp/lib/organization-manage/`，旧 `lib/duoqian/` 物理目录已删除。
- 已将 citizenapp 机构多签测试目录从 `citizenapp/test/duoqian/` 迁入 `citizenapp/test/organization-manage/`。
- 已将个人/机构共用的提案详情页迁入 `citizenapp/lib/proposal/shared/duoqian_manage_detail_page.dart`，避免放入机构多签专属目录。
- 已将个人多签使用的账户状态轻量模型收回 `citizenapp/lib/personal-manage/personal_manage_models.dart`，避免个人模块依赖机构多签模型。
- 已同步更新命名、协议和模块文档中的目录边界说明。

## 验证记录

- `npx tsc --noEmit`：通过。
- `flutter analyze --no-fatal-infos`：通过。
- `flutter test test/organization-manage test/personal-manage`：通过，49 项测试通过。
- 旧目录残留扫描：未发现 `citizenapp/lib/duoqian/`、node `offchain/organization_manage`、node 前端 `offchain/organization-manage` 等旧路径残留。
- `cargo check --manifest-path citizenchain/Cargo.toml -p node`：受项目统一 WASM 门禁阻塞，失败原因为未设置 `WASM_FILE` 环境变量；当前未发现可直接复用的本地 `.wasm` 产物。
