# CPMS Technical Notes

## 0. 系统概述

CPMS（Citizen Passport Management System）是公民档案/护照管理系统的后端服务，使用 Rust/Axum 实现，PostgreSQL 持久化。

核心能力：
- Sr25519 challenge-response 认证（支持 QR 扫码登录）
- 角色访问控制（SUPER_ADMIN / OPERATOR_ADMIN）
- 公民档案 CRUD 和档案号生成（V3 格式）
- QR 码生成、签名与打印记录
- 操作审计日志

## 1. 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust 2021 |
| Web 框架 | Axum 0.7 |
| 异步运行时 | Tokio |
| 数据库 | PostgreSQL 16 (sqlx 0.8) |
| 密码学 | schnorrkel (Sr25519), blake2 |
| 部署 | Docker / systemd |

## 2. 模块结构

| 模块 | 文件 | 说明 |
|------|------|------|
| main | `src/main.rs` | 入口、路由、公共工具函数、过期数据清理 |
| authz | `src/authz/mod.rs` | Bearer token 校验、角色检查 |
| login | `src/login/mod.rs` | Sr25519 challenge-response 登录、QR 扫码登录 |
| initialize | `src/initialize/mod.rs` | 系统初始化、QR 签名密钥生成（加密存储）、超级管理员绑定 |
| super_admin | `src/super_admin/mod.rs` | 操作员 CRUD、站点密钥注册、公民状态变更 |
| operator_admin | `src/operator_admin/mod.rs` | 档案创建/查询、QR 生成/打印 |
| dangan | `src/dangan/mod.rs` | 档案号生成、QR 载荷签名、省市代码校验 |

## 3. API 清单

### 系统初始化
- `GET /api/v1/install/status` — 初始化状态查询
- `POST /api/v1/install/initialize` — SFID QR 初始化
- `POST /api/v1/install/super-admin/bind` — 超级管理员绑定

### 认证
- `POST /api/v1/admin/auth/identify` — 公钥身份识别
- `POST /api/v1/admin/auth/challenge` — 获取签名挑战
- `POST /api/v1/admin/auth/verify` — 验证签名获取 token
- `POST /api/v1/admin/auth/qr/challenge` — QR 登录挑战
- `POST /api/v1/admin/auth/qr/complete` — QR 登录完成
- `GET /api/v1/admin/auth/qr/result` — QR 登录结果轮询
- `POST /api/v1/admin/auth/logout` — 登出

### 管理（SUPER_ADMIN）
- `GET /POST /api/v1/admin/operators` — 操作员列表/创建
- `PUT /DELETE /api/v1/admin/operators/:id` — 操作员更新/删除
- `PUT /api/v1/admin/operators/:id/status` — 操作员状态变更
- `POST /api/v1/admin/site-keys/registration-qr` — 站点密钥注册 QR
- `PUT /api/v1/archives/:archive_id/citizen-status` — 公民状态变更

### 档案（OPERATOR_ADMIN）
- `POST /GET /api/v1/archives` — 档案创建/列表
- `GET /api/v1/archives/:archive_id` — 档案详情
- `POST /api/v1/archives/:archive_id/qr/generate` — QR 码生成
- `POST /api/v1/archives/:archive_id/qr/print` — QR 打印记录

## 4. 安全设计

### 4.1 认证流程
1. 客户端提交 `admin_pubkey` → 服务端返回 challenge（含 nonce、过期时间）
2. 客户端用 Sr25519 私钥签名 challenge payload → 服务端验签
3. 验签通过 → 发放 Bearer token（CSPRNG 生成 32 字节随机值）
4. 后续请求携带 Bearer token，服务端查 sessions 表校验过期

### 4.2 QR 签名密钥
- 系统初始化时生成 3 把 Sr25519 密钥对（PRIMARY/BACKUP/EMERGENCY）
- 私钥使用环境变量 `CPMS_KEY_ENCRYPT_SECRET`（32 字节 hex）加密后存入 DB
- 加密方式：XOR(secret, blake2_256(master_key || key_id))
- 环境变量不存在时回退明文（日志警告），兼容旧部署

### 4.3 过期数据清理
- 后台任务每 5 分钟清理：过期 sessions、过期 login_challenges、10 分钟前的 qr_login_results

## 5. 数据库表

| 表 | 说明 |
|------|------|
| `system_install` | 系统初始化状态（单行） |
| `qr_sign_keys` | QR 签名密钥（加密存储） |
| `admin_users` | 管理员（SUPER_ADMIN / OPERATOR_ADMIN） |
| `sessions` | Bearer token 会话 |
| `login_challenges` | 登录挑战（challenge-response） |
| `qr_login_results` | QR 登录结果（轮询用） |
| `archives` | 公民档案 |
| `sequence_counters` | 档案号序列计数器 |
| `qr_print_records` | QR 打印记录 |
| `audit_logs` | 操作审计日志 |

## 6. 部署

### 环境变量
| 变量 | 说明 | 默认值 |
|------|------|--------|
| `CPMS_DATABASE_URL` | PostgreSQL 连接串 | `postgres://cpms:cpms@127.0.0.1:5433/cpms_dev` |
| `CPMS_BIND` | 监听地址 | `0.0.0.0:8080` |
| `CPMS_KEY_ENCRYPT_SECRET` | QR 密钥加密主密钥（32 字节 hex） | 无（回退明文） |

### 生产部署要求
- **必须**在前端部署 TLS（nginx/caddy），服务本身仅 HTTP
- **建议**设置 `CPMS_KEY_ENCRYPT_SECRET` 启用密钥加密

## 7. 测试

`cargo test -p cpms-backend` 覆盖：
- Sr25519 签名验证（hex/base64 输入、篡改拒绝、无效编码拒绝）
- QR 签名生成与验证
- 档案号 V3 格式稳定性
- 公民状态校验
