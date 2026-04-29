# 扫码支付 Step 2c-ii-a 技术说明 · wuminapp 收款端基础

- **日期**:2026-04-20
- **范围**:新建清算行**收款 QR 页**(与老 `ReceiveQrPage` 并存),`bind_clearing_bank_page`
  绑定成功写本地 shenfen_id 缓存,收款页读缓存并轮询余额。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2C_I_PAY_PAGE.md`(付款端新版)
- **后续**:`STEP2C_II_B_BALANCE_SUBSCRIBE.md`(WS 订阅 `PaymentSettled` 实时推送,
  取代当前 5 秒轮询)与 `STEP2C_III_COLD_WALLET.md`(冷钱包两段握手)

---

## 1. 本步目标

Step 2c-i 付款端可用后,demo 闭环缺收款端:商户扫不到"带 `bank` 字段"的 QR,
付款方无处扫码。本步给:
- 收款方生成含新清算行 `shenfen_id` 的 QR 码
- 生成 QR 的前置条件:知道收款方自己绑的清算行 `shenfen_id`
- 实时展示收款方清算行余额(到账后刷新)

端到端同行 MVP 至此**在 app 层可演示**。

---

## 2. 关键决策:`shenfen_id` 从哪里来?

链上 `OffchainTransaction::UserBank[user]` 存的是清算行**主账户** `AccountId32`
(32 字节),**不是** SFID `shenfen_id` 字符串。但收款 QR 里写的是 `shenfen_id`
(付款方用 SFID 公开 API 反查主账户做同行校验)。从 `AccountId32` 反查
`shenfen_id` 需要 SFID 后端配一个 "按主账户 hex 精确查" 端点,SFID 公开
`searchClearingBanks` 只支持 keyword 模糊匹配。

**本步务实方案**:`ClearingBankPrefs`(`SharedPreferences` 封装,key:
`clearing_bank_shenfen_id_{walletIndex}`):
- **写入**:`bind_clearing_bank_page` 链上绑定成功后立即 `save(walletIndex, sfidId)`
- **读取**:`offchain_clearing_receive_page` 初始化时 `load(walletIndex)`,为
  `null` 时提示"请先绑定"
- **失去缓存的处置**(重装 / 清数据 / CLI 或别机绑定):提示重新从"选择/绑定
  清算行"入口走一遍(即使链上已绑,重走到确认步会同步写回缓存)

**后续升级路径**(Step 3):SFID 后端补 `GET /api/v1/app/clearing-banks/by-main-account?address=`
后,收款页优先走 RPC 反查,失败再退回缓存。

---

## 3. 改动清单

### 3.1 新建

| 文件 | 内容 |
|---|---|
| `lib/trade/offchain/clearing_bank_prefs.dart` | 4 个静态方法 `save` / `load` / `clear`(按 walletIndex 隔离);空串等价于清除 |
| `lib/trade/offchain/offchain_clearing_receive_page.dart` | 新收款页:WalletProfile + 可选 `clearingNodeWssUrl`;读 prefs → 生成 `WUMIN_QR_V1 kind=user_transfer` QR(`address` / `name` / `amount` / `memo` / `bank=shenfen_id`);每 5 秒轮询 `offchain_queryBalance(user)` 刷新余额 |
| `test/trade/clearing_bank_prefs_test.dart` | 5 个单测(空 load / 双 walletIndex roundtrip / 空串等价清除 / 选择性 clear / 覆盖写入)。全部通过 |

### 3.2 修改

| 文件 | 变更 |
|---|---|
| `lib/trade/offchain/bind_clearing_bank_page.dart` | 绑定成功后 `await ClearingBankPrefs.save(wallet.walletIndex, widget.bank.sfidId)`,紧接 SnackBar 提示 |
| `lib/trade/offchain/clearing_payment_entry_page.dart` | 追加"生成收款码"入口,落 `OffchainClearingReceivePage`;提示文本同步更新(付款入口仍在主页扫码;老省储行页面 ADR-006 已退出) |

### 3.3 不动(保留老路径)

- `lib/wallet/ui/receive_qr_page.dart`:老省储行 `ReceiveQrPage`,`bankShenfenId`
  从 `OnchainRpc.queryClearingInstitution`(老 `bind_clearing_institution` call_index 9)
  取。与新 `offchain_clearing_receive_page` 并存,不冲突。Step 2b-iv-b runtime
  老 Calls 清理时再下架老页面 + 老 `wallet_page._openReceiveQr` 入口重写。

---

## 4. QR 载荷

`WUMIN_QR_V1 kind=user_transfer` envelope,body 为 `UserTransferBody`:

```json
{
  "address": "5GrwvaEF5zXb...",           // 收款方 SS58
  "name":    "钱包 A",                    // WalletProfile.walletName
  "amount":  "100",                       // 商户预填元,可空
  "symbol":  "GMB",
  "memo":    "午餐",
  "bank":    "SFR-GD-SZ01-CB01-N9-D8"      // 收款方清算行 shenfen_id
}
```

付款方 `offchain_clearing_pay_page`(Step 2c-i)扫到后:
- `toAddress = body.address`
- `recipientBankShenfenId = body.bank`
- `initialAmountYuan = body.amount`
- `memo = body.memo`

SFID `searchClearingBanks(keyword=recipientBankShenfenId)` 反查主账户,同行校
验 + 费率查询 + 签名提交。

---

## 5. 运行时流程

```
用户打开"扫码支付(清算行)" → ClearingPaymentEntryPage
  └─ "生成收款码" → OffchainClearingReceivePage(wallet, wssUrl?)

init:
  ClearingBankPrefs.load(walletIndex)
    ├─ null → "请先绑定清算行" + 返回按钮
    └─ "SFR-GD-..." → 渲染 QR + 启动余额轮询 Timer(5s)

每次输入金额/备注:
  _buildQrData() → QrEnvelope.toRawJson → 重绘 QR

余额 Timer tick:
  offchain_queryBalance(wallet.address)
    → setState(_balanceFen)
    └─ WSS 失败 → setState(_balanceError)(不 crash)

dispose:
  Timer.cancel() + controllers.dispose()
```

---

## 6. 编译验证

```
$ cd wuminapp && flutter analyze
No issues found!  (全项目)

$ flutter test test/trade/clearing_bank_prefs_test.dart
All tests passed!  (5 个)
```

---

## 7. 已知风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| `ClearingBankPrefs` 缓存失效(重装 / 清数据 / CLI 绑 / 别机绑) | **P2** | 收款页显示"请先绑定";重走绑定流程会同步写回。Step 3 SFID 加反查 API 自动回填 |
| 5 秒轮询浪费 RTT,用户停留页面越久浪费越多 | **P2** | `_balanceInFlight` 闸住重入;Step 2c-ii-b WS subscribe 取代 |
| 轮询失败时展示 SnackBar 不友好 | **P3** | 本步改为原地 `_balanceError` 文案 + 黄色图标;下次 tick 自动恢复 |
| `_buildQrData()` 每次 setState 重算,QR 渲染每输一字符跑一次 `QrCode.fromData` | **P3** | 金额/备注输入量小,实测无卡顿;性能成问题时 debounce 300ms |
| `_QrPainter` 无中央留白,和老 `ReceiveQrPage` 样式不同 | **P3** | Step 2c-ii-b 统一视觉;当前 MVP 优先功能 |
| 老 `ReceiveQrPage` + 新 `OffchainClearingReceivePage` 并存,同一钱包可能同时绑了老省储行 + 新清算行(两个完全独立的 bank 缓存) | **P2** | Step 2b-iv-b runtime 清老 Calls + `wallet_page._openReceiveQr` 重定向到新页后自然收敛 |
| `bind_clearing_bank_page` 调 `ClearingBankPrefs.save` **在**上链提交后 immediately,链上可能尚未最终确认;若 reorg 导致绑定被回滚,缓存仍指向未生效的清算行 | **P1** | Step 1 同行 MVP 下, reorg 概率极低;严格起见应改为监听 `UserBank` 变化后再写缓存(Step 2c-ii-b 与 listener 事件订阅同时接入) |

---

## 8. 不做(留后续)

- **Step 2c-ii-b**:WS 订阅 `PaymentSettled` 事件推送,取代轮询;收到支付时 Toast
  通知 + 动画高亮
- **Step 2c-iii**:冷钱包扫签两段握手,让冷钱包也能付款
- **Step 3**:SFID 后端补"按主账户反查 shenfen_id"API,移除本步本地缓存依赖;
  加历史收款列表(从 `PaymentSettled` 事件构建)
- **UI 精修**:中央 logo 留白 / 保存到相册 / 分享 / 识别粘贴板

---

## 9. 变更记录

- 2026-04-20:Step 2c-ii-a 完整落地。新增 `clearing_bank_prefs.dart` 小工具 +
  5 个单测(全绿);新增 `offchain_clearing_receive_page.dart`(QR 生成 + 5s 轮询
  余额);`bind_clearing_bank_page` 绑定成功后写缓存;`clearing_payment_entry_page`
  追加"生成收款码"入口。`flutter analyze` 零 issue;单测全通过;wuminapp 其他
  路径不变。
