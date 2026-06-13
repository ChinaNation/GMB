# RPC 模块技术文档（当前实现态）

> 说明：本文描述的是当前代码实现态。
> 当前实现已切为“PoW 专用轻节点 + 无 HTTP 回退”，以 [ADR-004](../../04-decisions/ADR-004-pow-light-client-without-http-fallback.md) 与 [POW 轻节点长期落地方案](./POW_LIGHT_CLIENT_ROADMAP.md) 为准。

## 1. 模块定位

`lib/rpc/` 是公民 与区块链节点通信的唯一收口模块。

职责：

- 默认通过 `smoldot` 轻节点接入 citizenchain P2P 网络
- 提供底层 JSON-RPC 调用能力
- 提供轻节点状态快照等逐步收口中的 typed capability
- 链上状态查询（余额、nonce、metadata 等）
- 链上交易构造与提交（转账、未来的投票/提案）
- 钱包交易流水增量监听：从 newHeads/finalizedHeads 读取 `System.Events`，只记录本机钱包开始跟踪后的余额变化

约束：所有链上通信必须通过本模块，业务模块不直接建立 RPC 连接。

补充说明：

- 当前 `smoldot` Dart 绑定已从 pub.dev 依赖切换为仓库内本地 fork：`wuminapp/third_party/smoldot-dart`
- 当前 `smoldot-light` Rust 内核通过 Git submodule 位于：`wuminapp/third_party/smoldot-pow`
- 这两层收编的目的，是为后续 PoW 专用 typed capability 改造建立可控演进入口
- Android 真机 ABI 只支持 `arm64-v8a` 与 `armeabi-v7a`。`scripts/build-smoldot-native.sh android` 必须同时构建 `aarch64-linux-android` 与 `armv7-linux-androideabi`，并分别写入 `android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so` 与 `android/app/src/main/jniLibs/armeabi-v7a/libsmoldot.so`；APK 构建入口必须显式传入 `--target-platform android-arm,android-arm64`，避免生成未适配 smoldot 的 x86 / x86_64 包内容。

## 2. 目录结构

```text
lib/rpc/
├── chain_tx_monitor.dart← 业务：钱包区块事件监听 + 本地流水写入
├── chain_rpc.dart       ← 底层：节点连接管理 + JSON-RPC 方法
├── onchain.dart         ← 业务：extrinsic 构造 + 转账 + 交易确认
├── rpc.dart             ← barrel export
└── RPC_TECHNICAL.md
```

## 3. 通信架构

```text
公民  --smoldot-->  bootNodes / P2P 网络
```

- 默认协议：`smoldot` 轻客户端 + Substrate JSON-RPC

## 4. chainspec 与轻节点模式

默认模式下，App 从 `assets/chainspec.json` 加载链规格，并使用其中的 `bootNodes` 加入网络。

当前要求：

- `chainspec.json` 必须与目标链的 genesis / properties / bootNodes 一致
- 如果打进 App 的 chainspec 错了，轻节点即使“连上了”，也可能连到错误链或错误引导节点
- `bootNodes` 的来源应以 `citizenchain/node/src/chain_spec.rs` 为准

## 5. 连接与同步策略

1. App 先完成 `runApp()` 和首帧渲染,再后台初始化 `SmoldotClientManager`
2. 轻节点读取 `SharedPreferences.smoldot_db_cache`，优先通过 `AddChainConfig.databaseContent` 恢复上次 finalized database
3. 如果缓存失效或与当前链状态不兼容，会自动清掉缓存并回退到无缓存重连，避免坏缓存永久卡死启动
4. 轻节点加入 `chainspec.json` 指定的 citizenchain 网络后，立即在后台预热同步
5. `ChainRpc` 在发起余额、nonce、metadata、storage、extrinsic 等链上请求前，先等待轻节点完成同步
6. 当轻节点未初始化、同步失败或链路降级时，typed capability 必须抛出真实错误，不能返回 `null` / `[]` / `{}` 伪装成“链上没有数据”

补充说明：

- 钱包余额不更新的首要风险点，不是 UI，而是“轻节点已初始化但尚未同步完成”时过早查询链上状态
- Android 系统弹出“公民没有响应/关闭应用/等待”属于 ANR,首要排查点是启动阶段是否在
  Flutter 首帧前等待 smoldot 初始化或同步;当前实现禁止在 `runApp()` 前 await 轻节点初始化。
- `smoldot` 返回 JSON-RPC error 时必须抛出，不能把错误吞成 `null`，否则上层会把真实故障误判为余额为 0、没有提案或机构不存在
- 当前代码已新增 `SmoldotClientManager.getStatusSnapshot()`，作为结构化轻节点状态接口；其底层已改为 Rust 原生 capability，不再由 Dart 层拼装 `system_health`
- `ChainProgressBanner` 只展示轻节点状态快照（peer / best / finalized / syncing），文案必须使用“轻节点状态/轻节点已就绪”。该状态不等同于某个业务页面的本地 Isar 写库成功，也不等同于所有链上 storage 查询已经完成。
- 连接诊断必须以有效 peer、best/finalized 状态是否可读或推进为准；未部署 bootNodes 的连接失败日志不是故障根因，不得把它解释成 wuminapp 网络不可用。
- 本地开发期 `30334` bootnode 只是可选调试兜底，不是 wuminapp 真机连接区块链网络的必要条件；没有本地 `30334` 也不应判定为连接异常。
- Flutter widget test 环境不具备真实 smoldot 轻节点链路，`ChainProgressBanner` 在测试中只渲染静态提示条，禁止读取链状态和创建轮询定时器，避免 `pumpAndSettle` 被后台链路轮询卡住。
- 当前代码已继续下沉原生能力，且已完成 **异步 FFI 迁移**：
  - `smoldot_get_status_snapshot_async`
  - `smoldot_get_system_account_async`
  - `smoldot_get_storage_value_async`
  - `smoldot_get_storage_values_async`
  - `smoldot_get_runtime_version_async`
  - `smoldot_get_metadata_async`
  - `smoldot_get_account_next_index_async`
  - `smoldot_get_block_hash_async`
  - `smoldot_get_block_extrinsics_async`
  - `smoldot_submit_extrinsic_async`
  这些异步导出通过 `DartCallback` 回调模式返回结果，不阻塞 Dart 主线程。
  旧的同步版本（不带 `_async` 后缀）已标记废弃，后续将删除。
  Dart 侧通过 `NativeCapabilityHandler`（`chain.dart`）统一管理异步回调注册
- 当前代码已开始切换业务主路径：
  - ADR-017 后 `fetchBalance()`(best 视图)已删除；余额一律 `fetchFinalizedBalance()`
  - `ChainRpc.fetchFinalizedBalance()` / `fetchFinalizedBalances()` / `fetchFinalizedTotalBalance()` 统一走 finalized storage proof，页面金额展示只允许使用这些方法
  - ADR-017 后 `fetchConfirmedNonce()` 已删除；签名 nonce 走豁免区 `fetchNonce()`(accountNextIndex 池视图)
  - `ChainRpc.fetchStorage()` / `fetchStorageBatch()` 在轻节点模式下已改走原生 storage 读取
  - `ChainRpc.fetchRuntimeVersion()` / `fetchMetadata()` 在轻节点模式下已改走原生 capability
  - `ChainRpc.fetchLatestBlock()` 在轻节点模式下已改为复用状态快照中的 `bestBlock`
  - `ChainRpc.fetchFinalizedBlock()` 在轻节点模式下复用状态快照中的 `finalizedBlock`，钱包流水确认只允许使用该方法
  - `ChainRpc.fetchNonce()` / `fetchGenesisHash()` / `fetchBlockExtrinsicHashes()` / `submitExtrinsic()` 在轻节点模式下已切到原生 capability
- 2026-03-23 新增状态能力治理：
  - `smoldot_get_status_snapshot` 底层已不再经 `system_health` 包装，而是直接读取 `sync_service/runtime_service`
  - 本地 `smoldot-dart` 的 `Chain.getInfo()` / `getPeerCount()` / `getStatus()` / `getBestBlock*()` 也已统一收口到原生 status snapshot
- 2026-03-23 新增链数据能力治理：
  - `smoldot_get_runtime_version` / `smoldot_get_metadata` 已改为读取 runtime service / runtime call，不再走 `state_getRuntimeVersion` / `state_getMetadata`
  - `smoldot_get_account_next_index` 已改为 runtime call `AccountNonceApi_account_nonce`，不再依赖 `system_accountNextIndex`
  - `smoldot_get_block_hash` 已改为“最近块缓存 + 当前同步视图”双层原生路径，不再保留 `chain_getBlockHash`
  - `smoldot_get_block_extrinsics` 已改为只走按 block hash 下载 block body 的轻节点原生路径，不再保留 `chain_getBlock` 兜底
  - `smoldot_get_storage_value` / `smoldot_get_storage_values` / `smoldot_get_system_account` 已改为只走 `sync_service.storage_query` proof 读取，不再保留 `state_getStorage` 兜底
  - `smoldot_get_finalized_storage_value` / `smoldot_get_finalized_storage_values` / `smoldot_get_finalized_system_account` 固定读取 finalized 状态根，供余额、总额、收益等金额展示使用
- 2026-03-23 本地探针验证结果：
  - `status`、`runtimeVersion`、`metadata`、`System.Account`、单个 storage、批量 storage、`accountNextIndex`、`genesisHash`、`block_extrinsics` 都已可在 smoldot 路径稳定读出
  - 对拍本地全节点后，`System.Account` 空账户返回 `null`，`:code` 与 `state_queryStorageAt` 的大值返回和全节点一致
  - 当前探针账户 Alice 在 dev 链上不存在，因此余额为空是链上事实，不是轻节点读取失败
  - 当前同一份轻量探针已不再出现 `system_health` / `state_getRuntimeVersion` / `chain_getBlockHash` / `chain_getBlock` / `state_getStorage` 的 legacy warning
  - 当前链上读取主路径已不再保留 legacy fallback；剩余工作主要是发布前真机验证与写路径持续治理

## 6. chain_rpc.dart — 底层 RPC 方法

### 6.1 余额查询

`ChainRpc.fetchFinalizedBalance(String pubkeyHex) → Future<double>`

1. 将 `pubkeyHex` 转为 32 字节 AccountId
2. 构造 `System.Account` storage key（见 6.5）
3. 轻节点模式通过 `smoldot_get_finalized_system_account_async` 异步走 finalized storage proof 读取
4. 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额
5. 分 → 元，返回 `double`

ADR-017 后已无 best 视图余额接口；所有余额读取 finalized(`fetchFinalizedBalance`/`fetchFinalizedBalances`/`fetchFinalizedTotalBalance`)。

`ChainRpc.fetchFinalizedTotalBalance()` 读取 finalized 块上的 `free + reserved`，用于钱包详情链上余额卡；钱包列表、治理机构详情、多签余额、转账页余额提示统一读取 finalized free 余额。

### 6.2 Nonce 查询

`ChainRpc.fetchNonce(String ss58Address) → Future<int>`

- 调用原生 `smoldot_get_account_next_index_async`，底层通过 runtime call `AccountNonceApi_account_nonce` 读取 `frame_system::Account.nonce`
- wuminapp 不允许缓存、自增、预占或回滚交易 nonce；所有 signed extrinsic 每次签名前都必须实时调用 `ChainRpc.fetchNonce`
- 返回值只用于本次 extrinsic 签名，不得作为投票是否成功的判断依据

### 6.3 运行时版本

`ChainRpc.fetchRuntimeVersion() → Future<RuntimeVersion>`

- 调用原生 `smoldot_get_runtime_version`

### 6.4 链信息查询

- `fetchGenesisHash() → Future<Uint8List>` — 创世块哈希（缓存）
  - 调用原生 `smoldot_get_block_hash(0)`，优先命中创世块快路径
- `fetchLatestBlock() → Future<({Uint8List blockHash, int blockNumber})>` — 最新块
  - 复用 `status snapshot.bestBlockHash/bestBlockNumber`
- `fetchFinalizedBlock() → Future<({Uint8List blockHash, int blockNumber})>` — 最新 finalized 块
  - 复用 `status snapshot.finalizedBlockHash/finalizedBlockNumber`
  - 钱包交易流水升级为 `finalized` 时只能使用该高度
- `fetchMetadata() → Future<RuntimeMetadata>` — 运行时 metadata（缓存，含 registry）
  - 调用原生 `smoldot_get_metadata`
- `fetchBlockExtrinsicHashes(int blockNumber) → Future<List<String>?>` — 区块 extrinsic 哈希列表
  - 先通过 `smoldot_get_block_hash` 解析块 hash，再通过 `smoldot_get_block_extrinsics` 下载 block body

### 6.5 Extrinsic 提交

`ChainRpc.submitExtrinsic(Uint8List encoded, {TxPoolWatchCallback? onWatchEvent}) → Future<Uint8List>`

- 调用原生 `smoldot_submit_extrinsic`
- 返回交易哈希 32 字节
- 返回交易哈希只代表 RPC 已接收交易，不代表已出块
- 可选 `onWatchEvent` 会接收后台 `author_submitAndWatchExtrinsic` 状态：
  - `ready / broadcast`：交易进入交易池或已广播
  - `inBlock / finalized`：交易被区块包含或最终化
  - `future / invalid / dropped / usurped / retracted / finalityTimeout / timeout / error`：交易未能按预期确认，业务页面必须停止“投票中”等待态并给出可操作提示；投票类业务必须回读投票引擎 storage，不能用交易 nonce 推断确认
- 业务层不得把 `txHash` 返回渲染为“投票成功”；链上投票是否生效必须继续读取对应 storage（如 `InternalVote::InternalVotesByAccount`）确认

`ChainRpc.submitExtrinsicAndWaitForInBlock(Uint8List encoded, {TxPoolWatchCallback? onWatchEvent})`

- 走 `author_submitAndWatchExtrinsic` 提交并等待 `inBlock / finalized` 状态
- 返回本地计算的交易哈希和入块状态中的区块哈希
- 用于提案创建和投票等必须确认链上执行结果的业务；调用方拿到区块哈希后必须继续读取 `System.Events` 或业务 runtime storage
- 该方法仍不等价于业务成功，业务成功由具体 pallet 事件或投票引擎 storage 决定

### 6.6 Storage Key 计算

`System.Account` 存储映射的 key 结构：

```text
prefix     = twox_128("System") + twox_128("Account")     // 32 字节，固定常量
accountKey = blake2_128(account_id) + account_id           // 48 字节（16 + 32）
fullKey    = prefix + accountKey                            // 80 字节
```

前缀常量（hex）：`26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9`

### 6.7 AccountInfo SCALE 解码

```text
偏移 0-3:   nonce (u32)
偏移 4-7:   consumers (u32)
偏移 8-11:  providers (u32)
偏移 12-15: sufficients (u32)
偏移 16-31: free (u128, 小端序)      ← 提取此字段
偏移 32-47: reserved (u128)
偏移 48-63: frozen (u128)
偏移 64-79: flags (u128)
```

### 6.8 币种单位

- 链上最小单位：分（fen），`Balance = u128`
- 显示单位：元（yuan），`100 fen = 1 yuan`
- `TOKEN_DECIMALS = 2`
- `TOKEN_SYMBOL = "GMB"`
- `EXISTENTIAL_DEPOSIT = 111 fen`（1.11 元）

## 7. onchain.dart — 链上交易操作

### 7.1 OnchainRpc 类

onchain 模块所有需要 RPC 的功能集中于此，供 `lib/onchain` 普通链上支付和 `governance` 等业务模块使用。

### 7.2 转账

`OnchainRpc.transferKeepAlive(...)` — 完成完整转账流程：

1. 获取/缓存 metadata、genesisHash
2. 并行获取 runtimeVersion、runtime nonce、latestBlock
3. 解码目标 SS58 地址为 32 字节 accountId
4. 转换金额：`BigInt.from((amountYuan * 100).round())`
5. 构造 `Balances::transfer_keep_alive` call data（pallet_index=2）
6. 用 polkadart `SigningPayload` 构造签名载荷
7. 通过回调获取 sr25519 签名（rpc 模块与钱包模块解耦）
8. 用 polkadart `ExtrinsicPayload` 构造最终 extrinsic
9. 提交到节点，返回 tx hash

### 7.3 Extrinsic 编码

利用 polkadart 0.7.1 内置的 `SigningPayload` 和 `ExtrinsicPayload`，通过 metadata 中的 registry 自动处理所有 signed extensions。

citizenchain 12 个 TxExtension：
- 标准扩展（`CheckMortality`/`CheckNonce`/`ChargeTransactionPayment`/`CheckMetadataHash`/`CheckSpecVersion`/`CheckTxVersion`/`CheckGenesis`）：polkadart 内置处理
- 自定义扩展（`AuthorizeCall`/`CheckNonZeroSender`/`CheckNonStakeSender`/`CheckWeight`/`WeightReclaim`）：metadata 中为 NullCodec，自动跳过

Call data 格式：`[pallet_index=2] [call_index=3] [0x00 + dest_32bytes] [compact_u128(fen)]`

### 7.4 普通转账提交后的确认口径

`OnchainRpc` 不再提供 nonce 轮询确认 API。普通转账提交后先写本机 `pending` 流水，交易池 watch 收到 included 后升级为 `inBlock`；`ChainTxMonitor` 只读取 finalized 高度的 `System.Events` 后升级为 `finalized`，禁止使用 best/latest block 作为已确认真源。

> 投票类交易同样不得使用 nonce 推进判定成功。内部投票必须读
> `InternalVote::InternalVotesByAccount`，联合投票必须读
> `JointVote::JointVotesByAdmin`。

### 7.5 手续费估算

`OnchainRpc.estimateTransferFeeYuan(double amountYuan)` — 纯客户端静态方法，无需 RPC。

citizenchain 使用自定义 `OnchainChargeAdapter`，标准 `payment_queryInfo` 返回 0。客户端按链上相同逻辑计算：

- 费率：`Perbill(1_000_000)` = 0.1%
- 最低手续费：10 fen = 0.10 元
- 公式：`fee = max(amount_fen × 0.001, 10 fen)`
- 舍入：half-up 到 fen 精度（与 Rust `mul_perbill_round` 一致）

## 8. 已落地优化

### 8.1 同步状态缓存（已实现）

当前实现：

```text
启动时：SharedPreferences.read('smoldot_db_cache')
        → addChain(chainSpec, databaseContent: cached)
        → 只同步缓存之后的新区块头

同步后：chain.request('chainHead_unstable_finalizedDatabase', [maxSize])
        → SharedPreferences.write('smoldot_db_cache', result)
```

补充约束：

- 缓存恢复失败时必须自动 `remove('smoldot_db_cache')` 后重试一次
- 不允许每次启动都主动清空缓存，否则会退化成“每次冷启动全量同步”
- 当前默认同步超时为 3 分钟，且 App 启动后会立即在后台预热同步

### 8.2 批量余额查询（已实现）

当前实现：`ChainRpc.fetchFinalizedBalances(List<String> pubkeyHexList)`

新增 `ChainRpc.fetchFinalizedBalances(List<String> pubkeyHexList) → Future<Map<String, double>>`：

```text
1. 对每个 pubkeyHex 构建 System.Account storage key：
   key = SYSTEM_ACCOUNT_PREFIX + blake2b_128(accountId) + accountId
2. 调用 fetchStorageBatch(allKeys)(平名即 finalized) — 一次 finalized storage proof 请求
3. 对每个返回值：从 SCALE 字节 offset 16 读 u128 LE → ÷100 → yuan
```

`wallet_page.dart` 的 `_refreshBalancesFromChain()` 已改为一次调用 `fetchFinalizedBalances(allPubkeys)`，并在轻节点不可用时向用户展示统一错误文案，而不是把失败静默吞成 0 余额。

ADR-017 后 `fetchBalances()`(best 批量)已删除；批量余额走 `fetchFinalizedBalances()`。

### 8.3 钱包交易流水监听（已实现）

当前实现：`ChainTxMonitor`

```text
钱包新建/导入本机
  → 建立 WalletTxSyncCursorEntity，起点为当前 finalized 区块
new head 到达
  → 读取该区块 System.Events
  → 解析 Balances::Transfer
  → 命中本机钱包时写入/升级 LocalTxEntity(status=inBlock)
启动 / 订阅重连 / finalized 后
  → 补扫 finalized+1..best 的未确认区块
  → 命中本机钱包时写入/升级 LocalTxEntity(status=inBlock)
finalized head 到达
  → 按游标读取区块 System.Events
  → 解析 Balances::Transfer
  → 命中本机钱包时写入/升级 LocalTxEntity(status=finalized)
```

约束：

- 不补扫导入前历史；删除钱包时删除本地流水和同步游标，再次导入从新的导入时刻重新记录。
- 收入写入正数 `amountDeltaFen`，支出写入负数 `amountDeltaFen`；业务方向由金额正负号推导，不保存 `direction`。
- `type` 只保存业务类型；区块事件记录唯一键为 `walletPubkeyHex:blockHash:eventIndex`，本机提交记录唯一键为 `walletPubkeyHex:pending:txHash`；写入时还要按同钱包、同区块、同发送方、同接收方、同转账本金做语义去重，防止 newHeads/finalized 重复处理同一事件。
- finalized 补同步只能使用 `finalizedBlockNumber/finalizedBlockHash`，不能使用 `bestBlockNumber/bestBlockHash` 升级为 `finalized`；`bestBlockNumber/bestBlockHash` 只允许用于补扫未确认区块并写入 `inBlock`。
- finalized 单轮最多补齐 120 个区块；未确认区块单轮最多补扫 32 个区块；若 `WalletIsar` 正在处理前台读写或本地库 busy，本轮直接让路并安排短延迟重试。
- 读取区块事件仍需要节点网络和处理器参与响应 RPC，因此 App 不做全历史扫描，避免增加全节点和手机端负担。

## 9. 依赖

- `polkadart`：RPC Provider、Hasher、SigningPayload、ExtrinsicPayload、RuntimeMetadata
- `polkadart_keyring`：SR25519 签名、SS58 地址解码
- `polkadart_scale_codec`：CompactBigIntCodec、ByteOutput

## 10. 错误处理

- 轻节点未同步完成：等待同步完成后再读链上状态；超时则抛出异常
- `smoldot` 返回 JSON-RPC error：直接抛出异常，禁止吞成空结果
- 账户不存在（`System.Account` / storage proof 返回空值）：返回余额 `0.0`，不报错
- 交易提交失败（`smoldot_submit_extrinsic` 返回错误）：抛出异常，由 service 层包装为 `OnchainPaymentException`

## 11. 调用方

| 模块 | 用途 | 状态 |
| --- | --- | --- |
| `wallet` | 余额查询（`ChainRpc.fetchFinalizedBalance`） | 已实现 |
| `wallet` | 钱包交易流水监听（`ChainTxMonitor`） | 已实现 |
| `onchain` | 普通链上转账（`OnchainRpc.transferKeepAlive`） | 已实现 |
| `governance` | 提案/投票 | 规划中 |
