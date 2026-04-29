# 扫码支付 Step 2c-i 技术说明 · wuminapp 付款端新版

- **日期**:2026-04-20；2026-04-29 补齐 Step 3 跨行与清算行目录
- **范围**:wuminapp 扫码付款页重写;对接清算行节点
  `offchain_submitPayment` RPC、清算行目录查询、绑定缓存与跨行收款方主导支付。
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

2026-04-29 Step 3 补齐后,付款页不再强制同行。扫码得到收款方 `bank`
后,wuminapp 先从链上 `ClearingBankNodes[sfid_id]` 读取收款方清算节点端点,
再把支付意图提交给**收款方清算节点**;页面同时展示付款方清算行、收款方
清算行与同行/跨行状态。冷钱包支付授权仍留给后续 QR 往返签名流程。

---

## 2. 改动清单

### 2.1 节点侧(`citizenchain/node/src/offchain/rpc.rs`)

新增 2 个只读 RPC + 2 个 storage key helper:

| RPC | 入参 | 返回 | 数据源 |
|---|---|---|---|
| `offchain_queryUserBank` | `user: AccountId32` | `Option<AccountId32>` | 链上 `OffchainTransaction::UserBank[user]` |
| `offchain_queryFeeRate` | `bank: AccountId32` | `FeeRateResp { rate_bp: u32, min_fee_fen: u128 }` | 链上 `OffchainTransaction::L2FeeRateBp[bank]` + 节点常量 `MIN_FEE_FEN=1` |

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
| `lib/rpc/clearing_bank_directory.dart` | 新增清算行目录服务:SFID 后端搜索候选机构,链上读取 `ClearingBankNodes` 端点、`UserBank` 绑定 |
| `lib/trade/onchain/onchain_trade_page.dart` | 扫码后按收款方 `bank` 查询链上端点,跳转到新付款页;不再依赖固定启动参数配置清算节点 |
| `lib/trade/offchain/clearing_bank_settings_page.dart` | 设置页从占位页变成真实搜索/当前绑定/绑定或切换入口 |
| `lib/trade/offchain/clearing_bank_prefs.dart` | 本地缓存从单一 sfid 字符串升级为 `ClearingBankBindingSnapshot`,记录 sfid、机构名、主/费用账户与节点端点 |
| `lib/wallet/ui/cards/wallet_action_card.dart` | 钱包卡读取清算行缓存并查询节点余额;未绑定时充值/提现提示先绑定 |

#### 删除(老省储行清算遗留)

| 文件 | 原因 |
|---|---|
| `lib/rpc/offchain.dart` | 老 `submitSignedTx` / `queryTxStatus` / `queryInstitutionRate` 客户端,节点侧 Step 2b-iv-a 已删对应 RPC |
| `lib/trade/offchain/offchain_pay_page.dart` | 老付款页,已由 `offchain_clearing_pay_page.dart` 替代 |

---

## 3. 付款页运行时流程

```
用户扫商户 QR → onchain_trade_page._openOffchainPay
  ├─ 校验:未选钱包 / 无 bank 字段 / 收款方未声明节点 → SnackBar 并返回
  └─ 跳转 OffchainClearingPayPage

OffchainClearingPayPage 初始化(_loadPrerequisites):
  1. node.queryUserBank(user)       → payer_bank SS58
     └─ null:"请先绑定清算行" + pop
  2. sfid.searchClearingBanks(keyword=qrBank) → recipient_bank main_account hex
     └─ 未上链 / 未查到:错误态
  3. node.queryFeeRate(recipient_bank) → rate_bp / min_fee_fen
     └─ rate_bp == 0:"费率未配置"
  4. ChainRpc().fetchLatestBlock()   → currentBlockNumber (for expires_at)

UI(ready):
  金额输入框(或 QR 预填)+ 自动显示手续费 + 合计 + 收款地址 + 付款方/收款方清算行 + 备注

用户点"确认并签名付款":
  5. node.queryNextNonce(user)
  6. intent = NodePaymentIntent{
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
  7. digest = blake2b_256("GMB_L3_PAY_V1" ++ scaleEncode(intent))
  8. WalletManager.authenticateForSigning()
       + signWithWalletNoAuth(walletIndex, digest)  → 64B sig
  9. node.submitPayment(intentHex, sigHex) → { tx_id, l2_ack_sig, accepted_at }
  10. 显示完成态 + 可复制 tx_id + "完成"按钮
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
常量与 runtime `offchain_transaction::batch_item::L3_PAY_SIGNING_DOMAIN`
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
| 跨行支付提交到收款方节点后,付款方余额/nonce 不在收款方本地 ledger | **已修复** | node RPC 会读链上 `DepositBalance[payer_bank][payer]` 与 `L3PaymentNonce[payer]`,并叠加本节点 pending 做早拒,不创建付款方 ghost 账户 |
| `offchain_queryFeeRate` 返回 `rate_bp==0` 时 UI 仅显示错误,用户体验欠缺 | **P3** | 本步先 hard-fail 提示联系运维,后续可引导到"查看清算行详情"页面 |
| 冷钱包 `isHotWallet==false` 直接 SnackBar 拒绝 | **P2** | Step 2c-iii 通过 QR 往返签名闭合 |
| `SFID_BASE_URL` 仍走 `String.fromEnvironment` | **P3** | 本地默认 `http://127.0.0.1:8080`;清算节点端点已改为链上 `ClearingBankNodes` 真源 |

---

## 7. 不做(留后续)

- **Step 2c-ii**:`receive_qr_page` 实时余额推送 + WUMIN_QR_V1 协议规范化(统一
  商户码格式)
- **Step 2c-iii**:冷钱包扫签(热→冷 sign request QR + 冷→热 sign response QR)
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
- 2026-04-29:Step 3 补齐收款方主导跨行支付。新增
  `clearing_bank_directory.dart`,设置页真实搜索并缓存节点端点;扫码付款按收款方
  `ClearingBankNodes` 选择节点,付款页放开跨行并按收款方费率计费;钱包卡接入
  绑定余额、充值、提现入口。`flutter analyze` 与清算行相关 widget/prefs 测试通过。
