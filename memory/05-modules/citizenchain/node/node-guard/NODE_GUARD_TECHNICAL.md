# 节点守卫技术文档

## 1. 定位

`NodeGuard` 是公民链节点中**除宪法外**所有永久规则的唯一 `BlockImport` 包装器。永久规则必须同时满足：

- 规则合法语义不允许由 runtime 升级改变；
- 节点能从父状态、区块执行结果或完整导入态独立判定；
- 违规区块必须在进入本地规范链前被拒绝。

公民宪法是整条链最高规则，继续由独立的 `ConstitutionGuard` 执法，不属于 `NodeGuard` 内部策略。

## 2. 固定导入顺序

网络区块导入和本地挖矿导入必须使用同一类型顺序：

```text
ConstitutionGuard<NodeGuard<PowBlockImport>>
```

调用方向为：

1. `ConstitutionGuard` 最外层先检查不可修改条款及修宪凭据；
2. 宪法检查通过后，`NodeGuard` 检查其内部全部永久策略；
3. 两层均通过后，才委派给 `PowBlockImport`。

禁止把 `ConstitutionGuard` 并入 `NodeGuard`，也禁止为发行、CID、机构号等后续永久规则新增平行 `BlockImport` 包装器。

## 3. 代码边界

| 路径 | 职责 |
|---|---|
| `citizenchain/node/src/core/node_guard/mod.rs` | 统一分阶段预执行正常区块、提取 finalize 前后 storage delta、校验完整导入态、执行 fail-closed、委派内层导入 |
| `citizenchain/node/src/core/node_guard/governance_skeleton.rs` | 固定治理骨架 RAW key、SCALE 镜像、触发条件与 I1..I7 纯判定 |
| `citizenchain/node/src/core/node_guard/fullnode_issuance.rs` | 全节点 PoW 发行 RAW key、编译期公式、PoW 作者、审计推进、创世与 warp 纯判定，并向共享 finalize 计划登记奖励 |
| `citizenchain/node/src/core/node_guard/citizen_issuance.rs` | 公民认证发行 RAW key、待发队列/身份镜像、双重防重、档位公式、创世空状态与逐块纯判定 |
| `citizenchain/node/src/core/node_guard/provincialbank_interest.rs` | 43 家省储行创立质押本金、100 年固定递减利息、年度审计及共享 finalize 计划纯判定 |
| `citizenchain/node/src/core/node_guard/genesis_pallet.rs` | 三项永久创世事实、两字段一次性阶段状态机、旧出块时间字段清除与 RAW/SCALE 纯判定 |
| `citizenchain/node/src/core/node_guard/cid_lifecycle.rs` | 公民 CID、公私权机构 CID、占号/运行/永久关闭单调状态机、固定创世机构与封存账户索引的节点原生判定 |
| `citizenchain/node/src/core/constitution/guard.rs` | 独立最外层宪法守卫，不受 `NodeGuard` 内部策略注册与重排影响 |
| `citizenchain/node/src/core/service.rs` | 在网络导入与挖矿导入两处装配同一守卫顺序 |

内部策略只负责以下内容：

- 定义不依赖 runtime metadata 的 RAW storage key；
- 判断 storage delta 是否触发本策略全检；
- 从目标状态读取并判定永久不变式；
- 返回可审计的失败原因。

内部策略不得自行执行区块、提交区块或包装新的 `BlockImport`。

## 4. 正常区块路径

`NodeGuard` 对带 body 的正常区块建立两份共享只读执行视图：

1. `initialize_block + apply_extrinsic`，得到 finalize 前状态；
2. `Core::execute_block`，得到 finalize 后状态。

全节点发行、公民认证发行与省储行固定利息分别从父状态、finalize 前和 finalize 后状态验证资格与审计，
再向同一个 `FinalizeIssuancePlan` 按收款账户登记金额。统一结算器按账户汇总后核对
`System::Account` free balance 与 `Balances::TotalIssuance`，并拒绝任何未登记的 finalize
账户变化；同一账户同时命中多项固定发行时不会互相覆盖。决议发行和链上发行仍在 extrinsic
阶段按各自治理边界执行，不进入固定 finalize 计划。CID 生命周期策略只检查本块
触及的规范 RAW key，并在 `:code` 变化时枚举全部规范表复核。固定治理骨架策略在以下任一
条件命中时执行完整检查：

- storage delta 触及 `PublicAdmins` pallet 前缀；
- storage delta 触及 `:code`，即发生 runtime 升级。

若未命中，依据上一状态已经通过守卫的归纳前提走快路径。若命中，则使用“本块 delta 优先、父状态补齐”的后置状态视图检查 I1..I7。违规或无法完成检查时返回 `KnownBad`，不调用内层导入器。

带 body 的普通导入缺少任何一份执行结果、交易有效性失败或 RAW 状态无法解码时均 fail-closed。独立 `ConstitutionGuard` 为保持最高规则边界，仍执行自己的独立检查。

区块 body 必须包含 timestamp inherent 之外至少一笔用户交易。该检查在任何 runtime
预执行之前完成，使网络空块和本地 proposal 竞态优先返回 `KnownBad`。这只是提前闸门；
`pow-difficulty` runtime 同时保留最终共识断言，防止修改或绕过 NodeGuard 的节点产出空块。

## 5. warp 与完整状态导入

当 `BlockImportParams::with_state()` 为真时，节点不能依赖普通区块 delta。`NodeGuard` 必须在提交前从 `ApplyChanges(Import)` 的完整下载态抽取策略所需 RAW storage：

- 导入态满足全部节点永久策略后才委派内层导入器；
- 状态形态无法识别、关键 key 缺失、SCALE 解码失败或不变式不符时一律 fail-closed；
- 当前一次扫描同时抽取 `PublicAdmins`、`FullnodeIssuance`、`CitizenIssuance`、`GenesisPallet`、`ProvincialBankInterest`、相关 `System::Account`、`Balances::TotalIssuance` 及 CID 规范表；后续策略必须继续复用同一份完整导入态；
- CID 删除/复用属于历史单调性，非创世单快照不能证明，因此 CID 策略只允许 block#0 完整状态导入，严格拒绝非 block#0 状态导入。

完整态实现使用一次共享分区扫描：输入 key 只遍历一次，并同时抽取治理骨架、全节点发行/账户、
公民发行、创世模块、省储行固定发行和 CID 生命周期所需状态。共享分区测试证明输入 key 只扫描一次，
并覆盖统一 `KnownBad` 返回和拒绝路径内层导入零调用；真实 warp 导入和峰值内存仍必须在任务关闭前完成。

## 6. 启动锚定

`NodeGuard::new` 使用 block#0 状态校验全部已注册策略。固定治理骨架规格来自编译进节点二进制的 `primitives::governance_skeleton`；全节点发行要求创世累计块数、累计金额均为 0 且不存在最近奖励审计；公民认证发行只允许 FRAME 规范空状态；GenesisPallet 要求三个创世事实逐字等于 primitives 且阶段/开发者开关使用规范缺省形态；省储行固定发行要求 43 个 `stake_account` 的完整账户状态逐户等于 `stake_amount` 且三项年度审计为空；CID 策略建立不可改写的创世账户索引基准。全部策略都不读取可被 runtime 升级改变的 metadata。

创世状态缺少固定机构、机构码/类型/状态/名额不符、NJD 护宪席位不为 7 或 FRG 省组不完整时，节点拒绝启动。

2026-07-11 使用当前 debug 节点和独立临时 base path 的真实启动基线：49,593 个创世公权机构的 fresh
链约 47 秒后达到 `chain_getBlockHash(0)` 可用，临时数据库约 240 MiB，创世哈希为
`0x3e3a23954fbe4301fe5ccbd9bdb96c2073626c99bfb1acc4218e0a9886fdff82`。该数据只是当前机器上的
debug 基线，不替代 release 峰值内存和各路径性能矩阵。冻结网络仍为 `0xb57c…9971`，两者连接会因
genesis mismatch 被拒绝；正式部署前必须重新烘焙唯一基线，不得保留双轨或旧 SCALE 兼容。

## 7. 当前策略：固定治理骨架

本策略只冻结永不合法改变的结构，不冻结依法轮换的成员身份：

- 固定治理机构及 43 个 FRG 省组必须存在；
- 固定机构码、`PublicInstitution` 类型与 `Active` 状态不得改变；
- NRC/PRC/PRB/NJD/FRG 省组名额分别固定为 19/9/9/15/5；
- NJD 的护宪大法官席位恒为 7；
- 保持结构不变的等长合法换人允许通过。

保持席位数量但整体替换为攻击者密钥的成员劫持，不在本策略的独立判定能力内；节点没有脱离链上选举真源的合法成员预言机。

## 8. 扩展规则

新增永久策略必须按分步方案单独确认，并满足：

1. 明确规则的编译期单源、创世基准或可独立验证证明；
2. 定义正常区块触发条件和 `:code` 升级后的全检行为；
3. 定义 warp/完整状态导入的提交前校验；
4. 检查失败统一 fail-closed，并记录具体策略与失败原因；
5. 网络导入和挖矿导入共用同一 `NodeGuard`，不得形成影子路径；
6. 补齐纯策略测试、恶意区块测试、状态导入测试和真实节点验收。

## 8.1 当前策略：全节点 PoW 发行

- 金额、起止高度由节点编译期 `primitives::pow_const` 决定；
- 作者必须来自 PoW pre-runtime digest；
- finalize 前审计累计值必须与父状态一致，防止 extrinsic 或恶意 runtime 提前改写；
- 奖励区间内，累计计数、累计发行、最近审计和最近作者高度必须精确变化，并向共享 finalize 计划登记收款账户与固定金额；
- 未绑定钱包时收款账户为作者，已绑定时使用 finalize 前已生效的钱包；
- 奖励截止后不得继续铸发或改写最近奖励审计；
- `Balances::TotalIssuance` 和账户余额由共享 finalize 计划统一核对；任何新增 finalize 铸发必须先纳入节点策略复算；
- warp 只能证明下载目标态满足累计公式与最近审计自洽，不能代替历史逐块重放证明。

## 8.2 当前策略：CID 与机构生命周期

规范真源只认以下 RAW storage：

- `CitizenIdentity::CidRegistry`；
- `PublicManage/PrivateManage::CidRegisteredAccount`；
- `PublicManage/PrivateManage::Institutions`；
- `PublicManage::ProtectedGenesisAccounts` 及其创世正反向账户索引。

机构产品状态只有三种：主账户已占号而尚无机构记录表示“占号中”，`Active` 表示“运行中”，`Closed` 表示“永久关闭”。节点允许运行中机构依法更新全称和简称，也允许新 CID 使用历史名称；节点冻结的是 CID 主体和状态单调性，不冻结名称字符串。

节点逐块强制：

- 公民 CID 不得删除或换注册局、承诺、居住省市、登记高度，只允许 `Active → Revoked`，吊销后逐字冻结；
- 公私权 CID 不得重复占用；机构码、创建高度、镇码不可变；主账户登记在关闭前不得删除；
- `Institutions` 不得删除，`Closed` 不得恢复或改写，已关闭 CID 不得重新建立主账户登记；
- 固定治理机构必须始终 `Active`，运行期不得新造同类固定机构；
- block#0 封存账户集合、`AccountRegisteredCid`、`CidRegisteredAccount`、`InstitutionAccounts` 四向关系逐字冻结；
- 所有 RAW key 都校验 `Blake2_128Concat` 哈希、SCALE 值完整解码及尾随字节，畸形状态 fail-closed。

## 8.3 当前策略：公民认证发行

- runtime 身份登记回调只建立本块待发队列，实际铸发在同块 `on_finalize` 完成；
- 节点从 `primitives::citizen_const` 编译期常量、父状态累计人数和连续队列独立推导每笔金额；
- 每笔必须对应首次出现的 `VotingIdentityByAccount`，CID 哈希和 `AccountByCid` 反向索引必须闭环；
- CID 哈希和账户同时做永久与本块临时防重，禁止同块重复、跨块重复、换 CID 重领和超过总人数上限；
- finalize 前不得提前推进永久累计/防重状态，finalize 后必须精确推进并清空全部临时 key；
- 公民奖励和 PoW 奖励进入同一 `FinalizeIssuancePlan`；账户 free balance、账户其他字段及
  `Balances::TotalIssuance` 必须与汇总计划完全一致，未登记账户的 finalize 变化直接拒绝；
- 事件与 metadata 只供审计，不参与节点判定。

## 8.4 当前策略：省储行创立质押与固定利息

- 43 家省储行、`main_account`、`stake_account` 和 `stake_amount` 只取节点编译期 `CHINA_CH`；
- block#0 的 `stake_account` 完整 `System::Account` 必须逐字段等于创世本金规范，后续永久不得变化；
- 年度固定为 87,600 块，首年 100 BP、每年递减 1 BP、连续发行 100 年；
- 年度利息只能在 finalize 发到对应 `main_account`，43 笔逐户加入共享发行计划；
- `LastSettledYear`、`TotalProvincialBankInterestIssued` 和最近年度审计必须按区块高度精确连续；
- 跳年、补年、提前发行、重复发行、错误收款、错误金额、未知省储行 storage 或本金改写全部拒块；
- runtime 不再存在 Root 跳年/补发 Call，年度失败必须修复后在原边界重新正确执行；
- `:code` 变化重新检查全部本金与当前年度审计，完整状态导入只接受规范 block#0。

## 8.5 当前策略：GenesisPallet 五字段

- `CitizensDeclaration`、`CountryDeclaration`、`CitizenMax` 分别由节点编译期 `CITIZENS`、
  `COUNTRY`、`GENESIS_CITIZEN_MAX` 重构准确 SCALE，创世后任何触碰都拒绝；
- `Phase` 与 `DeveloperUpgradeEnabled` 的规范创世 RAW 形态均为缺省 key，等价于
  `(Genesis, true)`；显式写回默认值也不是合法状态；
- 唯一允许的变化是同一个含 `:code` 的 runtime 升级区块原子写成
  `(Operation, false)`；部分、普通区块、反向、二次或重新开启开发者直升全部拒绝；
- `TargetBlockTimeMs` 已从 runtime 删除，旧 key 与其它同前缀未知状态均 fail-closed；
- PoW 六分钟目标属于独立 PoW 难度规则，不再进入 Genesis/Operation 阶段状态机；
- 启动、正常区块、`:code` 后全检和完整状态共享扫描使用同一组 RAW/SCALE 规则。

## 8.6 当前策略：PoW 动态难度

- `CurrentDifficulty`、`ActiveParams`、`PendingParams`、`WindowStartBlock`、`WindowStartMs`
  和 `LastAdjustment` 全部进入 NodeGuard RAW storage 分区，未知同前缀 key 或畸形 SCALE 一律拒绝；
- 创世状态必须等于 `PowDifficultyParams::genesis_default()`、`POW_INITIAL_DIFFICULTY` 和空窗口；
- 普通区块只能按父状态 `ActiveParams` 推进窗口和难度，`CurrentDifficulty` 为 0、窗口回退、
  非调整点改难度、调整点审计不一致全部拒绝；
- 参数只能由含 `:code` 的 runtime 升级块暂存，普通区块不得直接写 `ActiveParams` 或
  `PendingParams`；
- 参数激活发生在暂存后的下一块，激活块必须保持当前难度不变、清空 pending、重置窗口；
- runtime 升级块必须同时出现 `RuntimeUpgradeAudit` 与 `PendingParams`，code hash、旧/新参数 hash、
  `activate_at`、执行路径和算法版本全部一致，否则拒绝导入；
- `params_version` 必须随参数值变化递增；`algorithm_version` 必须等于节点支持的算法版本。
- 当前自动化基线：NodeGuard 76/76，ConstitutionGuard 40/40。
- 2026-07-12 真实运行态基线：普通 release WASM fresh 双节点临时链中，无交易停在 block#0；
  Alice 真实 signed remark 交易产出 block#1
  `0xaaf286249a775bcac3bb107b7e7f4c15ccb3fb2eaebb8d0cf87e81464d7ae7fb`，
  节点 2 同步到 block#1，NodeGuard 与 ConstitutionGuard 未拒绝合法新区块。

## 9. 第 3 步验收基线

- `fullnode-issuance` runtime 测试：19 个通过；
- `node_guard` 过滤测试：22 个通过；
- `constitution` 回归测试：38 个通过；
- node `cargo check` 通过；
- node `cargo fmt --check` 通过；
- 当前源码 WASM 的 `/tmp` 隔离双节点真实产出并传播 block#1；本地挖矿路径和网络导入路径均通过相同两层守卫；
- block#1 的计数为 1、累计发行与审计金额为 999,900 分、最近作者高度为 1，矿工余额与总发行量均精确增加 999,900 分；
- 冻结 chainspec 因已登记的旧管理员 SCALE 模型风险被 `NodeGuard` fail-closed 拒绝，正式部署前必须重新烘焙，不得增加旧模型兼容；
- 活动代码和当前文档不再保留旧治理骨架包装器或旧顶层模块入口。

## 10. 第 4 步验收基线

- `node_guard` 过滤测试：31 个通过，其中 CID 策略覆盖公民吊销终态、删除/换主体、机构占号/运行/关闭、禁止恢复、名称复用、公私权冲突、固定机构与非创世状态导入；
- `constitution` 回归测试：38 个通过；
- `citizen-identity`：21 个通过；`public-manage`：40 个通过；`private-manage`：38 个通过；
- 当前 runtime 真实 block#0 含 49,593 个公权机构，CID 创世基准构造、封存索引闭环与导入态复核通过；
- `WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path citizenchain/node/Cargo.toml` 通过；
- 隔离 fresh 双节点真实链产出至 block#3：矿工节点本地产块，禁用挖矿的对等节点从另一 block#2 分叉重组并网络导入 block#3；两端最佳哈希一致为 `0xffd035479826feadab4b2a7774f63bfb8a8d66b37dd5a63308938f44ad5badd3`；
- 验收临时 chainspec、数据库和临时签名代码已全部删除，仓库未保留验收辅助文件。

## 11. 第 5 步验收基线

- `citizen-issuance`：13 个单元测试与 5 个身份集成测试通过；
- `node_guard`：38 个通过，其中覆盖合法公民发行计划、队列缺号/残留、RAW key 哈希篡改、
  创世规范空状态、矿工与公民同账户合并、未计划账户变化和新账户精确形态；
- `constitution`：38 个通过；`citizen-identity`：21 个通过；node `cargo check` 与当前源码 WASM build 通过；
- release `runtime-benchmarks` build 与 `citizen_issuance` pallet benchmark 实跑通过，生成权重记录 7 reads / 8 writes；
- fresh 隔离双节点真实产出 block#1，矿工节点与禁用挖矿的全节点最佳哈希均为
  `0x702e65e7b64ae7df80dbfb1e16e99ea9909ba302628c3c9d6fc722f6714050c5`；
- 真实身份登记后 `RewardedCount=1`、临时队列为空、身份和 CID/账户防重标记存在；Alice 同时领取
  PoW 与公民奖励共 1,999,800 分，扣身份登记费 100 分后余额净增 1,999,700 分；
- 第二轮真实链将矿工 Alice 与新公民 Bob 分离：双方 block#1 哈希均为
  `0x26d751b62ef23cc5d5884153c1782f67a5922b1d2246f16c5e610e5e034823a6`，Alice 净增
  999,800 分（PoW 999,900 - 登记费 100），Bob 新账户精确收到 999,900 分；
- 真实启动发现并修正 FRAME pallet 存储版本 0 的合法创世表示误判，非零或未知创世状态仍 fail-closed；
- 临时 chainspec、节点数据库、签名辅助和测试密钥材料均已删除。

## 12. runtime 与 node 字段契约基线（2026-07-12）

NodeGuard 不读取 runtime metadata，而是按下表硬编码 RAW key 和 SCALE 镜像；runtime 侧测试钉死
声明序/判别值，node 侧测试钉死 pallet、storage、hasher 和 key 编码。字段重排、storage 改名、
hasher 变化或 enum 重排必须先重新评估永久规则，不能让两端静默漂移。

| 策略 | runtime storage / 类型 | node 固定标准 |
|---|---|---|
| 固定治理骨架 | `PublicAdmins::AdminAccounts`、`FederalRegistryProvinceGroups`；均为 `Blake2_128Concat` 的 `AdminAccount` | `institution_code/kind/admins/status` 固定为规格机构码、`PublicInstitution=0`、固定席位、`Active=1`；NJD 护宪角色恰 7，43 个 FRG 省组各 5；成员账户允许等长轮换 |
| 全节点发行 | `RewardWalletByMiner`、`LastAuthoredBlockByMiner`、`RewardedBlockCount:u32`、`TotalFullnodeIssued:u128`、`LastRewardAudit:(u32,AccountId,AccountId,u128)` | 高度 `1..=9_999_999` 每块固定 `999_900` 分；作者、钱包、累计、审计、账户完整字段和 `Balances::TotalIssuance` 差额精确 |
| 公民发行 | `RewardedCount:u64`、CID/账户永久墓碑、`PendingRewardCount:u32`、`PendingRewards<Twox64Concat,u32,(AccountId,Hash)>`、两张临时墓碑 | 队列 `0..count-1` 连续；finalize 后临时状态清空；前 `14_436_417` 人 `999_900` 分，其后 `99_900` 分；CID 与账户均只领一次 |
| GenesisPallet | `Phase`、`DeveloperUpgradeEnabled`、`CitizensDeclaration`、`CountryDeclaration`、`CitizenMax`，`StorageVersion=0` | 三个创世事实逐字冻结；只允许含 `:code` 的 `(Genesis,true) → (Operation,false)` 原子单向转换；旧 `TargetBlockTimeMs` 与未知 key 拒绝 |
| 省储行固定发行 | pallet `StorageVersion=0`、`LastSettledYear:u32`、`TotalProvincialBankInterestIssued:u128`、`LastProvincialBankInterestAudit:(u32,u32,u128)`；43 个 `System::Account[stake_account]` | block#0 本金逐户等于 `stake_amount` 且永久不变；87,600 块/年，100→1 BP 连续 100 年；利息只发 `main_account`，审计、账户与总发行精确闭环；未知 pallet key 拒绝 |
| 公民 CID | `CitizenIdentity::CidRegistry<Blake2_128Concat,CidNumber,CidRecord>` | `registrar_account/commitment/省码/市码/registered_at` 不变；只允许 `Active=0 → Revoked=1`，吊销后冻结 |
| 机构 CID | `PublicManage/PrivateManage::{CidRegisteredAccount,AccountRegisteredCid,Institutions,InstitutionAccounts}` | CID 不删除、不跨 namespace 复用；`town_code/institution_code/created_at` 不变；法定代表人姓名/CID/账户按当前 SCALE 顺序完整解码且必须同时存在或同时为空；名称与法定代表人仅 Active 时可依法更新；`Closed=2` 永久终态 |
| 创世封存账户 | `PublicManage::ProtectedGenesisAccounts` 及三张关联索引 | 与 block#0 逐字一致、始终 Active，不得删除、换 CID、换账户名或换地址 |

共同触发口径：普通区块只检查相关 delta；`:code` 变化强制全策略复核；完整状态只扫描一次后分区；
任一 RAW key hasher 错误、SCALE 解码失败或尾随字节均 fail-closed。`System::Account` 不能只比较
`free`，`nonce/consumers/providers/sufficients/reserved/frozen/flags` 均不得被 finalize 发行顺带改写。

防漂移测试位于 runtime 既有测试模块和 node 各策略内联测试。2026-07-12 最终验收：
`admin-primitives 3/3`、`entity-primitives 2/2`、`citizen-identity 22/22`、
`citizen-issuance 14/14 + 5/5`、`fullnode-issuance 20/20`、`node_guard 76/76`、
`constitution 40/40`。

`NodeGuard` 生产路径的预计算状态变更一致性校验直接使用
`sp_state_machine::StorageChanges`，因此 `sp-state-machine` 必须属于 node 正式依赖，不能仅放在
`dev-dependencies`。生产打包和测试构建必须使用同一依赖边界，避免测试可编译而 Tauri 二进制失败。

## 13. 第 6.2 步恶意状态与包装器拒绝矩阵

2026-07-12 在字段契约基线上完成最终纯策略与统一委派闸门矩阵，`node_guard` 定向测试增至 54 个：

- 固定治理骨架覆盖固定机构缺失、类型/状态/席位变化、NJD 护宪席位稀释、FRG 省组异常、
  SCALE 尾随字节、精确 RAW key 公式及 `:code` 全检触发；等人数合法轮换继续允许。
- 全节点发行覆盖错误作者/收款人/金额/累计/审计高度/审计矿工/最近出块高度、奖励结束后继续发行、
  SCALE 尾随字节和共享发行计划登记。
- 公民发行覆盖队列缺号、残留、临时标记缺失、身份哈希、反向索引、永久墓碑、累计人数、未知 key、
  Twox64Concat 篡改及共享计划溢出/总发行/未计划账户变化。
- CID 生命周期覆盖公民删除/换主体/吊销恢复、机构码/镇码/创建高度变化、Closed 墓碑改写或恢复、
  公私权重复、固定机构状态、创世封存索引、畸形 hasher 和尾随字节。
- `import_if_verified` 统一闸门连续两次拒绝均返回 `KnownBad` 且内层调用数不增加；随后合法输入仍能
  正常委派，证明闸门没有跨块污染状态。NodeGuard 与最外层 ConstitutionGuard 均只通过该闸门委派。

本步不包含完整状态/warp 真实导入、数据库不入库或三节点分叉，这些进入后续独立步骤。

## 14. 第 6.3 步完整状态与 warp 提交前校验

完整下载态的生产校验已收敛到 `verify_imported_policy_state`：先检查 CID 导入高度，再把输入 key
只遍历一次并分区，依次验证固定治理骨架、全节点发行、公民发行、省储行固定发行和 CID 生命周期，最后返回扫描统计。
`NodeGuard::verify_imported_state` 直接调用该函数，测试与生产不再各保留一套判定。

当前自动化证明：

- 当前 runtime 真实 block#0 全 storage 可通过全部 NodeGuard 策略，且 `scanned == 输入 key 总数`；
- 删除固定治理机构、把创世 PoW 累计改为非零、加入未知公民发行 key、删除创世封存账户，分别在
  对应策略处提交前拒绝；
- 任意非 block#0 完整快照在进入分区扫描前由 CID 策略返回
  `NonGenesisStateImportForbidden`，不得为了 warp 可用性放宽历史单调性；
- `ImportedPolicyStats` 只记录总扫描数和五个策略分区数，不缓存状态或跨区块结论。

## 14.1 第 6 步真实三节点与拒绝矩阵验收（2026-07-12）

- 临时 fresh 三节点网络：A 开启 CPU PoW，B/C 禁用挖矿并通过 A 的本地 WSS peer 地址加入；
  三端均达到 `peers=2`、`isSyncing=false`。
- 第一笔 Alice 真实 `System::remark` 交易进入 block#1，A/B/C 哈希一致：
  `0xe0fccc0790f9761226865a2fa96a5eb9e19eb34169191f49faf3afee4817b3c8`。
- 网络保持运行期间重跑拒绝矩阵：NodeGuard `76/76`、ConstitutionGuard `40/40`。矩阵覆盖永久治理骨架、
  全节点发行、公民发行、省储行固定发行、CID 生命周期、PoW 动态难度、runtime 升级审计、完整状态导入
  和护宪规则；拒绝路径返回 `KnownBad` 且不委派内层导入。
- 第二笔 Alice 真实 `System::remark` 交易进入 block#2，A/B/C 哈希一致：
  `0x961012a973cf9695367037b7f9554df2ef541cda17ed5315a7c72b2600bd2a0a`；
  Alice nonce=2，pending extrinsics=0，证明拒绝矩阵后合法链继续推进。
- 本次真实网络部分没有开放生产节点任意伪造块注入接口；坏块“不委派内层、不入库”的证据来自包装器矩阵，
  网络证据负责证明三节点合法链推进和同步一致。临时 chainspec、base-path、keystore、签名器和日志已删除。
- P2P 恶意候选块注入专项探测结论：当前节点 RPC 没有 `engine_*`、manual-seal 或任意 block submit
  接口；CLI `export-blocks/import-blocks` 可走文件导入队列，但仅篡改 JSON 会退化为 header/root/编码错误，
  不能代表 NodeGuard 永久规则坏块。后续按方案 A 在测试/导入层补齐结构完整的预计算坏块 harness，
  不向生产节点开放伪造块接口。
- 已在 `citizenchain/crates/blockchain-test-harness/` 创建专用区块链测试 harness crate，并加入 workspace。
  已提供 Alice `System::remark` signed extrinsic 构造、`export-blocks` JSON lines 摘要解析和基础
  stateRoot 篡改样本生成能力；后续坏块构造与导入验收应继续沉淀到该 crate，避免污染生产 node/runtime
  路径。当前验收：`cargo check -p blockchain-test-harness` 通过，`cargo test -p blockchain-test-harness`
  6/6 通过；真实 `import-blocks` 基线中，合法 block#0 文件可导入，篡改 stateRoot 后的 block#0 文件
  以 unknown parent 拒绝。该基线仍不等同于 NodeGuard 永久规则坏块。
- 结构完整执行校验坏块基线：使用 harness CLI 产出真实 Alice remark 交易，双节点产出合法 block#1，
  `export-blocks --from 1 --to 1` 导出后，合法 block#1 可导入；仅篡改同一 block#1 的 `stateRoot` 后，
  parent 仍存在、extrinsics=2、digest_logs=2，`import-blocks` 执行 runtime 时触发
  `Storage root must match that calculated`，NodeGuard 包装路径记录“只读执行区块失败”并 fail-closed 为
  `bad block`。这证明导入队列对结构完整但执行根不一致的候选块会拒绝；它仍不是“执行后状态根合法、
  但违反 NodeGuard 永久规则”的最终坏块。
- 完整导入态永久规则坏样本矩阵已沉淀到 harness：case 清单和期望拒绝前缀由
  `blockchain-test-harness` 提供，node 内部测试使用真实创世 storage 构造坏状态并验证导入前拒绝。
  覆盖固定治理骨架、全节点发行、公民认证发行、创世模块、省储行固定发行和 CID 生命周期；验证
  `cargo test -p node node_guard` 78/78 通过。
- 导入层包装器验收已补齐：测试直接构造 `ApplyChanges(Import(...))` 的完整状态导入参数，坏状态在
  `with_state` 入口返回 `KnownBad` 且 inner import 不被调用，合法 block#0 完整状态通过后才委派一次。
  该项覆盖 warp/状态导入路径，不等同于 P2P 手工伪造块注入。
- 方案 A 已补齐普通块预计算坏块导入层验收：NodeGuard 对普通块 `ApplyChanges(Changes(...))` 不再信任
  导入方预计算结果，必须与本节点 runtime 只读重放得到的 `transaction_storage_root`、主存储变更、子存储变更
  和 offchain 存储变更逐项一致；不一致直接 fail-closed，不委派内层 import。测试使用真实
  `BlockBuilder` 构造 timestamp + Alice remark 合法 block#1，再篡改 `GenesisPallet::citizen_max`
  预计算 delta，并基于父状态重算自洽 state root 与 backend transaction；合法 proposal 通过，
  自洽坏 proposal 返回 `KnownBad` 且 inner import 计数保持 0。该能力只存在于 test-only harness，
  不暴露生产 RPC/P2P 伪造入口。
- 方案 A/方案 B debug 矩阵：`cargo test -p node node_guard -- --nocapture` 81/81 通过；带 WASM 的真实服务级用例
  `WASM_BUILD_FROM_SOURCE=1 cargo test -p node precomputed_changes_must_match_reexecuted_normal_block -- --nocapture`
  通过；`WASM_BUILD_FROM_SOURCE=1 cargo test -p node self_consistent_bad_precomputed_block_is_known_bad_before_inner_import -- --nocapture`
  通过。无 WASM 的普通测试构建会显式跳过这两条真实服务级用例，避免常规 debug 测试误依赖内置 WASM。
- 方案 B 已补齐 P2P 层测试态坏块传播拒绝验收：新增 `service/p2p_bad_block_tests.rs` test-only 服务级
  harness，不向生产 RPC/P2P 暴露任意块提交接口。恶意测试节点复用真实 `new_partial` 组件，但只使用裸
  `PowBlockImport<GrandpaBlockImport>` 写入本地 DB，刻意绕过 `NodeGuard/ConstitutionGuard`；坏块为真实
  `BlockBuilder` 生成的 timestamp + Alice remark block#1，带合法 PoW pre-runtime digest、seal、
  PoW intermediate 和自洽 state root/backend transaction，但篡改 `GenesisPallet::citizen_max` 永久规则字段。
  诚实测试节点使用生产同构的 guarded import queue 和真实 `build_network`，通过 reserved peer 连接恶意节点，
  观察到恶意 peer 的 block#1 `best_hash/best_number` 后仍保持 best=genesis，且本地数据库不存在坏块 header。
- 方案 B 验收：`WASM_BUILD_FROM_SOURCE=1 cargo test -p node
  p2p_sync_rejects_self_consistent_bad_node_guard_block -- --nocapture` 1/1 通过，运行耗时 115.92s。失败重跑产生的
  `/tmp/gmb-p2p-bad-block-*` 临时目录已清理；成功路径也会在测试尾部删除两节点唯一临时 base path。
- Release 矩阵：普通 release build 通过；带 `WASM_BUILD_FROM_SOURCE=1` 的 release build 通过并可导出
  `citizenchain-fresh`；`cargo test --release -p node node_guard` 78/78、`cargo test --release -p node
  constitution` 40/40、`cargo test --release -p citizen-identity` 22/22、`cargo test --release -p
  citizen-issuance` 14/14 + 身份集成 5/5 均通过。
- Release 真实快路径：临时 fresh 双节点 A/B 通过本地 `/ws` 互联，Alice `System::remark` 进入
  block#1，A/B 最佳哈希一致为
  `0xdcda6a5958434dcffd7e9fa1e8cde583e9cfacc177005d1d66722e3480266be9`，block#1
  extrinsics=2、digest_logs=2，Alice nonce 0→1，pending=0，守卫拒绝日志 0；临时目录已删除。

## 15. 省储行固定发行守卫验收（2026-07-12）

- Runtime 删除 `force_settle_years`、`force_advance_year` 及所有跳年/批量补发分支，不保留旧 Call；
- 年度发行迁到 `on_finalize`，新增累计发行和最近年度审计，43 笔发行在同一存储事务中原子完成；
- NodeGuard 新增独立策略，创世、普通块、`:code` 全检和完整状态导入均使用同一固定公式；
- 省储行策略覆盖 RAW key、SCALE 字段序、43 笔本金、年度边界、100 年公式、错误审计、质押改写、
  共享发行计划以及 `Balances::TotalIssuance` 闭环；
- `provincialbank-interest` 10/10，runtime-benchmarks 11/11，NodeGuard 64/64；
- 正式 benchmark 为 45 reads / 46 writes，执行时间模型约 569 ms，权重文件已重新生成；
- 当前源码 fresh headless block#0 真实启动通过，创世哈希
  `0x6fc42816b55ce22f204d0dbddbf38a9ab4d3a1c78005b90e1fcbe376ef8585b1`，临时数据库约 352 MiB；
- 没有省储行守卫拒绝或 panic，所有 `/tmp/gmb-node-guard-provincial.*` 已删除。

2026-07-12 验收：NodeGuard `57/57`。使用当前源码 WASM 和独立 `/tmp` base path 启动 fresh 节点，
约 52 秒达到 `chain_getBlockHash(0)` 可用，创世哈希
`0xbdac261dac0c76d68f7d25470d7a1332ea3a7a891f0d5d917c18afea2ec6aea4`，临时数据库约 352 MiB；
没有守卫拒绝或 panic，临时目录已删除。该启动数据是 debug 环境观测，不替代后续专项性能结论。

## 16. 固定平均六分钟与 GenesisPallet 守卫验收（2026-07-12）

- Genesis/Operation 不再拥有不同出块时间；PoW 固定以 360,000ms 作为难度调整平均目标，
  有效工作量证明找到后 CPU/GPU 立即提交，没有最短等待或最晚期限；
- GenesisPallet 删除动态时间 storage、Runtime API、trait 和死事件，NodeGuard 新增五字段策略并接入
  启动、普通区块、`:code` 全检和完整状态共享扫描；
- 空块规则采用三层防线：本地交易池门控避免构造，NodeGuard 预执行前 fail-closed，
  `pow-difficulty` runtime 保留最终共识断言；
- PoW benchmark 50 steps / 20 repeats：调整路径 3 reads / 2 writes、7µs，旧 Genesis storage proof 清除；
- `pow-difficulty` runtime-benchmarks 12/12、GenesisPallet 7/7、NodeGuard 71/71、
  ConstitutionGuard 40/40；GPU feature、try-runtime 和 production WASM build 通过；
- 双节点真实在线验收中，真实签名交易到 block#1/block#2 可见分别约 1.988 秒和 1.897 秒；
  两端 block#2 哈希一致为
  `0x993d572e4d18bdea30441c5212df76699db16b0c1bacedc3c47db0bcf9814102`；
- 当时的竞态空 proposal 被 NodeGuard 明确拒绝，随后合法区块继续传播；后续复核确认不能因此删除
  runtime 最终拒绝，现已恢复 runtime 共识断言。全部 `/tmp` chainspec、keystore 和数据库已删除。
- 恢复后的真实双节点验收进一步发现：最佳块刚切换时交易池维护存在短暂 ready 残留，runtime 会正确
  拒绝本地空 proposal，但留下无效提案日志。节点门控已增加“新链头跳过一轮”，并禁止无本地矿工
  的节点构造 proposal；第二笔真实交易产出 block#2 后不再出现空 proposal，runtime 最终断言仍保留。
- 最终生产源码 fresh block#0 启动通过，创世哈希
  `0x6d1ae7386793e966fe2f17f73446f433b3a1aecfd4dd4b9bce2764ca44d98e84`，数据库约 352 MiB。
