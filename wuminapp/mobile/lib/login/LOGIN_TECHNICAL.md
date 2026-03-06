# Login 模块技术文档

## 1. 目标

`mobile/lib/login` 负责扫码登录业务编排，不负责钱包密钥持有。  
模块目标是：识别登录二维码、做登录挑战校验、触发签名前确认、生成登录回执二维码。

## 2. 目录

```text
login/
├── models/
│   ├── login_models.dart
│   └── login_exception.dart
├── pages/
│   └── qr_scan_page.dart
├── services/
│   ├── login_replay_guard.dart
│   ├── login_whitelist_policy.dart
│   └── login_whitelist_store.dart
└── LOGIN_TECHNICAL.md
```

## 3. 责任边界

- 登录模块负责：
  - 扫码识别登录挑战二维码
  - 登录 challenge 协议校验
  - `aud` 白名单校验
  - `request_id` 防重放
  - 用户交互确认与回执二维码展示
- 钱包模块负责：
  - 读取钱包机密材料
  - `sr25519` 签名
  - 钱包公钥一致性校验

## 4. 协议规范（当前口径）

### 4.1 挑战二维码（手机扫描输入）

- `proto`: 固定 `WUMINAPP_LOGIN_V1`
- `system`: 当前仅支持 `cpms`、`sfid`
- `request_id`: 挑战唯一 ID
- `challenge`: 随机挑战串
- `nonce`: 随机串
- `issued_at`: 秒级时间戳
- `expires_at`: 秒级时间戳
- `aud`: 登录来源标识（白名单校验字段）

说明：`origin` 已从手机端登录协议移除，不再参与扫码签名与白名单校验。

### 4.2 签名原文（手机端固定拼接）

```text
WUMINAPP_LOGIN_V1|system|aud|request_id|challenge|nonce|expires_at
```

### 4.3 回执二维码（手机展示输出）

- `proto`
- `request_id`
- `account`
- `pubkey`
- `sig_alg`（固定 `sr25519`）
- `signature`
- `signed_at`

## 5. 运行流程

1. `QrScanPage` 扫码识别 `proto == WUMINAPP_LOGIN_V1`。
2. `SignService.parseChallenge()` 校验字段完整性、系统范围、过期时间、固定 TTL=90 秒。
3. `LoginWhitelistPolicy.assertAllowed()` 校验 `aud` 白名单。
4. `UserIdentificationService.confirmBeforeSign()` 做生物识别确认（若开关开启）。
5. `LoginReplayGuard.assertNotConsumed(request_id)` 防重放。
6. 钱包模块读取当前钱包助记词并执行 `sr25519` 签名。
7. 生成回执 JSON 并展示二维码。
8. `LoginReplayGuard.consume(request_id)` 记录已消费。

## 6. 安全机制

- 协议与字段校验：拒绝非 `WUMINAPP_LOGIN_V1`、拒绝不支持系统、拒绝无效字段。
- 时效约束：challenge 必须未过期，且 `expires_at - issued_at == 90`。
- 白名单：仅校验 `aud`，默认：
  - `cpms -> cpms-local-app`
  - `sfid -> sfid-local-app`
- 防重放：本地持久化已消费 `request_id`，过期条目自动清理。
- 设备身份确认：签名前可要求生物识别。
- 钱包一致性：签名前校验“助记词派生公钥 == 当前钱包公钥”。

## 7. 本地存储

- `SharedPreferences`：
  - `login.used_request_ids`（防重放记录）
  - `login.whitelist_config.v1`（白名单配置 envelope）
- `flutter_secure_storage`：
  - `login.whitelist_hmac_secret.v1`（白名单配置签名密钥）

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

## 9. 对端系统联调要求（CPMS/SFID）

- 对端验签拼串必须使用当前手机端口径：
  - `WUMINAPP_LOGIN_V1|system|aud|request_id|challenge|nonce|expires_at`
- 对端挑战二维码不应再要求 `origin` 字段作为签名要素。
- 对端仍需执行：
  - `request_id` 一次性消费
  - 过期校验
  - 管理员权限判定

## 10. 测试

- 关键测试：`test/wallet/sign_service_test.dart`
- 覆盖点：
  - challenge 解析
  - TTL 校验
  - canonical 签名原文
  - 选定钱包签名
  - 防重放
