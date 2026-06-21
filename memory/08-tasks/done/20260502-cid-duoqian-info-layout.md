# CID duoqian-info 前后端与 runtime 目录收口

- 日期:2026-05-02
- 状态:done
- 完成日期:2026-05-02
- 归属:CID Agent + Blockchain Agent

## 目标

按最新边界整理 CID 机构相关目录:

- CID 前端旧的其他功能不动。
- CID 前端机构相关功能整体迁出 `src/views/institutions`。
- CID 前端机构 API 迁出 `src/api/institution.ts`。
- 新增 `frontend/chain/duoqian-info` 承载和 DUOQIAN 链交互的机构信息前端功能。
- CID 后端 `chain/institution_info` 改名为 `chain/duoqian_info`。
- runtime `cid-system` 新建 `duoqian_info` 目录,作为后续链上备案和 CID 交互逻辑的落点。

## 明确边界

- `citizencode/backend/institutions` 只负责 CID 本地机构创建、资料和账户名称维护。
- `citizencode/backend/app_core/duoqian_info` 负责 CID 与 DUOQIAN 链之间的机构信息交互。
- `citizencode/frontend/institutions` 负责机构相关前端页面。
- `citizencode/frontend/institutions/duoqian-info` 负责机构备案、链交互状态等前端组件。
- 不迁移登录、公民、联邦管理员、市管理员等旧前端功能。

## 验收

- 前端机构相关文件已移出 `src/views/institutions`。
- 前端机构 API 已移出 `src/api/institution.ts`。
- `tsconfig.json` 覆盖 `api`、`institutions`、`chain` 根层目录。
- 后端 `institution_info` 引用已改为 `duoqian_info`。
- runtime `cid-system/src/duoqian_info` 已建立并接入模块树。
- 文档同步目录边界。
- 完成后运行相关构建或检查。

## 完成记录

- `citizencode/frontend/institutions/` 已承接机构页面和清算行资格前端工具。
- `citizencode/frontend/institutions/api.ts` 已承接机构本地数据 API。
- `citizencode/frontend/institutions/` 与 `citizencode/frontend/institutions/chain_duoqian_info.ts` 已建立。
- `citizencode/backend/institutions/` 已改名为 `citizencode/backend/institutions/`,外部 HTTP 路径保持不变。
- `citizenchain/runtime/otherpallet/cid-system/src/duoqian_info/` 已建立,包含备案三字段类型、基础校验和单测。
- 已更新 repo-map、CID 前端布局文档、CID chain 文档、清算行资格文档、runtime cid-system 文档。

## 验证

- `npm run build` (`citizencode/frontend`) 通过。保留既有 Vite chunk size / dynamic import 警告。
- `cargo check` (`citizencode/backend`) 通过。保留既有 `province.rs` dead_code 警告。
- `cargo test -p cid-system` (`citizenchain`) 通过,33/33 tests passed。
