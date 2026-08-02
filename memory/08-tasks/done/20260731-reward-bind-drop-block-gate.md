# 全节点奖励账户绑定去掉「必须先出块」门槛

状态：completed（2026-08-02；业务修复、benchmark、权重、测试、注释与残留清理全部完成）

## 缺陷

`fullnode-issuance` 的 `bind_reward_account`（call index 0）曾要求矿工账户必须已经出过块，
否则拒绝。该门槛把奖励账户配置错误地依赖于概率性的首次出块。

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

删除 `bind_reward_account` 中的旧出块资格检查，语义改为「预绑定」：
任何账户可提前登记奖励接收账户，该登记只在其真实出块时生效。

保留的检查不变：
- `RewardAccountAlreadyBound`（一个矿工账户只能绑一次，改绑走 call 1）
- `RewardAccountCannotBeMiner`（收款账户不得等于矿工账户）

对应旧错误变体已删除，遵守「无残桩」死规则。`LastAuthoredBlockByMiner` 存储本身保留 ——
它仍是奖励发放与审计的事实来源，只是不再充当绑定门票。

## 实施结果

该业务修复最终随 2026-08-01 正式重新创世进入冻结 runtime：`bind_reward_account` 已无
旧出块资格分支，未出块账户可以预绑定奖励接收账户；Storage、call index、Event 和权限模型
未改变。2026-08-02 完成其余 runtime 收尾并把 `spec_version` 从 0 提升为 1。

## 验收

1. [x] 单测覆盖「未出块账户可成功绑定」，`Error` 枚举已删除旧错误变体。
2. [x] 重复绑定和绑定矿工自身账户的安全检查保持不变。
3. [x] 补充「预绑定账户收到首次出块奖励」测试，pallet 当前 20 项测试全部通过。
4. [x] 贵州 `prcgzs`、中枢 `prczss`、国家储委会 `nrcgch` 三个生产节点的 `powr`
   账户均已完成首次绑定；finalized `RewardAccountIdByMiner` 全部逐字节等于目标
   `account_id=0x1a16ee768af324002ea732b796b14c34a261e08ff6be89ea67ad2b7fa04bd94e`。
5. [x] 使用 Substrate Benchmark CLI 53.0.0、`steps=50`、`repeat=20` 重新生成权重；
   绑定与重绑均为 1 次读取、1 次写入，生成 ref time 分别为 6,000,000 与 7,000,000。
6. [x] `cargo check --workspace --all-targets --locked`、全 workspace 测试和
   `cargo clippy --workspace --all-targets --locked -- -D warnings` 全部通过。

## 结论

- 首块手续费销毁问题不在本卡范围。去掉门槛后节点可在部署时预绑定，首块分成即可正常入账，
  该问题自然消解；但**已经发生的销毁不追溯补发**。
- 本缺陷是 2026-07-30 创世前审计的漏网项。当时审查重点是「创世后无法通过 runtime 升级
  解决的阻塞性 bug」，该项因可升级修复未被列入，但它是实打实影响节点运维的功能缺陷，
  审计口径应当覆盖到。
- benchmark、生成权重、测试辅助函数、旧注释和旧命名残留已全部清理，本卡无待办项。
