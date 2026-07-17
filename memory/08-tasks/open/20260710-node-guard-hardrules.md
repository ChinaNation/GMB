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
- [x] 第 6 步方案 A：普通块预计算坏块导入层 harness 与不委派验收
- [x] 第 6 步方案 B：P2P 测试态自洽坏块传播拒绝验收
- [ ] 第 6 步 warp、三节点分叉与恶意链扩展验收
- [ ] 第 6 步性能与部署基线验收
- [ ] 第 6 步文档、注释、残留清理与任务归档
- [x] 省储行固定发行方案、runtime 路径及 NodeGuard 新文件确认
- [x] 删除 Root 跳年/补发并把固定利息迁入 finalize
- [x] 43 家创立质押本金和 100 年固定利息接入 `NodeGuard`
- [x] 省储行固定发行测试、benchmark 与 fresh block#0 真实验收
- [x] 省储行固定发行文档、中文注释与残留清理
- [x] 修复 `sp-state-machine` 误放开发依赖导致的 CitizenChain Tauri 生产打包失败

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

## 第 4 步历史执行结果（2026-07-10，机构永久生命周期规则已于 2026-07-16 被当前方案替代）

### 代码与规则

- 新建 `node_guard::cid_lifecycle`，使用节点原生 RAW key、SCALE 镜像和 block#0 基准，不读取 runtime metadata。
- 公民 `CidRegistry` 写入后不得删除或换注册局、档案承诺、居住省市和登记高度；只允许 `Active → Revoked`，吊销后永久终态。
- 机构统一为占号中、运行中、永久关闭三种业务状态：主账户登记存在而 `Institutions` 尚不存在即占号中，`Active` 即运行中，`Closed` 即永久关闭。
- 机构 CID 不得删除、跨公私权重复、换机构码/创建高度/镇码或关闭后恢复；主账户占号不得在关闭前删除。运行中机构名称允许依法更新，新 CID 允许使用历史名称。
- 固定治理机构只能来自 block#0；机构不保存生命周期状态。协议账户集合由 CID 制度约束永久保护，`InstitutionAccounts` 与 `AccountRegisteredCid` 必须闭环且协议账户不可删除或换地址。
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

- **省储行利息（当时评估，现已解决）**：第 5 步因 Root 可跳过到期年度而暂不接入；后续用户正式确认
  其为不可跳过固定规则，Root 跳年/补发入口已经删除，本金与年度利息现已接入 NodeGuard。
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
- 第 5 步当时因 Root 可跳年而暂缓省储行利息；该冲突现已通过删除 Root 出口并接入 NodeGuard 解决。
  resolution/onchain 发行仍是治理/非治理资产结果；交易费、
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

## 第 6 步阶段执行记录（2026-07-11 起，执行中）

### 已完成的自动化基线

- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard`：76/76 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`：40/40 通过。
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
- 当时 fresh 节点连接旧 bootnode 时，双方明确报告 genesis mismatch：fresh 为 `0x3e3a…ff82`，
  旧冻结网络为 `0xb57c…9971`。该历史部署缺口已由 2026-07-14 单例治理机构任务第 5 步重生唯一冻结基线解决。
- 节点停止后已删除 `/tmp/gmb-nodeguard-perf.*` 临时目录；仓库没有新增验收文件。

### 真实三节点最终验收（2026-07-12）

- 已按确认口径创建并删除 `/tmp/gmb-nodeguard-final-acceptance/`；目录只用于临时 fresh chainspec、
  三节点 base-path、keystore、日志和一次性 Alice 签名器，不进入 Git。
- 使用普通 release WASM 导出 fresh chainspec，清空临时 bootNodes，并仅额外资助标准测试账户 Alice；
  启动 A/B/C 三个本地隔离节点，其中 A `--mining-threads 1`，B/C `--mining-threads 0`。
- 三节点成功互联：A/B/C 均为 `peers=2`、`isSyncing=false`；A 作为无外部 bootnode 的本地引导节点，
  B/C 通过 A 的本地 WSS peer 地址加入。
- 第一笔真实 Alice `System::remark` 交易 hash：
  `0xfdde2768a593917f18984d9c197facecb1454305afe14b6998367f18c6fc1ff1`；
  A/B/C 均同步到 block#1，三端哈希一致：
  `0xe0fccc0790f9761226865a2fa96a5eb9e19eb34169191f49faf3afee4817b3c8`。
- 恶意拒绝矩阵在三节点网络保持运行期间重跑：NodeGuard `76/76`、ConstitutionGuard `40/40`；
  覆盖空块、固定治理骨架、全节点发行、公民发行、省储行固定发行、CID 和手续费制度，
  runtime upgrade audit、完整状态导入和护宪规则的拒绝路径。矩阵证明拒绝返回 `KnownBad` 且不委派内层导入。
- 第二笔真实 Alice `System::remark` 交易 hash：
  `0x89179977bd67be499ee5aa38031c9c8ecc6da851436208e21a2048b0887b571e`；
  拒绝矩阵后 A/B/C 继续同步到 block#2，三端哈希一致：
  `0x961012a973cf9695367037b7f9554df2ef541cda17ed5315a7c72b2600bd2a0a`。
- block#1 与 block#2 均包含 2 条 extrinsics（timestamp + Alice remark）和 2 条 digest logs；
  Alice nonce 从 0 推进到 2，pending extrinsics 清零。
- 本次未在 P2P 网络中手工注入伪造坏块；恶意候选“不委派内层、不入库”的证据来自包装器矩阵测试，
  真实网络部分证明合法链在矩阵后继续推进并保持三节点哈希一致。
- 三节点、临时 chainspec、数据库、keystore、签名器和日志全部删除，确认无临时验收进程残留。

### P2P 恶意候选块注入专项尝试（2026-07-12，方案 A 前置结论）

- 已按确认口径创建并删除 `/tmp/gmb-nodeguard-badblock-injection/`；目录只用于临时 chainspec、
  探测节点 base-path 和导出块文件，不进入 Git。
- 运行节点 RPC 能力探测：当前 RPC 仅提供 `author_submitExtrinsic`、`chain_*`、`state_*`、
  `system_*` 等交易与查询接口，没有 `engine_*`、manual-seal、dev block submit 或任意 block
  注入接口，因此不能通过 RPC/P2P 直接提交伪造块。
- 试跑 CLI 文件导入层：`export-blocks --from 0 --to 0` 可导出合法 block#0 JSON；
  `import-blocks` 可将该合法 block 文件导入新的临时数据库，证明文件导入队列入口可用。
- 不能把“篡改 JSON 导致 header/root/编码错误”当作 NodeGuard 恶意候选验收；那只能证明基础
  block 解码或 state root 校验失败，不能证明永久规则守卫拒绝。
- 真实 P2P 坏块注入需要一个临时恶意块生产器：能构造结构完整、PoW seal 完整、state root 可重算、
  但执行后违反 NodeGuard 永久规则的候选块，并通过网络或导入队列提交给诚实节点。当前生产节点不提供
  这个入口；后续只能在测试/导入层 harness 中补齐，不得为生产 RPC/P2P 暴露任意块提交能力。

### 区块链测试 harness crate（2026-07-12，已创建）

- 按确认在 `citizenchain/crates/blockchain-test-harness/` 新增专用测试工具 crate，并加入
  `citizenchain/Cargo.toml` workspace；该 crate 只用于真实验收、导入路径验证和后续恶意候选块构造，
  不得被生产 node、runtime 或业务模块依赖。
- 第一阶段沉淀已验证过的 Alice `System::remark` signed extrinsic 构造能力，后续三节点真实交易
  验收不再需要在 `/tmp` 反复生成一次性签名器。
- 第二阶段新增 `export-blocks` JSON lines 摘要解析和基础 stateRoot 篡改样本生成；该能力仅用于证明
  `import-blocks` / import queue 基础坏文件拒绝，不代表 NodeGuard 永久规则坏块。
- 使用 `/tmp/gmb-blockchain-test-harness-import/` 执行真实导入队列基线：合法 block#0 文件导入成功；
  篡改 stateRoot 后的 block#0 文件被 `import-blocks` 以退出码 1 拒绝，报错为 unknown parent（篡改
  header 后 genesis hash 改变）。临时目录已删除。
- 新增 `src/bin/harness.rs` 命令行入口，支持生成 Alice remark extrinsic、摘要导出块文件和生成
  stateRoot 篡改文件；后续验收不再需要临时签名器或 Python 篡改脚本。
- 使用 `/tmp/gmb-harness-block1-import/` 执行结构完整 block#1 导入基线：双节点产出合法 block#1，
  用 `export-blocks --from 1 --to 1` 导出后，合法 block#1 可导入新的临时数据库；同一 block#1
  仅篡改 `stateRoot` 后，parent 仍为 genesis、extrinsics=2、digest_logs=2，`import-blocks`
  执行 runtime 后因 `Storage root must match that calculated` 触发只读执行失败，NodeGuard 包装路径
  fail-closed，退出码 1，日志为 `bad block`。临时目录已删除。
- 第三阶段新增完整导入态永久规则坏样本矩阵：harness 提供稳定 case 清单和期望守卫前缀，node 内部测试
  使用真实创世 storage 构造坏状态并验证导入前拒绝。当前覆盖固定治理骨架、全节点发行、公民认证发行、
  创世模块、省储行固定发行和 CID 生命周期，不扩大 NodeGuard 生产接口。
- 第四阶段补齐导入层包装器验收：直接构造 `BlockImportParams::state_action =
  ApplyChanges(Import(...))` 的完整状态导入形态，坏状态返回 `KnownBad` 且 inner import 计数保持 0；
  合法 block#0 完整状态校验成功后 inner import 计数为 1。该验收覆盖真实导入队列/warp 的 `with_state`
  入口，但仍不冒充 P2P 手工伪造块注入。
- 方案 A 补齐普通块预计算坏块导入层验收：NodeGuard 新增 `ApplyChanges(Changes(...))` 一致性校验，
  导入方携带的预计算 state root、主存储变更、子存储变更和 offchain 存储变更必须与本节点 runtime
  只读重放结果一致，不一致即 fail-closed。node 内部 test-only harness 使用真实 `BlockBuilder`
  生成 timestamp + Alice remark 合法 block#1，随后篡改 `GenesisPallet::citizen_max` 预计算 delta，
  基于父状态重算自洽 state root 与 backend transaction；合法 proposal 通过，自洽坏 proposal
  返回 `KnownBad` 且 inner import 计数保持 0。该能力不进入生产节点 RPC/P2P 接口。
- 方案 B 补齐 P2P 测试态坏块传播拒绝验收：新增 `citizenchain/node/src/core/service/p2p_bad_block_tests.rs`
  test-only 服务级 harness，由恶意测试节点使用裸 `PowBlockImport<GrandpaBlockImport>` 将“PoW 合法、
  state root 自洽、但篡改 `GenesisPallet::citizen_max`”的 block#1 写入本地 DB，模拟改节点代码绕过
  `NodeGuard/ConstitutionGuard` 的攻击者；诚实测试节点使用生产同构 guarded import queue 和真实
  `build_network` 通过 P2P reserved peer 连接恶意节点，观察到恶意 peer 的 `best_hash/best_number`
  后仍保持 best=genesis，且本地数据库不存在坏块 header。该测试不新增生产伪造块接口。
- crate 内已用中文注释标明边界：测试 harness 可以构造验收交易和未来坏块材料，但不能成为生产路径。
- 验收：`cargo check -p blockchain-test-harness` 通过；`cargo test -p blockchain-test-harness` 6/6 通过；
  `cargo test -p node node_guard` 81/81 通过；`WASM_BUILD_FROM_SOURCE=1 cargo test -p node
  precomputed_changes_must_match_reexecuted_normal_block -- --nocapture` 通过；`WASM_BUILD_FROM_SOURCE=1
  cargo test -p node self_consistent_bad_precomputed_block_is_known_bad_before_inner_import -- --nocapture` 通过；
  `WASM_BUILD_FROM_SOURCE=1 cargo test -p node p2p_sync_rejects_self_consistent_bad_node_guard_block -- --nocapture`
  1/1 通过，耗时 115.92s。失败重跑残留的 `/tmp/gmb-p2p-bad-block-*` 已清理，成功路径会自动删除
  两节点唯一临时 base path。

### Release 性能与运行态矩阵（2026-07-12，已完成，第 3 步资产烘焙除外）

- 普通 release build：`cargo build --release -p node --bin citizenchain` 通过，耗时 101.38s，编译过程最大
  RSS 5,488,820,224 bytes。
- 带 WASM 的 release build：`WASM_BUILD_FROM_SOURCE=1 cargo build --release -p node --bin citizenchain`
  通过，耗时 46.11s，编译过程最大 RSS 5,686,214,656 bytes；该产物可导出 `citizenchain-fresh`。
- release NodeGuard 矩阵：`cargo test --release -p node node_guard` 78/78 通过；测试运行耗时 2.27s，
  `/usr/bin/time -l` 最大 RSS 3,998,023,680 bytes。
- release ConstitutionGuard 矩阵：`cargo test --release -p node constitution` 40/40 通过；测试运行耗时
  0.65s，最大 RSS 413,581,312 bytes。
- release 身份登记与公民发行路径：`cargo test --release -p citizen-identity` 22/22 通过；`cargo test
  --release -p citizen-issuance` 14/14、身份集成 5/5 通过。
- release 真实普通快路径：使用带 WASM release binary 导出临时 `citizenchain-fresh`，清空临时
  bootNodes，仅资助标准测试账户 Alice；A/B 双节点本地 `/ws` 互联均 `peers=1`，Alice
  `System::remark` 进入 block#1，A/B 最佳哈希一致为
  `0xdcda6a5958434dcffd7e9fa1e8cde583e9cfacc177005d1d66722e3480266be9`，block#1
  extrinsics=2、digest_logs=2，Alice nonce 0→1，pending=0，采样节点 RSS 峰值 A=1,927,568 KB、
  B=2,654,480 KB，守卫拒绝日志 0。临时目录 `/tmp/gmb-release-matrix` 已删除。

### 当前仍未满足的关闭条件

- 尚未完成更强的 warp/多恶意节点/坏链多高度扩展验收；当前已完成方案 A 的导入层预计算坏块
  `KnownBad`/不委派证据、方案 B 的 P2P 测试态自洽坏块传播拒绝证据、`with_state` 拒绝不委派证据、
  包装器 `KnownBad` 矩阵、完整导入态永久规则坏样本矩阵和真实多节点合法链继续推进证据。
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
  同一遍状态分区和五策略判定，不形成影子校验路径。
- 新增 `ImportedPolicyStats`，只记录总扫描及治理/全节点发行/公民发行/省储行固定发行/CID 分区数，不保存跨块状态。
- 当前 runtime 真实 block#0 全 storage 通过全部 NodeGuard 策略，且扫描计数严格等于输入 key 数。
- 删除固定治理机构、非零 PoW 创世累计、未知 CitizenIssuance key、删除创世封存账户均在对应策略拒绝。
- CID 完整状态允许任意高度导入校验；普通机构允许依法删除，block#0 精确机构基准仍禁止删除或替换。
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

## 省储行固定发行守卫结果（2026-07-12）

### 永久规则与实现

- `CHINA_CH` 的 43 组 `main_account/stake_account/stake_amount` 成为节点编译期真源；创世逐户质押本金
  必须精确写入 `stake_account`，完整 `System::Account` 后续永久不变。
- 年度规则固定为 87,600 块、首年 100 BP、逐年递减 1 BP、连续 100 年；利息只进入对应
  `main_account`，第 101 年起不再发行。
- runtime 删除 `force_settle_years`、`force_advance_year`、Root 跳年、批量补发和失败年度跳过；
  不保留旧 Call 或兼容分支。
- 固定利息迁到 `on_finalize`，新增 `TotalProvincialBankInterestIssued` 和
  `LastProvincialBankInterestAudit(year,bank_count,total_interest)`；43 笔发行与审计在同一存储事务原子提交。
- NodeGuard 新增 `provincialbank_interest` 策略，逐块核对年度审计并把 43 笔利息加入共享
  `FinalizeIssuancePlan`；决议发行和链上发行继续留在 extrinsic 阶段，不被误冻结。
- `:code` 变化强制复核全部质押本金和当前年度审计；block#0 完整状态分区同步纳入省储行策略，
  未知省储行 storage key、缺失本金、错误金额、跳年、提前或重复发行均 fail-closed。

### 自动化、benchmark 与真实运行

- `provincialbank-interest`：10/10；带 `runtime-benchmarks`：11/11。
- runtime 创世测试逐户验证 43 个质押地址与本金；primitives 测试钉死本金人口基数、整 BP 可除性、
  `stake_account` 唯一且不与任何 `main_account` 重合。
- NodeGuard：64/64；省储行定向策略：8/8；ConstitutionGuard：40/40；node `cargo check` 和 production WASM build 通过。
- 正式 pallet benchmark 以 50 steps / 20 repeats 重生权重：45 reads / 46 writes，时间模型约 569 ms，
  proof size 估算 112,919 bytes。
- 当前源码 fresh headless 节点在独立 `/tmp` base path 启动到 block#0，创世哈希
  `0x6fc42816b55ce22f204d0dbddbf38a9ab4d3a1c78005b90e1fcbe376ef8585b1`，数据库约 352 MiB；
  无 NodeGuard/ConstitutionGuard 拒绝或 panic，全部 `/tmp/gmb-node-guard-provincial.*` 已删除。

## 固定平均六分钟与 GenesisPallet 守卫（2026-07-12，已完成）

### 已完成代码

- PoW 时间语义统一为 `POW_TARGET_BLOCK_TIME_MS=360_000`：它只表示难度调整的长期平均目标，
  有效 PoW 找到后立即提交，没有最短等待或最晚出块期限。
- CPU/GPU 删除 `LAST_SUBMIT_NS`、目标时间参数、Runtime API 读取和全部 sleep 提交门控；
  `pallet_timestamp::MinimumPeriod` 固定为 1ms，只要求时间戳严格递增。
- 删除旧 `MILLISECS_PER_BLOCK`、`SLOT_DURATION`、runtime `MINUTES/HOURS/DAYS`，避免六分钟整数换算
  产生 `MINUTES=0`；制度日历继续使用明确的 `BLOCKS_PER_*` 固定区块数。
- `pow-difficulty` 不再依赖 GenesisPallet，固定使用 600 块 × 360,000ms = 60 小时目标窗口；
  旧目标时间 runtime API、trait、Cargo 依赖和 storage proof 全部删除。
- GenesisPallet 删除 `TargetBlockTimeMs`、动态时间 API/trait 和未使用事件，只保留三个创世事实、
  `Phase` 与 `DeveloperUpgradeEnabled` 五个字段。
- 新增 node `genesis_pallet` 策略：三个事实逐字冻结；阶段状态只允许含 `:code` 的
  `(Genesis,true) → (Operation,false)` 原子单向转换；旧时间 key、未知同前缀 key、畸形 SCALE、
  显式写回默认值、部分/反向/二次转换全部 fail-closed。
- GenesisPallet 已接入 NodeGuard 启动锚定、普通区块、runtime 升级全检和完整状态共享单遍分区。
- 空块最初被错误地从 `pow-difficulty` runtime 移到 NodeGuard；复核后已恢复三层规则：
  本地交易池门控避免构造、NodeGuard 在 runtime 前返回 `KnownBad`、runtime 最终共识断言独立拒绝。
- 清算行费率变更“7 天”按六分钟平均制度日历修正为 `1_680` 块；OnChina 上链等待改为
  20 分钟客户端确认观察窗口，窗口结束不再解释为 PoW 超过最晚时间或交易必然失效。

### 已完成自动化与真实运行

- runtime 时间口径测试通过；`pow-difficulty` 带 `runtime-benchmarks` 12/12；
  `genesis-pallet` 7/7；NodeGuard 71/71；ConstitutionGuard 40/40；OnChina chain_submit 2/2。
- PoW 正式 benchmark 50 steps / 20 repeats 完成：调整路径从 4 reads / 2 writes 降为
  3 reads / 2 writes，实测模型 7µs，旧 GenesisPallet proof 已清除。
- node GPU feature、pow/genesis try-runtime、production WASM build 均通过。
- `citizenchain-fresh` 独立 `/tmp` 数据库真实启动成功，GenesisPallet NodeGuard 未拒绝；创世哈希
  `0x6d1ae7386793e966fe2f17f73446f433b3a1aecfd4dd4b9bce2764ca44d98e84`，数据库约 352 MiB。
- 默认冻结 chainspec 仍因旧管理员 SCALE 被治理骨架正确拒绝，不增加兼容；全部启动临时目录已删除。
- 临时双节点使用同一当前源码 chainspec、Alice `powr` 测试密钥和真实签名交易：两端在线后，
  block#1 从提交到可见约 1.988 秒，block#2 约 1.897 秒；两端 block#2 哈希一致为
  `0x993d572e4d18bdea30441c5212df76699db16b0c1bacedc3c47db0bcf9814102`。
- block#2 前的竞态空 proposal 被 NodeGuard 记录为“空块不允许上链”，随后合法 block#2 正常传播。
  该结果只证明节点提前闸门有效，不能替代 runtime 最终拒绝；runtime 断言现已恢复。临时 chainspec、
  测试 keystore 和两份数据库均已删除。

## runtime 空块最终拒绝恢复（2026-07-12，已完成）

- `pow-difficulty::on_finalize` 在读取时间戳并确认非创世块后、任何难度窗口写入前检查
  `System::extrinsic_count()`；数量不大于 1 时以共识断言拒绝，错误空块不能推进
  `WindowStartMs` 或 `CurrentDifficulty`。
- NodeGuard 的 body 预检查继续保留，外部空块在 runtime 执行前返回 `KnownBad`；节点
  `should_propose` 与 CPU/GPU ready 交易池门控继续阻止诚实节点主动构造空 proposal。
- runtime 单元测试验证只有 timestamp 的区块独立失败且状态不变，timestamp 加交易正常完成；
  `pow-difficulty` 10/10、runtime-benchmarks 13/13、try-runtime 10/10、NodeGuard 71/71、
  ConstitutionGuard 40/40。
- 正式 benchmark 50 steps / 20 repeats 已重生：调整 3 reads / 2 writes、7µs；建窗
  2 reads / 1 write、4µs；普通路径 2 reads、5µs。
- 当前源码双节点真实运行：无交易且两端联网时持续停在 block#0；Alice 提交真实绑定交易后产出
  含 timestamp 和交易两笔 extrinsic 的 block#1，两端哈希一致为
  `0x1c0667425cf339697b7d803e9fcab15ddfc22c8aa2e44f903e870946c3991a51`。
- 首轮真实运行暴露最佳块切换与交易池维护竞态：runtime 正确拒绝了本地空 block#2 proposal，空块
  未入库。节点随后增加新链头稳定门控，并让 `mining_threads=0` 且无 GPU 的节点完全不 proposal。
- 修正后 Alice 提交真实重绑定交易，block#2 精确包含两笔 extrinsic；两端哈希一致为
  `0x040bddac4957705eb86b5f3637078ac90a34f95d9cf379398b35a06de87cb86f`。交易池清空后持续停在
  block#2，不再构造空 proposal，也没有 runtime panic；临时 chainspec、数据库、测试 keystore
  和日志验收后全部删除。

## 历史记录：PoW 动态难度 NodeGuard 策略（2026-07-12 实施，2026-07-16 已删除）

### 已完成代码

- `pow-difficulty` 新增版本化 `PowDifficultyParams`、`ActiveParams`、`PendingParams` 和
  `DifficultyAdjustmentAudit`；`CurrentDifficulty` 仍只能由算法推进，治理不能直接设置难度。
- `target_block_time_ms`、`adjustment_interval`、`max_adjust_up_factor`、
  `max_adjust_down_divisor` 允许随 runtime 升级原子变更；参数暂存后下一块激活，激活时保留当前难度并重置窗口。
- `stage_params` 与 `try_state/on_finalize` 均校验 `algorithm_version`，当前节点/runtime 只接受
  `POW_ALGORITHM_VERSION`，未来算法升级必须先让 runtime 与节点守卫共同支持新版本。
- `runtime-upgrade` 的提案和开发者直升路径均携带新 PoW 参数，并在执行成功时写入
  `LastRuntimeUpgradeAudit`；NodeGuard 用审计把 `:code`、旧/新参数 hash 和激活高度绑定。
- 节点挖矿改为直接读取 `PowDifficulty::CurrentDifficulty` RAW storage，不再保留
  `PowDifficultyApi` Runtime API 或固定难度兜底。
- 当时新增的 node `pow_difficulty` 守卫策略现已完整删除；PoW runtime、节点挖矿和链上难度读取保持原状，
  NodeGuard 不再复算或冻结 PoW 难度参数。
- runtime 空块最终拒绝继续保留；NodeGuard 预执行拒绝和本地交易池门控只作为前置防线。

### benchmark 与清理

- `pow-difficulty` 正式 benchmark 50 steps / 20 repeats 重生权重，新增
  `on_initialize_activate_params`；调整路径为 6 reads / 4 writes，参数激活路径为 2 reads / 4 writes。
- `runtime-upgrade` 正式 benchmark 50 steps / 20 repeats 通过真实 extrinsic 路径，权重为
  270 reads / 364 writes，时间模型约 9.023 ms。
- benchmark 环境暴露默认状态仍有旧 `AdminAccount` 编码；已在 `runtime-benchmarks` feature 下按当前结构种下
  NRC、43 个 PRC、43 个 PRB 管理员表，仅用于 benchmark，不增加生产兼容。
- 已清理旧 `PowDifficultyApi` 文档/注释口径，更新 `pow-difficulty`、`runtime-upgrade`、Node、NodeGuard、
  primitives 技术文档。

### 已完成自动化

- `pow-difficulty` runtime-benchmarks：17/17。
- `runtime-upgrade` runtime-benchmarks：18/18。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path Cargo.toml --release --features runtime-benchmarks --bin citizenchain`：通过。
- `pow_difficulty` 和 `runtime_upgrade` 两组正式 pallet benchmark 均已通过并覆盖现有 `weights.rs`。
- node `cargo check`：通过。
- NodeGuard：76/76；ConstitutionGuard：40/40。
- node frontend `npm run build`：通过；Vite 仅提示现有大 chunk 警告。
- CitizenApp `runtime_upgrade_service_test.dart`：3/3。

### 真实运行态验收

- 已按确认口径创建并删除 `/tmp/gmb-pow-difficulty-acceptance/`，目录只用于临时 chainspec、base-path、
  keystore、日志和一次性 Alice 签名器，不进入 Git。
- 当前普通 release WASM 重新导出 fresh chainspec 后，额外资助标准测试账户 Alice；
  单节点启动后 20 秒持续停在 block#0，验证无交易时不产空块。
- Alice 提交真实 `System::remark` signed extrinsic 后，交易池接受 hash
  `0x2aefe7b403a973a43daee7fbff501a5076acc2842e4c2c084743dfa6e49ec92b`；
  因节点挖矿门控要求非离线网络，第二个本地陪跑节点连入后，节点 1 立即产出 block#1。
- block#1 哈希
  `0xaaf286249a775bcac3bb107b7e7f4c15ccb3fb2eaebb8d0cf87e81464d7ae7fb`，
  区块包含 2 条 extrinsics（timestamp + Alice remark）和 2 条 digest logs；Alice nonce 从 0 变为 1，
  pending extrinsics 清零，节点 2 同步到 block#1。
- 当时验收期间无 NodeGuard / ConstitutionGuard 拒绝合法 block#1；临时节点、chainspec、
  签名器、数据库和日志已全部删除。

## CID 与手续费制度守卫收口（2026-07-16）

### 执行边界

- 本次只修改 `citizenchain/node/`、节点测试和文档，`citizenchain/runtime/` 零修改；没有版本迁移、
  旧数据兼容、Runtime API、manifest 或 WASM hash 白名单。
- 删除 NodeGuard 的 `pow_difficulty.rs` 及注册、完整状态分区和启动检查残留；PoW 共识本身不在本次范围。
- CID 节点规则只校验省市码、机构码、盈利属性和校验码。随机段与年份段只作为校验码原始载荷，
  不校验长度、字符、随机派生值或具体年份。
- 只有 block#0 精确机构禁止删除、跨命名空间复制或替换身份；普通机构继续由 runtime 依法创建、
  修改、关闭和删除，NodeGuard 只要求删除时账户与正反索引同步清空。
- 手续费固定口径为：链上费率 0.1%、链上最低 10 分、投票固定 100 分、链下最低 1 分、
  清算行及全局最高费率 0.1%。
- 链下费率只冻结最高值，不增加最低费率规则；费率为 0% 时每笔仍执行最低 1 分手续费。

### 拒块行为

- 普通区块核对链上转账/投票 `FeePaid`、链下 `PaymentSettled` 和清算行费率 storage。
- `:code` 变化时直接执行候选 WASM 的 50,000 分转账、1 分转账和一次投票，按付款账户余额差额
  判定行为，不要求候选 runtime 先随节点发布。
- 任一规则非法统一返回 `KnownBad` 且不委派内层导入器；已运行节点继续使用此前合法 runtime，
  P2P、RPC 和挖矿服务不因非法升级区块退出。

### 自动化

- `cargo check -p node --bin citizenchain` 通过。
- `cargo fmt -p node -- --check` 通过。
- `cargo test -p node core::node_guard --bin citizenchain -- --nocapture`：75/75 通过。
- `WASM_BUILD_FROM_SOURCE=1 cargo test -p node current_wasm_passes_candidate_runtime_fee_behavior_probes --bin citizenchain -- --nocapture`：1/1 通过；真实候选 WASM 完成链上费率、最低费和投票费隔离行为验证。
- CID 定向测试：普通机构删除通过、创世机构删除拒绝、四项 CID 规则通过。
- 手续费定向测试：0.1%、链上最低 10 分、投票 100 分、链下最低 1 分、最高 0.1% 及实际事件核对通过；当前链下入口被 `BaseCallFilter` 禁用时只接受精确 `CallFiltered`，以后开放入口必须通过真实最低费清算探针。
