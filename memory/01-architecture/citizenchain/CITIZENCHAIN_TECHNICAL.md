# CITIZENCHAIN 技术开发文档（当前实现基线）

## 1. 文档目的
- 固化 `citizenchain` 当前产品级技术基线，作为开发、联调、测试、运维、打包发布的统一参考。
- 说明 `citizenchain` 在 `GMB` 仓库中的定位，以及与 `CID`、`citizenapp` 的边界。
- 建立产品技术文档与模块技术文档之间的映射关系，避免后续只维护模块文档、不维护产品全局口径。

## 2. 文档体系定位

### 2.1 技术文档三层结构
- 仓库技术文档：`/Users/rhett/GMB/memory/01-architecture/gmb/GMB_TECHNICAL.md`
- 产品技术文档：`/Users/rhett/GMB/memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- 模块技术文档：位于 `memory/05-modules/citizenchain/`，描述单模块需求与实现细节。

### 2.2 本文范围内
- `node/`：区块链节点原生程序、桌面节点 UI、内嵌节点管理与打包入口。
- `runtime/`：链上运行时与统一状态机。
- `runtime/governance/`：治理类 pallet。
- `runtime/admins/`：管理员类 pallet。
- `runtime/private/`：私权类 pallet。
- `runtime/issuance/`：发行类 pallet。
- `runtime/transaction/`：交易与手续费类 pallet。
- `runtime/otherpallet/`：其他链上基础能力 pallet。
- `runtime/primitives/`：运行时共享常量、基础类型与制度数据。

### 2.3 本文范围外
- `CID` 的链外网站、签名服务与数据库内部实现。
- `citizenapp` 的移动端 UI、钱包与登录实现细节。
- 仓库级 CI/CD、安装器流水线、工具库、白皮书与宣传性文档。

## 3. 产品定位与边界

### 3.1 产品定位
- `citizenchain` 是 `GMB` 仓库中的主权区块链产品，负责链上状态、共识、治理、发行、交易结算与节点运行。
- 原生链名称为 `CitizenChain`，原生数字货币为 `GMB`。
- 产品同时包含两部分：
  - 区块链节点程序：`node/src/service.rs`、`node/src/command.rs` 等原生节点模块
  - 桌面节点软件：`node/src/desktop.rs`、`node/src/<功能名>` 与 `node/frontend`

### 3.2 对外协作边界
- 对 `CID`：提供绑定、资格校验、人口快照、公民投票凭证等链侧接口承载能力。
- 对 `citizenapp`：提供链上账户、交易、治理、节点状态、奖励与网络可观测能力。

## 4. 当前目录结构

```text
citizenchain/
├── node/            # 原生节点、桌面端 Rust 后端、React 前端与 Tauri 打包入口
├── onchina/         # 链上中国平台:注册局身份、行政区、机构登记、管理后台和链侧凭证
├── runtime/         # 运行时 wasm 与 runtime API
│   ├── governance/  # 治理 pallet 与治理文档
│   ├── admins/      # 管理员 pallet 与管理员文档
│   ├── private/     # 私权 pallet 与私权文档
│   ├── issuance/    # 发行 pallet 与发行文档
│   ├── transaction/ # 交易 pallet 与手续费文档
│   ├── otherpallet/ # 其他链上基础能力 pallet
│   └── primitives/  # 运行时共享常量、基础类型与制度数据
└── scripts/         # 本产品脚本
```

### 4.1 OnChina 注册局子系统

`citizenchain/onchina` 是公民链 workspace 成员 crate，承接链上中国平台、行政区、机构登记、管理后台和链侧凭证能力。任意机构可在办公室服务器安装节点后手动启动 OnChina；首次管理员冷钱包登录后由链上 active admin 关系确定并绑定本节点机构。

- 进程模型：OnChina 是公民链内置二进制能力，由节点桌面端设置页“链上中国平台”入口手动拉起为子进程、退出时一并停掉；节点启动后默认不启动 OnChina，避免只挖矿节点承担管理后台服务。OnChina 经节点 RPC 读写链，对内网托管 HTTPS API 与前端，固定入口为 `https://onchina.local:8964`。桌面 = 节点运维台，浏览器 = 机构管理员，并存不冲突。
- 数据两层：链上最小身份 + 承诺哈希(选择性/绑定触发上链)；链下明细存本市内嵌 PostgreSQL + 本地/NAS 文件仓库(文件哈希上链验真)。
- 当前进度：
  - Step0：crate 骨架 + node 拉起子进程的最小贯通（已完成）。
  - Step1：`citizenchain/onchina/src` 后端完成迁移和收敛，平台层切换为内嵌 PostgreSQL + 节点 RPC + 进程内本地限流；省/市 scope 与行政区维度保留。
  - Step2：`citizenchain/onchina/frontend` 前端完成迁移和收敛，OnChina 后端同源托管 `dist` + SPA 回退；桌面 `node/frontend` 与浏览器 `onchina/frontend` 两套独立前端并存。
  - 后续：链上管理员供给与扫码登录、公民护照直接录入收口、打包部署均按 OnChina 当前任务卡推进，不再引用旧注册局迁移任务口径。

## 5. 系统总体架构

### 5.1 分层结构
- Native Node 层：负责 CLI、网络、数据库、共识服务编排、RPC 服务、chain spec 加载。
- Runtime 层：负责所有链上状态转换、交易校验、治理规则、发行规则、手续费规则。
- Pallet 层：按治理、发行、交易、其他能力拆分功能模块。
- Desktop UI 层：由 `node/src/desktop.rs`、`node/src/<功能名>` 与 `node/frontend` 负责本地节点进程生命周期管理、参数设置、状态展示与安装包交付。

### 5.2 关键共享依赖
- `runtime/primitives/`：提供链常量、机构常量、SS58 参数、发行与人口基础常量。
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
- 交易扩展中显式拒绝 `stake` 账户作为发送方。
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
- 普通节点清库或首次安装后，必须先从现网导入区块，未接入网络或仍处于主同步阶段时禁止本地先出块，避免节点自发形成离线分叉。

### 8.2 最终性
- 最终性使用 GRANDPA。
- GRANDPA 最终性密钥治理能力由治理模块承接，而不是硬编码在 UI 或脚本层。
- 最终性是否推进取决于 GRANDPA authority 是否按当前链配置正确上线并参与投票。
- 节点刚安装完成时默认是普通同步节点；只有在本地导入 GRANDPA 私钥且该公钥匹配当前 authority set 后，节点才会切换为 GRANDPA 节点。
- 只有本地持有且匹配当前 authority set 的 GRANDPA 私钥节点，才会注册 GRANDPA 网络协议并启动 `grandpa-voter` 参与最终性投票。
- 普通节点不再注册 GRANDPA 网络协议，避免出现“协议已声明但无人消费”而触发 `EssentialTaskClosed` 并打断现网连接的回归。
- GRANDPA 持久化仅保留恢复与 proof 所需的覆盖写状态；按轮次追加的 `concluded_rounds` 已在本地 vendored `sc-consensus-grandpa` 中停用，用于限制多节点长期运行时的 AUX 膨胀。

### 8.3 链身份
- 地址显示格式使用自定义 `SS58 = 2027`。
- 链名、链 ID、Token 显示属性统一来自 `runtime/primitives` 与 chain spec 配置。

## 9. 链上模块分组

### 9.1 治理模块（`runtime/governance/`）
- 负责内部投票、联合投票、公民投票、最终性密钥治理、运行时升级治理、销毁治理，并为决议发行提供联合投票引擎。

当前模块：
- `grandpakey-change`
- `resolution-destro`
- `runtime-upgrade`
- `votingengine`

### 9.2 管理员模块（`runtime/admins/`）
- 负责公权机构管理员、私权机构管理员和个人多签管理员；固定治理机构初始管理员由创世写入，运行期治理归公权管理员模块。

当前模块：
- `admin-primitives`
- `public-admins`
- `private-admins`
- `personal-admins`

### 9.3 实体模块（`runtime/entity/`）
- 负责公权机构、私权机构、个人多签账户的创建、关闭、资金与生命周期治理。
- 机构管理已按公权/私权拆分两 pallet(取代旧 `organization-manage`)。

当前模块：
- `public-manage`（公权机构生命周期,idx32）
- `private-manage`（私权机构生命周期,idx33）
- `personal-manage`（个人多签）

### 9.4 发行模块（`runtime/issuance/`）
- 负责公民发行、全节点发行、省储行利息、决议发行完整流程。

当前模块：
- `citizen-issuance`
- `fullnode-issuance`
- `resolution-issuance`
- `provincialbank-interest`

### 9.5 交易模块（`runtime/transaction/`）
- 负责链上交易手续费、链下交易手续费、机构多签交易能力。

当前模块：
- `multisig-transfer`
- `institution-asset`
- `offchain-transaction`
- `onchain-transaction`

### 9.6 其他模块（`runtime/otherpallet/`）
- 负责 CID 链上绑定 / 资格校验、PoW 难度调整等基础能力。

当前模块：
- `pow-difficulty`
- `cid-system`

## 10. 桌面节点软件（`node/`）

### 10.1 定位
- `citizenchain/node` 是当前唯一桌面节点产品壳与原生节点实现目录。
- 历史 `node` 与独立 `node` 目录中的桌面职责已经收口到 `citizenchain/node`，旧目录已删除。
- 对最终用户仍然提供“安装即用”的节点软件，而不是要求用户手工管理原生 node 命令。

### 10.2 当前职责
- `node/src/desktop.rs` 负责 Tauri 桌面入口与 command 注册。
- `node/src/<功能名>` 负责桌面端 Rust 后端能力，不再保留 `node/src/ui` 目录层。
- `node/frontend/<功能名>` 负责 React 前端页面与交互。
- `citizenchain/node` 负责启动 / 停止内嵌节点进程，管理 bootnode 地址、奖励地址、GRANDPA 地址、节点名称等本地设置，并展示节点状态、链状态、网络概览、挖矿面板与其他辅助信息。
- 设置页的“全节点模式”当前展示归档全节点和普通全节点：默认归档全节点；普通全节点置灰不可选择；在底层剪裁能力完成前，节点实际仍按归档全节点运行。
- 设置页在“全节点模式”和“通信节点功能”之间提供“链上中国平台”手动启动行，显示 `未开启` / `启动中` / `已开启` 状态标签、固定入口 `https://onchina.local:8964` 和“启动 / 关闭”按钮；点击后必须二次确认，只启动或停止 OnChina 子进程，不自动打开浏览器；只有 `/api/v1/health` 真实健康检查通过后才显示 `已开启`。

### 10.3 打包边界
- 桌面端与原生节点在同一个 `node` crate 中构建，Tauri 打包从 `node/frontend/dist` 读取前端产物。
- 对用户交付形态始终保持单个桌面安装包；对工程实现来说仍是“UI 壳 + 内嵌 node 二进制”。

## 11. 变更与发布边界

### 11.1 需要 runtime 升级的改动
- `runtime/` 中的状态机、类型、交易校验、runtime API。
- `runtime/governance/`、`runtime/admins/`、`runtime/private/`、`runtime/issuance/`、`runtime/transaction/`、`runtime/otherpallet/` 中被 runtime 直接引用的链上逻辑。
- `runtime/primitives/` 中被 runtime 直接使用、并影响链上行为的常量 / 类型 /编码结构。

### 11.2 不需要 runtime 升级的改动
- `node/` 中的 CLI、RPC、服务编排、网络与本地运行逻辑。
- `node/` 的桌面 UI、设置页、Tauri 命令与安装包逻辑。
- 构建脚本、CI/CD、前端界面、说明文档。

### 11.3 CI 发布边界
- `citizenchain-wasm.yml` 的 push 自动 CI 只编译当前源码 WASM，不查询链上版本、不读取 SSH Secret、不连接服务器；手动 `Run workflow` 才允许使用 `GMB_SSH_KEY` 查询链上 `spec_version` 并在 CI 工作区临时提升构建版本。
- `citizenchain.yml` 的 push 自动 CI 只做桌面端打包检查和本次 run artifact 上传，不读取 Tauri updater 签名私钥、不发布 GitHub Release、不部署服务器。
- 只有手动 `Run workflow` 才允许使用 `GMB_TOP_KEY / GMB_TOP_PUBKEY` 生成 updater 签名产物、发布 `citizenchain-latest.json`、发布 GitHub Release 和使用 `GMB_SSH_KEY` 滚动部署 Linux 服务器。
- CitizenChain workflow 不得恢复系统专属 SSH secret 或复用移动端签名 secret。

### 11.4 特殊情况
- `node/src/chain_spec.rs` 变更通常不是“现有链 runtime 升级”，而是 chain spec / bootnodes / properties / 启动配置变更。
- `runtime/src/genesis_config_presets.rs` 变更若影响创世状态，通常对应新链或重建链，不等于自动给已运行链打补丁。

## 12. 产品级模块文档索引

### 12.1 治理
- `runtime/governance/grandpakey-change/GRANDPAKEYCHANGE_TECHNICAL.md`
- `runtime/governance/resolution-destro/RESOLUTIONDESTRO_TECHNICAL.md`
- `runtime/governance/runtime-upgrade/RUNTIMEUPGRADE_TECHNICAL.md`
- `runtime/votingengine/VOTINGENGINE_TECHNICAL.md`

### 12.1.1 管理员
- `runtime/admins/ADMINS_TECHNICAL.md`

### 12.1.2 实体（机构/个人生命周期）
- `runtime/entity/public-manage/PUBLIC_MANAGE_TECHNICAL.md`
- `runtime/entity/private-manage/PRIVATE_MANAGE_TECHNICAL.md`
- `runtime/entity/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`

### 12.2 发行
- `runtime/issuance/citizen-issuance/CITIZENISS_TECHNICAL.md`
- `runtime/issuance/fullnode-issuance/FULLNODE_TECHNICAL.md`
- `runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md`
- `runtime/issuance/provincialbank-interest/PROVINCIALBANK_TECHNICAL.md`

### 12.3 交易
- `runtime/transaction/multisig-transfer/MULTISIG_TRANSFER_TECHNICAL.md`
- `runtime/transaction/institution-asset/INSTITUTION_ASSET_TECHNICAL.md`
- `runtime/transaction/offchain-transaction/STEP1_TECHNICAL.md`
- `runtime/transaction/offchain-transaction/STEP2A_RUNTIME.md`
- `runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`

### 12.4 其他链上模块
- `runtime/otherpallet/pow-difficulty/POW_DIFFICULTY_TECHNICAL.md`
- `runtime/otherpallet/cid-system/CID_SYSTEM_TECHNICAL.md`

### 12.5 桌面节点 UI
- `memory/05-modules/citizenchain/node/home/HOME_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/mining/dashboard/MINING_DASHBOARD_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/mining/network_overview/NETWORK_OVERVIEW_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/other/other-tabs/OTHER_TABS_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/bootnodes_address/BOOTNODES_ADDRESS_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/device-password/DEVICE_PASSWORD_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/fee_account/FEE_ACCOUNT_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/grandpa_address/GRANDPA_ADDRESS_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`（第 9 节记录全节点模式设置边界）

## 13. 维护要求
- `citizenchain` 发生架构级、边界级、发布级改动时，必须同步更新本文档。
- 模块行为变更时，必须同时更新对应模块技术文档。
- 若产品级口径与模块级口径冲突，以代码实现为准，并应在本次改动中同时修正文档。
