# CPMS 技术开发文档（SFID_CPMS_V1 两码基线）

## 1. 系统定位
CPMS 是市公安局使用的离线公民档案管理系统。当前基线只保留两个 SFID/CPMS 业务二维码：

- `INSTALL`：SFID 签发给 CPMS，用于安装初始化。
- `ARCHIVE`：CPMS 签发给 SFID，用于档案录入、公民绑定和省市归档。

协议名固定为 `SFID_CPMS_V1`。机构 SFID 字段固定为 `sfid_number`。

## 2. 后端模块
- `main.rs`：应用启动、PostgreSQL 连接池、迁移、通用响应与审计。
- `initialize/`：消费 INSTALL 安装码、保存安装授权材料、生成 ARCHIVE 签发密钥、绑定超级管理员。
- `login/`：QR-only 扫码登录、会话查询和登出。
- `authz/`：HttpOnly Cookie session 校验与角色检查。
- `super_admin/`：操作员新增/删除和年度状态导出入口。
- `operator_admin/`：档案创建、查询、更新、软删除、ARCHIVE 生成和打印记录。
- `dangan/`：档案号生成、100 年硬删除、年度状态导出、`geo_seal` 加密、ARCHIVE 签名。
- `address.rs`：镇、村/路地址维护。

## 2.1 前端模块
- `cpms/frontend`：Vite 工程根目录，不再保留 `web` 或 `src` 包装层。
- `initialize/`：安装初始化页面、初始化 API 和安装状态类型。
- `login/`：QR-only 登录页面和登录 API。
- `authz/`：登录态上下文与路由守卫。
- `super_admin/`：超级管理员页面、操作员管理、年度报告导出 API 和类型。
- `operator_admin/`：档案列表、创建、详情、编辑、软删除签名、档案 QR 操作 API 和类型。
- `address/`：镇村查询 API 和类型。
- `qr/`：WUMIN_QR_V1 解析与浏览器扫码工具。
- `common/`：通用 HTTP 封装、共享类型和基础布局组件。

## 3. 初始化流程
1. CPMS 扫描或粘贴 SFID 生成的 `SFID_CPMS_V1 / INSTALL`。
2. 后端校验 `proto/type`、`sfid_number`、省市名称和 `install_secret`，离线保存 INSTALL 安装材料。
3. 后端写入 `system_install`：
   - `sfid_number`
   - `install_secret`
   - `install_secret_hash`
   - `province_name / city_name`
   - `cpms_pubkey`
4. 后端生成一把本机 ARCHIVE 签发密钥，写入 `qr_sign_keys.key_id = ARCHIVE`。
5. 绑定 1 个超级管理员。
6. CPMS 可直接创建档案并生成 ARCHIVE 档案二维码。

本阶段不提供旧库迁移兼容；旧数据库可清空后按新基准结构启动。

## 4. 角色模型
- `SUPER_ADMIN`：绑定产生，负责操作员管理、地址维护、公民状态和选举资格维护。
- `OPERATOR_ADMIN`：由超级管理员创建，负责档案录入、查询和二维码打印。

所有业务接口均由后端校验角色，不依赖前端按钮隐藏。

## 5. ARCHIVE 档案号
- 格式：`<26位Base32>-<2位Base32校验>`。
- 不编码省、市、机构、日期。
- 不使用固定业务前缀，避免把示例前缀固化成协议含义。
- 生成输入包含 `install_secret`、安全随机数、本机序列、终端 ID、管理员公钥。
- CPMS 本机用唯一索引避免重复；SFID 录入时以 `ano` 做全局唯一最终校验。

## 6. ARCHIVE 二维码

```json
{
  "proto": "SFID_CPMS_V1",
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

- `sfid_number`

`geo_seal` 使用 AES-256-GCM，加密密钥为 `blake2b_256(install_secret)`。
`geo_seal` AAD 固定为 `sfid-cpms-v1|geo-seal|{ano}|{cpms_pubkey}`，用于绑定档案号和 CPMS 本机公钥。

ARCHIVE 签名原文：

```text
sfid-cpms-v1|archive|{ano}|{cs}|{ve}|{cpms_pubkey}|{geo_seal_hash}
```

签名上下文：`substrate`。

## 7. 数据库表
- `system_install`：单行安装授权状态。
- `qr_sign_keys`：本机 ARCHIVE 签发密钥。
- `admin_users`：管理员账号。
- `sessions`：登录会话。
- `login_challenges`：登录挑战。
- `qr_login_results`：扫码登录结果。
- `archives`：公民档案和 `archive_qr_payload`。
- `sequence_counters`：本机序列。
- `qr_print_records`：打印记录。
- `address_towns` / `address_villages`：地址维护。
- `audit_logs`：审计日志。

## 8. 环境变量
- `CPMS_DATABASE_URL`：PostgreSQL 连接串。
- `CPMS_BIND`：监听地址。
- `CPMS_KEY_ENCRYPT_SECRET`：本机密钥加密主密钥，32 字节 hex。

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

超级管理员：
- `GET /api/v1/admin/operators`
- `POST /api/v1/admin/operators`
- `DELETE /api/v1/admin/operators/:id`
- `GET /api/v1/archives/status-export`

操作员：
- `POST /api/v1/archives`
- `GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`
- `PUT /api/v1/archives/:archive_id`
- `POST /api/v1/archives/:archive_id/wallet`
- `POST /api/v1/archives/:archive_id/qr/generate`
- `POST /api/v1/archives/:archive_id/qr/print`
- `POST /api/v1/archives/:archive_id/delete/challenge`
- `POST /api/v1/archives/:archive_id/delete/complete`

地址：
- `GET /api/v1/address/towns`
- `GET /api/v1/address/villages`

## 9. 安全边界
- 未经 SFID 签发 INSTALL 的 CPMS 无法产生可被 SFID 验证通过的 ARCHIVE。
- 伪 CPMS 即使仿造 ARCHIVE 明文字段，也无法构造正确 `geo_seal / sig`。
- 其他 CPMS 和普通扫码方不能从 ARCHIVE 明文字段看出档案号属于哪个市。
- CPMS 不直接连接 SFID 在线接口，不直接对接区块链。

## 10. 配置项
- `CPMS_BIND`：服务监听地址，默认 `0.0.0.0:8080`。
- `CPMS_DATABASE_URL`：PostgreSQL 连接串，优先级高于 `DATABASE_URL`。
- `DATABASE_URL`：PostgreSQL 连接串兜底配置。
- `CPMS_KEY_ENCRYPT_SECRET`：本机密钥加密主密钥，32 字节 hex。

## 11. 验证命令
- `cd cpms/backend && cargo fmt && cargo check && cargo test`
- `cd cpms/frontend && npm run build`
