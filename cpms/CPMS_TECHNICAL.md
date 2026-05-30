# CPMS Technical Notes

## 0. 系统概述
CPMS（Citizen Passport Management System）是市公安局使用的公民档案管理系统，后端使用 Rust/Axum，前端使用 React/Vite，数据使用 PostgreSQL 持久化。

当前实现基线：
- 管理员只允许使用 `WUMIN_QR_V1 / login_challenge` 扫码登录，登录态写入 HttpOnly Cookie。
- 角色访问控制：`SUPER_ADMIN / OPERATOR_ADMIN`。
- 消费 SFID 签发的 `SFID_CPMS_V1 / INSTALL` 安装码。
- CPMS 通用发行版只内置编译后的只读行政区数据，安装码决定运行实例所属市公安局。
- 安装后生成 `SFID_CPMS_V1 / ARCHIVE` 公民档案二维码；签出前必须先绑定用户投票账户。
- ARCHIVE 包含档案号、公民状态、选举资格、电子护照有效期、公民状态更新时间、CPMS 签发公钥、`geo_seal`、投票账户地址/公钥和签名，不包含 `code_id` 或使用次数。
- 档案号不暴露省、市、机构号。
- 档案号格式为 `<26位Base32>-<2位Base32校验>`，不带固定业务前缀。
- 护照号由 CPMS 后端自动生成，格式为 `<2位省代码><8位Crockford Base32主体><1位校验>`，总长 11 位；主体使用市代码派生的城市隔离编号和本地序列生成，原始市代码不明文出现。
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
| 前端 | React + TypeScript + Vite |
| 部署 | Docker / systemd |

## 2. 后端模块结构
| 模块 | 文件 | 说明 |
|------|------|------|
| main | `src/main.rs` | 入口、路由、公共工具函数、过期数据清理 |
| authz | `src/authz/mod.rs` | Cookie session 校验、角色检查 |
| login | `src/login/mod.rs` | QR-only 扫码登录、会话查询和登出 |
| initialize | `src/initialize/mod.rs` | INSTALL 初始化、ARCHIVE 签发密钥、超级管理员绑定 |
| sfid_tool_province | `src/main.rs` | 编译期直接引用 SFID 系统 `sfid/backend/sfid/province.rs` 行政区唯一源 |
| address | `src/address.rs` | 按安装码所属市重建镇/村路地址表并提供查询接口 |
| super_admin | `src/super_admin/mod.rs` | 操作员新增/删除、年度状态导出 |
| operator_admin | `src/operator_admin/mod.rs` | 档案创建/查询、软删除、ARCHIVE 更新/打印 |
| number | `src/number/mod.rs` | 档案号与护照号生成 |
| dangan | `src/dangan/mod.rs` | `geo_seal`、ARCHIVE 签名、电子护照有效期、年度状态导出、100 年硬删除 |

## 2.1 前端模块结构
| 模块 | 目录 | 说明 |
|------|------|------|
| authz | `frontend/authz/` | 登录态上下文与路由守卫 |
| initialize | `frontend/initialize/` | 安装初始化页面、API 和类型 |
| login | `frontend/login/` | QR-only 登录页面和 API |
| super_admin | `frontend/super_admin/` | 超级管理员系统设置、操作员管理、年度报告导出 |
| operator_admin | `frontend/operator_admin/` | 档案列表、创建、详情、编辑、软删除签名、档案 QR 操作 |
| address | `frontend/address/` | 镇村查询 API 和类型 |
| qr | `frontend/qr/` | WUMIN_QR_V1 解析和浏览器扫码工具 |
| common | `frontend/common/` | HTTP 封装、共享类型和通用组件 |

## 3. API 清单
系统初始化：
- `GET /api/v1/install/status`
- `POST /api/v1/install/initialize`
- `POST /api/v1/install/super-admin/bind`

认证：
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `GET /api/v1/admin/auth/me`
- `POST /api/v1/admin/auth/logout`

管理：
- `GET /POST /api/v1/admin/operators`
- `DELETE /api/v1/admin/operators/:id`
- `GET /api/v1/archives/status-export`

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
2. CPMS 离线解析 INSTALL，校验协议类型、字段格式、省市名称与 `sfid_number` 解码结果一致；CPMS 不引入外置 SFID 公钥验签流程，ARCHIVE 是否可信由 SFID 侧最终验真。
3. CPMS 根据 INSTALL 的 R5 段从内置 SFID 工具行政区数据中重建当前市镇/村路表。
4. CPMS 生成本机 `ARCHIVE` 签发密钥，公钥保存为 `cpms_pubkey`。
5. CPMS 创建档案时由 `number` 模块同步生成一对一绑定的档案号和护照号；档案号供 SFID 使用，护照号印刷在护照上。
6. 用户在 wumin 电子护照页出示投票账户地址二维码，CPMS 扫描后保存投票账户。
7. CPMS 生成携带钱包地址/公钥的 ARCHIVE 二维码。
8. SFID 扫描 ARCHIVE，解 `geo_seal`、验 CPMS 签名，并在 SFID 绑定阶段要求 wumin 钱包签名。

## 5. 数据库表
| 表 | 说明 |
|------|------|
| `system_install` | INSTALL 安装授权状态，显式保存 `sfid_number / province_code / city_code` |
| `qr_sign_keys` | ARCHIVE 签发密钥 |
| `admin_users` | 管理员；不保留停用状态字段，操作管理员删除即物理删除 |
| `sessions` | HttpOnly Cookie 对应的本机会话 |
| `login_challenges` | 扫码登录挑战 |
| `qr_login_results` | 扫码登录结果 |
| `archives` | 公民档案、护照号、投票账户与 `archive_qr_payload`；`birth_date / valid_from / valid_until` 使用 `DATE` |
| `archive_number_recycle_pool` | 满 100 年硬删除后释放的档案号和护照号对；只约束未使用号码唯一，允许多轮复用历史 |
| `archive_hard_delete_logs` | 满 100 年硬删除最小日志，不保存实名原文 |
| `cpms_status_exports` | 年度状态导出记录和已签名导出 JSON，用于重复下载同一份报告 |
| `archive_delete_challenges` | 档案软删除前的 wumin 签名挑战 |
| `address_towns` | 当前 CPMS 实例所属市的镇/街道 |
| `address_villages` | 当前 CPMS 实例所属市的村/路 |
| `sequence_counters` | 本机序列 |
| `qr_print_records` | 打印记录 |
| `audit_logs` | 操作审计日志 |

本阶段不提供旧库迁移兼容；旧库可删除后按当前基准结构初始化。
数据库层同时约束 `archives.status IN ('ACTIVE','DELETED')`、注销公民不得拥有投票资格、软删除档案必须有 `deleted_at`，防止绕过 API 写入非法状态组合。

## 6. 环境变量
| 变量 | 说明 | 默认值 |
|------|------|--------|
| `CPMS_DATABASE_URL` | PostgreSQL 连接串 | `postgres://cpms:cpms@127.0.0.1:5433/cpms_dev` |
| `CPMS_BIND` | 监听地址 | `0.0.0.0:8080` |
| `CPMS_KEY_ENCRYPT_SECRET` | 本机密钥加密主密钥，32 字节 hex；缺失时拒绝初始化或读取已加密密钥 | 无 |

## 7. 验证
- `cargo fmt && cargo check && cargo test`
- `cd frontend && npm run build`

## 8. 错误码

CPMS 后端错误响应包含数字 `code` 和稳定业务 `error_code`。HTTP `401` 只表示当前
管理员登录态无效；challenge 过期返回 `410`，签名验签失败返回 `422`，权限不足返回
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
| 创建/删除操作员 | 可以 | 不可以 |

后端档案业务接口统一使用 `authz::require_archive_admin`，允许超级管理员和操作员管理员。
系统设置、操作员管理继续使用 `SUPER_ADMIN` 专属权限。超级管理员不能被删除；操作管理员删除
为物理删除，并同步清理本机会话，只保留审计快照。

公民姓名字段统一为 `last_name / first_name`，前端、后端和数据库不再使用 `full_name`
作为新接口字段。公民信息列表搜索使用 `q` 参数，同时匹配 `last_name`、`first_name`、
`last_name || first_name` 与完整档案号 `archive_no`。
列表展示完整档案号和由出生日期计算的年龄，省份不在列表中重复展示；省市在详情页展示。
详情页在出生日期/年龄下方展示护照号和电子护照有效期。创建档案当天未满 16 周岁时有效期为
5 年，已满 16 周岁时有效期为 10 年；生日当天视为已满对应周岁。

## 9.1 档案软删除

公民档案删除不是物理删除。详情页点击“删除”后，CPMS 后端创建 `WUMIN_QR_V1 / sign_request`
删除签名请求，当前登录管理员必须使用 **wumin** 扫码签名。前端扫描 `sign_response` 后提交
`delete/complete`。删除签名二维码中的 `body.address / body.pubkey` 锁定当前登录 CPMS 管理员，
其中 `body.pubkey` 和 payload 内的 `admin_pubkey` 必须统一为 `0x` + 64 位小写 hex；CPMS 管理员表
内部可保存裸 hex，但进入 wumin 二维码前必须规范化，否则冷钱包会拒绝解析。

删除 payload 固定为:

```text
CPMS_ARCHIVE_DELETE_V1|challenge_id|archive_id|archive_no|0x_admin_pubkey|expires_at
```

前端删除弹窗采用与登录页一致的“双栏扫码”布局：左侧展示删除签名请求二维码，右侧扫描 wumin
返回的删除签名回执。后端校验:

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
ARCHIVE 二维码，不再使用“生成档案码”作为按钮文案。“打印”会记录打印审计并调用浏览器打印，
打印媒体只输出“公民档案详情”卡片，不打印侧栏、顶部栏和删除/编辑/返回列表/
更换/更新/下载/打印等操作按钮。详情页电子护照有效期按两行展示，第一行 `有效期：起始日期`，
第二行 `-截止日期`，第二行的 `-` 与第一行的 `：` 对齐。投票账户地址独占整行，按单行显示。

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
