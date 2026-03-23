# 步骤1：集成 smoldot Dart 包，替换 ChainRpc 底层 provider

## 状态：已完成

## 背景

wuminapp 当前通过 HTTP RPC 连接 44 个远程全节点访问 citizenchain。随着用户增长，集中式 RPC 节点会成为性能瓶颈和单点故障。决定迁移到 smoldot 轻节点方案，让用户手机直接参与 P2P 网络。

本任务是三步迁移的第一步，目标是在不改变上层接口的前提下，将底层网络层从 HttpProvider 替换为 smoldot 轻客户端。

## 所属系列

smoldot 轻节点迁移（1/3）

## 待完成项

### 1. 引入 smoldot Dart 包

- pubspec.yaml 添加 `smoldot: ^0.1.2` 依赖
- 确认 Android（arm64、armeabi-v7a）和 iOS 编译通过
- 确认 smoldot native library 正确链接

### 2. 导出 citizenchain chainspec

- 从 citizenchain 节点执行 `./citizenchain build-spec --chain=live --raw > chainspec.json`
- 将 chainspec.json 放入 wuminapp/assets/
- 在 pubspec.yaml 中注册 asset

### 3. 新建 smoldot_client.dart

- 位置：`wuminapp/lib/rpc/smoldot_client.dart`
- 职责：
  - App 启动时初始化 smoldot 轻客户端
  - 加载 citizenchain chainspec
  - 提供 JSON-RPC 请求/响应接口
  - 管理轻客户端生命周期（启动、关闭、同步状态查询）
- 对外暴露一个全局单例，供 ChainRpc 使用

### 4. 重写 chain_rpc.dart

- 将 `HttpProvider` 替换为 smoldot JSON-RPC 通道
- 删除 44 个 RPC 节点 URL 列表（`_nodes`）
- 删除故障转移逻辑（`_currentIndex`、`_switchToNextNode`）
- 删除 `WUMINAPP_RPC_URL` 环境变量支持
- 保持所有对外公开方法签名不变：
  - `fetchStorageBatch()` / `fetchNonce()` / `fetchRuntimeVersion()`
  - `fetchGenesisHash()` / `fetchLatestBlock()` / `fetchMetadata()`
  - `fetchStorage()` / `fetchBalance()` / `fetchCurrentSfidMainPubkeyHex()`
  - `submitExtrinsic()`
- 保留缓存机制（genesis hash、metadata、sfid pubkey）

### 5. 验证

- [ ] Android 编译通过
- [ ] iOS 编译通过
- [ ] smoldot 启动并成功同步区块头
- [ ] `fetchBalance` 查询返回正确余额
- [ ] `fetchStorage` 查询返回正确存储数据
- [ ] `submitExtrinsic` 交易提交成功

## 涉及文件

- `wuminapp/pubspec.yaml` — 新增依赖
- `wuminapp/assets/chainspec.json` — 新增
- `wuminapp/lib/rpc/smoldot_client.dart` — 新增
- `wuminapp/lib/rpc/chain_rpc.dart` — 重写底层 provider

## 不涉及文件

- `wuminapp/lib/rpc/chain_event_subscription.dart` — 步骤2处理
- `wuminapp/lib/governance/*_service.dart` — 步骤3处理
- `wuminapp/lib/wallet/capabilities/api_client.dart` — SFID HTTP 接口不受影响

## 风险点

- smoldot Dart 包版本较新（v0.1.2），可能存在未知兼容性问题
- 轻节点首次同步需要时间，需要在 UI 上给出同步状态提示
- citizenchain 的共识算法必须是 smoldot 支持的（Aura/BABE + GRANDPA）
