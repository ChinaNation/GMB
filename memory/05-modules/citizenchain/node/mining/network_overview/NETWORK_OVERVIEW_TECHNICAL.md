# Mining Network Overview 模块技术文档

## 0. 功能需求

- 页面需要展示网络总览指标，包括治理节点、在线节点、全节点和轻节点。
- 治理节点卡片内部展示国储会节点、省储会节点、省储行节点三个子计数。
- 模块需要优先基于本机节点当前可观测到的 `system_peers` 与 `system_localPeerId` 生成在线网络统计。
- 当本机节点正在运行时，模块需要把本机节点计入在线节点，并尽量识别其轻节点/全节点角色。
- 前端会定时轮询该接口，模块需要支持高频读取。
- 当 RPC 不可用、链指纹不匹配、PeerId 非法或部分字段缺失时，模块需要返回尽量可展示的统计结果，并通过 `warning` 告知降级原因。
- 统计口径需要保持自洽：`onlineNodes`、`fullNodes`、`lightNodes` 的去重和本机计入口径要一致，避免出现在线节点已计入但 full/light 漏计的情况。
- 模块需要避免把错误链或错误端口上的 RPC 数据误当作目标网络统计结果。

## 1. 模块位置

- 后端路径：`node/src/mining/network_overview/mod.rs`
- 前端入口：`node/frontend/mining/NetworkInlineSection.tsx`
- 前端 API/类型：`node/frontend/mining/api.ts` 与 `node/frontend/mining/types.ts`
- 对外命令：
  - `get_network_overview`

## 2. 模块职责

- 输出网络总览统计：
  - 治理节点（国储会/省储会/省储行）
  - 在线节点
  - 全节点/轻节点数
- 在数据不完整或链 RPC 异常时返回告警信息（`warning`）。

## 3. 对外数据模型

- `NetworkOverview {`
  - `online_nodes`
  - `nrc_nodes`
  - `prc_nodes`
  - `prb_nodes`
  - `full_nodes`
  - `light_nodes`
  - `warning`
- `}`

## 4. 统计来源

- 创世引导节点列表：`settings/bootnodes_address`。
- 实时在线节点：`system_peers`。
- 本机节点状态：`home/home-node::current_status` + `system_localPeerId`。

## 5. 核心规则

1. 在线节点去重：
   - 先采集 `system_peers` 到 `online_peer_ids`（集合）。
   - 本机节点运行时优先插入 `system_localPeerId`；失败则按本机在线 `+1` 估算。
2. 分类统计：
   - 按创世引导节点配置中的 `role` 精确匹配（`nrc`/`prc`/`prb`）对在线 PeerId 分类。
3. 全节点统计：
   - 远端 light 节点按唯一 PeerId 去重。
   - 本机在线但角色检测失败或角色未知时默认计入 full（降级行为，不产生告警）。
   - `full_nodes + light_nodes` 与在线节点口径保持一致。
4. 已删除统计：
   - 不再输出总节点数字段，不再维护 `known-peers.json`。
   - 不再输出清算节点字段，清算节点业务统计不属于当前挖矿页网络卡片。

## 6. 已移除的历史持久化

- 旧版总节点数曾通过 `known-peers.json` 补齐历史已见 PeerId。
- 当前页面已经删除总节点数卡片，后端也不再读取或写入 `known-peers.json`。

## 7. RPC 健壮性与链指纹校验

- RPC 通过共享模块 `node/src/shared/rpc.rs` 发起（`rpc::rpc_post`），统一使用 `rpc::RPC_REQUEST_TIMEOUT` 作为请求超时，避免各模块分散定义导致不一致。
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
- RPC 指纹校验失败
- `system_peers/system_localPeerId` 读取失败或格式异常
- 收到无效 peerId

`warning` 采用合并文本（中文分号分隔），前端直接展示。

## 9. 依赖关系

- 依赖 `home/process` 的 `current_status` 获取本机运行状态。
- 依赖 `settings/bootnodes_address` 的创世节点元数据。
- 依赖 `shared/rpc::verify_genesis_hash` 进行 genesis hash 校验。
