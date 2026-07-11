# FULLNODE Issuance Technical Notes

## 0. 功能需求
`fullnode-issuance` 的功能需求是：在固定的 PoW 奖励区块高度区间内，按照制度常量为成功出块的全节点作者发放固定金额奖励，并允许矿工自行管理奖励到账钱包。

模块必须满足以下要求：
- 奖励金额、起始高度、结束高度必须由制度常量固定，不能被链上治理动态修改。
- 只有在奖励区间内，且能够从当前区块共识 digest 识别出作者时，才允许发放奖励。
- 未绑定钱包时，奖励默认发到矿工自身账户；绑定钱包后发到绑定的钱包。
- 已经真实出过块的矿工必须支持首次绑定奖励钱包，并且可以自行重绑到新钱包。
- 奖励钱包必须与矿工身份账户分离；`rebind` 必须真正切换到不同的新钱包。
- 奖励结算必须发生在 `on_finalize`，只对"已经完成的出块行为"进行结算，不做预测性预发。
- 发行都必须链上可审计，便于后续核账。
- 模块不维护逐块奖励列表，只保留节点守卫逐块复算与 warp 累计核验所需的最小审计状态。

## 1. 模块定位
`fullnode-issuance` 是一个 FRAME pallet，用于在 PoW 链上按固定制度发放全节点铸块奖励。

核心目标：
- 奖励规则常量化（金额、起止高度写死在 `primitives::pow_const`）。
- 奖励触发客观化（仅基于区块高度 + 共识层 `FindAuthor`）。
- 发放可审计（发放均有链上事件）。
- 矿工自主管理收款地址（真实出块后支持首次绑定 + 重绑，无需治理）。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/issuance/fullnode-issuance/src/lib.rs`

---

## 2. 关键常量与配置
来自 `primitives::pow_const`：
- `FULLNODE_REWARD_START_BLOCK`（当前为 `1`）
- `FULLNODE_REWARD_END_BLOCK`（当前为 `9_999_999`）
- `FULLNODE_BLOCK_REWARD`（每块固定奖励）

Runtime 注入配置：
- `Config::Currency`：奖励铸造与记账货币实现
- `Config::FindAuthor`：从 PreRuntime Digest 解析区块作者

依赖边界：
- `fullnode-issuance` 不直接使用 `sp-std`，Cargo 依赖保持最小化，避免为未使用 crate 传播额外 feature。

---

## 3. 存储结构
- `LastAuthoredBlockByMiner: Map<AccountId -> u32>`
  - key：矿工身份账户（出块作者）
  - value：该账户最近一次被 PoW digest 证明为区块作者的区块高度
  - 语义：作为 `bind_reward_wallet` 的链上矿工资格来源；runtime 不读取节点本地 keystore
- `RewardWalletByMiner: Map<AccountId -> AccountId>`
  - key：矿工身份账户（出块作者）
  - value：奖励到账钱包账户
  - 语义：绑定后奖励发放到钱包；未绑定时奖励发到矿工自身账户；重绑会覆盖旧值
- `RewardedBlockCount: u32`
  - 已按固定规则成功发放奖励的区块数；节点按当前高度独立计算期望值
- `TotalFullnodeIssued: Balance`
  - 全节点 PoW 奖励累计发行额；必须恒等于 `RewardedBlockCount × FULLNODE_BLOCK_REWARD`
- `LastRewardAudit: Option<(u32, AccountId, AccountId, Balance)>`
  - 最近一次奖励的区块高度、PoW 作者、实际收款账户与金额
  - 三个审计字段都不是制度真源；节点二进制中的高度范围、金额常量和 PoW digest 才是判定依据

---

## 4. 事件与错误
主要事件：
- `RewardWalletBound { miner, wallet }`
- `RewardWalletRebound { miner, new_wallet }`
- `FullnodeIssuanceIssued { block, miner, wallet, amount }`
- `FullnodeIssuanceSkippedNoAuthor { block }`

主要错误：
- `RewardWalletAlreadyBound`
- `RewardWalletNotBound`
- `RewardWalletCannotBeMiner`
- `RewardWalletUnchanged`
- `MinerNeverAuthoredBlock`

---

## 5. 对外调用（Extrinsics）
### 5.1 `bind_reward_wallet(wallet)`（call index = 0）
- 权限：`Signed`
- 逻辑：
1. 校验调用者未绑定过奖励钱包
2. 校验 `wallet != miner`
3. 校验 `LastAuthoredBlockByMiner[miner]` 已存在，证明该账户真实出过块
4. 写入 `RewardWalletByMiner`
5. 发送 `RewardWalletBound`
- weight：`T::WeightInfo::bind_reward_wallet()`

说明：
- 当前不允许从未出块的账户提前绑定奖励钱包。
- 首次出块前没有绑定时，第一笔奖励会默认发到矿工身份账户；出块记录生成后，矿工可再绑定独立奖励钱包。
- 真实矿工资格来自链上 PoW digest 解析出的出块记录，不来自节点本地 keystore 或桌面端 UI。
- 当前要求奖励钱包必须与矿工身份账户不同，避免矿工身份与收款钱包混同。

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
  - 将 `n` 饱和转换为 `u64` 后判断奖励区间，避免 pallet 对 runtime `BlockNumber` 形成 `Into<u32>` 编译期耦合。
  - 当 `n` 在奖励区间 `[FULLNODE_REWARD_START_BLOCK, FULLNODE_REWARD_END_BLOCK]` 时，返回 `T::DbWeight::get().reads_writes(5, 7)`。
  - 区间外返回 `Weight::zero()`。
- 目的：为 `on_finalize` 的最坏路径预留区块 weight 预算，避免 finalize 工作"未计重"。

### 6.2 `on_finalize(n)`: 实际奖励结算
流程：
1. 将区块高度饱和转换为 `u64` 做奖励区间判断，若不在奖励区间则直接返回。
2. 进入固定奖励区间后，再转为 `u32` 写入存储和事件字段；该区间上界 `FULLNODE_REWARD_END_BLOCK = 9,999,999` 本身在 `u32` 范围内。
3. 从 `frame_system::digest()` 读取 pre-runtime digest。
4. 通过 `T::FindAuthor::find_author(...)` 解析作者：
   - `None`：发 `FullnodeIssuanceSkippedNoAuthor { block }` 并返回。
5. 写入 `LastAuthoredBlockByMiner[author] = block`，记录该账户已真实出块。
6. 查询 `RewardWalletByMiner`：
   - `Some(wallet)`：奖励发到绑定的钱包。
   - `None`：奖励发到矿工自身账户。
7. 以 `FULLNODE_BLOCK_REWARD` 铸造并发放到收款地址（`deposit_creating`）。
8. 原子更新 `RewardedBlockCount`、`TotalFullnodeIssued` 与 `LastRewardAudit`。
9. 发 `FullnodeIssuanceIssued { block, miner, wallet, amount }`。

---

## 7. Weight 策略
当前策略：
- 用户调用（bind/rebind）使用 benchmark 生成的 `T::WeightInfo`。
- `on_finalize` 的执行预算由 `on_initialize` 统一预申报。

注意事项：
- `bind_reward_wallet` 当前路径对应 2 次读取 + 1 次写入：读取真实出块记录与既有绑定，并写入奖励钱包绑定。
- `rebind_reward_wallet` 当前路径对应 1 次读取 + 1 次写入。
- `on_finalize` 的预算通过 `on_initialize` 的 `reads_writes(5,7)` 预申报，包含三个审计字段的读写。
- `src/weights.rs` 已按新增读取做保守补偿；后续重新跑 benchmark 时应以最新结果覆盖。
- Cargo feature：`runtime-benchmarks` 会向测试/benchmark runtime 使用的 `pallet-balances` 传播；`primitives` 当前不暴露 benchmark feature，不在传播列表中。

---

## 8. 测试覆盖（当前）
`cargo test --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml` 当前 19 项通过，覆盖：
- 一次性绑定与重复绑定拒绝
- 从未真实出块的账户无法绑定奖励钱包
- 首次出块会记录 `LastAuthoredBlockByMiner`
- 矿工首次出块后可绑定奖励钱包，后续奖励转入绑定钱包
- 绑定矿工自身账户为奖励钱包会被拒绝
- 起始边界块（`1`）发放
- 结束边界块（`9_999_999`）发放
- 区间外（`0`、`end+1`）不发放
- 未绑定钱包时奖励发到矿工自身账户
- 多区块累计奖励正确
- `FindAuthor = None` 跳过事件
- 未绑定钱包时奖励发到矿工并 emit FullnodeIssuanceIssued
- `rebind` 正常路径与未绑定拒绝
- `rebind` 到矿工自身账户会被拒绝
- `rebind` 到当前已绑定钱包会被拒绝
- `rebind` 后奖励端到端流向新钱包
- `on_initialize` 区间内外 weight 声明行为
- 每块审计计数、累计发行额和最近奖励审计元组
- 无作者、区间外与多区块累计时审计状态不被错误推进

节点侧 `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard` 当前 22 项通过，其中全节点发行策略覆盖固定边界、绑定/未绑定钱包、错误奖励金额、错误余额与总发行增量、缺失总发行状态、finalize 前篡改、截止后继续发行、创世基准和 warp 累计状态。

---

## 9. 运维与审计建议
- 发行审计同时检查累计状态与事件：
  - `RewardedBlockCount`
  - `TotalFullnodeIssued`
  - `LastRewardAudit`
  - 成功：`FullnodeIssuanceIssued`
  - 跳过（无作者）：`FullnodeIssuanceSkippedNoAuthor`
- 矿工钱包管理建议：
  - 矿工启动后即可出块获得奖励（首次未绑定时默认发到矿工自身账户）。
  - 矿工至少真实出过一次块后，才能通过 `bind_reward_wallet` 绑定独立奖励钱包。
  - 钱包迁移/风险处置时使用 `rebind_reward_wallet` 主动切换收款地址。

## 10. 矿工密钥架构

### 10.1 设计原则
矿工密钥有且仅有一把，唯一来源是 keystore 文件。

- **node 进程**（Substrate 框架）：唯一生成密钥的地方
- **keystore**：唯一存储密钥的地方
- **节点桌面端**：只读取 keystore 中的密钥，不生成、不写入

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
1. 节点桌面端调用 node 的自定义 RPC `reward_bindWallet(wallet_ss58)` 或 `reward_rebindWallet(new_wallet_ss58)`
2. node 从 keystore 读取 `powr` 公钥，使用 `keystore.sr25519_sign()` 签名交易
3. 构造完整的 `UncheckedExtrinsic` 并提交到交易池

节点桌面端 **不读取私钥、不签名**，仅传入收款钱包的 SS58 地址。签名使用与出块相同的 `sp_core` 密钥推导路径，确保签名身份与出块作者身份一致。

### 10.4 节点桌面端的角色
- 只读取默认链（`citizenchain`）keystore 文件名中的公钥（`local_powr_miner_account_hex`），用于前端展示矿工身份；不遍历其他链目录，避免旧链残留 keystore 导致身份错位
- 设置奖励钱包时在同步路径提前校验 wallet != miner，避免先存后验
- 通过 `state_getStorage` 查询链上 `RewardWalletByMiner` 状态，判断是否需要 bind 或 rebind
- 所有签名和交易提交委托给 node 端 RPC

### 10.5 密钥使用流程
1. 用户首次启动节点 → node 的 `ensure_powr_key()` 生成密钥并写入 keystore
2. 节点出块 → `author_pre_digest()` 从 keystore 读取公钥作为区块作者
3. 链上 `on_finalize` 解析 PoW digest，记录 `LastAuthoredBlockByMiner`
4. 用户绑定收款地址 → 节点桌面端调用 node RPC `reward_bindWallet` → node 用 keystore 密钥签名并提交
5. 链上 `bind_reward_wallet` 校验矿工已有真实出块记录后记录映射 → 后续奖励发到绑定的收款地址

由于出块和绑定使用的是同一把 keystore 密钥（同一个 `sr25519_sign` 路径），`RewardWalletByMiner` 映射的 key 与出块作者一致，奖励能正确发到绑定的收款地址。

---

## 11. 审查结论与建议
2026-05-01 修复结论：

当前状态：
1. `bind_reward_wallet` 已要求调用账户存在真实出块记录，普通付费账户不能再批量写入永久 `RewardWalletByMiner`。
2. 已明确禁止把矿工身份账户本身作为奖励钱包，避免矿工身份与收款地址混同。
3. `rebind` 当前不设冷却期，但必须切换到不同的新钱包；重复绑定当前钱包会直接拒绝，避免无意义写操作和事件。
4. `bind_reward_wallet` 已按新增出块记录读取补偿权重；`on_finalize` 预算已通过 `on_initialize` 上调。
5. 已补 `integrity_test` 校验 `FULLNODE_BLOCK_REWARD` 必须能完整装入 runtime `Balance`，避免未来有人调整 Balance 类型后发生静默截断。
6. 已移除 `BlockNumberFor<T>: Into<u32>` trait 约束，奖励区间判断改为 `u64`，未来 runtime 区块号扩展到 `u64` 时不会先在本 pallet 编译边界失败。

## 12. 节点原生永久守卫（2026-07-10）

`citizenchain/node/src/core/node_guard/fullnode_issuance.rs` 是本制度的节点原生共识判定层，注册在统一 `NodeGuard` 内，不新增平行 `BlockImport`。

普通区块采用两阶段只读执行：

1. `initialize_block + apply_extrinsic` 得到 finalize 前状态；
2. `Core::execute_block` 得到 finalize 后状态；
3. 节点从 PoW pre-runtime digest 解析作者，并以本机编译期常量复算期望奖励块数与金额；
4. 精确比较三个审计字段和 `LastAuthoredBlockByMiner`，再把收款账户与固定金额登记到共享 `FinalizeIssuancePlan`；
5. 统一结算器把本块全节点奖励与公民认证奖励按账户汇总，精确核对 `System::Account`、
   `Balances::TotalIssuance` 及未计划账户变化；
6. 任一缺失、解码失败或差额不精确时按 fail-closed 返回 `KnownBad`。

区间外必须保持审计元组不变，且不得向共享计划登记 PoW 奖励。无 body 的普通导入无法隔离
finalize，直接 fail-closed。warp/完整状态导入校验累计块数、累计发行、最近审计、最近作者高度和
当前收款关系；它只能验证目标态自洽，历史逐块到账仍依赖已守卫节点提供的可信 finalized 状态，
不能把累计字段表述为历史密码学证明。

`Balances::TotalIssuance` 的 finalize 净变化被固定为本块全部已登记发行计划之和。公民认证发行已作为
第二个 finalize 策略接入；若未来新增其他 `on_finalize` 铸发模块，必须先作为节点永久策略显式拆分、
登记并复算，不能无声明地混入共享总发行差额。

真实验收使用当前源码 WASM 和 `/tmp` 隔离双节点：本地挖矿与网络导入均接受 block#1；`RewardedBlockCount=1`、`TotalFullnodeIssued=999900`、审计金额 `999900`、作者余额及 `Balances::TotalIssuance` 均精确增加 `999900`。测试 chainspec 只额外资助标准测试账户以通过交易池存续检查，验收后已删除。
