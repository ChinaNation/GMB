# 任务卡：节点守卫分阶段固化 runtime 永久规则

## 任务需求

将不能通过 runtime 升级改变的永久规则同步固化到节点原生共识导入层。

- `ConstitutionGuard` 继续作为独立、最外层、最高优先级的 `BlockImport` 包装器。
- 除宪法外的节点永久规则统一收口到 `NodeGuard`，每个正常区块只做一次共享预执行。
- 每一步先输出完整技术方案并取得确认，再执行代码、文档、注释、残留清理和真实验收。
- 开发者直升不属于本任务范围；节点守卫只验证升级后是否突破永久规则。

## 所属模块

- `citizenchain/node`
- 后续经单独 runtime 二次确认后可能涉及 `citizenchain/runtime`
- `memory`

## 输入文档

- `memory/04-decisions/ADR-027-legislation-yuan.md`
- `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/primitives/PRIMITIVES_TECHNICAL.md`
- `memory/07-ai/module-checklists/citizenchain.md`
- `memory/07-ai/module-definition-of-done/citizenchain.md`

## 分阶段计划

1. 建立统一 `NodeGuard`，迁入现有固定治理骨架守卫。
2. 单独加固 `ConstitutionGuard`，保持独立包装器。
3. 将全节点 PoW 发行永久规则接入 `NodeGuard`。
4. 将 CID/机构号不可复用、固定机构不可关闭接入 `NodeGuard`。
5. 评估并接入公民认证发行及其余正式确认的永久规则。
6. 完成恶意状态、warp、多节点分叉和性能总验收。

## 第 1 步目标

- 新建 `citizenchain/node/src/core/node_guard/`。
- 新建 `NodeGuard<I>` 通用 `BlockImport` 包装器。
- 把固定治理骨架迁为 `node_guard::governance_skeleton` 纯策略。
- 导入顺序统一为 `ConstitutionGuard<NodeGuard<PowBlockImport>>`。
- 网络导入与本地挖矿使用同一顺序。
- 删除旧治理骨架独立包装器及旧顶层模块残留。
- 本步不修改 `citizenchain/runtime/`，不改变既有治理骨架规则语义。

## 第 1 步预计修改目录

- `citizenchain/node/src/core/`
  - 建立统一节点守卫代码边界、迁移治理骨架策略、调整两处导入装配并清理旧包装器。
- `memory/04-decisions/`
  - 更新宪法独立守卫与统一节点守卫的架构口径。
- `memory/05-modules/citizenchain/node/`
  - 更新节点技术文档并建立节点守卫专属技术文档。
- `memory/07-ai/`
  - 登记 `node_guard` / `NodeGuard` 新命名。
- `memory/08-tasks/open/`
  - 记录分阶段确认、执行和验收结果。

## 第 1 步验收标准

- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`
- `cargo check --manifest-path citizenchain/node/Cargo.toml`
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`
- 真实 headless 节点使用临时 base path 启动，`chain_getBlockHash(0)` 与 `constitution_getDocument` 成功。
- 活动代码与当前文档中的旧治理骨架包装器、旧顶层模块入口残留为 0。
- 文档已更新，关键逻辑已补中文注释，旧文件和旧口径已清理。

## 进度

- [x] 总体方案确认
- [x] 第 1 步技术方案确认
- [x] 第 1 步创建 `NodeGuard`
- [x] 第 1 步迁移治理骨架策略
- [x] 第 1 步调整导入顺序
- [x] 第 1 步测试与真实运行态验收
- [x] 第 1 步更新文档、完善注释、清理残留
- [x] 输出第 2 步完整技术方案并等待确认
- [x] 第 2 步拆分宪法展示与独立守卫
- [x] 第 2 步补齐 manifest、身份、哈希、条号和历史版本不变式
- [x] 第 2 步收紧正常、预计算 delta 与 warp 导入形态
- [x] 第 2 步测试与真实运行态验收
- [x] 第 2 步更新文档、完善注释、清理残留
- [x] 输出第 3 步完整技术方案并等待确认
- [x] 第 3 步 runtime 增加最小发行审计状态
- [x] 第 3 步全节点 PoW 发行接入 `NodeGuard`
- [x] 第 3 步普通区块分阶段执行与 warp 累计校验
- [x] 第 3 步测试与隔离双节点真实出块验收
- [x] 第 3 步更新文档、完善注释、清理残留
- [x] 输出第 4 步完整技术方案并等待确认
- [x] 第 4 步 CID/机构生命周期接入 `NodeGuard`
- [x] 第 4 步固定创世机构与封存账户索引加锚
- [x] 第 4 步单元、回归与隔离双节点真实导入验收
- [x] 第 4 步更新文档、完善注释、清理残留
- [x] 输出第 5 步完整技术方案并等待确认
- [x] 第 5 步技术方案与 runtime 五个路径二次确认
- [x] 第 5 步 runtime 同块 finalize 待发队列与 benchmark 权重
- [x] 第 5 步公民认证发行接入 `NodeGuard` 共享发行计划
- [x] 第 5 步单元、回归、真实双节点导入与余额闭环验收
- [x] 第 5 步更新文档、完善注释、清理残留
- [x] 输出第 6 步完整技术方案并等待确认
- [x] 第 6 步技术方案确认
- [x] 第 6 步恶意状态与包装器拒绝矩阵
- [ ] 第 6 步 warp、三节点分叉与恶意链真实验收
- [ ] 第 6 步性能与部署基线验收
- [ ] 第 6 步文档、注释、残留清理与任务归档

## 硬边界

- 宪法守卫不得并入 `NodeGuard`。
- 不保留旧治理骨架包装器兼容层或影子导入路径。
- 未取得对应步骤确认前不得提前实施后续规则。
- 任何 `citizenchain/runtime/` 修改必须另行列出完整路径并取得 runtime 二次确认。
- 不触碰或覆盖工作区中与本任务无关的用户改动。

## 第 1 步执行结果（2026-07-10）

### 代码与架构

- 建立 `node_guard::NodeGuard<I>`，统一正常区块预执行、storage delta、warp 完整态校验和 fail-closed。
- 固定治理骨架已迁为 `node_guard::governance_skeleton` 纯策略；旧独立包装器和旧顶层模块已删除。
- 网络导入与本地挖矿均固定为 `ConstitutionGuard<NodeGuard<PowBlockImport>>`。
- 本步没有修改 `citizenchain/runtime/` 源码，也没有引入兼容层或影子导入路径。

### 自动化验收

- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：11/11 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：30/30 通过。
- `cargo check --manifest-path citizenchain/node/Cargo.toml`：通过。
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`：通过。
- `git diff --check`：通过。
- 活动代码与当前技术文档中的旧治理骨架包装器、旧顶层模块入口残留为 0。

### 真实运行态验收

- 使用临时 base path、独立 P2P/RPC/Prometheus 端口和 `--mining-threads 0` 启动当前源码 `citizenchain-fresh`：成功运行到 block#0。
- `chain_getBlockHash(0)` 返回 `0xdc396b367c86adbffb29bd930ec25d2656f1fdf7fee2cf2ef5f04a4d4283dfd2`。
- `constitution_getDocument` 无 RPC error，返回 `source=legislation-raw`、`blake2_256=0x6639178146ab4ab38f0306158147da6378a88aee272305347502d8a35aa40b57`、HTML 长度 887,475。
- 节点收到 Ctrl-C 后正常退出；三处 `/tmp/gmb-node-guard-step1-*` 临时目录全部删除。

### 冻结 chainspec 部署风险

- 默认冻结 chainspec（创世哈希 `0xb57c…9971`）真实启动时，`NodeGuard::new` 在 NRC 管理员记录上返回 `AdminAccountDecodeFailed` 并按 fail-closed 拒绝启动。
- 该问题与固定治理骨架任务已记录的风险一致：冻结 chainspec 使用旧 `AdminAccount` SCALE 字段模型，而当前源码/当前 fresh 创世使用新模型；本次包装器迁移没有改变治理骨架解码或规则语义。
- 不允许为旧冻结状态增加兼容解码。正式部署前必须按预上线重新创世流程重新烘焙 chainspec、创世状态包及关联轻客户端资产。

## 第 2 步执行结果（2026-07-10）

### 代码与规则

- 原 `core/constitution.rs` 拆为 `constitution/mod.rs`、`guard.rs`、`render.rs` 和同目录 HTML 外壳。
- `ConstitutionGuard` 继续独立导出，网络与挖矿装配仍为 `ConstitutionGuard<NodeGuard<PowBlockImport>>`。
- block#0 启动新增完整不变式和真实历史版本 key 集检查。
- 运行期新增 manifest 逐字冻结、Law/版本身份、严格状态指针、内容哈希、条号唯一性、历史版本修改和隐藏版本检查。
- 无 body 的预计算 delta 现在必须检查；无 body 的执行型导入 fail-closed；warp 提交前检查全部版本。
- 不按恶意 `latest_version` 做超大循环，避免节点守卫自身成为 CPU DoS 面。
- 本步没有修改 `citizenchain/runtime/`，没有改变宪法正文或 runtime 业务流程。

### 验收

- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：38/38 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：11/11 通过。
- node `cargo check`、`cargo fmt --check`：通过。
- fresh headless 节点成功启动到 block#0；创世哈希仍为 `0xdc396b367c86adbffb29bd930ec25d2656f1fdf7fee2cf2ef5f04a4d4283dfd2`。
- 宪法 RPC 返回 `source=legislation-raw`、`blake2_256=0x6639178146ab4ab38f0306158147da6378a88aee272305347502d8a35aa40b57`、HTML 长度 887,475。
- 节点正常退出，`/tmp/gmb-constitution-guard-step2-20260710` 已删除。

### 明确信任上限

- 不可修改八条由 block#0 内容基准和节点原生二进制逐字保证，是本步真正的节点死规则。
- 公投与护宪凭据目前只保存计数，不含节点可独立验证的签名证明；本步保留一致性和阈值检查，但不再把它表述为完全恶意 runtime 下的密码学保证。

## 第 3 步执行结果（2026-07-10）

### 代码与规则

- runtime `fullnode-issuance` 新增 `RewardedBlockCount`、`TotalFullnodeIssued`、`LastRewardAudit` 三个最小审计状态；实际到账后才原子推进，制度真源仍是编译期常量与 PoW digest。
- `node_guard::fullnode_issuance` 使用 RAW key 和节点 SCALE 镜像，不读取 runtime metadata。
- `NodeGuard` 对普通区块共享两阶段只读执行：`initialize + extrinsics` 隔离 finalize 前状态，完整 `execute_block` 得到 finalize 后状态。
- 奖励区间内同时核对作者、收款钱包、累计公式、最近审计、最近作者、账户 free balance 和 `Balances::TotalIssuance` 的精确差额；区间外禁止继续铸发或改审计。
- 普通区块缺 body、作者缺失、执行失败、关键状态缺失/解码失败均 fail-closed；warp 与启动路径也注册发行策略。
- 未增加发行专用 `BlockImport`，网络与挖矿顺序仍为 `ConstitutionGuard<NodeGuard<PowBlockImport>>`。

### 自动化验收

- `cargo test --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml`：19/19 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：22/22 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：38/38 通过。
- `cargo check --manifest-path citizenchain/node/Cargo.toml`：通过。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path citizenchain/node/Cargo.toml`：通过并生成包含审计状态的当前 runtime WASM。

### 真实运行态验收

- 为避免使用任何现网密钥，在 `/tmp` 从当前 fresh chainspec 导出隔离验收链，仅额外资助标准测试账户 `//Alice` 10,000,000 分，并预置为临时 `powr` 矿工；WASM、宪法、治理骨架与发行规则保持当前源码值。
- 两个 headless 节点以 WSS 本地互联，矿工节点提交免费 `System::remark` 后产出 block#1；对等节点同步到 `0x1`，同时覆盖本地挖矿和网络导入两条守卫路径。
- block#1：`RewardedBlockCount=1`、`TotalFullnodeIssued=999900`、`LastRewardAudit=(1, Alice, Alice, 999900)`、`LastAuthoredBlock=1`。
- `Balances::TotalIssuance` 从 `48,451,807,756,600` 增至 `48,451,808,756,500`；Alice free balance 从 `10,000,000` 增至 `10,999,900`，两者差额均精确为 `999,900`。
- 节点均正常关闭，`/tmp/gmb-node-guard-step3-*`、临时 chainspec 和一次性签名器全部删除；未读取或使用现网/用户私钥。

### 明确信任上限

- warp 目标态可校验累计公式、最近审计和当前收款状态自洽，但无法仅凭一个状态快照证明全部历史区块逐笔到账；历史保证来自守卫节点逐块执行或可信 finalized 状态提供方，不把累计字段表述为密码学历史证明。

## 第 4 步执行结果（2026-07-10）

### 代码与规则

- 新建 `node_guard::cid_lifecycle`，使用节点原生 RAW key、SCALE 镜像和 block#0 基准，不读取 runtime metadata。
- 公民 `CidRegistry` 写入后不得删除或换注册局、档案承诺、居住省市和登记高度；只允许 `Active → Revoked`，吊销后永久终态。
- 机构统一为占号中、运行中、永久关闭三种业务状态：主账户登记存在而 `Institutions` 尚不存在即占号中，`Active` 即运行中，`Closed` 即永久关闭。
- 机构 CID 不得删除、跨公私权重复、换机构码/创建高度/镇码或关闭后恢复；主账户占号不得在关闭前删除。运行中机构名称允许依法更新，新 CID 允许使用历史名称。
- 固定治理机构只能来自 block#0 且必须永远 `Active`；`ProtectedGenesisAccounts` 及其 `AccountRegisteredCid`、`CidRegisteredAccount`、`InstitutionAccounts` 索引以创世逐字冻结。
- `:code` 变化时枚举全部 CID 规范表全检；非 block#0 状态导入不能证明历史单调性，严格拒绝。
- 本步只修改 `citizenchain/node/` 与文档，没有修改 `citizenchain/runtime/` 源码，没有增加兼容分支或平行 `BlockImport`。

### 自动化验收

- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：31/31 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：38/38 通过。
- `cargo test --manifest-path citizenchain/runtime/otherpallet/citizen-identity/Cargo.toml`：21/21 通过。
- `cargo test --manifest-path citizenchain/runtime/entity/public-manage/Cargo.toml`：40/40 通过。
- `cargo test --manifest-path citizenchain/runtime/entity/private-manage/Cargo.toml`：38/38 通过。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path citizenchain/node/Cargo.toml`：通过。
- 真实 runtime block#0 的 49,593 个公权机构、CID 主账户登记与创世封存索引完整通过节点基准构造和导入态复核。

### 真实运行态验收

- 从当前 `citizenchain-fresh` 导出隔离 chainspec，只临时资助标准测试账户 `//Alice`，不读取或使用任何现网/用户私钥。
- 双节点先覆盖同高度分叉导入，再把对等节点以 `--mining-threads 0` 重启；矿工节点本地产出 block#3，对等节点从另一 block#2 分叉重组并网络导入 block#3。
- 两端 `chain_getBlockHash` 最终一致为 `0xffd035479826feadab4b2a7774f63bfb8a8d66b37dd5a63308938f44ad5badd3`，本地产块与网络导入均通过 `ConstitutionGuard<NodeGuard<PowBlockImport>>`。
- 交易池清空后 runtime 按既有规则拒绝空块提案，该日志不是守卫失败，也没有产生额外区块。
- 两个节点正常关闭，临时 chainspec、数据库与临时签名代码全部删除；仓库没有遗留验收辅助文件。

## 第 5 步技术方案（已确认并执行）

### 目标

把公民轻节点认证发行变成 runtime 升级不能改变的节点永久规则，同时完成剩余候选规则的正式归类；本步不把尚有合法治理出口或尚未由用户确认的经济参数擅自冻结。

### 关键技术结论

当前公民认证奖励在身份登记 extrinsic 内立即铸发。只比较整块父/后状态时，同块转账、手续费和其他余额变化会与奖励混合，节点无法严格证明每名公民实际收到精确金额。为获得可独立验证的证据，方案把“资格排队”留在登记 extrinsic，把实际铸发移动到同一块 `on_finalize`：

1. 登记回调按现有双重防重、人数上限和奖励档位规则写入本块待发队列；
2. `NodeGuard` 从 finalize 前视图读取队列，按父状态独立验证首次身份、CID 哈希、账户、防重、人数与档位；
3. runtime 在 `on_finalize` 逐笔发放并清空队列，最终链上语义仍是“登记成功的同一区块到账”；
4. 节点把全节点奖励与公民奖励合并为一个 finalize 发行计划，按账户汇总后核对余额、`Balances::TotalIssuance` 和全部审计表，禁止未登记的 finalize 账户变化；同一账户同时是矿工和新公民时也能正确相加，不会误拒。

### runtime 预计改动

- `citizenchain/runtime/issuance/citizen-issuance/src/lib.rs`
  - 增加本块待发序号、待发记录和临时双重防重表；回调只排队/跳过，`on_finalize` 执行实际铸发、累计计数和永久防重写入，随后清空全部临时项。
- `citizenchain/runtime/issuance/citizen-issuance/src/tests/mod.rs`
  - 更新同块 finalize 时序测试，补队列完整清理、同块重复、档位跨界、上限和终态断言。
- `citizenchain/runtime/issuance/citizen-issuance/tests/integration_citizen_identity.rs`
  - 验证身份登记后 finalize 同块到账、事件 phase 和永久防重闭环。
- `citizenchain/runtime/issuance/citizen-issuance/src/benchmarks.rs`
  - 覆盖排队与 finalize 实际存储/余额成本。
- `citizenchain/runtime/issuance/citizen-issuance/src/weights.rs`
  - 用重新生成的 benchmark 权重替换旧立即铸发权重，不手写猜测权重。

以上五个 runtime 路径在实施前必须取得单独的 runtime 二次确认；不修改奖励金额、人数阈值、总人数上限或一次性规则常量。

### node 预计改动

- 新建 `citizenchain/node/src/core/node_guard/citizen_issuance.rs`
  - 定义 RAW key、待发/身份 SCALE 镜像、编译期奖励公式、创世检查、逐块资格与发行计划判定。
- 修改 `citizenchain/node/src/core/node_guard/mod.rs`
  - 注册公民发行策略；合并全节点和公民 finalize 发行计划；统一核对收款账户与总发行，并拒绝任何未登记的 finalize 账户变化。
- 修改 `citizenchain/node/src/core/node_guard/fullnode_issuance.rs`
  - 保留作者、钱包、区间和审计规则，只把余额/总发行最终核对交给共享发行计划，解决同一收款账户的合法叠加。

### 剩余候选规则归类

- **省储行利息**：固定账户、利率和年限具备候选价值，但当前 `force_advance_year` 允许 Root 合法跳过到期年度，因此现状不是“必须发行”的永久规则。本步只登记冲突，不接入守卫；需用户另行决定保留故障跳过权，还是把年度利息改成不可跳过后再单独设计。
- **resolution/onchain 发行**：本质是治理决议结果，不冻结为固定发行。
- **交易费、分账比例、PoW 难度算法**：当前没有“永不可改”的正式确认，且存在演进需要，不接入。
- **创世发行**：只存在于 block#0，已经由冻结创世锚点与启动校验约束，不重复增加运行期策略。
- **固定治理成员阈值**：现有节点守卫只冻结可独立验证的结构与席位数；成员合法性/投票证明仍受既有信任上限约束，本步不伪装成节点可独立证明。

### 验收

- citizen-issuance 单元和身份集成测试全部通过；新增恶意队列、错误档位、错误金额、错收款人、重复领取、越上限和残留临时项测试。
- node_guard/constitution 全量回归通过，fullnode 现有规则不退化。
- `cargo check`、`cargo fmt --check`、`git diff --check` 通过，runtime benchmark 权重重新生成。
- fresh 双节点真实注册一名公民身份并在同一 block finalize 到账；矿工与公民为同一账户、不同账户各做一次；对等节点网络导入后余额、计数、防重和链头一致。
- 临时链、测试密钥与验收辅助全部清理，文档、注释和候选规则归类同步更新。

### 第 5 步预计修改目录

- `citizenchain/runtime/issuance/citizen-issuance/`
  - **代码/测试/权重**：把立即铸发改为同块 finalize 可验证队列，更新单元、集成与 benchmark；不修改制度常量。
- `citizenchain/node/src/core/node_guard/`
  - **代码/新增策略/残留清理**：新增公民发行策略并统一 finalize 发行核算，清理发行策略内重复的余额核对逻辑。
- `memory/04-decisions/`
  - **文档**：登记公民认证发行的节点永久语义和省储行利息的制度冲突结论。
- `memory/05-modules/citizenchain/`
  - **文档**：更新 node、node-guard、citizen-issuance 技术说明、事件时序和验收基线。
- `memory/08-tasks/open/`
  - **文档**：记录第 5 步双确认、执行、测试、真实验收和候选规则归类结果。

## 第 5 步执行结果（2026-07-10）

### runtime 与节点规则

- `citizen-issuance` 回调改为本块资格排队，实际奖励在同块 `on_finalize` 逐项铸发；新增连续队列、
  CID 哈希/账户临时防重和 finalize 后完整清理，永久状态仍只有累计人数与双重领取墓碑。
- 节点新增 `node_guard::citizen_issuance`，用 RAW key、节点 SCALE 镜像和编译期
  `primitives::citizen_const` 复核首次身份、CID 哈希、反向索引、队列连续性、防重、人数上限与档位。
- `fullnode_issuance` 不再独占账户/总发行差额；全节点与公民奖励统一登记到
  `FinalizeIssuancePlan`，按账户合并后核对 `System::Account` 与 `Balances::TotalIssuance`，
  同一账户同时领取两类奖励也必须精确相加。
- 创世公民发行只接受 FRAME 规范空状态：pallet 存储版本 0、累计/待发计数零值；任何领取标记、
  队列、非零计数或未知 key 仍 fail-closed。
- 省储行利息因 Root 可跳年而不具备“必须发行”语义；resolution/onchain 发行是治理结果；交易费、
  分账、PoW 难度没有永久冻结确认。本步均不擅自纳入守卫。

### 自动化与 benchmark

- `citizen-issuance`：13/13；身份集成：5/5。
- `node_guard`：38/38；`constitution`：38/38；`citizen-identity`：21/21。
- node `cargo check`、当前源码 WASM build、runtime benchmark feature check 通过。
- release runtime-benchmarks build 通过；`citizen_issuance` pallet benchmark 以 50 steps / 20 repeats
  实跑，生成权重记录 7 reads、8 writes、估算 proof size 3,593 bytes。

### 真实运行态验收

- 从当前源码 WASM 创建 `/tmp` 隔离链，只资助标准测试账户 `//Alice`，并临时把 Alice 放入 GD
  联邦注册局省组管理员席位；不读取或使用任何现网/用户私钥。
- Alice 连续提交 `occupy_cid` 与 `register_voting_identity`，CID 为
  `GD000-CTZN6-616532784-2026`；矿工节点产出 block#1，禁用挖矿的全节点经 WSS 网络导入。
- 两端 block#1 哈希一致：
  `0x702e65e7b64ae7df80dbfb1e16e99ea9909ba302628c3c9d6fc722f6714050c5`。
- `RewardedCount=1`，`PendingRewardCount` 不存在，身份、CID 哈希领取墓碑和账户领取墓碑均存在。
- Alice 同时是矿工和新公民：PoW 奖励 999,900 + 公民奖励 999,900；身份登记制度费 100，
  余额由 1,000,000,000,000 增至 1,000,001,999,700，净增 1,999,700，闭环一致。
- 按已确认验收口径再执行不同账户场景：Alice 出块并代注册，Bob 以自己的公民签名首次登记；两端
  block#1 哈希一致为 `0x26d751b62ef23cc5d5884153c1782f67a5922b1d2246f16c5e610e5e034823a6`。
  Alice 获 PoW 奖励并支付 100 分登记费后净增 999,800 分，Bob 原本不存在的新账户精确收到
  公民奖励 999,900 分。
- 临时 chainspec、数据库、签名器、验收测试和测试密钥材料全部删除；chain-signing 与治理骨架文件
  恢复到验收前状态，没有留下辅助代码或兼容分支。

## 第 6 步技术方案（已确认，执行中）

### 目标

对已经落地的 `ConstitutionGuard<NodeGuard<PowBlockImport>>` 做最终安全与性能总验收。本步原则上
不新增业务永久规则、不改变制度常量，重点证明恶意状态、导入形态、分叉重组和大创世规模下没有
旁路、误放行或不可接受的资源放大。

### 1. 恶意状态矩阵

- 为宪法、固定治理骨架、全节点发行、公民发行和 CID 生命周期建立统一拒绝矩阵；每个策略至少覆盖
  值篡改、删除、未知 key、SCALE 尾随字节、map hasher 篡改、`:code` 升级触发和执行失败。
- 补共享发行计划组合测试：矿工/公民同账户、不同账户、多名公民、账户新建、已有账户、溢出、
  总发行差额错误、未计划账户变化和非 free 字段变化。
- 所有拒绝必须发生在内层 `PowBlockImport` 之前，并断言返回 `KnownBad`，避免只测纯函数而漏掉包装器行为。

### 2. warp 与完整状态导入

- 构造 block#0 合法完整态、缺 key、坏 key、错误累计、错误审计和恶意宪法版本集合，验证提交前拒绝。
- 明确验证 CID 历史单调性无法由单快照证明时，非 block#0 完整状态导入始终拒绝；不得为了 warp
  可用性放宽这一永久规则。
- 对合法 block#0 状态导入记录一次共享扫描次数和抽取规模，确认各策略复用同一 `ImportedState`，
  不产生每策略一遍全库扫描。

### 3. 多节点分叉与导入路径

- 建立至少三个隔离节点：两个矿工制造同高度分叉，一个禁用挖矿的全节点只负责网络导入；
- 让较长合法链触发重组，确认本地产块、网络导入、侧链导入和重组后的每个块都经过同一两层守卫；
- 注入一条能通过 runtime 执行但违反节点永久规则的候选分叉，确认守卫节点不入库、不切最佳链，
  并继续跟随合法分叉；
- 核对三端最佳哈希、余额、发行审计、CID 墓碑、宪法版本及固定治理骨架一致。

### 4. 性能与资源上限

- 在当前 49,593 个创世公权机构规模下测量：节点启动 block#0 守卫耗时、普通空触发快路径、普通
  身份登记块、`:code` 全检块和合法完整状态导入的耗时与峰值内存。
- 检查所有枚举均由真实 key 集驱动并设置边界，不按恶意计数/版本号做超大循环；记录最坏路径，
  对明显重复扫描做同一步内收敛。
- 性能优化只能减少重复读取/分配，不得缓存可被重组污染的未确认结论，也不得降低 fail-closed 强度。

### 5. 文档、残留与完成口径

- 把最终威胁矩阵、导入形态、信任上限、真实性能数据和运维要求写回节点守卫技术文档与 ADR；
- 清理临时恶意 chainspec、节点数据库、测试签名器、日志和所有一次性测试代码；
- 全仓搜索旧包装器、影子导入路径、旧发行独占总量口径和“关闭可恢复”等残留；
- 自动化、真实三节点验收、格式检查与 `git diff --check` 全部通过后，才把本任务卡移入 done。

### 第 6 步预计修改目录

- `citizenchain/node/src/core/node_guard/`
  - **代码/测试/性能收敛**：补齐恶意状态、共享发行组合、warp、包装器 `KnownBad` 与扫描计数测试；
    只在实测证明存在重复扫描时优化，不增加新制度规则。
- `citizenchain/node/src/core/constitution/`
  - **测试/边界复核**：补恶意版本集合、预计算 delta、warp 提交前拒绝和分叉导入回归；宪法守卫仍
    独立，不并入 `NodeGuard`。
- `citizenchain/node/src/core/service.rs`
  - **装配复核/必要注释**：只验证并注释网络、本地挖矿、侧链与重组共用同一包装顺序；无证据不改服务结构。
- `citizenchain/node/tests/`（仅当现有内联测试无法表达真实导入矩阵时才考虑）
  - **可能的新测试代码**：如确需新增集成测试文件，将先另列完整文件路径、用途、原因和 Git 跟踪状态，
    再取得新增文件确认；当前确认不自动授权创建。
- `memory/04-decisions/`
  - **文档**：更新最终威胁模型、warp 信任上限、分叉行为和性能决策，不新增兼容口径。
- `memory/05-modules/citizenchain/node/`
  - **文档/残留清理**：更新 node、constitution-guard、node-guard 的总验收基线、性能数据与运维要求。
- `memory/08-tasks/open/`、`memory/08-tasks/done/`
  - **任务文档/移动**：记录第 6 步确认与全部证据；验收完成后移动现有任务卡到 done，不新建任务卡。

### 预计不修改目录

- `citizenchain/runtime/`：第 6 步默认不修改 runtime；如真实验收发现必须调整，将停止执行，另列完整
  runtime 路径、改动和原因，重新取得 runtime 二次确认。
- `citizenapp/`、`citizenwallet/`、`citizenchain/onchina/`：不属于本步范围。

### 第 6 步验收标准

- 全部现有 node/runtime 回归通过，新增恶意矩阵与包装器拒绝测试通过；
- 合法 block#0 完整态可验，所有恶意完整态提交前拒绝，非 block#0 CID 状态导入继续拒绝；
- 三节点分叉、重组、恶意候选链隔离与合法链继续推进均真实完成；
- 输出启动、快路径、全检、身份发行块和完整态导入的耗时/内存数据；
- node `cargo check`、当前源码 WASM build、`cargo fmt --check`、`git diff --check` 通过；
- 文档、中文注释和残留清理完成，任务卡移动到 done，仓库不保留任何验收辅助文件。

## 第 6 步阶段执行记录（2026-07-11，执行中）

### 已完成的自动化基线

- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：47/47 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：39/39 通过。
- `cargo check --manifest-path citizenchain/node/Cargo.toml`：通过。
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`：通过。
- 守卫目录与任务卡 `git diff --check`：通过。
- 当前第 6 步代码已覆盖完整 SCALE 消费、畸形 RAW key、未知发行 key、共享发行计划溢出、
  总发行错误、未计划账户变化、非 free 字段变化、统一 `KnownBad` 闸门及内层导入零调用证明。
- 完整状态已收敛为一次 `partition_imported_state` 扫描，再把同一分区结果交给治理骨架、全节点发行、
  公民发行和 CID 生命周期策略；单元测试验证 5 个输入 key 只计数扫描 5 次，不按策略重复遍历输入。

### fresh 创世真实启动基线

- 使用 `CITIZENCHAIN_HEADLESS=1`、独立 `/tmp` base path、独立 P2P/RPC/Prometheus 端口启动当前
  debug 节点，不读取用户或生产密钥。
- 当前 49,593 个创世公权机构的 fresh 链从进程启动到 `chain_getBlockHash(0)` RPC 可用耗时约 47 秒。
- 本次 fresh 创世哈希为
  `0x3e3a23954fbe4301fe5ccbd9bdb96c2073626c99bfb1acc4218e0a9886fdff82`；临时数据库约 240 MiB。
- fresh 节点连接冻结主网 bootnode 时，双方明确报告 genesis mismatch：fresh 为 `0x3e3a…ff82`，
  冻结网络为 `0xb57c…9971`。因此当前源码不能直接加入冻结网络，正式部署基线仍未完成。
- 节点停止后已删除 `/tmp/gmb-nodeguard-perf.*` 临时目录；仓库没有新增验收文件。

### 当前仍未满足的关闭条件

- 尚未完成三节点真实分叉、恶意候选链不入库及拒绝后继续跟随合法链验收。
- 尚未完成 release 构建下的峰值内存、普通快路径、身份登记块、`:code` 全检和完整状态导入性能矩阵。
- 尚未重新烘焙并替换正式 chainspec、创世状态包和 CitizenApp 轻客户端资产；不得增加旧格式兼容。
- 在上述项目完成前，本任务继续保留在 `open`，不得移动到 `done`。

## 第 6.1 步 runtime/node 字段契约对齐结果（2026-07-12）

### 代码与契约

- runtime 业务逻辑、storage、制度常量和权重均未修改；仅在既有测试模块增加防漂移断言。
- `AdminAccount` 字段序、管理员 kind/status 判别值、护宪职务单源已与治理骨架镜像对齐。
- `InstitutionInfo` 字段序及 `Pending=0/Active=1/Closed=2` 已与机构生命周期镜像对齐。
- `CidRecord` 七字段声明序及 `Active=0/Revoked=1` 已与公民 CID 镜像对齐。
- `PendingCertificationReward=(who,cid_number_hash)` 与 `PendingRewards` 的 Twox64Concat key 已两端钉死。
- `LastRewardAudit=(block,miner,wallet,amount)`、奖励钱包和 `Balances::TotalIssuance` RAW key 已钉死。
- `LawVersion` 十字段声明序、Tier/LawStatus/VoteType 判别值和创世 Law[0] 已由 runtime 测试钉死，
  ConstitutionGuard 真实创世测试继续验证 node 镜像。
- NodeGuard 管理员 key 测试不再自比较，现按 pallet、storage 名和 `Blake2_128Concat` 公式构造期望值。
- 清理 `fullnode_issuance.rs` 未使用的 `MAccountData` 导入；新增测试均有中文注释，没有新增文件。

### 验收

- `admin-primitives`：3/3；`entity-primitives`：2/2；`citizen-identity`：22/22。
- `citizen-issuance`：14/14，身份集成 5/5；`fullnode-issuance`：20/20。
- `legislation-yuan`：enum 判别值与创世 LawVersion 字段序定向测试通过。
- `node_guard`：50/50；`constitution`：39/39。

### 下一步边界

- 本步只完成字段、编码和规则基线，不提前勾选“第 6 步恶意状态与包装器拒绝矩阵”。
- 下一步单独输出第 6.2 步恶意状态与包装器拒绝矩阵方案，确认后再实施。

## 第 6.2 步恶意状态与包装器拒绝矩阵结果（2026-07-12）

### 安全加固

- ConstitutionGuard 的历史版本 key 与两类修宪凭据 key 解析新增 Blake2_128Concat hash 重算；
  畸形 hasher 不再仅凭长度和尾部版本号被识别为规范 key。
- 全节点发行新增错误审计高度、错误矿工和错误最近出块高度拒绝测试。
- 公民发行新增临时标记缺失、CID 反向索引不闭环和 `RewardedCount` 错误拒绝测试。
- 机构生命周期新增机构码、镇码、创建高度不可变以及 Closed 墓碑保持 Closed 时也不得改写的测试。
- 统一委派闸门新增连续两次 `KnownBad`、内层零新增调用及之后合法输入恢复委派的无污染证明。

### 验收

- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：54/54 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：39/39 通过。
- 本步没有修改 runtime、service、chainspec 或其它产品模块，没有新增文件。
- 第 6 步“恶意状态与包装器拒绝矩阵”已完成；完整状态/warp、数据库不入库和三节点恶意链仍未完成。

## 第 6.3 步完整状态与 warp 提交前校验结果（2026-07-12）

### 代码与安全边界

- 从 `NodeGuard::verify_imported_state` 提取 `verify_imported_policy_state` 纯校验核心；生产与测试复用
  同一遍状态分区和四策略判定，不形成影子校验路径。
- 新增 `ImportedPolicyStats`，只记录总扫描及治理/全节点发行/公民发行/CID 分区数，不保存跨块状态。
- 当前 runtime 真实 block#0 全 storage 通过全部 NodeGuard 策略，且扫描计数严格等于输入 key 数。
- 删除固定治理机构、非零 PoW 创世累计、未知 CitizenIssuance key、删除创世封存账户均在对应策略拒绝。
- 非 block#0 完整快照继续由 CID 策略严格返回 `NonGenesisStateImportForbidden`。
- ConstitutionGuard 对版本和凭据 storage 前缀下的畸形 Blake2_128Concat key 新增
  `StorageKeyMalformed`，完整状态不能把畸形相关 key 当成无关 key 忽略。

### 自动化与真实运行

- `node_guard`：57/57；`constitution`：40/40。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path citizenchain/node/Cargo.toml`：通过。
- 当前源码 fresh 节点使用独立 `/tmp` base path 启动，约 52 秒达到 block#0 RPC 可用；创世哈希为
  `0xbdac261dac0c76d68f7d25470d7a1332ea3a7a891f0d5d917c18afea2ec6aea4`，临时数据库约 352 MiB。
- 没有 NodeGuard/ConstitutionGuard 拒绝或 panic；`/tmp/gmb-node-guard-step63.*` 已删除。
- 本步未修改 runtime、service、chainspec 或其他产品模块，没有新增文件。

### 任务状态

- 完整状态/warp 的代码级提交前矩阵与 fresh block#0 真实启动已完成。
- 任务卡中的“warp、三节点分叉与恶意链真实验收”是合并项；三节点尚未完成，因此本步不提前勾选。
