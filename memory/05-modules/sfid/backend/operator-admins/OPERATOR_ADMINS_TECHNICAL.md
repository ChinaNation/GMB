# OPERATOR ADMINS 技术文档

## 1. 模块定位

- 路径：`backend/src/operator-admins`
- 职责：操作管理员（`OperatorAdmin`）角色入口模块。
- 设计：本模块保持“轻路由适配”职责，核心业务实现下沉到业务模块，避免角色目录重复实现。

## 2. 当前接口职责

- `admin_cpms_status_scan`
  - 对应路由：`POST /api/v1/admin/cpms-status/scan`
  - 功能：操作管理员扫描并校验 CPMS 状态二维码。
  - 实现方式：转调 `operate::status::admin_cpms_status_scan`，在操作业务模块中完成权限、验签、审计和响应组装。

## 3. 依赖关系

- 上游：
  - `main.rs` 路由层将管理员接口绑定到 `operator-admins`。
- 下游：
  - `operate/status.rs`（实际业务逻辑）
  - 公共能力：鉴权、审计、状态存储、二维码签名校验（由 `crate::*` 提供）

## 4. 边界说明

- `operator-admins` 仅承接“操作管理员入口语义”，不直接维护底层存储和签名算法。
- 若后续新增操作管理员接口，优先保持“入口薄层 + 业务下沉”模式。
