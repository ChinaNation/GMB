# smoldot 轻节点 checkpoint 冻结方案

> 状态：当前方案
> 日期：2026-07-04
> 模块：CitizenApp + CitizenChain node + 冻结脚本

## 问题

CitizenApp 的 `assets/chainspec.json` 是 `genesis.stateRootHash` 轻形态，不携带完整创世存储。
smoldot 在这种形态下必须同时拿到 `lightSyncState` checkpoint；如果
`assets/light_sync_state.json` 为空对象或缺字段，启动会在 `addChain` 阶段失败，典型错误为
`ChainSpecNeitherGenesisStorageNorCheckpoint`。

## 当前唯一流程

1. `citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>` 启动临时节点物化创世。
2. 临时节点 RPC `sync_state_genLightSyncState` 只返回小体积 checkpoint：
   - `finalizedBlockHeader`
   - `grandpaAuthoritySet`
3. bake 脚本同步写入三份冻结资产：
   - `citizenchain/node/chainspecs/citizenchain.plain.json`
   - `citizenapp/assets/chainspec.json`
   - `citizenapp/assets/light_sync_state.json`
4. CitizenApp 启动时只从本地 asset 读取 checkpoint，内存注入 chainspec，不向远端 RPC 动态拉取。
5. `citizenapp/scripts/check-chainspec-frozen.sh` 在 CI 和本地启动前强制校验：
   - `chainspec.json` 必须是 `stateRootHash` 轻形态；
   - `chainspec.json` 不得内嵌 `lightSyncState`；
   - `light_sync_state.json` 必须非空并包含两个 checkpoint 字段。

## 边界

- `light_sync_state.json` 不参与 genesis hash；它是轻节点启动锚点资产。
- checkpoint 可以落后于最新 finalized block；smoldot 会从该点继续追赶。
- 正式链刚冻结且高度仍为 0 时，CitizenChain PoW 允许使用创世头 checkpoint；
  收编 smoldot fork 只对无 BABE epoch 的 PoW 链放开该路径，BABE 链仍拒绝 genesis checkpoint。
- App 不保留“缺 checkpoint 时从 genesis 冷启动”的分支，因为 `stateRootHash` 轻形态没有完整创世存储，缺 checkpoint 必然不能加入链。
- 旧 full spec checkpoint 响应已废弃；完整 plain spec 会超过 RPC 响应大小限制。

## 验收

- `bash citizenapp/scripts/check-chainspec-frozen.sh` 通过。
- 打包 APK 内 `assets/flutter_assets/assets/light_sync_state.json` 非空且含 `finalizedBlockHeader` / `grandpaAuthoritySet`。
- 真机日志出现 `已注入 lightSyncState checkpoint`、`轻节点已启动`、`区块头同步完成`，且不再出现 addChain checkpoint 错误。
