# PoW 轻节点长期落地方案

## 1. 目标与边界

本方案用于约束 `wuminapp` 的长期链访问方向。

目标：

- `wuminapp = 轻节点模式`
- `citizenchain = PoW + GRANDPA`
- 不保留 HTTP RPC 调试后门
- 不依赖聊天历史，而是以仓库文档和任务卡为准

边界：

- 不修改 `citizenchain` 的共识路线
- 不把 App 重新退回远程 RPC 客户端
- 不把根目录临时依赖继续留成未治理状态

## 2. 当前已确认的问题

~~当前实现能建立 `smoldot` 连接，但余额与批量 storage 查询不能正常工作，症状已经指向”轻节点内核与 Flutter 接口设计不匹配”，而不是钱包页面本身。~~

**2026-03-23 已解决：** FFI 桥接层已从同步 `block_on` 迁移到异步 `DartCallback` 回调模式（`spawn_native_capability_async`），不再阻塞 Dart 主线程。`Future.wait` 并行查询已生效，余额刷新失败时 UI 会提示用户。

~~当前主要缺口：~~（以下已全部解决）

- ~~Rust 依赖临时指向根目录 `smoldot/`，不利于版本治理与协作提交~~ → 已收编为 `third_party/smoldot-pow` submodule
- ~~Flutter 主链路仍以 legacy JSON-RPC 为核心接口~~ → 已切到 Rust 原生 typed capability
- ~~`wallet` 依赖 `state_getStorage`~~ → 已走 `smoldot_get_system_account_async`
- ~~`governance` 依赖 `state_queryStorageAt`~~ → 已走 `smoldot_get_storage_value_async` / `smoldot_get_storage_values_async`
- ~~现有设计把”轻节点”当作”App 内嵌 RPC 服务器”，而不是”PoW 轻节点能力内核”~~ → 已重构为 typed capability 架构

当前剩余工作：

- 真机验证（peer 重连、finalized sync 可靠性）
- 删除 Rust/Dart 中已废弃的旧同步 FFI 函数

**2026-03-23 新增优化方向：**

### 优化 A：同步状态缓存（冷启动加速）

现状：每次 app 启动从零同步区块头，耗时 1-2 分钟。

方案：smoldot 原生支持通过 `databaseContent` 参数恢复同步进度。

实现路径：
- 只改 `smoldot_client.dart` 的 `initialize()` 和 `_waitForSync()`
- 启动时：从 SharedPreferences 读取 `smoldot_db_cache` → 传入 `addChain(databaseContent: cached)`
- 同步完成后：通过 JSON-RPC `chainHead_unstable_finalizedDatabase` 导出 → 写回 SharedPreferences
- 不需要新增 FFI 函数，`addChain` 已支持 `databaseContent` 参数，`request()` 已支持 JSON-RPC 调用
- 缓存大小典型 1-50 KB，SharedPreferences 完全承受

效果：冷启动从分钟级降到秒级。

### 优化 B：批量余额查询（减少网络往返）

现状：`wallet_page.dart` 逐个钱包调用 `fetchBalance()`，N 个钱包 = N 次 storage proof 请求。

方案：在 Dart 层构建所有钱包的 System.Account storage key，用已有的 `fetchStorageBatch()` 一次查完。

实现路径：
- `chain_rpc.dart` 新增 `fetchBalances(List<String> pubkeyHexList)`：
  1. 构建 storage key：常量 prefix `26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9` + `blake2b_128(accountId) + accountId`
  2. 调用已有的 `fetchStorageBatch(allKeys)` — 一次网络请求
  3. 从返回的 SCALE 字节 offset 16 读 u128 little-endian 解码余额，除以 100 得 yuan
  - blake2b_128 用 polkadart 已有的 `Hasher.blake2b128`
- `wallet_page.dart` 的 `_refreshBalancesFromChain()` 改为收集所有 pubkeyHex 后一次调用 `fetchBalances()`
- 不需要改 Rust

效果：N 次网络往返变 1 次。

## 3. 目标架构

```text
wuminapp/lib/*
    ↓ 只调用 typed capability
wuminapp/lib/rpc/*
    ↓ Dart 适配层（不再透传 raw JSON-RPC 方法名）
wuminapp/native + wuminapp/rust
    ↓ PoW 轻节点能力层
wuminapp/third_party/smoldot-pow
    ↓ 自有 GitHub fork 的 submodule 引用
citizenchain P2P 网络
```

关键原则：

- Flutter 只面向“能力”编程，不面向“RPC 方法名”编程
- `smoldot` fork 是基础设施内核，不是临时下载产物
- 主仓库内只保留 submodule 引用，权威上游放在 GitHub fork

## 4. 仓库治理方案

### 4.1 权威上游

建立独立 GitHub 仓库，例如 `gmb-smoldot-pow`，作为 PoW 轻节点内核的唯一权威上游。

要求：

- 记录上游来源提交
- 记录所有 PoW 改动清单
- 能独立编译与测试
- 有明确版本标签或提交基线

### 4.2 主仓库落地形态

主仓库不再把临时 `smoldot/` 放在根目录。

改为：

```text
wuminapp/
 third_party/
    smoldot-pow/
```

规则：

- 目录以 Git submodule 方式引用 `https://github.com/ChinaNation/smoldot-pow.git`
- `wuminapp/rust/Cargo.toml` 仅依赖这个 submodule 路径
- 主仓库通过 `.gitmodules` 和 submodule commit 显式锁定版本，不能偷偷漂移
- `UPSTREAM.md` 继续保留在 `smoldot-pow` 仓库内，用于记录上游基线与 PoW 改动来源

同时，Dart FFI 包也要纳入主仓库治理：

```text
wuminapp/
  third_party/
    smoldot-dart/
```

规则：

- `wuminapp/pubspec.yaml` 只依赖仓库内 path 包
- 后续 typed capability 的 Dart 绑定统一在该本地 fork 中演进
- 不再把 pub.dev 上游包当作不可变黑盒

### 4.3 采用 submodule

当前已经选定 Git submodule。

原因：

- 用户明确要求只维护一处 `smoldot-pow` 源码
- `smoldot-pow` 已作为独立 GitHub fork 发布，适合由主仓库只引用提交指针
- 相比继续把整套源码当普通文件跟踪，submodule 更能避免父仓库提交噪音和嵌套 Git 干扰

配套要求：

- 克隆主仓库时需要同步初始化 submodule
- CI 与本地脚本需要显式执行 submodule 更新

## 5. 轻节点内核能力面

Flutter 最终只使用以下能力接口，不直接传 raw JSON-RPC 方法名：

### 5.1 生命周期与状态

- `initialize(chainSpec)`
- `shutdown()`
- `status()`：返回 `peer_count`、`sync_state`、`best_number`、`finalized_number`
- `wait_until_ready()`：明确区分“已连上 peer”和“已可读 finalized 状态”

### 5.2 链上下文

- `get_genesis_hash()`
- `get_runtime_version()`
- `get_metadata()`
- `subscribe_new_heads()`
- `subscribe_finalized_heads()`

### 5.3 账户与状态读取

- `get_system_account(account_id)`
- `get_balances_free(account_id)`
- `get_nonce(account_id)`
- `get_storage_value(key, at_finalized)`
- `get_storage_values(keys, at_finalized)`

说明：

- 即使底层仍需要 storage proof 查询，也由 Rust 层统一负责
- Dart 层不再自己管理 `state_getStorage` / `state_queryStorageAt` 的拼装、重试与错误分类

### 5.4 交易能力

- `submit_extrinsic(bytes)`
- `watch_extrinsic(hash)` 或基于头推进的确认辅助接口

## 6. PoW 专项内核改造

PoW 路线必须作为 fork 的一等特性维护，不能只停留在“能编过去”。

必须覆盖：

- PoW header 校验
- difficulty 规则校验
- PoW 与 GRANDPA finalized 结合后的同步策略
- finalized 块上的状态证明读取
- 新区块与 finalized 头推进

明确要求：

- 不能继续依赖只适用于 Aura / BABE 的默认假设
- 不能把 storage 读取失败包装成“余额为 0”这类业务结果
- 不能只验证 `system_chain`、`chain_getFinalizedHead` 这种浅层成功

## 7. Dart 侧迁移方案

### 阶段 1：收口接口

- `lib/rpc/chain_rpc.dart` 先从双通道结构改为单通道结构
- 删除“smoldot / HTTP 二选一”的接口设计
- 新增轻节点状态对象，供 UI 判断“同步中 / 可用 / 失败”

### 阶段 2：迁移钱包

- `wallet` 余额读取改为 `get_system_account()` / `get_balances_free()`
- UI 不再把链上读取异常吞掉为静默不刷新
- 钱包页展示真实同步状态与错误原因

当前进展：

- `status` 与 `System.Account` 的原生 FFI 导出已经落地
- `status` 已从 `system_health` legacy JSON-RPC 包装层迁到 `sync_service/runtime_service` 原生状态快照
- `ChainRpc.fetchBalance()` 与 `fetchConfirmedNonce()` 的轻节点分支已切到原生 `System.Account`
- `governance` 与通用链上读取依赖的批量 storage 已切到 native storage proof 读取
- `runtimeVersion`、`metadata`、`latestBlock` 的轻节点分支已切到轻节点能力层
- `fetchNonce()`、`fetchGenesisHash()`、`fetchBlockExtrinsicHashes()`、`submitExtrinsic()` 的轻节点分支已切到原生 capability
- `smoldot_get_runtime_version` / `smoldot_get_metadata` / `smoldot_get_account_next_index` 已摆脱 `state_getRuntimeVersion` / `state_getMetadata` / `system_accountNextIndex`
- `smoldot_get_block_hash` / `smoldot_get_block_extrinsics` / `smoldot_get_storage_value(s)` 已全部建立原生主路径，链上读取不再依赖 legacy fallback
- 当前轻量探针已不再触发 legacy warning，说明余额、交易、metadata、storage 主路径已经全部跑在轻节点能力层

### 阶段 3：迁移治理

- `governance` 的批量 storage 查询统一改走 native 批量读取能力
- 不再依赖 Flutter 侧直接发 `state_queryStorageAt`

阶段状态：

- 已完成

### 阶段 4：迁移交易

- `nonce`、`runtimeVersion`、`metadata`、`latestBlock` 全部改走轻节点能力层
- extrinsic 提交与确认流程只依赖轻节点路径

阶段状态：

- 已基本完成：`nonce`、`latestBlock`、`genesisHash`、extrinsic 提交与区块内 extrinsics 确认链路已迁移
- 已继续完成：`runtimeVersion` / `metadata` / `accountNextIndex` / `storage` / `block body` 主路径都已脱离 legacy JSON-RPC 语义
- 当前剩余工作：补齐发布前的真机验证矩阵，并继续治理写路径的长期演进

### 阶段 5：封板与口径清理

- 删除 `WUMINAPP_RPC_URL`
- 删除 HTTP provider 分支
- 删除 WebSocket 回退分支
- 删除相关脚本分支、文档说明与残留测试口径

阶段状态：

- 已完成第一轮：`ChainRpc` 已改为纯轻节点单通道，`main.dart` 不再读取 `WUMINAPP_RPC_URL`
- 已完成第一轮：`app-run.sh` / `app-clean-run.sh` 不再透传 HTTP RPC 配置
- 已完成第一轮：`ChainEventSubscription` 已移除 WebSocket 回退，只保留 smoldot 新区块订阅
- 当前剩余工作：清理更深层的历史文档/测试口径，并继续完善真机验证与发布封板

## 8. citizenchain 配套要求

在不改 PoW 路线的前提下，需要保证以下配套能力稳定：

- `chainspec.json` 导出流程固定，可随发布一起更新
- bootNodes 来源唯一，不允许 App 与链侧各写一套
- dev / test / release 链的链规格文件都有明确来源

可选优化：

- 发布时附带 finalized checkpoint / 启动基线元数据，缩短手机首次同步时间

说明：

- 这属于发布与构建产物治理，不属于修改链共识

## 9. 验证矩阵

没有 HTTP 回退后，以下检查必须全部通过，才能视为可发布：

- 轻节点冷启动后能进入可读 finalized 状态
- `System.Account` 可读，余额与全节点一致
- 治理批量 storage 查询可用，结果与全节点一致
- runtime metadata / version 正常获取
- nonce 正常获取
- extrinsic 正常提交
- 新区块订阅与确认链路正常
- App 前后台切换、断网重连后可恢复

## 10. 执行顺序

建议按以下顺序实施：

1. 先治理依赖与仓库结构
2. 再稳定 PoW 轻节点 fork
3. 再建设 Rust typed capability
4. 再迁移 Flutter 业务层
5. 最后压缩残余 fallback，并以验证矩阵封板

当前仓库治理状态补充：

- `https://github.com/ChinaNation/smoldot-pow` 已作为 PoW fork 发布
- `wuminapp/third_party/smoldot-pow` 已切换为 submodule 引用该仓库

## 11. 发布门槛

以下任一项未完成，不应宣称 `wuminapp` 已完成 PoW 轻节点化：

- 仍依赖隐藏 HTTP RPC 回退
- 仍把 raw JSON-RPC 作为 Flutter 主业务接口
- 仍存在根目录临时 `smoldot/` 嵌套仓库
- 仍无法在 finalized 状态下稳定读取账户与治理 storage
