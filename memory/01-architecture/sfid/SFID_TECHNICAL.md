# SFID 技术开发文档（统一版）

## 1. 文档目的
- 固化 SFID 最新业务流程与系统边界，作为开发、测试、联调、验收基线。
- 本文档已合并历史方案，不再依赖已删除的散落文档。
- 本系统仅保留本文件作为唯一技术文档基线；后续规格变更统一回写本文件。

## 2. 系统定位与边界
### 2.1 系统关系
- `CPMS`：完全离线独立系统，仅负责由线下工作人员获取并出具“带签名和档案号”的二维码。
- `SFID`：在线系统，负责扫码验签、档案号与区块链公钥绑定、解绑、自动对链响应。
- `Blockchain`：区块链系统，传入公钥和查询请求，接收 SFID 的绑定结果与资格校验结果。

### 2.2 范围内
- SFID 唯一联网管理员网站（Web）。
- SFID 公开查询页面（无需登录）。
- SFID API 服务。
- SFID Signer 签名服务。
- 数据库与审计系统。

### 2.3 范围外
- CPMS 内部建档与签章实现。
- CPMS 与 SFID 的系统直连、在线接口集成。
- 区块链 Runtime/Pallet 内部实现改造。

## 3. 参与方与职责
- 办证工作人员：在线下 CPMS 建档，生成二维码。
- SFID 机构管理员：具备除”密钥管理”外的全部业务与管理能力。
- SFID 系统管理员：执行绑定、解绑、查询用户信息等业务操作。
- 区块链端：调用 SFID 自动接口获取可投票人数、绑定有效性等结果。
- 普通用户：不登录管理员后台，但可使用公开查询页查询档案号、身份识别码、公钥地址。

## 3.1 区块链五项能力模块归属（对齐口径）
1. 机构 SFID 登记（多签创建前置）：`super-admins` 模块（`sfid_id` 对应内部 `site_sfid`）。
2. 公民身份绑定凭证：`chain` 模块（`/api/v1/bind/result`）。
3. 公民投票凭证：`chain` 模块（`/api/v1/vote/verify`）。
4. 联合投票人口快照：`chain` 模块（按 Runtime payload 输出 `eligible_total + snapshot_nonce + signature`，`who` 必入签名）。
5. SFID 验签主备账户管理：`key-admins` 模块（一主两备与轮换）。

## 4. 管理员模型与登录机制
### 4.1 管理员类型
- 密钥管理员（`KEY_ADMIN`）：
1. 数量固定 3 个（主密钥管理员 1 + 备用密钥管理员 2）。
2. 来源为当前一主两备公钥状态，随密钥轮换自动同步。
3. 权限：密钥管理最高权限（查看密钥状态、发起/提交主密钥轮换）、全局管理权限（可跨省管理系统管理员、可更换省级机构管理员、可执行业务接口查询与状态操作）。
4. 不具备机构管理权限（不能访问机构管理页、不能生成机构身份识别码、不能扫码录入机构）。
- 机构管理员（`INSTITUTION_ADMIN`）：
1. 数量固定 43 个（每省 1 个）。
2. 初始公钥清单在代码中固化（编译期常量）并在安装时入库；运行中可由密钥管理员按省替换。
3. 权限：除密钥管理外全权限（系统管理员管理、机构管理、绑定/解绑/查询、状态变更扫码等）。
4. 机构管理员角色不可降级；公钥替换仅允许密钥管理员执行。
5. 权限范围：01-43 号机构管理员仅可查看/启用/停用/删除自己创建的系统管理员。
- 系统管理员（`SYSTEM_ADMIN`）：
1. 数量不设上限。
2. 仅能由机构管理员创建、修改、停用、删除。
3. 权限：登录后执行绑定、解绑、查询用户信息，不可管理管理员账号。

### 4.4 前端统一形态
- 密钥管理员、机构管理员与系统管理员使用同一套管理员网站前端与同一登录流程。
- 三类管理员共享基础业务页面（绑定、解绑、查询、状态变更扫码）；机构管理员额外拥有”管理员/机构管理”，密钥管理员额外拥有”密钥管理/省级机构管理员更换”。
- 密钥管理员登录后额外显示”密钥管理”菜单；机构管理员登录后额外显示”管理员””机构管理”菜单。
- 菜单显示仅做体验控制，最终权限以后端 RBAC 为准。

### 4.2 账户模型
- 管理员账户标识为公钥（`admin_pubkey`）。
- 不使用用户名密码模式作为主登录机制。
- 管理员私钥仅用于签名，不上传、不落库。

### 4.3 登录流程（二维码挑战签名）
1. 管理员点击“生成登录二维码”，SFID 前端向后端申请一次性登录 challenge。
2. SFID 页面展示登录二维码（`WUMIN_LOGIN_V1.0.0` 协议）。
3. 管理员手机扫码后完成签名，并在手机端展示“签名结果二维码”。
4. SFID 前端扫描该签名结果二维码（或粘贴签名原文）后，提交到 `/api/v1/admin/auth/qr/complete`。
5. SFID 校验扫码公钥与签名：是管理员则登录管理员模式，不是管理员或签名失败则拒绝登录。
6. 页面轮询 challenge 结果，成功后自动写入会话并完成登录。
7. challenge 固定有效期 `90` 秒，且 `challenge` 一次性消费，防重放。
8. 登录二维码协议固定为 `WUMIN_LOGIN_V1.0.0`，字段必须包含：
`proto/system/challenge/issued_at/expires_at/sys_pubkey/sys_sig`（时间戳为秒级）。
9. 系统先使用自身私钥对登录二维码签名；手机验 `sys_pubkey + sys_sig` 后，管理员钱包再对登录 challenge 签名。
10. 手机端登录签名原文固定为：`WUMIN_LOGIN_V1.0.0|system|challenge|expires_at`。
11. `origin`/`domain`/`session_id` 仅作为网页侧上下文，不属于移动端扫码验签协议字段。

## 5. 核心业务流程
### 5.1 绑定流程（人工）
1. 用户在区块链前端提交公钥，公钥传入 SFID 形成待绑定记录。
2. 用户在线下办理后，由工作人员从 CPMS 获得签名二维码（包含档案号）。
3. SFID 管理员在在线前端网站扫码二维码。
4. SFID 验证二维码签名与档案号有效性。
5. SFID 将“档案号 + 公钥”写入绑定关系，状态置为 `ACTIVE`，并写入二维码中的初始状态（`NORMAL`/`ABNORMAL`）。
6. SFID 回传绑定成功消息给区块链。

### 5.2 解绑流程（人工）
1. 用户必须线下联系 SFID 管理员发起解绑。
2. 管理员在 SFID 前端执行解绑并填写原因。
3. SFID 更新状态为 `UNBOUND`，写入审计日志。
4. SFID 回传解绑结果给区块链。

### 5.3 自动查询/校验流程（无管理员参与）
- 区块链请求当前可投票公民数量，SFID 自动返回统计结果（仅统计状态为 `NORMAL` 且绑定有效用户）。
- 区块链请求“档案号-公钥绑定是否有效”，SFID 自动返回校验结果（含 CPMS 二维码状态）。

### 5.4 公开查询流程（无需登录）
1. 用户访问 SFID 公开查询页面。
2. 输入查询条件（档案号/身份识别码/公钥地址任一）。
3. 系统返回档案号、身份识别码、公钥地址三项对应信息（若存在）。

## 6. 功能清单
### 6.1 管理员人工功能
- 扫码并解析 CPMS 二维码。
- 机构管理员、系统管理员扫码”CPMS 状态变更二维码”更新用户状态（密钥管理员同样可执行）。
- 绑定确认（档案号、公钥唯一性校验）。
- 解绑处理（线下受理、线上执行）。
- 绑定查询与审计查询。
- 机构管理员与密钥管理员维护系统管理员账号（新增、启停用、删除；机构管理员按创建者隔离）。
- 系统管理员数据模型新增姓名字段 `admin_name`；新增管理员必须提交姓名与公钥。
- 管理员新增/修改系统管理员时，`admin_pubkey` 必须通过 `sr25519` 公钥格式校验（非法输入拒绝）。
- 管理员”修改”弹窗支持同时修改姓名和公钥。
- 身份信息页“操作”列内按钮文案固定为“绑定”“变更”（列名仍为“操作”）。
- 机构管理员维护机构管理：
1. 先在机构页生成机构身份识别码（调用 `sfid`，`A3=GFR`,`P1=0`，不输入公钥）。
2. 持 SFID 二维码去 CPMS 初始化系统，CPMS 初始化后生成机构公钥登记二维码。
3. 回到 SFID 机构页扫码录入机构，完成 3 把机构公钥入库并激活。

### 6.3 权限控制实现要求
- 绑定/解绑/扫码状态变更等写接口允许：`KEY_ADMIN`、`INSTITUTION_ADMIN`、`SYSTEM_ADMIN`（按具体接口约束）。
- 机构管理接口（机构身份识别码生成、机构扫码录入、机构更新/禁用/撤销/删除/查询）仅允许：`INSTITUTION_ADMIN`。
- 绑定信息查询接口允许：`KEY_ADMIN`、`INSTITUTION_ADMIN`、`SYSTEM_ADMIN`。
- 管理系统管理员接口允许：`KEY_ADMIN`、`INSTITUTION_ADMIN`（`KEY_ADMIN` 可跨创建者全局管理）。
- 密钥管理接口（`attestor/keyring`、`rotate/challenge`、`rotate/commit`）仅允许：`KEY_ADMIN`。
- 系统管理员管理接口的对象权限：按 `created_by` 隔离，仅可管理自己创建的系统管理员。
- 省级数据强隔离：每省 1 个机构管理员；机构管理员与其创建的系统管理员仅可查看和操作本省数据。
- 隔离范围：机构（CPMS 机构登记）、公民绑定信息、SFID 生成与状态变更。
- `cpms_site_keys` 必须记录 `admin_province`，并在机构查询、扫码验签、状态变更时做后端强校验。
- 待绑定公民在首次生成 SFID 后锁定所属省，后续仅该省管理员可见、可操作。
- 前端隐藏按钮不等于授权，后端必须对每个接口做角色校验。

### 6.2 系统自动功能
- 校验二维码签名。
- 同步绑定/解绑结果给区块链。
- 响应区块链资格查询与有效性查询。
- 提供无需登录的公开查询接口。
- 根据 CPMS 二维码状态同步投票资格（`NORMAL`/`ABNORMAL`）。

## 7. 技术架构与栈
### 7.1 前端（管理员网站）
- `React + TypeScript + Vite + Ant Design`
- Web 部署，不以桌面应用为目标形态。
- 登录页当前口径：
1. 整页背景图（含顶部区域）由前端静态资源提供（`/assets/login-bg.png`）。
2. 顶部左侧文案两行：`中华民族联邦共和国`（上）与 `身份识别码系统`（下，右下对齐）。
3. 登录流程为“页面出示登录二维码 + 手机扫码签名 + 前端扫描签名二维码提交”，前端轮询登录状态并自动登录。
4. 非管理员扫码登录直接拒绝；签名失败同样拒绝登录。

### 7.2 后端
- `Rust + Axum`
- `PostgreSQL`
- `Redis`（可选，用于限流和缓存）
- 运行态持久化：`DATABASE_URL` 指向 PostgreSQL；后端通过 `runtime_misc`、`runtime_meta` 持久化运行态数据，管理员与密钥槽位使用结构化表持久化（见 8.1）。
- 签名密钥缓存：运行时签名 keypair 缓存命中时不重复解码 seed，缓存中的 seed 文本使用 `SensitiveSeed` 存储并在释放时清零。
- 旧文件态与旧整包表状态说明：
1. 不再使用 `backend/data/runtime_state.json`。
2. 不再使用 `runtime_store`（已由迁移脚本下线并拆分）。
- 当前管理员与密钥结构化表：
1. `admins`
2. `provinces`
3. `super_admin_scope`
4. `operator_admin_scope`
5. `key_admin_keyring`
- 角色逻辑分层目录（已落地）：
1. `backend/src/key-admins/`：密钥管理员逻辑（密钥轮换、机构管理员替换、链证明签名/公钥输出）。
2. `backend/src/super-admins/`：机构管理员逻辑（已拆分为 `operators.rs` 管理员管理、`institutions.rs` 机构管理）。
3. `backend/src/operator-admins/`：系统管理员入口逻辑（角色入口与路由适配）。
4. `backend/src/business/`：共用后台查询与审计能力（查询、审计、省域隔离）。
5. `backend/src/operate/`：操作业务逻辑（管理员绑定流程、状态扫码、CPMS 二维码验签）。
6. `backend/src/chain/`：区块链业务接口逻辑（公钥绑定、公民数获取、投票验证、链侧校验/回执）。
7. `backend/src/sfid/`：SFID 生成与元数据模块（码生成工具 + 管理端 SFID 业务接口）。
8. `backend/src/models/`：统一数据结构模块（领域模型、接口 DTO、状态枚举）。
9. `backend/src/main.rs`：路由装配与启动骨架（核心能力已下沉到模块）。
- 架构冻结口径：管理员治理与机构治理继续在 `super-admins` 目录内演进，不新增独立模块。

### 7.3 签名与密钥
- 当前版本按角色目录管理签名密钥与验签流程：密钥管理与链证明在 `key-admins`，机构登记校验在 `super-admins`，CPMS 业务二维码验签在 `operator-admins`。
- CPMS 机构密钥方案：每个市级机构（`site_sfid`）维护 3 套二维码签名密钥。
- SFID 信任库存储维度：`site_sfid + key_id + pubkey`，按机构隔离验签。
- 密钥分期：
1. v1.0：国家级单签名私钥。
2. v2.0：Root + 省级分层密钥。

### 7.4 角色目录能力分配（已落地）
- `backend/src/key-admins/chain_keyring.rs`：SFID 区块链签名密钥（一主两备）管理、轮换状态机、轮换签名验签。
- `backend/src/key-admins/chain_proof.rs`：区块链业务证明签名封装（公民数、投票资格、公钥绑定/SFID/奖励相关证明）与公钥输出。
- `backend/src/super-admins/institutions.rs`：CPMS 机构管理与登记校验（`site_sfid` 与机构公钥信任建立）。
- `backend/src/operate/binding.rs`：管理员绑定扫码/确认/解绑实现。
- `backend/src/operate/status.rs`：CPMS 状态变更扫码业务。
- `backend/src/business/query.rs`：身份信息查询、按公钥查询等后台查询能力。
- `backend/src/operate/cpms_qr.rs`：CPMS 二维码原文规范化与验签共用方法。
- `backend/src/sfid/admin.rs`：管理端 SFID 生成、元数据与城市列表查询业务。
- `backend/src/business/audit.rs`：审计日志查询共用实现。
- `backend/src/business/scope.rs`：省域隔离与作用域判定共用实现。
- `backend/src/chain/binding.rs`：链侧公钥绑定请求/结果、绑定校验、奖励回执与状态查询。
- `backend/src/chain/voters.rs`：链侧公民数获取。
- `backend/src/chain/vote.rs`：链侧投票资格验证。
- `backend/src/chain/CHAIN_TECHNICAL.md`：链侧接口与参数对齐说明（含绑定凭证/投票凭证/人口快照对齐口径）。
- `backend/src/sfid/mod.rs`：SFID 码生成工具主实现（由管理端生成接口调用）。
- `backend/src/models/mod.rs`：后端统一数据结构定义（Store、DTO、状态枚举）。
- 主密钥轮换规则（强约束）：
1. 区块链验证只使用主公钥（`main`）。
2. 功能 1/2/3/4 的链上可信输出统一只认当前 `main`。
3. 更换主公钥只能由两把备用公钥之一发起。
4. 发起轮换时必须提交一把新公钥替换被提升的备用槽位。
5. 被用于发起的旧备用提升为新主公钥，旧主公钥退出活动集，结果始终保持“一主两备”。
- 轮换接口流程（Runtime 对齐口径）：
1. 密钥管理员（且必须为当前备用公钥）调用 `rotate/challenge` 生成一次性挑战原文。
2. 指定备用公钥对应私钥对挑战原文签名。
3. 调用 `rotate/verify` 校验签名确认为备用密钥签名。
4. 调用 `rotate/commit` 提交 `challenge_id + signature + new_backup_pubkey`，并由后端调用链上标准 extrinsic（如 `rotate_sfid_keys`）执行轮换。
5. 提交后必须回写 `chain_tx_hash` 与 `block_number`，用于对账与审计。
- 前端可视化流程（当前实现）：
1. 密钥管理员在“密钥管理”页面输入 `initiator_pubkey` 生成轮换二维码。
2. 备用私钥钱包扫码二维码并签名。
3. 前端摄像头扫码签名结果二维码，先执行 `rotate/verify`。
4. 验签通过后输入“新备用公钥（可选新备用 seed）”并提交 `rotate/commit`。
5. 轮换成功后页面自动刷新并展示最新一主两备状态。

## 8. 数据模型
### 8.1 核心表（当前落地）
- 当前主流程持久化表：
1. `runtime_cache_entries`：运行态分片缓存（JSONB，按 `entry_key` 存储）。
2. `runtime_misc`：运行态兼容快照。
3. `runtime_meta`：运行态元数据（签名种子/公钥，含加密载荷）。
4. `admins`：管理员主表（`KEY_ADMIN|INSTITUTION_ADMIN|SYSTEM_ADMIN`）。
5. `provinces`：省份维度表。
6. `super_admin_scope`：机构管理员省域归属（含 `scope_no`）。
7. `operator_admin_scope`：系统管理员归属机构管理员关系。
8. `key_admin_keyring`：密钥管理员一主两备槽位映射。
9. `chain_idempotency_requests`：链路幂等与防重放记录。
10. `binding_unique_locks`：绑定唯一性锁（公钥/档案号双向唯一）。
11. `bind_reward_states`：奖励回执状态机。
- 管理员视图：
1. `v_key_admins`
2. `v_super_admins`
3. `v_operator_admins`
- 历史兼容表（仍在迁移历史中）：
1. `bind_requests`
2. `archive_bindings`
3. `admin_login_challenges`
4. `audit_logs`
5. `cpms_site_keys`

### 8.2 关键约束
- `admins.admin_pubkey` 全局唯一。
- `super_admin_scope.province_name` 唯一（每省仅 1 名机构管理员）。
- `super_admin_scope.scope_no` 唯一（1..43 编号）。
- `key_admin_keyring.slot` 固定且唯一：`MAIN|BACKUP_A|BACKUP_B`。
- `chain_idempotency_requests` 双唯一：`(route_key, request_id)` 与 `(route_key, nonce)`。
- `binding_unique_locks`：`account_pubkey` 与 `archive_index` 均唯一。
- `bind_reward_states`：`account_pubkey` 与 `callback_id` 均唯一。

### 8.3 状态机
- `audit_logs.result`：`SUCCESS | FAILED`
- `admins.role`：`KEY_ADMIN | INSTITUTION_ADMIN | SYSTEM_ADMIN`
- 会话角色：`KEY_ADMIN | INSTITUTION_ADMIN | SYSTEM_ADMIN`
- `admins.status`：`ACTIVE | DISABLED`
- 登录挑战：运行态 `login_challenges`（一次性消费 + TTL）。
- 奖励状态：`PENDING | RETRY_WAITING | FAILED | REWARDED`。

## 9. API 设计（v1 建议）
### 9.1 通用返回
- 成功：`{ code: 0, message: "ok", data: ... }`
- 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`

### 9.2 管理员认证与账号管理接口
- `POST /api/v1/admin/auth/identify`：扫码识别管理员身份二维码（入参 `identity_qr`）。
- `POST /api/v1/admin/auth/challenge`：创建登录挑战二维码。
- `POST /api/v1/admin/auth/verify`：提交签名并完成登录。
- `POST /api/v1/admin/auth/qr/challenge`：生成网页登录二维码 challenge。
- `POST /api/v1/admin/auth/qr/complete`：提交签名结果（`challenge_id/request_id + admin_pubkey + signature`，`session_id` 可选）。
- `GET /api/v1/admin/auth/qr/result`：网页登录页轮询二维码登录结果。
- `GET /api/v1/admin/operators`：查询系统管理员列表（`INSTITUTION_ADMIN | KEY_ADMIN`）。
- `POST /api/v1/admin/operators`：新增系统管理员（`INSTITUTION_ADMIN | KEY_ADMIN`）。
- `PUT /api/v1/admin/operators/{id}`：修改系统管理员（`INSTITUTION_ADMIN | KEY_ADMIN`）。
- `DELETE /api/v1/admin/operators/{id}`：删除系统管理员（`INSTITUTION_ADMIN | KEY_ADMIN`）。
- `PUT /api/v1/admin/operators/{id}/status`：启用/停用系统管理员（`INSTITUTION_ADMIN | KEY_ADMIN`）。
- 系统管理员接口口径补充：列表返回 `admin_name` 与 `created_by_name`（创建者显示名）；新增接口提交 `admin_name + admin_pubkey`；修改接口支持同时更新姓名与公钥，且后端校验 `admin_pubkey` 格式。

### 9.6 机构管理员基线与变更策略（当前）
1. 机构管理员基线采用 `scope_no(1..43) + province_name + admin_pubkey` 固化清单初始化（迁移脚本维护）。
2. 运行中允许由密钥管理员通过接口按省替换机构管理员公钥：`PUT /api/v1/admin/super-admins/:province`。
3. 替换后必须同步写入审计日志，并保持 `super_admin_scope` 的省份唯一与编号唯一约束。
4. 非密钥管理员不得替换机构管理员公钥。

### 9.3 管理员业务接口（人工）
- `POST /api/v1/admin/bind/scan`：上传二维码内容并验签解析（仅允许扫描本省已登记机构的二维码）。
- `POST /api/v1/admin/bind/confirm`：确认绑定档案号与公钥（必须携带 `bind/scan` 返回的 `qr_id`）。
- `POST /api/v1/admin/bind/unbind`：执行解绑。
- `GET /api/v1/admin/bindings`：查询绑定关系。
- `GET /api/v1/admin/sfid/meta`：获取 SFID 生成工具元数据（A3 选项、机构选项、省列表、当前管理员省域限制）。
- `GET /api/v1/admin/sfid/cities?province=...`：按省加载可选市列表（省级管理员仅可查询本省）。
- `POST /api/v1/admin/sfid/generate`：使用后端 Rust 工具生成指定用户 SFID 码（首次生成后锁定该公民所属省）。
- `GET /api/v1/admin/attestor/keyring`：查询 SFID 区块链签名密钥当前一主两备状态（仅密钥管理员）。
- `POST /api/v1/admin/attestor/rotate/challenge`：发起主密钥轮换 challenge（仅指定发起备用公钥）。
- `POST /api/v1/admin/attestor/rotate/verify`：校验 challenge 签名是否来自发起备用公钥。
- `POST /api/v1/admin/attestor/rotate/commit`：提交 `challenge_id + signature + new_backup_pubkey` 执行轮换。
- 密钥管理员可在前端“密钥管理”页面完成可视化操作（二维码生成 + 扫码签名 + 验签 + 输入新备用公钥提交）。
- `GET /api/v1/admin/super-admins`：查询 43 省机构管理员列表（仅密钥管理员）。
- `PUT /api/v1/admin/super-admins/:province`：按省替换机构管理员公钥（仅密钥管理员）。
- `POST /api/v1/admin/cpms-keys/sfid/generate`：生成机构身份识别码与 SFID 签名初始化二维码（仅机构管理员）。
- `POST /api/v1/admin/cpms-keys/register-scan`：扫描并录入 CPMS 公钥登记二维码（仅机构管理员，且必须绑定对应 `init_qr_payload`）。
- `POST /api/v1/admin/cpms-status/scan`：机构管理员/系统管理员扫描 CPMS 状态变更二维码并更新用户状态（密钥管理员同样可执行；省级角色仅可操作本省机构与本省公民）。
- `GET /api/v1/admin/audit-logs`：查询审计日志（密钥管理员/机构管理员，可按 action/actor/keyword 过滤）。
- `GET /api/v1/admin/cpms-keys`：查询机构列表（仅机构管理员，返回本省机构）。

### 9.4 区块链接口（自动）
- `GET /api/v1/chain/voters/count?account_pubkey=<who>`：返回 `genesis_hash`、`who`、`eligible_total`、`snapshot_nonce`、`signature`；签名 payload 必须包含 `who(account)`。
- `POST /api/v1/chain/binding/validate`：校验档案号与公钥绑定是否有效。
- `POST /api/v1/chain/reward/ack`：区块链回执绑定奖励处理结果（`SUCCESS/FAILED`）。
- `GET /api/v1/chain/reward/state`：查询绑定奖励状态机。
- `GET /api/v1/bind/result`：查询某公钥绑定结果；绑定成功后返回持久化 Runtime 凭证（`genesis_hash/who/binding_id/bind_nonce/signature`），同一公钥重复查询不会生成新 `bind_nonce`。
- `GET /api/v1/bind/result`：`signature` 为 Runtime 绑定凭证签名，旧 `sfid_signature` 不再对链输出。
- `POST /api/v1/vote/verify`：`proposal_id` 必填，输出投票验签凭证字段对齐 Runtime（`genesis_hash/who/binding_id/proposal_id/vote_nonce/signature`），不返回 `sfid_code` 明文。
- 鉴权要求：仅接受区块链调用方请求，请求头必须携带：
  - `x-chain-token`
  - `x-chain-request-id`
  - `x-chain-nonce`
  - `x-chain-timestamp`（Unix 秒，5 分钟有效窗）
  - `x-chain-signature`（必填）
- 签名规范（当启用 `SFID_CHAIN_SIGNING_SECRET`）：
  - Canonical payload（按以下顺序、`\n` 分隔）：
    - `route=<route_key>`
    - `request_id=<x-chain-request-id>`
    - `nonce=<x-chain-nonce>`
    - `timestamp=<x-chain-timestamp>`
    - `fingerprint=<request_fingerprint>`
  - 签名值：`hex(blake2b_mac_256(blake2b_256(SFID_CHAIN_SIGNING_SECRET), payload))`
- 幂等与防重放：
  - 进程内：`chain_requests_by_key + chain_nonce_seen`（24 小时窗口）。
  - 数据库：`chain_idempotency_requests(route_key, request_id|nonce)` 双唯一约束。
- 投票资格规则：以 CPMS 二维码状态为准（SFID 记录并反馈），`ABNORMAL` 状态不可投票。
- `/api/v1/vote/verify` 使用 5 秒短缓存（按 `account_pubkey + proposal_id`），状态变更/绑定变更会即时失效缓存。
- 绑定凭证刷新规则：若当前 signer 公钥或 `key_id/key_version/alg` 与已持久化 Runtime 凭证不一致，会自动重签发并覆盖持久化凭证。

### 9.9 App API 接口（移动端专用）
- 路由组：`/api/v1/app/*`
- 鉴权方式：请求头 `x-app-token`，服务端与环境变量 `SFID_APP_TOKEN` 比对。
- 用途：为移动端（wuminapp）提供专用接口，采用静态 Token 鉴权，无需链路 HMAC 签名，认证复杂度低于区块链接口。
- 限流：共享全局限流器，与其他接口统一限流策略。
- 源码位置：`sfid/backend/src/chain/app_api.rs`

#### 9.9.1 人口快照查询
- `GET /api/v1/app/voters/count?who=<pubkey_hex>`
- 返回字段：`eligible_total`、`snapshot_nonce`、`signature`、`who`、`as_of`
- 核心逻辑复用 `build_population_signature()`，与链路 `/api/v1/chain/voters/count` 签名产出一致。

#### 9.9.2 公民投票凭证
- `POST /api/v1/app/vote/credential`
- 请求体：`{ "who": "<pubkey>", "proposal_id": 42 }`
- 返回字段：`binding_id`、`vote_nonce`、`vote_signature`（仅资格合格时签发）
- 核心逻辑复用 `build_vote_credential()`，与链路 `/api/v1/vote/verify` 凭证产出一致。

#### 9.9.3 身份绑定请求
- `POST /api/v1/app/bind/request`
- 请求体：`{ "account_pubkey": "<pubkey>", "callback_url": "..." }`
- 绑定业务逻辑与链路绑定接口相同，仅鉴权方式替换为 App Token。

#### 9.9.4 App Token 配置说明
- 新增环境变量：`SFID_APP_TOKEN`（在部署脚本中配置）。
- 移动端编译时通过 `--dart-define=WUMINAPP_API_TOKEN=<同一值>` 注入。
- App Token 与 Chain Token（`SFID_CHAIN_TOKEN`）为独立凭据，安全级别不同，不可混用。

### 9.8 CPMS 状态变更扫码接口（人工）
- `POST /api/v1/admin/cpms-status/scan`：机构管理员/系统管理员扫描 CPMS 状态变更二维码并更新用户状态。
- 鉴权要求：`INSTITUTION_ADMIN`、`SYSTEM_ADMIN`、`KEY_ADMIN` 可调用（省级角色仍受省域隔离）。

### 9.5 公开查询接口（Token 鉴权）
- `GET /api/v1/public/identity/search?archive_no=...`
- `GET /api/v1/public/identity/search?identity_code=...`
- `GET /api/v1/public/identity/search?account_pubkey=...`
- 返回字段：`found`, `archive_no`, `identity_code`, `account_pubkey`
- 访问控制：必须携带 `x-public-search-token`（服务端配置 `SFID_PUBLIC_SEARCH_TOKEN`）并受全局限流。
- 可选返回：`is_voting_eligible`, `citizen_status`（以 CPMS 二维码状态为准）。

### 9.7 回调/通知（已落地）
1. 轮询模式：区块链可调用 `/api/v1/bind/result` 查询绑定结果。
2. 回调模式：SFID 绑定成功后发送 `BIND_CONFIRMED` webhook（失败指数退避重试，最多 5 次）。
3. 回调地址约束（防 SSRF）：
   - 默认必须为 `https://`，仅开发联调可通过 `SFID_ALLOW_INSECURE_CALLBACK_HTTP=true` 放开 `http://`。
   - 禁止 `localhost` 与私网/本地 IP 字面量。
   - 可通过 `SFID_CALLBACK_ALLOWED_HOSTS`（逗号分隔，支持 `*.example.com`）限制可回调域名。
4. 回调签名验签：
   - 回调体包含 `callback_attestation`（SFID 签名封装）。
   - Header 返回 `x-sfid-callback-signature`、`x-sfid-callback-key-id`。
   - 区块链通过 `/api/v1/attestor/public-key` 获取验签公钥。
5. 奖励闭环状态机：
   - 绑定成功初始化 `PENDING`；
   - 区块链回执 `FAILED` -> `RETRY_WAITING`（超重试上限转 `FAILED`）；
   - 区块链回执 `SUCCESS` -> `REWARDED`。

## 10. 安全与合规
- 管理员登录采用“公钥身份识别 + challenge 二维码签名验签”。
- `demo-sign` 测试入口已下线，所有登录测试与联调均使用真实钱包签名。
- 绑定与解绑必须管理员执行，非管理员不可执行。
- 建议绑定/解绑启用双人复核。
- CPMS 二维码必须验签，防伪造与重放。
- 区块链接口必须鉴权，禁止匿名公网调用。
- 区块链接口必须执行 `request_id + nonce + timestamp` 防重放，并落库幂等表。
- 生产必须配置 `SFID_CHAIN_SIGNING_SECRET`，并强制 `x-chain-signature` 请求签名校验。
- 公开查询接口允许匿名访问，但必须启用限流、IP 频控和访问日志审计。
- 登录 challenge 必须一次性、短时效，并绑定 `session` 上下文。
- 审计日志保留不少于 3 年。
- 状态变更扫码、绑定确认、解绑、机构公钥登记、链路资格查询、链路计数查询必须写入审计日志（操作者、公钥/档案号、结果、时间、request_id、actor_ip）。
- CORS 不允许全开放；应通过 `SFID_CORS_ALLOWED_ORIGINS` 显式配置前端来源（禁止 `*`）。

## 11. CPMS 二维码规范（v1 冻结）
### 11.1 公民档案二维码字段定义
- `ver`：二维码协议版本，固定 `1`。
- `issuer_id`：签发方标识，固定 `cpms`。
- `site_sfid`：SFID 下发的机构唯一号。
- `sign_key_id`：机构签名密钥标识（`K1/K2/K3`）。
- `archive_no`：档案号（全局唯一用户标识）。
- `issued_at`：签发时间（Unix 时间戳）。
- `expire_at`：过期时间（Unix 时间戳）。
- `qr_id`：二维码唯一流水号（防重放）。
- `sig_alg`：签名算法，固定 `sr25519`。
- `citizen_status`：用户状态，`NORMAL` 或 `ABNORMAL`。
- `voting_eligible`：是否具备投票资格（由 `citizen_status` 映射）。
- `signature`：CPMS 私钥对二维码原文签名后的结果值。

### 11.2 明确约束
- 二维码中不包含 CPMS 公钥本体。
- 每个 CPMS 机构实例必须先输入 `site_sfid` 再初始化生成本机构 3 套签名密钥。
- SFID 按 `site_sfid + sign_key_id` 维护该机构受信任公钥并用于验签。
- 当前版本是单向验签：CPMS 签、SFID 验；CPMS 不需要保存 SFID 公钥。
- `archive_no` 作为唯一用户标识，不再引入额外用户 ID 字段。
- `archive_no` 结构固定：`省2 + 市3 + 校验1 + 随机9 + 日期8(YYYYMMDD)`（日期为档案号创建时间）。
- `archive_no` 的省市代码来源与 CPMS 同步使用 `sheng_cities` 数据。
- `archive_no` 不承载年龄与状态语义，SFID 不得从 `archive_no` 推导投票资格。

## 12. WUMINAPP 扫码登录协议规范（统一口径）

### 12.1 目标状态
- 协议：`WUMIN_LOGIN_V1.0.0`。
- 责任边界：`wuminapp` 负责挑战解析、系统身份验签、手机签名与回执生成；SFID 独立完成回执验签、授权与登录结果展示。
- 信任来源：WuminApp 通过区块链 RPC 获取 SFID 当前公钥；CPMS 通过 SFID 背书建立信任，不直接依赖区块链。

### 12.2 挑战码（SFID -> 手机）
```json
{
  "proto": "WUMIN_LOGIN_V1.0.0",
  "system": "sfid",
  "request_id": "uuid",
  "challenge": "string",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000090,
  "sys_pubkey": "0x...",
  "sys_sig": "0x..."
}
```

### 12.3 签名原文（固定）
```text
proto|system|request_id|challenge|nonce|issued_at|expires_at
```

### 12.4 回执码（手机 -> SFID）
```json
{
  "proto": "WUMIN_LOGIN_V1.0.0",
  "system": "sfid",
  "request_id": "uuid",
  "pubkey": "0x...",
  "sig_alg": "sr25519",
  "signature": "0x...",
  "signed_at": 1760000020,
  "payload_hash": "0x..."
}
```

说明：`system` 标识回执来源系统（`sfid` 或 `cpms`），`payload_hash` 为签名原文的 SHA-256 哈希，用于防篡改校验。

### 12.5 SFID 验签顺序
1. 解析回执并读取 `request_id/pubkey/signature`。
2. 按 `request_id` 查挑战缓存，校验 `proto/system/request_id/challenge/nonce/issued_at/expires_at` 字段完整性与格式。
3. 校验系统固定为 `sfid`。
4. 校验挑战固定 `90` 秒时效：`expires_at - issued_at == 90` 且当前未过期。
5. 按固定拼串 `WUMIN_LOGIN_V1.0.0|system|request_id|challenge|nonce|expires_at` 重建用户签名原文并执行 `sr25519` 验签。
6. 校验 `request_id` 未消费后一次性消费，再做管理员授权判定（是管理员登录，不是管理员拒绝）。
7. 服务端接收回执时应兼容 `request_id|challenge_id`、`pubkey|admin_pubkey|public_key`、`signature|sig` 字段别名。
- `archive_no` 校验位算法与 SFID `sfid_code` 统一：`BLAKE2b` 摘要字节和 `mod 10`。
- 投票资格最终以 CPMS 二维码状态为准（`NORMAL` 可投票，`ABNORMAL` 不可投票）。

### 11.3 验签规则
1. 解析二维码并校验必填字段完整性。
2. 校验 `issuer_id=cpms`、`sig_alg=sr25519`。
3. 校验 `expire_at` 未过期、`qr_id` 未被消费。
4. 以 `site_sfid + sign_key_id` 定位机构公钥并验签 `signature`。
5. 验签通过后返回 `qr_id + archive_no + citizen_status`，仅允许进入绑定确认；失败直接拒绝。
6. 绑定确认必须提交 `qr_id`，且 `qr_id` 对应的 `archive_no` 必须与本次绑定档案号一致。

### 11.4 公钥登记二维码（机构初始化）
- 用途：CPMS 新安装机构向 SFID 登记本机构 3 把签名公钥。
- SFID 初始化二维码字段（由 SFID 生成）：`ver`, `issuer_id=sfid`, `purpose=cpms_init`, `site_sfid`, `a3=GFR`, `p1=0`, `province`, `city`, `institution`, `issued_at`, `expire_at=0`, `qr_id`, `sig_alg=sr25519`, `key_id`, `key_version`, `public_key`, `signature`。
- CPMS 公钥登记二维码字段（由 CPMS 生成）：`site_sfid`, `pubkey_1`, `pubkey_2`, `pubkey_3`, `issued_at`, `init_qr_payload`, `checksum_or_signature`。
- 流程：
1. SFID 机构管理员先在机构管理页生成机构身份识别码（`site_sfid`）和 SFID 签名初始化二维码，机构记录状态为 `PENDING`。
2. CPMS 使用该初始化二维码完成安装初始化并生成 3 把机构公钥登记二维码。
3. SFID 机构管理员扫码录入 CPMS 公钥登记二维码。
4. SFID 校验该登记二维码是否绑定了 SFID 侧签发的 `init_qr_payload`，并校验 `init_qr_payload` 签名可信。
5. 校验通过后机构状态由 `PENDING` 变为 `ACTIVE`，3 把公钥生效。
6. 录入完成前，SFID 不认可该机构出具的公民档案二维码与状态变更二维码。
7. 只有完成上述链路并处于 `ACTIVE` 的机构，后续出具的“公民档案号二维码/状态变更二维码”才被 SFID 视为可信输入。

### 11.5 CPMS 对齐清单（执行项）
- 初始化口径：CPMS 必须使用 SFID 签发的初始化二维码完成首装初始化，不得本地自生成机构号。
- 绑定校验口径：SFID 在机构录入时必须校验 CPMS 二维码绑定的 `init_qr_payload` 与 SFID 侧该 `site_sfid` 生成记录一致。
- 可信链路口径：只有“SFID 签发初始化二维码 -> CPMS 初始化 -> SFID 扫码录入机构公钥”闭环完成，CPMS 该机构二维码才进入受信集合。
- 机构密钥口径：每个 `site_sfid` 固定维护 3 把签名密钥；轮换前需先完成 SFID 信任库更新。
- 绑定二维码字段：`ver, issuer_id, site_sfid, sign_key_id, archive_no, citizen_status, voting_eligible, issued_at, expire_at, qr_id, sig_alg, signature`。
- 状态变更二维码字段：`ver, issuer_id, site_sfid, sign_key_id, archive_no, citizen_status, voting_eligible, issued_at, expire_at, qr_id, sig_alg, signature`。
- 状态值口径：`NORMAL` 可投票，`ABNORMAL` 不可投票；CPMS 输出状态即业务最终状态源。
- 签名原文口径：按 11.3/11.4 固定顺序拼接，严禁字段重排、空格填充、编码漂移。
- 重放口径：同一 `qr_id` 只能使用一次；CPMS 不得重复出具同 `qr_id` 的二维码。
- 失败语义对齐：SFID 返回“站点未登记/签名失败/二维码过期/二维码已消费”时，CPMS 按同义错误码落日志并触发补发流程。
- 拒绝语义口径：验签失败、机构未登记、机构非 `ACTIVE`、初始化链路不一致时，SFID 必须拒绝该 CPMS 公民档案号二维码/状态变更二维码。
- 联调顺序：先登记机构公钥二维码，再联调公民绑定二维码，最后联调状态变更二维码。

### 11.6 机构管理页面冻结口径（2026-03）
- 机构列表中“机构号”统一为“身份识别码”，展示 `site_sfid`。
- 生成身份识别码弹窗不输入公钥；弹窗内 `A3` 固定公法人（`GFR`）、`P1` 固定非盈利（`0`）。
- 机构管理员账号有省份约束时，省份默认并锁定；可选市与机构类型（机构类型受 `GFR` 约束）。
- 生成后主按钮显示“完成”，点击返回列表并展示该 `site_sfid`；次按钮为“下载”二维码，不显示 JSON 文本框。
- 列表每条身份识别码后显示小二维码按钮；点击弹出该 `site_sfid` 对应二维码并支持再次下载。
- 身份识别码二维码长期有效，不展示“有效期至”文案。
- 机构页操作按钮为“禁用、删除、扫码”；不显示“撤销”按钮与“扫码录入机构”顶栏按钮。
- 每个机构公钥列分别展示”更新”按钮；”登记人”列显示”`{省名}机构管理员`”。

## 12. 部署与运维
- SFID 为在线部署系统，管理员通过浏览器访问前端网站。
- 仅管理员可登录，普通用户无后台入口权限。
- 数据库建议单主高可用，所有写入进主库。
- 可按区域部署只读副本承担查询流量。
- 监控至少覆盖：接口可用性、错误率、签名耗时、数据库延迟。
- 本地开发启动最小条件：
1. PostgreSQL 可达（示例：`docker` 容器 `sfid-pg` 映射 `127.0.0.1:5432`）。
2. 后端环境变量设置 `DATABASE_URL`。
3. 后端监听地址 `127.0.0.1:8899` 未被其他进程占用。
- 安全相关关键环境变量：
1. `SFID_CHAIN_TOKEN`：区块链接口静态 Token。
2. `SFID_CHAIN_SIGNING_SECRET`：必填，链路请求强制验签。
3. `SFID_PUBLIC_SEARCH_TOKEN`：公开身份检索鉴权 Token。
4. `SFID_SIGNING_SEED_HEX`：后端主签名种子（必填）。
5. `SFID_KEY_ID`：签名 key id（必填）。
6. `SFID_RUNTIME_META_KEY`：运行态元数据加密密钥（必填）。
7. `SFID_BIND_CALLBACK_URL`：默认绑定回调地址。
8. `SFID_BIND_CALLBACK_AUTH_TOKEN`：回调 Bearer Token（仅后端环境变量配置）。
9. `SFID_CALLBACK_ALLOWED_HOSTS`：回调地址域名白名单（逗号分隔）。
10. `SFID_ALLOW_INSECURE_CALLBACK_HTTP`：仅开发联调放开 `http://` 回调。
11. `SFID_CORS_ALLOWED_ORIGINS`：CORS 来源白名单（逗号分隔，禁止 `*`）。
12. `SFID_PII_KEY`：仅部署脚本兼容保留，非后端启动强依赖。
13. `SFID_APP_TOKEN`：移动端（wuminapp）App API 鉴权 Token（与 `SFID_CHAIN_TOKEN` 独立，不可混用）。
- 常见故障排查：
1. 前端提示 `Failed to fetch` 且 `curl` 返回 `Empty reply from server`：优先检查后端进程是否 panic。
2. 日志出现 `AddrInUse`：说明 `8899` 端口被旧进程占用，先清理占用进程再启动。
3. 日志出现 `connect postgres failed`：检查 PostgreSQL 容器状态、端口映射和 `DATABASE_URL` 账号密码。

## 13. 测试与验收
### 13.1 功能验收
- 可完成“区块链传公钥 -> 管理员扫码绑定 -> 区块链收到成功结果”闭环。
- 可完成线下受理解绑 -> 管理员执行解绑 -> 区块链收到解绑结果闭环。
- 可投票人数统计接口可稳定返回。
- 绑定有效性校验接口返回准确。
- 机构管理员可完成系统管理员增删改查；系统管理员无该权限。
- 非管理员公钥扫码登录必须被拒绝（返回 403）。
- 机构管理员与系统管理员使用同一前端页面；机构管理员仅多一个”系统管理员管理”功能域。
- 公开查询需携带查询 Token，可查询档案号、身份识别码、公钥地址三项信息。
- 状态为 `ABNORMAL` 的用户在绑定有效时仍不可计入可投票人数，且资格校验返回不可投票。

### 13.2 安全验收
- 二维码签名校验必须生效。
- 越权解绑、伪造二维码、重放请求应被拦截。
- 登录 challenge 重放必须被拦截。
- 关键动作具备完整审计记录。
- 系统管理员访问管理员管理接口必须返回权限拒绝。

### 13.3 自动化回归（已落地）
- `backend/scripts/smoke.sh` 覆盖主流程 + 异常流：
  - 重复 nonce 拒绝；
  - 过期 timestamp 拒绝；
  - 投票资格 `NORMAL/ABNORMAL` 正反校验；
  - 奖励回执状态机（`FAILED -> RETRY_WAITING/FAILED -> SUCCESS -> REWARDED`）。
- 单测覆盖：缺失防重放头拒绝、重复 nonce 拒绝。

## 14. 当前待定项
- 可投票人数统计口径（实时/准实时、是否按地域筛选）。
- 双人复核是否默认强制。

### 14.1 SFID码生成工具（Rust）
#### 14.1.1 代码位置
- 工具主模块：`backend/src/sfid/mod.rs`
- 省定义：`backend/src/sfid/province.rs`
- 市级代码表：`backend/src/sfid/city_codes/01_ZS.rs` 至 `backend/src/sfid/city_codes/43_HJ.rs`

#### 14.1.2 生成输入
- `account_pubkey`（必填）
- `a3`（必填）：`GMR | ZRR | ZNR | GFR | SFR | FFR`
- `p1`（按 A3 规则可选/必填）
- `province`（必填）
- `city`（必填，必须属于所选省）
- `institution`（必填，按 A3 限制）

#### 14.1.3 A3/P1/机构规则（已落地）
- `GMR`（公民人）：机构固定 `ZG`（中国），`P1` 固定 `1`（盈利）。
- `ZRR`（自然人）：机构固定 `TG`（他国），`P1` 固定 `1`（盈利）。
- `ZNR`（智能人）：机构固定 `ZG`（中国），`P1` 可选 `0/1`。
- `GFR`（公法人）：机构仅允许 `ZF/LF/SF/JC/JY/CB`，`P1` 固定 `0`（非盈利）。
- `SFR`（私法人）：机构仅允许 `ZG/CH/TG`，`P1` 可选 `0/1`。
- `FFR`（非法人）：机构仅允许 `ZG/TG`，`P1` 可选 `0/1`。
- `GMR / ZRR / ZNR` 生成时，`R5` 的后三位固定为省级占位码 `000`，不再暴露真实市。
- 各省真实市码统一从 `001` 起排；`000` 仅保留给省级占位，不属于真实市。

#### 14.1.4 前端行为
- 点击“生成”时，前端调用后端接口，不在前端本地生成。
- A3 为单机构场景时，机构字段自动设值并锁定不可编辑。
- 省级管理员进入生成弹窗时，省字段默认本省且锁定；市字段必须从本省市列表选择。

## 15. 优化路线（优先级）
### 15.1 P0（上线前建议完成）
- 登录 challenge 可绑定 `domain/session_id/nonce/expires_at` 作为网页侧上下文；`aud` 不再属于扫码协议字段。
- 冻结二维码签名原文规范（字段顺序、编码、时间格式），避免 CPMS/SFID 联调歧义。
- 重放防护双层化：`qr_id` 一次性消费 + `login_challenges` 一次性消费与 TTL。
- 机构管理员保护：除”不可删改角色”外，增加最小可用数量保护（避免误操作锁死）。

### 15.2 P1（首版稳定后建议）
- 从“固定 3 把公钥任意验签”升级为 `key_id` 精确验签 + 公钥状态管理（`ACTIVE/REVOKED`）。
- 区块链接口防滥用升级：请求签名或 mTLS + timestamp/nonce 防重放 + 限流（`timestamp/nonce` 已落地）。
- 审计增强：记录操作者公钥、IP、请求 ID、结果码，日志仅追加不可改写（UA 与摘要哈希待补）。
- 绑定事务幂等化：并发下保持“同档案号/同公钥”一致失败语义和可追踪错误码（已落地：`binding_unique_locks`）。

### 15.3 P2（运维与结构优化）
- 可观测完善：增加扫码验签失败率、登录验签失败率、绑定成功率、链查询 P95、重放拦截次数（链路 P95/P99、重放拦截、链请求失败、回调失败已落地）。
- 工程结构已采用：`frontend`（前端）、`backend`（后端）、`deploy`；数据库与脚本统一归档在 `backend/db`、`backend/scripts`，测试归档在 `backend/tests`。角色专属逻辑归档在 `backend/src/key-admins`、`backend/src/super-admins`、`backend/src/operator-admins`，共用后台查询与审计归档在 `backend/src/business`，操作业务归档在 `backend/src/operate`，区块链业务归档在 `backend/src/chain`，SFID 生成与元数据归档在 `backend/src/sfid`，统一数据结构归档在 `backend/src/models`。

## 16. 开发步骤（执行版）
### 16.1 里程碑 0：规格冻结（0.5 天）
- 冻结 `archive_no` 解析规则：字段位置、日期格式、异常处理策略。
- 冻结 API 字段、错误码、通用返回结构（`code/message/data|trace_id`）。
- 冻结权限矩阵：`INSTITUTION_ADMIN` 与 `SYSTEM_ADMIN` 的接口边界。
- 交付物：全部冻结规格统一写入本技术文档（不再拆分独立规格文档）。
- 验收标准：前后端、测试、区块链对接口与字段无歧义。

### 16.2 里程碑 1：数据层与系统初始化（2-3 天）
- 完成核心表与索引：`runtime_cache_entries`、`runtime_meta`、`admins`、`key_admin_keyring`、`chain_idempotency_requests`、`binding_unique_locks`、`bind_reward_states`。
- 完成管理员与密钥分表：`admins`、`provinces`、`super_admin_scope`、`operator_admin_scope`、`key_admin_keyring`。
- 完成运行态拆分：`runtime_misc`、`runtime_meta`，并下线 `runtime_store`。
- 运行态缓存升级：`runtime_cache_entries`（按键分片存储），`runtime_misc` 仅保留兼容快照。
- 完成运行态加密落地：`runtime_meta.payload_enc`。
- 交付物：`backend/db/migrations`、初始化器代码、数据字典。
- 当前落地迁移：
  - `backend/db/migrations/001_init_sfid.sql`
  - `backend/db/migrations/002_runtime_store.sql`
  - `backend/db/migrations/003_admin_role_partition.sql`
  - `backend/db/migrations/004_finalize_no_runtime_store.sql`
  - `backend/db/migrations/005_drop_sfid_prefix.sql`
  - `backend/db/migrations/006_super_admin_catalog.sql`
  - `backend/db/migrations/007_refresh_admin_views.sql`
  - `backend/db/migrations/008_chain_idempotency_reward_state.sql`
  - `backend/db/migrations/009_runtime_cache_and_pii_encryption.sql`
  - `backend/db/migrations/010_drop_plaintext_pii_columns.sql`
- 验收标准：重复执行迁移可幂等，视图和约束稳定可查询。

### 16.3 里程碑 2：管理员认证与 RBAC（2-3 天）
- 实现认证链路：`identify -> challenge -> verify`。
- challenge 绑定 `domain/session_id/nonce/expires_at`，一次性消费与 TTL 过期清理。
- 落地后端 RBAC 中间件，对每个管理接口进行角色强校验。
- 实现系统管理员管理接口（仅机构管理员可访问）。
- 交付物：认证 API、会话/JWT、权限中间件、管理员管理 API。
- 验收标准：非管理员公钥拒绝登录；系统管理员访问管理员管理接口返回 403。

### 16.4 里程碑 3：绑定/解绑主流程（3 天）
- 完成 CPMS 机构公钥登记流程：扫码录入 `site_sfid + keys[key_id,pubkey]`。
- 完成公民二维码验签：`issuer_id`、`sig_alg`、`expire_at`、`qr_id`、3 把公钥任意一把验签通过。
- 实现绑定确认：写入 `archive_no + account_pubkey`，状态置 `ACTIVE`，并发幂等。
- 实现解绑流程：管理员填写原因，状态置 `UNBOUND`，完整审计。
- 交付物：绑定/解绑 API、验签模块、审计日志落库。
- 验收标准：伪造二维码与重放请求被拦截；绑定与解绑闭环可复现。

### 16.5 里程碑 4：资格判定与区块链接口（2 天）
- 接入 CPMS 二维码状态（`NORMAL`/`ABNORMAL`）作为投票资格判定源。
- 实现 `/api/v1/chain/voters/count` 与 `/api/v1/chain/binding/validate`。
- 增加区块链调用方鉴权（token 或签名）与防重放基础能力。
- 交付物：链路自动接口、鉴权中间件、资格判定单测。
- 验收标准：`ABNORMAL` 用户不计入可投票人数，校验接口返回一致。

### 16.6 里程碑 5：公开查询与防滥用（1 天）
- 实现公开查询接口：按 `archive_no`/`identity_code`/`account_pubkey` 检索。
- 返回统一字段：`archive_no`、`identity_code`、`account_pubkey`（可选扩展资格字段）。
- 启用匿名访问限流、IP 频控、访问日志审计。
- 交付物：公开查询 API、限流策略、访问日志面板基础指标。
- 验收标准：匿名可查询，异常频率请求可拦截且留痕。

### 16.7 里程碑 6：前端收口（3-4 天）
- 管理端统一页面：登录、绑定、解绑、查询、管理员管理。
- 同一套前端支持两类管理员；机构管理员额外显示管理员管理模块。
- 完善错误态、空态、加载态与操作反馈。
- 交付物：`frontend` 联调版本、接口错误映射与提示规范。
- 验收标准：核心流程均可前端闭环完成，无阻断级 UI 缺陷。

### 16.8 里程碑 7：联调、测试与上线（4-5 天）
- 功能联调：认证、绑定、解绑、区块链接口、公开查询。
- 安全测试：越权、伪造二维码、重放、匿名滥用。
- 上线准备：部署配置、监控告警、回滚预案、值班手册。
- 发布策略：灰度 -> 指标观察 -> 全量切换。
- 交付物：测试报告、上线清单、回滚演练记录。
- 验收标准：P0/P1 缺陷清零后上线，关键指标达标。

### 16.9 推荐排期（3 周）
- 第 1 周：完成里程碑 0-2。
- 第 2 周：完成里程碑 3-4。
- 第 3 周：完成里程碑 5-7，灰度上线并进入稳定性观察。

### 16.10 稳定性观察（上线后 1 周）
- 重点监控：扫码验签失败率、登录验签失败率、绑定成功率、链路接口 P95、错误率。
- 对异常流量与资格误判进行快速修正并追加回归测试。

## 17. 变更控制
- 本文档作为当前冻结基线。
- 新需求必须通过 CR 流程更新本文档后再开发。
