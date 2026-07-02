# CID CPMS 与联邦管理员目录整改

- 创建时间:2026-05-02
- 状态:已完成

## 需求

检查并整改 `cid` 系统中 `sheng_admins` 目录职责过宽的问题。确认
`sheng_admins/institutions.rs` 实际承载 CPMS 系统注册与管理功能后,前后端新建
`cpms` 功能目录,把 CPMS 系统管理功能迁入该目录。

同时清理联邦管理员目录内已下架或无活跃调用方的区块链交互残留。联邦管理员与区块链
交互的功能当前只保留“更换联邦管理员”方向,后续新增链交互必须集中到
`sheng_admins/chain_*.rs` 中,普通页面展示和本人本地签名密钥管理不再伪装为链交互。

## 预计修改目录

```text
citizencode/backend/citizenpassport/                 # CPMS 系统注册、安装二维码、匿名证书、站点状态治理
citizencode/frontend/citizenpassport/                # CID 前端的 CPMS 系统管理组件与 API
citizencode/backend/sheng_admins/         # 只保留联邦管理员/市管理员治理与联邦管理员必要链交互
citizencode/frontend/sheng_admins/        # 只保留联邦管理员/市管理员页面和联邦管理员必要链交互
memory/05-modules/citizencode/            # 同步更新模块边界、链交互归属和目录说明
```

## 边界规则

- CPMS 系统管理功能归 `cpms` 目录,不再放在 `sheng_admins/institutions.rs`。
- 联邦管理员页面的一主两备展示不是链交互,不得使用 `chain_` 文件名。
- 联邦管理员本人签名密钥生成/更换是 CID 本地 seed 生命周期,不是链交互,不得使用 `chain_` 文件名。
- 已下架的 add/remove backup、activate/rotate signer 旧直推链文件必须删除或彻底脱离活跃导出。
- 代码修改后必须补充中文注释、更新文档、清理残留。

## 验收

- `citizencode/backend/sheng_admins/institutions.rs` 不再存在。
- `citizencode/backend/citizenpassport/` 承接 CPMS 后端 handler。
- `citizencode/frontend/citizenpassport/` 承接 CPMS 前端组件和 API。
- `sheng_admins` 目录内无无活跃路由的旧 `chain_add_backup`、`chain_remove_backup`、
  `chain_activate_signer`、`chain_rotate_signer`、`chain_pending_signs` 残留。
- 文档中的目录说明与代码一致。
- `cargo fmt`、`cargo check`、`npm run build` 通过。

## 执行记录

- 后端新增 `citizencode/backend/citizenpassport/`,将原 `sheng_admins/institutions.rs`
  迁为 `citizenpassport/handler.rs`,并新增 `citizenpassport/mod.rs` 导出。
- `main.rs` 的 CPMS 路由已全部改为 `cpms::*`。
- `citizens/binding.rs`、`citizens/status.rs` 已改为引用 `crate::cpms::*`。
- `sheng_admins` 后端已删除旧 `chain_add_backup.rs`、`chain_remove_backup.rs`、
  `chain_activate_signer.rs`、`chain_rotate_signer.rs`、`chain_pending_signs.rs`、
  `chain_roster_handler.rs`、`chain_roster_query.rs`、`bootstrap.rs`、
  `signing_metadata.rs`、`multisig.rs`。
- `sheng_admins/roster.rs` 只保留注册局页面一主两备展示。
- `sheng_admins/signing_keys.rs` 承接本人 signing seed 自动加载、生成、更换和
  元数据写回。
- 前端新增 `citizencode/frontend/citizenpassport/`,迁入 `CpmsRegisterModal.tsx`、
  `CpmsSitePanel.tsx`,并新增 `citizenpassport/api.ts`。
- 前端 `institutions/api.ts` 已移除 CPMS API;机构详情页改从 `citizenpassport/` 引用。
- 前端联邦管理员旧 `chain_sheng_admins.ts`、`chain_sheng_admins_types.ts`
  已拆为 `roster_api.ts`、`signing_keys_api.ts`、`types.ts`。
- 旧本地更换联邦管理员前端入口与后端 `PUT /api/v1/admin/federal-registry/:province`
  已下架;正式更换联邦管理员等待链上主备交换能力后重建。
- 已更新 `memory/05-modules/citizencode/backend/CHAIN_TECHNICAL.md`、
  `BACKEND_LAYOUT.md`、`frontend/FRONTEND_LAYOUT.md`、
  `backend/sheng_admins/SHENG_ADMINS_TECHNICAL.md` 和新增
  `backend/citizenpassport/CITIZENPASSPORT_TECHNICAL.md`。
- 已运行 `cargo fmt --manifest-path citizencode/backend/Cargo.toml`。
- 已运行 `cargo check --manifest-path citizencode/backend/Cargo.toml`,通过;仅剩既有
  `citizencode/province.rs` 静态码表 dead_code warning。
- 已运行 `npm run build`,通过;仅剩既有 Vite chunk 体积提示。
- 已扫描旧路径/旧文件名残留;剩余命中仅为文档中“已删除/不得引用/历史搬迁”的说明。
