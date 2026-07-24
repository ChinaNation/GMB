# RPC 模块技术文档（当前实现态）

> 说明：本文描述的是当前代码实现态。
> 当前实现已切为“PoW 专用轻节点 + 无 HTTP 回退”，以 [ADR-004](../../04-decisions/ADR-004-pow-light-client-without-http-fallback.md) 与 [POW 轻节点长期落地方案](./POW_LIGHT_CLIENT_ROADMAP.md) 为准。

## 1. 模块定位

`lib/rpc/` 是公民 与区块链节点通信的唯一收口模块。

职责：

- 默认通过 `smoldot` 轻节点接入 citizenchain P2P 网络
- 提供底层 JSON-RPC 调用能力
- 提供轻节点状态快照等逐步收口中的 typed capability
- 链上状态查询（余额、nonce、metadata 等）
- 链上交易构造与提交（转账、未来的投票/提案）
- 钱包交易流水增量监听：从 newHeads/finalizedHeads 读取 `System.Events`，只记录本机钱包开始跟踪后的余额变化

约束：所有链上通信必须通过本模块，业务模块不直接建立 RPC 连接。

补充说明：

- 当前 `smoldot` Dart 绑定已从 pub.dev 依赖切换为仓库内本地 fork：`citizenapp/smoldotdart`
- 当前 `smoldot-light` Rust 内核位于：`citizenapp/smoldotpow`
- 这两层收编的目的，是为后续 PoW 专用 typed capability 改造建立可控演进入口
- Android 唯一支持 64 位 ARM `arm64-v8a`。`scripts/build-smoldot-native.sh android` 只构建 `aarch64-linux-android` 并写入 `android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so`；APK 构建入口必须显式传入 `--target-platform android-arm64`。Gradle 的 ABI filter 只允许 `arm64-v8a`，并在 packaging 层排除插件携带的所有非 ARM64 native 库，禁止任何其他 Android ABI 进入最终 APK。
- macOS 桌面调试库只用于 Dart FFI / `flutter test` 本机验收。`scripts/build-smoldot-native.sh macos` 必须设置 `CARGO_PROFILE_RELEASE_STRIP=false`，否则 Rust release profile 的 `strip=true` 会导致 dyld 报 `mis-aligned LINKEDIT string pool`，OpenMLS native 测试会被误判为 native 库不可用。

## 2. 目录结构

```text
lib/rpc/
├── chain_tx_monitor.dart← 业务：钱包区块事件监听 + 本地流水写入
├── chain_rpc.dart       ← 底层：节点连接管理 + JSON-RPC 方法
├── onchain.dart         ← 业务：extrinsic 构造 + 转账 + 交易确认
├── rpc.dart             ← barrel export
└── RPC_TECHNICAL.md
```

## 3. 通信架构

```text
公民  --smoldot-->  bootNodes / P2P 网络
```

- 默认协议：`smoldot` 轻客户端 + Substrate JSON-RPC

## 4. chainspec 与轻节点模式

默认模式下，App 从 `assets/chainspec.json` 加载链规格，并使用其中的 `bootNodes` 加入网络。

当前要求：

- `chainspec.json` 必须与目标链的 genesis / properties / bootNodes 一致
- 如果打进 App 的 chainspec 错了，轻节点即使“连上了”，也可能连到错误链或错误引导节点
- `bootNodes` 的来源应以 `citizenchain/node/src/chain_spec.rs` 为准
- 正式创世后,CitizenApp 的 `assets/chainspec.json` 使用轻节点形态,只承载链身份、bootNodes 和 `stateRootHash`;不得内置全节点链数据库或 GB 级 raw state。
- `assets/public_institutions/` 从同一个 finalized 块直接读取 `PublicManage::Institutions` 与 `InstitutionAccounts` 后生成，manifest 必须包含 `snapshot_block_number / snapshot_block_hash / genesis_hash / state_root / public_institution_root / shard_hashes`。
- 公权机构唯一真源是链上 `PublicManage`；CitizenApp 内置快照和 Isar 只服务目录首屏，不是授权真源。身份、绑定、付款和权限操作必须精确读取 finalized storage。
- `assets/light_sync_state.json` 是安装包签名保护的 finalized 信任锚；当前锚点是创世块 `#0`，不是会随 Worker 响应静默变化的运行时配置。
- 安装包 checkpoint 永久固定 `#0`，不随链高更新。新用户只要 peer finalized 高于 `#0` 就 GRANDPA warp；已安装用户先从原生验证采用的本机 finalized database 高度 `H` 启动，peer finalized 高于 `H` 时再 warp。

## 5. 连接与同步策略

1. App 完成 `runApp()` 和首帧渲染后不自动初始化轻节点；只有主动链消费方首次调用时才进入 `SmoldotClientManager.ensureStarted()`
2. 轻节点读取 `SharedPreferences.smoldot_db_cache` 的 `citizenapp.smoldot.database.v1` 信封；schema、内置 `#0` 推导的 genesis hash、完整验证 finalized 高度/哈希和 database 正文先通过严格校验，再要求 addChain 的第一份原生快照证明 `source=localDatabase` 且启动高度/hash 与信封完全一致
3. 旧裸 database、损坏信封、未知字段、跨 genesis 数据或 smoldot 拒绝的缓存都会被清除，并回退到安装包固定 `#0`；回退后的第一份原生快照还必须证明 `source=bundledCheckpoint / startup=#0 / startupHash=genesisHash`，不保留未知 H、旧格式兼容或双轨读取
4. 主动链入口触发轻节点加入 `chainspec.json` 指定的 citizenchain 网络后，立即在后台预热同步
5. peer/best/finalized 进度展示只等待初始化；余额、nonce、finalized storage、extrinsic 和链事件订阅必须等待原生 `isUsable=true`。Dart 的历史 `_synced` 每次业务入口都会重新向原生确认，peer F 推进后不得继续沿用旧 ready
   - **界面用语与协议名词分开（2026-07-23）**：`ChainProgressBanner` 面向用户的文案已中文化 —— `peer N` → 「已连接节点 N」、`best #N` → 「最新区块 #N」、`finalized #N` → 「已验证区块 #N」、warp 分支 `peer finalized` → 「节点已验证区块」。**本文档及代码内部仍使用 peer / best / finalized 作为协议名词**，二者不得互相替换：UI 文案是给用户读的，协议名词对应 smoldot 的实际概念，混同会让技术描述失真。
6. 当轻节点未初始化、同步失败或链路降级时，typed capability 必须抛出真实错误，不能返回 `null` / `[]` / `{}` 伪装成“链上没有数据”
7. `SmoldotClientManager.ensureStarted()` 是唯一启动闸口：成功幂等、进行中复用同一 Future、失败后允许重试；`initialize()` 只保留为该闸口的对外别名
8. `dispose()` 必须异步等待 chain/client 释放；生命周期代际切换后，旧初始化、同步和后台重试不得再写回健康状态或清掉新 Future
9. `ChainEventSubscription.connect()` 必须等待启动与同步的真实结果；失败返回 false 供交易监控重试，禁止在异步初始化完成前假报连接成功

### 5.1 统一启动锚点与快速同步

- 当前 smoldot fork 已启用 GRANDPA warp，固定 `warp_sync_minimum_gap=0`。对启动锚点 `H` 和 peer finalized `F`：`F > H` 必须 warp，`F == H` 不发 warp。
- 新安装用户的 `H` 永远是签名安装包内的 `#0`。例如安装时正式链已到 `#100000`，客户端验证 GRANDPA warp proof 后直接建立 `#100000` finalized 链信息，不逐块验证 `#1..#99999`。
- 已安装用户的 `H` 是原生层实际从本机 database 恢复的 finalized。若本机为 `#36` 且 peer 仍为 `#36`，不发 warp；peer 到 `#37` 后从 `#36` warp 到 `#37`，warp 完成后才进入普通在线跟随。
- warp 先验证 GRANDPA authority set 交接与最终性 proof，再下载目标 finalized 块的 runtime 和必要 storage proof，随后切回普通同步追赶少量近头区块。成本主要随权威集变更与 proof 体积增长，不随普通区块高度线性增长。
- GRANDPA neighbor packet 到达前，GRANDPA 链不允许普通 block request 抢先改变同步锚点；warp 活跃期间也只调度 warp 请求，避免普通同步与 warp 竞态造成 fragment 被错误拒绝。
- 当前节点端已经为所有节点注册 GRANDPA 协议并挂载 warp proof provider；权威节点推进 finality，普通 observer 节点也能基于本地归档数据响应 proof。
- Cloudflare bootstrap v2 只补充通过本地 chain id、protocol id、genesis state root 校验的 bootnodes；协议中不存在远端 checkpoint 或轻同步资产下载字段。
- 2026-07-12 bootstrap v2 已重新发布到 staging（`692d472a-49ec-47e5-912d-51cf6e178545`）和 production（`418f3d65-ea13-4d40-a045-a66ba84822cc`），两端统一为 6 个已部署 bootnodes，旧节点和未部署节点不再下发。production 已真实验证 schema v2、无 checkpoint/RPC URL；staging 未登录请求返回预期 302 Access 跳转。
- warp 不可用时 smoldot 仍可能退化为普通逐块同步。App 必须把它视为可观测的服务降级，保持 Flutter 输入响应并告警节点运维，禁止改走 HTTP 链真源。
- 完整发布、节点数据保留和真实验收规则见 [checkpoint 与 GRANDPA warp 快速同步方案](./SMOLDOT_CHECKPOINT_PLAN.md)。

补充说明：

- 钱包余额不更新的首要风险点，不是 UI，而是“轻节点已初始化但尚未同步完成”时过早查询链上状态
- Android 系统弹出“公民没有响应/关闭应用/等待”属于 ANR，首要排查点是是否有非链页面误触发 smoldot，或同步原生线程持续挤占主线程资源；当前实现只在 `runApp()` 前等待旧实例释放，禁止等待新轻节点初始化或同步。
- 广场浏览、聊天页和“我的”身份徽章不得调用主动链入口；这些页面没有链消费行为时，进程内不得创建 smoldot client。
- `smoldot` 返回 JSON-RPC error 时必须抛出，不能把错误吞成 `null`，否则上层会把真实故障误判为余额为 0、没有提案或机构不存在
- 当前代码已新增 `SmoldotClientManager.getStatusSnapshot()`，作为结构化轻节点状态接口；其底层已改为 Rust 原生 capability，不再由 Dart 层拼装 `system_health`
- 状态快照的 `syncPhase` 是严格枚举：`regular`、`warpDownloadingFragments`、`warpVerifyingFragments`、`warpDownloadingTargetState`、`warpBuildingRuntime`、`warpBuildingChainInformation`。未知值直接拒绝，禁止降级成 ready。
- 快照分别携带启动 `H`、`currentVerifiedFinalizedBlockNumber/hash`、最高 peer finalized `F`、`warpTargetFinalizedBlockNumber`、已证明后才出现的目标 hash、fragment/storage/call-proof 当前活动请求数、累计 `warpRequestCount`、proof 收到/验证/拒绝数和最后失败。`finalizedBlock` 只代表普通订阅视图，不能代替完整验证 finalized。
- warp 活动真相必须来自内核：发现 peer finalized 高于可信锚点时进入 fragments 下载，收到后进入本地验证，随后依次下载目标状态、构建 runtime、构建 chain information；只有完整 chain information 成功且没有更高 peer finalized 时才进入 `regular`。服务层不得用请求历史、高度差或页面计时推测阶段。
- `isUsable` 由 Rust 原生层唯一计算：必须同时满足有 peer、runtime near-head、`syncPhase=regular`。Dart 只消费并校验该字段；原生字段相互冲突时直接抛出格式错误，不另造完成算法。
- database 序列化在原生 warp 活跃期间直接返回无 chain information；App 只有 `isUsable=true` 才发起导出，并要求导出前后 `currentVerifiedFinalizedBlockNumber/hash` 完全一致。同步失败和重试未完成路径不再调用“保存部分进度”。
- `ChainRpc` 的 finalized 缓存命名空间、runtime API 锚点和钱包确认高度统一读取 `currentVerifiedFinalized`；普通订阅 `finalizedBlock` 仅保留诊断展示，禁止进入业务 finalized 路径。
- 已删除跨 fragment、storage、runtime 和 chain-information 阶段的绝对 10 秒 watchdog；各网络请求只服从自身超时。fragment/storage/call-proof 以 request id 记录实际 peer，失败只处罚该请求 peer，取消或断连只清理命中记录并恢复重调度；虚假的“warp 无进展”失败枚举已删除。新版 Android ARM64 构建已完成；真实正高度差已发生，但运行中 H/F 验收因交易验证竞态触发 native crash 而失败。
- `ChainProgressBanner` 只展示原生轻节点阶段：fragments 下载/验证、目标状态下载、runtime 构建、chain information 构建和普通尾部同步各自使用独立文案；只有原生 `isUsable=true` 才显示“轻节点已就绪”。该状态不等同于某个业务页面的本地 Isar 写库成功，也不等同于所有链上 storage 查询已经完成。
- 既有真机记录已经证明固定 `#0` 会真实进入 GRANDPA warp 并生成可恢复的本机 database；此前 `Chain.waitUntilSynced()` 只看 runtime `isSyncing` 而提前返回的问题已删除。原生 `isUsable`、Dart `ChainStatus/wait`、App operational、缓存导出和 Banner 现在使用唯一完成语义。
- profile 结构化日志会在同步阶段变化与最终完成时输出 `phase/usable/source/startup/peer_finalized/current_verified/warp_target/active_requests/requests/received/verified/rejected/last_failure/best/surface_finalized`。第 1～5 阶段代码、本机分层自动化和 ARM64-only APK 静态验收已经完成；真实设备状态仍待后续阶段验收。
- 2026-07-11 ARM64-only profile APK 已重建：所有 native entry 只位于 Android 官方 ARM64 ABI 目录，smoldot 为 ELF64/AArch64 且 LOAD segment 以 16 KiB 对齐；APK 内固定 `#0` 的 `light_sync_state.json`、zipalign 和 v2 签名检查均通过。
- 同日 Pixel 8a 私密测试空间已真实验证新安装 `bundledCheckpoint/#0 → warp → peer F`，请求与 proof 计数为 `1/1/1/0`；随后 5,120-byte database 冷启动精确恢复 `localDatabase/H`，当 peer F 等于 H 时请求数为 0。断网冷启动保持 `peer=0 / usable=false` 并禁用交易，启动清单失败只回到本地 chainspec；恢复移动数据后 P2P 重新 ready。
- 正式链随后产生真实 `F > H`。原进程运行中追高时出现 `requests=3 / received=3 / verified=0 / rejected=3 / lastFailure=blockNumberNotIncrementing`，并在交易验证 future 调用 `pin_pinned_block_runtime()` 返回 `BlockNotPinned` 后执行 `unreachable!()`；ARM64 精确 linker map 将 panic location 定位到 `transactions_service.rs:1493`。由于 release 为 `panic=abort`，整个 App 发生 native `SIGABRT`。Android 重启进程后从已保存 H 以一次请求成功追到 F，只能证明重启恢复，不能视为运行中验收通过。
- 根因修复必须保持职责边界：`BlockNotPinned` 是 finalized 推进使旧验证锚过期的瞬态竞态，交易服务应结束旧 validation future，并让仍待处理的交易基于当前 best block 重新验证；不得 panic、不得把单块过期升级为整条 subscription 失效、不得错误丢弃交易。修复后需重新执行既有 smoldot-light、Rust FFI、CitizenApp RPC/交易页、smoldotdart、analyzer、ARM64 APK 与旧 ABI 残留检查，并以有待处理交易时的下一次真实 finalized 推进作最终验收。
- operational 后由单实例一分钟定时器低频检查 `currentVerifiedFinalized`。只有快照仍可用且该完整验证 finalized 严格高于最近持久化高度时才进入既有串行稳定导出；同高度不导出，dispose 取消定时器并等待刷新/写队列。下一次业务入口若发现原生重新进入 warp，会立即撤销本地 ready，完成后保存的新 F 成为下一次启动 H。
- 连接诊断必须以有效 peer、best/finalized 状态是否可读或推进为准；未部署 bootNodes 的连接失败日志不是故障根因，不得把它解释成 citizenapp 网络不可用。
- 本地开发期 `30334` bootnode 只是可选调试兜底，不是 citizenapp 真机连接区块链网络的必要条件；没有本地 `30334` 也不应判定为连接异常。
- Flutter widget test 环境不具备真实 smoldot 轻节点链路，`ChainProgressBanner` 在测试中只渲染静态提示条，禁止读取链状态和创建轮询定时器，避免 `pumpAndSettle` 被后台链路轮询卡住。
- 当前代码已继续下沉原生能力，且已完成 **异步 FFI 迁移**：
  - `smoldot_get_status_snapshot_async`
  - `smoldot_get_system_account_async`
  - `smoldot_get_storage_value_async`
  - `smoldot_get_storage_values_async`
  - `smoldot_get_runtime_version_async`
  - `smoldot_get_metadata_async`
  - `smoldot_get_account_next_index_async`
  - `smoldot_get_block_hash_async`
  - `smoldot_get_block_extrinsics_async`
  - `smoldot_submit_extrinsic_async`
  这些异步导出通过 `DartCallback` 回调模式返回结果，不阻塞 Dart 主线程。每个 native client 只创建 2 个 capability worker，并使用容量 64 的有界队列；队列满时显式返回 `native_capability_queue_full`，禁止为每次请求创建原生线程。
  旧的同步版本（不带 `_async` 后缀）已标记废弃，后续将删除。
  Dart 侧通过 `NativeCapabilityHandler`（`chain.dart`）统一管理异步回调注册
- 原生 Tokio runtime 固定使用 2 个 worker，线程名为 `cit-smol-0/1`；capability worker 为 `cit-cap-0/1`。Android 上四个线程统一设置 `nice=5`。该约束用于限制长链、warp 或异常分叉期间对 Flutter main/raster 的 CPU 竞争，不代表人为限制协议正确性或同步进度。
- Android logger 和 macOS logger 必须服从 Dart `maxLogLevel`。`maxChains / cpuRateLimit / wasmCpuMetering` 未被 Rust 实际消费，已经从 Dart 配置面删除；不得再用无效字段声称已完成资源限制。
- 当前代码已开始切换业务主路径：
  - ADR-017 后 `fetchBalance()`(best 视图)已删除；余额一律 `fetchFinalizedBalance()`
  - `ChainRpc.fetchFinalizedBalance()` / `fetchFinalizedBalances()` / `fetchFinalizedTotalBalance()` 统一走 finalized storage proof，页面金额展示只允许使用这些方法
  - ADR-017 后 `fetchConfirmedNonce()` 已删除；签名 nonce 走豁免区 `fetchNonce()`(accountNextIndex 池视图)
  - `ChainRpc.fetchStorage()` / `fetchStorageBatch()` 在轻节点模式下已改走原生 storage 读取
  - `ChainRpc.fetchRuntimeVersion()` / `fetchMetadata()` 在轻节点模式下已改走原生 capability
  - `ChainRpc.fetchLatestBlock()` 在轻节点模式下已改为复用状态快照中的 `bestBlock`
  - `ChainRpc.fetchFinalizedBlock()` 在轻节点模式下复用状态快照中的 `finalizedBlock`，钱包流水确认只允许使用该方法
  - `ChainRpc.fetchNonce()` / `fetchGenesisHash()` / `fetchBlockExtrinsicHashes()` / `submitExtrinsic()` 在轻节点模式下已切到原生 capability
- 2026-03-23 新增状态能力治理：
  - `smoldot_get_status_snapshot` 底层已不再经 `system_health` 包装，而是直接读取 `sync_service/runtime_service`
  - 本地 `smoldotdart` 的 `Chain.getInfo()` / `getPeerCount()` / `getStatus()` / `getBestBlock*()` 也已统一收口到原生 status snapshot
- 2026-03-23 新增链数据能力治理：
  - `smoldot_get_runtime_version` / `smoldot_get_metadata` 已改为读取 runtime service / runtime call，不再走 `state_getRuntimeVersion` / `state_getMetadata`
  - `smoldot_get_account_next_index` 已改为 runtime call `AccountNonceApi_account_nonce`，不再依赖 `system_accountNextIndex`
  - `smoldot_get_block_hash` 已改为“最近块缓存 + 当前同步视图”双层原生路径，不再保留 `chain_getBlockHash`
  - `smoldot_get_block_extrinsics` 已改为只走按 block hash 下载 block body 的轻节点原生路径，不再保留 `chain_getBlock` 兜底
  - `smoldot_get_storage_value` / `smoldot_get_storage_values` / `smoldot_get_system_account` 已改为只走 `sync_service.storage_query` proof 读取，不再保留 `state_getStorage` 兜底
  - `smoldot_get_finalized_storage_value` / `smoldot_get_finalized_storage_values` / `smoldot_get_finalized_system_account` 固定读取 finalized 状态根，供余额、总额、收益等金额展示使用
- 2026-03-23 本地探针验证结果：
  - `status`、`runtimeVersion`、`metadata`、`System.Account`、单个 storage、批量 storage、`accountNextIndex`、`genesisHash`、`block_extrinsics` 都已可在 smoldot 路径稳定读出
  - 对拍本地全节点后，`System.Account` 空账户返回 `null`，`:code` 与 `state_queryStorageAt` 的大值返回和全节点一致
  - 当前探针账户 Alice 在 dev 链上不存在，因此余额为空是链上事实，不是轻节点读取失败
  - 当前同一份轻量探针已不再出现 `system_health` / `state_getRuntimeVersion` / `chain_getBlockHash` / `chain_getBlock` / `state_getStorage` 的 legacy warning
  - 当前链上读取主路径已不再保留 legacy fallback；剩余工作主要是发布前真机验证与写路径持续治理

## 6. chain_rpc.dart — 底层 RPC 方法

### 6.1 余额查询

`ChainRpc.fetchFinalizedBalance(String accountId) → Future<double>`

1. 将 `accountId` 转为 32 字节 AccountId
2. 构造 `System.Account` storage key（见 6.5）
3. 轻节点模式通过 `smoldot_get_finalized_system_account_async` 异步走 finalized storage proof 读取
4. 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额
5. 分 → 元，返回 `double`

ADR-017 后已无 best 视图余额接口；所有余额读取 finalized(`fetchFinalizedBalance`/`fetchFinalizedBalances`/`fetchFinalizedTotalBalance`)。

`ChainRpc.fetchFinalizedTotalBalance()` 读取 finalized 块上的 `free + reserved`，用于钱包详情链上余额卡；钱包列表、治理机构详情、多签余额、转账页余额提示统一读取 finalized free 余额。

### 6.2 Nonce 查询

`ChainRpc.fetchNonce(String ss58Address) → Future<int>`

- 调用原生 `smoldot_get_account_next_index_async`，底层通过 runtime call `AccountNonceApi_account_nonce` 读取 `frame_system::Account.nonce`
- citizenapp 不允许缓存、自增、预占或回滚交易 nonce；所有 signed extrinsic 每次签名前都必须实时调用 `ChainRpc.fetchNonce`
- 返回值只用于本次 extrinsic 签名，不得作为投票是否成功的判断依据

### 6.3 运行时版本

`ChainRpc.fetchRuntimeVersion() → Future<RuntimeVersion>`

- 调用原生 `smoldot_get_runtime_version`

### 6.4 链信息查询

- `fetchGenesisHash() → Future<Uint8List>` — 创世块哈希（缓存）
  - 调用原生 `smoldot_get_block_hash(0)`，优先命中创世块快路径
- `fetchLatestBlock() → Future<({Uint8List blockHash, int blockNumber})>` — 最新块
  - 复用 `status snapshot.bestBlockHash/bestBlockNumber`
- `fetchFinalizedBlock() → Future<({Uint8List blockHash, int blockNumber})>` — 最新 finalized 块
  - 复用 `status snapshot.currentVerifiedFinalizedBlockHash/currentVerifiedFinalizedBlockNumber`
  - 钱包交易流水升级为 `finalized` 时只能使用该高度
- `fetchMetadata() → Future<RuntimeMetadata>` — 运行时 metadata（缓存，含 registry）
  - 调用原生 `smoldot_get_metadata`
- `fetchBlockExtrinsicHashes(int blockNumber) → Future<List<String>?>` — 区块 extrinsic 哈希列表
  - 先通过 `smoldot_get_block_hash` 解析块 hash，再通过 `smoldot_get_block_extrinsics` 下载 block body

### 6.5 Extrinsic 提交

`ChainRpc.submitExtrinsic(Uint8List encoded, {TxPoolWatchCallback? onWatchEvent}) → Future<Uint8List>`

- 调用原生 `smoldot_submit_extrinsic`
- 返回交易哈希 32 字节
- 返回交易哈希只代表 RPC 已接收交易，不代表已出块
- 可选 `onWatchEvent` 会接收后台 `author_submitAndWatchExtrinsic` 状态：
  - `ready / broadcast`：交易进入交易池或已广播
  - `inBlock / finalized`：交易被区块包含或最终化
  - `future / invalid / dropped / usurped / retracted / finalityTimeout / timeout / error`：交易未能按预期确认，业务页面必须停止“投票中”等待态并给出可操作提示；投票类业务必须回读投票引擎 storage，不能用交易 nonce 推断确认
- 业务层不得把 `txHash` 返回渲染为“投票成功”；链上投票是否生效必须继续读取对应完整票据 storage（如 `InternalVote::InternalVotesByTicket`）确认

`ChainRpc.submitExtrinsicAndWaitForInBlock(Uint8List encoded, {TxPoolWatchCallback? onWatchEvent})`

- 走 `author_submitAndWatchExtrinsic` 提交并等待 `inBlock / finalized` 状态
- 返回本地计算的交易哈希和入块状态中的区块哈希
- 用于提案创建和投票等必须确认链上执行结果的业务；调用方拿到区块哈希后必须继续读取 `System.Events` 或业务 runtime storage
- 该方法仍不等价于业务成功，业务成功由具体 pallet 事件或投票引擎 storage 决定

### 6.6 Storage Key 计算

`System.Account` 存储映射的 key 结构：

```text
prefix     = twox_128("System") + twox_128("Account")     // 32 字节，固定常量
accountKey = blake2_128(account_id) + account_id           // 48 字节（16 + 32）
fullKey    = prefix + accountKey                            // 80 字节
```

前缀常量（hex）：`26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9`

### 6.7 AccountInfo SCALE 解码

```text
偏移 0-3:   nonce (u32)
偏移 4-7:   consumers (u32)
偏移 8-11:  providers (u32)
偏移 12-15: sufficients (u32)
偏移 16-31: free (u128, 小端序)      ← 提取此字段
偏移 32-47: reserved (u128)
偏移 48-63: frozen (u128)
偏移 64-79: flags (u128)
```

### 6.8 币种单位

- 链上最小单位：分（fen），`Balance = u128`
- 显示单位：元（yuan），`100 fen = 1 yuan`
- `TOKEN_DECIMALS = 2`
- `TOKEN_SYMBOL = "GMB"`
- `EXISTENTIAL_DEPOSIT = 111 fen`（1.11 元）

## 7. onchain.dart — 链上交易操作

### 7.1 OnchainRpc 类

onchain 模块所有需要 RPC 的功能集中于此，供 `lib/onchain` 普通链上支付和 `governance` 等业务模块使用。

### 7.2 转账

`OnchainRpc.transferWithRemark(...)` — 完成完整转账流程：

1. 获取/缓存 metadata、genesisHash
2. 并行获取 runtimeVersion、runtime nonce、latestBlock
3. 解码目标 SS58 地址为 32 字节 accountId
4. 转换金额：`BigInt.from((amountYuan * 100).round())`
5. 校验转账备注不超过 99 UTF-8 字节，并构造 `OnchainTransaction::transfer_with_remark` call data（pallet_index=4, call_index=0）
6. 用 polkadart `SigningPayload` 构造签名载荷
7. 通过回调获取 sr25519 签名（rpc 模块与钱包模块解耦）
8. 用 polkadart `ExtrinsicPayload` 构造最终 extrinsic
9. 提交到节点，返回 tx hash

普通单账户链上转账唯一外部入口是 `OnchainTransaction::transfer_with_remark`；`Balances` 只作为 runtime 底层余额账本和内部 `Currency` 能力保留，公民 App 不得构造或广播 `Balances.transfer_*` 裸调用。

移动端 CI 与本机开发统一锁定 Flutter `3.44.4`，版本真源为仓库根目录 `.fvm/fvm_config.json`；CI workflow 不使用浮动 `channel: stable`。

### 7.3 Extrinsic 编码

利用 polkadart 0.7.1 内置的 `SigningPayload` 和 `ExtrinsicPayload`，通过 metadata 中的 registry 自动处理所有 signed extensions。

citizenchain 12 个 TxExtension：
- 标准扩展（`CheckMortality`/`CheckNonce`/`ChargeTransactionPayment`/`CheckMetadataHash`/`CheckSpecVersion`/`CheckTxVersion`/`CheckGenesis`）：polkadart 内置处理
- 自定义扩展（`AuthorizeCall`/`CheckNonZeroSender`/`CheckNonStakeSender`/`CheckWeight`/`WeightReclaim`）：metadata 中为 NullCodec，自动跳过

Call data 格式：`[pallet_index=2] [call_index=3] [0x00 + dest_32bytes] [compact_u128(fen)]`

### 7.4 普通转账提交后的确认口径

`OnchainRpc` 不再提供 nonce 轮询确认 API。普通转账提交后先写本机 `pending` 流水，交易池 watch 收到 included 后升级为 `inBlock`；`ChainTxMonitor` 只读取 finalized 高度的 `System.Events` 后升级为 `finalized`，禁止使用 best/latest block 作为已确认真源。

> 投票类交易同样不得使用 nonce 推进判定成功。内部投票必须读
> `InternalVote::InternalVotesByTicket`，联合投票必须读
> `JointVote::JointVotesByTicket`。机构查询键必须包含 CID、岗位码和钱包。

### 7.5 手续费估算

`OnchainRpc.estimateTransferFeeYuan(double amountYuan)` — 纯客户端静态方法，无需 RPC。

citizenchain 使用自定义 `OnchainChargeAdapter`，标准 `payment_queryInfo` 返回 0。客户端按链上相同逻辑计算：

- 费率：`Perbill(1_000_000)` = 0.1%
- 最低手续费：10 fen = 0.10 元
- 公式：`fee = max(amount_fen × 0.001, 10 fen)`
- 舍入：half-up 到 fen 精度（与 Rust `mul_perbill_round` 一致）

## 8. 已落地优化

### 8.1 同步状态缓存（已实现）

当前实现：

```text
首次主动链访问时：从内置 #0 header 推导 genesis hash
        → SharedPreferences.read('smoldot_db_cache')
        → 严格解析 citizenapp.smoldot.database.v1
        → addChain(chainSpec, databaseContent: envelope.database_content)
        → 原生快照必须证明 localDatabase + 信封高度/hash
        → peer finalized 更高则从本机锚点 warp，否则直接在线跟随

保存时：读取 finalized 高度/hash A
        → chainHead_unstable_finalizedDatabase
        → 读取 finalized 高度/hash B
        → A == B 才形成候选信封
        → 串行比较 persisted/candidate finalized
        → 只允许更高 finalized 覆盖
```

缓存信封唯一格式：

```json
{
  "schema": "citizenapp.smoldot.database.v1",
  "genesis_hash": "0x...",
  "finalized_block_number": 31,
  "finalized_block_hash": "0x...",
  "database_content": "..."
}
```

补充约束：

- database 导出、现有信封读取和 SharedPreferences 写入共用单一 Future 队列，不得并发落盘
- `addChain(databaseContent)` 返回后的第一份原生快照必须已经选择 database chain information。网络后来追到或超过信封高度不能冒充缓存恢复；来源、高度或 hash 任一不符都立即释放该 chain、清理缓存并从 `#0` 重建
- smoldot database 内部链信息只接受显式共识类型格式 v2；CitizenChain PoW 正文必须包含 `consensus=pow`。旧无标记正文不兼容，清理一次后由当前客户端重新导出
- 导出前后 finalized 发生变化时丢弃正文并最多重试一次；不得给正文绑定无法证明的高度或哈希
- 候选高度低于持久值时直接丢弃；同高度同 hash 不重写；同高度不同 hash 先清除无法信任的旧值，再写当前轻节点稳定导出的候选
- dispose 先递增生命周期代际并等待缓存队尾收口，新 client 启动前旧任务不得继续写缓存
- 缓存恢复失败时必须自动 `remove('smoldot_db_cache')` 后重试一次
- 不允许每次启动都主动清空缓存，否则会退化成“每次冷启动全量同步”
- 当前默认同步超时为 3 分钟；只有首次主动链访问加入网络后才会立即在后台预热同步
- 启动、同步重试和销毁都使用 Future 身份守卫；旧生命周期的异步完成不得覆盖新实例状态

### 8.2 批量余额查询（已实现）

当前实现：`ChainRpc.fetchFinalizedBalances(List<String> accountIds)`

新增 `ChainRpc.fetchFinalizedBalances(List<String> accountIds) → Future<Map<String, double>>`：

```text
1. 对每个 accountId 构建 System.Account storage key：
   key = SYSTEM_ACCOUNT_PREFIX + blake2b_128(accountId) + accountId
2. 调用 fetchStorageBatch(allKeys)(平名即 finalized) — 一次 finalized storage proof 请求
3. 对每个返回值：从 SCALE 字节 offset 16 读 u128 LE → ÷100 → yuan
```

`wallet_page.dart` 的 `_refreshBalancesFromChain()` 已改为一次调用
`fetchFinalizedBalances(accountIds)`，并在轻节点不可用时向用户展示统一错误文案，
而不是把失败静默吞成 0 余额。

ADR-017 后 `fetchBalances()`(best 批量)已删除；批量余额走 `fetchFinalizedBalances()`。

### 8.3 钱包交易流水监听（已实现）

当前实现：`ChainTxMonitor`

```text
钱包新建/导入本机
  → 建立 WalletTxSyncCursorEntity，起点为当前 finalized 区块
new head 到达
  → 读取该区块 System.Events
  → 优先解析 OnchainTransaction::TransferWithRemark
  → Balances::Transfer 只作为底层余额事件兜底
  → 命中本机钱包时写入/升级 LocalTxEntity(status=inBlock)
启动 / 订阅重连 / finalized 后
  → 补扫 finalized+1..best 的未确认区块
  → 命中本机钱包时写入/升级 LocalTxEntity(status=inBlock)
finalized head 到达
  → 按游标读取区块 System.Events
  → 优先解析 OnchainTransaction::TransferWithRemark
  → Balances::Transfer 只作为底层余额事件兜底
  → 命中本机钱包时写入/升级 LocalTxEntity(status=finalized)
```

约束：

- 不补扫导入前历史；删除钱包时删除本地流水和同步游标，再次导入从新的导入时刻重新记录。
- 收入写入正数 `amountDeltaFen`，支出写入负数 `amountDeltaFen`；业务方向由金额正负号推导，不保存 `direction`。
- `type` 只保存业务类型；区块事件记录唯一键为 `accountId:blockHash:eventIndex`，本机提交记录唯一键为 `accountId:pending:txHash`；写入时还要按同钱包、同区块、同发送方、同接收方、同转账本金做语义去重，防止 newHeads/finalized 重复处理同一事件。
- finalized 补同步只能使用 `currentVerifiedFinalizedBlockNumber/currentVerifiedFinalizedBlockHash`，不能使用普通订阅 finalized 或 `bestBlockNumber/bestBlockHash` 升级为 `finalized`；best 只允许用于补扫未确认区块并写入 `inBlock`。
- finalized 单轮最多补齐 120 个区块；未确认区块单轮最多补扫 32 个区块；若 `WalletIsar` 正在处理前台读写或本地库 busy，本轮直接让路并安排短延迟重试。
- 读取区块事件仍需要节点网络和处理器参与响应 RPC，因此 App 不做全历史扫描，避免增加全节点和手机端负担。

## 9. 依赖

- `polkadart`：RPC Provider、Hasher、SigningPayload、ExtrinsicPayload、RuntimeMetadata
- `polkadart_keyring`：SR25519 签名、SS58 地址解码
- `polkadart_scale_codec`：CompactBigIntCodec、ByteOutput

## 10. 错误处理

- 轻节点未同步完成：等待同步完成后再读链上状态；超时则抛出异常
- `smoldot` 返回 JSON-RPC error：直接抛出异常，禁止吞成空结果
- 账户不存在（`System.Account` / storage proof 返回空值）：返回余额 `0.0`，不报错
- 交易提交失败（`smoldot_submit_extrinsic` 返回错误）：抛出异常，由 service 层包装为 `OnchainPaymentException`

## 11. 调用方

| 模块 | 用途 | 状态 |
| --- | --- | --- |
| `wallet` | 余额查询（`ChainRpc.fetchFinalizedBalance`） | 已实现 |
| `wallet` | 钱包交易流水监听（`ChainTxMonitor`） | 已实现 |
| `onchain` | 普通链上转账（`OnchainRpc.transferWithRemark`，备注最多 99 UTF-8 字节） | 已实现 |
| `governance` | 提案/投票 | 规划中 |
