# Signer 模块技术文档（当前实现态）

## 1. 模块目标

`lib/signer` 是签名能力的独立模块，统一承载：

- 手机本机签名（私钥/助记词在手机本地）
- 扫码签名请求与回执结构校验

设计原则：

- `wallet` 管钱包与密钥材料生命周期
- `signer` 管签名算法与签名协议
- `login`、`onchain`、`myid` 只编排流程，不直接写签名细节
- 冷钱包离线解码和独立确认由 `citizenwallet/lib/signer/` 承担，CitizenApp 不实现公民钱包签名放行规则

## 2. 目录结构

```text
lib/signer/
├── local_signer.dart
├── qr_signer.dart
└── signer.dart
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

- 定义扫码签名协议 `CITIZEN_QR_V1`
- 定义请求/回执数据结构与校验
- 校验 request/response 关键信息一致性（`request_id`、`account`、`payload_hash`）
- 校验过期时间、时钟偏差、hex 字段合法性
- 请求包含 `display` 字段，作为人可读提示和业务独立校验输入

### CPMS 档案钱包账户

- 最后更新:2026-05-26
- 任务卡:`memory/08-tasks/open/20260526-cpms-wallet-address-only.md`

CitizenApp 在 CPMS 阶段不签名。CPMS 只扫描电子护照页展示的 `CITIZEN_QR_V1 / user_contact`
钱包地址二维码，并把地址写入真实档案。钱包私钥控制权验证统一放到 CID 绑定阶段，
由 CID 生成绑定 `sign_request`，CitizenApp 再输出 `CITIZEN_QR_V1 / sign_response`。

验收规则:

- 普通签名请求的 `body.pubkey` 必须与当前钱包公钥一致。
- 签名对象为 `payload_hex` 对应的业务原文。
- CID 绑定阶段的回执 `pubkey / payload_hash / signature` 必须能被 CID 验证。

签名场景：

- `login`
- `onchain_tx`

说明：

- 协议编解码与基础校验能力已接入登录、链上交易、治理、电子护照等业务页面。
- `QrSignSessionPage` 负责在线手机展示请求二维码并扫描外部签名回执。
- CitizenApp 不接收“无法独立验证但仍允许签名”的冷钱包结果；外部签名设备必须按 CitizenWallet 的两色规则独立验证后返回回执。
- `display` 不是签名内容，真实签名对象始终是 `payload_hex` 解码后的原始字节。

### 4.1 CID 电子护照绑定签名

文件：`lib/my/myid/myid_sign_page.dart`

职责：

- 校验 `sign_request.body.pubkey` 与当前钱包公钥一致。
- 校验 `sign_request.body.account` 与当前钱包 SS58 地址一致。
- 校验 `display.action = citizen_bind`。
- 独立解析 `payload_hex` 中的 `cid-citizen-bind-v1` 业务原文。
- 校验业务原文中的 `wallet_pubkey` 与当前钱包公钥一致。
- 若请求包含 `display.fields`，逐项校验 `mode / archive_no / voting_eligible / citizen_status / wallet_address` 与业务原文一致。
- 校验通过后才使用当前钱包对 `payload_hex` 原始字节签名。

### 4.2 公民钱包签名边界

- CitizenWallet 公民钱包负责独立解码链上 call data、CID 管理员操作、CPMS 档案删除等签名请求。
- CitizenWallet 公民钱包只能在独立验证通过时绿色放行，不能独立验证或展示字段不一致时红色拒签。
- CitizenApp 只负责展示在线请求二维码、扫描回执、校验 `request_id / pubkey / payload_hash / signature` 后提交业务。

## 5. 协议口径（CITIZEN_QR_V1）

### 5.1 签名请求（手机 -> 外部签名设备）

字段：

- `proto = CITIZEN_QR_V1`
- `type = sign_request`
- `request_id`
- `account`
- `pubkey`
- `sig_alg = sr25519`
- `payload_hex`
- `issued_at`
- `expires_at`
- `display` — 人可读交易摘要（`Map`），必须包含 `action` 和 `summary`

`GMB_DECRYPT_V1` 不是新的二维码协议。它只是在 `CITIZEN_QR_V1` 的
`payload_hex` 内部给"管理员解密清算密钥"挑战签名使用的业务域前缀,用于让冷钱包
在人眼确认时把该请求识别为本地密钥解密授权,不把它误判成链上 extrinsic。

### 5.2 签名回执（外部签名设备 -> 手机）

字段：

- `proto = CITIZEN_QR_V1`
- `type = sign_response`
- `request_id`
- `pubkey`
- `sig_alg = sr25519`
- `signature`
- `signed_at`
- `payload_hash` — SHA-256(payload bytes)，防止 payload 被篡改

## 6. 与其他模块关系

- `wallet`：
  - 负责 `WalletSecret` 来源与钱包激活态
  - 不再直接实现签名算法细节
- `qr/login`：
  - 负责挑战解析、防重放、系统签名验证、签名前确认
  - 热钱包通过 `WalletManager.signUtf8WithWallet()` 本机签名
  - 冷钱包通过 `QrSigner + QrSignSessionPage` 发起外部签名会话，由 CitizenWallet 独立验证后返回回执
- `onchain`：
  - 负责交易草稿校验、prepare/submit/status 编排
  - 热钱包通过 `WalletManager.signWithWallet()` 本机签名
  - 冷钱包通过 `QrSigner + QrSignSessionPage` 发起外部签名会话，由 CitizenWallet 校验 payload/display 后签名
- `governance`（规划）：
  - 负责提案/投票业务字段编排
  - 通过 `Signer` 完成链上交易签名
  - CID 凭证签名字段由外部 CID 系统生成，App 负责透传与校验格式

## 7. 签名域标准

### 7.1 登录签名域

- 固定拼串：

```text
CITIZEN_QR_V1|system|request_id|challenge|nonce|expires_at
```

### 7.2 转账/治理交易签名域

- `onchain_tx` / `gov_proposal` / `gov_vote` 均使用“链端待签名 payload 字节”签名。
- App 不自行重组链上 SCALE payload，必须直接签 `payload_hex` 解码后的原始字节。
- 签名结果统一为 `sr25519` 64 字节签名（`0x` hex）。

### 7.3 CID 凭证签名域（由 CID 系统产出）

- 人口快照签名（投票引擎人口快照准备流程字段）：

```text
(GMB, OP_SIGN_POP, genesis_hash, who, eligible_total, nonce)
```

- 公民投票凭证签名：

```text
(GMB, OP_SIGN_VOTE, genesis_hash, who, binding_id, proposal_id, nonce)
```

两类消息均采用 `blake2_256(SCALE.encode(payload))` 后做 `sr25519` 签名。

## 8. 安全要求

- 私钥/助记词仅在本地机密存储，不经二维码传输
- 登录签名与交易签名必须保持域隔离（不同签名消息）
- 转账签名与治理签名必须保持域隔离（不同 payload 来源）
- 扫码回执必须校验 `request_id` 和 `payload_hash`，拒绝错配签名或 payload 被篡改
- 仅支持 `sr25519`，避免算法混淆
- 治理相关 `nonce/signature` 字段必须校验字节长度上限（当前 64）
- 离线设备必须独立解码 payload 并与 display 交叉验证，不能独立验证时不得返回可用回执
- display 与 payload 解码结果 action 不一致时必须阻止签名

## 9. 后续扩展点

- 支持二维码分片/重组与重传
- 引入设备绑定与会话确认（防中间人替换二维码）
- 增加扫码签名链路的端到端测试
