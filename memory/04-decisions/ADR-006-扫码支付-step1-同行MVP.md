# ADR · 扫码支付清算体系 Step 1(同清算行内 MVP)

- **日期**:2026-04-19
- **状态**:Accepted
- **决策者**:GMB AI 主入口
- **范围**:citizenchain runtime / node,sfid backend,wuminapp
- **任务卡**:`memory/08-tasks/open/20260419-扫码支付-step1-同行MVP.md`

---

## 1. 背景

为支撑亿级 L3 高频小额支付,在链上 PoW 基础上新建**扫码支付清算体系**。Step 1 先交付"同一清算行内 L3↔L3 扫码支付"的最小闭环,不做跨清算行/费率治理/仲裁,后续由 Step 2、Step 3 补齐。

## 2. 四层架构(不变)

- **L0** citizenchain PoW 链
- **L1** 省储行(借贷、监督、仲裁,**退出日常清算**)
- **L2** 清算行(商业银行支行/第三方支付机构,**承担所有清算**)
- **L3** 轻节点(C/B 端,wuminapp)

## 3. 核心决策

### 3.1 清算行身份

- A3 必须是 **SFR(私法人)**或 **FFR(非法人)**两者之一 → 私权机构
- 每个清算行在 SFID 系统注册时一次性生成:
  - `sfid_id`
  - 主账户地址 = `blake2b_256("GMB_DUOQIAN_V1" || sfid_id || "主账户" || nonce_1)[0..32]`
  - 费用账户地址 = `blake2b_256("GMB_DUOQIAN_V1" || sfid_id || "费用账户" || nonce_2)[0..32]`
- **管理员数量 N 由链上 `propose_create` 时写入**,阈值 T ≥ ⌈N/2⌉,非固定 9
- 清算行在链上两步注册(SFID 颁证 + duoqian-manage),**不经省储行审批**

### 3.2 L3 绑定

- `bind_clearing_bank` = **绑定即开户,无预存,无业务开户费**
- 链上只产生"付费调用交易 1 元/次"(沿用现有 `OnchainTxAmountExtractor` 分类规则)
- 同时只能绑定一家清算行;自由切换(前置:余额清零),无次数/时间间隔限制
- 绑定对象必须是**主账户**(不能绑费用账户)

### 3.3 存款模型

- 链上权威存储(双 map):`DepositBalance<主账户, L3> → 余额(分)`
- 链上总额冗余:`BankTotalDeposits<主账户> → 总存款`
- 清算行节点本地 ledger 为缓存,从链上事件同步
- `deposit` / `withdraw` 走 L3 自持账户 ↔ 清算行主账户

### 3.4 扫码支付(本步仅同行)

- 前置铁律(无回落):
  - 付款方 L3 必须已绑定清算行
  - 收款方 L3 必须已绑定清算行
  - 收款方清算行 DuoqianAccount Active
- 二维码遵循 `memory/05-architecture/qr-protocol-spec.md` 的 `WUMIN_QR_V1 + user_transfer`,`body.bank` 填**收款方绑定的清算行主账户 SS58**
- 每笔 L3 私钥 sr25519 签名 `PaymentIntent`,防重放通过 `L3PaymentNonce` 单调 + `ProcessedOffchainTx`
- 费率 Step 1 **全局硬编码 5 bp(0.05%)**,最低 1 分;Step 2 改为 `L2FeeRateBp` Storage + 延迟生效
- 手续费**全部归收款方清算行费用账户**,无省储行分成
- 本步仅支持**同一清算行内** L3↔L3 支付,跨行在 Step 2

### 3.5 省储行的新定位

- 不审批清算行注册 / 不颁发许可 / 不暂停清算行
- 仅做:借贷给清算行、监督偿付(Step 2)、仲裁(Step 3)

## 4. 模块边界与代码组织

### 4.1 不新增 pallet

扫码支付全部逻辑合入现有 `citizenchain/runtime/transaction/offchain-transaction/`,按子文件拆分:

```
offchain-transaction/src/
├── lib.rs           # pallet 入口,聚合 Storage/Events/Errors/Calls
├── batch_item.rs    # Step 1 新增:PaymentIntent + 未来批次结构
├── bank_check.rs    # Step 1 新增:A3 ∈ {SFR,FFR} + Active 判定
├── deposit.rs       # Step 1 新增:bind/deposit/withdraw/switch_bank 实现
├── nonce.rs         # Step 1 新增:L3PaymentNonce 辅助
├── settlement.rs    # Step 2 起:execute_batch 重写(Step 1 暂不启用)
├── weights.rs       # 现有
└── benchmarks.rs    # 现有
```

### 4.2 与现有模块的边界

- `duoqian-manage`:**不动**,负责机构多签账户的链上注册
- `duoqian-transfer`:**不动**,负责机构多签账户之间的转账
- `onchain-transaction`:**不动**,链上支付独立入口
- `sfid-system`:**不动**
- `institution-asset`:Step 1 微调新增 4 个 Action(L3DepositIn / L3WithdrawOut / L2ClearingDebit / L2FeeCollect)

### 4.3 现有 lib.rs 中的旧逻辑处置

现有 `submit_offchain_batch / enqueue_offchain_batch / execute_batch / RecipientClearingInstitution / InstitutionRateBp` 等**保留运行**但属于"省储行清算模型"。Step 1 **不触碰、不删除**,作为 Step 2 的重写起点。Step 1 新增的 Storage 和 Call 独立于旧逻辑,不产生冲突。

### 4.4 节点层(citizenchain/node/src/)

Step 1 新建目录 `citizenchain/node/src/offchain/`(本步不立即删除旧 `offchain_*.rs`,标 deprecated,Step 2 删除):

```
node/src/offchain/
├── mod.rs               # 聚合 + 启动器
├── ledger.rs            # 本地账本(缓存)
├── rpc.rs               # 对 wuminapp RPC + WS
├── packer.rs            # 批次打包(Step 2 启用批次上链)
└── event_listener.rs    # 监听链上事件同步本地
```

## 5. 本步**明确不做**的项

| 项 | 归属 |
|---|---|
| 清算行间 libp2p gossip | Step 2 |
| 跨清算行 execute_batch 分账 | Step 2 |
| `L2FeeRateBp` Storage + 费率治理 | Step 2 |
| 偿付能力自动保护 | Step 2 |
| `dispute` 模块 + `raise_dispute` | Step 3 |
| `BankReserve` 保证金 | Step 3 |
| `close_clearing_bank` | Step 3 |
| 删除旧 `offchain_*.rs`(ledger/packer/gossip) | Step 2 完成后 |
| 重写现有 `execute_batch`(省储行分账) | Step 2 |

## 6. 手续费规则(对齐白皮书 + 20260401 付费调用任务卡)

| 交易 | 分类 | 链上费 |
|---|---|---|
| `bind_clearing_bank` / `switch_bank` | 付费调用 | **1 元/次**,8:1:1 分 |
| `deposit` / `withdraw` | 链上资金交易 | 金额 × 0.1%,最低 0.1 元 |
| `submit_offchain_batch`(Step 2 启用) | 链下资金交易 | sum(fee) × 0.1%,最低 0.1 元 |

## 7. 兼容性

- `chainspec.json` **不改动**
- 无需 runtime 升级(Step 1 的 pallet 变更是 storage 版本号递增 + 新增 call_index,做一次常规 setCode)
- 本步与旧省储行清算模型**并存**,L3 通过新 `bind_clearing_bank` 进入新模型,旧模型逐步退出(Step 2 完成)

## 8. 验收摘要

- Runtime 单元测试全部通过
- L3 绑定招商银行深圳福田支行(示例清算行)→ 充值 → 扫码付同一清算行内另一 L3 → 提现 → 切换
- 手续费自动进入该清算行费用账户
- 跨清算行场景返回 `CrossBankNotSupportedInMvp`

## 9. 后续

Step 2 承接:跨行清算、费率治理、偿付保护、节点旧文件删除。
Step 3 承接:争议仲裁、保证金、清算行主动退出、白皮书 5.4.4 发布。

---

## 变更记录

- 2026-04-19:初版,Step 1 范围定稿。
