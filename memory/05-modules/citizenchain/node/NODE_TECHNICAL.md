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

本机重新创世脚本例外：

- [clean-run.sh](../../../../citizenchain/scripts/clean-run.sh) 会下载最新成功的 `citizenchain-wasm` CI artifact。
- 脚本用 `citizenchain-fresh` 入口生成 `citizenchain/target/fresh-genesis/citizenchain.fresh.raw.json`。
- fresh raw chainspec 的 `bootNodes` 必须为空，避免清链后重新接回旧网络。
- fresh raw chainspec 的 genesis `:code` 必须与下载的 CI WASM 字节一致。
- 脚本通过 `CITIZENCHAIN_CHAIN_SPEC` 让桌面内嵌节点使用该 fresh raw chainspec 启动；仓库冻结主网 JSON 不被覆盖。

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
  - `node/src/governance/registry.rs` 直接读取 `runtime/primitives/china/china_cb.rs`、`runtime/primitives/china/china_ch.rs` 和 `NRC_ANQUAN_ADDRESS`
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

## 7. Runtime 升级 node 端边界

2026-05-09 起，node 端 runtime 升级入口按“协议升级 / 开发升级”拆分，并统一收口在治理模块的 runtime-upgrade 目录。

- 后端实现：
  - `node/src/governance/runtime_upgrade/commands.rs`：Tauri 命令入口，保留 `build_propose_upgrade_request`、`submit_propose_upgrade`、`build_developer_upgrade_request`、`submit_developer_upgrade` 命令名。
  - `node/src/governance/runtime_upgrade/call_data.rs`：RuntimeUpgrade pallet call_data 编码，只承载 `propose_runtime_upgrade` 与 `developer_direct_upgrade`。
  - `node/src/governance/runtime_upgrade/signing.rs`：Runtime WASM 大 payload 的 QR 签名请求构建，通用签名校验仍复用 `node/src/governance/signing.rs`。
  - 开发升级命令会校验签名公钥属于本机已激活国储会管理员，避免绕过前端直接调用 Tauri 命令。
- 前端实现：
  - `node/frontend/governance/runtime-upgrade/ProtocolUpgradeProposalPage.tsx`：国储会详情页“协议升级”，提交运行期 Runtime 升级提案，进入联合投票。
  - `node/frontend/governance/runtime-upgrade/DeveloperUpgradePage.tsx`：国储会详情页“开发升级”，只使用当前国储会已激活管理员发起开发期直升。
  - `node/frontend/governance/runtime-upgrade/api.ts`：Runtime 升级专用 Tauri API；`governance/api.ts` 不再承载 runtime 升级创建/提交接口。
- 入口约束：
  - 国储会详情页原“状态升级”按钮已改为“协议升级”。
  - “开发升级”是独立按钮，放在“协议升级”后，不与协议升级合并。
  - 设置页不再保留任何开发升级入口或 `settings/developer-upgrade` 代码。
- 当前边界：
  - 第 1 步只调整 node 前后端入口、目录收口和 node 侧开发升级管理员校验。
  - Runtime pallet 的开发期权限收窄和运行期参数削减放在第 2 步处理；node 端当前仍按现有 runtime call 参数兼容。

## 8. 文件索引

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
| `src/desktop/mod.rs` | 120 | 桌面端 Tauri 入口与命令注册 |
| `src/governance/runtime_upgrade/` | 5 files | Runtime 升级 node 后端，含 Tauri 命令、签名请求和 call_data 编码 |
| `frontend/governance/runtime-upgrade/` | 4 files | Runtime 升级 node 前端，含协议升级、开发升级和专用 API |
| `src/desktop/node_runner.rs` | 164 | 桌面端进程内节点启动器 |
| `src/home/transaction/mod.rs` | 339 | 首页交易、冷钱包、本地钱包与转账提交 |
| `src/main.rs` | 70 | CLI / 桌面入口分发,release 走 windows subsystem 不弹控制台 |
| `vendor/` | ~13,854 | sc-consensus-grandpa v0.40.0（GPL-3.0） |
| `libp2p-websocket/` | 6 files | 本地覆盖 crates.io `libp2p-websocket`，用于 WSS TLS 客户端配置扩展 |

目录收敛约定：
- 节点核心能力统一在 `src/core/`，避免根层散落 CLI、service、RPC、chain spec 等基础文件。
- 桌面壳入口统一在 `src/desktop/`，只负责 Tauri 启动、命令注册和进程内节点运行器。
- 挖矿页后端统一在 `src/mining/`，包含收益看板、资源监控、网络概览、出块记录与 GPU 挖矿。
- 首页交易能力统一在 `src/home/transaction/`，与前端 `node/frontend/home/transaction/` 保持一致。
- 跨功能复用能力统一在 `src/shared/`，例如 RPC 客户端、keystore、安全路径与 SFID 服务地址配置。

前端目录收敛约定：
- `frontend/app/`：React/Tauri 前端入口，包含 `App.tsx`、`main.tsx` 与全局样式。
- `frontend/core/`：前端基础适配层，目前统一封装 Tauri `invoke` 与错误消息清理。
- `frontend/shared/`：跨功能复用能力，包含金额格式化、SS58 编解码与 `shared/qr/` 扫码协议组件。
- `frontend/home/`、`frontend/mining/`、`frontend/governance/`、`frontend/offchain/`、`frontend/settings/`、`frontend/other/`：与后端 `src/<功能名>` 保持同名边界。
- 各功能目录自持 `api.ts` 与 `types.ts`；根层不再保留全局 `api.ts`、`types.ts`、`format.ts`，避免新功能继续污染前端根层。
- 前端构建脚本使用 `tsc --noEmit && vite build`；`vite.config.ts` 由主 `tsconfig.json` 直接类型检查，不再通过 `tsconfig.node.json` 产出 `vite.config.js` / `vite.config.d.ts` 或 `*.tsbuildinfo`。

## 9. 安全风险（已知）

### 9.1 奖励钱包 RPC 代签无鉴权
`reward_bindWallet` / `reward_rebindWallet` RPC 收到请求即用本地 `powr` 密钥签名发交易，无额外鉴权。
- **当前缓解**：桌面内嵌节点只面向本机端口使用，奖励钱包 RPC 不转移余额。
- **风险场景**：节点桌面端启动时使用 `--unsafe-rpc-external --rpc-methods Unsafe --rpc-cors all`，会将代签 RPC 暴露到外部网络。
- **建议**：生产部署必须限制 RPC 绑定地址或加鉴权中间件；或改为节点桌面端本地签名后提交。

矿工热钱包转账不复用上述裸 RPC 模式：`transaction_submitMinerTransfer` 必须携带进程内一次性令牌，令牌只在设备开机密码校验通过后由 Tauri 命令签发，RPC 调用后立即消费。

### 9.2 空块策略仍与 runtime panic 耦合
当前 `service.rs` 已要求：
- `pre_digest` 中放入矿工 `sr25519` 公钥
- `seal` 中附带 `(nonce, 签名)`
- `SimplePow::verify` 同时验证难度和矿工对 `pre_hash` 的签名

但 `pow-difficulty` 仍在 `on_finalize` 中对空块执行 `assert!(extrinsic_count > 1)`。
- **影响**：节点层虽然已经在交易池为空时停止挖矿，但 runtime 仍把“运营策略兜底”实现成 panic 型链规则；一旦有空块漏过节点侧门控，可能直接触发拒块甚至停链风险。
- **当前缓解**：CPU / GPU 矿工都在交易池为空时跳过挖矿，代码中也明确写了“避免触发 runtime 的空块 assert panic”。
- **建议**：后续应把空块限制从 runtime panic 改成非 panic 的制度约束或完全下沉到节点策略，避免状态机层面承受运营错误。

## 10. 已知限制

1. `target_block_time_ms` 仅启动时读取一次，链上迁移修改后需重启节点生效。
2. 节点层无单元测试（Substrate 节点模板通病，功能验证依赖集成测试）。
3. `BuildSpec` 子命令已标注废弃（2026-04-01 后移除），使用 `ExportChainSpec` 替代。
4. `fee_blockFees` RPC 已修复为同时累加 `FeePaid.fee`（base_fee）和 `TransactionFeePaid.tip`。
