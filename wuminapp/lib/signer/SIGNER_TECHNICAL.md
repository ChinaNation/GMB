# Signer 模块技术文档（当前实现态）

## 1. 模块目标

`lib/signer` 是签名能力的独立模块，统一承载：

- 冷钱包扫码签名协议（手机不存私钥，外部设备签名）
- 统一公钥验签基础能力（目标新增）
- 冷热钱包统一签名编排入口（目标新增）

设计原则：

- `wallet` 管钱包与密钥材料生命周期，以及热钱包唯一私钥签名入口
- `signer` 管签名协议、验签能力、冷热签名编排
- `login`、`trade/onchain` 只编排流程，不直接写签名细节

## 2. 目录结构

```text
lib/signer/
├── qr_signer.dart
├── signature_verifier.dart      ← 目标新增：统一公钥验签
├── signing_coordinator.dart     ← 目标新增：冷热统一签名入口
├── signer.dart
└── SIGNER_TECHNICAL.md
```

## 3. 模块边界（冻结）

- **热钱包私钥签名不在 `lib/signer/` 内实现**
- 唯一热钱包私钥签名入口：`WalletManager.signWithWallet()` / `signUtf8WithWallet()`
- `lib/signer/` 不得新增第二套本机 seed 签名实现
- 冷钱包签名协议统一收口在 `QrSigner`
- 系统身份验证、CPMS 背书验证等“公钥验签”能力统一收口在 `lib/signer/`

说明：

`local_signer.dart` 若保留，仅应作为历史兼容件或内部过渡件；业务模块不得直接依赖它执行热钱包签名。

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

- 当前版本已提供协议编解码与校验能力，UI 会话页面在 `lib/qr/pages/qr_sign_session_page.dart`
- 目标改造后由 `SigningCoordinator` 统一调起，不再由各业务页面各自 new `QrSigner`

## 5. 统一验签与统一签名编排（目标方案）

### 5.1 SignatureVerifier（目标新增）

职责：

- 提供统一 `sr25519` 公钥验签能力
- 统一处理：
  - hex 解码
  - 公钥/签名字节长度校验
  - UTF-8 原文或原始字节验签

边界：

- 只做“公钥验签”
- 不接触 seed
- 不执行任何热钱包私钥签名

### 5.2 SigningCoordinator（目标新增）

职责：

- 作为业务层唯一签名编排入口
- 根据 `signMode` 统一分流：
  - `local` → `WalletManager`
  - `external` → `QrSigner + QrSignSessionPage`

目标接口：

- `signUtf8(...)`
- `signBytes(...)`

目标收益：

- 登录、转账、治理共享一套签名编排流程
- 业务层不再直接关心“热签”还是“冷签”
- 避免每个页面重复维护扫码签名流程

## 6. 协议口径（WUMINAPP_QR_SIGN_V1）

### 6.1 签名请求（手机 -> 外部签名设备）

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

### 6.2 签名回执（外部签名设备 -> 手机）

字段：

- `proto = WUMINAPP_QR_SIGN_V1`
- `type = sign_response`
- `request_id`
- `pubkey`
- `sig_alg = sr25519`
- `signature`
- `signed_at`

## 7. 与其他模块关系

- `wallet`：
  - 负责钱包生命周期与热钱包私钥签名
  - `WalletManager` 是唯一热钱包私钥签名入口
- `qr/login`：
  - 负责挑战解析、防重放、系统签名验证、签名前确认
  - 目标改造后通过 `SigningCoordinator` 获取签名结果
- `trade/onchain`：
  - 负责交易草稿校验、prepare/submit/status 编排
  - 目标改造后通过 `SigningCoordinator` 对 signer payload 签名
- `governance`（规划）：
  - 负责提案/投票业务字段编排
  - 目标改造后通过 `SigningCoordinator` 完成链上交易签名
  - SFID 凭证签名字段由外部 SFID 系统生成，App 负责透传与校验格式

## 8. 签名域标准

### 8.1 登录签名域

- 固定拼串：

```text
WUMINAPP_LOGIN_V1|system|request_id|challenge|nonce|expires_at
```

### 8.2 转账/治理交易签名域

- `onchain_tx` / `gov_proposal` / `gov_vote` 均使用“链端待签名 payload 字节”签名。
- App 不自行重组链上 SCALE payload，必须直接签 `payload_hex` 解码后的原始字节。
- 签名结果统一为 `sr25519` 64 字节签名（`0x` hex）。

### 8.3 SFID 凭证签名域（由 SFID 系统产出）

- 人口快照签名（联合提案字段）：

```text
("GMB_SFID_POPULATION_V2", genesis_hash, who, eligible_total, nonce)
```

- 公民投票凭证签名：

```text
("GMB_SFID_VOTE_V2", genesis_hash, who, sfid_hash, proposal_id, nonce)
```

两类消息均采用 `blake2_256(SCALE.encode(payload))` 后做 `sr25519` 签名。

## 9. 安全要求

- 私钥/助记词仅在本地机密存储，不经二维码传输
- 热钱包私钥签名只能由 `WalletManager` 执行
- 登录签名与交易签名必须保持域隔离（不同签名消息）
- 转账签名与治理签名必须保持域隔离（不同 scope、不同 payload 来源）
- 扫码回执必须校验 `request_id`，拒绝错配签名
- 仅支持 `sr25519`，避免算法混淆
- 治理相关 `nonce/signature` 字段必须校验字节长度上限（当前 64）

## 10. 后续扩展点

- 新增 `SignatureVerifier`
- 新增 `SigningCoordinator`
- 清理业务模块对 `LocalSigner` 的直接依赖
- 支持二维码分片/重组与重传
- 引入设备绑定与会话确认（防中间人替换二维码）
- 增加扫码签名链路的端到端测试
