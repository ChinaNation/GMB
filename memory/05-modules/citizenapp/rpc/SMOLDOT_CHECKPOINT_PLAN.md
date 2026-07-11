# smoldot 轻节点 checkpoint 与 GRANDPA warp 快速同步方案

> 状态：当前唯一方案
> 日期：2026-07-10
> 模块：CitizenApp + CitizenChain node + 冻结脚本

> 当前实施状态：统一 `H → warp → F` 改造的第 1～5 阶段已删除错误 watchdog
> 和服务层人工活动状态，完成 fragment/storage/call-proof 请求级 peer 归属、失败恢复、
> 六阶段原生真源、FFI/Dart 严格状态、唯一原生 `isUsable` 以及完整验证 finalized 与
> warp 目标分离，并完成固定 #0/本机 H 启动证明、非黏性 `isUsable`、稳定 F 落盘和
> 下一次 H 恢复闭环；ARM64-only Android 原生库、profile APK、新安装 `#0 → F`、
> 本机 H 等于 peer F 时零 warp 恢复、断网 fail-closed 和 P2P 恢复真机验收已完成。
> 正式链在本轮观察窗口没有产生更高 F，运行中真实 `H < F` 推进仍待补验，当前不得
> 按“整项修复完成”发布。

## 问题

CitizenApp 的 `assets/chainspec.json` 是 `genesis.stateRootHash` 轻形态，不携带完整创世存储。
smoldot 在这种形态下必须同时拿到 `lightSyncState` checkpoint；如果
`assets/light_sync_state.json` 为空对象或缺字段，启动会在 `addChain` 阶段失败，典型错误为
`ChainSpecNeitherGenesisStorageNorCheckpoint`。

## 当前事实

- 正式链只有一条，2026-07-10 本轮真机验收时为 `best #36 / finalized #36`。
- 当前 `assets/light_sync_state.json` 的 `finalizedBlockHeader` 是创世块 `#0`：parent hash 全零，SCALE compact block number 为 `0x00`。
- CitizenApp 收编版 smoldot 的 `warp_sync_minimum_gap=0`。只要 peer finalized `F` **严格高于**启动锚点 `H` 就走 GRANDPA warp；`F == H` 不发 warp。
- CitizenChain 所有节点都注册 GRANDPA 网络协议，并挂载 `warp_proof::NetworkProvider`；权威节点运行 voter，普通节点运行 observer。
- 节点每次 finalized 推进都会覆盖保存最新 GRANDPA justification；authority set 切换块的 header 与 justification 会单独持久化。当前节点模式只允许归档模式，生产 service 使用 `--state-pruning archive`，默认 block pruning 保留 finalized 正典链，满足 proof 生成所需数据。

## 新安装用户唯一同步路径

```text
签名 App 内置 chainspec + 固定 #0 finalized checkpoint
  → 校验 chain id / protocol id / genesis state root
  → 连接多个公开归档节点
  → peer finalized F > #0：请求 GRANDPA warp proof
  → 验证历史 authority set 交接和最终性签名
  → 下载 F 对应 runtime / 必要 storage proof
  → warp 完成后才用普通同步跟随 best 尾部和后续新区块
  → 导出本机 finalized database，后续启动增量恢复
```

该路径的历史同步成本主要随 GRANDPA authority set 变更次数和 proof 体积增长，不随普通区块总高度线性增长。warp proof 单次响应上限为 8 MiB，超过后可分段继续。安装包只固定 `#0`，不随链高更新，也不再设计发行版 `#32` 锚点。

已安装用户的唯一路径是：先严格验证 `citizenapp.smoldot.database.v1` 信封，再要求原生层实际选择 database 中的 finalized `H`（高度和 hash 均相同）。peer finalized `F == H` 时直接进入在线跟随；`F > H` 时从 `H` warp 到 `F`。网络追到 `H` 不能冒充缓存恢复成功。

## 固定 checkpoint 规则

1. `chainspec.json` 的 chain id、protocol id、genesis state root 和创世链身份永久冻结。
2. `light_sync_state.json` 是轻客户端固定 finalized 信任锚，不改变链的 genesis；当前正式资产是 `#0`，其 header 的 Blake2-256 必须等于本链 genesis block hash，并作为本机 database 缓存的链身份校验值。本方案不要求随链高更新安装包 checkpoint。
3. 新安装用户连接 P2P 后，只要 peer finalized 高于 `#0` 就自动 GRANDPA warp 到该 finalized；不允许普通同步与 warp 抢跑并提前改变验证锚点。
4. 无有效本机 database 时，addChain 后第一份原生快照必须证明 `source=bundledCheckpoint / startup=#0 / startupHash=genesisHash`；任一字段不一致立即释放 chain，禁止带着未知 H 继续。
5. 已安装用户把完整验证 finalized database 保存为 `citizenapp.smoldot.database.v1` 严格信封；只有同 genesis 且 finalized 更高的候选可以覆盖。恢复成功必须由原生快照证明 `source=localDatabase` 且启动高度/hash 等于信封。
6. smoldot finalized database 内部链信息使用显式共识类型格式 v2，PoW 必须编码为 `consensus=pow`。旧的无 PoW 标记正文不可兼容，严格清理后从 `#0` 重建一次。
7. Cloudflare 启动清单 v2 只允许更新经本地链身份校验的 bootnodes，协议中不得出现动态 checkpoint 或轻同步资产下载字段。
8. 不得回退到 HTTP RPC、动态 checkpoint 或全节点数据库下载来解决链高增长问题。

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
- checkpoint 可以落后于最新 finalized block；任何严格正高度差都选择 GRANDPA warp，零高度差不发 warp。
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
- Android APK 只允许包含 `lib/arm64-v8a/`；构建前后的 smoldot 必须为 ELF64/AArch64，LOAD segment 必须满足 16 KiB 对齐，禁止任何其他 ABI 的 native entry。
- 真机日志出现 `已注入 lightSyncState checkpoint`、`轻节点已启动`、`链状态同步完成`；完成日志必须同时包含 `phase=regular / usable=true`、启动来源/锚点、当前完整验证 finalized、请求数、proof 收到/验证/拒绝数、最后失败原因和可读 best/finalized。
- [x] Pixel 8a 私密测试空间的新安装路径精确证明 `source=bundledCheckpoint / startup=#0`，以 1 次 warp 请求完成到 peer F，proof `received=1 / verified=1 / rejected=0`，并保存 5,120-byte database。
- [x] 同设备强停冷启动精确证明 `source=localDatabase / startup=H`；peer F 等于 H 时 `requests=0`，连接 peer 后进入 `regular + usable=true`。
- [x] 断网冷启动保持 `peer=0 / peer_finalized=null / usable=false`，交易按钮禁用；启动清单不可用时只使用本地 chainspec，不回退 HTTP 链真源。恢复网络后 P2P 可重新进入 ready。
- [ ] 等待正式链自然产生更高 finalized 后，补验运行中 ready 撤销、`H → warp → F` 和新 F 单调落盘；没有真实 `H < F` 时不得用同高度重连冒充。
- 已安装用户缓存验收必须解码 `smoldot_db_cache`，确认 schema、genesis hash、finalized 高度/哈希和 database 正文完整；addChain 后的第一份原生快照必须证明 `source=localDatabase` 和相同高度/hash，不等待网络追高来冒充恢复。
- 2026-07-10 Pixel 8a / Android 16 已实测新用户等价路径 `#0 → warp → #36`：`requests=1 / received=1 / verified=1 / rejected=0 / lastFailure=null`，约 2–3 秒完成并在同一会话 ready 后保存 v2 database。
- 同机随后冷启动已实测旧用户路径：原生日志为 `Database starting at #36`，快照为 `source=localDatabase / startup=#36 / requests=0`；peer finalized 同为 `#36`，未发 warp，页面进入 `best/finalized #36` ready。
- warp 验收必须以结构化状态快照中的 `syncPhase / isUsable`、`startupFinalizedSource`、启动高度/hash、`currentVerifiedFinalized`、最高 peer finalized、`warpTargetFinalized`、三类当前活动请求数、累计请求数、proof 收到/验证/拒绝数及最后失败为证据；禁止通过高度突然变化或 UI 文案猜测 warp 已发生。
- warp 验收同时记录连接节点、内置锚点高度、远端 finalized、warp 目标高度、proof 分段数、总耗时、峰值/稳定 CPU、`cit-smol-*` 与 `cit-cap-*` 线程 CPU、Flutter 输入响应和失败原因；仅看到最终高度一致不算完成。
- 资源验收要求每个进程只有 2 个 Tokio worker 和 2 个 capability worker，Android nice 均为 5；capability 队列保持有界且不得出现无上限原生线程增长。
- 运维验收必须覆盖：单个节点下线、单个节点返回坏 proof、authority set 发生一次正常治理切换、节点重启后仍能从旧发行锚点生成 proof。

## `#36` 统一锚点验收发现的问题与修复

- 历史故障：runtime near-head heuristic 已返回“非 syncing”时，sync service 仍在执行 GRANDPA warp。旧 `Chain.waitUntilSynced()` 因只看 `isSyncing` 提前返回并导出 database，页面却仍不能进入 ready。
- 当前修复：Rust 原生快照直接输出唯一 `isUsable`；仅在有 peer、runtime near-head 且 `syncPhase=regular` 时为真。Dart 只消费并校验该值，`ChainStatus`、wait、App operational 门禁、缓存导出和 Banner 轮询全部复用同一字段，不再存在两套完成标准。
- `finalizedBlock` 是普通订阅视图，不能证明 warp 已经构建出完整 chain information。缓存信封和 finalized 业务读取统一使用 `currentVerifiedFinalizedBlockNumber/hash`；活动 warp 的 fragment 目标单独使用 `warpTargetFinalizedBlockNumber/hash`，两者禁止混用。
- 原生 database 序列化在 warp 活跃时直接关闭；Dart 同时要求 `isUsable=true`，并用导出前后相同的 `currentVerifiedFinalized` 夹住正文，禁止保存 H 冒充 F。
- Dart `_synced` 不是永久真相。每个业务入口都会重新读取原生 `isUsable`；运行期间 peer 的 F 推进并重新进入 warp 时，立即撤销 operational、停止缓存刷新并等待完整验证完成。
- PoW database 根因：旧序列化器没有保存 PoW 共识类型，导出正文虽然有 finalized header 和 GRANDPA 信息，解码时却被误判为残缺 BABE 数据。当前内部格式 v2 显式保存 `consensus=pow`，并有同文件 round-trip 回归测试。
- 第二次启动通过：严格信封和原生启动锚点双重校验后进入 `regular`，best/finalized 均为 `#36`，没有重复 warp。
- 已安装用户在 operational 后每分钟低频检查一次 finalized；只有高度严格推进才复用同一稳定导出和单调信封写入路径，同高度不重复导出，避免安装后缓存永久停在旧高度。
- 链从 `#36` 推进到 `#37` 后仍需补做已安装用户 `#36 → warp → #37` 的正高度差验收；当前链未到 `#37`，不得伪造该运行证据。
