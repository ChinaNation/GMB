# FULLNODE PoW Reward Technical Notes

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

---

## 5. 对外调用（Extrinsics）
### 5.1 `bind_reward_wallet(wallet)`（call index = 0）
- 权限：`Signed`
- 逻辑：
1. 校验调用者未绑定过奖励钱包
2. 写入 `RewardWalletByMiner`
3. 发送 `RewardWalletBound`
- weight：`T::DbWeight::get().reads_writes(1, 1)`

说明：
- 当前不做“是否真实矿工”的额外资格校验。
- 这是一个已知 trade-off；若 runtime 后续引入矿工注册表/白名单，可在此处接入资格检查。

### 5.2 `rebind_reward_wallet(new_wallet)`（call index = 1）
- 权限：`Signed`
- 逻辑：
1. 校验调用者已存在绑定
2. 覆盖写入新钱包
3. 发送 `RewardWalletRebound`
- weight：`T::DbWeight::get().reads_writes(1, 1)`

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
- 用户调用（bind/rebind）使用静态 `DbWeight` 估算。
- `on_finalize` 的执行预算由 `on_initialize` 统一预申报。

注意事项：
- `bind/rebind` 的 `reads_writes(1,1)` 覆盖了核心存储路径。
- 更精确的权重可在后续引入 benchmark 后替换。

---

## 8. 测试覆盖（当前）
`cargo test -p fullnode-pow-reward` 当前覆盖：
- 一次性绑定与重复绑定拒绝
- 起始边界块（`1`）发放
- 结束边界块（`9_999_999`）发放
- 区间外（`0`、`end+1`）不发放
- 未绑定钱包不发放
- 多区块累计奖励正确
- `FindAuthor = None` 跳过事件
- 未绑定钱包跳过事件
- `rebind` 正常路径与未绑定拒绝
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
