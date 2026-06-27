# NODE Technical Notes

## 0. 模块定位

`node` 是 CitizenChain 全节点可执行程序，实现双共识架构（PoW + GRANDPA）、自定义 RPC 接口和挖矿子系统。

代码位置：`/Users/rhett/GMB/citizenchain/node/`

## 1. 双共识架构

### 1.1 PoW 共识
- 算法：`SimplePow` — `blake2_256(pre_hash ++ nonce_le_bytes)` 与目标值比较
- 难度：从链上 `PowDifficultyApi::current_pow_difficulty` Runtime API 读取
- 密钥类型：`powr`（sr25519），首次启动自动生成 BIP39 并写入 keystore 磁盘
- 出块间隔：从 `genesis-pallet::target_block_time_ms()` 读取（启动时获取一次）

### 1.2 GRANDPA 最终性
- 权威节点（本地有 GRANDPA ed25519 密钥）：运行 `grandpa-voter`
- 普通节点：运行 `grandpa-observer`（只接收最终性结果不投票）
- 所有节点统一注册 GRANDPA 网络协议，保证协议栈一致
- Justification 周期：64 块
- vendor 目录：`sc-consensus-grandpa` v0.40.0（独立 GPL-3.0 许可）

### 1.3 libp2p WebSocket 本地覆盖
- 本地目录：`citizenchain/node/libp2p-websocket/`
- 覆盖方式：`citizenchain/Cargo.toml` 通过 `[patch.crates-io]` 将 crates.io 的 `libp2p-websocket` 指向该本地目录。
- 包名约束：本地 crate 的 `name` 必须继续保持 `libp2p-websocket`，否则 Cargo patch 无法覆盖上游同名包。
- 当前改动点：公开 `tls::Config` 的 `client` 字段，支持节点在 WSS transport 中注入自定义 TLS 客户端。TLS 层只负责传输加密，P2P 身份认证仍由 Noise 协议通过 peer ID 保证。

## 2. 挖矿子系统

### 2.1 CPU 挖矿
- 线程数：`--mining-threads`（默认 CPU 可用并行度，0 禁用）
- nonce 空间：低半区（bit63=0），基于 pre_hash 前 8 字节的随机基址 + 线程号错位
- 哈希率统计：thread 0 每 100,000 次哈希采样，乘以线程数得总哈希率
- 提交门控：AtomicU64 无锁实现，防止出块频率超过 target_block_time

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

| 方法 | 说明 |
|------|------|
| `mining_cpuHashrate` | CPU 全线程合计哈希率（hashes/sec） |
| `mining_gpuHashrate` | GPU 哈希率（仅 gpu-mining feature） |
| `reward_bindWallet(ss58)` | 节点端签名提交 bind_reward_wallet 交易 |
| `reward_rebindWallet(ss58)` | 节点端签名提交 rebind_reward_wallet 交易 |
| `transaction_submitMinerTransfer(ss58, amount_fen, token)` | 节点端使用 `powr` 密钥提交矿工热钱包转账，要求进程内一次性令牌 |
| `fee_blockFees(block_hash_hex)` | 读取指定区块的 FeePaid 事件累计手续费 |
| `sync_state_genSyncSpec` | 返回 lightSyncState（自定义实现，替代 BABE 依赖的标准 RPC） |

### RPC 交易签名
- 使用 `powr` keystore 密钥签名
- spec_version 从链上 WASM 运行时读取（非 native 编译时常量），防止升级后 BadProof
- TxExtension 与 benchmarking.rs 保持一致
- 矿工热钱包转账 RPC 额外要求一次性令牌；令牌由桌面 Tauri 命令在设备密码校验通过后生成并由 RPC 消费

## 4. Chain Spec（冻结铁律）

主网创世后,chainspec 永久冻结(memory/feedback_chainspec_frozen.md)。

- 冻结资产：[citizenchain/node/chainspecs/citizenchain.raw.json](../../../../citizenchain/node/chainspecs/citizenchain.raw.json),raw 格式 1.3 MB,含 44 个权威节点 bootnode、token 属性、协议 ID、扁平化 genesis state(含 `:code` 下的 runtime WASM 字节)
- 加载方式：[chain_spec.rs](../../../../citizenchain/node/src/core/chain_spec.rs) 用 `include_bytes!` 把 JSON 字节烤进二进制,启动时 `ChainSpec::from_json_bytes` 反序列化。**不再 `with_genesis_config_patch` 现编创世**
- 全网一致性保证：任何平台、任何 commit 编出来的 binary,内嵌的都是同一份 JSON 字节 → genesis_hash 全网恒等 → 所有节点 P2P handshake 必通过

冻结流程(只做一次,**永不重做**):

1. 主网在线权威节点上跑 `citizenchain export-chain-spec --chain citizenchain --raw > /tmp/citizenchain.raw.json`
2. scp 回 `citizenchain/node/chainspecs/citizenchain.raw.json`
3. git commit 入库

后续 runtime 升级一律走链上 `setCode`(governance/runtime-upgrade),**绝不**重新 `export-chain-spec` 覆盖这份 JSON。

预上线重新创世脚本例外：

- [clean-run.sh](../../../../citizenchain/scripts/clean-run.sh) 会下载最新成功的 `citizenchain-wasm` CI artifact。
- 需要真正替换仓库 SSOT 时,唯一入口是 [bake-chainspec.sh](../../../../citizenchain/scripts/bake-chainspec.sh)。
- `bake-chainspec.sh` 用 `citizenchain-fresh` 入口生成新的 raw chainspec,并保留当前 SSOT 中的 44 个权威节点 bootNodes。
- fresh raw chainspec 的 genesis `:code` 必须与下载的 CI WASM 字节一致。
- 脚本同时写回 `citizenchain/node/chainspecs/citizenchain.raw.json` 与 `citizenapp/assets/chainspec.json`,保证全节点和轻节点使用同一创世。

2026-06-19 预上线重新创世收口:

- runtime 本地 release WASM blake2:`f213cdc476fb0d1e723421a5bd1f5afafc792b5180852d2266346b967386e680`
- raw chainspec sha256:`cdf74fd89148ab8d681b020c65f59ff8f93e238a1404da44a7b47fae8bb4757a`
- `citizenchain/node/chainspecs/citizenchain.raw.json` 与 `citizenapp/assets/chainspec.json` 完全一致。
- bootNodes 保持 44 个;伊犁省权威节点域名为 `prcyls.crcfrcn.com`。

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
  - `治理 -> 国储会 / 省储会 / 省储行` 页面的 `主账户 / 费用账户 / 安全基金账户 / 永久质押账户` 不再允许 node 侧手抄第二份地址表
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
  - 开发升级命令会校验签名公钥属于本机已激活国储会管理员，避免绕过前端直接调用 Tauri 命令。
- 前端实现：
  - `node/frontend/governance/runtime-upgrade/ProtocolUpgradeProposalPage.tsx`：国储会详情页“协议升级”，提交运行期协议升级提案，进入联合投票。
  - `node/frontend/governance/runtime-upgrade/DeveloperUpgradePage.tsx`：国储会详情页“开发升级”，只使用当前国储会已激活管理员发起开发期直升。
  - `node/frontend/governance/runtime-upgrade/api.ts`：协议升级专用 Tauri API；`governance/api.ts` 不再承载协议升级创建/提交接口。
- 入口约束：
  - 国储会详情页使用“协议升级”入口。
  - “开发升级”是独立按钮，放在“协议升级”后，不与协议升级合并。
  - 设置页不再保留任何开发升级入口或 `settings/developer-upgrade` 代码。
- 当前边界：
  - 第 1 步只调整 node 前后端入口、目录收口和 node 侧开发升级管理员校验。
  - node 端协议升级业务调用只提交 `reason + code`，不获取人口快照、不透传联合签名、不保存投票状态。

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

2026-05-23 起，桌面设置页新增“全节点模式”入口；2026-06-15 起，通信节点能力从全节点模式中拆出，固定为独立 IM 功能开关。

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
  - 全节点模式只描述链数据保存方式，不承载 IM 通信开关。
  - 后续普通全节点真正实现时，少数据模式切换到多数据模式应从网络补充同步数据；多数据模式切换到少数据模式应删除不再需要的本地数据。
  - 在普通全节点底层能力完成前，不得让设置页暗示当前已执行剪裁或删除数据。

### 9.1 通信节点 IM 边界

2026-06-14 起，通信节点 IM 能力统一规划到 `citizenchain/node/src/im/`，并按 ADR-020 固定为只服务本机用户的通信节点；2026-06-15 起，桌面设置页通过独立“通信节点功能”开关启用，不再作为全节点模式选项。

- 模块定位：通信节点用于让公民用户全天候实时在线，承接自己的端到端加密 IM 收件箱、KeyPackage 池、密文消息投递、设备绑定和通信端点管理。
- 私人边界：一台通信节点可以服务同一用户多台手机和多个钱包聊天号；不互为中继，不做公共 Relay / DHT / rendezvous，不替第三方存消息。
- 设置边界：归档全节点和普通全节点都可以开启通信节点功能；开启/关闭通信节点功能不改变链数据库、同步策略或全节点数据模式。
- 网络能力：复用当前节点已经固定使用的 `sc-network/libp2p` 后端，注册 IM 专用 request-response 协议 `/gmb/im/1`，不默认另起 swarm。
- 可达性：节点端点支持 IPv4、IPv6、用户自有 `dns4` / `dnsaddr`；不可达时消息留在发送队列重试，不退化成借别人节点中继。
- 存储边界：通信节点只保存密文 `ImEnvelope`、附件密文分片、KeyPackage 池和必要索引，不读取明文；聊天内容、通信端点、设备公钥、PeerId、更新时间和撤销状态都只进入 IM 专属存储。
- 账户边界：公民钱包地址是用户可见聊天号；钱包私钥只用于设备绑定证明，不作为 IM 消息加密密钥。
- 业务边界：`src/im/` 不处理治理、投票、实名信息或交易业务；联系人详情里的“转账”继续归公民既有交易页面处理。
- 用户入口：公民端从“我的 -> 用户资料”设置通信账户，从“我的通讯录 -> 联系人详情 -> 消息”进入聊天；“信息”Tab 只展示会话列表。
- 配对入口：桌面设置页 `frontend/settings/communication-node/CommunicationNodeSection.tsx` 读取 `get_communication_node`，展示 `im_node_pairing` 固定二维码；公民端在“我的 -> 设置 -> 设置通信节点”扫码保存或更换自己的电脑通信节点。
- 当前实现：`src/im/` 已提供通信节点策略结构、端点校验、钱包账户设备绑定、密文信封、多钱包账号 mailbox、多钱包账号 KeyPackage 池、`/gmb/im/1` request-response 配置、incoming handler、显式端点直连投递和 KeyPackage 拉取/消费 helper；`src/settings/communication-node/mod.rs` 已提供独立开关、IPv4/IPv6 端点生成和不含 RPC URL / 有效期的 QR_V1 配对二维码生成。
- 当前命令：
  - `get_communication_node`：读取通信节点功能状态、PeerId、multiaddr 和配对二维码 payload。
  - `set_communication_node_enabled`：独立开启/关闭通信节点功能，不改变归档/普通全节点模式，不注册手机 RPC。
  - `get_im_private_node_policy`：查询通信节点边界。
  - `get_im_direct_network_capability`：查询 `/gmb/im/1` 直连网络 Spike 能力。
  - `validate_im_node_endpoint`：校验 IPv4 / IPv6 / dns4 / dnsaddr multiaddr 与 PeerId 是否匹配。
  - `validate_im_direct_delivery_request`：校验直连投递请求是否满足显式端点和目标钱包 mailbox 边界。
  - `submit_im_direct_encrypted_envelope`：通过已启动的 sc-network 向对方通信节点直连投递密文信封。
  - `fetch_im_direct_key_packages`：通过已启动的 sc-network 从对方通信节点直连拉取 KeyPackage。
  - `consume_im_direct_key_package`：通过已启动的 sc-network 向对方通信节点声明消费一次性 KeyPackage。
  - `register_im_owner_device`：登记钱包聊天账户、IM 设备、公钥、节点 PeerId、端点和钱包签名。
  - `submit_im_encrypted_envelope`：提交投递给目标钱包地址的密文信封，拒绝未授权 mailbox。
  - `fetch_im_pending_envelopes`：已授权设备拉取本钱包地址待收密文。
  - `ack_im_envelope`：已授权设备确认并移除密文信封。
  - `publish_im_key_package`：已授权手机向自己的通信节点发布 OpenMLS KeyPackage。
  - `fetch_im_key_packages`：查询本机钱包地址 KeyPackage 池，供调试和本机验收使用。
  - `consume_im_key_package`：消费本机钱包地址 KeyPackage，供调试和本机验收使用。
- 正式手机连接：禁止使用节点 RPC。公民手机会离开家庭局域网，后续必须通过专用 IM P2P 通道连接自己的通信节点。
- IM RPC 边界：通信节点不提供正式或调试节点 RPC 入口；手机连接自己的通信节点必须走后续专用 IM P2P 通道。
- 持久化边界：mailbox 快照落在 `base-path/im/mailbox.json`，包含多钱包账号设备绑定、pending envelope 和 ack tombstone；KeyPackage 池快照落在 `base-path/im/keypackages.json`，包含多钱包账号 KeyPackage、TTL 和消费时间；节点启动时加载，登记/投递/ack/发布/消费后写回。
- sc-network Spike 结论：当前节点能注册 request-response 协议并处理 incoming；outbound 直连会先把显式 `PeerId + multiaddr` 写入地址簿，再用 `NetworkService::request(..., TryConnect)` 发起请求；该路径不使用 DHT、rendezvous 或 Relay。
- 双节点运行态验收：`citizenchain/scripts/im-two-node-smoke.sh` 已用两个临时 headless 节点验证 KeyPackage 发布/重启恢复/直连拉取/消费、A→B 密文投递、B 重启后 pending 不丢、B 授权设备拉取、ack、ack 后重启不重复、第三方 mailbox 拒绝和 ack 后重复投递不入队。
- 未完成能力：设备撤销、正式容量配置、TTL 后台清理、真机双手机联调和近场原生能力仍需后续任务落地。

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
| `src/core/service.rs` | 830 | 服务工厂、PoW 算法、CPU 挖矿、GRANDPA 角色选择 |
| `src/core/rpc.rs` | 419 | 节点核心 RPC、钱包绑定签名、哈希率查询、轻节点同步 |
| `src/mining/gpu_miner.rs` | 392 | OpenCL 初始化、GPU kernel 调度、哈希率统计 |
| `src/core/command.rs` | 237 | CLI 子命令路由 |
| `src/core/chain_spec.rs` | 25 | 冻结 chainspec 加载入口(`include_bytes!` + `from_json_bytes`),bootnode/token 属性/genesis state 全在 `chainspecs/citizenchain.raw.json` |
| `src/core/benchmarking.rs` | 180 | Benchmark extrinsic 构建器 |
| `src/core/cli.rs` | 83 | CLI 参数定义 |
| `src/core/tls_cert.rs` | 107 | WSS 传输 TLS 证书校验 |
| `src/desktop/mod.rs` | 161 | 桌面端 Tauri 入口、插件与命令注册 |
| `src/home/process/mod.rs` | 405 | 首页节点生命周期管理，含打开 App 自动启动、首页手动启停、设置保存即重启、锁占用状态和退出清理 |
| `src/settings/desktop_update.rs` | 15 | 设置页点击更新前的节点停止准备命令 |
| `src/settings/node-mode/mod.rs` | 230 | 设置页全节点模式后端，当前只允许归档全节点，普通全节点由后端拒绝选择；旧 `communication` 本地值读取时清理回归档 |
| `src/settings/communication-node/mod.rs` | 265 | 设置页通信节点功能后端，独立读写 `<app_data>/communication-node.json`，生成 IPv4/IPv6 固定配对二维码，不返回 RPC URL |
| `src/im/` | 10 files | 通信节点 IM 边界模块，当前提供策略、端点、绑定、密文信封、持久化 mailbox、持久化 KeyPackage 池和 `/gmb/im/1` 网络接入；不提供节点 RPC 入口 |
| `src/governance/runtime_upgrade/` | 5 files | 协议升级 node 后端，含 Tauri 命令、签名请求和 call_data 编码 |
| `frontend/governance/runtime-upgrade/` | 4 files | 协议升级 node 前端，含协议升级、开发升级和专用 API |
| `frontend/settings/node-mode/NodeModeSection.tsx` | 85 | 设置页全节点模式选择器，只展示归档/普通两种链数据模式，并将普通全节点置灰禁用 |
| `frontend/settings/communication-node/CommunicationNodeSection.tsx` | 126 | 设置页通信节点功能面板，独立开关、状态标签、PeerId/multiaddr 摘要和公民扫码配对二维码 |
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
- 挖矿页后端统一在 `src/mining/`，包含收益看板、资源监控、网络概览、出块记录与 GPU 挖矿。
- 首页左侧节点状态能力统一在 `src/home/` 与 `frontend/home/`；首页右侧链上交易能力统一在 `src/transaction/onchain_transaction/` 与 `frontend/transaction/onchain-transaction/`。
- 跨功能复用能力统一在 `src/shared/`，例如 RPC 客户端、keystore、安全路径与 CID 服务地址配置。

前端目录收敛约定：
- `frontend/app/`：React/Tauri 前端入口，包含 `App.tsx`、`main.tsx` 与全局样式。
- `frontend/core/`：前端基础适配层，目前统一封装 Tauri `invoke` 与错误消息清理。
- `frontend/shared/`：跨功能复用能力，包含金额格式化、SS58 编解码与 `shared/qr/` 扫码协议组件。
- `frontend/home/`、`frontend/mining/`、`frontend/governance/`、`frontend/transaction/offchain-transaction/`、`frontend/settings/`、`frontend/other/`：与后端 `src/<功能名>` 保持同名边界。
- 各功能目录自持 `api.ts` 与 `types.ts`；根层不再保留全局 `api.ts`、`types.ts`、`format.ts`，避免新功能继续污染前端根层。
- 前端构建脚本使用 `tsc --noEmit && vite build`；`vite.config.ts` 由主 `tsconfig.json` 直接类型检查，不再通过 `tsconfig.node.json` 产出 `vite.config.js` / `vite.config.d.ts` 或 `*.tsbuildinfo`。

## 12. 安全风险（已知）

### 12.1 奖励钱包 RPC 代签无鉴权
`reward_bindWallet` / `reward_rebindWallet` RPC 收到请求即用本地 `powr` 密钥签名发交易，无额外鉴权。
- **当前缓解**：桌面内嵌节点只面向本机端口使用，奖励钱包 RPC 不转移余额。
- **风险场景**：节点桌面端启动时使用 `--unsafe-rpc-external --rpc-methods Unsafe --rpc-cors all`，会将代签 RPC 暴露到外部网络。
- **建议**：生产部署必须限制 RPC 绑定地址或加鉴权中间件；或改为节点桌面端本地签名后提交。

矿工热钱包转账不复用上述裸 RPC 模式：`transaction_submitMinerTransfer` 必须携带进程内一次性令牌，令牌只在设备开机密码校验通过后由 Tauri 命令签发，RPC 调用后立即消费。

### 12.2 空块策略仍与 runtime panic 耦合
当前 `service.rs` 已要求：
- `pre_digest` 中放入矿工 `sr25519` 公钥
- `seal` 中附带 `(nonce, 签名)`
- `SimplePow::verify` 同时验证难度和矿工对 `pre_hash` 的签名

但 `pow-difficulty` 仍在 `on_finalize` 中对空块执行 `assert!(extrinsic_count > 1)`。
- **影响**：节点层虽然已经在交易池为空时停止挖矿，但 runtime 仍把“运营策略兜底”实现成 panic 型链规则；一旦有空块漏过节点侧门控，可能直接触发拒块甚至停链风险。
- **当前缓解**：CPU / GPU 矿工都在交易池为空时跳过挖矿，代码中也明确写了“避免触发 runtime 的空块 assert panic”。
- **建议**：后续应把空块限制从 runtime panic 改成非 panic 的制度约束或完全下沉到节点策略，避免状态机层面承受运营错误。

## 13. 已知限制

1. `target_block_time_ms` 仅启动时读取一次，链上迁移修改后需重启节点生效。
2. 节点层已有 `home::sync_guard` 判定单元测试；Substrate 服务级功能验证仍主要依赖集成测试。
3. `BuildSpec` 子命令已标注废弃（2026-04-01 后移除），使用 `ExportChainSpec` 替代。
4. `fee_blockFees` RPC 已修复为同时累加 `FeePaid.fee`（base_fee）和 `TransactionFeePaid.tip`。
