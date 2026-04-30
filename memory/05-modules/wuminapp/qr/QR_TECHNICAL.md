# QR 模块技术文档（当前实现态）

## 1. 模块目标

`lib/qr/` 是 WuminApp 所有二维码能力的统一收口模块，负责：

- 统一协议定义与版本管理
- 扫码内容路由分发
- 登录码（挑战-回执）全流程编排
- 收款码生成与解析
- 用户码（通讯录交换）生成与解析
- 扫码页面与回执展示

设计原则：

- 所有二维码内容为 JSON 格式，通过 `proto` 字段识别类型
- 一个 `QrRouter` 统一路由，各子模块各自负责解析与校验
- 登录模块原位于 `lib/login/`，因其唯一用途为扫码登录，已整体迁入 `lib/qr/login/`
- 签名算法由 `lib/signer/` 提供，本模块不直接实现签名细节

## 2. 目录结构

```text
lib/qr/
├── qr_protocols.dart          ← 协议常量
├── qr_router.dart             ← 统一路由器
├── login/
│   ├── login_models.dart      ← 登录挑战/回执模型与错误码
│   ├── login_service.dart     ← 登录签名编排
│   └── login_replay_guard.dart← 防重放守卫
├── transfer/
│   └── transfer_qr_models.dart← 收款码模型
├── contact/
│   └── contact_qr_models.dart ← 用户码模型
├── pages/
│   ├── qr_scan_page.dart      ← 统一扫码页面
│   └── qr_sign_session_page.dart ← 冷钱包扫码签名会话页面
└── QR_TECHNICAL.md
```

## 3. 协议标识

所有协议标识定义在 `qr_protocols.dart`：

| 协议常量 | 值 | 用途 |
| --- | --- | --- |
| `login` | `WUMIN_QR_V1` | 登录、绑定签名验证 |
| `sign` | `WUMIN_QR_V1` | 冷钱包离线交易签名 |
| `user` | `WUMIN_QR_V1` | 用户信息传输（联系人、付款，通过 purpose 字段区分） |

## 4. 路由器（QrRouter）

文件：`qr_router.dart`

统一接收扫码原始字符串，返回 `QrRouteResult`，供上层页面分发处理。

路由优先级：

1. 尝试 JSON 解析，按 `proto`（或 `type`）字段匹配协议
2. 匹配 `gmb://account/<address>` 格式 → `legacyAddress`
3. 匹配裸 SS58 地址（Base58，30-80 字符）→ `legacyAddress`
4. 以上均不匹配 → `unknown`

路由类型（`QrRouteType`）：

| 类型 | 触发条件 |
| --- | --- |
| `login` | `proto == WUMIN_QR_V1` |
| `transfer` | `proto == WUMIN_QR_V1` 且 `purpose == transfer` |
| `contact` | `proto == WUMIN_QR_V1` 且 `purpose == contact`（或无 purpose） |
| `sign` | `proto == WUMIN_QR_V1` |
| `legacyAddress` | `gmb://account/...` 或裸 SS58 地址 |
| `unknown` | 无法识别 |

## 5. 登录码协议（WUMIN_QR_V1）

### 5.1 系统架构

- SFID：运行在一台云服务器上的在线系统
- CPMS：运行在千千万万台电脑上的离线系统
- 登录协议只用于 `sfid/cpms` 扫码登录，不用于链上转账、投票或治理签名
- 登录为双层签名：
  - 第一层：系统使用自身私钥对登录二维码签名，手机验 `sys_pubkey + sys_sig`
  - 第二层：管理员钱包对 challenge 签名，系统再用内置管理员公钥名单验签

### 5.2 挑战码字段（系统 → 手机）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `proto` | string | 是 | 固定 `WUMIN_QR_V1` |
| `system` | string | 是 | `sfid` 或 `cpms` |
| `challenge` | string | 是 | 随机挑战值 |
| `issued_at` | int | 是 | 签发时间（秒级 epoch） |
| `expires_at` | int | 是 | 过期时间（秒级 epoch） |
| `sys_pubkey` | string | 是 | 系统公钥（`0x` + hex） |
| `sys_sig` | string | 是 | 系统对挑战字段的签名（`0x` + hex） |

### 5.3 系统签名验证

系统签名原文：

```text
proto|system|challenge|issued_at|expires_at|sys_pubkey
```

验证逻辑：

- SFID 场景：
  - 用二维码中的 `sys_pubkey` 验证 `sys_sig`
- CPMS 场景：
  - 用二维码中的 `sys_pubkey` 验证 `sys_sig`
  - 不再要求链上或 SFID 证书链参与手机端登录验签

### 5.4 挑战校验规则

- TTL 固定 90 秒（`expires_at - issued_at == 90`）
- 最大时钟偏差 30 秒（`issued_at` 不超过当前时间 + 30 秒）
- `challenge` 长度 4-512 字符，不含空白
- `sys_pubkey`/`sys_sig` 必须为合法偶数字节 hex
- 载荷总长度不超过 4096 字符

### 5.5 用户签名原文（手机签名）

```text
WUMIN_QR_V1|system|challenge|expires_at
```

说明：不包含 `aud` 字段，系统身份通过 `sys_pubkey`/`sys_sig` 密码学验证。

### 5.6 回执码字段（手机 → 系统）

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `proto` | string | 固定 `WUMIN_QR_V1` |
| `system` | string | 回执来源系统（`sfid` 或 `cpms`） |
| `challenge` | string | 与挑战码对应 |
| `pubkey` | string | 用户公钥（`0x` + hex） |
| `sig_alg` | string | 固定 `sr25519` |
| `signature` | string | 签名（`0x` + hex） |
| `signed_at` | int | 签名时间（秒级 epoch） |
| `payload_hash` | string | 签名原文的 SHA-256 哈希（`0x` + hex），用于防篡改校验 |

说明：回执码不包含 `account`（地址）字段，仅提供 `pubkey`。`system` 标识回执来源系统，`payload_hash` 用于服务端验证签名原文未被篡改。

### 5.7 防重放

- 基于 `challenge` 一次性消费
- 存储：`SharedPreferences`（键 `login.used_challenges`）
- 过期条目自动清理

### 5.7.1 服务端回执兼容要求

为兼容不同系统前端实现，服务端接收登录回执时应同时兼容以下字段别名：

- `challenge` 或 `challenge_id` 或 `request_id`
- `pubkey` 或 `admin_pubkey` 或 `public_key`
- `signature` 或 `sig`

`sig_alg`、`signed_at` 属于可选扩展字段，服务端可记录审计但不应作为当前版本的必填拒绝条件。

### 5.8 错误码

| 错误码 | 常量 | 含义 |
| --- | --- | --- |
| L1001 | `invalidFormat` | 二维码格式错误 |
| L1002 | `invalidProtocol` | 不支持的协议 |
| L1003 | `invalidSystem` | 不支持的系统 |
| L1004 | `missingField` | 缺少必要字段 |
| L1005 | `invalidField` | 字段格式错误 |
| L1101 | `expired` | 挑战已过期 |
| L1102 | `replay` | 请求已使用（重放攻击） |
| L1103 | `invalidTtl` | 有效期不合法 |
| L1201 | `invalidSystemSignature` | 系统签名验证失败 |
| L1202 | `untrustedSystem` | 系统不可信（CPMS 证书链验证失败） |
| L1301 | `walletMissing` | 未创建钱包 |
| L1302 | `walletNotFound` | 指定钱包不存在 |
| L1303 | `walletMismatch` | 签名密钥与钱包不一致 |
| L1401 | `biometricUnavailable` | 生物识别不可用 |
| L1402 | `biometricRejected` | 生物识别被拒绝 |

## 6. 收款码协议（WUMIN_QR_V1（purpose=transfer））

### 6.1 字段

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `proto` | string | 是 | 固定 `WUMIN_QR_V1（purpose=transfer）` |
| `to` | string | 是 | 收款地址（SS58 格式） |
| `amount` | string | 否 | 金额（字符串避免浮点精度） |
| `symbol` | string | 否 | 币种，默认 `GMB` |
| `memo` | string | 否 | 备注（展示用） |
| `bank` | string | 否 | 清算省储行标识（预留，链下支付用） |

### 6.2 使用流程

1. 收款方生成收款码二维码（可指定金额或留空）
2. 付款方扫码后自动填入转账表单
3. 金额为空时由付款方手动输入

### 6.3 向后兼容

扫码页同时支持：

- `WUMIN_QR_V1（purpose=transfer）` JSON 格式 → 完整解析
- `gmb://account/<address>` 格式 → 仅填充收款地址
- 裸 SS58 地址 → 仅填充收款地址

## 7. 用户码协议（WUMIN_QR_V1）

### 7.1 新版字段

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `proto` | string | 是 | 固定 `WUMIN_QR_V1` |
| `address` | string | 是 | 链上地址（SS58 格式） |
| `name` | string | 是 | 用户昵称 |

### 7.2 旧版兼容（WUMINAPP_USER_CARD_V1）

旧版字段映射：

| 旧版字段 | 新版字段 |
| --- | --- |
| `type` | `proto` |
| `account_pubkey` | `address` |
| `nickname` | `name` |

解析时自动识别旧版格式并映射为新版模型。

### 7.3 设计变更说明

- 旧版使用 `account_pubkey`（裸公钥 hex），新版使用 `address`（SS58 地址）
- SS58 地址包含链标识（ss58 = 2027），更安全且用户可读
- 生成二维码统一使用新版 `WUMIN_QR_V1` 格式
- 解析二维码同时兼容新旧两版

## 8. 统一扫码页面

文件：`lib/qr/pages/qr_scan_page.dart`

### 8.1 页面职责

- 使用 `mobile_scanner` 扫描二维码
- 通过 `QrRouter` 路由分发
- 按类型执行不同处理逻辑

### 8.2 扫码结果处理

| 路由类型 | 处理方式 |
| --- | --- |
| `login` | 进入登录回执流程 |
| `transfer` | 返回 `QrScanTransferResult`（含地址、金额、币种） |
| `legacyAddress` | 返回 `QrScanTransferResult`（仅地址） |
| `contact` | 由调用方处理 |
| `sign` | 提示用户在转账页面发起签名后使用 |
| `unknown` | 提示错误 |

### 8.3 登录回执页面

扫描到登录挑战后展示回执页面：

1. 解析挑战码
2. 验证系统签名（预留）
3. 调用 `LoginService.buildReceiptPayload()` 生成回执
4. 展示回执二维码与倒计时
5. 倒计时结束自动关闭

## 9. 与其他模块关系

- `signer/`：
  - `LocalSigner` 执行 sr25519 登录签名
  - `QrSigner` 提供扫码签名协议（`WUMIN_QR_V1`）
  - `OfflineSignService` 为离线设备执行 `sign_request -> sign_response`（含 payload 交叉验证）
  - `PayloadDecoder` 独立解码 SCALE call data，用于离线端防盲签验证
- `wallet/`：
  - `WalletManager` 提供钱包密钥材料
  - `capabilities/sign_service.dart` 已重构为 re-export `qr/login/` 的兼容层
- `onchain/`：
  - 使用 `QrScanTransferResult` 预填转账表单
- `user/`：
  - `UserQrPayload` 已迁移到 `WUMIN_QR_V1` 格式
  - 扫码页面由 `qr/pages/qr_scan_page.dart` 统一提供

## 10. 安全要求

- 登录挑战 TTL 固定 90 秒，不可配置
- `challenge` 一次性消费，防重放
- 系统身份通过密码学签名验证（`sys_pubkey`/`sys_sig`），不依赖白名单
- 签名域隔离：登录签名与交易签名使用不同签名消息格式
- 私钥/助记词不经二维码传输
- 回执仅包含公钥，不包含地址（防止信息泄露超出必要范围）

## 11. 测试覆盖

- `test/qr/qr_router_test.dart`
  - 各协议路由匹配（V2 协议标识）
  - `gmb://account/` 和裸 SS58 地址识别
  - 旧版用户码兼容
  - 空值和未知格式处理
- `test/qr/qr_sign_session_test.dart`
  - 会话页面展示与倒计时
  - 取消返回 null
  - 过期状态 UI
- `test/wallet/sign_service_test.dart`
  - 挑战解析与校验
  - 签名原文格式（不含 `aud`）
  - 回执不含 `account` 字段
  - 防重放
  - 钱包缺失/不匹配
- `test/signer/qr_signer_test.dart`
  - V2 请求/回执编解码往返（含 display、payloadHash）
  - display 校验（缺失 display、缺失 action）
  - 过期校验
  - `request_id` 和 `payload_hash` 匹配校验
  - `computePayloadHash` 确定性
- `test/signer/offline_sign_service_test.dart`
  - 签名与 display 交叉验证（matched / mismatched / decodeFailed）
  - display mismatch 阻止签名
  - pubkey 不匹配拒绝
  - 冷钱包拒绝
- `test/signer/payload_decoder_test.dart`
  - transfer_keep_alive 解码
  - vote_transfer 解码（赞成/反对）
  - joint_vote 解码
  - 未知 pallet / 过短 payload / 空值返回 null

## 12. 冷钱包扫码签名会话

文件：`lib/qr/pages/qr_sign_session_page.dart`

### 12.1 页面职责

两阶段交互页面，用于冷钱包转账签名：

1. **展示请求二维码** — 将 `QrSignRequest` 编码为 JSON 后生成二维码，供离线设备扫描
2. **扫描回执二维码** — 用户点击"扫描回执"打开简单扫码页，扫描离线设备生成的签名回执

### 12.2 交互流程

```
转账页面                    QrSignSessionPage               离线设备
  │ sign(payload) 被调用 ──→│                                │
  │                  展示请求二维码（含 payload_hex）           │
  │                           │──── 用户扫码 ────→            │
  │                           │                    离线设备签名
  │                           │                    展示回执二维码
  │                  点击"扫描回执"                             │
  │                           │──── 扫描回执 ────→             │
  │                  parseResponse() 校验                      │
  │ ←── pop(QrSignResponse) ──│                               │
  │ 继续 submitTransfer       │                               │
```

### 12.3 输入/输出

- **输入：** `QrSignRequest` + 编码后的 JSON 字符串 + 当前页面期望签名公钥 `expectedPubkey`
- **输出：** `QrSignResponse`（成功）或 `null`（取消/超时）
- 通过 `Navigator.pop()` 返回结果

### 12.3.1 回执校验规则

- `request_id` 必须与当前会话一致
- `pubkey` 必须与页面发起签名时选中的钱包公钥一致
- 任一校验失败都必须拒绝回执，不能把错误钱包的签名继续交给业务模块

### 12.4 UI 元素

- 倒计时状态栏（绿色/红色，从 `expiresAt` 倒数）
- 请求二维码（`QrImageView`, 240px）
- 提示文字
- 按钮行："取消" + "扫描回执"（过期后 disable）

### 12.5 内嵌简单扫码页

`_SimpleScanner`：最小扫码页面，扫到任何 QR 码后返回原始字符串。不做协议路由，路由由 `QrSignSessionPage` 通过 `QrSigner.parseResponse()` 负责。

### 12.6 离线执行端页面

`QrOfflineSignPage`：离线设备入口页面，负责扫描 `sign_request`、调用 `OfflineSignService.verifyPayload()` 交叉验证 display 与 payload、展示验证结果（三态颜色标识）、完成本机签名，并展示 `sign_response` 回执二维码。

交叉验证状态展示：
- 绿色横幅 — payload 解码与 display 一致（`matched`）
- 红色横幅 — payload 解码与 display 不一致（`mismatched`），签名按钮禁用
- 橙色横幅 — payload 无法解码（`decodeFailed`），仅展示 display 内容

## 13. 后续扩展

- 为用户码增加签名与时效控制，防篡改
- 收款码增加签名验证（可选），防伪造收款地址
- 登录防重放迁移到 Isar（`LoginReplayEntity`）
- 二维码分片/重组支持（大载荷场景）
