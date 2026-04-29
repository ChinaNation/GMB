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
| `fee_blockFees(block_hash_hex)` | 读取指定区块的 FeePaid 事件累计手续费 |
| `sync_state_genSyncSpec` | 返回 lightSyncState（自定义实现，替代 BABE 依赖的标准 RPC） |

### RPC 交易签名
- 使用 `powr` keystore 密钥签名
- spec_version 从链上 WASM 运行时读取（非 native 编译时常量），防止升级后 BadProof
- TxExtension 与 benchmarking.rs 保持一致

## 4. Chain Spec

- 链名：`CHAIN_NAME`，链 ID：`CHAIN_ID`，SS58 前缀：`SS58_FORMAT`
- 44 个权威节点 bootnode（DNS 多地址）
- 创世配置：`genesis_config_presets::genesis_config()`
- 链类型：`ChainType::Live`

## 5. CLI 参数

| 参数 | 说明 |
|------|------|
| `--mining-threads COUNT` | 挖矿线程数（0 禁用，默认 CPU 并行度） |
| `--gpu-device INDEX` | GPU 设备编号 |
| `--no-gpu` | 强制禁用 GPU |
| 子命令 | key / export-chain-spec / check-block / export-blocks / import-blocks / purge-chain / revert / benchmark / chain-info |

## 6. 治理桌面页账户数据链路

- 地址真源：
  - `node/src/ui/governance/registry.rs` 直接读取 `runtime/primitives/china/china_cb.rs`、`runtime/primitives/china/china_ch.rs` 和 `NRC_ANQUAN_ADDRESS`
  - `治理 -> 国储会 / 省储会 / 省储行` 页面的 `主账户 / 费用账户 / 安全基金账户 / 永久质押账户` 不再允许 node 侧手抄第二份地址表
- 金额真源：
  - `node/src/ui/governance/institution.rs` 先取 `chain_getFinalizedHead`
  - 再用同一个 `block_hash` 调 `state_getStorage(System::Account)` 读取 `free` 余额
  - 同一详情页内所有账户金额必须来自同一个 finalized 快照
- 实时刷新：
  - `node/src/ui/governance/balance_watch.rs` 在详情页打开时启动 watcher
  - watcher 每秒检查一次 finalized hash，哈希变化后重新查询当前页面全部账户余额
  - 查询结果通过 Tauri 事件 `governance-balance-updated` 推给前端
- 前端约束：
  - `node/frontend/governance/InstitutionDetailPage.tsx` 只监听事件并覆盖现有 state
  - 不改 UI 布局、不改卡片顺序、不改现有中文命名

## 7. 文件索引

| 文件 | 行数 | 说明 |
|------|------|------|
| `src/service.rs` | 695 | 服务工厂、PoW 算法、CPU 挖矿、GRANDPA 角色选择 |
| `src/rpc.rs` | 382 | RPC 模块、钱包绑定签名、哈希率查询、轻节点同步 |
| `src/gpu_miner.rs` | 322 | OpenCL 初始化、GPU kernel 调度、哈希率统计 |
| `src/command.rs` | 242 | CLI 子命令路由 |
| `src/chain_spec.rs` | 95 | Chain spec、44 个 bootnode、token 属性 |
| `src/benchmarking.rs` | 180 | Benchmark extrinsic 构建器 |
| `src/cli.rs` | 64 | CLI 参数定义 |
| `src/main.rs` | 15 | 入口 |
| `vendor/` | ~13,854 | sc-consensus-grandpa v0.40.0（GPL-3.0） |

## 8. 安全风险（已知）

### 7.1 RPC 代签无鉴权
`reward_bindWallet` / `reward_rebindWallet` RPC 收到请求即用本地 `powr` 密钥签名发交易，无额外鉴权。
- **当前缓解**：默认 `--rpc-methods Safe` 限制为本地调用。
- **风险场景**：nodeui 启动时使用 `--unsafe-rpc-external --rpc-methods Unsafe --rpc-cors all`，会将代签 RPC 暴露到外部网络。
- **建议**：生产部署必须限制 RPC 绑定地址或加鉴权中间件；或改为 nodeui 本地签名后提交。

### 7.2 空块策略仍与 runtime panic 耦合
当前 `service.rs` 已要求：
- `pre_digest` 中放入矿工 `sr25519` 公钥
- `seal` 中附带 `(nonce, 签名)`
- `SimplePow::verify` 同时验证难度和矿工对 `pre_hash` 的签名

但 `pow-difficulty` 仍在 `on_finalize` 中对空块执行 `assert!(extrinsic_count > 1)`。
- **影响**：节点层虽然已经在交易池为空时停止挖矿，但 runtime 仍把“运营策略兜底”实现成 panic 型链规则；一旦有空块漏过节点侧门控，可能直接触发拒块甚至停链风险。
- **当前缓解**：CPU / GPU 矿工都在交易池为空时跳过挖矿，代码中也明确写了“避免触发 runtime 的空块 assert panic”。
- **建议**：后续应把空块限制从 runtime panic 改成非 panic 的制度约束或完全下沉到节点策略，避免状态机层面承受运营错误。

## 9. 已知限制

1. `target_block_time_ms` 仅启动时读取一次，链上迁移修改后需重启节点生效。
2. 节点层无单元测试（Substrate 节点模板通病，功能验证依赖集成测试）。
3. `BuildSpec` 子命令已标注废弃（2026-04-01 后移除），使用 `ExportChainSpec` 替代。
4. `fee_blockFees` RPC 已修复为同时累加 `FeePaid.fee`（base_fee）和 `TransactionFeePaid.tip`。
