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
- M1（链路打通）：完成 SFID 绑定真实上链、余额查询与基础转账提交
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
