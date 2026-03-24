# RPC 模块技术文档（当前实现态）

> 说明：本文描述的是当前代码实现态。
> 当前实现已切为“PoW 专用轻节点 + 无 HTTP 回退”，以 [ADR-004](../../04-decisions/ADR-004-pow-light-client-without-http-fallback.md) 与 [POW 轻节点长期落地方案](./POW_LIGHT_CLIENT_ROADMAP.md) 为准。

## 1. 模块定位

`lib/rpc/` 是手机 App 与区块链节点通信的唯一收口模块。

职责：

- 默认通过 `smoldot` 轻节点接入 citizenchain P2P 网络
- 提供底层 JSON-RPC 调用能力
- 提供轻节点状态快照等逐步收口中的 typed capability
- 链上状态查询（余额、nonce、metadata 等）
- 链上交易构造与提交（转账、未来的投票/提案）

约束：所有链上通信必须通过本模块，业务模块不直接建立 RPC 连接。

补充说明：

- 当前 `smoldot` Dart 绑定已从 pub.dev 依赖切换为仓库内本地 fork：`wuminapp/third_party/smoldot-dart`
- 当前 `smoldot-light` Rust 内核通过 Git submodule 位于：`wuminapp/third_party/smoldot-pow`
- 这两层收编的目的，是为后续 PoW 专用 typed capability 改造建立可控演进入口

## 2. 目录结构

```text
lib/rpc/
├── chain_rpc.dart       ← 底层：节点连接管理 + JSON-RPC 方法
├── onchain.dart         ← 业务：extrinsic 构造 + 转账 + 交易确认
├── rpc.dart             ← barrel export
└── RPC_TECHNICAL.md
```

## 3. 通信架构

```text
手机 App  --smoldot-->  bootNodes / P2P 网络
```

- 默认协议：`smoldot` 轻客户端 + Substrate JSON-RPC

## 4. chainspec 与轻节点模式

默认模式下，App 从 `assets/chainspec.json` 加载链规格，并使用其中的 `bootNodes` 加入网络。

当前要求：

- `chainspec.json` 必须与目标链的 genesis / properties / bootNodes 一致
- 如果打进 App 的 chainspec 错了，轻节点即使“连上了”，也可能连到错误链或错误引导节点
- `bootNodes` 的来源应以 `citizenchain/node/src/chain_spec.rs` 为准

## 5. 连接与同步策略

1. App 启动时初始化 `SmoldotClientManager`
2. 轻节点加入 `chainspec.json` 指定的 citizenchain 网络
3. `ChainRpc` 在发起余额、nonce、metadata、storage、extrinsic 等链上请求前，先等待轻节点完成同步
4. 同步完成后才继续真正的 JSON-RPC 请求，避免把“尚未同步”误判成“链上没有数据”

补充说明：

- 钱包余额不更新的首要风险点，不是 UI，而是“轻节点已初始化但尚未同步完成”时过早查询链上状态
- `smoldot` 返回 JSON-RPC error 时必须抛出，不能把错误吞成 `null`，否则上层会把真实故障误判为余额为 0 或账户不存在
- 当前代码已新增 `SmoldotClientManager.getStatusSnapshot()`，作为结构化轻节点状态接口；其底层已改为 Rust 原生 capability，不再由 Dart 层拼装 `system_health`
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
  - `ChainRpc.fetchBalance()` 在轻节点模式下已优先走 `SmoldotClientManager.getSystemAccountSnapshot()`
  - `ChainRpc.fetchConfirmedNonce()` 在轻节点模式下已优先走原生 `System.Account` 快照中的 nonce
  - `ChainRpc.fetchStorage()` / `fetchStorageBatch()` 在轻节点模式下已改走原生 storage 读取
  - `ChainRpc.fetchRuntimeVersion()` / `fetchMetadata()` 在轻节点模式下已改走原生 capability
  - `ChainRpc.fetchLatestBlock()` 在轻节点模式下已改为复用状态快照中的 `bestBlock`
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
- 2026-03-23 本地探针验证结果：
  - `status`、`runtimeVersion`、`metadata`、`System.Account`、单个 storage、批量 storage、`accountNextIndex`、`genesisHash`、`block_extrinsics` 都已可在 smoldot 路径稳定读出
  - 对拍本地全节点后，`System.Account` 空账户返回 `null`，`:code` 与 `state_queryStorageAt` 的大值返回和全节点一致
  - 当前探针账户 Alice 在 dev 链上不存在，因此余额为空是链上事实，不是轻节点读取失败
  - 当前同一份轻量探针已不再出现 `system_health` / `state_getRuntimeVersion` / `chain_getBlockHash` / `chain_getBlock` / `state_getStorage` 的 legacy warning
  - 当前链上读取主路径已不再保留 legacy fallback；剩余工作主要是发布前真机验证与写路径持续治理

## 6. chain_rpc.dart — 底层 RPC 方法

### 6.1 余额查询

`ChainRpc.fetchBalance(String pubkeyHex) → Future<double>`

1. 将 `pubkeyHex` 转为 32 字节 AccountId
2. 构造 `System.Account` storage key（见 6.5）
3. 轻节点模式通过 `smoldot_get_system_account_async` 异步走 storage proof 读取
4. 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额
5. 分 → 元，返回 `double`

### 6.2 Nonce 查询

`ChainRpc.fetchNonce(String ss58Address) → Future<int>`

- 调用原生 `smoldot_get_account_next_index_async`
- 返回下一个可用 nonce（含交易池 pending）

### 6.3 运行时版本

`ChainRpc.fetchRuntimeVersion() → Future<RuntimeVersion>`

- 调用原生 `smoldot_get_runtime_version`

### 6.4 链信息查询

- `fetchGenesisHash() → Future<Uint8List>` — 创世块哈希（缓存）
  - 调用原生 `smoldot_get_block_hash(0)`，优先命中创世块快路径
- `fetchLatestBlock() → Future<({Uint8List blockHash, int blockNumber})>` — 最新块
  - 复用 `status snapshot.bestBlockHash/bestBlockNumber`
- `fetchMetadata() → Future<RuntimeMetadata>` — 运行时 metadata（缓存，含 registry）
  - 调用原生 `smoldot_get_metadata`
- `fetchBlockExtrinsicHashes(int blockNumber) → Future<List<String>?>` — 区块 extrinsic 哈希列表
  - 先通过 `smoldot_get_block_hash` 解析块 hash，再通过 `smoldot_get_block_extrinsics` 下载 block body

### 6.5 Extrinsic 提交

`ChainRpc.submitExtrinsic(Uint8List encoded) → Future<Uint8List>`

- 调用原生 `smoldot_submit_extrinsic`
- 返回交易哈希 32 字节

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

onchain 模块所有需要 RPC 的功能集中于此，供 `trade/onchain` 和未来的 `governance` 模块使用。

### 7.2 转账

`OnchainRpc.transferKeepAlive(...)` — 完成完整转账流程：

1. 获取/缓存 metadata、genesisHash
2. 并行获取 runtimeVersion、nonce、latestBlock
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
- 自定义扩展（`AuthorizeCall`/`CheckNonZeroSender`/`CheckNonKeylessSender`/`CheckWeight`/`WeightReclaim`）：metadata 中为 NullCodec，自动跳过

Call data 格式：`[pallet_index=2] [call_index=3] [0x00 + dest_32bytes] [compact_u128(fen)]`

### 7.4 交易确认

`OnchainRpc.isTxConfirmed(address, usedNonce)` — 通过 nonce 对比判断交易是否已被打包。

### 7.5 手续费估算

`OnchainRpc.estimateTransferFeeYuan(double amountYuan)` — 纯客户端静态方法，无需 RPC。

citizenchain 使用自定义 `PowOnchainChargeAdapter`，标准 `payment_queryInfo` 返回 0。客户端按链上相同逻辑计算：

- 费率：`Perbill(1_000_000)` = 0.1%
- 最低手续费：10 fen = 0.10 元
- 公式：`fee = max(amount_fen × 0.001, 10 fen)`
- 舍入：half-up 到 fen 精度（与 Rust `mul_perbill_round` 一致）

## 8. 依赖

- `polkadart`：RPC Provider、Hasher、SigningPayload、ExtrinsicPayload、RuntimeMetadata
- `polkadart_keyring`：SR25519 签名、SS58 地址解码
- `polkadart_scale_codec`：CompactBigIntCodec、ByteOutput

## 9. 错误处理

- 轻节点未同步完成：等待同步完成后再读链上状态；超时则抛出异常
- `smoldot` 返回 JSON-RPC error：直接抛出异常，禁止吞成空结果
- 账户不存在（`System.Account` / storage proof 返回空值）：返回余额 `0.0`，不报错
- 交易提交失败（`smoldot_submit_extrinsic` 返回错误）：抛出异常，由 service 层包装为 `OnchainTradeException`

## 10. 调用方

| 模块 | 用途 | 状态 |
| --- | --- | --- |
| `wallet` | 余额查询（`ChainRpc.fetchBalance`） | 已实现 |
| `trade/onchain` | 转账（`OnchainRpc.transferKeepAlive`） | 已实现 |
| `governance` | 提案/投票 | 规划中 |
