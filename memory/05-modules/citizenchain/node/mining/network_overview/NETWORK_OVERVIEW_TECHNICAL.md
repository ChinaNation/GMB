# Mining Network Overview 模块技术文档

## 0. 功能需求

- 页面网络分组只保留“治理节点”“在线节点”两张并排卡片，不再为全节点、轻节点建立独立卡片。
- 治理节点卡片内部展示国家储委会、省储委会、省储行三个子计数。
- 在线节点卡片内部展示在线节点、全节点、轻节点三个子计数，并与治理节点卡片使用相同的三列信息层级。
- 模块需要优先基于本机节点当前可观测到的 `system_peers` 与 `system_localPeerId` 生成在线网络统计。
- 当本机节点正在运行时，模块需要把本机节点计入在线节点，并尽量识别其轻节点/全节点角色。
- 前端会定时轮询该接口，模块需要支持高频读取。
- 当 RPC 不可用、链指纹不匹配、PeerId 非法或部分字段缺失时，模块需要返回尽量可展示的统计结果，并通过 `warning` 告知降级原因。
- 全节点数按实测 peer 统计；轻节点数按链上已注册的有效 CID 数统计；在线节点数恒为两者之和。
- 模块需要避免把错误链或错误端口上的 RPC 数据误当作目标网络统计结果。

## 1. 模块位置

- 后端路径：`node/src/mining/network_overview.rs`
- 前端入口：`node/frontend/mining/NetworkInlineSection.tsx`
- 前端 API/类型：`node/frontend/mining/api.ts` 与 `node/frontend/mining/types.ts`
- 对外命令：
  - `get_network_overview`

## 2. 模块职责

- 输出网络总览统计：
  - 治理节点（国家储委会/省储委会/省储行）
  - 在线节点
  - 全节点/轻节点数
- 在数据不完整或链 RPC 异常时返回告警信息（`warning`）。

前端展示边界：

- 网络分组顶层固定为两列：左侧治理节点，右侧在线节点。
- 两张卡片都使用三列子网格，数字在上、节点类型在下。
- 全节点和轻节点只作为在线节点卡片的分类计数展示，不得恢复为独立顶层卡片。

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
- 实测 peer：本机 `system_peers`，外加对全部引导节点 `http://<domain>:9944` 并发发起的
  远程 `system_peers`（单次 3 秒、总超时 5 秒），按 peerId 去重合并。
- 本机节点状态：`src/home/identity::current_status` + `system_localPeerId` + `system_nodeRoles`。
- 轻节点数：链上 `CitizenIdentity::CidCount`，一次 `state_getStorage` 定长读。

## 5. 核心规则

1. 实测 peer 分两个互斥集合，都按 peerId 去重：
   - `roles` 含 `light` 的进 light 集合，其余进 full 集合。
   - 本机自身按 `system_localPeerId` + `system_nodeRoles` 并入对应集合；
     角色判不出来时**不猜**，写告警且本机不计入（宁可少计一台，不归错类）。
2. `full_nodes` = full 集合大小。
   - light 集合不再对外显示，但必须继续维护：它负责把轻节点 peer 排除在全节点之外。
     删掉它，CitizenApp 的 smoldot 会被算成全节点。
3. `light_nodes` = 链上 `CitizenIdentity::CidCount`，即当前有效（`Active`）CID 数。
   - 存储键 = `twox_128("CitizenIdentity") ++ twox_128("CidCount")`，节点侧单测钉住字节口径；
     pallet 或 storage 改名后 `state_getStorage` 只会安静返回 null，没有这条测试就无从察觉。
   - 值按 8 字节 SCALE `u64` 解码，多余字节判非规范。
   - 它是链上注册量，不是 P2P 观测量：一个 CID 可多设备、也可长期离线。
4. `online_nodes` = `full_nodes + light_nodes`。
5. 治理节点分类：
   - 按创世引导节点配置中的 `role` 精确匹配（`nrc`/`prc`/`prb`），
     只在两个实测集合的并集里匹配，与 `full_nodes` 同源。
6. 已删除统计：
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
- `system_nodeRoles` 读不到，本机轻/全节点无法判定
- 收到无效 peerId
- 引导节点角色未命中“国家储委会/省储委会/省储行”
- 链上 `CidCount` 键不存在（runtime 升级尚未落链）或读取失败，轻节点数按 0 显示

`warning` 采用合并文本（中文分号分隔），前端直接展示。

轻节点读不到时按 0 显示并配告警，不做静默兜底：数字为 0 的原因必须能从卡片下方看到。

## 9. 依赖关系

- 依赖 `home/identity` 的 `current_status` 获取本机运行状态；该状态会识别 `lock_held` 和 `exited`，网络总览只在 `running=true` 时计入本机节点。
- 依赖 `settings/bootnodes_address` 的创世节点元数据。
- 依赖 `shared/rpc::verify_genesis_hash` 进行 genesis hash 校验。
- 依赖链上 `citizen-identity` 的 `CidCount` 存储（占号 +1 / 吊销 −1），
  详见 `memory/05-modules/citizenchain/runtime/misc/citizen-identity/CITIZEN_IDENTITY_TECHNICAL.md`。
