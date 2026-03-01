# WUMINAPP 技术总文档（整合版）

## 1. 目标与边界

- `wuminapp` 是 GMB 体系的轻节点应用（移动端为主），面向公民/访客用户。
- 核心职责：钱包、SFID 绑定、公民投票、交易入口、轻社交能力（分期）。
- 工程边界：
  - 区块链共识与 Runtime 逻辑在 `citizenchain/`。
  - 机构治理与全节点运维不在 `wuminapp` 内实现。
  - `wuminapp/backend` 负责移动端安全代理、链交互聚合、风控与审计。

## 2. 产品定位

- 产品名称：`wuminapp`
- 目标用户：访客轻节点、公民轻节点
- 核心业务域：钱包、转账、SFID 绑定、公民投票、隐私通信、动态发布
- 非目标范围：全节点运维、委员会后台审批

## 3. 与当前区块链实现对齐约束

### 3.1 链侧已实现能力（wuminapp 必须对齐）

- SFID 绑定与验证：`citizenchain/otherpallet/sfid-code-auth`
  - 绑定入口：`bind_sfid`
  - 一人一票基础：`SfidToAccount` / `AccountToSfid` / `BoundCount`
  - 防重放：`UsedCredentialNonce`、`UsedVoteNonce`
  - 投票验签接口：`SfidVoteVerifier`
- 公民轻节点认证发行：`citizenchain/issuance/citizen-lightnode-issuance`
  - 绑定成功触发 `OnSfidBound` 回调发奖
  - 奖励规则常量来自 `primitives/src/citizen_const.rs`：
    - 前 `14,436,417` 个：`9999.00` 元（`999900` 分）
    - 后续：`999.00` 元（`99900` 分）
    - 总量上限：`CITIZEN_LIGHTNODE_MAX_COUNT`
    - 同一 SFID 仅一次奖励：`CITIZEN_LIGHTNODE_ONE_TIME_ONLY=true`
- Runtime 验签规则：`citizenchain/runtime/src/configs/mod.rs`
  - SFID 绑定签名消息域：`GMB_SFID_BIND_V1`
  - 公民投票签名消息域：`GMB_SFID_VOTE_V1`
  - 算法：`sr25519`，签名长度 64 字节

### 3.2 制度需求（仓库约束）

- SFID 与地址一对一绑定后，用户从“访客轻节点”升级为“公民轻节点”。
- 仅完成 SFID 绑定的轻节点可参与公民投票。
- 公民轻节点认证发行遵循两阶段奖励与总量上限。

## 4. 技术栈与架构分层

### 4.1 技术栈

- 移动端：`Flutter + Dart`（一套代码覆盖 iOS/Android）
- 链接入：Substrate API（推荐 `PAPI` / `Dedot` 的服务端适配层）
- 后端：`Rust`（HTTP/WebSocket）
- 通信：`Matrix`（E2EE）+ `WebRTC`（语音/视频）
- 存储：
  - 本地：`SQLite`/`Hive`
  - 服务端：`PostgreSQL`
- 推送：APNs（iOS）+ FCM（Android）

### 4.2 架构分层

- UI 层：Flutter 页面与组件
- 应用层：状态管理、路由、用例编排
- 服务层：链交互、认证、消息、推送、文件
- 领域层：账户、交易、投票、社交模型
- 基础设施层：本地数据库、缓存、日志、网络

### 4.3 目录建议

```text
wuminapp/
├── backend/
│   ├── src/
│   ├── tests/
│   └── migrations/
├── mobile/
│   ├── pubspec.yaml
│   ├── lib/
│   │   ├── main.dart
│   │   ├── pages/
│   │   ├── widgets/
│   │   ├── services/
│   │   └── utils/
│   ├── assets/
│   ├── test/
│   ├── ios/
│   └── android/
└── docs/
    ├── TECH-BASELINE.md
    ├── API-SPEC.md
    └── RELEASE.md
```

## 5. 当前实现现状（代码事实）

### 5.1 mobile（Flutter）

- 已实现：
  - 4 Tab 框架：首页/投票/交易/我的（投票和交易仍为“开发中”页面）
  - 后端健康检查：`GET /api/v1/health`（`api_client.dart`）
  - 本地钱包创建/导入：`sr25519 + SS58(2027)`（`wallet_service.dart`）
  - 多钱包本地管理（SharedPreferences）
  - 扫码登录挑战识别与签名（`fcrc_login_service.dart` + `qr_scan_page.dart`）
  - 收款码识别并生成转账草稿页（UI 草稿）
  - 公钥到机构角色映射能力（`wallet_type_service.dart`，本地静态映射表）
- 未完成：
  - 链上 `bind_sfid` 真实交易构造与提交
  - 公民投票交易签名与提交
  - 转账交易签名、广播、回执追踪
  - 本地密钥安全区存储（当前助记词明文存于 SharedPreferences）

### 5.2 backend（Rust/Axum）

- 已实现：
  - `127.0.0.1:8787` 本地监听
  - 根路由与健康检查：`/`、`/api/v1/health`
  - 统一响应包裹：`{ code, message, data }`
- 未完成：
  - SFID 绑定 API、投票 API、钱包余额/交易 API
  - 与 citizenchain RPC 的连接层与交易回执订阅
  - 鉴权、限流、审计追踪、错误码体系落地

## 6. 关键流程设计

### 6.1 SFID 绑定流程

1. 客户端请求后端获取绑定挑战（nonce + trace_id）。
2. 客户端使用本地 `sr25519` 对链规则要求的 payload 签名。
3. 后端校验请求结构后提交链上 `bind_sfid`。
4. 链上执行：
   - 验签通过，建立 `SFID <-> Account` 映射
   - 消耗 nonce 防重放
   - 触发 `OnSfidBound`（可能发放认证奖励）
5. 后端回传交易哈希、区块高度、奖励结果（如有）。

### 6.2 公民投票流程

1. 客户端拉取提案与投票资格摘要。
2. 客户端构造 `GMB_SFID_VOTE_V1` 对应签名消息并签名。
3. 后端提交投票交易。
4. 链上依据 `SfidVoteVerifier + UsedVoteNonce` 完成验签与防重放。

### 6.3 资产与交易流程

1. 客户端本地签名交易（`sr25519`）。
2. 后端执行广播与状态订阅（入池/上链/失败原因）。
3. 后端返回统一状态机给移动端（`pending/confirmed/failed`）。

### 6.4 统一离线双向扫码登录流程（`WUMINAPP_LOGIN_V1`）

- 登录模式统一为“离线双向扫码”：
  - 第一次扫码：`wuminapp` 扫描系统挑战二维码。
  - 第二次扫码：系统扫描 `wuminapp` 展示的签名回执二维码。
- 私钥仅在手机端本地使用，离线签名，私钥不出端。
- 三个系统统一认证方式，授权策略分离：
  - `cpms`：仅允许本地 RBAC 账户（3 个超级管理员 + 超级管理员新增的 n 个操作管理员）。
  - `sfid`：仅允许内置 45 个超级管理员及其新增操作管理员。
  - `citizenchain`：内置管理员账户进入对应界面；其他账户默认进入“全节点”界面。

离线登录时序：

1. 登录端（PC/前端软件）生成挑战并展示二维码（挑战码）。
2. `wuminapp` 扫码后校验协议、时效、系统标识与本地白名单。
3. 用户在手机端确认登录信息（系统名、设备标识、账户地址）。
4. `wuminapp` 用本地账户私钥签名并生成回执二维码（签名结果）。
5. 登录端扫描回执二维码，完成验签与授权判定。
6. 通过则进入对应界面，失败则给出拒绝原因并记录审计日志。

### 6.5 三端统一设计（手机端统一 + 系统端核心统一）

- 手机端统一（`wuminapp`）：
  - 三端登录统一使用 `mobile/lib/login/` 模块。
  - 统一协议解析、字段校验、白名单校验、离线签名、回执二维码生成、防重放。
  - 不为 `cpms/sfid/citizenchain` 分叉实现三套扫码代码。
- 系统端核心统一（`shared/wumin_login_core`）：
  - 统一挑战码生成、回执解析、签名原文拼接、`sr25519` 验签、过期校验、`request_id` 一次性消费。
  - 输入输出结构与错误码统一，避免三端实现差异。
- 授权层三端分离（Adapter）：
  - `cpms_login_adapter`：本地 RBAC（3 超管 + 操作员）判定。
  - `sfid_login_adapter`：内置 45 超管 + 操作员名单判定。
  - `citizenchain_login_adapter`：三类内置管理员角色映射；其余用户进入全节点界面。

## 7. API 规范（MVP + 扩展）

### 7.1 基础约定

- Base URL：`http://<host>:8787`
- Prefix：`/api/v1`
- 响应结构：
  - 成功：`{ code: 0, message: "ok", data: ... }`
  - 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`

### 7.2 已有接口

- `GET /api/v1/health`

示例：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "service": "wuminapp-backend",
    "version": "0.0.1",
    "status": "UP"
  }
}
```

### 7.3 占位接口（待实现）

- `POST /api/v1/auth/sfid/bind`
  - body 草案：`account`、`sfid_code`、`credential_nonce`、`signature`
- `GET /api/v1/wallet/balance?account=<address>`
- `POST /api/v1/tx/create`
- `GET /api/v1/tx/history?account=<address>&page=1&page_size=20`

### 7.4 下一步最小闭环建议

- `POST /api/v1/auth/sfid/challenge`
- `POST /api/v1/auth/sfid/bind`
- `GET /api/v1/wallet/balance?account=...`
- `POST /api/v1/tx/submit`
- `GET /api/v1/vote/proposals`
- `POST /api/v1/vote/cast`

### 7.5 错误码规划

- `1xxx` 参数/校验
- `2xxx` 身份/权限
- `3xxx` 业务规则
- `5xxx` 系统依赖

### 7.6 离线扫码登录协议规范（`WUMINAPP_LOGIN_V1`）

挑战二维码（系统 -> 手机）：

```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "cpms|sfid|citizenchain",
  "request_id": "uuid",
  "challenge": "base64-32bytes",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000060,
  "aud": "local-app-id",
  "origin": "local-device-id"
}
```

签名原文（固定串联顺序）：

```text
WUMINAPP_LOGIN_V1|system|aud|origin|request_id|challenge|nonce|expires_at
```

回执二维码（手机 -> 系统）：

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

登录错误码建议：

- `1101`：二维码协议头无效（`proto` 不匹配）
- `1102`：二维码已过期
- `1103`：挑战重复使用（`request_id` 已消费）
- `1201`：签名验签失败
- `1202`：账户与公钥不一致
- `2201`：账户不在 `cpms` 授权名单
- `2202`：账户不在 `sfid` 授权名单
- `2203`：`citizenchain` 角色判定失败

## 8. 区块链与通信集成原则

- 链上：资产、认证结果、投票、关键审计锚点。
- 链下：聊天消息、动态正文、媒体文件、推荐流。
- 私钥不出端：签名默认在移动端完成。
- 交易流程：本地构造与签名 -> 广播 -> 事件订阅 -> 状态回执。
- 私聊/群聊：Matrix E2EE（Olm/Megolm）。
- 语音/视频：WebRTC，采用端到端媒体加密。
- 动态发布：正文与媒体走链下存储，哈希与签名可选锚定上链。
- 内容审核：服务端做策略过滤，不触碰用户私钥。

## 9. 安全基线

- P0：助记词/私钥仅存系统安全区（iOS Keychain / Android Keystore）。
- P0：所有 SFID/投票相关请求必须携带 nonce 与过期时间，后端做二次时效校验。
- P0：后端落地 `trace_id + account + sfid_hash + tx_hash + result` 审计日志。
- P0：离线登录挑战 `request_id` 必须一次性消费，禁止复用。
- P0：离线登录挑战有效期建议 `60s`，超时必须拒绝。
- P0：`wuminapp` 必须校验 `system/aud/origin` 白名单，不可信来源禁止签名。
- P1：签名前人机确认（金额、收款方、提案编号、链 ID）。
- P1：设备完整性校验接入（当前 `attestation_service.dart` 为占位实现）。
- 敏感操作建议启用二次确认（生物识别或本地 PIN）。

## 10. 工程质量与发布

### 10.1 质量基线

- Flutter：`dart format`、`flutter analyze`、`flutter test`
- Rust：`cargo fmt`、`cargo clippy`、`cargo test`
- 提交门禁：格式化 + 静态检查 + 单元测试

### 10.2 发布与环境

- 平台：iOS、Android
- 制品：IPA、AAB/APK
- 环境：`dev` / `staging` / `prod`
- 最小配置：
  - `CHAIN_RPC_URL`
  - `MATRIX_HOMESERVER_URL`
  - `BACKEND_BASE_URL`
  - `PUSH_PROVIDER_CONFIG`
  - `LOG_LEVEL`

## 11. 分期里程碑

- M1：钱包、链上转账、SFID 绑定、交易记录
- M2：公民投票、消息通知、多账户
- M3：E2EE 聊天、语音视频、动态发布

补充的工程落地里程碑：
- M1（链路打通）：完成 `WUMINAPP_LOGIN_V1` 离线双向扫码登录、SFID 绑定真实上链、余额查询与基础转账提交
- M2（治理能力）：完成提案列表与投票上链、投票回执与状态同步
- M3（体验与安全）：完成安全区密钥迁移、通知/审计看板/异常告警

## 12. 开发与联调

### 12.1 Prerequisites

- 安装 Flutter SDK（stable）
- 执行 `flutter doctor` 并完成 iOS/Android 工具链检查

### 12.2 Quick Start

```bash
cd /Users/rhett/GMB/wuminapp/backend
cargo run

cd /Users/rhett/GMB/wuminapp/mobile
flutter pub get
flutter run
```

- Android 模拟器访问后端：`http://10.0.2.2:8787`
- iOS/桌面调试访问后端：`http://127.0.0.1:8787`

## 13. iOS 启动图资产说明

- 目录：`wuminapp/mobile/ios/Runner/Assets.xcassets/LaunchImage.imageset/`
- 可通过替换该目录图片文件来自定义 launch screen。
- 也可在 Xcode 打开 `ios/Runner.xcworkspace` 后，在 `Runner/Assets.xcassets` 拖拽替换图片。

## 14. 实施检查清单

- [ ] Flutter 工程骨架已初始化
- [ ] 链交互服务已完成抽象
- [ ] 私钥与签名安全策略已落地
- [ ] Matrix + WebRTC 链路已联调
- [ ] API 规范与错误码已统一
- [ ] iOS/Android CI 构建可用

## 15. 当前结论

- `wuminapp` 已完成轻节点最小骨架（钱包本地能力 + 扫码登录签名 + 后端健康探针）。
- 区块链侧 SFID 绑定、认证发行、防重放与投票验签规则已在 `citizenchain` 落地。
- 下一阶段重点是按现有链规则打通移动端/后端全链路并补齐安全基线。

## 16. 扫码登录实现态对齐（以本节为准）

### 16.1 当前已实现范围

- 已实现完整离线流程：`扫码挑战 -> 用户确认 -> 本地签名 -> 展示回执二维码 -> 系统端扫码验签 -> 登录结果`。
- 协议固定：`WUMINAPP_LOGIN_V1`。
- 算法固定：`sr25519`。
- 当前联调状态：`sfid` 已实测登录成功；`cpms/citizenchain` 按同一协议与核心逻辑对齐。
- 登录边界固定：`wuminapp` 仅负责挑战签名与展示签名二维码，不接收、不轮询、不回传登录结果状态。

### 16.2 挑战二维码字段（系统 -> 手机）

```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "sfid|cpms|citizenchain",
  "request_id": "uuid",
  "challenge": "string",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000060,
  "aud": "local-app-id",
  "origin": "local-device-id"
}
```

### 16.3 手机端签名原文（固定顺序）

```text
WUMINAPP_LOGIN_V1|system|aud|origin|request_id|challenge|nonce|expires_at
```

### 16.4 回执二维码字段（手机 -> 系统）

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

### 16.5 手机端校验与交互（已实现）

- 协议校验：`proto` 必须为 `WUMINAPP_LOGIN_V1`。
- 系统校验：`system` 仅允许 `cpms/sfid/citizenchain`。
- 时效校验：`expires_at` 过期直接拒绝签名。
- 白名单校验：对 `system/aud/origin` 执行本地白名单策略。
- 防重放：`request_id` 本地一次性消费，已消费请求拒绝再次签名。
- 人机确认：扫码后必须点击“本地签名并生成回执”，不自动签名。

### 16.6 系统端验签对齐要求

- 按 16.3 的固定原文重建消息后做 `sr25519` 验签。
- 验签公钥优先使用 `signer_pubkey`（如有），否则使用 `account/admin_pubkey`。
- `request_id` 必须一次性消费，重复提交直接拒绝。
- 校验 `system/aud/origin` 与本系统配置一致。
- 验签通过后再做本系统授权判定（管理员名单/角色映射）。
- 登录成功/失败提示只在系统端展示，手机签名端不做结果回执链路。

### 16.7 模块完成清单（wuminapp 侧）

- 已完成：扫码解析、协议校验、白名单校验、防重放、本地签名、签名二维码展示。
- 已完成：签名前用户确认、可选生物识别开关、开启时生物识别验证。
- 已完成：钱包助记词从本地偏好存储迁移到系统安全存储（Keychain/Keystore）。
- 已完成：登录白名单设置页（按 `system` 管理 `aud/origin`，支持重置默认）。
- 已完成：错误码化异常（含协议错误、白名单拒绝、过期、重放、生物识别失败、钱包缺失）。

### 16.8 已知限制（当前版本）

- 白名单仍是本地配置，尚未接入多端统一下发与签名更新机制。
- 仅支持离线双向扫码登录，不提供手机端登录结果回传。
