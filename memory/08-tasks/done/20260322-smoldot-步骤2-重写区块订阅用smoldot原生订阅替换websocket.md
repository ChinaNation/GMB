# 步骤2：重写 ChainEventSubscription，用 smoldot 原生订阅替换 WebSocket

## 状态：已完成

## 背景

当前 `chain_event_subscription.dart` 手动建立 WebSocket 连接到远程 RPC 节点，订阅 `chain_subscribeNewHeads`。在轻节点模式下，smoldot 自身就在同步区块头，应通过 smoldot 的 JSON-RPC 接口订阅，不再需要独立的 WebSocket 连接。

本任务是三步迁移的第二步，依赖步骤1完成。

## 所属系列

smoldot 轻节点迁移（2/3）

## 前置依赖

- 步骤1完成：smoldot_client.dart 可用，ChainRpc 已切换到 smoldot

## 待完成项

### 1. 重写 chain_event_subscription.dart

- 删除 `WebSocketChannel` 手动连接逻辑
- 删除 HTTP → WS URL 转换逻辑
- 删除 3 秒自动重连逻辑
- 改为通过 smoldot_client 的 JSON-RPC 接口发起 `chain_subscribeNewHeads` 订阅
- 保持 `events` Stream<ChainEvent> 接口不变
- 保持 `connect()` / `disconnect()` 方法签名不变（内部实现改为操作 smoldot 订阅）

### 2. 验证

- [ ] 新区块事件实时推送到 UI
- [ ] `all_proposals_view.dart` 的提案列表实时刷新正常
- [ ] 断网后恢复，订阅自动恢复
- [ ] App 切后台/回前台，订阅状态正确

## 涉及文件

- `wuminapp/lib/rpc/chain_event_subscription.dart` — 重写

## 不涉及文件

- `wuminapp/lib/governance/all_proposals_view.dart` — 接口不变，无需改动
- 其他 UI 页面 — 无需改动

## 风险点

- smoldot 订阅的回调格式可能与原始 WebSocket RPC 响应格式略有差异，需要适配解析
