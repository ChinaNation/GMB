# 全节点奖励账户绑定去掉「必须先出块」门槛

状态：open（2026-07-31，用户已定方向：下次 runtime 升级时改，不重新创世）

## 缺陷

`fullnode-issuance` 的 `bind_reward_account`（call_index 0）要求矿工账户必须已经出过块，
否则拒绝：

```rust
// runtime/issuance/fullnode-issuance/src/lib.rs:165
ensure!(
    LastAuthoredBlockByMiner::<T>::contains_key(&miner_account_id),
    Error::<T>::MinerNeverAuthoredBlock
);
```

而 `LastAuthoredBlockByMiner` 全仓只有一处写入 —— `on_finalize` 里真实出块时
（`lib.rs:260` `insert(&author, block_number)`），没有第二个入口。

新装节点的 `powr` 矿工账户由节点首启自动生成，既无余额也无出块记录；能否出块又是
PoW 概率事件。于是「要绑定就得先出块，出不了块就永远绑不上」。

## 影响

三层递进伤害：

1. **首块手续费必然损失。** 绑定资格来自第一个块，该块结算时必然尚未绑定，走
   `BurnReason::RewardAccountUnbound` 把全节点分成销毁。分成比例是
   `ONCHAIN_FEE_FULLNODE_PERCENT = 80`（`primitives/src/fee_policy.rs:76`），即每个节点
   入网的第一块，手续费的 80% 直接烧掉，无法规避。
2. **弱算力节点长期无法配置。** 1 核节点与多核节点竞争时期望出块时间极长，期间收款账户
   绑不上，运维无法在部署时一次配齐。
3. **本链当前是完全锁死。** `node/src/core/service.rs:627` 空交易池跳过 propose，链上无
   交易时三台矿工哈希率全为 0、谁都不出块，新节点永远拿不到出块记录。2026-07-31 部署的
   中枢省 `prczss`、贵州省 `prcgzs` 实测正卡在此处：两笔 `reward_bindAccount` 已进块 #9
   执行，`RewardAccountIdByMiner` 未写入，节点日志持续刷
   `burn fullnode fee share: author found but reward account not bound`。

出块奖励本身不丢：未绑定时 `unwrap_or_else(|| author.clone())` 发给矿工账户自身
（`lib.rs:265`），国储会 1,999,800 分正是 2 块 × `FULLNODE_BLOCK_REWARD` 999,900。丢的
只有手续费分成。

## 该约束不值得保留

代码注释称意图是「只有共识 digest 证明真实出过块的账户，才允许后续绑定奖励接收账户」，
即防止 `RewardAccountIdByMiner` 被垃圾数据占用。但：

- 该表只在出块时被 `get(&author)` 读取，未出块账户绑了也永远读不到，是纯死数据；
- 绑定交易本身要付手续费，已有经济门槛；
- 防 storage 垃圾的标准做法是 storage deposit（押金），不是拿"必须先干活"当门票。

用一个读不到的死数据风险，换掉新节点的可配置性，代价远大于收益。

## 方案（用户选定：去掉出块限制）

删除 `bind_reward_account` 中的 `MinerNeverAuthoredBlock` 检查，语义改为「预绑定」：
任何账户可提前登记奖励接收账户，该登记只在其真实出块时生效。

保留的检查不变：
- `RewardAccountAlreadyBound`（一个矿工账户只能绑一次，改绑走 call 1）
- `RewardAccountCannotBeMiner`（收款账户不得等于矿工账户）

`Error::MinerNeverAuthoredBlock` 变体在 call 0 不再触发；若无其它引用应一并删除，遵守
「无残桩」死规则。`LastAuthoredBlockByMiner` 存储本身保留 —— 它仍是奖励发放与审计的
事实来源，只是不再充当绑定门票。

## 为什么不需要重新创世

该 `ensure!` 位于 call 的执行逻辑内，不改 storage 布局、不改创世状态、不改
`genesis_hash`/`state_root`。通过正式链 `system.setCode` 升级即可生效，不构成硬分叉。
这也是本卡定位为「下次 runtime 升级顺带改」而非阻塞项的原因。

## 验收

1. 单测：新增「未出块账户可成功绑定」用例；原
   `tests/cases.rs:47` 断言 `MinerNeverAuthoredBlock` 的用例须同步改写或删除。
2. 单测：已绑定账户重复绑定仍报 `RewardAccountAlreadyBound`；收款账户等于矿工账户仍报
   `RewardAccountCannotBeMiner`。
3. `cargo check` 全 workspace 通过，`Error` 枚举无未使用变体残留。
4. 升级后在正式链实测：中枢省 `prczss`、贵州省 `prcgzs` 在未出块状态下成功绑定
   `w5CB1UoqD2PyKpnxmEgJB7TXEqHyAj3s5nDEeGkLNYJKoCdPN`，读回 `RewardAccountIdByMiner`
   逐字节核对。

## 遗留

- 首块手续费销毁问题不在本卡范围。去掉门槛后节点可在部署时预绑定，首块分成即可正常入账，
  该问题自然消解；但**已经发生的销毁不追溯补发**。
- 本缺陷是 2026-07-30 创世前审计的漏网项。当时审查重点是「创世后无法通过 runtime 升级
  解决的阻塞性 bug」，该项因可升级修复未被列入，但它是实打实影响节点运维的功能缺陷，
  审计口径应当覆盖到。
