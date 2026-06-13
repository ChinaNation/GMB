# SFID 禁止兼容清理与真实验收修复

- 状态:已完成
- 模块:SFID / AI 编程系统规则
- 创建时间:2026-06-12

## 任务需求

把 AI 编程系统硬规则强化为“禁止兼容、彻底清理、真实验收”,并执行 SFID 当前故障修复:

- 禁止任何默认兼容旧流程、旧格式、旧数据、旧命名、旧文案、旧目录或旧交易载荷。
- 重构必须彻底清理旧代码、旧注释、旧文档和旧数据。
- 任务完成必须经过真实运行态接口验收,不能只用编译或前端 build 代替。
- 修复 SFID 后端机构详情行映射越界 panic 导致 `postgres client lock poisoned` 的运行态故障。
- 清理私权机构拆分后遗留的旧/半成品私权数据。
- 修正 SFID 登录页对扫码端职责的错误文案。

## 预计修改目录

- `AGENTS.md` / `memory/AGENTS.md`
  - 中文注释:新线程最高优先级入口规则;强化禁止兼容、彻底清理、真实验收。
- `memory/07-ai/`
  - 中文注释:AI 编程系统长期规则、完成标准和提交前清单;写入不可绕过的执行门禁。
- `memory/08-tasks/`
  - 中文注释:当前任务卡和任务索引;记录本次执行范围和验收结果。
- `sfid/backend/`
  - 中文注释:修复机构详情查询越界 panic;不恢复旧兼容字段或旧 handler。
- `sfid/frontend/auth/`
  - 中文注释:修正登录页扫码端职责文案,避免把登录挑战引导到不支持登录的 wuminapp。
- `memory/05-modules/sfid/`
  - 中文注释:同步 SFID 后端/前端文档,记录真实验收和禁止兼容口径。
- PostgreSQL `sfid` 数据库
  - 中文注释:删除不符合六类私权目标态的旧私权残留数据;不做字段补丁兼容。

## 验收标准

- `rg` 不再发现默认兼容旧流程的规则缺口。
- `cargo fmt` 和 `cargo check` 通过。
- 前端 build 通过。
- 后端重启后真实接口连续请求不再出现 `postgres client lock poisoned`。
- 登录二维码 challenge 生成、登录结果轮询接口返回稳定目标态错误或成功状态,不再因连接池污染 500。
- 市管理员、联邦管理员、联邦注册局接口连续多次真实请求稳定返回。
- 私权旧残留数据删除后,`PRIVATE_INSTITUTION` 不再存在 `private_type IS NULL` 或 `name IS NULL` 的旧记录。

## 执行记录

- 已强化 `AGENTS.md`、`memory/AGENTS.md`、`memory/07-ai/agent-rules.md`、`memory/07-ai/workflow.md`、`memory/07-ai/definition-of-done.md`、`memory/07-ai/pre-submit-checklist.md` 的硬规则,写入禁止兼容、彻底清理、真实验收。
- 已修复 `sfid/backend/main.rs` 中机构详情行映射越界问题:`institution_from_subject_row()` 的法人代表照片大小字段改为读取 SELECT 中真实存在的第 27 列,避免 panic 污染 PostgreSQL 客户端锁。
- 已删除数据库中不符合六类私权目标态的旧私权残留数据:4 条 `subjects`、4 条 `private`、8 条 `accounts`、4 条 `ids`,未做字段补丁兼容。
- 已删除旧私权前端入口残留:`sfid/frontend/private/PrivateView.tsx`、`sfid/frontend/private/api.ts`、旧根层创建/列表组件和清算行资格文件;新入口只保留六类子目录与 `common` 共用壳。
- 已删除旧私权后端根层 handler 与清算行文件:`sfid/backend/private/handler.rs`、`sfid/backend/private/clearing.rs`;后端入口改为六类私权子模块和 `common/participants`。
- 已修正 `sfid/frontend/auth/LoginView.tsx` 登录扫码文案,登录二维码明确使用 `wumin` 公民钱包,不再引导到不支持登录二维码的 `wuminapp`。
- 已同步更新 SFID 架构、后端布局、前端布局、登录技术文档和主体技术文档,清除旧口径残留。
- 已执行真实验收:
  - `cargo fmt --manifest-path backend/Cargo.toml` 通过。
  - `cargo check --manifest-path backend/Cargo.toml` 通过。
  - `npm run build` 通过。
  - 后端重启后 `/api/v1/admin/auth/check` 连续 8 次 `200 + code=0`。
  - `/api/v1/admin/federal-registry` 连续 6 次 `200 + code=0`。
  - `/api/v1/admin/city-admins` 连续 6 次 `200 + code=0`。
  - `/api/v1/institutions/federal-registry` 连续 6 次 `200 + code=0`。
  - `/api/v1/private/sole|partnership|company|corporation|welfare|association` 均返回 `200 + code=0`。
  - 登录二维码 challenge/result 接口返回 `code=0`,result 为 `PENDING`,不再触发连接池锁污染 500。
  - 数据库残留查询结果为 `0|0|0`,即 `PRIVATE_INSTITUTION` 总数、`private_type IS NULL`、`name IS NULL` 均为 0。
  - 真实前端页面登录态渲染包含 `个体经营 / 合伙企业 / 股权公司 / 股份公司 / 公益组织 / 注册协会`,未出现旧总入口或加载类错误提示。
  - 修改前端错误文案后已重新 build 并重启 SFID 服务;重启后的接口、数据库和真实前端 DOM 复验继续通过。
