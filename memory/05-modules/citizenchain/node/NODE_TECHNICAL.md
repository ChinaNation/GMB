# NODE Technical Notes

## 0. 模块定位

`node` 是 CitizenChain 全节点可执行程序，实现双共识架构（PoW + GRANDPA）、自定义 RPC 接口和挖矿子系统。

代码位置：`/Users/rhett/GMB/citizenchain/node/`

## 1. 双共识架构

### 1.1 PoW 共识
- 算法：`SimplePow` — `blake2_256(pre_hash ++ nonce_le_bytes)` 与目标值比较
- 难度：节点共识从链上 `PowDifficulty::CurrentDifficulty` RAW storage 读取；NodeGuard 不再复算或冻结 PoW 难度参数
- 密钥类型：`powr`（sr25519），首次启动自动生成 BIP39 并写入 keystore 磁盘
- 出块时序：有效 PoW 找到后立即提交；固定平均六分钟只用于 runtime 难度窗口，不是节点等待时间

### 1.2 GRANDPA 最终性
- 权威节点（本地有 GRANDPA ed25519 密钥）：运行 `grandpa-voter`
- 普通节点：运行 `grandpa-observer`（只接收最终性结果不投票）
- 所有节点统一注册 GRANDPA 网络协议，保证协议栈一致
- Justification 周期：64 块
- vendor 目录：`sc-consensus-grandpa` v0.40.0（独立 GPL-3.0 许可）

### 1.2.1 GRANDPA warp 服务

- `src/core/service.rs` 为所有节点挂载 `warp_proof::NetworkProvider`；权威 voter 和普通 observer 都可基于本地数据响应轻客户端 warp proof。
- 节点每次 finalized 推进都会覆盖保存最新 GRANDPA justification；64 块周期只控制普通历史块 justification 的额外挂载，authority set 切换块始终持久化 justification。
- proof 生成依赖请求起点的 finalized 正典 hash/header、各 authority set 切换块 header/justification、最新 best justification 和目标 finalized state。生产节点必须保持 `--state-pruning archive`，并保留 finalized 正典块；不得把缺失这些数据的剪裁节点作为 CitizenApp 新安装用户的 warp 服务节点。
- warp 单段 proof 上限 8 MiB，超过后由客户端从上一段末尾继续请求；同步成本主要随 authority set 变更数量增长。
- 公开网络进入规模化使用前，至少 3 个彼此独立的归档节点必须长期在线提供 warp。运维监控要区分 peer 可连接、finalized 是否推进、warp proof 是否成功，不能只监控 P2P 端口存活。

### 1.3 libp2p WebSocket 本地覆盖
- 本地目录：`citizenchain/node/libp2p-websocket/`
- 覆盖方式：`citizenchain/Cargo.toml` 通过 `[patch.crates-io]` 将 crates.io 的 `libp2p-websocket` 指向该本地目录。
- 包名约束：本地 crate 的 `name` 必须继续保持 `libp2p-websocket`，否则 Cargo patch 无法覆盖上游同名包。
- 当前改动点：公开 `tls::Config` 的 `client` 字段，支持节点在 WSS transport 中注入自定义 TLS 客户端。TLS 层只负责传输加密，P2P 身份认证仍由 Noise 协议通过 peer ID 保证。

### 1.4 节点永久规则导入层

- 所有区块统一按 `ConstitutionGuard<NodeGuard<PowBlockImport>>` 导入；本地产块和网络区块没有旁路。
- `ConstitutionGuard` 保持独立、最外层，只负责公民宪法最高规则。
- `NodeGuard` 统一承载固定治理骨架、全节点 PoW 发行、公民认证发行和 CID/机构生命周期，不为单项规则新增平行包装器。
- 全节点与公民认证两类 `on_finalize` 铸发进入共享发行计划，按账户汇总后统一核对余额与总发行；未登记的 finalize 发行直接拒绝。
- CID 策略只永久保护 block#0 机构不被删除、跨命名空间复制或替换身份；普通机构继续由 runtime 依法创建、修改、关闭和删除，删除时必须同步清理账户正反索引。
- 固定治理骨架当前合计保护 90 个机构：原 89 个公权机构继续读取 `PublicAdmins/PublicManage`，公民链技术发展基金会读取 `PrivateAdmins/PrivateManage`；基金会的协议账户、一名管理员、同一钱包的三项固定岗位任职、固定权限和机构阈值闭环同样 fail-closed。
- 详细规则、信任上限与验收基线见 `memory/05-modules/citizenchain/node/node-guard/NODE_GUARD_TECHNICAL.md`。
- 2026-07-12 最终三节点验收：A/B/C 临时 fresh 网络同步到 block#1
  `0xe0fccc0790f9761226865a2fa96a5eb9e19eb34169191f49faf3afee4817b3c8` 和 block#2
  `0x961012a973cf9695367037b7f9554df2ef541cda17ed5315a7c72b2600bd2a0a`；同期 NodeGuard `76/76`、
  ConstitutionGuard `40/40`，拒绝矩阵后合法链继续推进。

## 2. 挖矿子系统

### 2.1 CPU 挖矿
- 线程数：`--mining-threads`（默认 CPU 可用并行度，0 禁用）
- nonce 空间：低半区（bit63=0），基于 pre_hash 前 8 字节的随机基址 + 线程号错位
- 哈希率统计：thread 0 每 100,000 次哈希采样，乘以线程数得总哈希率
- 提交门控：ready 交易池、major sync、引导状态和 proposal 版本门控；找到有效 PoW 即提交，不按目标时间 sleep

### 2.2 GPU 挖矿（可选）
- 编译 feature：`gpu-mining`（依赖 `ocl` crate）
- CLI 参数：`--gpu-device INDEX`，`--no-gpu` 强制禁用
- nonce 空间：高半区（bit63=1），与 CPU 不重叠
- 批次大小：2^24（~16M nonces/batch）
- OpenCL kernel：`kernels/blake2b_pow.cl`

### 2.3 出块策略
- 空交易池（`pool.status().ready == 0`）时跳过挖矿，避免空块
- 离线或 major sync 时禁止出块，防止本地分叉
- 非引导节点必须先从网络导入至少 1 个块才允许出块

### 2.4 交易池生命周期
- CitizenChain 默认把 Substrate 交易池固定为 `SingleState`。当前链不依赖 fork-aware 多视图池；上游 fork-aware 后台子任务在本链 fresh / 普通节点启动场景会提前结束，进而触发 `txpool-background` essential task 关闭服务。
- 用户显式传入 `--pool-type` 时仍尊重 CLI 参数；生产默认路径不要求用户手工追加参数。
- `service::new_full` 会让 `TaskManager` 持有交易池 clone，防止交易池句柄在服务组装后被提前释放。

## 3. RPC 接口

生产权威引导节点的 RPC 固定为本机能力：`127.0.0.1:9944`、`--rpc-methods Safe`，不得使用 `--rpc-external`、`--unsafe-rpc-external` 或 `--rpc-cors all`。后续 Cloudflare 链连接只能通过 Access + Tunnel 到达本机 RPC，不开放 Oracle/主机防火墙入站端口。

| 方法 | 说明 |
|------|------|
| `mining_cpuHashrate` | CPU 全线程合计哈希率（hashes/sec） |
| `mining_gpuHashrate` | GPU 哈希率（仅 gpu-mining feature） |
| `reward_bindAccount(account_id)` | 节点端签名提交 `bind_reward_account` 交易 |
| `reward_rebindAccount(account_id)` | 节点端签名提交 `rebind_reward_account` 交易 |
| `transaction_submitMinerTransfer(ss58, amount_fen, remark, token)` | 节点端使用 `powr` 密钥提交矿工热钱包 `OnchainTransaction::transfer_with_remark` 转账，备注最多 99 UTF-8 字节，要求进程内一次性令牌 |
| `fee_blockFees(block_hash_hex)` | 读取指定区块的 FeePaid 事件累计手续费 |
| `sync_state_genLightSyncState` | 返回小体积 lightSyncState checkpoint（finalized header + GRANDPA authority set） |

### 3.1 权威引导节点网络基线

- 44 个权威引导节点使用同一套安装包和 P2P 端口策略，第 1 个为国储会权威节点。
- 唯一固定公网业务入口为 `/ip4/0.0.0.0/tcp/30333/wss`；当前链没有 UDP/QUIC P2P 监听，因此不开放 `30333/UDP`。
- 云节点显式限制 `in_peers=32`、`in_peers_light=100`、`out_peers=8`、`max_parallel_downloads=5`；这些数值是当前 SDK 默认安全基线，扩容前必须压测。
- Prometheus 默认关闭，OnChina 不随节点自动启动；没有运维需求时不得开放 SSH、OnChina 或数据库端口。

### RPC 交易签名
- 使用 `powr` keystore 密钥签名
- spec_version 从链上 WASM 运行时读取（非 native 编译时常量），防止升级后 BadProof
- TxExtension、SignedPayload 和 UncheckedExtrinsic 统一由 `citizenchain/crates/chain-signing` 构造
- 矿工热钱包转账 RPC 额外要求一次性令牌；令牌由桌面 Tauri 命令在设备密码校验通过后生成并由 RPC 消费

## 4. Chain Spec 与创世链状态包（冻结铁律）

主网创世后,chainspec 与创世链状态包都必须永久冻结。公权机构唯一真源是链上
`genesis + 后续交易状态`;节点本地数据库只是链状态副本。

- 冻结 chainspec：[citizenchain/node/chainspecs/citizenchain.plain.json](../../../../citizenchain/node/chainspecs/citizenchain.plain.json),plain 形态只保存 runtime WASM、genesis patch、当前 6 个已部署 bootnode、token 属性和协议 ID；44 个权威节点是规划身份目录，不得把未部署节点写入当前联网基线。
- 创世配置：正式安装包同时内置冻结 plain chainspec 和经 release 清单验证的
  `genesis-state/chains/citizenchain/db`。首启复制状态包到独立节点数据目录；开发/排障
  缺少状态包时才允许按同一 plain chainspec 本地物化。preview 状态包永远不得进入安装包。
- 当前唯一冻结锚点（2026-07-16，runtime 源提交 `7abac7982a5c5ee25580583d456523ce2132743e`，冻结资产提交 `80f58aa5cfe19713edfba7331ea2896cacf09b62`，GitHub `CitizenChain WASM` run `29530114067`）：`genesis_hash=0x840d5b12c541a010783e54069c9168a13d102ba63cd8f3a00263440c1803aad9`、`state_root=0x99b4cb3031baa5e87536a22190dc81bf6bf49d3678c0abae86a312268506fe09`、`runtime_wasm_hash=be4585ce369e658e6799be667ed5be692fc050f9c6196ab14c53f7dfa5dc6e70`、`chainspec_hash=5e609d166e8517d20ec0cd2095b88825146e34e64b3ebaba54152c7bde9d1f60`、`light_sync_state_hash=4b05735ed59a8ef3756bf6445f1e4fa744730d2161ad14a62be1e16856bbfb9a`、`public_institution_root=ecff487ce7d2bac6cb89d064a456187b453acd27f4bee2b140f474a48d072682`。
- 同一次正式 bake 已通过公民宪法创世校验并在 50 秒完成物化；正式包的仓库外隔离副本使用默认内嵌链规范真实启动后，RPC 返回上述 block#0/state root、`isSyncing=false`，进程正常退出。正式包目录禁止直接作为 `--base-path`，避免写入节点密钥和网络运行状态。旧管理员 storage 布局及旧创世锚点不再作为当前发布基线。
- 加载方式：[chain_spec.rs](../../../../citizenchain/node/src/core/chain_spec.rs) 用 `include_bytes!` 加载冻结 plain JSON；启动流程优先安装 release 创世状态包，随后仍由 `GenesisBlockBuilder` 对同一块 0 规格进行一致性校验。
- 当前限制：即使已复制 `genesis-state` RocksDB,Substrate 启动仍会根据 plain spec 调 `GenesisBlockBuilder` 校验链初始块;这不是重新生成并写入链数据库,但会产生分钟级 CPU 成本。首次本地数据准备显示“初始化中”，已有数据库显示“启动中”，均需等待 `chain_getBlockHash(0)` 成功后才进入“运行中”。
- 全网一致性保证：plain JSON、CI WASM、创世链状态包 manifest 中的 `genesis_hash/state_root/runtime_wasm_hash/chainspec_hash/public_institution_root` 必须一致，manifest 还必须绑定 `runtime_wasm_ci_run_id/runtime_wasm_ci_head_sha`;后续 runtime 升级一律走链上 `setCode`,不重写创世包。
- 发布与服务器安装：`citizenchain-ci.yml` 构建四个平台节点软件时只能消费
  `artifact_stage=release` 且 hash/provenance 全部匹配的状态包；服务器部署保留节点身份
  密钥与 GRANDPA keystore，并只在新链数据目录安装冻结块 0 数据库。
- 部署控制台批量操作：CitizenChain“部署服务器”会并发启动所有配置齐全节点；成功节点不回显过程日志，失败节点实时回显脱敏失败日志，结束后输出成功/失败/跳过汇总；节点卡片“部署该节点”仍只执行并显示单节点日志。
- 运行态可用标准：进程内节点线程存活不等于节点可用。首页状态和链上中国启动前置检查都必须等待 `chain_getBlockHash(0)` 成功；RPC ready 前按本地数据是否首次准备分别显示“初始化中”或“启动中”，不得标记“运行中”。桌面启动路径固定使用 `SingleState` 交易池；必要后台任务退出时必须立即保存并展示真实原因，禁止继续等待 RPC 超时。

正式创世冻结流程(只做一次,**永不重做**):

1. GitHub `CitizenChain WASM` CI 成功后,下载同一提交的 `citizenchain-wasm` artifact。
2. 取 `citizenchain.compact.compressed.wasm` 作为创世 `:code` 字节源。
3. 执行 `citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM> --wasm-ci-run-id <RUN_ID> --wasm-ci-head-sha <HEAD_SHA>`；公权机构分片及根必须由脚本从同一块 0 直接生成，不接受外部参数。
4. 脚本必须通过 `check-constitution-genesis.py --expect-code-file <CI_WASM>` 校验,确认 `:code` 字节等于 CI WASM,公民宪法 `law_id=0`、v1 直接生效且无待生效版。
5. 脚本原子准备全节点 plain、release 状态包、CitizenApp chainspec/checkpoint、43 个
   公权机构分片以及 Cloudflare 各环境链身份锚点，并在覆盖正式文件前完成交叉校验。
6. `prepack.sh` / `prepack.ps1` 只复制 `artifact_stage=release` 的状态包；preview、缺失 CI
   provenance、chainspec hash 不一致或包含清单外路径一律失败关闭。

后续 runtime 升级一律走链上 `setCode`(governance/runtime-upgrade),**绝不**重新烘焙或覆盖这份 JSON。

预上线重新创世脚本例外：

- [clean-run.sh](../../../../citizenchain/scripts/clean-run.sh) 只用于本机清开发链后按当前源码现造 fresh genesis；runtime WASM 来自当前源码构建，不下载 CI artifact。
- clean-run 不再执行 `seed-federal-admins`；FRG 在 `PublicAdmins::AdminAccounts` 中只有一个 215 人管理员集合，43 个省专员岗位及每岗 5 人任职由 genesis 写入 entity 真源。
- 正式创世前需要真正替换仓库 SSOT 时,唯一入口是 [bake-chainspec.sh](../../../../citizenchain/scripts/bake-chainspec.sh)。
- `bake-chainspec.sh` 用 `citizenchain-fresh` 入口生成新的 plain chainspec,并保留当前 SSOT 中的 6 个已部署 bootNodes。
- fresh plain chainspec 的 genesis `:code` 必须与下载的 CI WASM 字节一致。
- 脚本同时写回 `citizenchain/node/chainspecs/citizenchain.plain.json` 与 `citizenapp/assets/chainspec.json`,并导出 `target/chainspec/genesis-state/`,保证全节点和轻节点使用同一创世锚点。
- 不带 `--finalize` 时只在 `target/chainspec/` 生成 `artifact_stage=preview` 候选；不得覆盖
  正式 node/App/Cloudflare 资产，也不得被 prepack 接受。

2026-07-16 第 5 步仅完成创世准备：preview 候选块 0 为
`0x8347f61bd28c93c4ce6d6b98f4b5a70f185841e0ac87b0bab9eb8c6caf8375ed`，state root 为
`0x467996c0094900833e30ff0a11e668aaf234abc35acdb4917f858702642ee707`。该值已经过隔离
node、NodeGuard、OnChina 真实投影/API/页面验证，但不是冻结值；2026-07-14 正式锚点保持不变，
后续必须等 CI WASM、release freeze 与正式创世阶段再执行 `--finalize`。

2026-07-01 正式创世冻结收口:

- GitHub WASM run:`28492547251`,提交 `208ae60d81828d04946239e21b648b8f1ba0c2a0`,artifact `citizenchain-wasm` id `7999877697`。
- CI WASM `citizenchain.compact.compressed.wasm` sha256:`b6d8c9dcee90df963dcda89c96b18c8f3361d37f31c52686dddda0480195df92`。
- 当时仍处 raw 过渡口径;2026-07-04 后正式口径改为 plain SSOT + genesis-state + CitizenApp stateRootHash。
- genesis hash:`0x6c88667d43f5a2690f2cb176f5883e051a057db6bee5fa56bc8337becbf23417`。
- 宪法创世检查通过:`law_id=0`、`tier=Constitution`、`effective_version=1`、`latest_version=1`、`pending_version=None`,不可修改条款 `1,2,3,17,19,24,34,42` manifest 与创世条文摘要一致。
- 临时全新 base-path 真实节点烟测通过,`constitution_getDocument.source=legislation-raw`。

2026-06-19 预上线重新创世收口:

- runtime 本地 release WASM blake2:`f213cdc476fb0d1e723421a5bd1f5afafc792b5180852d2266346b967386e680`
- 当时产物为历史 raw 形态;2026-07-04 后不再作为当前部署口径。
- 该历史阶段曾预登记 44 个计划节点；2026-07-12 已按实际部署状态收口为 6 个 bootnode，未部署域名不再进入 chainspec。

历史:2026-05-06 首次冻结,源 nrcgch.crcfrcn.com,sha256 `2b9f46e4aefb66f892d5dc170b2c2bfc33b6b12a88192617b06c18e8ea38a2db`。

## 5. CLI 参数

| 参数 | 说明 |
|------|------|
| `--mining-threads COUNT` | 挖矿线程数（0 禁用，默认 CPU 并行度） |
| `--gpu-device INDEX` | GPU 设备编号 |
| `--no-gpu` | 强制禁用 GPU |
| 子命令 | key / export-chain-spec / check-block / export-blocks / import-blocks / purge-chain / revert / benchmark / chain-info |

## 6. 治理桌面页账户数据链路

- 地址真源：
  - `node/src/governance/registry.rs` 直接读取 `runtime/primitives/cid/china/china_cb.rs`、`runtime/primitives/cid/china/china_ch.rs` 和 `SAFETY_FUND_ACCOUNT`
  - `治理 -> 国家储委会 / 省储委会 / 省储行` 页面的 `主账户 / 费用账户 / 安全基金账户 / 永久质押账户` 不再允许 node 侧手抄第二份地址表
- 金额真源：
  - `node/src/governance/institution.rs` 先取 `chain_getFinalizedHead`
  - 再用同一个 `block_hash` 调 `state_getStorage(System::Account)` 读取 `free` 余额
  - 同一详情页内所有账户金额必须来自同一个 finalized 快照
- 实时刷新：
  - `node/src/governance/balance_watch.rs` 在详情页打开时启动 watcher
  - watcher 每秒检查一次 finalized hash，哈希变化后重新查询当前页面全部账户余额
  - 查询结果通过 Tauri 事件 `governance-balance-updated` 推给前端
- 前端约束：
  - `node/frontend/governance/InstitutionDetailPage.tsx` 只监听事件并覆盖现有 state
  - 不改 UI 布局、不改卡片顺序、不改现有中文命名

## 7. 协议升级 node 端边界

2026-05-09 起，node 端协议升级入口按“协议升级 / 开发升级”拆分，并统一收口在治理模块的 runtime-upgrade 目录。

- 后端实现：
  - `node/src/governance/runtime_upgrade/commands.rs`：Tauri 命令入口，保留 `build_propose_upgrade_request`、`submit_propose_upgrade`、`build_developer_upgrade_request`、`submit_developer_upgrade` 命令名。
  - `node/src/governance/runtime_upgrade/call_data.rs`：RuntimeUpgrade pallet call_data 编码，只承载 `propose_runtime_upgrade` 与 `developer_direct_upgrade`。
  - `node/src/governance/runtime_upgrade/signing.rs`：Runtime WASM 大 payload 的 QR 签名请求构建，通用签名校验仍复用 `node/src/governance/signing.rs`。
  - 开发升级命令从治理概览读取国家储委会 `cid_number`，显式编码固定委员岗位并构造 `developer_direct_upgrade(actor_cid_number, actor_role_code=COMMITTEE_MEMBER, code, pow_params)`；签名公钥还必须属于该 CID 的已激活 `admins`，最终由 runtime 校验完整三项授权。
- 前端实现：
  - `node/frontend/governance/runtime-upgrade/ProtocolUpgradeProposalPage.tsx`：国家储委会详情页“协议升级”，提交运行期协议升级提案，进入联合投票。
  - `node/frontend/governance/runtime-upgrade/DeveloperUpgradePage.tsx`：国家储委会详情页“开发升级”，只使用当前国家储委会已激活管理员发起开发期直升。
  - `node/frontend/governance/runtime-upgrade/api.ts`：协议升级专用 Tauri API；`governance/api.ts` 不再承载协议升级创建/提交接口。
- 入口约束：
  - 国家储委会详情页使用“协议升级”入口。
  - “开发升级”是独立按钮，放在“协议升级”后，不与协议升级合并。
  - 设置页不再保留任何开发升级入口或 `settings/developer-upgrade` 代码。
- 当前边界：
  - 第 1 步只调整 node 前后端入口、目录收口和 node 侧开发升级管理员校验。
  - node 端协议升级业务调用显式携带国家储委会 `actor_cid_number`，并提交 `reason + code`；不获取人口快照、不透传联合签名、不保存投票状态。

## 8. 桌面端更新边界

桌面端更新按“打开软件检查、设置页点击安装”执行：

- Tauri 插件：
  - Rust：`tauri-plugin-updater`、`tauri-plugin-process`
  - 前端：`@tauri-apps/plugin-updater`、`@tauri-apps/plugin-process`
- 更新源：
  - `tauri.conf.json` 的 `plugins.updater.endpoints` 指向 GitHub Release 资产 `citizenchain-latest.json`
  - 手动发布 CI 生成 `citizenchain-latest.json`，其中包含版本号、各平台下载 URL 和 Tauri 签名
- 前端行为：
  - `frontend/app/App.tsx` 在 App 打开后调用 updater `check()`，只检查，不下载、不安装
  - `frontend/settings/settings-panel/SettingsSection.tsx` 仅在存在新版本时，于“节点程序版本”版本号前显示“更新”按钮
  - 用户点击“更新”后才调用 `downloadAndInstall()` 和 `relaunch()`
- 后端协同：
  - `src/settings/desktop_update.rs` 暴露 `prepare_desktop_update`
  - 该命令只负责调用 `home::stop_node_blocking` 停止进程内节点，释放 RocksDB LOCK；安装与重启由 Tauri updater/process 插件负责
- CI 边界：
  - `push main` 只构建检查并上传 run artifact
  - GitHub 手动 `Run workflow` 才生成 updater 签名产物、发布 GitHub Release、部署服务器

## 9. 全节点模式设置边界

2026-05-23 起，桌面设置页新增“全节点模式”入口；2026-07-05 起，CitizenApp 私密聊天不再由区块链节点承载，桌面设置页不再提供 Chat 通信节点功能开关。

- 当前默认模式：归档全节点。
- 当前可选模式：归档全节点。
- 当前待完成模式：普通全节点。
- 展示项：
  - 归档全节点：可选择，保存完整链数据。
  - 普通全节点：置灰不可选择，后续用于剪裁历史链数据。
- 本地配置：
  - `src/settings/node-mode/mod.rs` 通过 Tauri 命令 `get_node_mode` / `set_node_mode` 读写 `<app_data>/node-mode.json`。
  - 当前版本只允许写入归档全节点；普通全节点仍会被后端拒绝。
  - 如果本机旧配置曾错误保存 `communication`，读取时按归档全节点清理。
- 切换边界：
  - 全节点模式只描述链数据保存方式，不承载 Chat 通信开关或聊天投递能力。
  - 后续普通全节点真正实现时，少数据模式切换到多数据模式应从网络补充同步数据；多数据模式切换到少数据模式应删除不再需要的本地数据。
  - 在普通全节点底层能力完成前，不得让设置页暗示当前已执行剪裁或删除数据。

### 9.1 链上中国平台启动边界

2026-06-29 起，节点桌面端只在用户手动确认后启动链上中国平台，不再随节点程序启动自动拉起 OnChina 子进程。

- 固定入口：`https://onchina.local:8964`。
- 设置页入口：`frontend/settings/OnChinaPlatformSection.tsx` 位于“全节点模式”之后，左侧显示“链上中国平台”，右侧显示 `未开启` / `启动中` / `已开启` / `启动失败` 状态标签，状态标签右侧显示固定入口，最右侧按钮按进程状态显示“启动”或“关闭”。
- 二次确认：点击“启动”或“关闭”只打开确认弹窗；确认后调用 `start_onchina_platform` 或 `stop_onchina_platform`，不自动打开浏览器。
- 后端命令：`src/settings/onchina_platform.rs` 提供 `get_onchina_platform` / `start_onchina_platform` / `stop_onchina_platform`，只返回本进程管理的 OnChina 子进程状态、`/api/v1/health` 真实健康结果和固定入口；只有健康接口返回 `UP` 才显示 `已开启`。进程已存在但健康检查暂未通过时只显示 `启动中`，不附带红色失败详情；启动动作最终失败或超时才显示 `启动失败` 和失败原因。
- 子进程管理：`src/onchina_proc.rs` 负责解析随包或开发期 `onchina` 二进制、注入链 RPC / 内嵌 PG / TLS / 前端资源环境变量、启动进程、清理已退出句柄和 App 退出时停掉已启动子进程。启动前会清理上一轮异常退出后遗留的旧 OnChina 孤儿进程和 8964 端口监听，避免旧服务占口或持有陈旧数据库连接池；如果 8964 被非 OnChina 进程占用则 fail-closed 并返回明确错误。
- 默认行为：`src/desktop/mod.rs` 仍自动启动区块链节点和同步守护，但不会自动启动链上中国平台，避免只挖矿节点承担 PostgreSQL、HTTPS 管理后台和浏览器业务入口。
- HTTPS 入口：OnChina TLS 证书目标主机为 `onchina.local`；旧 `localhost/127.0.0.1` 证书会在下次启动时按主机标记重新生成。

### 9.2 Chat 聊天承载边界

CitizenApp 私密聊天只保留 Cloudflare 瞬时转发、WebRTC 设备附件和手机近场聊天，区块链节点软件不承载聊天投递、密钥池或手机配对入口。

- 删除边界：已删除 `citizenchain/node/src/chat/`、`src/settings/communication-node/`、`frontend/settings/communication-node/`、已删除的节点聊天协议 注册、通信节点 Tauri 命令、桌面通信节点二维码和 `citizenchain/scripts/im-two-node-smoke.sh`。
- 全节点边界：归档全节点和普通全节点只描述链数据保存方式；节点同步、挖矿、治理、交易和链上中国平台启动不受 Chat 方案调整影响。
- App 聊天边界：CitizenApp 私聊和群聊使用钱包地址、OpenMLS 和 `GMB_CHAT_V1`；互联网密文由 Worker/DO 瞬时转发，附件由 WebRTC 设备间传输，近场由手机蓝牙/Wi-Fi 直连。
- 二维码边界：`QR_V1/k=5 chat_node_pairing` 已删除，扫码解析端应按未知类型拒绝；桌面节点不再生成聊天配对二维码。
- 禁止恢复：不得把聊天功能接回节点 RPC、`sc-network/libp2p` request-response 或通信节点开关；Chat 验收以 CitizenApp + Cloudflare staging 和双真机为准。

## 10. 桌面同步守护边界

2026-05-17 起，桌面端新增本机同步守护 `src/home/sync_guard.rs`，用于检测“底层 P2P 已连接、交易能广播，但 block sync peer 表为空，本机区块不同步”的进程内脱钩状态。

守护边界：
- 只采样本机 `127.0.0.1` RPC，不常规请求公网参考节点，不把引导节点当成持续依赖。
- 不以区块高度增长作为重启条件。当前出块策略会在交易池为空时跳过挖矿，网络无交易时高度停滞是正常状态。
- 不清链、不删除数据库、不改 `ws/wss`，也不自动重启进程内 Substrate 服务；Substrate/RocksDB 释放滞后时，同进程自动重启会触发 `lock hold by current process` 并让节点进入锁占用状态。
- 命中脱钩条件达到阈值后进入 `degraded` 状态并写审计日志；采样恢复正常后回到 `healthy`。
- 节点生命周期由 `src/home/process/mod.rs` 显式维护 `starting/running/stopping/restarting/failed/lock_held/exited/stopped`，首页会把同进程 DB 锁占用展示为“数据库锁未释放”。

准确触发条件以 `home/HOME_TECHNICAL.md` 为准，核心是 `system_health.peers == 0`、`system_peers == []`，同时 `system_unstable_networkState.connectedPeers` 仍存在带版本和 ping 的已识别 peer。

## 11. 文件索引

| 文件 | 行数 | 说明 |
|------|------|------|
| `src/core/service.rs` | 874 | 服务工厂、PoW 算法、CPU 挖矿、GRANDPA 角色选择；网络与挖矿统一装配 `ConstitutionGuard<NodeGuard<PowBlockImport>>` |
| `src/core/constitution/mod.rs` | 1603 | 宪法 RAW key、SCALE 镜像、创世基准、严格不变式与 38 个策略/渲染测试 |
| `src/core/constitution/guard.rs` | 214 | 独立最外层 `ConstitutionGuard`：启动全检、正常/预计算 delta、warp 提交前校验与 fail-closed |
| `src/core/constitution/render.rs` | 164 | 桌面端宪法 HTML 渲染，与共识守卫物理分离 |
| `src/core/constitution/constitution_shell.html` | 723 | 公民宪法桌面展示 HTML/CSS 外壳 |
| `src/core/node_guard/mod.rs` | 1928 | `NodeGuard` 统一 `BlockImport` 包装器；共享 finalize 前/后只读执行、固定治理岗位子树、发行计划、手续费、warp 扫描、fail-closed 与内层委派 |
| `src/core/node_guard/cid_lifecycle.rs` | 1026 | 公民 CID 既有永久规则、机构 CID 四项格式规则、创世机构身份基准、普通机构删除与账户正反索引完整性 |
| `src/core/node_guard/runtime_policy.rs` | 830 | 链上 0.1% 与最低 10 分、投票 100 分、链下最低 1 分和最高 0.1% 的区块结果与候选 WASM 行为守卫 |
| `src/core/node_guard/citizen_issuance.rs` | 535 | 公民认证发行永久策略；按首次身份、待发队列、双重防重、编译期档位及共享余额计划逐块复算 |
| `src/core/node_guard/fullnode_issuance.rs` | 737 | 全节点 PoW 发行永久策略；按 PoW digest、编译期常量、共享发行计划和审计状态逐块复算 |
| `src/core/node_guard/governance_skeleton.rs` | - | `NodeGuard` 创世治理骨架策略：校验 89 个公权机构及 1 个私权创世公民链基金会的完整身份、管理员人数、固定岗位、席位、任职和 admins 一致性；普通机构不触发该策略 |
| `src/core/node_guard/national_body_composition.rs` | - | 国家级成员机构组成策略：允许 NSN/NRP/NED 创世未组成，组成后永久校验法定岗位人数与 admins 闭环；普通写入和 `:code` 升级均校验固定治理机构内部阈值快照，六个国家单例不施加固定阈值 |
| `src/core/rpc.rs` | 419 | 节点核心 RPC、钱包绑定签名、哈希率查询、轻节点同步 |
| `src/mining/gpu_miner.rs` | 392 | OpenCL 初始化、GPU kernel 调度、哈希率统计 |
| `src/core/command.rs` | 237 | CLI 子命令路由 |
| `src/core/chain_spec.rs` | 25 | 冻结 plain chainspec 加载入口(`include_bytes!` + `from_json_bytes`),创世链状态包由启动流程复制到本地链数据库 |
| `src/core/benchmarking.rs` | 180 | Benchmark extrinsic 构建器 |
| `src/core/cli.rs` | 83 | CLI 参数定义 |
| `src/core/tls_cert.rs` | 107 | WSS 传输 TLS 证书校验 |
| `src/desktop/mod.rs` | 161 | 桌面端 Tauri 入口、插件与命令注册 |
| `src/home/process/mod.rs` | 405 | 首页节点生命周期管理，含打开 App 自动启动、首页手动启停、设置保存即重启、锁占用状态和退出清理 |
| `src/settings/desktop_update.rs` | 15 | 设置页点击更新前的节点停止准备命令 |
| `src/settings/node-mode/mod.rs` | 230 | 设置页全节点模式后端，当前只允许归档全节点，普通全节点由后端拒绝选择；旧 `communication` 本地值读取时清理回归档 |
| `src/settings/onchina_platform.rs` | 137 | 设置页链上中国平台后端，返回固定 HTTPS 入口并在用户确认后手动启动 / 停止 OnChina 子进程；启动中不显示失败详情，启动失败才返回错误状态 |
| `src/onchina_proc.rs` | 359 | 节点桌面端 OnChina 子进程管理，负责手动启动、运行状态检查、环境变量注入、退出清理，并在启动前清理旧 OnChina 孤儿进程 / 8964 监听 |
| `src/governance/runtime_upgrade/` | 5 files | 协议升级 node 后端，含 Tauri 命令、签名请求和 call_data 编码 |
| `frontend/governance/runtime-upgrade/` | 4 files | 协议升级 node 前端，含协议升级、开发升级和专用 API |
| `frontend/settings/node-mode/NodeModeSection.tsx` | 85 | 设置页全节点模式选择器，只展示归档/普通两种链数据模式，并将普通全节点置灰禁用 |
| `frontend/settings/OnChinaPlatformSection.tsx` | 69 | 设置页链上中国平台启动行，展示状态标签、固定 HTTPS 入口、启动 / 关闭按钮和二次确认弹窗 |
| `src/desktop/node_runner.rs` | 204 | 桌面端进程内节点启动器，含后台线程活跃标记和失败线程 join |
| `src/home/sync_guard.rs` | 421 | 本机同步守护，检测 raw P2P 已连但 block sync peer 表为空并进入降级状态 |
| `src/transaction/onchain_transaction/mod.rs` | 508 | 首页交易后端，包含钱包列表、矿工热钱包、余额查询、转账签名请求与提交 |
| `frontend/home/HomeNodeSection.tsx` | 236 | 首页左侧节点状态、状态文字右侧手动启停按钮、二次确认弹窗、锁占用状态提示、链状态、节点身份与发行/质押展示 |
| `frontend/transaction/onchain-transaction/TransactionPanel.tsx` | 105 | 首页右侧链上交易面板 |
| `src/main.rs` | 70 | CLI / 桌面入口分发,release 走 windows subsystem 不弹控制台 |
| `vendor/` | ~13,854 | sc-consensus-grandpa v0.40.0（GPL-3.0） |
| `libp2p-websocket/` | 6 files | 本地覆盖 crates.io `libp2p-websocket`，用于 WSS TLS 客户端配置扩展 |

目录收敛约定：
- 节点核心能力统一在 `src/core/`，避免根层散落 CLI、service、RPC、chain spec 等基础文件。
- 桌面壳入口统一在 `src/desktop/`，只负责 Tauri 启动、命令注册和进程内节点运行器。
- 挖矿页后端统一在 `src/mining/`，包含收益看板、网络概览、出块记录与 GPU 挖矿。
- 首页左侧节点状态能力统一在 `src/home/` 与 `frontend/home/`；首页右侧链上交易能力统一在 `src/transaction/onchain_transaction/` 与 `frontend/transaction/onchain-transaction/`。
- 跨功能复用能力统一在 `src/shared/`，例如 RPC 客户端、keystore、安全路径与 OnChina 服务地址配置。

前端目录收敛约定：
- `frontend/app/`：React/Tauri 前端入口，包含 `App.tsx`、`main.tsx` 与全局样式。
- `frontend/core/`：前端基础适配层，目前统一封装 Tauri `invoke` 与错误消息清理。
- `frontend/shared/`：跨功能复用能力，包含金额格式化、SS58 编解码与 `shared/qr/` 扫码协议组件。
- `frontend/home/`、`frontend/mining/`、`frontend/governance/`、`frontend/transaction/offchain-transaction/`、`frontend/settings/`、`frontend/other/`：与后端 `src/<功能名>` 保持同名边界。
- 各功能目录自持 `api.ts` 与 `types.ts`；根层不再保留全局 `api.ts`、`types.ts`、`format.ts`，避免新功能继续污染前端根层。
- 前端构建脚本使用 `tsc --noEmit && vite build`；`vite.config.ts` 由主 `tsconfig.json` 直接类型检查，不再通过 `tsconfig.node.json` 产出 `vite.config.js` / `vite.config.d.ts` 或 `*.tsbuildinfo`。

## 12. 安全风险（已知）

### 12.1 奖励账户 RPC 代签无鉴权
`reward_bindAccount` / `reward_rebindAccount` RPC 收到请求即用本地 `powr` 密钥签名发交易，无额外鉴权。
- **当前缓解**：桌面内嵌节点只面向本机端口使用，奖励账户 RPC 不转移余额。
- **禁止场景**：任何节点使用 `--rpc-external`、`--unsafe-rpc-external` 或公网反向代理暴露裸 RPC，都会把本机 RPC 和代签能力置于公网攻击面。
- **生产边界**：权威引导节点只允许回环 RPC；Worker 仅通过 Access 服务令牌和独立 Tunnel 调用 `state_getStorage`、`author_submitExtrinsic`，强制 HTTPS、超时、响应限长和禁止重定向，不提供通用 JSON-RPC 代理或自动重试广播。

矿工热钱包转账不复用上述裸 RPC 模式：`transaction_submitMinerTransfer` 必须携带进程内一次性令牌，并显式传入 `remark`；令牌只在设备开机密码校验通过后由 Tauri 命令签发，RPC 调用后立即消费。

### 12.2 空块三层共识防线
当前 `service.rs` 已要求：
- `pre_digest` 中放入矿工 `sr25519` 公钥
- `seal` 中附带 `(nonce, 签名)`
- `SimplePow::verify` 同时验证难度和矿工对 `pre_hash` 的签名

空块规则不能只依赖可被修改的节点代码。`pow-difficulty` runtime 在任何难度状态写入前检查
extrinsic count，只有 timestamp inherent 而没有用户交易时以共识无效结束执行；即使恶意出块节点
删除 NodeGuard，诚实节点重新执行正式 runtime WASM 时仍会拒绝该区块。`NodeGuard` 保留预执行
`KnownBad`，用于在外部空块进入 runtime 前低成本拦截。CPU / GPU 和 mining worker 同时按 ready
交易池门控 proposal；最佳块变化后 mining worker 还会跳过一轮，等待交易池在新链头上完成维护，
避免已打包交易短暂残留为 ready 时构造空候选块。没有本地 CPU/GPU 矿工的节点完全不构造
proposal。三层规则互不替代。

2026-07-12 动态难度真实验收进一步确认：单节点临时 fresh 链在无交易时保持 block#0，不构造
空块；由于节点防离线分叉门控要求 peer 非离线，第二个本地陪跑节点连入后，Alice 的真实
`System::remark` 交易被 PoW 打包进 block#1
`0xaaf286249a775bcac3bb107b7e7f4c15ccb3fb2eaebb8d0cf87e81464d7ae7fb`。
验收临时 chainspec、base-path、keystore、签名器和日志已删除。

## 13. 已知限制

1. 固定平均六分钟已进入 `pow-difficulty` 版本化参数和 NodeGuard 复算；参数值只能随 runtime 升级原子变更，`CurrentDifficulty` 只能由算法推进。
2. 节点层已有 `home::sync_guard` 判定单元测试；Substrate 服务级功能验证仍主要依赖集成测试。
3. `BuildSpec` 子命令已标注废弃（2026-04-01 后移除），使用 `ExportChainSpec` 替代。
4. `fee_blockFees` RPC 只累计 `FeePaid.fee`；tip 的唯一协议值是 0，不读取 FRAME `TransactionFeePaid.tip` 拼接第二套口径。
