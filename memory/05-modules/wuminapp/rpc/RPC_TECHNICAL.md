# RPC 模块技术文档（当前实现态）

## 1. 模块定位

`lib/rpc/` 是手机 App 与区块链节点通信的唯一收口模块。

职责：

- 默认通过 `smoldot` 轻节点接入 citizenchain P2P 网络
- 在开发调试场景下回退到 HTTP JSON-RPC
- 提供底层 JSON-RPC 调用能力
- 链上状态查询（余额、nonce、metadata 等）
- 链上交易构造与提交（转账、未来的投票/提案）

约束：所有链上通信必须通过本模块，业务模块不直接建立 RPC 连接。

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
默认：
手机 App  --smoldot-->  bootNodes / P2P 网络

调试回退：
手机 App  --HTTP JSON-RPC-->  指定节点 :9944
```

- 默认协议：`smoldot` 轻客户端 + Substrate JSON-RPC
- 回退协议：HTTP JSON-RPC
- 回退端口：9944（与 P2P 端口 30333 不同）
- 回退模式下，节点仍需开启 `--rpc-external --rpc-cors=all`

## 4. chainspec 与回退模式

默认模式下，App 从 `assets/chainspec.json` 加载链规格，并使用其中的 `bootNodes` 加入网络。

当前要求：

- `chainspec.json` 必须与目标链的 genesis / properties / bootNodes 一致
- 如果打进 App 的 chainspec 错了，轻节点即使“连上了”，也可能连到错误链或错误引导节点
- `bootNodes` 的来源应以 `citizenchain/node/src/chain_spec.rs` 为准

### 4.1 地址覆盖

支持通过环境变量切换到 HTTP RPC 回退模式：

```bash
flutter run --dart-define=WUMINAPP_RPC_URL=http://127.0.0.1:9944
```

设置后 App 不再使用 `smoldot`，而是只访问该单一 RPC 地址。用于本地开发调试。

手机访问某个 RPC 地址时，不要求一定连公网互联网，但要求手机对该地址具备网络可达性：

- `http://10.x.x.x:9944` / `http://192.168.x.x:9944`：手机与节点必须处于同一局域网、热点、VPN 或 USB 网络共享链路
- `http://<公网域名>:9944`：手机需要普通互联网连接
- `http://127.0.0.1:9944`：只对手机自身有效，真机调试时不能拿来访问电脑上的节点

## 5. 连接与同步策略

1. App 启动时初始化 `SmoldotClientManager`
2. 轻节点加入 `chainspec.json` 指定的 citizenchain 网络
3. `ChainRpc` 在发起余额、nonce、metadata、storage、extrinsic 等链上请求前，先等待轻节点完成同步
4. 同步完成后才继续真正的 JSON-RPC 请求，避免把“尚未同步”误判成“链上没有数据”
5. 开发调试时如设置 `WUMINAPP_RPC_URL`，则直接走 HTTP RPC，不经过轻节点同步门槛

补充说明：

- 钱包余额不更新的首要风险点，不是 UI，而是“轻节点已初始化但尚未同步完成”时过早查询链上状态
- `smoldot` 返回 JSON-RPC error 时必须抛出，不能把错误吞成 `null`，否则上层会把真实故障误判为余额为 0 或账户不存在

## 6. chain_rpc.dart — 底层 RPC 方法

### 6.1 余额查询

`ChainRpc.fetchBalance(String pubkeyHex) → Future<double>`

1. 将 `pubkeyHex` 转为 32 字节 AccountId
2. 构造 `System.Account` storage key（见 6.5）
3. 调用 `state_getStorage`
4. 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额
5. 分 → 元，返回 `double`

### 6.2 Nonce 查询

`ChainRpc.fetchNonce(String ss58Address) → Future<int>`

调用 `system_accountNextIndex`，返回下一个可用 nonce（含交易池 pending）。

### 6.3 运行时版本

`ChainRpc.fetchRuntimeVersion() → Future<RuntimeVersion>`

调用 `state_getRuntimeVersion`，返回 `specVersion` + `transactionVersion`。

### 6.4 链信息查询

- `fetchGenesisHash() → Future<Uint8List>` — 创世块哈希（缓存）
- `fetchLatestBlock() → Future<({Uint8List blockHash, int blockNumber})>` — 最新块
- `fetchMetadata() → Future<RuntimeMetadata>` — 运行时 metadata（缓存，含 registry）

### 6.5 Extrinsic 提交

`ChainRpc.submitExtrinsic(Uint8List encoded) → Future<Uint8List>`

调用 `author_submitExtrinsic`，返回交易哈希 32 字节。

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
- HTTP RPC 回退模式请求失败：抛出异常，由调用方 UI 展示错误提示
- 账户不存在（`state_getStorage` 返回 `null`）：返回余额 `0.0`，不报错
- 交易提交失败（`author_submitExtrinsic` 返回错误）：抛出异常，由 service 层包装为 `OnchainTradeException`

## 10. 调用方

| 模块 | 用途 | 状态 |
| --- | --- | --- |
| `wallet` | 余额查询（`ChainRpc.fetchBalance`） | 已实现 |
| `trade/onchain` | 转账（`OnchainRpc.transferKeepAlive`） | 已实现 |
| `governance` | 提案/投票 | 规划中 |
