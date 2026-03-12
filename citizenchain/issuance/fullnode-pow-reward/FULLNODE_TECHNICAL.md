# FULLNODE PoW Reward Technical Notes

## 0. 功能需求
`fullnode-pow-reward` 的功能需求是：在固定的 PoW 奖励区块高度区间内，按照制度常量为成功出块的全节点作者发放固定金额奖励，并允许矿工自行管理奖励到账钱包。

模块必须满足以下要求：
- 奖励金额、起始高度、结束高度必须由制度常量固定，不能被链上治理动态修改。
- 只有在奖励区间内，且能够从当前区块共识 digest 识别出作者时，才允许发放奖励。
- 奖励只发放给已经绑定奖励钱包的矿工身份；未绑定时必须跳过并留下链上审计事件。
- 矿工必须支持首次绑定奖励钱包，并且可以自行重绑到新钱包。
- 奖励钱包必须与矿工身份账户分离；`rebind` 必须真正切换到不同的新钱包。
- 奖励结算必须发生在 `on_finalize`，只对“已经完成的出块行为”进行结算，不做预测性预发。
- 发行和跳过都必须链上可审计，便于后续核账。
- 模块应避免引入不必要的发行状态存储，不能额外维护“已奖励区块列表”之类的冗余状态。

## 1. 模块定位
`fullnode-pow-reward` 是一个 FRAME pallet，用于在 PoW 链上按固定制度发放全节点铸块奖励。

核心目标：
- 奖励规则常量化（金额、起止高度写死在 `primitives::pow_const`）。
- 奖励触发客观化（仅基于区块高度 + 共识层 `FindAuthor`）。
- 发放可审计（发放与跳过均有链上事件）。
- 矿工自主管理收款地址（支持首次绑定 + 重绑，无需治理）。

代码位置：
- `/Users/rhett/GMB/citizenchain/issuance/fullnode-pow-reward/src/lib.rs`

---

## 2. 关键常量与配置
来自 `primitives::pow_const`：
- `FULLNODE_REWARD_START_BLOCK`（当前为 `1`）
- `FULLNODE_REWARD_END_BLOCK`（当前为 `9_999_999`）
- `FULLNODE_BLOCK_REWARD`（每块固定奖励）

Runtime 注入配置：
- `Config::Currency`：奖励铸造与记账货币实现
- `Config::FindAuthor`：从 PreRuntime Digest 解析区块作者

---

## 3. 存储结构
- `RewardWalletByMiner: Map<AccountId -> AccountId>`
  - key：矿工身份账户（出块作者）
  - value：奖励到账钱包账户
  - 语义：奖励只发放到已绑定的钱包；重绑会覆盖旧值

---

## 4. 事件与错误
主要事件：
- `RewardWalletBound { miner, wallet }`
- `RewardWalletRebound { miner, new_wallet }`
- `PowRewardIssued { block, miner, wallet, amount }`
- `PowRewardSkippedNoAuthor { block }`
- `PowRewardSkippedNoBoundWallet { block, miner }`

主要错误：
- `RewardWalletAlreadyBound`
- `RewardWalletNotBound`
- `RewardWalletCannotBeMiner`
- `RewardWalletUnchanged`

---

## 5. 对外调用（Extrinsics）
### 5.1 `bind_reward_wallet(wallet)`（call index = 0）
- 权限：`Signed`
- 逻辑：
1. 校验调用者未绑定过奖励钱包
2. 校验 `wallet != miner`
3. 写入 `RewardWalletByMiner`
4. 发送 `RewardWalletBound`
- weight：`T::WeightInfo::bind_reward_wallet()`

说明：
- 当前不做“是否真实矿工”的额外资格校验。
- 当前要求奖励钱包必须与矿工身份账户不同，避免矿工身份与收款钱包混同。
- 这是一个已知 trade-off；若 runtime 后续引入矿工注册表/白名单，可在此处接入资格检查。

### 5.2 `rebind_reward_wallet(new_wallet)`（call index = 1）
- 权限：`Signed`
- 逻辑：
1. 读取当前绑定；未绑定则拒绝
2. 校验 `new_wallet != miner`
3. 校验 `new_wallet != current_wallet`
4. 覆盖写入新钱包
5. 发送 `RewardWalletRebound`
- weight：`T::WeightInfo::rebind_reward_wallet()`

说明：
- 当前不设冷却期；矿工可按需重绑，但必须真正切换到不同的新钱包。

---

## 6. 生命周期逻辑（Hooks）
### 6.1 `on_initialize(n)`: finalize 预算预申报
- 行为：
  - 当 `n` 在奖励区间 `[FULLNODE_REWARD_START_BLOCK, FULLNODE_REWARD_END_BLOCK]` 时，返回 `T::DbWeight::get().reads_writes(3, 3)`。
  - 区间外返回 `Weight::zero()`。
- 目的：为 `on_finalize` 的最坏路径预留区块 weight 预算，避免 finalize 工作“未计重”。

### 6.2 `on_finalize(n)`: 实际奖励结算
流程：
1. 将区块高度转换为 `u32`，若不在奖励区间则直接返回。
2. 从 `frame_system::digest()` 读取 pre-runtime digest。
3. 通过 `T::FindAuthor::find_author(...)` 解析作者：
   - `None`：发 `PowRewardSkippedNoAuthor { block }` 并返回。
4. 查询 `RewardWalletByMiner`：
   - `None`：发 `PowRewardSkippedNoBoundWallet { block, miner }` 并返回。
5. 以 `FULLNODE_BLOCK_REWARD` 铸造并发放到绑定钱包（`deposit_creating`）。
6. 发 `PowRewardIssued { block, miner, wallet, amount }`。

---

## 7. Weight 策略
当前策略：
- 用户调用（bind/rebind）使用 benchmark 生成的 `T::WeightInfo`。
- `on_finalize` 的执行预算由 `on_initialize` 统一预申报。

注意事项：
- `src/weights.rs` 已由 benchmark 自动生成，`bind/rebind` 当前路径对应 1 次读取 + 1 次写入。
- `on_finalize` 的预算仍通过 `on_initialize` 的 `reads_writes(3,3)` 预申报。

---

## 8. 测试覆盖（当前）
`cargo test -p fullnode-pow-reward` 当前覆盖：
- 一次性绑定与重复绑定拒绝
- 绑定矿工自身账户为奖励钱包会被拒绝
- 起始边界块（`1`）发放
- 结束边界块（`9_999_999`）发放
- 区间外（`0`、`end+1`）不发放
- 未绑定钱包不发放
- 多区块累计奖励正确
- `FindAuthor = None` 跳过事件
- 未绑定钱包跳过事件
- `rebind` 正常路径与未绑定拒绝
- `rebind` 到矿工自身账户会被拒绝
- `rebind` 到当前已绑定钱包会被拒绝
- `rebind` 后奖励端到端流向新钱包
- `on_initialize` 区间内外 weight 声明行为

---

## 9. 运维与审计建议
- 发行审计优先看三类事件：
  - 成功：`PowRewardIssued`
  - 跳过（无作者）：`PowRewardSkippedNoAuthor`
  - 跳过（未绑定）：`PowRewardSkippedNoBoundWallet`
- 矿工钱包管理建议：
  - 首次上线后尽快绑定奖励钱包。
  - 钱包迁移/风险处置时使用 `rebind_reward_wallet` 主动切换收款地址。

## 10. 审查结论与建议
本轮没有发现新的高风险权限绕过或重复发奖漏洞。

当前状态：
1. 仍允许任意签名账户先绑定一个奖励钱包，即使它从未真正出块；这不会造成多发奖励，但会带来少量无效状态膨胀。若后续引入矿工注册表/白名单，可在 `bind_reward_wallet` 接资格检查。  
2. 已明确禁止把矿工身份账户本身作为奖励钱包，避免矿工身份与收款地址混同。  
3. `rebind` 当前不设冷却期，但必须切换到不同的新钱包；重复绑定当前钱包会直接拒绝，避免无意义写操作和事件。  
4. `bind/rebind` 已接入 benchmark 生成的权重；`on_finalize` 预算仍通过 `on_initialize` 预申报。  
5. 已补 `integrity_test` 校验 `FULLNODE_BLOCK_REWARD` 必须能完整装入 runtime `Balance`，避免未来有人调整 Balance 类型后发生静默截断。  
