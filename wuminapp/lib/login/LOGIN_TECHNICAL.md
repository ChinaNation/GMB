# Login 模块技术文档（当前实现态）

## 1. 模块目标

`lib/login` 负责扫码登录流程编排，不直接持有钱包密钥。

模块职责：

- 扫描并识别登录挑战二维码
- 校验协议与字段
- 校验白名单（`aud`）
- 防重放（`request_id`）
- 触发签名前身份确认
- 展示回执二维码供对端扫码

签名能力由 `lib/signer/` 提供，登录模块通过
`wallet/capabilities/sign_service.dart` 调用 `LocalSigner` 执行签名。

## 2. 目录结构

```text
login/
├── models/
│   ├── login_models.dart
│   └── login_exception.dart
├── pages/
│   ├── qr_scan_page.dart
│   ├── settings_page.dart
│   └── login_whitelist_page.dart
├── services/
│   ├── login_replay_guard.dart
│   ├── login_whitelist_policy.dart
│   └── login_whitelist_store.dart
└── LOGIN_TECHNICAL.md
```

## 3. 协议口径

### 3.1 挑战二维码（系统 -> 手机）

字段：

- `proto`：固定 `WUMINAPP_LOGIN_V1`
- `system`：当前支持 `cpms`、`sfid`
- `request_id`
- `challenge`
- `nonce`
- `issued_at`（秒）
- `expires_at`（秒）
- `aud`

说明：`origin` 已移除，不参与签名与白名单。

### 3.2 签名原文

```text
WUMINAPP_LOGIN_V1|system|aud|request_id|challenge|nonce|expires_at
```

### 3.3 回执二维码（手机 -> 系统）

字段：

- `proto`
- `request_id`
- `account`
- `pubkey`
- `sig_alg`（固定 `sr25519`）
- `signature`
- `signed_at`

## 4. 执行流程

1. `QrScanPage` 扫码，识别 `proto`
2. `SignService.parseChallenge()` 校验字段、时效、TTL（固定 90 秒）
3. `LoginWhitelistPolicy.assertAllowed()` 校验 `aud` 白名单
4. `UserIdentificationService.confirmBeforeSign()`（开启时）
5. `LoginReplayGuard.assertNotConsumed(request_id)`
6. `LocalSigner` 执行 `sr25519` 签名
7. 生成并展示回执二维码
8. `LoginReplayGuard.consume(request_id)` 记录已消费

回执页交互：

- 按钮 `重新扫码`
- 按钮 `完成`
- 页面显示剩余有效期倒计时

## 5. 安全机制

- 协议强校验：拒绝非 `WUMINAPP_LOGIN_V1`
- 系统强校验：仅允许 `cpms/sfid`
- 时效校验：过期拒绝
- TTL 校验：`expires_at - issued_at == 90`
- 防重放：`request_id` 一次性消费
- 白名单：按 `system -> aud 集合` 校验
- 签名前身份确认：生物识别守卫（开关控制）
- 钱包一致性：助记词派生公钥必须匹配当前钱包公钥
- 域隔离：登录签名串不得复用于转账/提案/投票签名或 SFID 投票凭证

扫码签名扩展：

- 协议层复用 `QrSigner`（`WUMINAPP_QR_SIGN_V1`）
- 当前登录 UI 仍以本机签名为主

## 6. 本地存储

- `SharedPreferences`
  - `login.used_request_ids`（防重放）
  - `login.whitelist_config.v1`（白名单配置 envelope）
- `flutter_secure_storage`
  - `login.whitelist_hmac_secret.v1`（白名单配置签名密钥）

## 7. 默认白名单

- `cpms -> cpms-local-app`
- `sfid -> sfid-local-app`

## 8. 错误码

- `L1001` 二维码格式错误
- `L1002` 协议不支持
- `L1003` 系统不支持
- `L1004` 缺少字段
- `L1005` 字段格式错误
- `L1101` challenge 过期
- `L1102` challenge 重放
- `L1103` TTL 非 90 秒
- `L1201` `aud` 未授权
- `L1301` 钱包缺失
- `L1302` 指定钱包不存在
- `L1303` 钱包公钥不一致
- `L1401` 生物识别不可用
- `L1402` 生物识别拒绝

## 9. 对端联调要求（CPMS / SFID）

- 验签拼串必须与 3.2 完全一致
- 对端必须自行做 `request_id` 一次性消费
- 对端必须做过期校验与业务授权校验

## 10. 测试

关键测试文件：

- `test/wallet/sign_service_test.dart`
- `test/login/login_replay_guard_test.dart`
- `test/login/login_whitelist_store_test.dart`
