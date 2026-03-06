# SUPER_ADMINS_TECHNICAL

## 1. 模块目标
- 本模块负责后台治理能力，覆盖三类资源：
1. 省级超级管理员目录（查询/更换）。
2. 操作管理员（列表/新增/修改/删除/启停）。
3. 机构管理（身份识别码生成、扫码录入公钥、查询、更新、禁用、撤销、删除）。

- 架构口径（冻结）：
1. 管理员功能与机构管理功能继续保持在 `super-admins` 模块内迭代。
2. 不新增独立“管理员模块”或“机构模块”。

- 代码归档：
1. `backend/src/super-admins/catalog.rs`：省级超级管理员目录治理。
2. `backend/src/super-admins/operators.rs`：操作管理员治理。
3. `backend/src/super-admins/institutions.rs`：机构身份识别码与机构公钥治理。

## 2. 权限口径（当前冻结）
1. `SUPER_ADMIN`：
   - 可管理操作管理员（含按创建者隔离）。
   - 可管理机构（生成机构身份识别码、扫码录入机构公钥、更新/禁用/撤销/删除、查询）。
2. `KEY_ADMIN`：
   - 可管理操作管理员。
   - 可查询并更换 43 省超级管理员。
   - 不具备机构管理权限。
3. `OPERATOR_ADMIN`：
   - 不属于本模块治理角色；仅可执行绑定/解绑/状态扫码等业务接口。

## 3. 省域隔离口径
1. `SUPER_ADMIN` 受 `admin_province` 约束，不能管理外省机构/外省业务数据。
2. `KEY_ADMIN` 不受省域隔离，但不开放机构管理接口。
3. 操作管理员列表与治理：`SUPER_ADMIN` 仅能操作自己创建的操作管理员；`KEY_ADMIN` 可跨创建者治理。

## 4. API 矩阵（已实现）

### 4.1 省级超级管理员目录（catalog）
1. `GET /api/v1/admin/super-admins`
   - 权限：`KEY_ADMIN`
   - 功能：查询省级超级管理员列表。
2. `PUT /api/v1/admin/super-admins/:province`
   - 权限：`KEY_ADMIN`
   - 功能：按省更换超级管理员公钥；迁移相关操作管理员 `created_by`；写审计 `SUPER_ADMIN_REPLACE`。

### 4.2 操作管理员（operators）
1. `GET /api/v1/admin/operators`
   - 权限：`SUPER_ADMIN | KEY_ADMIN`
   - 范围：`SUPER_ADMIN` 仅看自己创建；`KEY_ADMIN` 看全局。
   - 返回：包含 `admin_name`（操作管理员姓名）与 `created_by_name`（创建者显示名）。
2. `POST /api/v1/admin/operators`
   - 权限：`SUPER_ADMIN | KEY_ADMIN`
   - 功能：新增操作管理员（默认 `ACTIVE`）；写审计 `OPERATOR_CREATE`。
   - 必填字段：`admin_name` + `admin_pubkey`。
   - 输入校验：`admin_pubkey` 必须是有效 `sr25519` 公钥（hex/0x hex 或可解析 SS58）。
3. `PUT /api/v1/admin/operators/:id`
   - 权限：`SUPER_ADMIN | KEY_ADMIN`
   - 功能：修改操作管理员信息（姓名与公钥）；写审计 `OPERATOR_UPDATE`。
   - 输入校验：若提交新公钥，必须通过 `sr25519` 公钥格式校验；若提交姓名，不能为空字符串。
4. `DELETE /api/v1/admin/operators/:id`
   - 权限：`SUPER_ADMIN | KEY_ADMIN`
   - 功能：删除操作管理员；写审计 `OPERATOR_DELETE`。
5. `PUT /api/v1/admin/operators/:id/status`
   - 权限：`SUPER_ADMIN | KEY_ADMIN`
   - 功能：启用/停用；写审计 `OPERATOR_STATUS_UPDATE`。

### 4.3 机构管理（institutions）
1. `GET /api/v1/admin/cpms-keys`
   - 权限：`SUPER_ADMIN`
   - 范围：仅本省机构。
2. `POST /api/v1/admin/cpms-keys/sfid/generate`
   - 权限：`SUPER_ADMIN`
   - 功能：调用 `sfid` 生成机构身份识别码（`A3=GFR`,`P1=0`），并生成 SFID 签名初始化二维码。
3. `POST /api/v1/admin/cpms-keys/register-scan`
   - 权限：`SUPER_ADMIN`
   - 功能：扫码录入 CPMS 初始化后产生的机构公钥二维码；写审计 `CPMS_KEYS_REGISTER_SCAN`。
4. `PUT /api/v1/admin/cpms-keys/:site_sfid`
   - 权限：`SUPER_ADMIN`
   - 功能：更新机构三把公钥并恢复 `ACTIVE`；写审计 `CPMS_KEYS_UPDATE`。
5. `PUT /api/v1/admin/cpms-keys/:site_sfid/disable`
   - 权限：`SUPER_ADMIN`
   - 功能：机构状态置为 `DISABLED`；写审计 `CPMS_KEYS_STATUS_UPDATE`。
6. `PUT /api/v1/admin/cpms-keys/:site_sfid/revoke`
   - 权限：`SUPER_ADMIN`
   - 功能：机构状态置为 `REVOKED`；写审计 `CPMS_KEYS_STATUS_UPDATE`。
7. `DELETE /api/v1/admin/cpms-keys/:site_sfid`
   - 权限：`SUPER_ADMIN`
   - 功能：删除机构记录；写审计 `CPMS_KEYS_DELETE`。

## 5. 机构数据模型
`CpmsSiteKeys` 关键字段：
1. `site_sfid`
2. `pubkey_1 | pubkey_2 | pubkey_3`
3. `status`：`PENDING | ACTIVE | DISABLED | REVOKED`
4. `version`：内部版本号（用于状态/更新追踪）
5. `last_register_issued_at`
6. `init_qr_payload`：SFID 侧签发的机构初始化二维码原文（用于后续登记校验）
7. `admin_province`
8. `created_by | created_at`
9. `updated_by | updated_at`

## 6. 关键流程（机构）

### 6.1 生成机构身份识别码
1. `SUPER_ADMIN` 在机构管理页发起“生成身份识别码”。
2. 后端调用 `sfid` 生成 `site_sfid`：
   - 不输入机构公钥。
   - `A3` 固定 `GFR`，`P1` 固定 `0`。
3. 后端生成 `purpose=cpms_init` 的 SFID 签名二维码（`qr_payload`）。
4. 后端写入机构记录为 `PENDING`，并保存 `init_qr_payload`。

### 6.2 CPMS 初始化与扫码录入机构
1. 用户携带 SFID 系统生成的机构二维码去 CPMS 做初始化。
2. CPMS 用该二维码完成初始化并生成自身机构公钥登记二维码（含 3 把公钥 + `init_qr_payload` 绑定信息）。
3. `SUPER_ADMIN` 在 SFID 机构页扫码录入。
4. SFID 校验点：
   - `register` 二维码结构与时间窗口。
   - `checksum` 必须绑定 `init_qr_payload` 哈希。
   - `init_qr_payload` 必须是 SFID 可信签名公钥签发且验签通过。
   - `site_sfid` 必须已存在且当前为 `PENDING`。
   - 录入时提交的 `init_qr_payload` 必须与该 `site_sfid` 生成阶段保存值一致。
5. 校验通过后机构置为 `ACTIVE`，写入 3 把公钥。

## 7. 前端对接口径（机构页）
1. 列表列名为“身份识别码”，展示 `site_sfid`。
2. 行内支持小二维码预览与下载。
3. 顶部按钮为“生成身份识别码”（不显示“扫码录入机构”顶栏按钮）。
4. 行操作保留“禁用、删除、扫码”；去掉“撤销”按钮。
5. 每个公钥列单独提供“更新”按钮。
6. “登记人”显示为“`{省名}超级管理员`”。
7. 不展示“版本”列。

## 8. 前端对接口径（管理员页）
1. 管理员列表“创建者”显示创建者名称（优先 `created_by_name`），不再展示纯公钥。
2. 管理员列表“序号”显示为分页连续序号（1,2,3...），不直接使用数据库 `id`。
3. 管理员列表新增“姓名”列，展示 `admin_name`。
4. 新增管理员弹窗为同一行双输入框：姓名在前、公钥在后；点击确认后提交 `admin_name + admin_pubkey`。
5. “修改”按钮弹窗支持同时修改姓名与公钥。
6. 新增/修改管理员时后端会对 `admin_pubkey` 做强校验；非法公钥返回参数错误。

## 9. 安全与一致性
1. 机构接口统一使用 `require_super_admin`。
2. 省域隔离由 `in_scope_cpms_site` 强校验。
3. 机构登记二维码有防重放 token（24 小时窗口）。
4. 只有 `ACTIVE` 机构可用于后续 CPMS 业务二维码验签。

## 10. 审计事件
1. `SUPER_ADMIN_REPLACE`
2. `OPERATOR_CREATE`
3. `OPERATOR_UPDATE`
4. `OPERATOR_DELETE`
5. `OPERATOR_STATUS_UPDATE`
6. `CPMS_SFID_GENERATE`
7. `CPMS_KEYS_REGISTER_SCAN`
8. `CPMS_KEYS_UPDATE`
9. `CPMS_KEYS_STATUS_UPDATE`
10. `CPMS_KEYS_DELETE`

## 11. 路由挂载与文件索引
1. 路由定义：`backend/src/main.rs`（`/api/v1/admin/*`）。
2. 模块导出：`backend/src/super-admins/mod.rs`。
3. 业务实现：
   - `backend/src/super-admins/catalog.rs`
   - `backend/src/super-admins/operators.rs`
   - `backend/src/super-admins/institutions.rs`
4. 省域判定：`backend/src/business/scope.rs`
5. CPMS 状态扫码联动：`backend/src/operate/status.rs`
