# 链下支付系统完整设计方案：省储行清算 + 银行存取

- **日期**: 2026-04-05
- **版本**: v2.0
- **状态**: 设计评审

---

## 一、系统架构

### 1.1 四方角色

```
┌─────────────────────────────────────────────────────────────┐
│                        公民币区块链                           │
│                     （共识层 / 结算层）                        │
└─────────┬───────────────────────────────────┬───────────────┘
          │                                   │
    ┌─────┴─────┐                       ┌─────┴─────┐
    │  省储行 ×43 │                       │  省储行 ×43 │
    │  （清算中心）│                       │  （清算中心）│
    └─────┬─────┘                       └─────┬─────┘
          │                                   │
    ┌─────┼─────┐                       ┌─────┼─────┐
    │     │     │                       │     │     │
  银行A  银行B  银行C                   银行D  银行E  银行F
  （存取网点）                           （存取网点）
    │     │                               │     │
  用户们  用户们                          用户们  用户们
```

| 角色 | 数量 | 身份类型 | 核心职责 |
|------|------|---------|---------|
| **省储行** | 43 个（每省 1 个） | 制度机构（china_ch.rs 硬编码） | 清算打包上链、跨行结算、费率治理 |
| **银行** | 不限 | 注册多签机构（链上注册+省储行许可） | 接收用户存款、处理用户付款/提现、维护用户账本 |
| **付款用户** | 不限 | 链上地址 | 在银行开户充值，扫码付款 |
| **收款用户** | 不限 | 链上地址 | 在银行开户，接收清算入账 |

### 1.2 与传统银行体系的对应

| 公民币区块链 | 传统银行体系 |
|------------|------------|
| 省储行 | 省级人民银行分行（清算中心） |
| 银行 | 商业银行网点（工行/建行/农行等） |
| 用户链上账户 | 现金（自己持有） |
| 用户银行存款 | 活期存款账户 |
| 充值 | 存款（现金→银行） |
| 提现 | 取款（银行→现金） |
| 扫码付款 | 转账（银行间清算） |

---

## 二、银行注册与许可

### 2.1 银行注册流程

银行使用现有的 `duoqian-manage-pow` 多签注册机制：

```
1. 银行发起人获取 SFID 机构身份码
2. 调用 register_sfid_institution() 注册链上身份
3. 调用 propose_create() 创建多签账户（管理员 2-64 人）
4. 投票通过后，银行获得 duoqian_address（多签账户）
```

### 2.2 清算许可

银行注册后，需要向所在省的省储行申请清算许可：

```
1. 银行管理员调用 apply_clearing_license(target_prb_institution)
2. 省储行管理员投票（内部投票 T≥6）
3. 投票通过 → 链上记录 BankClearingLicense
4. 银行获得开展存取款和链下支付业务的资格
```

### 2.3 银行要求

- 必须是 SFID 注册的多签机构
- 必须通过省储行清算许可投票
- duoqian_address 最低保证金 ≥ 10,000 元
- 必须运行全节点软件
- 一个银行只绑定一个省储行，不可更换

---

## 三、用户操作流程

### 3.1 开户

```
用户 → 选择银行 → 调用 open_deposit_account(bank_address)
                     ↓ 链上交易，1 元手续费
               UserBank 写入绑定关系
                     ↓
               开户成功，可以充值
```

- 一个用户同时只能在一个银行开户
- 开户不需要银行审批（链上交易即可）

### 3.2 充值

```
用户 → 输入金额 → 调用 deposit(bank_address, amount)
                     ↓ 链上交易
    Currency::transfer(用户链上账户 → 银行 duoqian_address)
    DepositBalance[银行][用户] += amount
                     ↓
               充值成功，存款余额增加
```

- 需要等出块确认（约 6 分钟）
- 链上手续费 0.1% 由用户支付

### 3.3 扫码付款（即时）

```
付款用户扫码 → wuminapp 显示金额和手续费
           → 用户确认并签名
           → RPC 发送到付款用户的银行节点
           → 银行验证签名、检查存款余额
           → 银行本地账本扣减（即时确认）
           → 用户收到"付款成功"

后台：银行缓存待结算交易
     → 省储行定期收集并打包上链
     → 链上执行：银行A duoqian → 银行B duoqian（主金额）
                银行A duoqian → 省储行 fee_address（手续费）
     → 扣减 DepositBalance[银行A][付款用户]
     → 增加 DepositBalance[银行B][收款用户]
```

- 付款即时确认（银行本地扣减，不需要链上操作）
- 清算在后台完成（用户无感）
- 收款方入账时机 = 清算上链后

### 3.4 收款

```
省储行打包清算上链后：
  DepositBalance[银行B][收款用户] += 收款金额
  收款用户在 wuminapp 看到存款余额增加
```

- 收款用户不需要做任何操作
- 入账时间 = 省储行清算周期（约 60 分钟）

### 3.5 提现

```
用户 → 输入金额 → 调用 withdraw(bank_address, amount)
                     ↓ 链上交易
    ensure DepositBalance[银行][用户] >= amount
    Currency::transfer(银行 duoqian_address → 用户链上账户)
    DepositBalance[银行][用户] -= amount
                     ↓
               提现成功，链上余额增加
```

- 需要等出块确认（约 6 分钟）
- 链上手续费 0.1% 由用户支付

### 3.6 换绑银行

```
1. 先从旧银行提现全部余额（链上交易）
2. 调用 switch_bank(new_bank_address)（链上交易）
3. 向新银行充值（链上交易）
```

---

## 四、清算流程

### 4.1 同省同行支付

付款用户和收款用户在同一个银行：

```
银行A 本地账本：
  DepositBalance[用户甲] -= 100 元 + 手续费
  DepositBalance[用户乙] += 100 元

省储行清算上链时：
  链上 DepositBalance[银行A][用户甲] -= 100 + fee
  链上 DepositBalance[银行A][用户乙] += 100
  银行A duoqian_address → 省储行 fee_address（手续费）
```

银行 duoqian_address 总余额不变（内部划转），只有手续费流出。

### 4.2 同省跨行支付

付款用户在银行 A，收款用户在银行 B，两家银行绑定同一省储行：

```
银行A 本地账本：
  DepositBalance[用户甲] -= 100 元 + 手续费

省储行清算上链时：
  Currency::transfer(银行A duoqian → 银行B duoqian, 100 元)
  Currency::transfer(银行A duoqian → 省储行 fee_address, 手续费)
  链上 DepositBalance[银行A][用户甲] -= 100 + fee
  链上 DepositBalance[银行B][用户乙] += 100
```

### 4.3 跨省跨行支付

付款用户在 A 省银行甲，收款用户在 B 省银行乙：

```
银行甲（A 省）本地账本：
  DepositBalance[付款用户] -= 100 元 + 手续费

银行甲提交 enqueue_offchain_batch 到链上队列
  ↓
B 省省储行执行 process_queued_batch：
  Currency::transfer(银行甲 duoqian → 银行乙 duoqian, 100 元)
  Currency::transfer(银行甲 duoqian → B省储行 fee_address, 手续费)
  链上 DepositBalance[银行甲][付款用户] -= 100 + fee
  链上 DepositBalance[银行乙][收款用户] += 100
```

清算由收款方绑定的省储行负责执行。银行甲和银行乙不需要直接通信——链上队列就是共识层。

### 4.4 清算触发方式

银行自主提交批次到链上队列（`enqueue_offchain_batch`），省储行负责执行（`process_queued_batch`）。

```
银行节点后台：
  每 10 个区块（约 60 分钟）或积累 10 万笔交易
  → 打包本地待结算交易为批次
  → 银行管理员签名
  → 提交 enqueue_offchain_batch 上链

省储行节点后台：
  监听链上队列中属于本省的待处理批次
  → 调用 process_queued_batch 执行
```

---

## 五、双花防护

### 5.1 为什么不存在双花

| 攻击场景 | 防护机制 |
|---------|---------|
| 用户同时在银行 A 两次付款 | 银行 A 单点控制账本，串行处理 |
| 用户同时在银行 A 和银行 B 付款 | 用户只能在一个银行开户（UserBank 唯一绑定） |
| 用户付款后立即提现 | 提现检查 DepositBalance，已被本地扣减的余额无法提现 |
| 银行伪造用户余额 | 链上 DepositBalance 是最终真相，清算时校验 |
| 银行 duoqian 余额不足 | 清算时 Currency::transfer 会失败，批次回滚 |

### 5.2 安全保障

- 用户存款记录在链上（DepositBalance），银行无法篡改
- 银行 duoqian_address 是多签账户，单个管理员无法挪用资金
- 省储行清算时校验链上 DepositBalance，银行本地账本仅为缓存
- 银行保证金制度：duoqian_address 余额必须 ≥ 总存款额

---

## 六、Runtime 设计

### 6.1 新增 pallet：`offchain-deposit-banking`

#### Storage

```rust
/// 银行清算许可：bank_duoqian_address → 绑定的省储行 InstitutionPalletId
pub type BankClearingLicense<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId,
    InstitutionPalletId, OptionQuery,
>;

/// 用户在银行的存款余额（分）：(bank_address, user_address) → balance
pub type DepositBalance<T> = StorageDoubleMap<
    _, Blake2_128Concat, T::AccountId,
    Blake2_128Concat, T::AccountId,
    u128, ValueQuery,
>;

/// 银行存款总额：bank_address → total
pub type BankTotalDeposits<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId,
    u128, ValueQuery,
>;

/// 用户绑定的银行：user_address → bank_address
pub type UserBank<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId,
    T::AccountId, OptionQuery,
>;
```

#### Extrinsic

| call_index | 函数 | 调用者 | 说明 |
|------------|------|--------|------|
| 0 | `apply_clearing_license` | 银行管理员 | 向省储行申请清算许可，发起内部投票 |
| 1 | `vote_clearing_license` | 省储行管理员 | 投票审批银行清算许可 |
| 2 | `open_deposit_account` | 用户 | 在银行开户（绑定 UserBank） |
| 3 | `deposit` | 用户 | 充值到银行（链上转账 + 更新 DepositBalance） |
| 4 | `withdraw` | 用户 | 从银行提现（链上转账 + 扣减 DepositBalance） |
| 5 | `switch_bank` | 用户 | 换绑银行（需先清零旧银行余额） |
| 6 | `revoke_clearing_license` | 省储行管理员 | 撤销银行清算许可（发起投票） |

### 6.2 修改 `offchain-transaction-pos`

#### OffchainBatchItem 新增字段

```rust
pub struct OffchainBatchItem<AccountId, Balance, Hash> {
    pub tx_id: Hash,
    pub payer: AccountId,
    pub payer_bank: AccountId,      // 新增：付款方银行
    pub recipient: AccountId,
    pub recipient_bank: AccountId,  // 新增：收款方银行
    pub transfer_amount: Balance,
    pub offchain_fee_amount: Balance,
}
```

#### execute_batch 改造

```rust
for item in batch.iter() {
    // 同行支付：只移动存款账本，不动链上余额
    if item.payer_bank == item.recipient_bank {
        DepositBalance::mutate(bank, payer, |b| *b -= transfer + fee);
        DepositBalance::mutate(bank, recipient, |b| *b += transfer);
        // 手续费从银行转给省储行
        Currency::transfer(payer_bank → fee_address, fee);
    } else {
        // 跨行支付：链上转账 + 两边账本更新
        Currency::transfer(payer_bank → recipient_bank, transfer);
        Currency::transfer(payer_bank → fee_address, fee);
        DepositBalance::mutate(payer_bank, payer, |b| *b -= transfer + fee);
        DepositBalance::mutate(recipient_bank, recipient, |b| *b += transfer);
    }
    BankTotalDeposits::mutate(payer_bank, |t| *t -= transfer + fee);
    if payer_bank != recipient_bank {
        BankTotalDeposits::mutate(recipient_bank, |t| *t += transfer);
    }
}
```

### 6.3 InstitutionAssetGuard 新增

```rust
pub enum InstitutionAssetAction {
    // ... 现有 ...
    BankSettlementDebit,     // 银行清算扣款（银行→银行 / 银行→省储行）
    BankWithdrawalDebit,     // 银行代用户提现（银行→用户）
}
```

### 6.4 AmountExtractor

| 调用 | Amount | 谁付链上手续费 |
|------|--------|-------------|
| `deposit` | Amount(充值金额) | 用户 |
| `withdraw` | Amount(提现金额) | 用户 |
| `open_deposit_account` | Amount(100000) = 1 元 | 用户 |
| `enqueue_offchain_batch` | Amount(批次手续费总额) | 银行 fee_address 代付 |
| `process_queued_batch` | Amount(批次手续费总额) | 省储行 fee_address 代付 |

---

## 七、Node 改动

### 7.1 删除

| 文件 | 说明 |
|------|------|
| `offchain_gossip.rs` | 整个删除，不再需要 P2P 广播 |
| `offchain_packer.rs` | 整个删除 |
| service.rs 中 gossip 注册 | 全部删除 |

### 7.2 改造 offchain_ledger.rs

```rust
/// 银行节点的本地存款缓存
pub struct DepositLedger {
    /// 用户存款缓存：user → (链上余额, 本地已确认待清算扣减)
    balances: HashMap<AccountId32, (u128, u128)>,
    /// 待清算交易列表
    pending_txs: Vec<OffchainTxItem>,
}

impl DepositLedger {
    /// 可用余额 = 链上存款余额 - 本地待清算扣减
    pub fn available_balance(&self, user: &AccountId32) -> u128 {
        let (onchain, pending) = self.balances.get(user).copied().unwrap_or((0, 0));
        onchain.saturating_sub(pending)
    }

    /// 确认付款：从可用余额扣减
    pub fn confirm_payment(&mut self, payer: &AccountId32, amount: u128) -> Result<(), String> {
        let avail = self.available_balance(payer);
        if avail < amount { return Err("存款余额不足".into()); }
        self.balances.entry(payer.clone()).or_insert((0, 0)).1 += amount;
        Ok(())
    }
}
```

### 7.3 RPC

#### 银行节点 RPC

| 方法 | 参数 | 说明 |
|------|------|------|
| `offchain_submitSignedTx` | payer, recipient, amount, fee, signature, tx_id | 用户扫码付款提交（改造现有） |
| `offchain_queryDepositBalance` | user_address | 查用户存款余额 |
| `offchain_queryTxStatus` | tx_id | 查交易状态（保留现有） |

#### 省储行节点 RPC

| 方法 | 参数 | 说明 |
|------|------|------|
| `offchain_queryInstitutionRate` | — | 查费率（保留现有） |

### 7.4 银行节点打包流程

```
银行节点后台 worker：
  loop {
    // 检查是否达到打包条件
    if pending_count >= 100_000 || blocks_since_last_pack >= 10 {
      // 取出所有待结算交易
      let items = ledger.take_all_pending();
      // 银行管理员签名
      let signature = signing_key.sign(batch_message);
      // 提交到链上队列
      submit_extrinsic(enqueue_offchain_batch(institution, seq, items, sig));
      // 上链成功后同步账本
      ledger.sync_from_chain();
    }
    wait_for_new_block();
  }
```

---

## 八、WuMinApp 改动

### 8.1 新增页面

| 页面 | 功能 |
|------|------|
| **银行列表页** | 展示已注册且有清算许可的银行，支持搜索 |
| **开户页** | 选择银行 → 确认开户 → 链上交易 |
| **充值页** | 输入金额 → 签名 → 链上交易 → 等待确认 |
| **提现页** | 显示存款余额 → 输入金额 → 签名 → 链上交易 |
| **存款详情页** | 显示开户银行、存款余额、充值/提现入口 |

### 8.2 修改现有页面

| 页面 | 改动 |
|------|------|
| **offchain_pay_page.dart** | 余额检查从链上余额改为存款余额；新增"余额不足请充值"提示 |
| **钱包主页** | 新增"链下支付账户"卡片，显示开户银行和存款余额 |
| **institutions.dart**（原 `clearing_banks.dart`，2026-04-15 合并重构） | 从硬编码改为链上查询已注册机构 |

### 8.3 新增 RPC 调用

```dart
/// 查询用户存款余额
Future<int> queryDepositBalance(String bankWssUrl, String userAddress);

/// 查询已注册银行列表（从链上 BankClearingLicense 存储）
Future<List<BankInfo>> queryRegisteredBanks();
```

---

## 九、废弃清单

| 组件 | 操作 | 原因 |
|------|------|------|
| `offchain_gossip.rs` | 删除 | 不再需要跨行 P2P 广播 |
| `offchain_packer.rs` | 删除 | 打包逻辑改为银行自主提交 |
| `offchain_ledger.rs` 的 remote_pending | 删除 | 不存在跨行竞态 |
| `offchain_ledger.rs` 的 virtual_balance | 删除 | 改为 deposit_balance |
| service.rs gossip 注册 | 删除 | 不再注册 P2P 协议 |
| 任务卡 20260404-offchain-packer-worker | 关闭 | packer 不再需要 |
| 任务卡 20260404-admin-replacement-mutex | 保留 | 与本方案无关 |

---

## 十、安全模型

### 10.1 银行信任模型

银行是注册多签机构，不是匿名节点：
- SFID 实名认证
- 多签管理员控制（2-64 人）
- 省储行投票许可
- 保证金制度
- 链上 DepositBalance 为最终真相，银行无法篡改

### 10.2 资金安全

| 风险 | 防护 |
|------|------|
| 银行挪用存款 | duoqian_address 多签控制，单个管理员无法转出 |
| 银行 duoqian 余额不足 | 清算时链上转账失败，批次回滚；保证金制度 |
| 银行跑路 | 用户存款记录在链上 DepositBalance，可由省储行强制清算 |
| 清算失败 | 链上队列支持重试（现有 QueuedBatches 机制） |
| 用户双花 | UserBank 唯一绑定 + 银行单点控制 |

### 10.3 偿付能力监控

```
银行偿付能力 = 银行 duoqian_address 链上余额 / BankTotalDeposits

偿付率 < 100%：发出链上警告事件
偿付率 < 80%：省储行可暂停银行清算许可
偿付率 < 50%：省储行可强制触发用户提现清算
```

---

## 十一、实施阶段

### Phase 1：Runtime 基础（2-3 周）
1. 新建 offchain-deposit-banking pallet
   - Storage：BankClearingLicense、DepositBalance、BankTotalDeposits、UserBank
   - Extrinsic：apply/vote_clearing_license、open_deposit_account、deposit、withdraw、switch_bank
   - 事件和错误定义
   - 单元测试
2. configs/mod.rs 配置（AmountExtractor、FeePayerExtractor、InstitutionAssetGuard）
3. institution-asset-guard 新增 BankSettlementDebit、BankWithdrawalDebit

### Phase 2：Runtime 清算改造（1-2 周）
1. OffchainBatchItem 新增 payer_bank、recipient_bank 字段
2. execute_batch 改造（同行/跨行逻辑）
3. 清算时更新 DepositBalance 和 BankTotalDeposits
4. 集成测试：同行支付、跨行支付、跨省支付

### Phase 3：Node 改造（1 周）
1. 删除 offchain_gossip.rs、offchain_packer.rs
2. 改造 offchain_ledger.rs（DepositLedger）
3. 改造 rpc.rs（submitSignedTx 改用存款余额、新增 queryDepositBalance）
4. 银行节点打包 worker（enqueue_offchain_batch 提交）
5. 清理 service.rs

### Phase 4：WuMinApp（1-2 周）
1. 新增开户/充值/提现页面
2. 修改 offchain_pay_page.dart（存款余额）
3. 新增银行列表查询
4. 钱包主页存款余额显示

### Phase 5：清理与文档（2-3 天）
1. 删除废弃代码和文档
2. 更新技术文档
3. 更新白皮书 2.5 节
4. 关闭相关任务卡

### 总估算：5-8 周
