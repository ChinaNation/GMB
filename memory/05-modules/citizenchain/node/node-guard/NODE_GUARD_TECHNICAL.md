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

全节点发行与公民认证发行先分别从父状态、finalize 前和 finalize 后状态验证资格与审计，
再向同一个 `FinalizeIssuancePlan` 按收款账户登记金额。统一结算器按账户汇总后核对
`System::Account` free balance 与 `Balances::TotalIssuance`，并拒绝任何未登记的 finalize
账户变化；同一账户同时领取矿工与公民奖励时不会互相覆盖。CID 生命周期策略只检查本块
触及的规范 RAW key，并在 `:code` 变化时枚举全部规范表复核。固定治理骨架策略在以下任一
条件命中时执行完整检查：

- storage delta 触及 `PublicAdmins` pallet 前缀；
- storage delta 触及 `:code`，即发生 runtime 升级。

若未命中，依据上一状态已经通过守卫的归纳前提走快路径。若命中，则使用“本块 delta 优先、父状态补齐”的后置状态视图检查 I1..I7。违规或无法完成检查时返回 `KnownBad`，不调用内层导入器。

带 body 的普通导入缺少任何一份执行结果、交易有效性失败或 RAW 状态无法解码时均 fail-closed。独立 `ConstitutionGuard` 为保持最高规则边界，仍执行自己的独立检查。

## 5. warp 与完整状态导入

当 `BlockImportParams::with_state()` 为真时，节点不能依赖普通区块 delta。`NodeGuard` 必须在提交前从 `ApplyChanges(Import)` 的完整下载态抽取策略所需 RAW storage：

- 导入态满足全部节点永久策略后才委派内层导入器；
- 状态形态无法识别、关键 key 缺失、SCALE 解码失败或不变式不符时一律 fail-closed；
- 当前一次扫描同时抽取 `PublicAdmins`、`FullnodeIssuance`、`CitizenIssuance`、相关 `System::Account`、`Balances::TotalIssuance` 及 CID 规范表；后续策略必须继续复用同一份完整导入态；
- CID 删除/复用属于历史单调性，非创世单快照不能证明，因此 CID 策略只允许 block#0 完整状态导入，严格拒绝非 block#0 状态导入。

完整态实现使用一次共享分区扫描：输入 key 只遍历一次，并同时抽取治理骨架、全节点发行/账户、
公民发行和 CID 生命周期所需状态。2026-07-11 的第 6 步单元验收已经证明分区计数与输入 key 数一致，
并覆盖统一 `KnownBad` 返回和拒绝路径内层导入零调用；真实 warp 导入和峰值内存仍必须在任务关闭前完成。

## 6. 启动锚定

`NodeGuard::new` 使用 block#0 状态校验全部已注册策略。固定治理骨架规格来自编译进节点二进制的 `primitives::governance_skeleton`；全节点发行要求创世累计块数、累计金额均为 0 且不存在最近奖励审计；公民认证发行只允许 FRAME 规范空状态，即 pallet 存储版本 0、累计数和待发数为精确零值，不得存在领取标记或队列；CID 策略枚举公民登记、公私权机构、账户占号与创世封存表，建立不可改写的创世账户索引基准。全部策略都不读取可被 runtime 升级改变的 metadata。

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
