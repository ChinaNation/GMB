# 任务卡：Phase 2 — 废弃 CID 管理员直写,改冷签 QR

- 任务编号：20260621-admins-action-deprecate
- 状态：open
- 所属模块：citizencode/backend/admins + citizencode/frontend + CitizenWallet
- 当前负责人：CID Agent（链交互）+ Mobile Agent（钱包 decoder）
- 创建时间：2026-06-21

## 任务需求

清除"第二真源":废弃 `actions.rs` 里直写 postgres `admins` 的注册局管理员增删路径,改为生成 `propose_admin_set_change` / `propose_create_institution` 的 CITIZEN_QR_V1 凭证,由 CitizenWallet 冷签提交,CID 不代签。详见 ADR-023。

## 落地内容

- `citizencode/backend/admins/actions.rs`：`apply_create_federal_registry_conn`/`apply_delete_federal_registry_conn`/`apply_create_city_registry_conn`/`apply_delete_city_registry_conn` 改为生成 QR payload 或返回引导错误;`apply_update_*`（昵称）保留。
- `citizencode/backend/admins/operation_auth.rs`：`CreateFederalRegistry/DeleteFederalRegistry/CreateCityRegistry/DeleteCityRegistry` 四变体删除或标记 deprecated。
- `citizencode/frontend`：管理员增删页改"生成 QR → CitizenWallet 扫码冷签 → 等 indexer 同步"。
- CitizenWallet：确认 `propose_admin_set_change`（AdminsChange call_index=0）有 decoder 分支,无则补。

## 必须遵守（评审 CRITICAL-1,阻塞,必须先于本卡完成）

- **市注册局省份反查必须先从 `created_by` JOIN 改成 `admins.city_name → china.sqlite → province`**（`repo.rs:135-176, 278-283`）。否则 Phase 2 一旦让 indexer 用 `created_by='SYSTEM'` 写市注册局 admin,现有 JOIN 断裂 → 所有市注册局管理员 scope=None → 403。这一步是本卡的前置依赖,放在 Phase 1 收尾或本卡开头先做。

## 输出物

- 代码 + 中文注释 + 文档更新 + 残留清理

## 待确认问题

- 暂无（依赖 Phase 1 完成）
