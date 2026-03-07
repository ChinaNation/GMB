# BUSINESS 模块技术文档

## 1. 模块定位

- 路径：`backend/src/business`
- 职责：承载“共用后台查询、审计、公钥归一化与作用域判定能力”。
- 当前不再承载操作业务与区块链业务：
  - 操作业务在 `backend/src/operate`
  - 区块链业务在 `backend/src/chain`
  - SFID 管理端业务在 `backend/src/sfid/admin.rs`

## 2. 模块结构

- `audit.rs`
  - `admin_list_audit_logs`
  - 用途：管理员审计日志查询
- `query.rs`
  - `admin_list_citizens`
  - `admin_query_by_pubkey`
  - `public_identity_search`
  - 用途：后台查询与公开身份查询
- `scope.rs`
  - `province_scope_for_role`
  - `in_scope`
  - `in_scope_pending`
  - `in_scope_cpms_site`
  - 用途：角色作用域与省域隔离判定
- `pubkey.rs`
  - `normalize_admin_pubkey`
  - `normalize_cpms_pubkey`
  - `same_admin_pubkey`
  - `same_cpms_pubkey`
  - 用途：统一 `sr25519` 公钥格式归一化与语义比较，避免多模块重复实现
- `mod.rs`
  - 统一导出 `audit/pubkey/query/scope`

## 3. 路由接线（当前）

- `GET /api/v1/admin/audit-logs` -> `business::audit::admin_list_audit_logs`
- `GET /api/v1/admin/citizens` -> `business::query::admin_list_citizens`
- `GET /api/v1/admin/bind/query` -> `business::query::admin_query_by_pubkey`
- `GET /api/v1/public/identity/search` -> `business::query::public_identity_search`

## 4. 边界与依赖

- 对外边界：
  - 仅提供查询/审计/作用域能力
  - 不处理绑定扫码、状态扫码、链请求签名与链回执
- 主要依赖：
  - `crate::*` 公共能力（鉴权、存储、审计结构）
  - `sfid::province`（省份映射能力，经 `scope.rs` 使用）
