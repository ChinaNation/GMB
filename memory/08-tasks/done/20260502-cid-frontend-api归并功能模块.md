# CID 前端 API 归并到功能模块

- 创建时间:2026-05-02
- 状态：done

## 需求

整改 CID 前端 API 目录边界:不再单独维护旧全局 API 目录。
今后某个功能需要调用后端 API,就在该功能模块目录中创建 API 专用文件。
链交互 API 继续遵守 `chain_` 文件规则。

## 边界规则

- 禁止新增或恢复旧全局业务 API 目录。
- 通用 HTTP 请求封装放 `citizencode/frontend/utils/http.ts`,不承载业务 API。
- 登录/会话 API 放 `citizencode/frontend/auth/api.ts`。
- 机构 API 放 `citizencode/frontend/institutions/api.ts`。
- 公民 API 放 `citizencode/frontend/citizens/api.ts`。
- 联邦管理员 API 放 `citizencode/frontend/sheng_admins/api.ts`。
- 市管理员 API 放 `citizencode/frontend/shi_admins/api.ts`。
- 链交互 API 保持 `chain_` 前缀,例如 `institutions/chain_duoqian_info.ts`。

## 预计修改目录

- `citizencode/frontend/utils/`
  - 中文注释:前端通用工具目录;新增 `http.ts`,只放 request/adminRequest/401 拦截等基础 HTTP 能力。
- `citizencode/frontend/auth/`
  - 中文注释:登录与会话目录;新增/调整 `api.ts` 和 `types.ts`,承接登录、登出、鉴权检查、二维码登录 API 与登录态类型。
- `citizencode/frontend/institutions/`
  - 中文注释:机构前端目录;承接原 `api/institution.ts` 和机构/CPMS 相关 API。
- `citizencode/frontend/citizens/`
  - 中文注释:公民前端目录;承接公民列表、绑定、解绑、推链绑定、CPMS 状态扫码 API。
- `citizencode/frontend/sheng_admins/`
  - 中文注释:联邦管理员前端目录;承接联邦管理员本地业务 API,链交互 API 仍在 `chain_sheng_admins.ts`。
- `citizencode/frontend/shi_admins/`
  - 中文注释:市管理员前端目录;承接操作员管理 API。
- 旧全局 API 目录
  - 中文注释:旧全局 API 目录;已删除,不再作为业务接口入口。
- `memory/05-modules/citizencode/`、`memory/07-ai/`、`memory/AGENTS.md`
  - 中文注释:文档和 AI 规则目录;固化“前端 API 跟随功能模块”规则。

## 验收

- 旧全局 API 目录不存在。
- `rg "../api|./api|frontend/api|citizencode/frontend/api" citizencode/frontend` 无旧目录引用。
- 前端业务 API 都位于对应功能模块。
- 通用 HTTP 工具不包含业务接口。
- `cd citizencode/frontend && npm run build` 通过。
- 文档、中文注释、残留清理完成。

## 完成记录

- 新增 `citizencode/frontend/utils/http.ts`,通用 HTTP 能力只保留请求封装和 401 拦截。
- 新增/调整 `auth/api.ts`、`citizencode/api.ts`、`institutions/api.ts`、`citizens/api.ts`、`sheng_admins/api.ts`、`shi_admins/api.ts`。
- 删除旧全局 API 目录，保留 `citizencode/frontend/utils/http.ts` 作为通用 HTTP 封装。
- 更新所有前端引用,不再引用 `../api/client` 或 `../api/institution`。
- 更新 `citizencode/frontend/tsconfig.json`,移除旧 `api`、`chain` include,新增 `cid` include。
- 更新 `memory/05-modules/citizencode/frontend/FRONTEND_LAYOUT.md`、`memory/07-ai/agent-rules.md`、`memory/AGENTS.md`。
- 已执行 `cd citizencode/frontend && npm run build`,构建通过。

## 完成信息

- 完成时间：2026-05-02 14:50:39
- 完成摘要：CID 前端全局 api 目录已拆分到 auth/citizencode/institutions/citizens/sheng_admins/shi_admins 各功能模块,通用 HTTP 收口到 utils/http.ts,旧目录和引用已清理,前端构建通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
