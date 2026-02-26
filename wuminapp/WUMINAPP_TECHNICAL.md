# wuminapp 技术文档（结合当前链实现）

## 1. 目标与边界

- `wuminapp` 是 GMB 体系的轻节点应用（移动端为主），面向公民/访客用户。
- 职责：钱包、SFID 绑定、公民投票、交易入口、轻社交能力（分期）。
- 边界：
  - 区块链共识与 Runtime 逻辑在 `citizenchain/`。
  - 机构治理与全节点运维不在 `wuminapp` 内实现。
  - `wuminapp/backend` 负责移动端安全代理、链交互聚合、风控与审计。

## 2. 与当前区块链实现的对齐约束

### 2.1 链侧已实现能力（wuminapp 必须对齐）

- SFID 绑定与验证：`citizenchain/otherpallet/sfid-code-auth`。
  - 绑定入口：`bind_sfid`。
  - 一人一票基础：`SfidToAccount` / `AccountToSfid` / `BoundCount`。
  - 防重放：`UsedCredentialNonce`、`UsedVoteNonce`。
  - 投票验签接口：`SfidVoteVerifier`。
- 公民轻节点认证发行：`citizenchain/issuance/citizen-lightnode-issuance`。
  - 绑定成功触发 `OnSfidBound` 回调发奖。
  - 奖励规则常量来自 `primitives/src/citizen_const.rs`：
    - 前 `14,436,417` 个：`9999.00` 元（`999900` 分）
    - 后续：`999.00` 元（`99900` 分）
    - 总量上限：`CITIZEN_LIGHTNODE_MAX_COUNT`
    - 同一 SFID 仅一次奖励：`CITIZEN_LIGHTNODE_ONE_TIME_ONLY=true`
- Runtime 验签规则：`citizenchain/runtime/src/configs/mod.rs`。
  - SFID 绑定签名消息域：`GMB_SFID_BIND_V1`
  - 公民投票签名消息域：`GMB_SFID_VOTE_V1`
  - 算法：`sr25519`，签名长度 64 字节。

### 2.2 业务制度需求（来自当前仓库 README）

- SFID 与地址一对一绑定后，用户从“访客轻节点”升级为“公民轻节点”。
- 仅完成 SFID 绑定的轻节点可参与公民投票。
- 公民轻节点认证发行遵循两阶段奖励与总量上限。

## 3. wuminapp 当前实现现状（代码事实）

### 3.1 mobile（Flutter）

- 技术栈：Flutter + Dart（`wuminapp/mobile/pubspec.yaml`）。
- 已实现：
  - 4 Tab 框架：首页/投票/交易/我的（投票和交易仍为“开发中”页面）。
  - 后端健康检查：`GET /api/v1/health`（`api_client.dart`）。
  - 本地钱包创建/导入：`sr25519 + SS58(2027)`（`wallet_service.dart`）。
  - 多钱包本地管理（SharedPreferences）。
  - 扫码：
    - FCRC 登录挑战识别与签名（`fcrc_login_service.dart` + `qr_scan_page.dart`）。
    - 收款码识别并生成转账草稿页（UI 草稿）。
  - 公钥到机构角色映射能力（`wallet_type_service.dart`，本地静态映射表）。
- 未完成/占位：
  - 链上 `bind_sfid` 真实交易构造与提交。
  - 公民投票交易签名与提交。
  - 转账交易签名、广播、回执追踪。
  - 本地密钥安全区存储（当前助记词明文存于 SharedPreferences）。

### 3.2 backend（Rust/Axum）

- 技术栈：Axum + Tokio + Serde + Tracing。
- 已实现：
  - `127.0.0.1:8787` 本地监听。
  - 根路由与健康检查：`/`、`/api/v1/health`。
  - 统一响应包裹：`{ code, message, data }`。
- 未完成：
  - SFID 绑定 API、投票 API、钱包余额/交易 API。
  - 与 citizenchain RPC 的连接层与交易回执订阅。
  - 鉴权、限流、审计追踪、错误码体系落地。

## 4. 目标架构（建议落地）

```text
wuminapp/
├── mobile/                        # Flutter 客户端
│   ├── lib/services/              # API + 签名 + 本地安全封装
│   └── lib/pages/                 # 钱包/绑定/投票/交易 UI
├── backend/                       # Rust 网关与业务编排
│   ├── src/routes/                # /api/v1/*
│   ├── src/services/              # 链交互/SFID/投票/交易服务
│   └── src/models/                # DTO 与错误模型
└── WUMINAPP_TECHNICAL.md          # 本文档
```

- 客户端负责“持钥与签名”（私钥不出端）。
- 后端负责“鉴权、策略、聚合链访问、审计日志、抗重放校验补强”。
- 链上状态为最终权威，后端仅做缓存与查询聚合。

## 5. 关键流程设计（对齐链）

### 5.1 SFID 绑定流程

1. 客户端请求后端获取绑定挑战（nonce + trace_id）。
2. 客户端使用本地 `sr25519` 对链规则要求的 payload 签名。
3. 后端校验请求结构后提交链上 `bind_sfid`。
4. 链上执行：
   - 验签通过 -> 建立 `SFID <-> Account` 映射
   - 消耗 nonce 防重放
   - 触发 `OnSfidBound`（可能发放认证奖励）
5. 后端回传交易哈希、区块高度、奖励结果（如有）。

### 5.2 公民投票流程

1. 客户端拉取提案与投票资格摘要。
2. 客户端构造 `GMB_SFID_VOTE_V1` 对应签名消息并签名。
3. 后端提交投票交易。
4. 链上依据 `SfidVoteVerifier + UsedVoteNonce` 完成验签与防重放。

### 5.3 资产与交易流程

1. 客户端本地签名交易（sr25519）。
2. 后端执行广播与状态订阅（入池/上链/失败原因）。
3. 后端返回统一状态机给移动端（pending/confirmed/failed）。

## 6. API 规划（在现有 `/api/v1` 基础上扩展）

- 已有：`GET /api/v1/health`
- 下一步最小闭环：
  - `POST /api/v1/auth/sfid/challenge`
  - `POST /api/v1/auth/sfid/bind`
  - `GET /api/v1/wallet/balance?account=...`
  - `POST /api/v1/tx/submit`
  - `GET /api/v1/vote/proposals`
  - `POST /api/v1/vote/cast`

统一响应：
- 成功：`{ code: 0, message: "ok", data: ... }`
- 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`

## 7. 安全基线（按优先级）

- P0：钱包助记词迁移到系统安全区（iOS Keychain / Android Keystore），禁止明文持久化。
- P0：所有 SFID/投票相关请求必须携带 nonce 与过期时间，后端做二次时效校验。
- P0：后端落地 `trace_id + account + sfid_hash + tx_hash + result` 审计日志。
- P1：签名前人机确认（金额、收款方、提案编号、链 ID）。
- P1：设备完整性校验接入（当前 `attestation_service.dart` 为占位实现）。

## 8. 迭代里程碑（建议）

- M1（链路打通）：
  - 完成 SFID 绑定真实上链
  - 完成余额查询与基础转账提交
- M2（治理能力）：
  - 完成公民投票提案列表与投票上链
  - 完成投票回执与状态同步
- M3（体验与安全）：
  - 完成安全区密钥存储迁移
  - 完成通知、审计看板与异常告警

## 9. 开发与联调

### 9.1 backend

```bash
cd /Users/rhett/GMB/wuminapp/backend
cargo run
```

默认监听：`http://127.0.0.1:8787`

### 9.2 mobile

```bash
cd /Users/rhett/GMB/wuminapp/mobile
flutter pub get
flutter run
```

- Android 模拟器默认访问后端：`http://10.0.2.2:8787`
- iOS/桌面调试默认访问后端：`http://127.0.0.1:8787`

## 10. 当前结论

- `wuminapp` 已完成轻节点最小骨架（钱包本地能力 + 扫码登录签名 + 后端健康探针）。
- 区块链侧 SFID 绑定、认证发行、防重放与投票验签规则已在 `citizenchain` 落地。
- 下一阶段重点不是“再写新规则”，而是把移动端/后端按现有链规则完整接通并补齐安全基线。
