# FULLNODE PoW Reward Technical Notes

## 0. 功能需求
`fullnode-pow-reward` 的功能需求是：在固定的 PoW 奖励区块高度区间内，按照制度常量为成功出块的全节点作者发放固定金额奖励，并允许矿工自行管理奖励到账钱包。

模块必须满足以下要求：
- 奖励金额、起始高度、结束高度必须由制度常量固定，不能被链上治理动态修改。
- 只有在奖励区间内，且能够从当前区块共识 digest 识别出作者时，才允许发放奖励。
- 未绑定钱包时，奖励默认发到矿工自身账户；绑定钱包后发到绑定的钱包。
- 矿工必须支持首次绑定奖励钱包，并且可以自行重绑到新钱包。
- 奖励钱包必须与矿工身份账户分离；`rebind` 必须真正切换到不同的新钱包。
- 奖励结算必须发生在 `on_finalize`，只对"已经完成的出块行为"进行结算，不做预测性预发。
- 发行都必须链上可审计，便于后续核账。
- 模块应避免引入不必要的发行状态存储，不能额外维护"已奖励区块列表"之类的冗余状态。

## 1. 模块定位
`fullnode-pow-reward` 是一个 FRAME pallet，用于在 PoW 链上按固定制度发放全节点铸块奖励。

核心目标：
- 奖励规则常量化（金额、起止高度写死在 `primitives::pow_const`）。
- 奖励触发客观化（仅基于区块高度 + 共识层 `FindAuthor`）。
- 发放可审计（发放均有链上事件）。
- 矿工自主管理收款地址（支持首次绑定 + 重绑，无需治理）。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/issuance/fullnode-pow-reward/src/lib.rs`

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
  - 语义：绑定后奖励发放到钱包；未绑定时奖励发到矿工自身账户；重绑会覆盖旧值

---

## 4. 事件与错误
主要事件：
- `RewardWalletBound { miner, wallet }`
- `RewardWalletRebound { miner, new_wallet }`
- `PowRewardIssued { block, miner, wallet, amount }`
- `PowRewardSkippedNoAuthor { block }`

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
- 当前不做"是否真实矿工"的额外资格校验。
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
- 目的：为 `on_finalize` 的最坏路径预留区块 weight 预算，避免 finalize 工作"未计重"。

### 6.2 `on_finalize(n)`: 实际奖励结算
流程：
1. 将区块高度转换为 `u32`，若不在奖励区间则直接返回。
2. 从 `frame_system::digest()` 读取 pre-runtime digest。
3. 通过 `T::FindAuthor::find_author(...)` 解析作者：
   - `None`：发 `PowRewardSkippedNoAuthor { block }` 并返回。
4. 查询 `RewardWalletByMiner`：
   - `Some(wallet)`：奖励发到绑定的钱包。
   - `None`：奖励发到矿工自身账户。
5. 以 `FULLNODE_BLOCK_REWARD` 铸造并发放到收款地址（`deposit_creating`）。
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
- 未绑定钱包时奖励发到矿工自身账户
- 多区块累计奖励正确
- `FindAuthor = None` 跳过事件
- 未绑定钱包时奖励发到矿工并 emit PowRewardIssued
- `rebind` 正常路径与未绑定拒绝
- `rebind` 到矿工自身账户会被拒绝
- `rebind` 到当前已绑定钱包会被拒绝
- `rebind` 后奖励端到端流向新钱包
- `on_initialize` 区间内外 weight 声明行为

---

## 9. 运维与审计建议
- 发行审计优先看两类事件：
  - 成功：`PowRewardIssued`
  - 跳过（无作者）：`PowRewardSkippedNoAuthor`
- 矿工钱包管理建议：
  - 矿工启动后即可出块获得奖励（默认发到矿工自身账户）。
  - 需要将收益发到独立钱包时，通过 `bind_reward_wallet` 绑定。
  - 钱包迁移/风险处置时使用 `rebind_reward_wallet` 主动切换收款地址。

## 10. 矿工密钥架构

### 10.1 设计原则
矿工密钥有且仅有一把，唯一来源是 keystore 文件。

- **node 进程**（Substrate 框架）：唯一生成密钥的地方
- **keystore**：唯一存储密钥的地方
- **nodeui**：只读取 keystore 中的密钥，不生成、不写入

### 10.2 密钥生成（node 进程）
代码位置：`node/src/service.rs` → `ensure_powr_key()`

节点启动时自检 keystore 中是否已有 `powr` 类型密钥：
- 有 → 直接使用，不再生成
- 没有 → 调用 `sr25519_generate_new(POW_AUTHOR_KEY_TYPE, None)`

Substrate 框架的 `sr25519_generate_new(key_type, None)` 行为：
1. 生成 12 个单词的 BIP39 助记词
2. 从助记词推导 sr25519 密钥对
3. 将助记词写入 keystore 磁盘文件（JSON 编码字符串）
4. 文件名格式：`{key_type_hex}{pubkey_hex}`

注意：`sr25519_generate_new(key_type, Some(suri))` 只存内存不写磁盘，进程退出后丢失，不可用。

### 10.3 奖励钱包绑定（node 自定义 RPC）
代码位置：`node/src/rpc.rs` → `reward_bindWallet` / `reward_rebindWallet`

绑定/重绑奖励钱包完全由 node 端完成：
1. nodeui 调用 node 的自定义 RPC `reward_bindWallet(wallet_ss58)` 或 `reward_rebindWallet(new_wallet_ss58)`
2. node 从 keystore 读取 `powr` 公钥，使用 `keystore.sr25519_sign()` 签名交易
3. 构造完整的 `UncheckedExtrinsic` 并提交到交易池

nodeui **不读取私钥、不签名**，仅传入收款钱包的 SS58 地址。签名使用与出块相同的 `sp_core` 密钥推导路径，确保签名身份与出块作者身份一致。

### 10.4 nodeui 的角色
- 只读取默认链（`citizenchain`）keystore 文件名中的公钥（`local_powr_miner_account_hex`），用于前端展示矿工身份；不遍历其他链目录，避免旧链残留 keystore 导致身份错位
- 设置奖励钱包时在同步路径提前校验 wallet != miner，避免先存后验
- 通过 `state_getStorage` 查询链上 `RewardWalletByMiner` 状态，判断是否需要 bind 或 rebind
- 所有签名和交易提交委托给 node 端 RPC

### 10.5 密钥使用流程
1. 用户首次启动节点 → node 的 `ensure_powr_key()` 生成密钥并写入 keystore
2. 节点出块 → `author_pre_digest()` 从 keystore 读取公钥作为区块作者
3. 用户绑定收款地址 → nodeui 调用 node RPC `reward_bindWallet` → node 用 keystore 密钥签名并提交
4. 链上 `bind_reward_wallet` 以矿工身份记录映射 → 后续奖励发到绑定的收款地址

由于出块和绑定使用的是同一把 keystore 密钥（同一个 `sr25519_sign` 路径），`RewardWalletByMiner` 映射的 key 与出块作者一致，奖励能正确发到绑定的收款地址。

---

## 11. 审查结论与建议
本轮没有发现新的高风险权限绕过或重复发奖漏洞。

当前状态：
1. 仍允许任意签名账户先绑定一个奖励钱包，即使它从未真正出块；这不会造成多发奖励，但会带来少量无效状态膨胀。若后续引入矿工注册表/白名单，可在 `bind_reward_wallet` 接资格检查。
2. 已明确禁止把矿工身份账户本身作为奖励钱包，避免矿工身份与收款地址混同。
3. `rebind` 当前不设冷却期，但必须切换到不同的新钱包；重复绑定当前钱包会直接拒绝，避免无意义写操作和事件。
4. `bind/rebind` 已接入 benchmark 生成的权重；`on_finalize` 预算仍通过 `on_initialize` 预申报。
5. 已补 `integrity_test` 校验 `FULLNODE_BLOCK_REWARD` 必须能完整装入 runtime `Balance`，避免未来有人调整 Balance 类型后发生静默截断。
