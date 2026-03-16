# RPC 模块技术文档（当前实现态）

## 1. 模块定位

`lib/rpc/` 是手机 App 与区块链节点通信的唯一收口模块。

职责：

- 维护 44 个引导节点的 RPC 地址列表
- 节点自动选择与故障切换
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
手机 App  --HTTP JSON-RPC-->  引导节点 :9944
```

- 协议：Substrate 标准 JSON-RPC（HTTP 模式）
- 端口：9944（与 P2P 端口 30333 不同）
- 节点要求：引导节点启动时须添加 `--rpc-external --rpc-cors=all`

## 4. 节点列表

内置 44 个引导节点的 RPC 地址，域名来源于 `citizenchain/node/src/chain_spec.rs` 中的 P2P 引导节点定义。

RPC 地址格式：`http://<域名>:9944`

完整节点列表见 `WUMINAPP_TECHNICAL.md` 第 6.2 节。

### 4.1 地址覆盖

支持通过环境变量覆盖默认节点：

```bash
flutter run --dart-define=WUMINAPP_RPC_URL=http://127.0.0.1:9944
```

设置后 App 仅使用该单一地址，不走节点列表。用于本地开发调试。

## 5. 节点选择策略

1. 本地节点（`127.0.0.1:9944`）放最前，其余 43 个节点随机打乱
2. 依次尝试连接，使用第一个可达的节点
3. 缓存当前可用节点，后续请求复用
4. 当前节点请求失败时，标记为不可用，自动切换到下一个
5. 最多尝试 3 个节点，全部不可达时抛出异常
6. 单节点超时：8 秒

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

## 8. 依赖

- `polkadart`：RPC Provider、Hasher、SigningPayload、ExtrinsicPayload、RuntimeMetadata
- `polkadart_keyring`：SR25519 签名、SS58 地址解码
- `polkadart_scale_codec`：CompactBigIntCodec、ByteOutput

## 9. 错误处理

- 单节点请求失败：静默切换下一节点，不中断业务
- 全部节点不可达：抛出异常，由调用方 UI 展示错误提示
- 账户不存在（`state_getStorage` 返回 `null`）：返回余额 `0.0`，不报错
- 交易提交失败（`author_submitExtrinsic` 返回错误）：抛出异常，由 service 层包装为 `OnchainTradeException`

## 10. 调用方

| 模块 | 用途 | 状态 |
| --- | --- | --- |
| `wallet` | 余额查询（`ChainRpc.fetchBalance`） | 已实现 |
| `trade/onchain` | 转账（`OnchainRpc.transferKeepAlive`） | 已实现 |
| `governance` | 提案/投票 | 规划中 |
