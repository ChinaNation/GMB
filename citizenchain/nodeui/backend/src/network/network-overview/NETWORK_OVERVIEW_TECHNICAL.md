# Network Overview 模块技术文档

## 0. 功能需求

- 页面需要展示网络总览指标，包括总节点数、在线节点数、国储会节点、省储会节点、省储行节点、全节点数和轻节点数。
- 模块需要优先基于本机节点当前可观测到的 `system_peers` 与 `system_localPeerId` 生成在线网络统计。
- 当本机节点正在运行时，模块需要把本机节点计入在线节点，并尽量识别其轻节点/全节点角色。
- 模块需要持续维护历史已见 PeerId 集合，用于在实时在线数据不完整时补足“总节点数”视角。
- 前端会定时轮询该接口，模块需要支持高频读取，并避免每次请求都无意义重写 `known-peers.json`。
- 当 RPC 不可用、链指纹不匹配、PeerId 非法或部分字段缺失时，模块需要返回尽量可展示的统计结果，并通过 `warning` 告知降级原因。
- 统计口径需要保持自洽：`onlineNodes`、`fullNodes`、`lightNodes` 的去重和本机计入口径要一致，避免出现在线节点已计入但 full/light 漏计的情况。
- 模块需要避免把错误链或错误端口上的 RPC 数据误当作目标网络统计结果。

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
   - 按创世节点名称前缀精确匹配（`starts_with("国储会")`/`starts_with("省储会")`/`starts_with("省储行")`）对在线 PeerId 分类。
3. 全节点统计：
   - 远端 light 节点按唯一 PeerId 去重。
   - 本机在线但角色未知时默认计入 full。
   - `full_nodes + light_nodes` 与在线节点口径保持一致。
4. 总节点统计：
   - `创世节点数 + 已见非创世节点数`。

## 6. known-peers 持久化策略

- 存储路径：`<app_data_dir>/known-peers.json`。
- 仅保留合法 libp2p PeerId（ASCII 字母数字、`12D3KooW` 前缀、长度 46–128）。
- 设置上限 `KNOWN_PEERS_MAX = 5000`，超限时从头部（最久未见）截断。
- 合并策略采用 LRU：已知且在线的 peer 移到队尾，全新 peer 追加到队尾，使活跃节点不易被淘汰。
- 使用内存缓存 + 脏标记（`CachedKnownPeers`）：
  - 首次访问从文件加载到内存。
  - 后续合并新 peers 在内存中操作，设置 `dirty = true`。
  - 仅 `dirty` 时写入文件，写入后重置标记。
  - 避免每 5 秒轮询时重复读写文件。

## 7. RPC 健壮性与链指纹校验

- RPC 通过共享模块 `nodeui/backend/src/shared/rpc.rs` 发起（`rpc::rpc_post`），统一使用 `rpc::RPC_REQUEST_TIMEOUT` 作为请求超时，避免各模块分散定义导致不一致。
- 共享 RPC 客户端使用 `OnceLock<Client>` + 初始化互斥锁：
  - 首次成功后复用连接池；
  - 初始化失败不会缓存错误，后续调用会重试；
  - 初始化互斥保证并发下只会有一个线程执行初始化。
- RPC 调用统一具备：
  - connect + request timeout
  - 响应上限（4MB，含 Content-Length 预检查与流式读取限流）
  - HTTP 状态码检查（必须 200）
  - JSON-RPC `error` 显式报错
- 统计前先做链指纹校验：
  - `system_properties.ss58Format == 2027`
  - `system_name` 非空
  - genesis hash 与首次连接缓存一致（`shared::rpc::verify_genesis_hash`），且缓存/比对前都要求满足 `0x` + 64 位十六进制格式
- 任一指纹项校验失败时不信任网络统计，返回告警并降级输出。

## 8. 告警策略

以下场景会写入 `warning`：
- known-peers 读写失败
- RPC 指纹校验失败
- `system_peers/system_localPeerId` 读取失败或格式异常
- 收到无效 peerId
- known-peers 超限被截断

`warning` 采用合并文本（中文分号分隔），前端直接展示。

## 9. 依赖关系

- 依赖 `home/process` 的 `current_status` 获取本机运行状态。
- 依赖 `settings/bootnodes-address` 的创世节点元数据。
- 依赖 `shared/security` 提供应用数据目录。
- 依赖 `shared/rpc::verify_genesis_hash` 进行 genesis hash 校验。
