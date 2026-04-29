# 扫码支付(清算行)手工 E2E 冒烟测试 SOP

- **日期**:2026-04-20
- **范围**:同行 MVP 付款 + 收款的**可复现验证步骤**。目的是:(1) Layer A 协议字节对齐之上再验证真实 wuminapp ↔ 清算行节点 ↔ runtime 全链路;(2) 作为 demo 脚本。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置技术文档**:
  - `STEP2C_I_PAY_PAGE.md`(付款端)
  - `STEP2C_II_A_RECEIVE_QR.md`(收款端)
  - `STEP2C_GOLDEN_VECTORS.md`(协议字节锁)
  - `memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_2_B_INTEGRATION.md`(清算行节点启动)

---

## 0. 前置条件(不在本 SOP 范围内,由运维/SFID Agent 准备)

- citizenchain 链已在运行,能通过 WSS 访问
- SFID 后端已部署,`GET /api/v1/app/clearing-banks/search` 可用
- 至少 1 家清算行已在 SFID 注册 + 链上上链:
  - `main_account`(主账户 SS58)已提交 `register_sfid_institution`
  - 至少 1 名管理员签名密钥已通过节点 `offchain_keystore` 加密落盘
  - 链上 `L2FeeRateBp[bank_main]` 已通过 `propose_l2_fee_rate`(等 7 天 `on_initialize` 激活)或 `set_max_l2_fee_rate` (Root) 配置为 `> 0`(推荐 5 bp)
- 2 个有余额的 L3 账户(各 ≥ 200 元,用于绑定付费 + 充值测试)

若以上任一不满足,先回到运维 / SFID / 链上治理流程,不要在本 SOP 内尝试解决。

---

## 1. 启动清算行节点(无头 CLI 模式)

```bash
cd /Users/rhett/GMB/citizenchain

# 编译(首次或改代码后)
cargo build --release --bin citizenchain

# 启动节点(以清算行角色)
./target/release/citizenchain \
  --base-path /tmp/citizenchain-test \
  --chain citizenchain \
  --rpc-port 9944 \
  --rpc-methods Unsafe \
  --rpc-cors all \
  --no-prometheus \
  --clearing-bank <主账户SS58> \
  --clearing-bank-password '<keystore 密码>' \
  --clearing-reserve-monitor-interval-secs 60
```

### 期望日志关键字

```
[ClearingBank] 签名密钥已解锁
[ClearingBank] 清算行组件已启动,bank_main=<SS58>
[ReserveMonitor] 启动主账对账 interval=60s bank=...
[EventListener] 开始订阅 offchain_transaction 事件
```

**故障排查**
- `签名密钥未加载(密码或密钥文件缺失)` → `--clearing-bank-password` 错 or keystore 文件未落盘 → 回运维
- 启动后立即 `[ReserveMonitor] 对账偏差!` → 本地 ledger 持久化文件与链上 `BankTotalDeposits` 不一致 → 删除 `<base-path>/offchain_step1/ledger.enc` 重启(会重新从事件恢复)

---

## 2. 启动 wuminapp(连清算行节点 + SFID)

```bash
cd /Users/rhett/GMB/wuminapp

flutter run -d <device> \
  --dart-define SFID_BASE_URL=https://<sfid-host>/api
```

### 期望

- App 正常启动,钱包页可见
- 清算行节点端点来自链上 `ClearingBankNodes`;如收款方清算行未声明节点,
  扫码付款页会提示"收款方清算行尚未声明节点"。

---

## 3. 准备两个钱包(付款方 A / 收款方 B)

步骤:**钱包页 → 创建钱包**。创建两次,分别命名 `A` / `B`。

### 前置金额(由运维向钱包转账)

- A:≥ 200 元(本 SOP 会扣 1 元绑定 + 100 元充值 + 10 元付款)
- B:无需预置(收款后自然有余额)

### 观察点

- 钱包页能看到两个钱包,切换时地址不同。
- A/B 各自 `Balance` 字段显示链上余额。

---

## 4. A 绑定清算行(SFR/FFR 主账户)

步骤:**A 钱包详情 → 扫码支付(清算行) → 选择/绑定清算行 → 搜索对应机构 → 确认绑定**。

### 期望观察

- 屏幕 SnackBar:`绑定已提交,tx=0x...,等待链上确认`
- 清算行节点日志:
  - `[EventListener] ... BankBound { user: <A 的 pubkey>, bank: ... }`
- A 钱包 `Balance` 减少约 1 元(`OnchainTxAmountExtractor` 对 `bind_clearing_bank` 定为付费调用)
- 链上查询 `OffchainTransaction::UserBank[A]` = `<bank_main>` 主账户
- `SharedPreferences` 写入 `clearing_bank_shenfen_id_<A.walletIndex>` = 机构 `shenfen_id`

**故障排查**
- `该清算行主账户尚未上链` → 运维先把 `main_account` 上链
- 长时间 `tx=0x...,等待链上确认` 后无反应 → 检查节点 import 日志,可能是 `CheckNonce` 拒绝

---

## 5. A 充值 100 元

步骤:**A 钱包 → 扫码支付(清算行) → 充值 → 输入 100 → 确认**。

### 期望观察

- 屏幕提示:`充值已提交`
- 链上 `pallet_balances::Transfer` 事件:A → `<bank_main>` 10000 分(= 100 元)+ 手续费(按 `0.1%` min 10 分)
- 清算行节点日志:
  - `[EventListener] ... Deposited { user: <A>, bank: <bank_main>, amount: 10000 }`
- 链上 `DepositBalance[<bank_main>, A]` = 10000
- 链上 `BankTotalDeposits[<bank_main>]` 增量 += 10000
- RPC:`offchain_queryBalance(A)` 返回 `10000`
- `ReserveMonitor` tick 应打 `ok local=chain` debug 日志

**故障排查**
- `offchain_queryBalance(A)` 返回 `0` 但链上 `DepositBalance` 有值 → listener 漏事件 → 重启节点(ledger 会从事件流重建)
- `ReserveMonitor 对账偏差!` → 同上

---

## 6. B 生成收款码(10 元)

步骤:**B 钱包 → 扫码支付(清算行) → 生成收款码**。

### 先决条件

B 必须先在 SFID 系统搜索对应清算行并走完步骤 4 的绑定(否则页面只显示"请先绑定清算行")。

实际上 Step 1 同行 MVP:B 绑定的清算行必须与 A 相同;搜 SFID 时选同一家。

### 期望观察

- 页面顶部显示 `收款地址`(B 的 SS58)与 `清算行`(B 的 `shenfen_id`)
- 输入金额 `10`(元)
- QR 码即时渲染,扫出来是 JSON,含:
  - `proto=WUMIN_QR_V1`, `kind=user_transfer`
  - `body.address` = B 的 SS58
  - `body.bank` = B 绑定的清算行 `shenfen_id`
  - `body.amount` = `"10"`
- 页面底部 `可用余额(清算行,每 5s 刷新)` 显示 `0.00 元`(B 还没被转过钱)

---

## 7. A 扫码付款

步骤:**A 钱包 → 主页"扫码支付" → 扫 B 的收款码**。

### 期望观察(页面)

- 跳转到 `OffchainClearingPayPage`
- loading → ready:
  - `收款方地址` = B 的 SS58
  - `收款方清算行` = B 的 `shenfen_id`
  - `金额` 字段自动填 `10`(由 QR 预填,不可再编辑)
  - `费率` 显示 `<L2FeeRateBp>` bp
  - `手续费` = `0.01 元`(5 bp × 10 元 = 0.005 元 → 四舍五入不足 1 分 → 兜底 `MIN_FEE_FEN=1` = 0.01 元)
  - `合计扣款` = `10.01 元`
- 点"确认并签名付款"
- 进入 submitting → done:
  - ✅ 绿色图标 + `支付已受理,清算行会在下一批次上链`
  - 显示 `tx_id: 0x...`

### 期望观察(节点 log)

立即(RPC 接受):
```
无特殊日志(ledger.accept_payment 是内存操作,不打印)
```

30 秒内(packer tick):
```
[ClearingPacker] batch ok tx=0x<批次 tx_id>
```

出块后(listener):
```
[EventListener] ... PaymentSettled { tx_id=0x<本笔 tx_id>, payer=A, recipient=B, ... }
```

---

## 8. 验证最终状态

### 链上查询

- `DepositBalance[<bank_main>, A]` = `10000 - 1000 - 1` = `8999` 分(90.00 - 10.01 = 79.99? 这里算 100 - 10 - 0.01 = 89.99 元 = 8999 分 ✓)
- `DepositBalance[<bank_main>, B]` = `1000` 分(10.00 元)
- `BankTotalDeposits[<bank_main>]` 比付款前减少 `1` 分(= fee,因为同行 fee 流出到 fee_account)
- `pallet_balances::Balance[bank_main]` 减 `1` 分(同行 fee 转出)
- `pallet_balances::Balance[fee_account]` 加 `1` 分

### wuminapp

- A 的收款页或充值页余额查询:`89.99 元`
- B 的收款页:`10.00 元`(5 秒轮询内刷新)

### 节点对账

- `ReserveMonitor` 应打 `ok local=chain=<new_total>` debug
- 偏差日志绝对不应该出现

---

## 9. 常见故障排查

| 现象 | 原因 | 处置 |
|---|---|---|
| 扫码付款页 `请先绑定清算行` | `UserBank[A]` 空 / 本地绑定缓存缺失 | 回步骤 4 确认绑定成功;重新进入清算行设置页刷新缓存 |
| `收款方清算行 ... 未在 SFID 系统查到` | QR 的 `bank` 字段与 SFID 已注册机构不匹配 | 收款方换绑定 → 重生成 QR |
| `收款方清算行尚未声明节点` | 链上 `ClearingBankNodes[bank]` 不存在 | 收款方清算行管理员先在节点端声明清算节点 |
| `清算行费率未配置(rate_bp=0)` | `L2FeeRateBp[bank_main]` = 0 | 由清算行管理员 `propose_l2_fee_rate(bank, 5)` → 等 7 天或 Root `set_max_l2_fee_rate` 后再提 |
| 提交后 30 秒过了 packer 无反应 | keystore 密码不对 → `sign_batch` Err 回滚 | 查节点 `[ClearingPacker]` 日志是否有 rollback;重启节点传正确密码 |
| packer 反复 `TxPool full / Invalid nonce` | 清算行管理员账户 nonce 不对齐(有其他提交在排队) | 等当前队列清空,或用 `system_accountNextIndex` 强制拉一次 |
| `PaymentSettled` 事件到了但 wuminapp 余额不刷新 | wuminapp 5s 轮询失败(WSS 断开) | 检查 WSS 连通;wuminapp 自动下一 tick 恢复 |
| `[ReserveMonitor] 对账偏差!` | listener 漏事件 / ledger 持久化损坏 | 删 `<base-path>/offchain_step1/ledger.enc` 重启(从事件流重建) |

---

## 10. 执行 checklist(建议每次 demo 前核对)

```
[ ] 清算行节点启动日志三件套全有
    · ClearingBank 组件已启动
    · ReserveMonitor 启动
    · EventListener 订阅开始
[ ] wuminapp 启动 + dart-define 两个 URL 正确
[ ] SFID 能搜到目标清算行
[ ] 两个钱包 A/B 都已绑定到同一家清算行
[ ] A 余额 ≥ 110 元(绑定 1 元 + 充值 100 + 手续费冗余)
[ ] packer interval 按默认 30s(或 tick 频率调过)
[ ] reserve_monitor 首次 tick 前不报偏差(启动刚完成跳过)
[ ] 全流程走完 step 8 的链上查询全部与期望一致
```

---

## 11. 后续

- **Layer B 自动化**(1-2 天):把本 SOP 的步骤 4-8 编成 Rust 集成测试(需 `chain_spec.rs` 加 dev preset + SFID mock + 清算行机构 seed)。本 SOP 可作为测试 case 清单。
- **Demo 脚本衍生**:用本 SOP 第 4-8 步录屏,作为产品演示素材;步骤 9 常见故障可直接用于产品侧话术。
- 失败 case 补充:实际执行中遇到但本文未覆盖的故障,追加到第 9 节。

---

## 12. 变更记录

- 2026-04-20:首版落地。基于 Step 2c-ii-a 完成点写就。范围聚焦 wuminapp ↔ 清算行节点 ↔ runtime 的付款/收款交互;创建清算行机构 / 配置费率 / 注册 SFID 等前置环境准备明确列为 out-of-scope。
