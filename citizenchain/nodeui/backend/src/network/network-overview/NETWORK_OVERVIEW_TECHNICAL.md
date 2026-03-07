# Network Overview 模块技术文档

## 1. 模块位置

- 路径：`nodeui/backend/src/network/network-overview/mod.rs`
- 对外命令：
  - `get_network_overview`

## 2. 模块职责

- 输出网络总览统计：
  - 总节点数
  - 在线节点
  - 国储会/省储会/省储行节点数
  - 全节点/轻节点数
- 维护历史已见节点集合（`known-peers.json`）。
- 在数据不完整或链 RPC 异常时返回告警信息（`warning`）。

## 3. 对外数据模型

- `NetworkOverview {`
  - `total_nodes`
  - `online_nodes`
  - `guochuhui_nodes`
  - `shengchuhui_nodes`
  - `shengchuhang_nodes`
  - `full_nodes`
  - `light_nodes`
  - `warning`
- `}`

## 4. 统计来源

- 创世引导节点列表：`settings/bootnodes-address`。
- 实时在线节点：`system_peers`。
- 本机节点状态：`home/home-node::current_status` + `system_localPeerId`。
- 历史已见节点：`known-peers.json`。

## 5. 核心规则

1. 在线节点去重：
   - 先采集 `system_peers` 到 `online_peer_ids`（集合）。
   - 本机节点运行时优先插入 `system_localPeerId`；失败则按本机在线 `+1` 估算。
2. 分类统计：
   - 按创世节点名称关键字（国储会/储行/省储会）对在线 PeerId 分类。
3. 全节点统计：
   - `full_nodes = online_nodes - light_nodes`（本机在线但角色未知时默认计入 full）。
4. 总节点统计：
   - `创世节点数 + 已见非创世节点数`。

## 6. known-peers 持久化策略

- 存储路径：`<app_data_dir>/known-peers.json`。
- 仅保留合法 PeerId（ASCII 字母数字，非空，长度 <= 128）。
- 设置上限 `KNOWN_PEERS_MAX = 5000`，超限时截断。
- 仅在集合变化时写盘，避免每次查询都落盘。

## 7. RPC 健壮性与链指纹校验

- RPC 调用统一具备：
  - connect/read/write timeout
  - 响应上限（4MB）
  - HTTP 状态行检查（必须 200）
  - JSON-RPC `error` 显式报错
- 统计前先做链指纹校验：
  - `system_properties.ss58Format == 2027`
  - `system_name` 非空
- 指纹不匹配时不信任网络统计，返回告警并降级输出。

## 8. 告警策略

以下场景会写入 `warning`：
- known-peers 读写失败
- RPC 指纹校验失败
- `system_peers/system_localPeerId` 读取失败或格式异常
- 收到无效 peerId
- known-peers 超限被截断

`warning` 采用合并文本（中文分号分隔），前端直接展示。

## 9. 依赖关系

- 依赖 `home/home-node` 的 `current_status` 获取本机运行状态。
- 依赖 `settings/bootnodes-address` 的创世节点元数据。
- 依赖 `settings/security` 提供应用数据目录。
