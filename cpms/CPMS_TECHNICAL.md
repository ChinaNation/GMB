# CPMS Technical Notes

## 0. 系统概述
CPMS（Citizen Passport Management System）是市公安局使用的公民档案管理系统，后端使用 Rust/Axum，前端使用 React/Vite，数据使用 PostgreSQL 持久化。

当前实现基线：
- 管理员只允许使用 `CITIZEN_QR_V1 / login_challenge` 扫码登录，登录态写入 HttpOnly Cookie；管理员 15 分钟无活动过期，操作员 30 分钟无活动过期。
- 角色访问控制：`admins / operators`。
- 消费 SFID 签发的 `SFID_CPMS_V1 / INSTALL` 安装码。
- CPMS 通用发行版只内置编译后的只读行政区数据，安装码决定运行实例所属市公安局。
- 行政区快照来自开发库 `sfid/backend/china/china.sqlite`；CPMS 必须离线运行，不主动联网拉取，发布包随附本地只读 `china.sqlite`。
- 安装后生成 `SFID_CPMS_V1 / ARCHIVE` 公民档案二维码；签出前必须先绑定用户投票账户，并满足档案码完整性门槛。
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
| 部署 | systemd 主机安装包，后端直接托管前端静态文件 |

## 2. 后端模块结构
| 模块 | 文件 | 说明 |
|------|------|------|
| main | `main.rs` | 入口、AppState 装配、路由挂载、安全响应头、过期数据清理 |
| authz | `authz/mod.rs` | Cookie session 校验、角色检查 |
| login | `login/mod.rs` | QR-only 扫码登录、会话查询和登出 |
| initialize | `initialize/mod.rs` | INSTALL 初始化、ARCHIVE 签发密钥、管理员绑定 |
| common | `common/`（与前端 `common/` 对齐的共享层） | 横切低层工具 + 跨模块共享响应/DTO/helper |
| common::response | `common/response.rs` | `ApiResponse`/`ApiError`/`ok`/`err`/错误码映射（≈ 前端 http.ts） |
| common::types | `common/types.rs` | 跨模块共享 DTO `AdminUser`/`Archive`（≈ 前端 types.ts） |
| common::admin | `common/admin.rs` | 管理员记录查询 helper |
| common::audit | `common/audit.rs` | 审计日志写入 helper |
| common::encoding | `common/encoding.rs` | hex/base64 字节解码 |
| common::rate_limit | `common/rate_limit.rs` | 登录、初始化、删除签名和资料上传的本机内存限流 |
| common::ss58 | `common/ss58.rs` | SS58 ↔ hex 公钥编解码（prefix=2027） |
| address | `address/mod.rs` | 按安装码所属市重建镇/地址段表并提供查询接口（CPMS 自有地址业务） |
| address::china | `address/china.rs` | address 的源适配子模块：运行时用 rusqlite 只读开发库派生的 `china.sqlite` 行政区快照（安装包随附只读拷贝，路径走 `CPMS_CHINA_DB`），按安装码所属市窄查询镇和地址段 |
| admins | `admins/mod.rs` | 管理员新增、姓名编辑、删除、年度状态导出 |
| number | `number/mod.rs` | 档案号与护照号生成 |
| dangan | `dangan/` | 档案创建/查询、游标分页、软删除、ARCHIVE 更新/打印、`geo_seal`、电子护照有效期、公民资料库、档案操作记录、年度状态导出、100 年硬删除 |

## 2.1 前端模块结构
| 模块 | 目录 | 说明 |
|------|------|------|
| authz | `frontend/authz/` | 登录态上下文与路由守卫 |
| initialize | `frontend/initialize/` | 安装初始化页面、API 和类型 |
| login | `frontend/login/` | QR-only 登录页面和 API |
| admins | `frontend/admins/` | 管理员系统设置、管理员管理、年度报告导出 |
| dangan | `frontend/dangan/` | 档案列表、创建、详情左右导航、编辑、资料库、操作记录、软删除签名、档案 QR 操作 |
| address | `frontend/address/` | 镇和地址段查询 API 和类型 |
| qr | `frontend/qr/` | CITIZEN_QR_V1 解析和浏览器扫码工具 |
| components | `frontend/components/` | 通用展示与输入组件，日期输入统一使用 `DateInput` |
| common | `frontend/common/` | HTTP 封装、共享类型和通用组件 |

## 3. API 清单
系统初始化：
- `GET /api/v1/install/status`
- `POST /api/v1/install/initialize`
- `POST /api/v1/install/admins/bind`

认证：
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `GET /api/v1/admin/auth/me`
- `POST /api/v1/admin/auth/logout`

管理：
- `GET /POST /api/v1/admin/admins`
- `PUT /DELETE /api/v1/admin/admins/:id`
- `GET /api/v1/archives/status-export`
- `GET /api/v1/archives/status-export/state`

档案：
- `POST /GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`
- `POST /api/v1/archives/:archive_id/wallet`
- `GET /POST /api/v1/archives/:archive_id/materials`
- `GET /api/v1/archives/:archive_id/materials/:material_id/download`
- `DELETE /api/v1/archives/:archive_id/materials/:material_id`
- `GET /api/v1/archives/:archive_id/audit-logs`
- `POST /api/v1/archives/:archive_id/qr/generate`
- `POST /api/v1/archives/:archive_id/qr/print`
- `POST /api/v1/archives/:archive_id/delete/challenge`
- `POST /api/v1/archives/:archive_id/delete/complete`

## 4. 两码流程
1. SFID 生成 `SFID_CPMS_V1 / INSTALL`。
2. CPMS 离线解析 INSTALL，校验协议类型、字段格式、省市名称与 `sfid_number` 解码结果一致；CPMS 不引入外置 SFID 公钥验签流程，ARCHIVE 是否可信由 SFID 侧最终验真。
3. CPMS 根据 INSTALL 的 R5 段从内置 SFID 行政区快照中重建当前市镇和地址段表。
4. CPMS 生成本机 `ARCHIVE` 签发密钥，公钥保存为 `cpms_pubkey`。
5. CPMS 创建档案时由 `number` 模块同步生成一对一绑定的档案号和护照号；档案号供 SFID 使用，护照号印刷在护照上。
6. 用户在 citizenwallet 电子护照页出示投票账户地址二维码，CPMS 扫描后保存投票账户。
7. CPMS 生成携带钱包地址/公钥的 ARCHIVE 二维码。
8. SFID 扫描 ARCHIVE，解 `geo_seal`、验 CPMS 签名，并在 SFID 绑定阶段要求 citizenwallet 钱包签名。

## 5. 数据库表
| 表 | 说明 |
|------|------|
| `system_install` | INSTALL 安装授权状态，显式保存 `sfid_number / province_code / city_code` |
| `qr_sign_keys` | ARCHIVE 签发密钥 |
| `admin_users` | 管理员；不保留停用状态字段，初始管理员不可删除，其他管理员删除即物理删除 |
| `sessions` | HttpOnly Cookie 对应的本机会话 |
| `login_challenges` | 扫码登录挑战 |
| `qr_login_results` | 扫码登录结果 |
| `archives` | 公民档案、护照号、投票账户与 `archive_qr_payload`；`birth_date / valid_from / valid_until` 使用 `DATE` |
| `archive_stats` | 档案列表总量统计；创建和注销软删除同事务维护，列表页不得实时 `COUNT(*)` |
| `archive_materials` | 公民资料库元数据；文件正文保存在本机 `CPMS_MATERIALS_DIR` 或默认资料目录 |
| `archive_number_recycle_pool` | 满 100 年硬删除后释放的档案号和护照号对；只约束未使用号码唯一，允许多轮复用历史 |
| `archive_hard_delete_logs` | 满 100 年硬删除最小日志，不保存实名原文 |
| `cpms_status_exports` | 年度状态导出记录和已签名导出 JSON，用于重复下载同一份报告 |
| `archive_delete_challenges` | 档案软删除前的 citizenwallet 签名挑战 |
| `address_towns` | 当前 CPMS 实例所属市的镇 |
| `address_units` | 当前 CPMS 实例所属市的镇下地址段 |
| `sequence_counters` | 本机序列 |
| `qr_print_records` | 打印记录 |
| `audit_logs` | 操作审计日志 |

本阶段不提供旧库迁移兼容；旧库可删除后按当前基准结构初始化。
数据库层同时约束 `archives.status IN ('ACTIVE','DELETED')`、注销公民不得拥有投票资格、软删除档案必须有 `deleted_at`，防止绕过 API 写入非法状态组合。16 周岁年龄线由后端 API 根据出生日期统一判断；未满 16 周岁不得保存为有选举资格。

## 5.1 数据库 migration 规则

- CPMS 尚未正式发行前，开发期 migration 只保留当前完整基线 `0001_init_cpms_pg.sql`；修改基线后必须执行 `./cpms.sh --reset` 重建开发库。
- `MIGRATOR.run()` 必须保留，用于保护数据库结构与当前程序携带的 migration 校验一致。
- 正式版发布时冻结当时全部 migration；从第一个正式版本开始，任何已经发布的 migration 文件都不得再修改。
- 正式版后续升级只允许新增 migration，例如 `0002_xxx.sql`、`0003_xxx.sql`，由程序启动时自动执行，用户数据库不得清空。
- `schema.sql` 只表示当前完整结构，供开发、测试和人工审查使用；正式安装包不携带、不执行
  `schema.sql / seed.sql`，全新正式库也统一由后端 `MIGRATOR.run()` 创建。

## 5.2 年度状态导出

- 年度报告类型固定为 `SFID_CPMS_V1 / CPMS_STATUS_EXPORT`，只导出给 SFID 手工导入的绑定状态 JSON。
- 管理员从每年 UTC 1 月 1 日起可以导出上一年度数据；如果存在多年未导出，CPMS 按最早未导出年度依次补导。
- 首个需要导出的年度从 `system_install.initialized_at` 所在年份开始，新装 CPMS 不补导安装前历史年度。
- 年度报告导出按钮始终从当前最新档案数据重新生成报告；同一年度重复导出会覆盖 `cpms_status_exports` 中该年度记录，不返回旧 JSON。
- UTC 每年 1 月 11 日起，如果存在已超过 1 月 10 日仍未导出的年度报告，操作员登录和已有会话会被锁定，直到管理员完成补导。
- CPMS 不判断 SFID 是否收到文件，也不禁用安装码；SFID 逾期禁用 CPMS 授权由 SFID 系统单独实现。
- `GET /api/v1/archives/status-export/state` 返回系统设置页按钮状态、角标状态和操作员锁定状态。
- `GET /api/v1/archives/status-export` 生成或返回最早未导出年度的已签名报告。
- `citizen_binding_records` 导出当前仍有钱包绑定的档案快照：`archive_no / wallet_address / wallet_pubkey / wallet_sig_alg / wallet_bound_at / citizen_status / voting_eligible / status_updated_at`；导出时再次按公民状态和 16 周岁年龄线计算有效选举资格，不把未成年档案导出为有选举资格。
- `binding_release_records` 只导出当年度满 100 年硬删除后需要 SFID 释放三者绑定关系的 `archive_no / released_at / release_reason`。
- 年度报告不得导出姓名、出生日期、地址、护照号；护照号是 CPMS 内部号码，与 SFID 导入无关。

## 5.3 档案列表分页与检索

- 档案列表归属 `dangan` 模块，不归属操作员角色模块；`admins / operators` 只是访问权限。
- `GET /api/v1/archives` 使用游标分页，不接受 `page / page_size / q` 小表分页参数，也不接受 `archive_no / passport_no / name` 选择器式查询参数。
- 默认每页 `50` 条，前端可选 `20 / 50 / 100`，后端最大限制 `100`。
- 默认排序固定为 `created_at DESC, archive_id DESC`，cursor 由后端编码 `created_at / archive_id`，前端只透传。
- 响应返回 `items / limit / next_cursor / has_next / total_active`，不返回总页数。
- 前端列表在“年龄”和“公民状态”之间显示“市镇”列；内容只显示当前档案 `town_code` 对应的镇名称，不显示市名、地址段或详细地址。
- 前端列表第一列为当前页序号，第二列为档案号；整行点击进入公民档案详情，不设置单独“操作/详情”列。
- `total_active` 来自 `archive_stats.active_count`，不得在列表请求中实时执行 `COUNT(*) FROM archives`。
- 前端只提供统一精确检索输入框，参数为 `search`；后端用 `archive_no = search OR passport_no = search OR (last_name || first_name) = search` 精确匹配，不做字段选择器。
- 检索只允许索引化精确检索：`search / birth_date / town_code / address_unit_id / citizen_status`；不得恢复 `%keyword% LIKE` 全表模糊搜索。

## 5.4 公民资料库

- 公民资料库后端主体在 `dangan/materials.rs`，档案详情页入口在 `frontend/dangan/ArchiveDetail.tsx` 的“资料库”左侧导航 tab。
- 支持资料类型：照片、出生纸、复印件、视频和其他资料；后端按类型校验 MIME，单文件上限 100 MB。
- 数据库 `archive_materials` 只保存元数据、哈希和本机存储文件名，不保存文件正文。
- 开发默认文件正文保存在 `data/archive-materials/<archive_id>/`；正式离线安装包固定通过
  `CPMS_MATERIALS_DIR=/var/lib/cpms/materials` 写入本机资料目录。
- 软删除档案仍可查看和下载已有资料，但不能新增或删除资料。
- 上传、下载、删除资料写入审计；上传或删除资料会清空旧档案码，100 年硬删除档案时同步清理该档案资料文件目录。
- 前端资料上传入口只显示为“上传”按钮，点击后弹窗录入资料类型、文件和备注；资料库标题区不重复显示数量，数量只保留在左侧“资料库”tab。
- 资料上传入口有本机 IP 级限流；单文件仍由 100 MB 请求体上限兜底。

## 5.5 档案详情与操作记录

- 公民档案详情页固定为左侧导航和右侧内容区：左侧依次为“返回列表、档案详情、资料库、操作记录”，右侧只展示当前选中 tab 的内容。
- 左侧导航保留 CPMS 当前“返回列表”图标；“档案详情、资料库、操作记录”图标语义对齐 SFID 机构详情共享导航的房子、文件夹和历史记录图标。
- 档案详情字段使用两列网格对齐；有效期固定显示在护照号下一行，公民状态和选举资格固定纳入同一网格行，不得恢复为独立外层表单行。
- “档案详情”和“资料库”沿用原 CPMS 业务内容；旧的上下堆叠详情页结构不得恢复。
- “操作记录”读取 `GET /api/v1/archives/:archive_id/audit-logs`，后端从 `audit_logs` 中按档案 ID、档案号和审计 detail 中的档案事实聚合最近 100 条。
- 操作记录表格列固定为“操作、操作者账户、详情、时间”；操作者账户由后端从管理员公钥转换为可读账户地址，结果状态并入详情展示。
- 操作记录只展示 CPMS 本机审计事实，不创建新业务流程，不写入额外审计事件。

## 5.6 安全运行约束

- 后端统一设置 `Content-Security-Policy`、`X-Frame-Options`、`X-Content-Type-Options`、`Referrer-Policy` 和 `Permissions-Policy`。
- `/api/` 路径不得落到前端 `index.html` 兜底；API 未命中统一返回 JSON 错误，前端 HTTP 封装也会拒绝非 JSON 响应并显示具体请求路径。
- 登录 QR、安装初始化、管理员初始化绑定、删除签名完成和资料上传入口使用本机内存限流；触发后返回 `429 / CPMS_RATE_LIMITED`。
- `CPMS_KEY_ENCRYPT_SECRET` 在已初始化实例启动时必须能解密 `system_install.install_secret` 和 `qr_sign_keys` 中的 ARCHIVE 私钥，否则拒绝启动。
- 正式安装包按 CPU 架构分为 `cpms-ubuntu24-amd64.run` 和 `cpms-ubuntu24-arm64.run`，均包含后端、
  `frontend/dist`、PostgreSQL/nginx/openssl 等 Ubuntu 24.04 离线 deb 依赖、systemd、nginx
  配置、证书生成脚本和 `CPMS安装配置手册.pdf`；安装过程不得联网。
- 后端正式部署只监听 `127.0.0.1:8080`，局域网入口统一由 nginx 提供
  `https://www.cpms.com/`。客户端 DNS 由公安局内网自行配置到 CPMS 主机地址。
- 安装时生成 `/etc/cpms/certs/cpms-root-ca.crt` 和 `www.cpms.com` 服务端证书；客户端需要信任该
  本机私有 CA 后访问 HTTPS。
- 安装手册安装到 `/opt/cpms/docs/CPMS安装配置手册.pdf`，手动 CI artifact 也单独包含同一份 PDF。
- 前端所有二维码读取入口统一走 `frontend/qr/CameraQrScanner.tsx`：支持摄像头扫码与上传图片本地解码两种模式，二者共用浏览器原生 `BarcodeDetector` 与同一 `onDetected` 回调，不在页面内另起第二套扫码逻辑。上传二维码只在前端本地解析，不把图片文件传后端。初始化「扫描安装码」步骤开启上传入口（`allowUpload`）；绑定管理员的公民钱包码步骤只用摄像头。

## 6. 环境变量
| 变量 | 说明 | 默认值 |
|------|------|--------|
| `CPMS_DATABASE_URL` | PostgreSQL 连接串 | `postgres://cpms:cpms@127.0.0.1:5433/cpms_dev` |
| `CPMS_MATERIALS_DIR` | 公民资料库文件正文保存目录；正式安装为 `/var/lib/cpms/materials` | `data/archive-materials` |
| `CPMS_BIND` | 监听地址；正式安装由 nginx 反代到本机回环地址 | `127.0.0.1:8080` |
| `CPMS_KEY_ENCRYPT_SECRET` | 本机密钥加密主密钥，32 字节 hex；缺失时拒绝初始化或读取已加密密钥 | 无 |
| `CPMS_FRONTEND_DIR` | 正式部署前端静态文件目录；设置后必须存在 `index.html` | 未设置时使用 `./frontend` |
| `CPMS_COOKIE_SECURE` | 设置为 `true/1/yes` 时给 session Cookie 增加 `Secure`，用于 HTTPS 部署 | 未启用 |
| `CPMS_CHINA_DB` | 开发库派生的离线行政区 SQLite 快照路径；安装包默认随附 | `/opt/cpms/data/china.sqlite` |

## 7. 验证
- `cargo fmt && cargo check && cargo test`
- `cd frontend && npm run build`

## 8. 错误码

CPMS 后端错误响应包含数字 `code` 和稳定业务 `error_code`。HTTP `401` 只表示当前
管理员登录态无效；challenge 过期返回 `410`，签名验签失败返回 `422`，权限不足返回
`403`。完整规则见 `memory/05-modules/cpms/ERROR_CODES.md`。

前端收到 `401` 时只清理本地用户镜像并通知认证上下文，不在 HTTP 封装中强制跳转；
根路由先检查安装状态，未初始化进入 `/install`，受保护页面未登录才进入 `/login`。

## 9. 管理员权限

CPMS 只有两种管理员:

| 功能 | `admins` | `operators` |
|------|------|------|
| 查看公民列表 | 可以 | 可以 |
| 创建/编辑公民档案 | 可以 | 可以 |
| 绑定/更换投票账户 | 可以 | 可以 |
| 生成/打印档案码 | 可以 | 可以 |
| 修改公民状态 | 可以 | 可以 |
| 删除公民档案 | 可以 | 可以 |
| 查看系统设置 | 可以 | 不可以 |
| 创建管理员 | 可以 | 不可以 |
| 编辑管理员姓名 | 可以 | 不可以 |
| 删除非初始管理员 | 可以 | 不可以 |

后端档案业务接口统一使用 `authz::require_archive_admin`，允许管理员和操作员。
系统设置、管理员管理继续使用 `admins` 专属权限。初始化时绑定的管理员固定为
不可删除的初始管理员，并固定显示在管理员列表第一行；后续新增的管理员和操作员都
只能编辑姓名，且都可以被删除。管理员总数最多 5 个，包括初始化时的 1 个和后续新增的
最多 4 个。管理员删除为物理删除，并同步清理本机会话，只保留审计快照。

公民姓名字段统一为 `last_name / first_name`，前端、后端和数据库不再使用 `full_name`
作为新接口字段。公民档案列表使用游标分页和索引化精确检索，不再使用 `q` 参数或总页数分页。
列表展示当前 CPMS 安装省市、完整档案号和由出生日期计算的年龄；省市只来自 `system_install`。
详情页在出生日期/年龄下方展示护照号和电子护照有效期。创建档案当天未满 16 周岁时有效期为
5 年，已满 16 周岁时有效期为 10 年；生日当天视为已满对应周岁。
公民档案创建/编辑时，详细地址、公民状态和选举资格均为必填；出生日期必须早于当前 UTC 日期，
不得选择当天或未来日期。选举资格必须同时满足公民状态 `NORMAL` 和已满 16 周岁；公民状态为
`REVOKED` 或出生日期未满 16 周岁时，选举资格固定为 `false`，前端不可选择“有选举资格”，后端也拒绝保存。

前端所有日期输入必须使用 `frontend/components/DateInput.tsx`，不得在页面中散落原生日期
输入标签。出生日期和出生日期搜索默认使用该组件的昨日 `max` 约束，保证档案
列表搜索、创建档案和编辑档案的年份、月份、日期输入行为一致。

## 9.1 档案软删除

公民档案删除不是物理删除。详情页点击“删除”后，CPMS 后端创建 `CITIZEN_QR_V1 / sign_request`
删除签名请求，当前登录管理员必须使用 **citizenwallet** 扫码签名。前端扫描 `sign_response` 后提交
`delete/complete`。删除签名二维码中的 `body.address / body.pubkey` 锁定当前登录 CPMS 管理员，
其中 `body.pubkey` 和 payload 内的 `admin_account` 必须统一为 `0x` + 64 位小写 hex；CPMS 管理员表
内部可保存裸 hex，但进入 citizenwallet 二维码前必须规范化，否则冷钱包会拒绝解析。
冷钱包确认页的人机展示只显示档案号、管理员 SS58 地址和过期时间，`archive_id` 与原始
`admin_account` 只参与 payload 验真，不作为普通确认字段展示。

删除 payload 固定为:

```text
CPMS_ARCHIVE_DELETE_V1|challenge_id|archive_id|archive_no|0x_admin_account|expires_at
```

前端删除弹窗采用与登录页一致的“双栏扫码”布局：左侧展示删除签名请求二维码，右侧扫描 citizenwallet
返回的删除签名回执。后端校验:

- challenge 未过期、未消费，且绑定当前档案和当前登录管理员。
- `sign_response.pubkey` 等于当前登录管理员 `admin_account`。
- `payload_hash` 等于删除 payload 的 SHA-256。
- sr25519 签名验证通过。

通过后 `archives.status` 更新为 `DELETED`，并记录 `deleted_at / deleted_by / delete_reason`。
任何删除 challenge、签名人、payload hash、过期时间或验签失败都会写入 `DELETE_ARCHIVE / FAILED`
审计，不消费 challenge，也不修改档案。
列表默认隐藏已删除档案；已删除档案不能继续编辑、绑定投票账户、更新档案码、下载或打印。

## 10. 投票账户

ARCHIVE 投票账户字段、签名原文和签出流程见
`memory/05-modules/cpms/ARCHIVE_WALLET_PROOF.md`。无钱包地址的档案不得签出
ARCHIVE；姓氏、名字、性别、身高、出生日期、护照号、有效期、省份、城市、公民状态、选举资格、
投票账户、照片和出生纸未齐全时也不得签出 ARCHIVE。公民状态必须为 `NORMAL`，选举资格必须为
`true`，照片和出生纸各至少 1 张。同一个钱包账户在档案生命周期内只能绑定一个公民档案；软删除期间仍占用钱包账户，
只有满 100 年硬删除并物理删除档案后，钱包账户、档案号和护照号才自然释放。档案详情页档案码操作统一显示为“更新 / 下载 / 打印”；“更新”表示刷新当前
ARCHIVE 二维码，不再使用“生成档案码”作为按钮文案。“打印”会记录打印审计并调用浏览器打印，
打印媒体只输出“公民档案详情”卡片，不打印侧栏、顶部栏和删除/编辑/返回列表/
更换/更新/下载/打印等操作按钮。详情页电子护照有效期按两行展示，第一行 `有效期：起始日期`，
第二行 `-截止日期`，第二行的 `-` 与第一行的 `：` 对齐。投票账户地址独占整行，按单行显示。

## 11. 行政区数据

- 开发库 `sfid/backend/china/china.sqlite` 是行政区数据唯一源头。
- CPMS 后端源码目录不保存行政区第二份文件，也不维护 `province.rs` 或 `city_codes/*.rs`
  的第二份源码。
- CPMS 后端 `china` 模块运行时用 rusqlite 只读 `china.sqlite`，路径三层兜底：
  ① 环境变量 `CPMS_CHINA_DB`（生产由 install_host 写入 `/opt/cpms/data/china.sqlite`）；
  ② 二进制旁 `<exe 目录>/../data/china.sqlite`（部署自定位，env 丢失也能找到）；
  ③ 编译期 `CARGO_MANIFEST_DIR` 相对的 SFID 唯一源（本地 `cargo run` 零配置即通）。
  发行包随附该 SQLite 的只读拷贝（安装到 `/opt/cpms/data/china.sqlite`）。
- `cpms/scripts/build_linux_host_installer.sh` 把 SFID 唯一源 `china.sqlite` 拷入安装包 payload；
  任何在 CPMS 源码树恢复 `province.rs`/`city_codes` 第二份行政区源码的改动都属于残留回退。
- 一个 CPMS 通用发行包可以安装到任意市公安局；运行时由 SFID 签发的 INSTALL 安装码锁定
  唯一市公安局。
- CPMS 初始化和已初始化实例启动时会按安装码 R5 段重建 `address_towns/address_units`，
  地址接口只返回当前市数据。
- 公民档案的出生日期、性别、身高、地址段和详细地址输入段均为必填；出生日期固定 `YYYY-MM-DD` 且必须
  早于当前 UTC 日期，身高范围为 `30-260 cm`；未满 16 周岁的公民不得设置为有选举资格。
- 前端日期控件统一走 `frontend/components/DateInput.tsx`，出生日期类输入默认不允许选择
  当天或未来日期。

## 12. Ubuntu 24.04 离线主机安装

- 发行产物名称固定为 `cpms-ubuntu24-amd64.run` 和 `cpms-ubuntu24-arm64.run`，分别由 GitHub
  Actions 的 `ubuntu-24.04` 与 `ubuntu-24.04-arm` runner 构建；正式交付只使用这两份离线自解压安装包。
- push / pull_request 只执行编译与测试 CI，不上传正式安装包；只有手动 `workflow_dispatch`
  运行成功后才上传正式版 artifact。
- `cpms/scripts/build_linux_host_installer.sh` 构建自解压 `.run`：payload 包含 `cpms-backend`、
  前端静态文件、安装配置 PDF 手册、systemd 文件、nginx 配置、证书脚本、备份脚本和 Ubuntu
  24.04 当前架构运行依赖 deb 闭包；正式 payload 不包含 `schema.sql / seed.sql`。
- 每个安装包内置 `payload/manifest.env`，声明 `CPMS_PACKAGE_ARCH=amd64|arm64`；安装脚本必须用
  `dpkg --print-architecture` 校验目标机架构，架构不一致时拒绝安装。
- 运行依赖 deb 闭包必须在官方 `ubuntu:24.04` Docker 容器内解析和下载，禁止使用 GitHub
  runner 主机 apt 环境，避免第三方源、预装软件或虚拟包污染依赖版本。
- `cpms/deploy/linux/install_host.sh` 只从 payload 的 `debs/` 安装依赖，禁止 `apt-get update`
  或访问外部 apt 源；目标机无需联网。
- 正式安装脚本只创建 PostgreSQL 角色、数据库、schema 权限和服务环境文件，不导入数据库
  SQL；数据库表结构唯一由后端服务启动时的 `MIGRATOR.run()` 创建。脚本会修正旧错误安装残留
  对象的 owner 和权限，避免 `permission denied for table system_install` 导致后端启动失败。
- 安装后后端服务为 `cpms-backend.service`，工作目录 `/var/lib/cpms`，环境文件
  `/etc/cpms/cpms-backend.env`，资料目录 `/var/lib/cpms/materials`。
- nginx 站点文件为 `/etc/nginx/sites-available/cpms.conf`，启用后监听 80/443，80 自动跳转
  HTTPS，443 反代到 `127.0.0.1:8080`。
- 证书由 `/opt/cpms/bin/generate_cpms_certs.sh` 安装时生成；Root CA 路径为
  `/etc/cpms/certs/cpms-root-ca.crt`，服务端证书只绑定 `DNS:www.cpms.com`。
- 备份脚本 `/opt/cpms/bin/backup_to_storage.sh` 同时备份 PostgreSQL dump、`/var/lib/cpms/runtime`
  和 `/var/lib/cpms/materials`，并生成同批次 sha256 校验文件。
- 卸载脚本只移除 CPMS 服务、nginx 站点和 `/opt/cpms/bin` 内程序；PostgreSQL、数据库、
  `/etc/cpms`、`/var/lib/cpms` 和 `/var/backups/cpms` 默认保留，避免误删实名档案资料。
