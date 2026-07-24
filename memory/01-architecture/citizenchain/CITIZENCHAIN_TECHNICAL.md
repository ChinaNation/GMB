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
- `runtime/misc/`：其他链上基础能力 pallet。
- `runtime/primitives/`：运行时共享常量、基础类型与制度数据。

### 2.3 本文范围外
- `CID` 的链外网站、签名服务与数据库内部实现。
- `citizenapp` 的移动端 UI、钱包与登录实现细节。
- 仓库级 CI/CD、安装器流水线、工具库、白皮书与宣传性文档。

## 3. 产品定位与边界

### 3.1 产品定位
- `citizenchain` 是 `GMB` 仓库中的主权区块链产品，负责链上状态、共识、治理、发行、交易结算与节点运行。
- 原生链名称为 `CitizenChain`，原生数字货币为 `GMB`。
- 产品作为一个安装包交付，包含三类核心能力：
  - 区块链节点程序：`node/src/service.rs`、`node/src/command.rs` 等原生节点模块。
  - 链上状态机：`runtime/` 编译出的 wasm 与所有 pallet。
  - 链上中国平台：`onchina/` 多机构工作台，由节点桌面端按需拉起。
- 桌面节点软件由 `node/src/desktop.rs`、`node/src/<功能名>` 与 `node/frontend` 提供本地节点运维、设置和打包入口。

### 3.2 对外协作边界
- 对 `CID`：提供绑定、资格校验、人口快照、投票凭证等链侧接口承载能力。
- 对 `citizenapp`：提供链上账户、交易、治理、节点状态、奖励与网络可观测能力；CitizenApp 默认通过内置 smoldot 轻节点连接 P2P 网络并验证 finalized 链状态，不把公网 HTTP API 当作链上真源。
- 对 Cloudflare 边缘层：首期由国储会权威引导节点通过本机 RPC + 受控 Tunnel 提供链事件投影和已签名交易广播能力；Cloudflare 不运行 Substrate 节点，不保存用户私钥，也不获得公网 RPC。

### 3.3 账户标识目标契约

- runtime 账户类型统一为 `AccountId`；单一账户字段统一为 `account_id`，具有业务角色的第二个及后续账户使用 `<role>_account_id`。
- `AccountId` 是链上账户身份和权限比较值；签名公钥使用 `public_key` / `signer_public_key`；SS58 仅以 `ss58_address` 作为派生展示值。
- 机构岗位授权必须同时验证 `cid_number + role_code + account_id`；公民身份必须同时验证 `cid_number + account_id`。命名统一不改变管理员名册、岗位、CID 或投票引擎职责边界。
- 跨 RPC/JSON 的 32 字节账户和公钥统一编码为小写 `0x` 加 64 位十六进制；runtime 内部继续使用强类型和原始 32 字节值。
- Node 与桌面前端的账户输入边界严格执行 `^0x[0-9a-f]{64}$`，不接受无前缀、大写或混合大小写文本；签名路径先校验 `signer_public_key`，再派生并比较 `signer_account_id`。
- Node 本地缓存、桌面命令和私有 RPC 只保存、传递 `account_id`；SS58 输入在边界解析为账户 ID，输出时可另行派生 `ss58_address`，不得把 SS58 当作授权或缓存主键。
- 挖矿奖励账户私有 RPC 固定为 `reward_bindAccount` / `reward_rebindAccount`，本地非密钥配置固定为 `reward-account.json`。旧 RPC、旧 JSON 和兼容读取均已删除。
- 完整目标与无兼容实施顺序见 ADR-040 和任务卡 `20260722-account-id-official-unify.md`。当前旧字段只描述实施前代码，不得用于新增实现。

## 4. 当前目录结构

```text
citizenchain/
├── node/            # 原生节点、桌面端 Rust 后端、React 前端与 Tauri 打包入口
├── onchina/         # 链上中国平台:多机构工作台、注册局业务、行政区、机构登记、管理后台和链侧凭证
├── runtime/         # 运行时 wasm 与 runtime API
│   ├── governance/  # 治理 pallet 与治理文档
│   ├── admins/      # 管理员 pallet 与管理员文档
│   ├── private/     # 私权 pallet 与私权文档
│   ├── issuance/    # 发行 pallet 与发行文档
│   ├── transaction/ # 交易 pallet 与手续费文档
│   ├── misc/ # 其他链上基础能力 pallet
│   └── primitives/  # 运行时共享常量、基础类型与制度数据
└── scripts/         # 本产品脚本
```

### 4.1 OnChina 多机构工作台

`citizenchain/onchina` 是公民链 workspace 成员 crate，承接链上中国平台、多机构工作台、注册局业务、行政区、机构登记、管理后台和链侧凭证能力。任意机构可在办公室服务器安装节点后手动启动 OnChina；首次管理员冷钱包登录后可由链上 admins 关系确定人员所属机构候选，但具体业务能力必须继续按有效 `RoleSubject` 解析。

- 进程模型：OnChina 是公民链内置二进制能力，由节点桌面端设置页“链上中国平台”入口手动拉起为子进程、退出时一并停掉；节点启动后默认不启动 OnChina，避免只挖矿节点承担管理后台服务。OnChina 经节点 RPC 读写链，对内网托管 HTTPS API 与前端，固定入口为 `https://onchina.local:8964`。桌面 = 节点运维台，浏览器 = 机构管理员，并存不冲突。
- 工作台模型：登录账户属于哪个链上机构 admins 人员集合，就显示哪个机构候选；同一账户属于多个机构时先选择机构。工作台操作列表和每次链写都必须按该账户的有效岗位任职与 `RoleBusinessPermission` 过滤，不能因登录成功获得机构业务权限。注册局是 `workspace` 的一类，司法院、立法院、学校、公司、公益组织等机构按自己的工作台 UI 进入“操作 / 显示 / 记录”页面。
- 数据两层：链上最小身份 + 承诺哈希(选择性/绑定触发上链)；链下明细存本市内嵌 PostgreSQL + 本地/NAS 文件仓库(文件哈希上链验真)。
- 创世机构：49,593 个公权机构和私权非营利法人“公民链技术发展基金会”统一在创世阶段上链。公民链基金会使用 `PrivateManage/PrivateAdmins`，其 CID、协议账户、一名程伟管理员、同一账户的三项固定岗位任职、机构阈值 2 和法定代表人引用来自 runtime primitives 单一常量源；OnChina 运行期读取链上机构、admins、岗位权限和有效任职，并用本地投影补齐展示字段，不生成第二套机构授权真源。
- 公民档案先本地建档并发电子护照,不要求链账户;注册局推送链上投票身份时才录入 `account_id`、要求目标公民签名,并由注册局管理员提交 `CitizenIdentity.register_voting_identity`。
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
- `node/src/core/command.rs` 当前把省略 `--chain`、`citizenchain`、`dev`、`local`、`staging` 统一加载为同一份冻结正式 chainspec。
- `citizenchain-fresh` 只允许用于重新生成冻结 chainspec 的本机 bake 流程。
- 生产部署必须显式使用 `--chain citizenchain`；`mainnet` 不是内置链标识，会被当成文件路径解析。

### 6.3 当前运行形态
- 数据库存储：RocksDB
- 网络层：`libp2p` / `litep2p`
- 默认本地 RPC：`127.0.0.1:9944`
- 默认本地 Prometheus：`127.0.0.1:9615`

### 6.4 云节点角色与公民端接入

冻结 chainspec 固定包含 44 个权威引导节点：第 1 个是国储会权威节点，其余 43 个后续逐步部署。权威节点和公开 bootnode 是同一台安装 CitizenChain 软件的服务器，不拆成两种节点；其公网职责与私有职责按端口隔离：

- 公网 P2P：每个权威引导节点开放 `30333/TCP` 的 WSS/libp2p 入口，供 CitizenApp 轻节点、普通全节点和其他权威节点连接。
- 本机 RPC：`9944/TCP` 只监听回环地址，不配置 `--rpc-external`，不下发给 CitizenApp。
- 监控和管理：Prometheus、OnChina、数据库和 SSH 默认不向公网开放；没有运维需求时不创建对应公网入口。
- Cloudflare 链连接：首期 Worker 只通过 Access + 独立 Tunnel 访问国储会节点的本机 RPC；Worker 使用远端 Secret 保存 HTTPS URL 与 Access 服务令牌，只允许内部固定的 `state_getStorage`、`author_submitExtrinsic`，不提供通用 JSON-RPC 代理。后续最多选择少量权威引导节点作为私有 RPC 备用，不连接全部 44 个节点。

CitizenApp P2P 暂时不可用时，聊天和广场不依赖链节点 RPC，继续走 Cloudflare；链上关键状态必须等待轻节点恢复或通过 Worker 受控接口完成已签名交易广播后，再由 finalized 链状态确认。

## 7. Runtime（`runtime/`）

### 7.1 定位
- `citizenchain` 是统一链上状态机。
- 账户体系、交易扩展、链上 pallet 装配、runtime API、创世配置都由这里统一编译到 wasm。

### 7.2 当前实现特征
- `AccountId` 与公钥等价，链上账户体系直接以公钥签名身份为主。
- 交易扩展中显式拒绝 `stake` 账户作为发送方。
- runtime 当前直接依赖本产品的治理、发行、交易、其他 pallet。
- 创世配置由 `runtime/src/genesis_config_presets.rs` 提供。
- 公民宪法创世正文唯一文件为 `runtime/public/legislation-yuan/src/constitution.scale`；该文件是结构化 `章>节>条>款` SCALE 数据，运行态注入为 `law_id=0`，修改必须经 runtime 二次确认并通过 `legislation-yuan` 解码/创世测试。
- 创世法律版本标签唯一常量在 `runtime/primitives/src/genesis.rs`：`GENESIS_LAW_VERSION_LABELS` 目前固定写入 `(law_id=0, version=1) -> 创世版 / Genesis Edition`；runtime 创世构建把该常量写入 `LegislationYuan.LawVersionLabels`，显示端不得本地推断 `v1=创世版`。

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
- 所有节点统一注册 GRANDPA 网络协议并挂载 warp proof provider；只有本地持有且匹配当前 authority set 私钥的节点启动 `grandpa-voter` 参与最终性投票。
- 普通节点启动 `grandpa-observer` 消费最终性通知，不参与投票；该 observer 同时避免协议接收端提前关闭触发 `EssentialTaskClosed`。
- GRANDPA 持久化仅保留恢复与 proof 所需的覆盖写状态；按轮次追加的 `concluded_rounds` 已在本地 vendored `sc-consensus-grandpa` 中停用，用于限制多节点长期运行时的 AUX 膨胀。
- 新安装 CitizenApp 的快速接入依赖 GRANDPA warp：客户端从签名安装包内置 finalized 锚点验证 authority set 交接 proof，再下载近头 runtime/state proof。公开权威引导节点必须维持归档状态和 finalized 正典历史，不得只提供 peer discovery。

### 8.3 链身份
- 地址显示格式使用自定义 `SS58 = 2027`。
- 链名、链 ID、Token 显示属性统一来自 `runtime/primitives` 与 chain spec 配置。

## 9. 链上模块分组

### 9.1 治理模块（`runtime/governance/`）
- 投票引擎负责内部投票、联合投票、立法投票和选举投票的资格快照、票据、阈值、计票及终态；最终性密钥、运行时升级、销毁、决议发行等具体业务仍归各自业务模块。
- ADR-039 目标中，每个业务模块先按完整 `RoleSubject(cid_number, role_code)` 校验发起权限，在代码中静态指定唯一投票引擎并绑定 VotePlan；投票引擎不得由调用方选择，也不得执行具体业务。

当前模块：
- `grandpakey-change`
- `resolution-destro`
- `runtime-upgrade`
- `votingengine`

### 9.2 管理员模块（`runtime/admins/`）
- 负责公权机构管理员、私权机构管理员和个人多签管理员；固定治理机构初始管理员由链配置写入，运行期治理归公权管理员模块。
- 机构管理员集合真源归 `admins`，机构管理员是可任职人员。公权、私权机构值结构统一为 `Admin { account_id, cid_number, family_name, given_name }`，非空公民 CID 必须与 `citizen-identity` 的 `AccountIdByCid` / `CidByAccountId` 双向真源一致。个人多签虽复用同一 SCALE 结构，但按个人多签规则处理字段完整性。管理员账户本身没有机构业务权限。
- ADR-039 目标授权主体为 `RoleSubject(cid_number, role_code)`。岗位、强类型 `RoleBusinessPermission` 和任职真源归 `entity`；业务动作、指定投票引擎和执行真源归业务模块。个人多签保持独立 `AuthorizationSubject::PersonalMultisig`。
- `public-admins`、`private-admins`、`personal-admins` 的 `AdminAccounts` 统一接受四字段 `Admin` SCALE 布局；旧纯账户、旧三字段、旧合并姓名和历史 storage migration 均已删除，不保留兼容或双轨。字段是否必须非空由机构类型、岗位和个人多签规则分别判定。

当前模块：
- `admin-primitives`
- `public-admins`
- `private-admins`
- `personal-admins`

### 9.3 实体模块（`runtime/entity/`）
- 负责公权机构、私权机构、个人多签账户的创建、关闭、资金与生命周期治理。
- 机构管理已按公权/私权拆分两 pallet(取代旧 `organization-manage`)。
- 公权/私权 entity 已保存岗位、岗位权限、任职、`InstitutionRoleNonce` 和永久 `UsedRoleCodes`，并提供 CID 能力封顶、有效任职和岗位权限的统一查询。所有机构强制存在可空缺的 `LR`；普通机构原子创建将在独立业务模块中同时建立至少一个初始治理岗位、权限、任职和投票规则。
- 动态岗位码由 runtime 使用所属 pallet 的 `MODULE_TAG` 作哈希域生成，调用方不得提交；岗位码及权限不可修改，岗位名可以依法修改。

当前模块：
- `public-manage`（公权机构生命周期,idx30）
- `private-manage`（私权机构生命周期,idx31）
- `personal-manage`（个人多签）

### 9.4 公权业务模块（`runtime/public/`）
- 负责公权机构的业务壳。业务壳只解释业务规则和写回业务真源，不复刻投票流程。
- `legislation-yuan` 是立法业务壳；立法表决、计票和公投流程归 `legislation-vote`。
- 开发期通用选举业务骨架已经删除。未来每种具体公权选举业务在 `runtime/public/` 下新增独立模块；具体业务模块定义规则，选举投票、计票和结果快照统一归 `election-vote`。

当前模块：
- `legislation-yuan`（idx25）

已删除的开发期通用选举业务壳原占用 index 32；该编号永久留空，不复用。

### 9.5 发行模块（`runtime/issuance/`）
- 负责公民发行、全节点发行、省储行利息、决议发行完整流程。

当前模块：
- `citizen-issuance`
- `fullnode-issuance`
- `resolution-issuance`
- `provincialbank-interest`

### 9.6 交易模块（`runtime/transaction/`）
- 负责链上交易手续费、链下交易手续费、机构多签交易能力。

当前模块：
- `multisig`
- `offchain`
- `onchain`

### 9.7 其他模块（`runtime/misc/`）
- 负责链上公民身份、人口统计、PoW 难度调整等基础能力。
- `citizen-identity` 是链上投票身份、参选身份和全国、省、市、镇四级有效人口数据的唯一真源；投票引擎按提案需要消费这些人口数据并生成快照。投票身份载荷保留注册时核验的 `citizen_age_years` 以执行最低年龄门禁；竞选身份另保存出生日期，并由链上日期实时计算竞选年龄。

当前模块：
- `pow-difficulty`
- `citizen-identity`

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
- 管理员、治理、转账、清算和奖励设置等桌面桥接统一把账户字段输出为 `account_id` / `<role>_account_id`，把签名公钥输出为 `signer_public_key`，把展示地址输出为 `ss58_address`；前端不得重新创造同义字段。
- Node 端所有公民钱包离线扫码签名 UI 统一由 `node/frontend/shared/qr/CitizenSignaturePanel.tsx` 和 `CitizenSignatureModal.tsx` 承载：左侧固定“扫码签名”，右侧固定“识别签名”，面板只显示二维码有效期倒计时，不显示内部 request id 或签名账户地址；业务页面只负责构造请求、验签和提交交易。地址扫码填入等非签名二维码不纳入该组件。
- 设置页的“全节点模式”当前展示归档全节点和普通全节点：默认归档全节点；普通全节点置灰不可选择；在底层剪裁能力完成前，节点实际仍按归档全节点运行。
- 设置页在“全节点模式”之后提供“链上中国平台”手动启动行，显示 `未开启` / `启动中` / `已开启` 状态标签、固定入口 `https://onchina.local:8964` 和“启动 / 关闭”按钮；点击后必须二次确认，只启动或停止 OnChina 子进程，不自动打开浏览器；只有 `/api/v1/health` 真实健康检查通过后才显示 `已开启`。

### 10.3 打包边界
- 桌面端与原生节点在同一个 `node` crate 中构建，Tauri 打包从 `node/frontend/dist` 读取前端产物。
- 对用户交付形态始终保持单个桌面安装包；对工程实现来说仍是“UI 壳 + 内嵌 node 二进制”。

## 11. 变更与发布边界

正式创世前，CitizenChain runtime 的 `authoring/spec/impl/transaction/system` 五项版本全部为 `0`，所有项目 pallet `StorageVersion` 为 `0`，workspace/Node/runtime 本地程序包版本为 `0.0.0`。当前结构调整直接落到创世终态，不编写 migration 或兼容分支；第三方依赖和 Substrate runtime API trait 协议版本不属于项目版本归零范围。

### 11.1 需要 runtime 升级的改动
- `runtime/` 中的状态机、类型、交易校验、runtime API。
- `runtime/governance/`、`runtime/admins/`、`runtime/private/`、`runtime/issuance/`、`runtime/transaction/`、`runtime/misc/` 中被 runtime 直接引用的链上逻辑。
- `runtime/primitives/` 中被 runtime 直接使用、并影响链上行为的常量 / 类型 /编码结构。

### 11.2 不需要 runtime 升级的改动
- `node/` 中的 CLI、RPC、服务编排、网络与本地运行逻辑。
- `node/` 的桌面 UI、设置页、Tauri 命令与安装包逻辑。
- 构建脚本、CI/CD、前端界面、说明文档。

### 11.3 CI 发布边界
- `citizenchain-wasm.yml` 只允许手动触发并始终按仓库源码原样编译 WASM，不查询链、不读取 SSH/RPC Secret、不连接服务器，也不在 CI 工作区改写版本。正式创世前版本保持 `0`；正式创世后只有公民控制台「运行 WASM CI」读取已配置目标链，并要求 RPC 实际 genesis hash 与本机明确保存的 `CHAIN_GENESIS_HASH` 相等；随后在源码版本严格等于链上版本时把源码 `spec_version` 及现有测试断言同步加一、提交并触发带版本/genesis 校验的升级构建。其他手动触发只做普通源码构建，不提高版本；该入口只生成 WASM，不自动执行链上升级。
- `citizenchain-ci.yml` 的 push 与 `mode=ci` 只做桌面端打包检查和本次 run artifact 上传，不读取 Tauri updater 签名私钥、不发布 GitHub Release、不部署服务器。
- 根 `citizenconsole/` 控制台选择“正式 Release”时以 `mode=release` 自动触发 updater 签名、`citizenchain-latest.json` 和 GitHub Release。“部署服务器”是与本地工作区、当前 HEAD 和 Release 解耦的独立生产入口：从 `institution-catalog.json` 选择节点后，只消费 GitHub `main` 最新成功 CitizenChain CI 的 Linux amd artifact；目标服务器使用 GitHub 短期签名地址直接下载，本机只传服务配置和节点密钥，不下载或转传安装包。
- 每个权威节点的服务器 IP、节点身份 Ed25519 私钥和 GRANDPA 验证私钥按 `node-01` 至 `node-44` 隔离保存在 macOS Keychain。由部署控制台管理的服务器统一使用 `deploy` SSH 身份，私钥复制到对应节点 Keychain 项但不得保留 `.ssh` 明文文件，本机只保留 `deploy.pub`；网页只返回 IP、公开 PeerId、公开 GRANDPA 公钥和“已配置/缺失”状态，且 SSH 项只有完整私钥才算已配置。保存、更换与部署均逐次要求 Touch ID。
- 节点私钥保存前必须从私钥推导公开身份，并分别与 `institution-catalog.json` 的 PeerId 和 GRANDPA 公钥精确匹配；不匹配时禁止写入。部署时只把选中节点的密钥写入权限为 `0600` 的节点身份文件和 `gran` keystore，清理远端临时文件，并真实检查 systemd、冻结块 0 哈希、RPC health、Authority/Validator 角色和本节点 PeerId。
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
- `runtime/misc/pow-difficulty/POW_DIFFICULTY_TECHNICAL.md`
- `runtime/misc/citizen-identity/CITIZEN_IDENTITY_TECHNICAL.md`

### 12.5 桌面节点 UI
- `memory/05-modules/citizenchain/node/home/HOME_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/mining/dashboard/MINING_DASHBOARD_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/mining/network_overview/NETWORK_OVERVIEW_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/other/other-tabs/OTHER_TABS_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/bootnodes_address/BOOTNODES_ADDRESS_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/device-password/DEVICE_PASSWORD_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/reward_account/REWARD_ACCOUNT_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/settings/grandpa_address/GRANDPA_ADDRESS_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`（第 9 节记录全节点模式设置边界）

## 13. 维护要求
- `citizenchain` 发生架构级、边界级、发布级改动时，必须同步更新本文档。
- 模块行为变更时，必须同时更新对应模块技术文档。
- 若产品级口径与模块级口径冲突，以代码实现为准，并应在本次改动中同时修正文档。

## 14. 本地 fresh Node 验收口径

- macOS 节点二进制默认进入桌面模式；需要命令行指定 `--chain citizenchain-fresh --tmp` 做隔离验收时，必须同时设置 `CITIZENCHAIN_HEADLESS=1`，否则命令行 chain 参数不会代表实际启动的桌面节点状态。
- fresh 验收必须读取 RPC 的 block 0、health、六项项目 Runtime 版本、metadata、genesis hash 与 state root，并在结束后停止节点。与既有 bootnode 的 genesis 不一致只说明正式 chainspec 尚未统一，不得通过削弱 NodeGuard 或复用旧链数据规避。
- 2026-07-22 最终验收：block #0/genesis hash `0x4bd7e3f65f5ad4788e6ac8917abce9b0683f0c93d286766a7512854084ff0dd9`，state root `0xd15b1a20d972f0cc5f64aa9a08a09f6793fe51886f9445c6dc953c0f9d438f7b`，`peers=0`、`isSyncing=false`，六项项目 Runtime 版本均为 `0`，metadata 二进制 220,247 字节；验收节点已停止，未生成正式 chainspec。
