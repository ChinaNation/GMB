# CPMS 技术开发文档（统一版）

## 1. 文档目的
- 固化 CPMS 最终架构：完全离线、桌面壳访问 Web、单一权威数据库。
- 明确管理员公钥登录、权限模型、公民资料录入与二维码打印流程。
- 作为开发、测试、验收与运维的唯一基准文档。

## 2. 系统定位与边界
### 2.1 系统定位
- `CPMS`：完全离线公民护照管理系统。
- 负责公民资料录入、存储、查询、二维码签发与打印。
- 前端形态为 Web 页面，通过桌面壳启动，UI 采用 Tabler 视觉体系。
- 每个市级机构安装实例使用独立 `site_sfid` 与独立签名密钥集合。

### 2.2 系统关系
- `SFID`：在线系统，读取用户携带的 CPMS 二维码并执行绑定流程。
- CPMS 与 SFID 无在线直连，仅通过二维码进行线下数据交付。

### 2.3 范围外
- 区块链交互实现。
- 互联网访问能力。
- SFID 内部业务实现。

## 3. 总体架构
### 3.1 架构原则
- 完全离线内网部署。
- 多终端共享同一份中心数据。
- 认证与授权以后端为准。

### 3.2 组件
- `frontend`：统一前端目录（Web 管理页面 + 桌面壳启动器）。
- `cpms-backend`：后端服务（内网）。
- `postgresql`：中心数据库（唯一权威库）。
- `file-storage`：照片、指纹、材料文件存储（加密）。

### 3.3 部署拓扑
- 一台内网主机：部署 `cpms-backend + postgresql + file-storage`。
- 多台作业电脑：安装 `frontend/desktop-shell`，连接同一个内网 API 地址。
- 禁止每台电脑独立数据库写入生产数据。

### 3.4 一体化安装要求（主机侧）
- 安装包必须包含数据库运行能力，不要求用户单独安装 PostgreSQL。
- 生产落地方式：Linux 主机安装包内置安装脚本，自动安装 PostgreSQL 与后端服务。
- 安装入口：`deploy/linux/install_host.sh`（由安装包调用）。
- 对外访问地址由主机内网 IP + `CPMS_HTTP_PORT` 组成，默认 `8080`。
- 后端监听默认 `CPMS_BIND=0.0.0.0:8080`，确保局域网终端可访问。
- Windows 终端不安装 CPMS，不安装数据库，只使用浏览器访问主机地址。

### 3.5 备份与容灾（主机 -> 储存电脑）
- 主机需具备定时备份能力，按日将数据库备份传输到独立储存电脑。
- 备份实现：
  - 脚本：`deploy/linux/backup_to_storage.sh`
  - systemd：`cpms-backup.service + cpms-backup.timer`
  - 配置：`/etc/cpms/backup.env`
- 传输协议：SSH + rsync（局域网内）。
- 备份内容：
  - PostgreSQL 全库 `pg_dump --format=custom`
  - `/var/lib/cpms/runtime` 目录打包
- 默认计划：每日 `02:15` 执行，`Persistent=true`（主机离线后会补跑）。
- 默认保留策略：
  - 远端与主机本地默认永久保留（`RETENTION_DAYS=0`、`LOCAL_RETENTION_DAYS=0`）
  - 若需清理，可将上述值改为正整数天数

## 4. 管理员模型与权限
### 4.1 管理员类型
- 超级管理员（`SUPER_ADMIN`）：
  - 不在安装时自动生成，来源于 `wuminapp` 扫码绑定。
  - 固定上限 `3` 个，与 `K1/K2/K3` 一一对应管理。
  - 可增删改查操作管理员。
  - 可查看全量审计与系统配置。
- 操作管理员（`OPERATOR_ADMIN`）：
  - 录入、修改、查询公民资料。
  - 生成并打印二维码。
  - 不可管理管理员账号。

### 4.2 权限边界
- 管理操作管理员接口仅允许 `SUPER_ADMIN`。
- 机构公钥登记二维码接口仅允许 `SUPER_ADMIN`。
- 档案录入、资料上传、二维码打印接口允许 `SUPER_ADMIN | OPERATOR_ADMIN`。
- 前端按钮隐藏仅用于体验控制，后端强制 RBAC 校验。

### 4.3 初始化要求（超级管理员）
1. 首次安装必须扫码 SFID 下发的机构初始化二维码（`SFID_CPMS_INSTALL`）。
2. CPMS 使用内置 `SFID_ROOT_PUBKEY` 验签成功后，才生成本机构 `3` 把二维码签名密钥（`K1/K2/K3`）。
3. 初始化时不生成超级管理员私钥；每把签名密钥对应一个“超级管理员绑定二维码”供 `wuminapp` 扫码。
4. `wuminapp` 回传绑定签名后，CPMS 写入 `admin_users`（角色固定 `SUPER_ADMIN`，最多 3 个）。
5. 初始化结果写入 `CPMS_INSTALL_FILE`（默认 `runtime/cpms_install_init.json`），重启后复用。

## 5. 登录机制（对齐 SFID）
### 5.1 账户标识
- 管理员账户标识为公钥（`admin_pubkey`）。
- 不使用用户名密码作为主登录方式。

### 5.2 扫码签名登录流程
1. 登录页点击“生成登录二维码”，系统下发一次性 challenge 二维码。
2. 管理员手机扫码后，手机端回传 `admin_pubkey + signature` 到 CPMS。
3. 后端判定扫码公钥是否为 CPMS 管理员；是管理员继续，不是管理员直接拒绝登录。
4. 后端验证签名与 challenge 一致，失败直接拒绝登录。
5. 登录页轮询 challenge 结果，成功后自动建立会话并进入系统。
6. challenge 固定有效期 `90` 秒，且 `request_id` 一次性消费，防重放。

### 5.3 安全要求
- 私钥不上传、不落库。
- 登录 challenge 绑定 `nonce + expire_at + session`。
- 认证失败与越权请求必须记录审计日志。

### 5.4 密钥边界
- `CPMS_ADMIN_KEYS`：用于 CPMS 管理员扫码签名登录（超级管理员与操作管理员账户体系，私钥在 `wuminapp`）。
- `CPMS_QR_SIGN_KEYS`：用于 CPMS 生成给 SFID 的二维码签名，每个机构安装时生成 3 把私钥。
- `CPMS_QR_SIGN_KEYS` 不参与超级管理员安装初始化，不由超级管理员登录密钥派生。
- 两套密钥用途必须隔离，不允许复用同一密钥对。

## 6. 使用场景
- 一栋楼内多台电脑同时录入公民资料。
- 每台电脑同一时刻登录一个管理员账号。
- 各电脑数据实时写入同一中心数据库。
- 录入完成后现场打印二维码交给用户。

## 7. 核心业务流程
### 7.1 录入流程
1. 操作管理员通过扫码签名登录。
2. 创建或查询公民档案。
3. 录入档案号、上传照片和指纹等资料。
4. 后端写入中心数据库并加密存储敏感数据。
5. 写入审计日志。

### 7.2 二维码生成与打印流程
1. 选择目标档案。
2. 生成二维码载荷并完成签名。
3. 打印纸质二维码。
4. 用户持二维码到 SFID 流程完成绑定。

### 7.3 管理员管理流程
1. 超级管理员登录。
2. 执行操作管理员增删改查。
3. 写入权限变更审计日志。

## 8. 数据设计
### 8.1 核心实体
- `admin_users`：管理员账户（含公钥、角色、状态）。
- `admin_login_challenges`：登录 challenge 记录与消费状态。
- `citizen_archives`：公民档案主数据。
- `biometric_assets`：照片、指纹等资料。
- `qr_print_records`：二维码生成与打印记录。
- `audit_logs`：审计日志。

### 8.2 关键约束
- 超级管理员最多 `3` 个（`K1/K2/K3` 各 1 个），绑定后禁止降级。
- `admin_users.admin_pubkey` 全局唯一。
- `citizen_archives.archive_no` 全局唯一。
- `admin_login_challenges` 一次性消费与过期控制。

### 8.3 存储策略
- 数据库为单一权威库（PostgreSQL）。
- 敏感字段（证件号、生物数据元信息）加密存储。
- 文件按档案分区存储并加密。

### 8.4 档案号规则（v1 冻结）
- 组成顺序：`省代码2 + 市代码3 + 校验码1 + 随机码9 + 日期码8`。
- 省市代码来源：与 SFID 使用同一套 `sheng_cities` 数据源。
- 省市代码接入方式：CPMS 在编译阶段从 SFID 仓库 `backend/src/sfid-tool` 读取并编译内置。
- 日期码：`YYYYMMDD`，取“档案号创建时间”。
- 公民可投票状态不进入档案号，状态仅在二维码字段中单独表达。
- 档案号仅允许后端生成，前端不可直传覆盖。

### 8.5 校验码算法（与 SFID 统一）
- 算法与 SFID `sfid_code` 校验位保持一致：`BLAKE3` 摘要后“字节和 mod 10”。
- 输入载荷（不含校验码本身）：
  - `cpms-archive-v3|{province2}{city3}{random9}{created_date8}`
- 计算步骤：
1. 对输入载荷做 `BLAKE3` 得到摘要字节数组。
2. 对摘要所有字节做无符号累加。
3. 对累加结果取模 `10`，得到 `0-9` 的 1 位校验码。
- 说明：
  - CPMS 的输入载荷字段与 SFID 不同，但校验位算法完全一致。
  - 算法版本变更时，必须升级 `cpms-archive-v*` 前缀并保留兼容解析。

### 8.6 随机码生成规则（9位）
- 随机码固定 9 位数字（`000000000` - `999999999`）。
- 随机码生成绑定因子：`生成时间戳 + 终端号 + 操作管理员公钥 + nonce`。
- 生成建议：对绑定因子做哈希/HMAC 后取模 `1_000_000_000`，不足左补零。
- 去重策略：`archive_no` 建唯一约束；冲突时自增 `nonce` 重算并重试。

### 8.7 版本兼容策略（规则扩展）
- 当前 `v3`：档案号为 `省2+市3+校验1+随机9+创建日期8`，不包含状态位。
- 若未来扩展档案号字段语义，按档案号版本升级解析规则，不回写历史档案号。
- 版本前缀变更时，必须保留历史版本的兼容解析。

## 9. API 设计（v1）
### 9.1 通用
- Base Path：`/api/v1`
- 成功：`{ code: 0, message: "ok", data: ... }`
- 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`

### 9.2 安装初始化接口
- `GET /install/status`
- `POST /install/initialize`（提交 SFID 初始化二维码内容）
- `POST /install/super-admin/bind`（`wuminapp` 绑定超级管理员）

### 9.3 认证接口（扫码签名）
- `POST /admin/auth/identify`
- `POST /admin/auth/challenge`
- `POST /admin/auth/verify`
- `POST /admin/auth/qr/challenge`
- `POST /admin/auth/qr/complete`
- `GET /admin/auth/qr/result`
- `POST /admin/auth/logout`

### 9.4 管理员管理接口
- `GET /admin/operators`
- `POST /admin/operators`
- `PUT /admin/operators/{id}`
- `DELETE /admin/operators/{id}`
- `PUT /admin/operators/{id}/status`
- `POST /admin/site-keys/registration-qr`（仅 `SUPER_ADMIN`）

### 9.5 公民资料接口
- `POST /archives`
- `PUT /archives/{archive_id}/citizen-status`
- `PUT /archives/{archive_id}`
- `GET /archives/{archive_id}`
- `GET /archives`
- `POST /archives/{archive_id}/biometrics/photo`
- `POST /archives/{archive_id}/biometrics/fingerprint`
- 约束：`PUT /archives/{archive_id}/citizen-status` 仅允许 `SUPER_ADMIN`。
- 约束：更新 `citizen_status` 时不得改变 `archive_no`，新状态通过二维码字段体现。

### 9.6 二维码接口
- `POST /archives/{archive_id}/qr/generate`
- `POST /archives/{archive_id}/qr/print`

### 9.7 接口文档归并说明
- 接口规范以本章为唯一基线，不再维护独立 `docs/openapi.yaml` 与 `docs/postman_collection.json`。
- 联调、测试、验收统一按本章接口、字段与约束执行。

## 10. 二维码规范（供 SFID 绑定）
### 10.1 字段
- `ver`
- `issuer_id`（固定 `cpms`）
- `site_sfid`
- `sign_key_id`（机构签名密钥标识：`K1/K2/K3`）
- `archive_no`
- `issued_at`
- `expire_at`
- `qr_id`
- `sig_alg`（`sr25519`）
- `citizen_status`（`NORMAL`|`ABNORMAL`，首绑二维码必填）
- `voting_eligible`（布尔，`citizen_status` 派生）
- `signature`

### 10.2 约束
- 二维码内容必须可被 SFID 扫码识别。
- 首绑二维码必须包含 `citizen_status`，用于 SFID 初始化用户状态。
- SFID 投票资格判定以二维码中的 `citizen_status/voting_eligible` 为准，不依赖 `archive_no` 推导状态。
- CPMS 负责签发与打印，SFID 负责绑定处理。
- CPMS 与 SFID 不进行在线通信。
- 每个机构安装初始化时必须先扫码 SFID 下发的 `SFID_CPMS_INSTALL` 二维码并完成验签。
- 每个机构初始化生成 3 把二维码签名私钥（主用/备用/应急）。
- 每把签名密钥生成后，CPMS 必须提供对应“超级管理员绑定二维码”供 `wuminapp` 扫码绑定。
- SFID 按 `site_sfid + sign_key_id` 录入该机构公钥后才认可该机构二维码（与 CPMS 超级管理员登录公钥分离）。
- 仅当机构状态在 SFID 侧为 `ACTIVE`，且该机构登记二维码可追溯到 SFID 原始签发的初始化载荷，后续业务二维码才进入可信链路。
- 若业务二维码验签失败、机构未登记、机构非 `ACTIVE`、或初始化链路校验不一致，SFID 必须拒绝受理。

### 10.3 公钥登记二维码（机构初始化）
- 用途：将机构 `site_sfid` 与本机构 3 把签名公钥登记到 SFID。
- 字段：`site_sfid`、`keys[{key_id,purpose,status,pubkey}]`、`issued_at`、`sign_key_id`、`signature`。
- 规则：SFID 超级管理员未录入该登记二维码前，SFID 不认可该机构出具的公民档案二维码。
- 权限：仅 `SUPER_ADMIN` 可生成公钥登记二维码。

### 10.4 与 SFID 对齐执行清单
- 机构号来源：必须使用 SFID 下发的 `site_sfid`，不得在 CPMS 本地自生成。
- 初始化顺序：先扫码 `SFID_CPMS_INSTALL` 并验签，再生成机构 3 把二维码签名密钥。
- 超管来源：通过 `wuminapp` 扫描“超级管理员绑定二维码”并回传绑定签名，CPMS 才创建 `SUPER_ADMIN` 账户。
- 公钥登记前置：未完成 SFID 侧公钥登记前，不得出具给用户使用的业务二维码。
- 首次绑定二维码：必须携带 `citizen_status` 字段，作为用户初次状态。
- 状态变更二维码：必须携带 `archive_no + citizen_status`，由 SFID 超级管理员扫码更新状态。
- 状态口径：`NORMAL` 可投票，`ABNORMAL` 不可投票。
- 原文与签名：严格按固定字段顺序与 UTF-8 编码签名，禁止字段重排。
- 重放约束：同一 `qr_id` 只能使用一次，不得重复签发同 `qr_id`。
- 联调顺序：机构公钥登记 -> 公民绑定二维码 -> 状态变更二维码。
- 信任闭环成立条件：`SFID 初始化二维码签发 -> CPMS 初始化 -> CPMS 公钥登记二维码回传并被 SFID 录入激活`。
- 闭环未成立时（含未按 SFID 初始化码初始化、或链路不匹配），CPMS 出具的公民档案号二维码/状态二维码在 SFID 侧不可信且应被拒绝。
- 验签失败或链路校验失败时，SFID 拒绝录入机构并拒绝后续该机构业务二维码。

## 11. 桌面壳方案
### 11.1 目标
- 安装后在电脑桌面生成 `CPMS` 图标。
- 点击图标直接打开系统登录页面。

### 11.2 行为
- 桌面壳固定打开内网地址（如 `http://cpms-host:8899/login`）。
- 使用独立窗口运行，不依赖用户手动打开浏览器标签页。
- 支持开机后快速启动（可选）。
- 目录归属：桌面壳位于 `frontend/desktop-shell/`。

## 12. 非功能要求
- 可用性：支持多电脑并发录入，核心服务可持续运行。
- 一致性：所有作业终端读取同一数据库，避免数据分叉。
- 安全性：公钥签名登录、最小权限、敏感数据加密。
- 可审计性：登录、管理、录入、打印全链路留痕。

## 13. 验收标准
### 13.1 功能验收
- 桌面壳安装后可在桌面生成图标并一键打开登录页。
- 管理员可通过公钥扫码签名登录。
- 安装初始化后可通过 `wuminapp` 完成最多 `3` 个超级管理员绑定。
- 超级管理员可增删改查操作管理员。
- 多台电脑可同时录入并看到同一份数据。
- 可成功打印二维码并被 SFID 流程使用。

## 14. WUMINAPP 扫码登录实现对齐（当前口径）

### 14.1 当前状态
- 协议统一：`WUMINAPP_LOGIN_V1`。
- 登录模式：离线双向扫码（挑战码 -> 手机确认签名 -> 回执码 -> 系统验签）。
- 实施口径：与 SFID/CitizenNode 使用同一签名原文规则和回执字段。
- 责任边界：`wuminapp` 仅负责挑战签名并展示签名二维码；CPMS 端独立判定验签是否成功，不向手机端回传登录结果。

### 14.2 挑战码（CPMS -> 手机）
```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "cpms",
  "request_id": "uuid",
  "challenge": "string",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000090,
  "aud": "cpms-local-app",
  "origin": "cpms-device-id"
}
```

### 14.3 签名原文（固定）
```text
WUMINAPP_LOGIN_V1|cpms|aud|origin|request_id|challenge|nonce|expires_at
```

### 14.4 回执码（手机 -> CPMS）
```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "request_id": "uuid",
  "account": "ss58-address",
  "pubkey": "0x...",
  "sig_alg": "sr25519",
  "signature": "0x...",
  "signed_at": 1760000020
}
```

### 14.5 CPMS 验签顺序
1. 解析回执并读取 `request_id/account/signature`。
2. 按挑战缓存重建签名原文。
3. 使用 `sr25519` 验签。
4. 校验挑战固定 `90` 秒时效与 `request_id` 一次性消费。
5. 验签通过后判定是否为管理员（3 超管 + 操作管理员），是则登录，否则拒绝。

### 13.2 安全验收
- 非管理员公钥无法登录。
- challenge 重放被拦截。
- 非超级管理员无法管理操作管理员。
- 敏感数据未明文落盘。

## 14. 变更控制
- 本文档为 CPMS 当前冻结技术基线。
- 新需求必须先更新本文档并评审后再开发。

## 15. 开发实施计划（v1）
### 15.1 计划目标
- 按本技术文档完成 CPMS v1 全量交付：离线部署、扫码签名登录、管理员分权、公民档案录入、二维码签发打印、全链路审计。
- 以“可部署、可联调、可验收”为计划闭环，不以单点功能完成作为里程碑。

### 15.2 总体排期（基线）
- 计划起始日期：2026-02-26。
- 计划结束日期：2026-05-07。
- 总周期：10 周（含联调与验收）。

### 15.3 阶段划分与交付物
#### 阶段 0：基线冻结（2026-02-26 ~ 2026-03-01）
- 目标：冻结范围、接口、数据字典与验收口径。
- 交付物：
  - 范围基线清单（In/Out Scope）。
  - API 清单与命名对齐结果。
  - 数据字典与字段口径说明。
  - 功能/安全验收用例草案。

#### 阶段 1：架构与基础设施（2026-03-02 ~ 2026-03-15）
- 目标：完成后端可运行骨架与离线部署底座。
- 交付物：
  - PostgreSQL 结构与迁移脚本落地。
  - 文件加密存储目录结构与访问策略。
  - 统一配置管理（站点、密钥、存储、日志）。
  - 审计日志中间件与 trace_id 贯通。
  - 内网部署脚本初版与启动手册。

#### 阶段 2：认证与权限（2026-03-16 ~ 2026-03-29）
- 目标：完成管理员公钥登录链路与后端 RBAC 强校验。
- 交付物：
  - `/api/v1/admin/auth/identify`、`/challenge`、`/verify`、`/logout`。
  - challenge 一次性消费、过期控制与防重放。
  - 安装初始化扫码验签 + `wuminapp` 绑定 `SUPER_ADMIN`（最多 3 个）。
  - 越权请求拦截与审计日志记录。

#### 阶段 3：管理员管理（2026-03-30 ~ 2026-04-05）
- 目标：完成操作管理员管理闭环。
- 交付物：
  - `/api/v1/admin/operators` 增删改查与状态管理。
  - 仅 `SUPER_ADMIN` 可执行管理员管理接口。
  - 前端管理页面与权限态控制。
  - 关键操作审计日志（创建、禁用、重置、删除）。

#### 阶段 4：档案与生物资料（2026-04-06 ~ 2026-04-19）
- 目标：完成公民档案核心业务闭环。
- 交付物：
  - `/api/v1/archives` 新建、更新、查询、列表。
  - 档案号生成规则落地（省2+市3+校验1+随机9+创建日期8）。
  - 校验码算法（BLAKE3 字节和 mod 10）落地与测试向量。
  - 照片/指纹上传接口与加密存储。
  - `archive_no` 唯一约束冲突重试机制（nonce 递增）。

#### 阶段 5：二维码签发与打印（2026-04-20 ~ 2026-04-26）
- 目标：完成 SFID 对接所需二维码交付链路。
- 交付物：
  - `/api/v1/archives/{archive_id}/qr/generate`、`/qr/print`。
  - 二维码载荷字段与签名格式实现（`site_sfid`、`archive_no`、`signature` 等）。
  - 机构初始化 3 把签名密钥（主用/备用/应急）与公钥登记二维码。
  - 打印记录与可追溯审计。

#### 阶段 6：桌面壳与联调验收（2026-04-27 ~ 2026-05-07）
- 目标：完成端到端联调、压测与验收交付。
- 交付物：
  - `frontend/desktop-shell` 安装包（桌面图标、一键打开登录页）。
  - 多终端并发录入与同库一致性验证报告。
  - 功能、安全、可审计验收报告。
  - 上线包与运维交接文档（备份、恢复、故障处理）。

### 15.4 里程碑与通过标准
- M1（2026-03-15）：基础设施完成，可部署可启动，数据库与审计链路可用。
- M2（2026-03-29）：扫码签名登录与 RBAC 生效，越权拦截与重放防护通过测试。
- M3（2026-04-19）：档案录入主流程跑通，档案号规则与生物资料上传稳定。
- M4（2026-05-07）：二维码签发打印可用，桌面壳可交付，整体验收通过。

### 15.5 全程保障任务
- 测试保障：单元测试、集成测试、回归测试、安全测试（重放、越权、明文检查）。
- 运维保障：离线部署手册、备份恢复演练、日志轮转与容量预警。
- 安全保障：登录密钥与二维码签名密钥隔离，密钥轮换与应急预案。

### 15.6 关键风险与控制措施
- 风险：现有实现与本技术文档接口定义不一致。
  - 控制：阶段 0 输出“差异清单”，未对齐项不得进入开发。
- 风险：签名登录联调复杂度高。
  - 控制：先完成验签测试向量与模拟器，再接入前端。
- 风险：并发写入导致档案号冲突。
  - 控制：数据库唯一约束 + nonce 重试，冲突全量审计。
- 风险：打印设备兼容差异影响现场交付。
  - 控制：提前确定打印机型号并建立兼容性测试矩阵。

### 15.7 计划变更规则
- 任一阶段范围、排期、验收口径发生变更，必须先更新本章节并评审通过后执行。
- 未更新本章节的新增需求，不得进入开发与发布流程。
