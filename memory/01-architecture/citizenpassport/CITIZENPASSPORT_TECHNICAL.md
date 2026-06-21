# CPMS 技术开发文档（CID_CPMS_V1 两码基线）

## 1. 系统定位
CPMS 是市公安局使用的离线公民档案管理系统。当前基线只保留两个 CID/CPMS 业务二维码：

- `INSTALL`：CID 签发给 CPMS，用于安装初始化。
- `ARCHIVE`：CPMS 签发给 CID，用于档案录入、公民绑定和省市归档。

协议名固定为 `CID_CPMS_V1`。机构 CID 字段固定为 `cid_number`。

## 2. 后端模块
- `main.rs`：应用启动、PostgreSQL 连接池、迁移、通用响应与审计。
- `initialize/`：消费 INSTALL 安装码、保存安装授权材料、生成 ARCHIVE 签发密钥、绑定初始管理员。
- `login/`：QR-only 扫码登录、会话查询和登出。
- `authz/`：HttpOnly Cookie session 校验与角色检查。
- `rate_limit.rs`：登录、初始化、删除签名和资料上传的本机内存限流。
- `admins/`：管理员新增、姓名编辑、删除和年度状态导出入口。
- `dangan/`：档案创建、查询、游标分页、更新、软删除、ARCHIVE 生成和打印记录、公民资料库、100 年硬删除、年度状态导出、`geo_seal` 加密、ARCHIVE 签名。
- `address.rs`：镇、村/路地址维护。

## 2.1 前端模块
- `citizenpassport/frontend`：Vite 工程根目录，不再保留 `web` 或 `src` 包装层。
- `initialize/`：安装初始化页面、初始化 API 和安装状态类型。
- `login/`：QR-only 登录页面和登录 API。
- `authz/`：登录态上下文与路由守卫。
- `admins/`：管理员页面、管理员管理、年度报告导出 API 和类型。
- `dangan/`：档案列表、创建、详情、编辑、公民资料库入口、软删除签名、档案 QR 操作 API 和类型。
- `address/`：镇村查询 API 和类型。
- `qr/`：CITIZEN_QR_V1 解析与浏览器扫码工具。
- `common/`：通用 HTTP 封装、共享类型和基础布局组件；`401` 只通知认证上下文清理用户镜像，不直接改写路由。

## 3. 初始化流程
1. CPMS 扫描或粘贴 CID 生成的 `CID_CPMS_V1 / INSTALL`。
2. 后端校验 `proto/type`、`cid_number`、省市名称和 `install_secret`，离线保存 INSTALL 安装材料。
3. 后端写入 `system_install`：
   - `cid_number`
   - `install_secret`
   - `install_secret_hash`
   - `province_name / city_name`
   - `cpms_pubkey`
4. 后端生成一把本机 ARCHIVE 签发密钥，写入 `qr_sign_keys.key_id = ARCHIVE`。
5. 绑定 1 个初始管理员。
6. CPMS 可直接创建档案并生成 ARCHIVE 档案二维码。

本阶段不提供旧库迁移兼容；旧数据库可清空后按新基准结构启动。
CPMS 正式发布前，开发期 migration 只保留当前完整基线，修改基线后必须清空开发库重建。
正式发布后，已发布 migration 永久冻结，数据库变化只允许新增 migration，由启动时自动执行，
不得清空正式用户数据库。

## 4. 角色模型
- `admins`：市公安局机构管理员在 CPMS 的本地登录镜像，负责管理员管理、地址维护、公民状态、选举资格维护和年度状态导出；总数最多 5 个，初始管理员不可删除。
- `operators`：CPMS 内部操作员，由管理员创建，负责档案录入、查询和二维码打印。

所有业务接口均由后端校验角色，不依赖前端按钮隐藏。
登录态用户镜像包含 `user_id / user_group / admin_display_name`，前端顶部按 `管理员 · 姓名` 或
`操作员 · 姓名` 展示；姓名为空时显示预留名。

档案创建/编辑时，出生日期必须早于当前 UTC 日期；详细地址、公民状态和选举资格为必填。
公民状态为 `REVOKED` 时，选举资格固定为无选举资格；只有 `NORMAL` 状态允许选择选举资格。

## 5. ARCHIVE 档案号
- 格式：`<26位Base32>-<2位Base32校验>`。
- 不编码省、市、机构、日期。
- 不使用固定业务前缀，避免把示例前缀固化成协议含义。
- 生成输入包含 `install_secret`、安全随机数、本机序列、终端 ID、管理员公钥。
- CPMS 本机用唯一索引避免重复；CID 录入时以 `ano` 做全局唯一最终校验。

## 6. ARCHIVE 二维码

```json
{
  "proto": "CID_CPMS_V1",
  "type": "ARCHIVE",
  "ano": "K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "cs": "NORMAL",
  "ve": true,
  "cpms_pubkey": "0x...",
  "geo_seal": "g1.<nonce_hex>.<cipher_hex>",
  "sig": "0x..."
}
```

明文字段不包含省、市、CPMS 机构号。归属信息只存在于 `geo_seal`：

- `cid_number`

`geo_seal` 使用 AES-256-GCM，加密密钥为 `blake2b_256(install_secret)`。
`geo_seal` AAD 固定为 `cid-cpms-v1|geo-seal|{ano}|{cpms_pubkey}`，用于绑定档案号和 CPMS 本机公钥。

ARCHIVE 签名原文：

```text
cid-cpms-v1|archive|{ano}|{cs}|{ve}|{cpms_pubkey}|{geo_seal_hash}
```

签名上下文：`substrate`。

## 7. 数据库表
- `system_install`：单行安装授权状态，显式保存 `cid_number / province_code / city_code`。
- `qr_sign_keys`：本机 ARCHIVE 签发密钥。
- `admin_users`：管理员和操作员账号；不保留停用状态字段，操作员删除即物理删除。
- `sessions`：登录会话。
- `login_challenges`：登录挑战。
- `qr_login_results`：扫码登录结果。
- `archives`：公民档案和 `archive_qr_payload`；`birth_date / valid_from / valid_until` 使用数据库 `DATE`。
- `archive_materials`：公民资料库元数据；文件正文由 `dangan/materials.rs` 保存到本机资料目录。
- `archive_number_recycle_pool`：满 100 年硬删除后释放的档案号和护照号对；只约束未使用号码唯一，允许多轮复用历史。
- `archive_hard_delete_logs`：满 100 年硬删除最小日志，不保存实名原文。
- `cpms_status_exports`：年度状态导出记录和已签名导出 JSON，用于重复下载同一份报告。
- `sequence_counters`：本机序列。
- `qr_print_records`：打印记录。
- `address_towns` / `address_villages`：地址维护。
- `audit_logs`：审计日志。

## 8. 环境变量
- `CPMS_DATABASE_URL`：PostgreSQL 连接串。
- `CPMS_BIND`：监听地址。
- `CPMS_KEY_ENCRYPT_SECRET`：本机密钥加密主密钥，32 字节 hex。
- `CPMS_FRONTEND_DIR`：正式部署前端静态文件目录，设置后必须存在 `index.html`。
- `CPMS_COOKIE_SECURE`：设置为 `true/1/yes` 时给 session Cookie 增加 `Secure`。
- `CPMS_MATERIALS_DIR`：公民资料库文件正文保存目录，默认 `data/archive-materials`。

## 8. API 总览
初始化：
- `GET /api/v1/install/status`
- `POST /api/v1/install/initialize`
- `POST /api/v1/install/super-admin/bind`

登录：
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `GET /api/v1/admin/auth/me`
- `POST /api/v1/admin/auth/logout`

管理员：
- `GET /api/v1/admin/admins`
- `POST /api/v1/admin/admins`
- `PUT /api/v1/admin/admins/:id`
- `DELETE /api/v1/admin/admins/:id`
- `GET /api/v1/archives/status-export/state`
- `GET /api/v1/archives/status-export`

操作员：
- `POST /api/v1/archives`
- `GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`
- `PUT /api/v1/archives/:archive_id`
- `POST /api/v1/archives/:archive_id/wallet`
- `GET /POST /api/v1/archives/:archive_id/materials`
- `GET /api/v1/archives/:archive_id/materials/:material_id/download`
- `DELETE /api/v1/archives/:archive_id/materials/:material_id`
- `POST /api/v1/archives/:archive_id/qr/generate`
- `POST /api/v1/archives/:archive_id/qr/print`
- `POST /api/v1/archives/:archive_id/delete/challenge`
- `POST /api/v1/archives/:archive_id/delete/complete`

地址：
- `GET /api/v1/address/towns`
- `GET /api/v1/address/villages`

## 9. 安全边界
- 未经 CID 签发 INSTALL 的 CPMS 无法产生可被 CID 验证通过的 ARCHIVE。
- 伪 CPMS 即使仿造 ARCHIVE 明文字段，也无法构造正确 `geo_seal / sig`。
- 其他 CPMS 和普通扫码方不能从 ARCHIVE 明文字段看出档案号属于哪个市。
- CPMS 不直接连接 CID 在线接口，不直接对接区块链。
- CPMS 只负责年度状态导出文件生成和操作员逾期锁定；CID 是否收到文件、是否禁用 CPMS 安装码由 CID 系统实现。
- 同一个钱包账户在档案生命周期内只能绑定一个未硬删除档案；软删除不释放钱包账户、档案号或护照号。
- 初始化绑定的管理员不可删除且列表固定第一；后续新增管理员可删除。管理员总数最多 5 个，所有管理员只能编辑姓名。

## 10. 配置项
- `CPMS_BIND`：服务监听地址；正式安装固定为 `127.0.0.1:8080`，由 nginx 提供局域网 HTTPS 入口。
- `CPMS_DATABASE_URL`：PostgreSQL 连接串，优先级高于 `DATABASE_URL`。
- `DATABASE_URL`：PostgreSQL 连接串兜底配置。
- `CPMS_KEY_ENCRYPT_SECRET`：本机密钥加密主密钥，32 字节 hex。
- `CPMS_FRONTEND_DIR`：前端静态文件目录；正式安装脚本写入 `/opt/citizenpassport/frontend`。
- `CPMS_COOKIE_SECURE`：HTTPS 部署时启用 Secure Cookie，内网 HTTP 部署默认关闭。

## 11. 安全运行约束
- 管理员 15 分钟无活动过期，操作员 30 分钟无活动过期；所有请求成功鉴权后按角色滑动续期。
- 登录 QR、安装初始化、初始管理员绑定、删除签名完成和资料上传入口使用本机 IP 级限流，超限返回 `429 / CPMS_RATE_LIMITED`。
- 已初始化实例启动时必须用 `CPMS_KEY_ENCRYPT_SECRET` 解密现有 `install_secret` 和 ARCHIVE 私钥，失败则拒绝启动。
- 后端统一设置 CSP、禁止 iframe 嵌入、禁止 MIME 嗅探、无 referrer 和最小浏览器权限策略。
- 删除档案完成接口在 challenge、签名人、payload hash、过期时间或验签失败时写 `DELETE_ARCHIVE / FAILED` 审计，失败不消费 challenge、不修改档案。

## 12. 部署
- 正式安装包由 `citizenpassport/scripts/build_linux_host_installer.sh` 构建，脚本同时构建后端 release binary 和前端 `dist`。
- `install_host.sh` 把前端静态文件安装到 `/opt/citizenpassport/frontend`，后端通过 `CPMS_FRONTEND_DIR` 直接托管。
- 正式安装不导入 `schema.sql / seed.sql`；安装脚本只创建 PostgreSQL 角色、数据库和 schema 权限，
  数据库结构由后端启动时的 `MIGRATOR.run()` 创建。
- 安装手册随安装包提供，并安装到 `/opt/citizenpassport/docs/CitizenPassport安装配置手册.md`。
- CPMS 不再提供 Docker 部署入口；开发联调使用本机数据库脚本和 Vite dev server。

## 13. 验证命令
- `cd citizenpassport/backend && cargo fmt && cargo check && cargo test`
- `cd citizenpassport/frontend && npm run build`
