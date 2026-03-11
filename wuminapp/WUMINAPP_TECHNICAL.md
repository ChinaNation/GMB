# WUMINAPP 技术总文档（当前实现态）

## 1. 项目定位

`wuminapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- SFID 与链交互由外部服务系统承载
- `wuminapp` 负责端上钱包、登录签名、链上交易入口、绑定指令发起、状态展示

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 外部接口：HTTP API（由 SFID/网关系统提供）

## 3. 当前目录结构

```text
wuminapp/
├── android/
├── ios/
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── governance/
│   ├── login/
│   ├── signer/
│   ├── user/
│   ├── wallet/
│   └── trade/
├── test/
└── WUMINAPP_TECHNICAL.md
```

说明：

- 原 `mobile/` 内容已上移到项目根目录
- 原 `backend/` 已移除

## 4. App 当前实现

### 4.1 主导航

底部 5 Tab：

- `广场`
- `治理`
- `消息`
- `金融`
- `我的`

### 4.2 治理页

- 机构分类卡片已内置：
  - 国储会 1
  - 省储会 43
  - 省储行 43
- 提案/投票链上交互仍在治理模块开发阶段（规范已落文档）

### 4.3 金融页

- 入口页标题为“金融”
- 已接入“链上交易”页面
- 链下交易仍为开发中占位

### 4.4 钱包与签名

钱包能力收口在 `lib/wallet/`：

- `core/`：钱包生命周期、Isar、机密 key 规范、生物识别守卫
- `capabilities/`：登录签名编排、余额/API、管理员目录、绑定、证明态
- `ui/`：钱包页面

签名能力收口在 `lib/signer/`：

- `local_signer.dart`：手机本机签名（助记词在手机）
- `qr_signer.dart`：扫码签名协议（私钥在外部设备）

签名算法：`sr25519`。

调用点：

- 登录扫码签名前
- 链上交易签名前

签名前守卫：`UserIdentificationService.confirmBeforeSign()`。

### 4.5 登录模块

登录模块在 `lib/login/`，负责：

- 扫码识别挑战码
- 协议校验
- `aud` 白名单校验
- 防重放（`request_id`）
- 展示回执二维码

关键口径：

- 协议：`WUMINAPP_LOGIN_V1`
- 当前系统白名单：`cpms`、`sfid`
- 签名串：

```text
WUMINAPP_LOGIN_V1|system|aud|request_id|challenge|nonce|expires_at
```

### 4.6 双签名模式（技术方案）

- 模式 A：本机签名
  - 私钥/助记词仅保存在手机 secure storage
  - 交易和登录均由 `LocalSigner` 在手机完成签名
- 模式 B：扫码签名
  - 手机不保存私钥，仅保存钱包地址/公钥
  - 手机生成待签名请求二维码，外部设备签名后返回签名回执二维码
  - 协议由 `QrSigner` 统一编解码与校验（`WUMINAPP_QR_SIGN_V1`）

## 5. 手机端三层存储（当前）

### 5.1 机密层（Secure Storage）

仅存高敏感数据：

- `wallet.secret.<wallet_id>.mnemonic.v1`
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

钱包域核心集合：

- `WalletProfileEntity`
- `WalletSettingsEntity`
- `TxRecordEntity`
- `AdminRoleCacheEntity`
- `ObservedAccountEntity`
- `LoginReplayEntity`
- `AppKvEntity`

当前 schema 版本：`wallet.data.schema.version = 4`。

### 5.3 偏好层（SharedPreferences）

仍有少量非机密配置使用（按模块逐步收口）：

- 登录白名单配置：`login.whitelist_config.v1`
- 登录防重放记录：`login.used_request_ids`
- SFID 绑定状态：`sfid.bind.*`
- 用户资料：
  - `user.profile.nickname`
  - `user.profile.avatar_path`

## 6. 外部 API 对接（当前）

App 通过 `ApiClient` 访问外部服务，当前已使用接口：

- `GET /api/v1/health`
- `GET /api/v1/wallet/balance`
- `POST /api/v1/tx/prepare`
- `POST /api/v1/tx/submit`
- `GET /api/v1/tx/status/:tx_hash`
- `POST /api/v1/chain/bind/request`
- `GET /api/v1/admins/catalog`

### 6.1 区块链能力矩阵（转账 / 提案 / 投票）

| 能力 | 链上入口 | 手机端模块 | 签名域 | 当前状态 |
| --- | --- | --- | --- | --- |
| 转账 | 链上转账 extrinsic（由外部网关 prepare/submit 封装） | `lib/trade/onchain` | `onchain_tx` | 已上线（本机签名主链路） |
| 提案 | 业务治理 pallet `propose_*` | `lib/governance`（规范已定） | `onchain_tx`（交易签名）+ SFID 快照签名字段 | 待开发 |
| 投票 | 业务治理 `vote_*` / 投票引擎 `submit_joint_institution_vote` / `citizen_vote` | `lib/governance`（规范已定） | `onchain_tx`（交易签名）+ SFID 投票凭证签名字段 | 待开发 |

### 6.2 区块链字段与格式标准（总则）

- 地址：SS58 字符串（当前链 `ss58 = 2027`）。
- 机构 ID：链上 `[u8; 48]`，App 统一使用 `0x` + 96 hex 表达。
- 签名算法：统一 `sr25519`。
- `nonce/signature`：治理场景均使用字节向量（运行时上限当前为 64 字节）。
- 提案状态：`voting/passed/rejected`（内部执行失败状态由业务 pallet 事件单独体现）。
- 投票引擎外部禁用项：
  - `create_joint_proposal`（外部调用禁止）
  - `internal_vote`（外部调用禁止）
  - 必须通过业务治理 pallet 发起。

详细字段与流程见：

- `lib/trade/onchain/ONCHAIN_TECHNICAL.md`（转账）
- `lib/governance/GOVERNANCE_TECHNICAL.md`（提案/投票）

## 7. 安全基线（当前）

- 私钥/助记词不落 Isar 与远端服务
- 登录与交易签名前有设备侧身份确认能力（可开关）
- 登录白名单配置有本地 HMAC 完整性保护
- 绑定请求与交易状态依赖外部服务返回

## 8. 已知限制

- 登录防重放当前仍在 `SharedPreferences`，尚未切到 Isar 的 `LoginReplayEntity`
- `SfidBindingService` 状态仍在 `SharedPreferences`（`sfid.bind.*`）
- 链下交易模块仍为占位
- 扫码签名当前已完成协议层实现，业务 UI 仍以本机签名为主

## 9. 本地开发

```bash
cd /Users/rhett/GMB/wuminapp
flutter pub get
flutter run \
  --dart-define=WUMINAPP_API_BASE_URL=http://<外部服务地址> \
  --dart-define=WUMINAPP_API_TOKEN=<token>
```

真机调试时 `WUMINAPP_API_BASE_URL` 需为手机可达地址，不可用 `127.0.0.1`。

## 10. 关联模块文档

- 登录模块：`lib/login/LOGIN_TECHNICAL.md`
- 签名模块：`lib/signer/SIGNER_TECHNICAL.md`
- 治理模块：`lib/governance/GOVERNANCE_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
- 链上交易模块：`lib/trade/onchain/ONCHAIN_TECHNICAL.md`
