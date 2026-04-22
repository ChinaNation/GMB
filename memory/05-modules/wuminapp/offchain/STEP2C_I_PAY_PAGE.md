# 扫码支付 Step 2c-i 技术说明 · wuminapp 付款端新版

- **日期**:2026-04-20
- **范围**:wuminapp 扫码付款页重写(同行 MVP);对接清算行节点
  `offchain_submitPayment` 新 RPC;配套 2 个前置查询 RPC;删除老 dart 文件。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_IV_A_CLEANUP.md`(老节点代码 + wuminapp 老入口下架)
- **后续**:`STEP2C_II_RECEIVE_QR.md`(收款码与 WUMIN_QR_V1 协议完整化)与
  `STEP2C_III_COLD_WALLET.md`(冷钱包扫签两段握手)

---

## 1. 本步目标

Step 2b-iv-a 把老省储行清算代码从节点和 wuminapp 的 onchain 入口下架后,用户扫
商户收款码只会看到"暂不可用"提示。本步接通新清算行体系:**wuminapp 付款端**
经扫码 → 构造 `NodePaymentIntent` → 本地 sr25519 签名 → 调清算行节点
`offchain_submitPayment` → 显示 tx_id;runtime + node 链路(Step 2b-iii-b 已
落地)自动把这笔支付打包上链并清理 pending。

**只做**同行(`payer_bank == recipient_bank`)+ 热钱包。跨行(Step 3)与冷钱包
(Step 2c-iii)在校验层拦住并提示。

---

## 2. 改动清单

### 2.1 节点侧(`citizenchain/node/src/offchain/rpc.rs`)

新增 2 个只读 RPC + 2 个 storage key helper:

| RPC | 入参 | 返回 | 数据源 |
|---|---|---|---|
| `offchain_queryUserBank` | `user: AccountId32` | `Option<AccountId32>` | 链上 `OffchainTransactionPos::UserBank[user]` |
| `offchain_queryFeeRate` | `bank: AccountId32` | `FeeRateResp { rate_bp: u32, min_fee_fen: u128 }` | 链上 `OffchainTransactionPos::L2FeeRateBp[bank]` + 节点常量 `MIN_FEE_FEN=1` |

`OffchainClearingRpcImpl` 新增 `client: Arc<FullClient>` 字段(通过
`start_clearing_bank_components` 额外参数注入)用于 `client.storage()` 单点读。
`rate_bp == 0` 时上报给调用方"费率未配置",节点不代为补缺省。单元测试 5 个:
storage key 布局稳定性 + 不同 bank 不同 key + hex 编解码 roundtrip + 奇数长度
拒绝(功能回归留 dev-chain 集成测试覆盖)。

### 2.2 节点侧(`citizenchain/node/src/offchain/mod.rs` + `service.rs`)

- `start_clearing_bank_components` 参数追加 `client: Arc<FullClient>`,转交给
  `OffchainClearingRpcImpl::new`。`start_clearing_bank_components_with_noop`
  同步追加。
- `service.rs::new_full` 清算行启动块调用处加 `client.clone()`,无新 CLI
  参数。对非清算行节点无影响(整个分支不进入)。

### 2.3 wuminapp 侧

#### 新建

| 文件 | 内容 |
|---|---|
| `lib/trade/offchain/payment_intent.dart` | `NodePaymentIntent` 数据类 + SCALE 编码(固定 204 字节)+ `signingHash()` + 随机 tx_id + `calcFeeFen` 本地费用计算(与 runtime 四舍五入对齐)+ hex 编解码 |
| `lib/trade/offchain/offchain_clearing_pay_page.dart` | **新付款确认页**。5 阶段状态机:loading → ready → submitting → done/error。流程见第 3 节 |

#### 修改

| 文件 | 变更 |
|---|---|
| `lib/rpc/offchain_clearing.dart` | 追加 `queryUserBank` / `queryFeeRate` / `submitPayment` 3 个方法(WSS over JSON-RPC 的 record 返回值) |
| `lib/trade/onchain/onchain_trade_page.dart` | `_openOffchainPay` 恢复扫码后跳转到新页面(之前 2b-iv-a 临时下架为 SnackBar);通过 `String.fromEnvironment` 读 `SFID_BASE_URL` 与 `CLEARING_NODE_WSS`,与 `wallet_page._openClearingPaymentEntry` 同口径 |

#### 删除(老省储行清算遗留)

| 文件 | 原因 |
|---|---|
| `lib/rpc/offchain.dart` | 老 `submitSignedTx` / `queryTxStatus` / `queryInstitutionRate` 客户端,节点侧 Step 2b-iv-a 已删对应 RPC |
| `lib/trade/offchain/offchain_pay_page.dart` | 老付款页,已由 `offchain_clearing_pay_page.dart` 替代 |

---

## 3. 付款页运行时流程

```
用户扫商户 QR → onchain_trade_page._openOffchainPay
  ├─ 校验:未选钱包 / 无 bank 字段 / 未配置 WSS → SnackBar 并返回
  └─ 跳转 OffchainClearingPayPage

OffchainClearingPayPage 初始化(_loadPrerequisites):
  1. node.queryUserBank(user)       → payer_bank SS58
     └─ null:"请先绑定清算行" + pop
  2. sfid.searchClearingBanks(keyword=qrBank) → recipient_bank main_account hex
     └─ 未上链 / 未查到:错误态
  3. payer_bank(hex) == recipient_bank(hex) ?
     └─ 否:"Step 1 仅支持同行"(跨行 Step 3)
  4. node.queryFeeRate(payer_bank)   → rate_bp / min_fee_fen
     └─ rate_bp == 0:"费率未配置"
  5. ChainRpc().fetchLatestBlock()   → currentBlockNumber (for expires_at)

UI(ready):
  金额输入框(或 QR 预填)+ 自动显示手续费 + 合计 + 收款地址 + 备注

用户点"确认并签名付款":
  6. node.queryNextNonce(user)
  7. intent = NodePaymentIntent{
       tx_id = Random.secure 32B,
       payer = wallet.pubkeyHex,
       payer_bank = ss58ToBytes(payer_bank),
       recipient = _decodeAccount(qr.toAddress),    // 兼容 SS58 / 0x hex
       recipient_bank = hexToBytes(recipient_bank_hex),
       amount = amount_fen,
       fee = calcFeeFen(amount, rate_bp, min_fee_fen),
       nonce,
       expires_at = currentBlockNumber + 100,        // ~10 分钟缓冲
     }
  8. digest = blake2b_256("GMB_L3_PAY_V1" ++ scaleEncode(intent))
  9. WalletManager.authenticateForSigning()
       + signWithWalletNoAuth(walletIndex, digest)  → 64B sig
  10. node.submitPayment(intentHex, sigHex) → { tx_id, l2_ack_sig, accepted_at }
  11. 显示完成态 + 可复制 tx_id + "完成"按钮
```

---

## 4. PaymentIntent 跨端字节一致性(最重要的回归风险)

`NodePaymentIntent` 的 SCALE 布局与 Dart 侧 `scaleEncode()` **必须逐字节对齐**,
否则节点 `sr25519_verify` 必败。本步以**定长 204 字节**锁死:

```
[0..32)   tx_id          H256,            raw
[32..64)  payer          AccountId32,     raw
[64..96)  payer_bank     AccountId32,     raw
[96..128) recipient      AccountId32,     raw
[128..160) recipient_bank AccountId32,    raw
[160..176) amount         u128 little-endian
[176..192) fee            u128 little-endian
[192..200) nonce          u64  little-endian
[200..204) expires_at     u32  little-endian
```

签名域:`b"GMB_L3_PAY_V1"`(13 字节 ASCII,`payment_intent.dart::signingDomain`
常量与 runtime `offchain_transaction_pos::batch_item::L3_PAY_SIGNING_DOMAIN`
一致)。

**若 runtime 侧新增/删除 `NodePaymentIntent` 字段**:必须同改 node `ledger.rs`、
Dart `payment_intent.dart` 布局注释 + 长度 assert 常量、本文档第 4 节表格。
任何一处疏漏,端到端签名立刻失败。

---

## 5. 编译验证

```
$ cd citizenchain && WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node --tests
(仅 Tauri `frontend/dist` proc macro 门禁;其他零 error)

$ cd wuminapp && flutter analyze
No issues found!  (全项目)
```

---

## 6. 已知风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| `NodePaymentIntent` 字段变动 → 跨端布局错位 → 签名失败 | **P0** | 本文档第 4 节与 `payment_intent.dart` 注释双处锁布局;`scaleEncode` 末尾 `assert(bytes.length == 204)` 立即抛 |
| `expires_at` 设为 `currentBlockNumber + 100`,签名到节点打包约 30s,但用户看到确认页到点提交可能>10min | **P2** | 100 块 ≈ 10 分钟(6s 出块)。极慢用户可能遇到 `ExpiredIntent`,此时报错提示重扫;后续可把缓冲改为 200 块或查询 runtime `target_block_time_ms` 动态算 |
| 费率查询 / 提交走独立 WSS 连接,RTT 叠加 | **P2** | 每次方法连→发→收首帧→断开,确认流程总 5 次 RTT;Step 2c-ii 考虑复用长连接 |
| 跨行(`payer_bank != recipient_bank`)在提交前就被 wuminapp 拦住,但若绕过(比如 QR 造假),节点 `accept_payment` 不检清算行匹配 | **P1** | 节点依赖后续 on-chain `settlement::execute_clearing_bank_batch` 的 `payer_bank == institution_main` 校验拒绝;付款方 L3 会在 packer 提交后的下一批 revert 时发现失败。Step 3 修 runtime 侧校验同步到 node 层 RPC 早拒 |
| `offchain_queryFeeRate` 返回 `rate_bp==0` 时 UI 仅显示错误,用户体验欠缺 | **P3** | 本步先 hard-fail 提示联系运维,后续可引导到"查看清算行详情"页面 |
| 冷钱包 `isHotWallet==false` 直接 SnackBar 拒绝 | **P2** | Step 2c-iii 通过 QR 往返签名闭合 |
| `SFID_BASE_URL` / `CLEARING_NODE_WSS` 走 `String.fromEnvironment` 占位 | **P3** | 与 `wallet_page._openClearingPaymentEntry` 同口径,Step 2c-ii 统一配置中心 |

---

## 7. 不做(留后续)

- **Step 2c-ii**:`receive_qr_page` 实时余额推送 + WUMIN_QR_V1 协议规范化(统一
  商户码格式)
- **Step 2c-iii**:冷钱包扫签(热→冷 sign request QR + 冷→热 sign response QR)
- **Step 3**:跨行扫码(含 `recipient_bank` 主账户 SS58 的二次验证 + 节点侧
  `accept_payment` 早拒)
- **提交成功后本地历史记录**:暂不写入本地 `LocalTxStore`(老路径用,新清算行
  历史改由订阅 `PaymentSettled` 事件沉淀,Step 2c-ii 一并实现)

---

## 8. 变更记录

- 2026-04-20:Step 2c-i 完整落地。节点侧新增 2 个只读 RPC(`queryUserBank` /
  `queryFeeRate`)+ `OffchainClearingRpcImpl` 持 `client`;wuminapp 新增
  `payment_intent.dart`(SCALE 编码 + 签名哈希)+ `offchain_clearing_pay_page.dart`
  (5 阶段状态机)+ `offchain_clearing.dart` 3 方法;删除 `lib/rpc/offchain.dart`
  和 `lib/trade/offchain/offchain_pay_page.dart` 两个老文件;`onchain_trade_page`
  恢复扫码跳转。`cargo check -p node --tests` 零 error;`flutter analyze` 零 issue。
