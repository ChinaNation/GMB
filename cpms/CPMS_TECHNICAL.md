# CPMS Technical Notes

## 0. 系统概述
CPMS（Citizen Passport Management System）是市公安局使用的公民档案管理系统，后端使用 Rust/Axum，数据使用 PostgreSQL 持久化。

当前实现基线：
- 管理员 Sr25519 challenge-response 登录，支持扫码登录。
- 角色访问控制：`SUPER_ADMIN / OPERATOR_ADMIN`。
- 消费 SFID 签发的 `SFID_CPMS_V1 / INSTALL` 安装码。
- 安装后生成 `SFID_CPMS_V1 / ARCHIVE` 公民档案二维码；签出前必须先绑定用户钱包账户。
- ARCHIVE 包含档案号、档案状态、电子护照有效期、CPMS 签发公钥、`geo_seal`、钱包地址/公钥和签名，不包含状态更新时间、`code_id` 或使用次数。
- 档案号不暴露省、市、机构号。
- 档案号格式为 `<26位Base32>-<2位Base32校验>`，不带固定业务前缀。
- ARCHIVE 明文不暴露省、市、机构号，归属信息写入加密 `geo_seal`。
- 操作审计日志。

## 1. 技术栈
| 组件 | 技术 |
|------|------|
| 语言 | Rust 2021 |
| Web 框架 | Axum 0.7 |
| 异步运行时 | Tokio |
| 数据库 | PostgreSQL 16 / sqlx 0.8 |
| 密码学 | schnorrkel, blake2, aes-gcm |
| 部署 | Docker / systemd |

## 2. 模块结构
| 模块 | 文件 | 说明 |
|------|------|------|
| main | `src/main.rs` | 入口、路由、公共工具函数、过期数据清理 |
| authz | `src/authz/mod.rs` | Bearer token 校验、角色检查 |
| login | `src/login/mod.rs` | Sr25519 challenge-response 登录、扫码登录 |
| initialize | `src/initialize/mod.rs` | INSTALL 初始化、ARCHIVE 签发密钥、超级管理员绑定 |
| super_admin | `src/super_admin/mod.rs` | 操作员 CRUD、公民状态变更 |
| operator_admin | `src/operator_admin/mod.rs` | 档案创建/查询、ARCHIVE 生成/打印 |
| dangan | `src/dangan/mod.rs` | 档案号生成、`geo_seal`、ARCHIVE 签名 |

## 3. API 清单
系统初始化：
- `GET /api/v1/install/status`
- `POST /api/v1/install/initialize`
- `POST /api/v1/install/super-admin/bind`

认证：
- `POST /api/v1/admin/auth/identify`
- `POST /api/v1/admin/auth/challenge`
- `POST /api/v1/admin/auth/verify`
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `POST /api/v1/admin/auth/logout`

管理：
- `GET /POST /api/v1/admin/operators`
- `PUT /DELETE /api/v1/admin/operators/:id`
- `PUT /api/v1/admin/operators/:id/status`
- `PUT /api/v1/archives/:archive_id/citizen-status`

档案：
- `POST /GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`
- `POST /api/v1/archives/:archive_id/wallet`
- `POST /api/v1/archives/:archive_id/qr/generate`
- `POST /api/v1/archives/:archive_id/qr/print`

## 4. 两码流程
1. SFID 生成 `SFID_CPMS_V1 / INSTALL`。
2. CPMS 离线解析 INSTALL 并保存 `sfid_number / province_name / city_name / install_secret`，省市代码仅从 `sfid_number` 内部解码。
3. CPMS 生成本机 `ARCHIVE` 签发密钥，公钥保存为 `cpms_pubkey`。
4. CPMS 创建档案时生成全局随机档案号。
5. 用户在 wuminapp 电子护照页出示钱包地址二维码，CPMS 扫描后保存钱包账户。
6. CPMS 生成携带钱包地址/公钥的 ARCHIVE 二维码。
7. SFID 扫描 ARCHIVE，解 `geo_seal`、验 CPMS 签名，并在 SFID 绑定阶段要求 wuminapp 钱包签名。

## 5. 数据库表
| 表 | 说明 |
|------|------|
| `system_install` | INSTALL 安装授权状态 |
| `qr_sign_keys` | ARCHIVE 签发密钥 |
| `admin_users` | 管理员 |
| `sessions` | Bearer token 会话 |
| `login_challenges` | 登录挑战 |
| `qr_login_results` | 扫码登录结果 |
| `archives` | 公民档案、钱包账户与 `archive_qr_payload` |
| `sequence_counters` | 本机序列 |
| `qr_print_records` | 打印记录 |
| `audit_logs` | 操作审计日志 |

本阶段不提供旧库迁移兼容；旧库可删除后按当前基准结构初始化。

## 6. 环境变量
| 变量 | 说明 | 默认值 |
|------|------|--------|
| `CPMS_DATABASE_URL` | PostgreSQL 连接串 | `postgres://cpms:cpms@127.0.0.1:5433/cpms_dev` |
| `CPMS_BIND` | 监听地址 | `0.0.0.0:8080` |
| `CPMS_KEY_ENCRYPT_SECRET` | 本机密钥加密主密钥，32 字节 hex | 无 |

## 7. 验证
- `cargo fmt && cargo check && cargo test`
- `cd frontend/web && npm run build`

## 8. 错误码

CPMS 后端错误响应包含数字 `code` 和稳定业务 `error_code`。HTTP `401` 只表示当前
管理员登录态无效;challenge 过期返回 `410`,签名验签失败返回 `422`,管理员停用返回
`403`。完整规则见 `memory/05-modules/cpms/ERROR_CODES.md`。

## 9. 钱包账户

ARCHIVE 钱包账户字段、签名原文和签出流程见
`memory/05-modules/cpms/ARCHIVE_WALLET_PROOF.md`。无钱包地址的档案不得签出
ARCHIVE。
