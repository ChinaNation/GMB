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

- 正式链只有一条，2026-07-10 用户确认节点链高为 `#32`。
- 当前 `assets/light_sync_state.json` 的 `finalizedBlockHeader` 是创世块 `#0`：parent hash 全零，SCALE compact block number 为 `0x00`。
- smoldot 的 `warp_sync_minimum_gap=32`，只有远端 finalized **严格大于**本地锚点高度加 32 才发起 warp。因此从 `#0` 出发时，`#32` 仍走普通短链同步，最早从 `#33` 开始尝试 warp。
- CitizenChain 所有节点都注册 GRANDPA 网络协议，并挂载 `warp_proof::NetworkProvider`；权威节点运行 voter，普通节点运行 observer。
- 节点每次 finalized 推进都会覆盖保存最新 GRANDPA justification；authority set 切换块的 header 与 justification 会单独持久化。当前节点模式只允许归档模式，生产 service 使用 `--state-pruning archive`，默认 block pruning 保留 finalized 正典链，满足 proof 生成所需数据。

## 新安装用户唯一同步路径

```text
签名 App 内置 chainspec + finalized checkpoint
  → 校验 chain id / protocol id / genesis state root
  → 连接多个公开归档节点
  → 高度差 ≤ 32：普通短尾同步
  → 高度差 > 32：请求 GRANDPA warp proof
  → 验证历史 authority set 交接和最终性签名
  → 下载目标 finalized 块 runtime / 必要 storage proof
  → 普通同步少量近头区块
  → 导出本机 finalized database，后续启动增量恢复
```

该路径的历史同步成本主要随 GRANDPA authority set 变更次数和 proof 体积增长，不随普通区块总高度线性增长。warp proof 单次响应上限为 8 MiB，超过后可分段继续。

## checkpoint 发布规则

1. `chainspec.json` 的 chain id、protocol id、genesis state root 和创世链身份永久冻结。
2. `light_sync_state.json` 是轻客户端 finalized 信任锚，不参与 genesis hash；当前正式资产是 `#0`。
3. 后续正式发布 CitizenApp 时，从受控归档节点导出当时已经 finalized 的 `lightSyncState`，再用上一发行锚点启动无缓存 smoldot 验证候选 finalized 链，并与至少 3 个独立归档节点公布的 finalized hash 交叉一致后，才写入签名安装包。新安装用户优先使用该发行锚点，降低长距离 warp 的长期权威集信任窗口。
4. 已安装用户的本机 finalized database 如果高于安装包锚点，smoldot 使用本机更高进度；App 更新不得让同步高度倒退。
5. Cloudflare 启动清单只允许更新经本地链身份校验的 bootnodes，不得把 Worker URL + Worker 同源 SHA-256 下发的动态 checkpoint 当作链信任锚。
6. 即使旧版本 App 的内置锚点较旧，只要仍与同一 genesis 绑定且公开节点能提供完整 proof，smoldot 仍通过 warp 接近链头；不得回退到 HTTP RPC 或下载全节点数据库。

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

- `light_sync_state.json` 不参与 genesis hash；它是轻节点启动锚点资产。
- checkpoint 可以落后于最新 finalized block；高度差超过 32 时 smoldot 自动选择 GRANDPA warp，否则同步短尾。
- 正式链刚冻结且高度仍为 0 时，CitizenChain PoW 允许使用创世头 checkpoint；
  收编 smoldot fork 只对无 BABE epoch 的 PoW 链放开该路径，BABE 链仍拒绝 genesis checkpoint。
- App 不保留“缺 checkpoint 时从 genesis 冷启动”的分支，因为 `stateRootHash` 轻形态没有完整创世存储，缺 checkpoint 必然不能加入链。
- 旧 full spec checkpoint 响应已废弃；完整 plain spec 会超过 RPC 响应大小限制。
- bootnode 只负责发现和承载 P2P 协议，不等于可信 checkpoint。客户端必须自行验证 warp fragment、authority set 交接、runtime 和 storage proof。
- GRANDPA finalized 必须持续推进；只有 best 增长而 finalized 停滞时，warp 无法把新安装用户带到近头。
- 公开服务节点必须持续保留 finalized 正典 header、authority set 切换块 justification、最新 best justification 和目标 finalized state。未来实现普通剪裁节点时，不得让缺失这些数据的节点承担新安装用户 warp 服务。
- 至少应有 3 个彼此独立的公开归档节点长期在线并支持 GRANDPA warp；单个 bootnode 可连接不代表快速同步服务具备可用性。
- smoldot 自身说明 GRANDPA warp 存在长程攻击信任窗口。因此发行锚点要随正式签名 App 发布推进，但不得改成运行时静默信任单一远端服务。

## 验收

- `bash citizenapp/scripts/check-chainspec-frozen.sh` 通过。
- 打包 APK 内 `assets/flutter_assets/assets/light_sync_state.json` 非空且含 `finalizedBlockHeader` / `grandpaAuthoritySet`。
- 真机日志出现 `已注入 lightSyncState checkpoint`、`轻节点已启动`、`区块头同步完成`，且不再出现 addChain checkpoint 错误。
- 当前 `#32` 只能验收普通短链同步。真实 warp 验收必须等正式链 finalized 至少达到 `#33`，使用全新 App 数据或独立测试 profile（无 `smoldot_db_cache`），确认出现 warp request、fragment 验证、高度跳转、近头追赶和本机数据库落盘。
- warp 验收同时记录连接节点、内置锚点高度、远端 finalized、warp 目标高度、proof 分段数、总耗时、峰值/稳定 CPU、Flutter 输入响应和失败原因；仅看到最终高度一致不算完成。
- 运维验收必须覆盖：单个节点下线、单个节点返回坏 proof、authority set 发生一次正常治理切换、节点重启后仍能从旧发行锚点生成 proof。
