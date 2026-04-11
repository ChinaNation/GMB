# 任务卡:WUMIN_QR_V1 协议统一

- 创建日期:2026-04-09
- 状态:open / in-progress
- 最后更新:2026-04-10
- 负责入口:GMB 主聊天

## 背景

历史上存在三个 QR 协议并行:

- `WUMIN_USER_V1.0.0`(用户码/收款码/联系人/多签)
- `WUMIN_LOGIN_V1.0.0`(登录挑战/回执)
- `WUMIN_SIGN_V1.0.0`(离线签名请求/回执)

三者 wire format 同构但字段名散乱(`to`/`address`/`account`/`account_pubkey`/`admin_pubkey`/`public_key`、`challenge`/`request_id`/`challenge_id`、`type`/`purpose`/`msg_type`),导致:

1. **治理发起转账提案**、**设置-手续费收款地址**、**安全基金提案**扫 wuminapp 用户码失败(`parseAddressQr.ts` 读 `obj.to`,wuminapp 写 `address`)
2. SFID 登录回执后端用 `pubkey ?? admin_pubkey ?? public_key`、`signature ?? sig` 堆叠字段别名兼容
3. 没有任何跨端 golden fixture,改任一侧不会报警

## 目标

把三个协议**字面替换**为一个 `WUMIN_QR_V1`,统一字段名与 kind 枚举,**不改任何业务功能**,不搞任何兼容,不留任何残留。

**范围**:全仓库。**例外**:CPMS 的 4 个安装码(QR1/QR2/QR3/QR4)保持独立,不动。

## 原则

1. **功能 0 增 0 减**:现在能做什么,改完还能做什么,UI 流程一分不差
2. **0 兼容**:不搞 `a ?? b` 字段别名,不搞过渡期,不留旧常量
3. **0 残留**:所有旧 proto 字符串、旧类型名、旧字段别名必须全仓库 grep 0 命中
4. **单一事实源**:`memory/05-architecture/qr-protocol-spec.md` 是唯一规范,fixtures 是强制对齐手段

## 规范摘要(详见 qr-protocol-spec.md)

### 顶层 envelope

```jsonc
{
  "proto": "WUMIN_QR_V1",
  "kind":  "<7 选 1>",
  "id":    "<临时码必填,固定码省略>",
  "issued_at":  <临时码必填,固定码省略>,
  "expires_at": <临时码必填,固定码省略>,
  "body":  { ... }
}
```

### 7 个 kind

| kind | 固/临 | 生成者 | 扫描者 |
|---|---|---|---|
| `login_challenge` | 临时 | SFID/CPMS 后端 | wumin |
| `login_receipt` | 临时 | wumin | SFID/CPMS 后端 |
| `sign_request` | 临时 | wuminapp | wumin |
| `sign_response` | 临时 | wumin | wuminapp |
| `user_contact` | **固定** | wuminapp | wuminapp / citizenchain / sfid 前端 |
| `user_transfer` | 临时 | wuminapp | wuminapp / citizenchain |
| `user_duoqian` | **固定** | wuminapp | wuminapp |

**固定码规则**:`id` / `issued_at` / `expires_at` 三字段**直接不出现**在 JSON 里(不是 null、不是 0、不是空串)。

### 统一字段名铁律

- 地址 → `address`(SS58)
- 公钥 → `pubkey`(0x hex)
- 一次性 ID → 顶层 `id`
- 消息类型 → 顶层 `kind`
- 签名算法 → `sig_alg`
- 签名结果 → `signature`
- 其余规范字段:`sys_pubkey` / `sys_sig` / `payload_hex` / `payload_hash` / `signed_at` / `system` / `spec_version` / `display` / `name` / `amount` / `symbol` / `memo` / `bank` / `proposal_id`

### 签名原文拼接

```
WUMIN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<address 或 pubkey>
```

所有需要 sr25519 签名的 kind(`login_receipt`、`sign_response`)都走这个唯一函数。

## 执行步骤(9 步,一条路)

### Step 1:spec + fixtures 先行
- [ ] 写 `memory/05-architecture/qr-protocol-spec.md`
- [ ] 写 7 个 fixture:`memory/05-architecture/qr-protocol-fixtures/{login_challenge,login_receipt,sign_request,sign_response,user_contact,user_transfer,user_duoqian}.json`
- [ ] 写 `memory/05-modules/wuminapp-vs-wumin.md`(角色边界说明)

### Step 2:Dart 新建统一类型(wuminapp + wumin 各一份,逐字节一致)
- [ ] `wuminapp/lib/qr/envelope.dart`:`QrEnvelope` + 7 值 `QrKind` 枚举
- [ ] `wuminapp/lib/qr/signature_message.dart`:唯一 `buildSignatureMessage()`
- [ ] `wuminapp/lib/qr/bodies/*.dart`:7 个 body 类
- [ ] `wumin/lib/qr/envelope.dart`:与 wuminapp 逐字节一致
- [ ] `wumin/lib/qr/signature_message.dart`:同上
- [ ] `wumin/lib/qr/bodies/*.dart`:7 个 body 类

### Step 3:TS 新建统一类型
- [ ] `citizenchain/node/frontend/qr/wuminQr.ts`:完整 envelope 类型
- [ ] `sfid/frontend/src/qr/wuminQr.ts`:同上
- [ ] `cpms/frontend/web/src/qr/wuminQr.ts`:同上

### Step 4:Rust 后端新建统一类型
- [ ] `sfid/backend/src/qr/envelope.rs`:`QrEnvelope<T>` + `QrKind`
- [ ] `cpms/backend/src/qr/envelope.rs`:同上

### Step 5:所有生成点切 envelope
- [x] wuminapp:`signer/qr_signer.dart`(sign_request)、`wallet/ui/receive_qr_page.dart`(user_transfer)、`user/user.dart:425,1042`(user_contact)、`governance/duoqian_qr_sheet.dart`(user_duoqian)
- [x] wuminapp 治理 8 页面修复(2026-04-10):fee_rate_detail / duoqian_close_proposal / personal_duoqian_create / runtime_upgrade / transfer_proposal / runtime_upgrade_detail / transfer_proposal_detail / duoqian_manage_detail — 全部从 Map display + `account:` 切换为 `SignDisplay` + `address:`
- [ ] wumin:`login/login_qr_handler.dart`(login_receipt)、`signer/qr_signer.dart`(sign_response)
- [ ] sfid backend:`login/mod.rs`(login_challenge)
- [ ] cpms backend:`login/mod.rs`(login_challenge)

### Step 6:所有解析点切 envelope
- [ ] wuminapp:`qr/qr_router.dart`、`signer/qr_signer.dart`(sign_response 解析)、`qr/pages/qr_scan_page.dart`、`governance/duoqian_institution_list_page.dart`
- [ ] wumin:`login/login_qr_handler.dart`(login_challenge 解析)、`signer/qr_signer.dart`(sign_request 解析)、`signer/offline_sign_service.dart`、`qr/offline_sign_page.dart`
- [ ] citizenchain:`node/frontend/governance/parseAddressQr.ts`(删 WUMINAPP_USER_CARD_V1 分支,改读 envelope.body.address)、`AddressScanModal.tsx` + 3 个使用页
- [ ] sfid frontend:`components/ScanAccountModal.tsx`、`views/auth/LoginView.tsx`、`api/client.ts`(删字段别名)
- [ ] cpms frontend:`login/LoginPage.tsx`(install/admin 不动)
- [ ] sfid backend:`login/mod.rs`(login_receipt 验证)
- [ ] cpms backend:`login/mod.rs`(login_receipt 验证)

### Step 7:删除旧类型、旧常量、旧字段别名
- [ ] 删 `wuminapp/lib/qr/contact/contact_qr_models.dart`
- [ ] 删 `wuminapp/lib/qr/transfer/transfer_qr_models.dart`
- [ ] 删 `wuminapp/lib/qr/login/login_models.dart`
- [ ] 重写 `wuminapp/lib/qr/qr_protocols.dart`(去掉三个旧常量)
- [ ] 重写 `wumin/lib/qr/qr_protocols.dart`(同上)
- [ ] 删 `wumin/lib/qr/` 和 `wuminapp/lib/signer/` 下所有旧类型(LoginChallenge/LoginReceipt/QrSignRequest/QrSignResponse/TransferQrPayload/UserQrPayload)
- [ ] 后端 Rust 旧 struct 删除
- [ ] 所有 TS 前端字段别名兼容代码(`pubkey ?? admin_pubkey ?? public_key`、`signature ?? sig`、`address ?? to` 等)删除

### Step 8:零命中扫描验收
排除路径:`cpms/backend/src/initialize/` / `cpms/backend/src/dangan/` / `cpms/frontend/web/src/install/` / `cpms/frontend/web/src/admin/`(CPMS 4 码保留)

必须全仓库 0 命中:
```
WUMIN_USER_V1.0.0
WUMIN_LOGIN_V1.0.0
WUMIN_SIGN_V1.0.0
WUMINAPP_USER_CARD_V1
TransferQrPayload
UserQrPayload
LoginChallenge
LoginReceipt
QrSignRequest
QrSignResponse
```

QR 相关代码里 0 命中:
```
account_pubkey
admin_pubkey
public_key
user_address
request_id    (作为字段名)
challenge_id
nickname      (作为字段名)
```

### Step 9:验证(10 个场景必须全绿)
- [ ] wumin 扫 SFID 登录挑战 → 登录成功
- [ ] wumin 扫 CPMS 登录挑战 → 登录成功
- [ ] wuminapp 发起转账 → 生成 sign_request → wumin 扫签名 → 展示 sign_response → wuminapp 扫 → 广播成功
- [ ] wuminapp 联系人扫码 → 添加成功
- [ ] wuminapp 多签账户码扫码 → 加入成功
- [ ] wuminapp 收款码生成 → 扫码填单成功
- [ ] citizenchain 治理发起转账提案扫 wuminapp 用户码 → 地址填入成功(**现在坏,改完好**)
- [ ] citizenchain 设置手续费地址扫 wuminapp 用户码 → 填入成功(**现在坏,改完好**)
- [ ] wuminapp / wumin 生成的每个 kind JSON 逐字节等于 fixture
- [ ] Dart / TS / Rust 所有测试全绿

## 顺手修的现存 bug

- 治理转账提案收款地址扫码失败(字段名 `to` vs `address` 不一致)
- 设置手续费地址扫码失败(同上)
- 安全基金提案收款地址扫码失败(同上)
- SFID 登录回执字段别名堆叠(`pubkey ?? admin_pubkey ?? public_key`)

这些是字段不一致的自然产物,统一后自动消失,不算新功能。

## 已知残留(不影响功能)

- `citizenchain/node/src/ui/governance/signing.rs`:sign_request display JSON 中仍有 `action_label` 和 `key` 冗余字段。`SignDisplay.fromJson` 解析时自动忽略多余字段，不影响功能。需在下次 citizenchain 代码变更时一并清理。

## 验收标准

1. Step 8 零命中 grep 全通过
2. Step 9 十个场景全绿
3. 所有测试(Dart/TS/Rust)全绿
4. `memory/05-architecture/qr-protocol-spec.md` 与 fixture、与 7 份 body 实现三方对齐
