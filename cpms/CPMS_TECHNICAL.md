# CPMS Technical Notes

## 0. 系统概述
CPMS（Citizen Passport Management System）是市公安局使用的公民档案管理系统，后端使用 Rust/Axum，数据使用 PostgreSQL 持久化。

当前实现基线：
- 管理员 Sr25519 challenge-response 登录，支持扫码登录。
- 角色访问控制：`SUPER_ADMIN / OPERATOR_ADMIN`。
- 消费 SFID 签发的 `SFID_CPMS_V1 / INSTALL` 安装码。
- CPMS 通用发行版只内置编译后的只读行政区数据，安装码决定运行实例所属市公安局。
- 安装后生成 `SFID_CPMS_V1 / ARCHIVE` 公民档案二维码；签出前必须先绑定用户投票账户。
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
| 密码学 | schnorrkel, blake2, sha2, aes-gcm |
| 部署 | Docker / systemd |

## 2. 模块结构
| 模块 | 文件 | 说明 |
|------|------|------|
| main | `src/main.rs` | 入口、路由、公共工具函数、过期数据清理 |
| authz | `src/authz/mod.rs` | Bearer token 校验、角色检查 |
| login | `src/login/mod.rs` | Sr25519 challenge-response 登录、扫码登录 |
| initialize | `src/initialize/mod.rs` | INSTALL 初始化、ARCHIVE 签发密钥、超级管理员绑定 |
| sfid_tool_province | `src/main.rs` | 编译期直接引用 SFID 系统 `sfid/backend/sfid/province.rs` 行政区唯一源 |
| address | `src/address.rs` | 按安装码所属市重建镇/村路地址表并提供查询接口 |
| super_admin | `src/super_admin/mod.rs` | 操作员 CRUD、公民状态变更 |
| operator_admin | `src/operator_admin/mod.rs` | 档案创建/查询、软删除、ARCHIVE 更新/打印 |
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
- `POST /api/v1/archives/:archive_id/delete/challenge`
- `POST /api/v1/archives/:archive_id/delete/complete`

## 4. 两码流程
1. SFID 生成 `SFID_CPMS_V1 / INSTALL`。
2. CPMS 离线解析 INSTALL 并保存 `sfid_number / province_name / city_name / install_secret`，省市代码仅从 `sfid_number` 内部解码。
3. CPMS 根据 INSTALL 的 R5 段从内置 SFID 工具行政区数据中重建当前市镇/村路表。
4. CPMS 生成本机 `ARCHIVE` 签发密钥，公钥保存为 `cpms_pubkey`。
5. CPMS 创建档案时生成全局随机档案号。
6. 用户在 wumin 电子护照页出示投票账户地址二维码，CPMS 扫描后保存投票账户。
7. CPMS 生成携带钱包地址/公钥的 ARCHIVE 二维码。
8. SFID 扫描 ARCHIVE，解 `geo_seal`、验 CPMS 签名，并在 SFID 绑定阶段要求 wumin 钱包签名。

## 5. 数据库表
| 表 | 说明 |
|------|------|
| `system_install` | INSTALL 安装授权状态 |
| `qr_sign_keys` | ARCHIVE 签发密钥 |
| `admin_users` | 管理员 |
| `sessions` | Bearer token 会话 |
| `login_challenges` | 登录挑战 |
| `qr_login_results` | 扫码登录结果 |
| `archives` | 公民档案、投票账户与 `archive_qr_payload` |
| `archive_delete_challenges` | 档案软删除前的 wumin 签名挑战 |
| `address_towns` | 当前 CPMS 实例所属市的镇/街道 |
| `address_villages` | 当前 CPMS 实例所属市的村/路 |
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

## 9. 管理员权限

CPMS 只有两种管理员:

| 功能 | `SUPER_ADMIN` | `OPERATOR_ADMIN` |
|------|------|------|
| 查看公民列表 | 可以 | 可以 |
| 创建/编辑公民档案 | 可以 | 可以 |
| 绑定/更换投票账户 | 可以 | 可以 |
| 生成/打印档案码 | 可以 | 可以 |
| 修改公民状态 | 可以 | 可以 |
| 删除公民档案 | 可以 | 可以 |
| 查看系统设置 | 可以 | 不可以 |
| 创建/停用/删除操作员 | 可以 | 不可以 |

后端档案业务接口统一使用 `authz::require_archive_admin`，允许超级管理员和操作员管理员。
系统设置、操作员管理继续使用 `SUPER_ADMIN` 专属权限。超级管理员是上级角色，不能被日常
档案业务接口挡在外面。

公民姓名字段统一为 `last_name / first_name`，前端、后端和数据库不再使用 `full_name`
作为新接口字段。公民信息列表搜索使用 `q` 参数，同时匹配 `last_name`、`first_name`、
`last_name || first_name` 与完整档案号 `archive_no`。
列表展示完整档案号和由出生日期计算的年龄，省份不在列表中重复展示；省市在详情页展示。

## 9.1 档案软删除

公民档案删除不是物理删除。详情页点击“删除”后，CPMS 后端创建 `WUMIN_QR_V1 / sign_request`
删除签名请求，当前登录管理员必须使用 **wumin** 扫码签名。前端扫描 `sign_response` 后提交
`delete/complete`，后端校验:

- challenge 未过期、未消费，且绑定当前档案和当前登录管理员。
- `sign_response.pubkey` 等于当前登录管理员 `admin_pubkey`。
- `payload_hash` 等于删除 payload 的 SHA-256。
- sr25519 签名验证通过。

通过后 `archives.status` 更新为 `DELETED`，并记录 `deleted_at / deleted_by / delete_reason`。
列表默认隐藏已删除档案；已删除档案不能继续编辑、绑定投票账户、更新档案码、下载或打印。

## 10. 投票账户

ARCHIVE 投票账户字段、签名原文和签出流程见
`memory/05-modules/cpms/ARCHIVE_WALLET_PROOF.md`。无钱包地址的档案不得签出
ARCHIVE。档案详情页档案码操作统一显示为“更新 / 下载 / 打印”；“更新”表示刷新当前
ARCHIVE 二维码，不再使用“生成档案码”作为按钮文案。

## 11. 行政区数据

- SFID 系统 `sfid/backend/sfid` 是行政区数据唯一源头。
- CPMS 后端源码目录不保存行政区第二份文件，也不维护 `province.rs` 或 `city_codes/*.rs`
  的第二份源码。
- CPMS 后端编译期直接引用 `sfid/backend/sfid/province.rs`；该文件继续引用同目录
  `city_codes/*.rs`。发行包只内置编译后的只读数据。
- `cpms/scripts/build_linux_host_installer.sh` 不再执行行政区源码复制脚本；任何恢复复制脚本
  或恢复 CPMS 行政区第二份源码的改动都属于残留回退。
- 一个 CPMS 通用发行包可以安装到任意市公安局；运行时由 SFID 签发的 INSTALL 安装码锁定
  唯一市公安局。
- CPMS 初始化和已初始化实例启动时会按安装码 R5 段重建 `address_towns/address_villages`，
  地址接口只返回当前市数据。
- 公民档案的出生日期、性别、身高均为必填；出生日期固定 `YYYY-MM-DD`，身高范围为
  `30-260 cm`。
