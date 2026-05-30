# SYSTEM ADMINS 技术文档（原 OPERATOR ADMINS）

## 1. 模块定位

- 路径：`backend/shi-admins`
- 职责：市级管理员（`ShiAdmin`，原 `OperatorAdmin`）角色入口模块。
- 设计：本模块保持”轻路由适配”职责，核心业务实现下沉到业务模块，避免角色目录重复实现。
- 角色变更说明：`OPERATOR_ADMIN`（操作管理员）已重命名为 `SHI_ADMIN`（市级管理员），负责日常操作。

## 2. 当前接口职责

- 本目录不再承接 CPMS 公民状态扫码入口。
- CPMS 年度报告导入统一归属 `citizens/status_export_import.rs`，路由为 `POST /api/v1/admin/citizens/cpms-status-export/import`，开放给所有管理员。

## 3. 依赖关系

- 上游：
  - `main.rs` 路由层将管理员接口绑定到 `shi-admins`。
- 下游：
  - 公共能力：鉴权、审计、状态存储（由 `crate::*` 提供）

## 4. 边界说明

- `shi-admins` 仅承接”市级管理员入口语义”，不直接维护底层存储和签名算法。
- 若后续新增市级管理员接口，优先保持”入口薄层 + 业务下沉”模式。
