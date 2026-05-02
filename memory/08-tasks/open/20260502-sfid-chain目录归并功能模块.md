# SFID chain 目录归并到功能模块

- 创建时间:2026-05-02
- 状态:进行中

## 需求

整改 SFID 系统前后端目录边界:不再单独维护 `chain/` 业务目录。
今后 SFID 系统各功能模块只要有和区块链交互的代码,就在该功能模块目录中创建
独立文件,文件名必须以 `chain_` 开头。

## 边界规则

- 后端禁止新增 `sfid/backend/chain/` 业务目录。
- 前端禁止新增 `sfid/frontend/chain/` 业务目录。
- 机构链交互放 `institutions/chain_*`。
- 公民链交互放 `citizens/chain_*`。
- 省管理员链交互放 `sheng_admins/chain_*`。
- 跨模块复用的链底层工具放 `app_core/chain_*`。
- 普通业务 handler/service/model 不混入链交互逻辑。

## 预计修改目录

- `sfid/backend/institutions/`
  - 中文注释:机构模块后端目录;新增 `chain_duoqian_info.rs`,承接机构查询、注册信息凭证、清算行候选等机构与区块链交互接口。
- `sfid/frontend/institutions/`
  - 中文注释:机构模块前端目录;新增 `chain_duoqian_info.ts`,承接机构注册信息凭证与链侧机构查询 API/type。
- `sfid/backend/citizens/`
  - 中文注释:公民模块后端目录;新增 `chain_citizens.rs`,承接公民绑定推链、投票凭证、联合投票人数快照等公民链交互。
- `sfid/backend/sheng_admins/`
  - 中文注释:省管理员模块后端目录;新增 `chain_sheng_admins.rs`,承接省管理员三槽名册、签名公钥激活/轮换、待签缓存等链交互。
- `sfid/frontend/sheng_admins/`
  - 中文注释:省管理员模块前端目录;新增/迁移 `chain_*.tsx` 与 `chain_sheng_admins.ts`,承接原省管理员链交互页面、API 和类型。
- `sfid/backend/app_core/`
  - 中文注释:后端基础设施目录;新增 `chain_runtime.rs`、`chain_client.rs`、`chain_url.rs`,承接跨业务链 RPC、genesis hash、SCALE 编码与交易包装 helper。
- `sfid/backend/main.rs`
  - 中文注释:后端路由装配文件;删除 `mod chain;`,路由引用改为各功能模块的 `chain_*` 文件。
- `memory/05-modules/sfid/`、`memory/08-tasks/`
  - 中文注释:文档与任务卡目录;同步废除独立 `chain/` 目录规则,记录新边界。

## 执行计划

1. 迁移后端共享链工具到 `app_core/chain_*`。
2. 迁移后端机构、公民、省管理员链交互到各自功能模块 `chain_*` 文件。
3. 迁移前端 `frontend/chain/*` 到对应功能模块。
4. 删除空旧目录并更新所有引用。
5. 更新文档、中文注释和残留检查。
6. 运行后端 `cargo check` 与前端 `npm run build`。

## 验收

- 后端不存在 `mod chain;`。
- 前端不存在 `sfid/frontend/chain/`。
- 后端旧 `sfid/backend/chain/` 目录无业务文件残留。
- 所有链交互文件都位于功能模块下,文件名以 `chain_` 开头。
- 机构注册信息凭证字段语义不变。
- 文档和任务卡同步更新。

## 执行结果(2026-05-02)

### 后端

- 删除 `sfid/backend/chain/` 目录。
- `sfid/backend/app_core/`
  - 新增 `chain_runtime.rs`、`chain_client.rs`、`chain_url.rs`。
- `sfid/backend/institutions/`
  - 新增 `chain_duoqian_info.rs`、`chain_duoqian_info_dto.rs`、`chain_duoqian_info_handler.rs`。
- `sfid/backend/citizens/`
  - 新增 `chain_binding.rs`、`chain_vote.rs`、`chain_joint_vote.rs`。
- `sfid/backend/sheng_admins/`
  - 新增 `chain_roster_handler.rs`、`chain_roster_query.rs`、`chain_add_backup.rs`、
    `chain_remove_backup.rs`、`chain_activate_signer.rs`、`chain_rotate_signer.rs`、
    `chain_pending_signs.rs`。
- `sfid/backend/main.rs`
  - 删除 `mod chain;`。
  - 路由全部改为业务模块 `chain_*` 文件引用。

### 前端

- 删除 `sfid/frontend/chain/` 目录。
- `sfid/frontend/institutions/chain_duoqian_info.ts`
  - 承接原机构链查询和注册信息凭证 API/type。
- `sfid/frontend/sheng_admins/`
  - 新增 `chain_sheng_admins.ts`、`chain_sheng_admins_types.ts`。
  - 新增 `chain_RosterPage.tsx`、`chain_ActivationPage.tsx`、`chain_RotatePage.tsx`。
- `sfid/frontend/App.tsx`、`api/client.ts`、`auth/types.ts`
  - 引用改到功能模块内的 `chain_` 文件。

### 文档与规则

- 更新 `memory/05-modules/sfid/backend/chain/CHAIN_TECHNICAL.md` 为“链交互归属规则”。
- 更新 `memory/05-modules/sfid/frontend/FRONTEND_LAYOUT.md`。
- 更新 `memory/05-modules/sfid/backend/institutions/INSTITUTIONS_TECHNICAL.md`。
- 更新 `memory/05-modules/sfid/backend/citizens/CITIZENS_TECHNICAL.md`。
- 更新 `memory/05-modules/sfid/backend/sheng_admins/SHENG_ADMINS_TECHNICAL.md`。
- 更新 `memory/07-ai/agent-rules.md` 和 `memory/AGENTS.md`,固化 SFID `chain_` 文件规则。

### 验证

- `cd sfid/backend && cargo fmt && cargo check`:通过,仅剩既有 `sfid/province.rs` 静态码表 warning。
- `cd sfid/frontend && npm run build`:通过,仅剩既有 Vite chunk 体积提示。
