# CITIZENCHAIN 技术开发文档（当前实现基线）

## 1. 文档目的
- 固化 `citizenchain` 当前产品级技术基线，作为开发、联调、测试、运维、打包发布的统一参考。
- 说明 `citizenchain` 在 `GMB` 仓库中的定位，以及与 `SFID`、`CPMS`、`wuminapp` 的边界。
- 建立产品技术文档与模块技术文档之间的映射关系，避免后续只维护模块文档、不维护产品全局口径。

## 2. 文档体系定位

### 2.1 技术文档三层结构
- 仓库技术文档：`/Users/rhett/GMB/GMB_TECHNICAL.md`
- 产品技术文档：`/Users/rhett/GMB/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- 模块技术文档：位于 `citizenchain` 各模块目录下，描述单模块需求与实现细节。

### 2.2 本文范围内
- `node/`：区块链节点原生程序。
- `runtime/`：链上运行时与统一状态机。
- `governance/`：治理类 pallet。
- `issuance/`：发行类 pallet。
- `transaction/`：交易与手续费类 pallet。
- `otherpallet/`：其他链上基础能力 pallet。
- `nodeuitauri/`：旧版 Tauri 桌面节点 UI 与内嵌节点管理。
- `nodeui/`：新版 Flutter Desktop 桌面节点 UI。

### 2.3 本文范围外
- `SFID` 的链外网站、签名服务与数据库内部实现。
- `CPMS` 的离线档案录入与打印系统内部实现。
- `wuminapp` 的移动端 UI、钱包与登录实现细节。
- 仓库级 CI/CD、安装器流水线、工具库、白皮书与宣传性文档。

## 3. 产品定位与边界

### 3.1 产品定位
- `citizenchain` 是 `GMB` 仓库中的主权区块链产品，负责链上状态、共识、治理、发行、交易结算与节点运行。
- 原生链名称为 `CitizenChain`，原生数字货币为 `GMB`。
- 产品同时包含两部分：
  - 区块链节点程序：`node/`
  - 旧版桌面节点软件：`nodeuitauri/`
  - 新版桌面节点软件：`nodeui/`

### 3.2 对外协作边界
- 对 `SFID`：提供绑定、资格校验、人口快照、公民投票凭证等链侧接口承载能力。
- 对 `CPMS`：不直接集成，只通过 `SFID` 间接承接公民身份可信输入。
- 对 `wuminapp`：提供链上账户、交易、治理、节点状态、奖励与网络可观测能力。

## 4. 当前目录结构（迁移期）

```text
citizenchain/
├── node/            # 原生节点程序（CLI、service、RPC、chain spec）
├── runtime/         # 运行时 wasm 与 runtime API
│   ├── governance/  # 目标治理目录（已建骨架）
│   ├── issuance/    # 目标发行目录（已建骨架）
│   ├── transaction/ # 目标交易目录（已建骨架）
│   ├── otherpallet/ # 目标其他模块目录（已建骨架）
│   └── primitives/  # 目标运行时常量目录（已建骨架）
├── governance/      # 旧版治理 pallet 目录（迁移中）
├── issuance/        # 旧版发行 pallet 目录（迁移中）
├── transaction/     # 旧版交易与手续费 pallet 目录（迁移中）
├── otherpallet/     # 旧版其他基础能力 pallet 目录（迁移中）
├── nodeuitauri/     # 旧版 Tauri 桌面节点 UI
├── nodeui/          # 新版 Flutter Desktop 节点 UI
└── scripts/         # 本产品脚本
```

## 5. 系统总体架构

### 5.1 分层结构
- Native Node 层：负责 CLI、网络、数据库、共识服务编排、RPC 服务、chain spec 加载。
- Runtime 层：负责所有链上状态转换、交易校验、治理规则、发行规则、手续费规则。
- Pallet 层：按治理、发行、交易、其他能力拆分功能模块。
- Desktop UI 层：负责本地节点进程生命周期管理、参数设置、状态展示与安装包交付；当前处于 Tauri 向 Flutter Desktop 迁移期。

### 5.2 关键共享依赖
- `primitives/`：提供链常量、机构常量、SS58 参数、发行与人口基础常量。
- `polkadot-sdk`：提供 Substrate / FRAME / client / consensus 依赖。

## 6. 节点程序（`node/`）

### 6.1 职责
- 提供 `BuildSpec`、`ExportBlocks`、`ImportBlocks`、`PurgeChain`、`Benchmark` 等标准节点能力。
- 加载 `CitizenChain` 主网 chain spec。
- 编排 PoW 出块、GRANDPA 最终性、交易池、RPC 服务与数据库。

### 6.2 当前 chain spec 口径
- `node/src/command.rs` 当前仅接受：
  - `mainnet`
  - 省略 `--chain`
  - 自定义 chain spec JSON 文件路径
- `dev` 与 `local` 已被显式禁用，不允许作为默认开发链口令。

### 6.3 当前运行形态
- 数据库存储：RocksDB
- 网络层：`libp2p` / `litep2p`
- 默认本地 RPC：`127.0.0.1:9944`
- 默认本地 Prometheus：`127.0.0.1:9615`

## 7. Runtime（`runtime/`）

### 7.1 定位
- `citizenchain` 是统一链上状态机。
- 账户体系、交易扩展、链上 pallet 装配、runtime API、创世配置都由这里统一编译到 wasm。

### 7.2 当前实现特征
- `AccountId` 与公钥等价，链上账户体系直接以公钥签名身份为主。
- 交易扩展中显式拒绝 `keyless` 账户作为发送方。
- runtime 当前直接依赖本产品的治理、发行、交易、其他 pallet。
- 创世配置由 `runtime/src/genesis_config_presets.rs` 提供。

### 7.3 Runtime 升级边界
- 改动 `runtime/` 内部逻辑，通常属于 runtime 变更。
- 改动被 runtime 直接依赖的 pallet，也属于 runtime 变更。
- 改动 genesis patch / chain spec，不一定是“现有链 runtime 升级”，很多情况下更接近“新链配置”或“重发 chain spec”。

## 8. 共识与链运行模型

### 8.1 出块
- 当前新区块生产采用 PoW。
- 节点使用独立 `powr` key type 生成 / 管理本地 PoW 作者身份。
- 首次启动若不存在 `powr` 密钥，节点会自动生成。

### 8.2 最终性
- 最终性使用 GRANDPA。
- GRANDPA 最终性密钥治理能力由治理模块承接，而不是硬编码在 UI 或脚本层。
- 最终性是否推进取决于 GRANDPA authority 是否按当前链配置正确上线并参与投票。

### 8.3 链身份
- 地址显示格式使用自定义 `SS58 = 2027`。
- 链名、链 ID、Token 显示属性统一来自 `primitives` 与 chain spec 配置。

## 9. 链上模块分组

### 9.1 治理模块（`governance/`）
- 负责内部投票、联合投票、公民投票、最终性密钥治理、管理员权限治理、运行时升级治理、决议发行治理、销毁治理。

当前模块：
- `admins-origin-gov`
- `grandpa-key-gov`
- `resolution-destro-gov`
- `resolution-issuance-gov`
- `runtime-root-upgrade`
- `voting-engine-system`

### 9.2 发行模块（`issuance/`）
- 负责轻节点认证发行、全节点 PoW 奖励、省储行质押利息、决议发行执行。

当前模块：
- `citizen-lightnode-issuance`
- `fullnode-pow-reward`
- `resolution-issuance-iss`
- `shengbank-stake-interest`

### 9.3 交易模块（`transaction/`）
- 负责链上交易手续费、链下交易手续费、机构多签交易能力。

当前模块：
- `duoqian-transaction-pow`
- `offchain-transaction-pos`
- `onchain-transaction-pow`

### 9.4 其他模块（`otherpallet/`）
- 负责 SFID 链上绑定 / 资格校验、PoW 难度调整等基础能力。

当前模块：
- `pow-difficulty-module`
- `sfid-code-auth`

## 10. 桌面节点软件（`nodeuitauri/` 与 `nodeui/`）

### 10.1 定位
- `nodeuitauri` 是当前可运行的旧版桌面节点产品壳，采用 Tauri。
- `nodeui` 是新版 Flutter Desktop 节点 UI，作为未来正式实现建设。
- 对最终用户仍然提供“安装即用”的节点软件，而不是要求用户手工管理原生 node 命令。

### 10.2 当前职责
- `nodeuitauri` 当前负责启动 / 停止内嵌节点进程。
- `nodeuitauri` 当前负责管理 bootnode 地址、奖励地址、GRANDPA 地址、节点名称等本地设置。
- `nodeuitauri` 当前负责展示节点状态、链状态、网络概览、挖矿面板与其他辅助信息。
- `nodeui` 当前负责承接 Flutter Desktop 版的新架构入口和迁移目标。

### 10.3 打包边界
- `nodeuitauri` 当前通过 sidecar 方式内嵌节点二进制。
- `nodeui` 完成迁移前，桌面交付仍以 `nodeuitauri` 为准。
- 对用户交付形态始终保持单个桌面安装包；对工程实现来说仍是“UI 壳 + 内嵌 node 二进制”。

## 11. 变更与发布边界

### 11.1 需要 runtime 升级的改动
- `runtime/` 中的状态机、类型、交易校验、runtime API。
- `governance/`、`issuance/`、`transaction/`、`otherpallet/` 中被 runtime 直接引用的链上逻辑。
- `primitives/` 中被 runtime 直接使用、并影响链上行为的常量 / 类型 /编码结构。

### 11.2 不需要 runtime 升级的改动
- `node/` 中的 CLI、RPC、服务编排、网络与本地运行逻辑。
- `nodeuitauri/` 的桌面 UI、设置页、Tauri 命令与安装包逻辑。
- `nodeui/` 的 Flutter Desktop UI、页面与桌面工程逻辑。
- 构建脚本、CI/CD、前端界面、说明文档。

### 11.3 特殊情况
- `node/src/chain_spec.rs` 变更通常不是“现有链 runtime 升级”，而是 chain spec / bootnodes / properties / 启动配置变更。
- `runtime/src/genesis_config_presets.rs` 变更若影响创世状态，通常对应新链或重建链，不等于自动给已运行链打补丁。

## 12. 产品级模块文档索引

### 12.1 治理
- `governance/admins-origin-gov/ADMINSORIGIN_TECHNICAL.md`
- `governance/grandpa-key-gov/GRANDPAKEYGOV_TECHNICAL.md`
- `governance/resolution-destro-gov/RESOLUTIONDESTRO_TECHNICAL.md`
- `governance/resolution-issuance-gov/RESOLUTIONISSUANCEGOV_TECHNICAL.md`
- `governance/runtime-root-upgrade/RUNTIMEROOT_TECHNICAL.md`
- `governance/voting-engine-system/VOTINGENGINE_TECHNICAL.md`

### 12.2 发行
- `issuance/citizen-lightnode-issuance/CITIZENISS_TECHNICAL.md`
- `issuance/fullnode-pow-reward/FULLNODE_TECHNICAL.md`
- `issuance/resolution-issuance-iss/RESOLUTIONISS_TECHNICAL.md`
- `issuance/shengbank-stake-interest/SHENGBANK_TECHNICAL.md`

### 12.3 交易
- `transaction/duoqian-transaction-pow/DUOQIAN_TECHNICAL.md`
- `transaction/offchain-transaction-pos/OFFCHAIN_TECHNICAL.md`
- `transaction/onchain-transaction-pow/ONCHAIN_TECHNICAL.md`

### 12.4 其他链上模块
- `otherpallet/pow-difficulty-module/POW_DIFFICULTY_TECHNICAL.md`
- `otherpallet/sfid-code-auth/SFIDCODEAUTH_TECHNICAL.md`

### 12.5 桌面节点 UI
- `nodeuitauri/backend/src/home/HOME_TECHNICAL.md`
- `nodeuitauri/backend/src/mining/mining-dashboard/MINING_DASHBOARD_TECHNICAL.md`
- `nodeuitauri/backend/src/network/network-overview/NETWORK_OVERVIEW_TECHNICAL.md`
- `nodeuitauri/backend/src/other/other-tabs/OTHER_TABS_TECHNICAL.md`
- `nodeuitauri/backend/src/settings/bootnodes-address/BOOTNODES_ADDRESS_TECHNICAL.md`
- `nodeuitauri/backend/src/settings/device-password/DEVICE_PASSWORD_TECHNICAL.md`
- `nodeuitauri/backend/src/settings/fee-address/FEE_ADDRESS_TECHNICAL.md`
- `nodeuitauri/backend/src/settings/grandpa-address/GRANDPA_ADDRESS_TECHNICAL.md`

## 13. 维护要求
- `citizenchain` 发生架构级、边界级、发布级改动时，必须同步更新本文档。
- 模块行为变更时，必须同时更新对应模块技术文档。
- 若产品级口径与模块级口径冲突，以代码实现为准，并应在本次改动中同时修正文档。
