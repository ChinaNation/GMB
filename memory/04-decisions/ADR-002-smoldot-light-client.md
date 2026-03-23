# ADR-002: wuminapp 从 HTTP RPC 迁移到 smoldot 轻节点

## 背景

wuminapp 原先通过 HTTP RPC（HttpProvider）连接 44 个远程全节点访问 citizenchain。随着用户增长，这种架构面临以下问题：

- 集中式 RPC 节点成为性能瓶颈和单点故障
- WebSocket 订阅连接数受限（Substrate 节点默认 100-500）
- 投票日等高峰期的突发流量无法有效分散
- 维护大量 RPC 节点的服务器资源成本高

## 决策

将 wuminapp 的区块链访问层从 HTTP RPC 迁移到 smoldot 轻节点。

- 使用 `smoldot` Dart 包（v0.1.1，pub.dev）在 App 内嵌入轻客户端
- 轻节点通过 P2P 网络直接参与 citizenchain，不依赖远程 RPC 服务器
- 保留 `WUMINAPP_RPC_URL` 环境变量作为开发调试时的 HTTP RPC 回退模式
- chainspec 从 citizenchain 节点导出，打包到 App assets 中

## 影响

- `lib/rpc/chain_rpc.dart`：底层 provider 从 HttpProvider 替换为 SmoldotClientManager
- `lib/rpc/chain_event_subscription.dart`：smoldot 模式下使用原生订阅替代 WebSocket
- `lib/rpc/smoldot_client.dart`：新增 smoldot 轻节点管理器（全局单例）
- 启动脚本（`app-run.sh`、`app-clean-run.sh`）：默认不再传入 RPC URL
- 服务层和 UI 层不需要修改（ChainRpc 接口保持不变）
- 需要编译 smoldot native library（Rust → .so/.dylib）
- 需要从 citizenchain 导出 chainspec.json

## 备选方案

| 方案 | 否决原因 |
|---|---|
| RPC 代理层（Nginx/HAProxy） | 需要大量服务器资源，与用户数线性增长 |
| 只读 RPC 节点集群 | 同上，且运维复杂 |
| 客户端智能路由 | 治标不治本，总并发仍受节点限制 |
| 链下索引服务（Subsquid） | 需要额外基础设施，不解决交易提交问题 |

## 后续动作

1. 编译 citizenchain 并导出 chainspec：`scripts/generate-chainspec.sh`
2. 编译 smoldot native library：`scripts/build-smoldot-native.sh`
3. 真机验证全流程（余额查询、提案、投票、转账）
4. 确认 citizenchain 全节点开启了轻客户端协议支持
