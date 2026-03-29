# CPMS 技术开发文档（当前实现基线）

## 1. 文档目的
- 固化 CPMS 当前代码实现的技术基线，作为开发、联调、测试、验收的唯一参考。
- 明确模块边界：登录、初始化、权限、机构管理员、系统管理员、档案算法相互解耦。
- 统一跨端口径：CPMS 与 wuminapp/SFID 的扫码协议、签名原文、字段顺序保持一致。

## 2. 系统定位与业务范围
- CPMS 是离线运行的管理系统，当前实现聚焦“管理员体系 + 档案录入 + 档案二维码签发/打印”。
- 管理员仅两类角色：
  - `INSTITUTION_ADMIN`（机构管理员）：管理系统管理员、管理档案状态、生成机构公钥登记二维码。
  - `SYSTEM_ADMIN`（系统管理员）：录入档案、查询档案、生成并打印档案二维码。
- 业务主线：
  - 机构管理员通过初始化绑定产生（最多 3 个，固定映射 `K1/K2/K3`）。
  - 系统管理员由机构管理员维护。
  - 系统管理员录入档案后生成二维码，交给外部系统（SFID）使用。

## 3. 后端模块架构（`cpms/backend/src`）

### 3.1 模块目录
- `main.rs`：应用启动、PostgreSQL 连接池与迁移、通用错误响应、审计写入。
- `initialize/`：安装初始化与机构管理员绑定。
- `login/`：管理员登录（普通 challenge + 扫码 challenge）。
- `authz/`：Bearer token 鉴权与角色校验。
- `super_admin/`：机构管理员接口（操作员管理、档案状态管理、公钥登记二维码）。
- `operator_admin/`：系统管理员接口（档案录入、查询、二维码生成与打印）。
- `dangan/`：档案号与二维码算法（含 `province_codes.rs` 省市代码数据）。

### 3.2 模块边界
- `login` 只负责登录流程，不承担业务授权和业务操作。
- `authz` 只负责“是否登录 + 角色匹配”判定。
- `super_admin`（机构管理员）与 `operator_admin`（系统管理员）承担业务入口与权限分层。
- `dangan` 只提供算法和载荷构建能力，被业务模块调用。
- `initialize` 统一承载安装引导与初始化安全链路。

## 4. 角色与权限模型

### 4.1 角色定义
- `INSTITUTION_ADMIN`（机构管理员）
  - 来源：安装后通过 `wuminapp` 绑定。
  - 上限：固定 3 个，对应 `K1/K2/K3`。
  - 关键能力：操作员管理、档案状态更新、机构公钥登记二维码生成。
- `SYSTEM_ADMIN`（系统管理员）
  - 来源：由机构管理员创建。
  - 关键能力：档案创建/查询、档案二维码生成与打印。

### 4.2 权限校验实现
- 权限入口在 `authz::require_role(...)`，从 `Authorization: Bearer <token>` 读取会话。
- 会话过期或 token 无效返回 `401`，角色不匹配返回 `403(code=2008)`。
- 所有管理与业务接口均由后端强制校验，不依赖前端按钮隐藏。

## 5. 初始化模块（`initialize/`）

### 5.1 路由
- `GET /api/v1/install/status`
- `POST /api/v1/install/initialize`
- `POST /api/v1/install/super-admin/bind`

### 5.2 初始化流程
1. `install/initialize` 接收 `sfid_init_qr_content`（支持 JSON 或 Base64(JSON)）。
2. 校验 `qr_type=SFID_CPMS_INSTALL`，并使用环境变量 `SFID_ROOT_PUBKEY` 验签。
3. 初始化成功后写入 PostgreSQL：
   - `system_install.site_sfid`
   - `qr_sign_keys`（固定 3 把：`K1/K2/K3`，主/备/应急）
   - 机构管理员绑定信息写入 `admin_users`（`managed_key_id` 标识归属键位）
4. `install/super-admin/bind` 接收 `key_id/admin_pubkey/bind_nonce/signature` 绑定超管：
   - `key_id` 仅允许固定键位。
   - 每个 `key_id` 只能绑定一次，`admin_pubkey` 不可重复。
   - 固定账号映射：`K1->u_super_admin_01`，`K2->u_super_admin_02`，`K3->u_super_admin_03`。

### 5.3 安全约束
- 未设置 `SFID_ROOT_PUBKEY` 时拒绝初始化。
- `system_install.site_sfid` 已存在时拒绝重复初始化。
- 绑定时 `managed_key_id` 与 `admin_pubkey` 全局唯一，防止重复绑定。

## 6. 登录模块（`login/`）

### 6.1 路由
- `POST /api/v1/admin/auth/identify`
- `POST /api/v1/admin/auth/challenge`
- `POST /api/v1/admin/auth/verify`
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `POST /api/v1/admin/auth/logout`

### 6.2 登录形态
- 普通 challenge 登录：`challenge -> signature -> token`。
- 扫码登录：后端生成挑战二维码，手机签名后回传，后端验签并落登录结果，页面轮询拿 token。

### 6.3 与 wuminapp 对齐口径（当前）
- 协议：`WUMIN_LOGIN_V1.0.0`
- 挑战字段：`proto/system/challenge/issued_at/expires_at/sys_pubkey/sys_sig`
- 签名原文固定：

```text
WUMIN_LOGIN_V1.0.0|system|challenge|expires_at
```

- `origin` 不参与移动端扫码验签，可仅作为网页侧上下文保留。
- `sys_pubkey`：CPMS 当前登录系统公钥。
- `sys_sig`：CPMS 对挑战原文的签名。
- 登录协议仅用于 `sfid/cpms` 登录，不用于链上转账、投票或治理签名。
- 手机先验 `sys_pubkey + sys_sig` 确认二维码由 CPMS 私钥签发，再由管理员钱包完成第二层登录签名。

### 6.4 安全约束
- challenge 有效期固定 90 秒。
- challenge 一次性消费，防重放。
- 登录会话默认有效期 30 分钟。
- 管理员状态必须是 `ACTIVE`。

## 7. 机构管理员模块（`super_admin/`）

### 7.1 路由（均要求 `INSTITUTION_ADMIN`）
- `GET /api/v1/admin/operators`
- `POST /api/v1/admin/operators`
- `PUT /api/v1/admin/operators/:id`
- `DELETE /api/v1/admin/operators/:id`
- `PUT /api/v1/admin/operators/:id/status`
- `POST /api/v1/admin/site-keys/registration-qr`
- `PUT /api/v1/archives/:archive_id/citizen-status`

### 7.2 关键行为
- 系统管理员增删改查与状态更新。
- 生成机构公钥登记二维码（`CPMS_SITE_KEYS_REGISTER`）供 SFID 录入。
- 更新档案 `citizen_status`（`NORMAL/ABNORMAL`），并派生 `voting_eligible`。

## 8. 系统管理员模块（`operator_admin/`）

### 8.1 路由（均要求 `SYSTEM_ADMIN`）
- `POST /api/v1/archives`
- `GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`
- `POST /api/v1/archives/:archive_id/qr/generate`
- `POST /api/v1/archives/:archive_id/qr/print`

### 8.2 关键行为
- 创建档案时校验省市代码、出生日期、性别、公民状态。
- 档案号由后端算法生成，前端不可覆盖。
- 支持分页与按姓名模糊查询。
- 二维码生成与打印均记录审计（打印还落 `qr_print_records`）。

## 9. 档案号与二维码算法（`dangan/`）

### 9.1 档案号规则（v3）
- 格式：`省2 + 市3 + 校验1 + 随机9 + 日期8`
- 总长度：23
- 日期：`YYYYMMDD`
- 市级真实代码统一从 `001` 起排；`000` 为 SFID 省级占位码，不用于 CPMS 档案号。

### 9.2 校验位算法
- 输入串：`cpms-archive-v3|{province2}{city3}{random9}{created_date8}`
- 算法：`BLAKE2b` 摘要后做字节和，再 `mod 10` 得 1 位数字。

### 9.3 随机 9 位生成
- 输入因子：`timestamp_ms + terminal_id + admin_pubkey + nonce`
- 通过哈希后 `mod 1_000_000_000`，左补零到 9 位。
- 冲突重试：最多 20 次。

### 9.4 业务二维码签名
- 档案二维码签名原文：

```text
cpms-qr-v1|site_sfid|sign_key_id|archive_no|citizen_status|voting_eligible|issued_at|qr_id
```

- 签名算法：`sr25519`，上下文 `CPMS-QR-SIGN-V1`。

## 10. 数据模型与持久化

### 10.1 当前持久化形态
- 当前实现已完全切换为 PostgreSQL 持久化（无 JSON 运行时快照）。
- 启动时自动执行迁移：`backend/db/migrations/0001_init_cpms_pg.sql`。

### 10.2 核心表
- `system_install`
- `qr_sign_keys`
- `admin_users`
- `sessions`
- `login_challenges`
- `qr_login_results`
- `archives`
- `sequence_counters`
- `qr_print_records`
- `audit_logs`

### 10.3 审计落库时机
- 初始化、登录、管理员管理、档案创建、二维码生成、二维码打印等关键动作均实时写入 `audit_logs`。

## 11. API 总览（当前实现）

### 11.1 健康检查
- `GET /api/v1/health`

### 11.2 初始化
- `GET /api/v1/install/status`
- `POST /api/v1/install/initialize`
- `POST /api/v1/install/super-admin/bind`

### 11.3 登录
- `POST /api/v1/admin/auth/identify`
- `POST /api/v1/admin/auth/challenge`
- `POST /api/v1/admin/auth/verify`
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `POST /api/v1/admin/auth/logout`

### 11.4 机构管理员
- `GET /api/v1/admin/operators`
- `POST /api/v1/admin/operators`
- `PUT /api/v1/admin/operators/:id`
- `DELETE /api/v1/admin/operators/:id`
- `PUT /api/v1/admin/operators/:id/status`
- `POST /api/v1/admin/site-keys/registration-qr`
- `PUT /api/v1/archives/:archive_id/citizen-status`

### 11.5 系统管理员
- `POST /api/v1/archives`
- `GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`
- `POST /api/v1/archives/:archive_id/qr/generate`
- `POST /api/v1/archives/:archive_id/qr/print`

## 12. 配置项（环境变量）
- `CPMS_BIND`：服务监听地址，默认 `0.0.0.0:8080`。
- `CPMS_DATABASE_URL`：PostgreSQL 连接串（优先级高于 `DATABASE_URL`）。
- `DATABASE_URL`：PostgreSQL 连接串兜底配置。
- `SFID_ROOT_PUBKEY`：SFID 初始化二维码验签公钥（初始化必填）。
- `CPMS_LOGIN_SYSTEM_KEY_*`：登录系统公钥/私钥配置（命名待实现统一）。
- `CPMS_LOGIN_SYS_CERT`：SFID 对当前 CPMS 登录系统公钥的背书结果（命名待实现统一）。

## 13. 错误码口径（摘要）
- `1001`：请求参数非法或字段缺失。
- `2001`：token 缺失或无效。
- `2002`：管理员不存在或非 ACTIVE。
- `2003`：challenge 不存在。
- `2004`：challenge 与请求上下文不匹配。
- `2005`：challenge 已消费。
- `2006`：challenge 已过期。
- `2007`：签名校验失败。
- `2008`：权限不足。
- `2009`：token 过期。
- `3001~3005`：管理员/档案业务冲突与不存在等业务错误。
- `4001~4005`：初始化冲突或初始化链路错误。
- `5001+`：服务内部错误。

## 14. 与 wuminapp / SFID 联调要点
- wuminapp 扫码登录验签串必须与 CPMS 完全一致（6 段，不含 `aud/origin`）。
- CPMS 初始化必须基于 SFID 签发的 `SFID_CPMS_INSTALL` 挑战，并通过 `SFID_ROOT_PUBKEY` 验签。
- SFID 录入机构公钥使用 CPMS 生成的 `CPMS_SITE_KEYS_REGISTER` 二维码。
- 业务二维码与机构公钥体系分离于管理员登录公钥体系，不可混用。
- 登录信任链固定为：区块链持有 SFID 当前公钥 -> SFID 背书 CPMS 登录公钥 -> CPMS 签发登录挑战。

## 15. 模块文档索引
- `backend/src/initialize/INITIALIZE_TECHNICAL.md`
- `backend/src/login/LOGIN_TECHNICAL.md`
- `backend/src/dangan/DANGAN_TECHNICAL.md`
- `backend/src/super_admin/mod.rs`
- `backend/src/operator_admin/mod.rs`
- `backend/src/authz/mod.rs`

本文件描述的是“当前实现基线”。若接口、字段、签名串、角色边界、持久化方案发生变更，必须同步更新本文件与对应模块技术文档。
