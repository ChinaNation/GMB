# 2026-05-09 duoqian-transfer 严格边界清理

## 任务需求

按用户确认的最终边界继续清理多签转账功能：

- `duoqian-transfer` 是独立多签转账模块。
- runtime 只放在 `citizenchain/runtime/transaction/duoqian-transfer/`。
- node 后端只放在 `citizenchain/node/src/duoqian_transfer/`。
- node 前端只放在 `citizenchain/node/frontend/duoqian-transfer/`。
- wuminapp 只放在 `wuminapp/lib/duoqian-transfer/`。
- `proposal`、`governance`、`duoqian`、`offchain`、`institution`、`vote` 中不得保留多签转账实现逻辑；如需入口或列表聚合，只能调用模块边界内导出的通用适配能力。
- 同步清理错误注释、旧文档和路径残留。

## 影响范围

- `citizenchain/node/src/duoqian_transfer/`：承接 node 后端多签转账详情解码和链上查询。
- `citizenchain/node/src/governance/`：移除多签转账详情结构、解码和存储查询实现，仅保留通用提案聚合调用。
- `citizenchain/node/frontend/duoqian-transfer/`：承接 node 前端多签转账详情展示组件和类型。
- `citizenchain/node/frontend/governance/`：移除多签转账详情展示实现，仅保留通用页面组合入口。
- `wuminapp/lib/duoqian-transfer/`：承接 wuminapp 机构页、投票页需要的多签转账列表适配和跳转能力。
- `wuminapp/lib/institution/`、`wuminapp/lib/vote/`、`wuminapp/lib/duoqian/`、`wuminapp/lib/proposal/`：移除多签转账业务实现残留。
- `memory/01-architecture/`、`memory/04-decisions/`、`memory/05-modules/`、`memory/08-tasks/`：更新文档、任务索引和历史边界说明。

## 风险点

- node 后端 JSON 字段名必须兼容前端现有详情页，避免治理详情页面 404/空详情。
- wuminapp 机构页和投票页仍需要显示多签转账事件，但显示和跳转逻辑必须由 `duoqian-transfer` 导出。
- 不能误删 `wuminapp/lib/duoqian/` 中个人/机构多签账户管理能力。
- 历史任务卡默认保留，不作为功能残留删除。

## 执行状态

- [x] 创建任务卡
- [x] 迁移 node 后端多签转账详情解码到 `duoqian_transfer`
- [x] 迁移 node 前端多签转账详情展示到 `duoqian-transfer`
- [x] 收拢 wuminapp 机构页、投票页和账户页的多签转账适配
- [x] 更新文档、注释和任务索引
- [x] 运行残留扫描和必要校验

## 验证记录

- `npm run build`（`citizenchain/node/frontend`）：通过，保留既有 chunk size warning。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/node/Cargo.toml --bin citizenchain`：通过，保留既有 dead_code warning。
- `flutter analyze lib/duoqian-transfer lib/proposal/shared/proposal_models.dart lib/institution/institution_detail_page.dart lib/vote/vote_view.dart lib/duoqian/shared/duoqian_account_info_page.dart lib/app_modules/duoqian_account_slots.dart`：通过。
- 残留扫描：`proposal/transfer`、旧 `transfer_proposal_service`、错误 `call_index=4/6` 投票注释、`TRANSFER_APP_TECHNICAL.md` 均已清理；`governance/proposal` 不再直接引用 `duoqian_transfer`。
