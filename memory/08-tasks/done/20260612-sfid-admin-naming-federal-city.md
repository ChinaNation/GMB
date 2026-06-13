# SFID 管理员命名彻底统一为联邦管理员和市管理员

- 状态:已完成
- 模块:SFID / wumin / wuminapp / AI 编程系统规则
- 创建时间:2026-06-12
- 完成时间:2026-06-12

## 任务需求

按最新口径彻底统一管理员和公民端命名:

- `wumin` 中文名固定为 `公民钱包`。
- `wuminapp` 中文名固定为 `公民`。
- 联邦注册局管理员,简称 `联邦管理员`,角色值固定为 `FEDERAL_ADMIN`。
- 市注册局管理员,简称 `市管理员`,角色值固定为 `CITY_ADMIN`。
- 删除旧口径、旧实现、旧路由、旧数据对象、旧注释和旧文档,不做历史兼容。

## 修改目录

- `AGENTS.md` / `memory/AGENTS.md`
  - 中文注释:全局规则入口,已写入产品命名、管理员命名、禁止兼容、彻底清理、真实验收硬规则。
- `memory/07-ai/`
  - 中文注释:统一命名、协议和执行门禁,已同步目标管理员口径和产品口径。
- `sfid/backend/admins/`
  - 中文注释:管理员模型、仓储、守卫、安全动作和路由,已拆为联邦管理员与市管理员目标实现。
- `sfid/backend/core/`
  - 中文注释:数据库结构和启动期校验,已清理旧角色约束、旧动作类型和旧数据对象。
- `sfid/backend/main.rs`
  - 中文注释:路由注册、错误码映射和启动期数据状态,已只注册目标管理员接口。
- `sfid/backend/scope/`、`sfid/backend/subjects/`、`sfid/backend/cpms/`
  - 中文注释:权限作用域、创建者角色和业务守卫,已同步目标角色值与中文文案。
- `sfid/frontend/auth/`、`sfid/frontend/hooks/`
  - 中文注释:前端角色类型、能力和作用域,已只保留目标角色。
- `sfid/frontend/admins/`
  - 中文注释:联邦管理员、市管理员页面、API 和安全动作,已删除旧页面和旧 API 文件。
- `sfid/frontend/App.tsx`、`sfid/frontend/utils/notice.ts`
  - 中文注释:顶部身份、注册局入口和错误提示文案,已同步目标管理员显示。
- `wumin/`、`wuminapp/`
  - 中文注释:签名解码和页面文案,已同步公民钱包/公民口径和目标管理员名称。
- `memory/01-architecture/`、`memory/05-modules/`、`memory/08-tasks/`
  - 中文注释:当前架构文档、模块文档、错误码文档和任务索引,已清理旧称谓残留。
- PostgreSQL `sfid` 数据库
  - 中文注释:已迁移管理员角色数据、重建角色约束、改名作用域表和索引约束,不保留旧数据分支。

## 执行记录

- 后端管理员角色枚举、数据库角色约束、安全动作、错误码、作用域守卫、接口路由已统一为 `FEDERAL_ADMIN` 与 `CITY_ADMIN`。
- 前端管理员类型、导航入口、管理员列表、管理员新增弹窗、API 调用和扫码登录文案已统一为联邦管理员、市管理员、公民钱包。
- 公民钱包签名载荷解码已支持目标管理员动作并补充测试。
- SFID 管理员 Passkey/安全动作链路已从旧签名确认口径改为公民钱包确认,短期挑战旧字段不做兼容。
- SFID、wuminapp 和记忆文档中的签名流程可见文案已统一为公民钱包签名/确认。
- wumin PQC 技术文档已改名为 `WUMIN_PQC_TECHNICAL.md`,内容按公民钱包离线签名端口径更新。
- AI 编程系统规则已写入产品命名、管理员命名、禁止兼容、彻底清理和真实验收硬规则。
- 本地数据库已完成旧数据对象清理和约束重建。

## 验收结果

- `cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- `cargo build --manifest-path sfid/backend/Cargo.toml` 通过。
- `npm run build` 在 `sfid/frontend/` 通过。
- `flutter test test/signer/payload_decoder_test.dart` 在 `wumin/` 通过。
- `flutter test test/qr/qr_sign_session_test.dart` 在 `wuminapp/` 通过。
- SFID 本地服务健康检查通过。
- 新管理员接口真实 HTTP 返回 `200 + code=0`。
- 旧管理员接口真实 HTTP 返回 `404`。
- 前端登录页真实页面显示 `公民钱包`,未显示旧产品文案。
- 代码、文档、任务记录已完成旧管理员口径、旧产品口径和旧签名确认口径残留扫描。
