# RPC 模块技术文档（当前实现态）

## 1. 模块定位

`lib/rpc/` 是手机 App 与区块链节点通信的唯一收口模块。

职责：

- 维护 44 个引导节点的 RPC 地址列表
- 节点自动选择与故障切换
- 提供底层 JSON-RPC 调用能力
- 链上状态查询（余额、账户信息等）

约束：所有链上通信必须通过本模块，业务模块不直接建立 RPC 连接。

## 2. 目录结构

```text
lib/rpc/
├── chain_rpc.dart       ← 核心：节点连接管理 + 链上查询方法
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

1. 启动时随机打乱 44 个节点的顺序
2. 依次尝试连接，使用第一个可达的节点
3. 缓存当前可用节点，后续请求复用
4. 当前节点请求失败时，标记为不可用，自动切换到下一个
5. 全部节点不可达时抛出异常

## 6. 链上查询能力

### 6.1 余额查询

入口方法：`ChainRpc.fetchBalance(String pubkeyHex)`

技术流程：

1. 将 `pubkeyHex`（`0x` + 64 hex）转为 32 字节 AccountId
2. 构造 `System.Account` 的 storage key
3. 调用 `state_getStorage` JSON-RPC
4. 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额
5. 将分（fen）转换为元（yuan），返回 `double`

### 6.2 Storage Key 计算

`System.Account` 存储映射的 key 结构：

```text
prefix     = twox_128("System") + twox_128("Account")     // 32 字节，固定常量
accountKey = blake2_128(account_id) + account_id           // 48 字节（16 + 32）
fullKey    = prefix + accountKey                            // 80 字节
```

前缀常量（hex）：`26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9`

### 6.3 AccountInfo SCALE 解码

`AccountInfo` 结构体的字节布局：

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

### 6.4 币种单位

- 链上最小单位：分（fen），`Balance = u128`
- 显示单位：元（yuan），`100 fen = 1 yuan`
- `TOKEN_DECIMALS = 2`
- `TOKEN_SYMBOL = "GMB"`
- `EXISTENTIAL_DEPOSIT = 111 fen`（1.11 元）

转换公式：`显示余额(元) = 链上余额(fen) / 100`

## 7. 依赖

- `polkadart`：Substrate RPC Provider、Hasher（`twoxx128`、`blake2b128`）
- `http`：HTTP 传输层

`polkadart` 已作为 `polkadart_keyring` 的传递依赖存在，提升为直接依赖。

## 8. 错误处理

- 单节点请求失败：静默切换下一节点，不中断业务
- 全部节点不可达：抛出异常，由调用方 UI 展示错误提示
- 账户不存在（`state_getStorage` 返回 `null`）：返回余额 `0.0`，不报错

## 9. 调用方

| 模块 | 用途 | 状态 |
| --- | --- | --- |
| `wallet` | 余额查询 | 已实现 |
| `trade/onchain` | 转账（构造/提交 extrinsic） | 规划中 |
| `governance` | 提案/投票 | 规划中 |

## 10. 规划扩展

后续 `chain_rpc.dart` 将新增：

- `submitExtrinsic(hex)` — 提交已签名的 extrinsic
- `getNonce(pubkeyHex)` — 查询账户 nonce（构造交易用）
- `getRuntimeVersion()` — 查询运行时版本（构造交易用）
- `getGenesisHash()` — 查询创世区块哈希（构造交易用）
- `queryStorage(palletName, storageName, key)` — 通用 storage 查询

这些方法将在转账/提案/投票直连 RPC 时逐步实现。
