# Signer 模块技术文档（当前实现态）

## 1. 模块目标

`lib/signer` 是签名能力的独立模块，统一承载：

- 手机本机签名（私钥/助记词在手机本地）
- 扫码签名协议（手机不存私钥，外部设备签名）

设计原则：

- `wallet` 管钱包与密钥材料生命周期
- `signer` 管签名算法与签名协议
- `login`、`trade/onchain` 只编排流程，不直接写签名细节

## 2. 目录结构

```text
lib/signer/
├── local_signer.dart
├── offline_sign_service.dart
├── qr_signer.dart
├── signer.dart
├── system_signature_verifier.dart
└── SIGNER_TECHNICAL.md
```

## 3. 本机签名（LocalSigner）

文件：`local_signer.dart`

职责：

- 使用 `WalletSecret`（钱包资料 + seed hex）执行 `sr25519` 签名
- 校验本地派生公钥与钱包记录公钥一致，防止错签
- 返回统一签名结果：
  - `account`
  - `pubkeyHex`
  - `sigAlg`
  - `signatureHex`

错误码：

- `emptyPayload`
- `unsupportedAlgorithm`
- `walletMismatch`

## 4. 扫码签名（QrSigner）

文件：`qr_signer.dart`

职责：

- 定义扫码签名协议 `WUMINAPP_QR_SIGN_V1`
- 定义请求/回执数据结构与校验
- 校验 request/response 关键信息一致性（`request_id`、`account`）
- 校验过期时间、时钟偏差、hex 字段合法性

签名场景：

- `login`
- `onchain_tx`
- `gov_proposal`（规划中）
- `gov_vote`（规划中）

说明：

- 协议编解码与校验能力已接入登录、链上交易、治理等业务页面
- `QrSignSessionPage` 负责在线手机展示请求二维码并扫描回执
- `QrOfflineSignPage` 负责离线设备扫描请求并展示签名回执二维码

### 4.1 离线签名执行服务（OfflineSignService）

文件：`offline_sign_service.dart`

职责：

- 解析在线设备展示的 `sign_request`
- 校验请求中的 `account/pubkey` 与当前本机热钱包完全一致
- 调用 `WalletManager.signWithWallet()` 在本机完成签名
- 生成统一 `sign_response` 回执二维码数据

### 4.2 登录系统签名验证（LoginSystemSignatureVerifier）

文件：`system_signature_verifier.dart`

职责：

- 用二维码中的 `sys_pubkey` 验证 `sys_sig`
- 从链上读取 `SfidCodeAuth::SfidMainAccount`，得到当前 SFID 主验签公钥
- `sfid` 场景校验二维码公钥与链上当前 SFID 公钥一致
- `cpms` 场景校验 `sys_cert` 是否由链上 SFID 当前公钥签发
- 对 CPMS 证书时间窗口做覆盖校验，拒绝挑战有效期超出证书范围的请求

## 5. 协议口径（WUMINAPP_QR_SIGN_V1）

### 5.1 签名请求（手机 -> 外部签名设备）

字段：

- `proto = WUMINAPP_QR_SIGN_V1`
- `type = sign_request`
- `scope = login | onchain_tx`（已实现）
- `scope = gov_proposal | gov_vote`（规划中，向后兼容扩展）
- `request_id`
- `account`
- `pubkey`
- `sig_alg = sr25519`
- `payload_hex`
- `issued_at`
- `expires_at`

### 5.2 签名回执（外部签名设备 -> 手机）

字段：

- `proto = WUMINAPP_QR_SIGN_V1`
- `type = sign_response`
- `request_id`
- `pubkey`
- `sig_alg = sr25519`
- `signature`
- `signed_at`

## 6. 与其他模块关系

- `wallet`：
  - 负责 `WalletSecret` 来源与钱包激活态
  - 不再直接实现签名算法细节
- `qr/login`：
  - 负责挑战解析、防重放、系统签名验证、签名前确认
  - 热钱包通过 `WalletManager.signUtf8WithWallet()` 本机签名
  - 冷钱包通过 `QrSigner + OfflineSignService` 完成外部签名
- `trade/onchain`：
  - 负责交易草稿校验、prepare/submit/status 编排
  - 热钱包通过 `WalletManager.signWithWallet()` 本机签名
  - 冷钱包通过 `QrSigner + QrSignSessionPage + OfflineSignService` 完成外部签名
- `governance`（规划）：
  - 负责提案/投票业务字段编排
  - 通过 `Signer` 完成链上交易签名
  - SFID 凭证签名字段由外部 SFID 系统生成，App 负责透传与校验格式

## 7. 签名域标准

### 7.1 登录签名域

- 固定拼串：

```text
WUMINAPP_LOGIN_V1|system|request_id|challenge|nonce|expires_at
```

### 7.2 转账/治理交易签名域

- `onchain_tx` / `gov_proposal` / `gov_vote` 均使用“链端待签名 payload 字节”签名。
- App 不自行重组链上 SCALE payload，必须直接签 `payload_hex` 解码后的原始字节。
- 签名结果统一为 `sr25519` 64 字节签名（`0x` hex）。

### 7.3 SFID 凭证签名域（由 SFID 系统产出）

- 人口快照签名（联合提案字段）：

```text
("GMB_SFID_POPULATION_V3", genesis_hash, who, eligible_total, nonce)
```

- 公民投票凭证签名：

```text
("GMB_SFID_VOTE_V3", genesis_hash, who, binding_id, proposal_id, nonce)
```

两类消息均采用 `blake2_256(SCALE.encode(payload))` 后做 `sr25519` 签名。

## 8. 安全要求

- 私钥/助记词仅在本地机密存储，不经二维码传输
- 登录签名与交易签名必须保持域隔离（不同签名消息）
- 转账签名与治理签名必须保持域隔离（不同 scope、不同 payload 来源）
- 扫码回执必须校验 `request_id`，拒绝错配签名
- 仅支持 `sr25519`，避免算法混淆
- 治理相关 `nonce/signature` 字段必须校验字节长度上限（当前 64）

## 9. 后续扩展点

- 支持二维码分片/重组与重传
- 引入设备绑定与会话确认（防中间人替换二维码）
- 增加扫码签名链路的端到端测试
