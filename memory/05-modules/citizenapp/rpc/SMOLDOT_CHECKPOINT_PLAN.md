# smoldot 轻节点 checkpoint 与 GRANDPA warp 快速同步方案

> 状态：当前唯一方案
> 日期：2026-07-10
> 模块：CitizenApp + CitizenChain node + 冻结脚本

## 问题

CitizenApp 的 `assets/chainspec.json` 是 `genesis.stateRootHash` 轻形态，不携带完整创世存储。
smoldot 在这种形态下必须同时拿到 `lightSyncState` checkpoint；如果
`assets/light_sync_state.json` 为空对象或缺字段，启动会在 `addChain` 阶段失败，典型错误为
`ChainSpecNeitherGenesisStorageNorCheckpoint`。

## 当前事实

- 正式链只有一条，2026-07-10 已实测 `best #33 / finalized #33`。
- 当前 `assets/light_sync_state.json` 的 `finalizedBlockHeader` 是创世块 `#0`：parent hash 全零，SCALE compact block number 为 `0x00`。
- smoldot 的 `warp_sync_minimum_gap=32`，只有远端 finalized **严格大于**本地锚点高度加 32 才发起 warp。因此从 `#0` 出发时，`#32` 仍走普通短链同步，最早从 `#33` 开始尝试 warp。
- CitizenChain 所有节点都注册 GRANDPA 网络协议，并挂载 `warp_proof::NetworkProvider`；权威节点运行 voter，普通节点运行 observer。
- 节点每次 finalized 推进都会覆盖保存最新 GRANDPA justification；authority set 切换块的 header 与 justification 会单独持久化。当前节点模式只允许归档模式，生产 service 使用 `--state-pruning archive`，默认 block pruning 保留 finalized 正典链，满足 proof 生成所需数据。

## 新安装用户唯一同步路径

```text
签名 App 内置 chainspec + 固定 finalized checkpoint
  → 校验 chain id / protocol id / genesis state root
  → 连接多个公开归档节点
  → 高度差 ≤ 32：普通短尾同步
  → 高度差 > 32：请求 GRANDPA warp proof
  → 验证历史 authority set 交接和最终性签名
  → 下载目标 finalized 块 runtime / 必要 storage proof
  → 普通同步少量近头区块
  → 导出本机 finalized database，后续启动增量恢复
```

该路径的历史同步成本主要随 GRANDPA authority set 变更次数和 proof 体积增长，不随普通区块总高度线性增长。warp proof 单次响应上限为 8 MiB，超过后可分段继续。固定 checkpoint 无论是当前真实资产的 `#0`，还是将来一次性冻结为 `#32`，都不需要随着链高持续更新。

## 固定 checkpoint 规则

1. `chainspec.json` 的 chain id、protocol id、genesis state root 和创世链身份永久冻结。
2. `light_sync_state.json` 是轻客户端固定 finalized 信任锚，不改变链的 genesis；当前正式资产是 `#0`，其 header 的 Blake2-256 必须等于本链 genesis block hash，并作为本机 database 缓存的链身份校验值。本方案不要求随链高更新安装包 checkpoint。
3. 新安装用户连接 P2P 后，远端 finalized 与固定锚点高度差不超过 32 时最多普通验证 32 个块；高度差超过 32 时自动 GRANDPA warp 到当前 finalized 附近。因此链增长不会让首次安装退化为从固定锚点逐块追完整历史。
4. 已安装用户把 finalized database 保存为 `citizenapp.smoldot.database.v1` 严格信封；只有同 genesis 且 finalized 更高的候选可以覆盖。下次启动从本机更高进度继续，固定安装包锚点不会覆盖或拖低本机进度。
5. Cloudflare 启动清单 v2 只允许更新经本地链身份校验的 bootnodes，协议中不得出现动态 checkpoint 或轻同步资产下载字段。
6. 不得回退到 HTTP RPC、动态 checkpoint 或全节点数据库下载来解决链高增长问题。

## 创世冻结流程

1. `citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>` 启动临时节点物化创世。
2. 临时节点 RPC `sync_state_genLightSyncState` 只返回小体积 checkpoint：
   - `finalizedBlockHeader`
   - `grandpaAuthoritySet`
3. bake 脚本同步写入三份冻结资产：
   - `citizenchain/node/chainspecs/citizenchain.plain.json`
   - `citizenapp/assets/chainspec.json`
   - `citizenapp/assets/light_sync_state.json`
4. CitizenApp 首次主动访问链时只从本地 asset 读取 checkpoint，内存注入 chainspec，不向远端 RPC 动态拉取。
5. `citizenapp/scripts/check-chainspec-frozen.sh` 在 CI 和本地启动前强制校验：
   - `chainspec.json` 必须是 `stateRootHash` 轻形态；
   - `chainspec.json` 不得内嵌 `lightSyncState`；
   - `light_sync_state.json` 必须非空并包含两个 checkpoint 字段。

## 边界

- `light_sync_state.json` 不定义或改变 genesis；当前 `#0` header 只用于在启动前重新计算已经冻结的 genesis block hash，从而拒绝跨链本机缓存。
- checkpoint 可以落后于最新 finalized block；高度差超过 32 时 smoldot 自动选择 GRANDPA warp，否则同步短尾。
- 正式链刚冻结且高度仍为 0 时，CitizenChain PoW 允许使用创世头 checkpoint；
  收编 smoldot fork 只对无 BABE epoch 的 PoW 链放开该路径，BABE 链仍拒绝 genesis checkpoint。
- App 不保留“缺 checkpoint 时从 genesis 冷启动”的分支，因为 `stateRootHash` 轻形态没有完整创世存储，缺 checkpoint 必然不能加入链。
- 旧 full spec checkpoint 响应已废弃；完整 plain spec 会超过 RPC 响应大小限制。
- bootnode 只负责发现和承载 P2P 协议，不等于可信 checkpoint。客户端必须自行验证 warp fragment、authority set 交接、runtime 和 storage proof。
- GRANDPA finalized 必须持续推进；只有 best 增长而 finalized 停滞时，warp 无法把新安装用户带到近头。
- 公开服务节点必须持续保留 finalized 正典 header、authority set 切换块 justification、最新 best justification 和目标 finalized state。未来实现普通剪裁节点时，不得让缺失这些数据的节点承担新安装用户 warp 服务。
- 至少应有 3 个彼此独立的公开归档节点长期在线并支持 GRANDPA warp；单个 bootnode 可连接不代表快速同步服务具备可用性。
- GRANDPA warp 的信任从固定 checkpoint 内的 authority set 开始，逐段验证后续 authority set 交接和最终性签名；不得用单一远端服务返回的新 checkpoint 绕过这条验证链。

## 验收

- `bash citizenapp/scripts/check-chainspec-frozen.sh` 通过。
- 打包 APK 内 `assets/flutter_assets/assets/light_sync_state.json` 非空且含 `finalizedBlockHeader` / `grandpaAuthoritySet`。
- 真机日志出现 `已注入 lightSyncState checkpoint`、`轻节点已启动`、`链状态同步完成`，且完成日志必须同时包含 `mode=regular`、精确 request/fragment 计数与可读的 best/finalized；不再出现 addChain checkpoint 错误。
- 已安装用户缓存验收必须解码 `smoldot_db_cache`，确认 schema、genesis hash、finalized 高度/哈希和 database 正文完整；连续冷启动必须记录“已验证同步缓存信封”，并在 addChain 后有界等待 database 异步应用，从同一或更高 finalized 恢复。
- `#33` 真实 warp 已在 Pixel 8a 独立 managed test profile 执行：从固定 `#0` 进入 `warpFragments`，页面显示 peer 5、warp `#33`、peer finalized `#33`，约 4.05 秒后保存 finalized `#33` / 5,111-byte database；重启后严格验证信封并进入 best/finalized `#33` 的 ready 状态。
- 2026-07-10 已修复首次 warp 完成语义：原生 syncing 同时受 runtime near-head 与 sync mode 约束，Dart/App/UI 统一只在 `LightClientStatusSnapshot.isUsable` 时完成；任何 warp 阶段都不得开放业务或导出缓存。
- 修复版在仍为 `#33` 的正式链两次干净重跑均由普通同步抢先，结构化诊断精确记录 `requests=0 / fragments=0`；其中一轮 best/finalized 在约 12 秒到达 `#33`，但直到约 75 秒 `regular + syncing=false` 才进入 ready，证明高度追上不会再提前完成。非零计数须等正式链高度进一步增加、真实 warp 稳定触发后补验。
- warp 验收必须以结构化状态快照中的 `syncMode`、`startupFinalizedBlockNumber`、`highestPeerFinalizedBlockNumber`、`warpFinalizedBlockNumber`、`warpRequestCount`、`warpFragmentCount` 为证据；禁止通过高度突然变化或 UI 文案猜测 warp 已发生。
- warp 验收同时记录连接节点、内置锚点高度、远端 finalized、warp 目标高度、proof 分段数、总耗时、峰值/稳定 CPU、`cit-smol-*` 与 `cit-cap-*` 线程 CPU、Flutter 输入响应和失败原因；仅看到最终高度一致不算完成。
- 资源验收要求每个进程只有 2 个 Tokio worker 和 2 个 capability worker，Android nice 均为 5；capability 队列保持有界且不得出现无上限原生线程增长。
- 运维验收必须覆盖：单个节点下线、单个节点返回坏 proof、authority set 发生一次正常治理切换、节点重启后仍能从旧发行锚点生成 proof。

## `#33` 真实验收发现的问题与修复

- 历史故障：runtime near-head heuristic 已返回“非 syncing”时，sync service 仍处于 `warpFragments`。旧 `Chain.waitUntilSynced()` 因只看 `isSyncing` 提前返回并导出 database，页面却仍不能进入 ready。
- 当前修复：原生快照只在 runtime near-head 且 `syncMode=regular` 时返回非 syncing；Dart `ChainStatus`、wait、App operational 门禁、缓存导出和 Banner 轮询全部复用 `isUsable`，不再存在两套完成标准。
- 第二次启动通过：严格信封恢复后进入 `regular`，best/finalized 均为 `#33`。这证明 warp proof 和缓存正文有效，故障边界是首次会话完成判定/状态收口，而不是固定 `#0` 架构或 database 恢复。
- 已安装用户在 operational 后每分钟低频检查一次 finalized；只有高度严格推进才复用同一稳定导出和单调信封写入路径，同高度不重复导出，避免安装后缓存永久停在旧高度。
- 下一次真实 warp 验收仍必须同时满足：`isSyncing=false`、`syncMode=regular`、best/finalized 可读；结构化 profile 日志必须记录非零 `warpRequestCount/warpFragmentCount` 的精确值，禁止只凭阶段文案推断。
