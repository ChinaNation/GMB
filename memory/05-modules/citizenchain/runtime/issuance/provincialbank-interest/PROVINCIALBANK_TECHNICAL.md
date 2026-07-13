# 省储行创立质押与固定年度利息技术文档

## 1. 制度定位

`provincialbank-interest` 只执行一项不可由治理或 runtime 升级改变的原生货币规则：以
`primitives::cid::china::china_ch::CHINA_CH` 中固定 43 家省储行的创立质押本金为基数，
在连续 100 个制度年度向各省储行 `main_account` 发放固定递减利息。

该规则与 `resolution-issuance`、`onchain-issuance` 边界明确：后两者继续属于治理发行和
非治理链上资产发行；省储行创立质押及其利息属于 NodeGuard 可独立复算的永久发行。

## 2. 固定规则

- 省储行集合：`CHINA_CH` 中 43 家，不允许增加、删除、替换或重复。
- 创立质押账户：每家的 `stake_account`。
- 创立质押本金：每家的 `stake_amount`，创世逐户写入且后续永久不得变化。
- 利息收款账户：每家的 `main_account`，不是 `stake_account`。
- 第一年利率：`100 BP`（1.00%）。
- 每年递减：`1 BP`（0.01%）。
- 发放年限：100 年，第 100 年利率为 `1 BP`，第 101 年起不再发行。
- 年度周期：`87,600` 区块。
- 单户公式：`stake_amount × rate_bp(year) ÷ 10,000`。
- 全部 `stake_amount` 均可被 10,000 整除，节点可无舍入误差复算累计值。

制度常量位于：

- `citizenchain/runtime/primitives/src/core_const.rs`
- `citizenchain/runtime/primitives/src/pow_const.rs`
- `citizenchain/runtime/primitives/cid/china/china_ch.rs`

## 3. 创世本金

`citizenchain/runtime/src/genesis.rs` 把 43 笔创立发行逐户写入无私钥 `stake_account`。
创世总注入仍按三类独立来源核算：

```text
GENESIS_ISSUANCE
+ 43 家省储行 stake_amount 合计
+ HE_FUND_ISSUANCE
```

Runtime 测试逐户核对 `stake_account` 和 `stake_amount`；NodeGuard 启动、完整状态导入和
runtime 升级全检会复核完整 `System::Account`，不仅比较 free balance，也冻结 nonce、引用计数、
reserved、frozen 和 flags。任何本金增减、账户删除或字段改写均拒块。

## 4. Runtime 状态

模块只保留三项审计 storage：

- `LastSettledYear: u32`：最后完整结算年度，创世为 0。
- `TotalProvincialBankInterestIssued: u128`：省储行利息累计发行量，创世为 0。
- `LastProvincialBankInterestAudit: Option<ProvincialBankInterestAudit>`：最近年度、银行数量和年度总利息；首个年度前不存在。

FRAME pallet `StorageVersion` 保持 0；NodeGuard 同时校验该规范 key，非零版本或同 pallet 前缀下的
任何未知影子 key 都会 fail-closed。

`ProvincialBankInterestAudit` 的 SCALE 字段序固定为：

```text
(year: u32, bank_count: u32, total_interest: u128)
```

Runtime 与 node 均有字段序防漂移测试。

## 5. 执行阶段与原子性

`on_initialize` 只判断是否为第 1..=100 个年度边界，并预留 benchmark 权重；实际铸发统一在
`on_finalize` 执行，使 NodeGuard 能把省储行利息与全节点、公民认证奖励合并到同一份
`FinalizeIssuancePlan`，同时与 extrinsic 阶段允许的决议发行、链上发行隔离。

年度结算必须满足：

1. `LastSettledYear == year - 1`；
2. 逐户从固定 `stake_amount` 计算利息；
3. 利息只进入对应 `main_account`；
4. 43 笔全部成功后才更新累计量、年度和最近审计；
5. 任一解码或算术错误使整个存储事务回滚，不保留部分发行；
6. NodeGuard 因缺少精确固定发行计划而拒绝该区块。

模块不再暴露任何 `Call`。旧 `force_settle_years`、`force_advance_year`、跳年、批量补年和
Root 恢复分支均已删除，不保留兼容调用或影子流程。年度边界若失败，链停在边界前，必须修复
runtime 后重新正确执行该年度，不能跳过应发利息。

## 6. 事件

- `ProvincialBankInterestMinted { year, account, amount }`：单户固定利息到账。
- `ProvincialBankYearSettled { year, bank_count, total_interest }`：43 家全部到账并推进审计。

失败事务会整体回滚，错误只写 runtime 日志；链上不会保留“失败但部分发行”的事件或状态。

## 7. NodeGuard 边界

节点策略位于：

- `citizenchain/node/src/core/node_guard/provincialbank_interest.rs`

节点不读取 runtime metadata/API，独立固定 RAW key、SCALE 镜像和公式，并检查：

- block#0 的 43 笔永久质押本金；
- 普通块父状态、finalize 前和 finalize 后审计；
- 年度连续性、年度总额、累计量和最近审计；
- 43 个 `main_account` 的精确利息计划；
- `Balances::TotalIssuance` 与所有固定 finalize 发行的合计；
- 未登记收款账户、错误金额、错误账户、提前发行、重复发行和跳年；
- `:code` 变化后的全量本金与当前年度审计；
- block#0 完整状态导入中的本金、规范空审计和未知 pallet key。

## 8. 权重

正式 pallet benchmark：

- 命令参数：50 steps / 20 repeats；
- `on_finalize_settlement`：45 reads / 46 writes；
- 实测执行时间模型约 569 ms；
- 估算 proof size 112,919 bytes；
- 生成文件：`src/weights.rs`。

非年度边界和第 101 年后的区块返回零额外权重；年度边界在 `on_initialize` 预留
`on_finalize_settlement()` 权重。

## 9. 验收基线（2026-07-12）

- `provincialbank-interest`：10/10；带 `runtime-benchmarks`：11/11。
- runtime 创世测试逐户验证 43 个 `stake_account` 和 `stake_amount`。
- NodeGuard 全量：64/64；省储行策略定向：8/8。
- 当前源码 production WASM 与 node build 通过。
- fresh headless 节点使用独立 `/tmp` 数据库启动到 block#0，创世哈希
  `0x6fc42816b55ce22f204d0dbddbf38a9ab4d3a1c78005b90e1fcbe376ef8585b1`，临时数据库约 352 MiB。
- NodeGuard 没有误拒绝当前真实创世本金；临时数据库已删除。
