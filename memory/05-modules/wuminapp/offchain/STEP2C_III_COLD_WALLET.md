# 扫码支付 Step 2c-iii 技术说明 · 冷钱包扫签落地

- **日期**:2026-04-20
- **范围**:wuminapp 扫码付款页把签名步骤抽象为 `热钱包直签 | 冷钱包两段 QR 握手` 分支;冷钱包路径复用现有 `QrSigner` + `QrSignSessionPage` 基础设施,无需改动 wumin 冷钱包 app。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2C_I_PAY_PAGE.md`(付款端 v1,仅热钱包)+ `STEP2C_II_A_RECEIVE_QR.md`(收款端)+ `STEP2C_GOLDEN_VECTORS.md`(字节对齐锁)
- **后续**:Layer B 自动化 E2E / Step 2b-iv-b runtime 清理 / 跨行 ghost account 修复

---

## 1. 本步目标

Step 2c-i 付款页对冷钱包 `isColdWallet` 直接 SnackBar 拒绝,MVP 只覆盖热钱包用户。本步用与 `bind_clearing_page.dart` 同款模式,让冷钱包也能签 `PaymentIntent`。

### 为什么冷钱包可盲签

- 冷钱包 wumin app 扫 `sign_request` 后,显示 `display` 字段(action / summary / fields)让用户目视确认
- 用 sr25519 私钥对 `payload_hex`(本步 = `NodePaymentIntent.signingHash()` 32 字节)做签名,返回 64 字节签名 hex
- 冷钱包**不需要知道 payload 是什么结构**,只处理字节流
- 节点侧 `offchain_submitPayment` 拿到热钱包转发的 64 字节签名 + intent 原文,`sr25519_verify(signature, signing_hash, payer_pubkey)` 通过

所以 Step 2c-iii 只改 wuminapp 付款页,**wumin 冷钱包 app 零改动**。

---

## 2. 改动清单

### 2.1 `lib/trade/offchain/offchain_clearing_pay_page.dart`

| 变更 | 说明 |
|---|---|
| 新 `import` | `qr/bodies/sign_request_body.dart`(`SignDisplay` / `SignDisplayField`)、`qr/pages/qr_sign_session_page.dart`、`signer/qr_signer.dart`(`QrSigner` / `SignResponseEnvelope`) |
| 删 `isHotWallet` 短路 | `_confirmAndSubmit` 开头的 "冷钱包扫码支付暂未开放" 拒绝去掉 |
| 抽 `_signSigningHash(signingHash, amountFen, feeFen)` | 热钱包:`WalletManager.authenticateForSigning` + `signWithWalletNoAuth`;冷钱包:`QrSigner.buildRequest(payloadHex=signingHash.hex) → encodeRequest → QrSignSessionPage` → 接 `SignResponseEnvelope` → 提签名 |
| `SignDisplay` 字段 | `action=offchain_clearing_pay` / `summary=清算行扫码付款 X 元 → 收款方` / 4 个 fields(金额/手续费/合计/收款方/收款清算行) |
| 其他改动 | 无(费率查询 / intent 构造 / RPC 提交 / 结果展示全部复用) |

### 2.2 不改动

- `offchain_clearing_receive_page.dart`:收款端无需冷钱包签名
- wumin 冷钱包 app:盲签字节流,无需适配新 payload 格式

---

## 3. 运行时流程

### 3.1 热钱包(行为与 Step 2c-i 一致)

```
_confirmAndSubmit
  → _signSigningHash → WalletManager.signWithWalletNoAuth(walletIndex, hash)
  → _nodeRpc.submitPayment
```

### 3.2 冷钱包

```
_confirmAndSubmit
  → _signSigningHash
    ├─ QrSigner.buildRequest(
    │    requestId='offchain-pay-<rand>',
    │    address=wallet.address,
    │    pubkey='0x' + wallet.pubkeyHex,
    │    payloadHex='0x' + hex(signingHash),
    │    specVersion=chain fetched,
    │    display={ action: 'offchain_clearing_pay', summary, fields[5] }
    │  )
    │
    ├─ QrSignSessionPage:
    │    · 顶部显示 sign_request QR(wumin 冷钱包扫)
    │    · 底部扫 sign_response QR(wumin 冷钱包签完后展示)
    │
    └─ 返 SignResponseEnvelope → body.signature (hex) → hexToBytes → 64B sig
  → _nodeRpc.submitPayment(intent_hex, payer_sig_hex)
```

wumin 冷钱包侧(已有能力,本步不改):
1. 扫 wuminapp 展示的 `sign_request` QR
2. 目视确认 `display` 字段(action + summary + 5 个 fields)
3. 用户同意 → 冷钱包 app 用私钥 sr25519 签 `payload_hex` 字节 → 生成 `sign_response` QR 展示
4. wuminapp 在 `QrSignSessionPage` 扫回 `sign_response`

---

## 4. 编译验证

```
$ cd wuminapp && flutter analyze
No issues found!  (全项目)
```

本步无新增单测。Layer A golden vectors 已锁 SCALE + signing_hash,冷钱包对 32 字节 signing_hash 盲签不会动布局;冷钱包路径的集成验证靠手工 SOP(参照 `STEP2C_MANUAL_SMOKE.md` 把热钱包替换为冷钱包重跑第 7 步扫签)。

---

## 5. 已知风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| 冷钱包 display 字段中文长度超过 wumin app 屏幕显示上限 | **P3** | `summary` 含金额 + 地址(32 字节 SS58 较长),实测如有截断再压缩;当前 5 个 fields 足够目视校验 |
| `QrSignSessionPage` 扫回 response 与 request 不对应 | **P2** | `QrSigner.verifyResponse`(既有实现)会校验 `request_id` + `expected_pubkey`,本步依赖其现有保护 |
| 冷钱包签完交回时 `expires_at` 已近逾期 | **P2** | `expires_at = currentBlock + 100` ≈ 10 分钟。冷钱包扫签+确认+回传通常 < 2 分钟,余量充足;超 10 分钟会在 runtime `settlement` 触发 `ExpiredIntent`,用户重扫即可 |
| 冷钱包扫到 `sign_request` 后,若其他 app 协议 clash(如其他 `action` 误识别) | **P3** | `action=offchain_clearing_pay` 是新字符串,与 `bind_clearing` / `onchain_transfer` 等已有 action 不冲突 |

---

## 6. 不做(留后续)

- **Step 3** 跨行:冷钱包 display `fields` 加 "付款方清算行" 供用户目视两端不同行的场景
- **wumin 冷钱包 UI** 针对 `offchain_clearing_pay` action 的差异化展示(当前走通用 `sign_request` 视图,足够)
- 历史记录:提交成功后本地落盘,Step 2c-ii-b 与 WS 订阅一并做

---

## 7. 变更记录

- 2026-04-20:Step 2c-iii 完成。`offchain_clearing_pay_page.dart` 引入 `_signSigningHash` 抽象,热冷钱包分流;冷钱包路径复用 `QrSigner` + `QrSignSessionPage` 既有设施,`SignDisplay.action=offchain_clearing_pay` + 5 fields 细节展示,wumin 冷钱包 app 零改动。`flutter analyze` 零 issue。
