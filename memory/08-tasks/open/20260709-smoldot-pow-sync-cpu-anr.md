# 20260709 公民 ANR：smoldot 生命周期、同步与资源约束

## 任务目标

Pixel 8a / Android 16 曾出现 `Input dispatching timed out` ANR。任务目标是先切断非链页面无条件启动轻节点，再验证 warp、分叉与原生资源占用的真实边界，最终让链同步不能持续挤占 Flutter 主线程所需资源。

本卡是当前 ANR 修复任务的唯一任务卡，不再按旧的“fork 没有 warp，因此必然从创世逐块同步”结论实施。

## 已确认事实

1. 历史 ANR trace 显示进程处于高 CPU 状态；临时关闭 `main.dart` 的 smoldot 启动后，采样量和 CPU 显著下降，证明 smoldot 启动/同步管线是当次 ANR 的必要触发因素。
2. 用户在约 31 块的当前短链上重新实测，profile 包约 `0.5% CPU / 10s`，ANR 未复现。因此当前没有正在持续复现的短链故障，后续验收必须覆盖长链和分叉压力，不能拿短链无 ANR 代替完成。
3. `citizenapp/smoldot-pow` 已完整接入 GRANDPA warp sync，`sync/all.rs` 会创建 warp 状态机，`sync/warp_sync.rs` 明确支持 PoW；CitizenApp 唯一策略为 `warp_sync_minimum_gap=0`，任何严格更高的 peer finalized 都走 warp。
4. `citizenchain/node/src/core/service.rs` 已配置 GRANDPA `warp_proof::NetworkProvider`。原卡“fork 没实现 warp”的结论错误，已删除。
5. smoldot 的区块同步主要在 Rust/Tokio 内完成；现有证据不能证明“每验证一块都经 FFI 传给 Dart”，也不能仅按 DSO 百分比推导 FFI 放大倍数。FFI/回调是否构成额外压力需要单独采样。
6. 历史代码曾在首次同步超时和后台重试失败后调用 database 保存，但严格可用性门禁会拒绝正文；第 11 步第 4 阶段已删除这些无效调用，当前只保存完整验证 F。
7. smoldot 会比较本地 finalized database 与 chainspec checkpoint 并使用有效的本机进度；App 还必须由原生快照证明实际选择了 database 来源、高度和 hash，不能用网络追高冒充恢复。
8. 当前 runtime 源码仍为 `POW_INITIAL_DIFFICULTY=100`、`DIFFICULTY_ADJUSTMENT_INTERVAL=600`、`DIFFICULTY_MAX_ADJUST_FACTOR=4`；现有仓库证据不能证明分叉风暴已经通过 `setCode` 修复。

## 分步实施

### 第 1 步：轻节点生命周期基础

目标：不改变当前启动后预热行为，先消除初始化并发、初始化/销毁竞态和旧后台同步污染新生命周期。

- [x] `SmoldotClientManager.ensureStarted()` 合并进行中的初始化，成功幂等，失败允许重试。
- [x] `initialize()` 保留为统一启动闸口的对外别名。
- [x] `dispose()` 改为异步等待原生资源释放，并由 `main()` 在 `runApp()` 前等待。
- [x] 生命周期代际号阻止 dispose 前的初始化或同步任务写回新状态。
- [x] 同步与后台重试 Future 使用身份守卫，旧任务不得清掉新任务。
- [x] 删除 `TEMP-DIAG` 原生日志透传并恢复正式日志等级。
- [x] 新增并发合并、失败重试、dispose 后重启、旧初始化失效、销毁期间重启测试。
- [x] Pixel 8a profile 真实生命周期验收：连续三次冷启动均正常恢复 7,071-byte 同步缓存；同步耗时约 7–11 秒，同步后五组 CPU 采样为 0.0%–2.0%；同步期间连续注入滑动/点击后 MainActivity 仍为 resumed，无 ANR、输入超时、SIGABRT 或崩溃。
- [x] 全量 Flutter 测试基线已在第 2 步清理旧入口文案断言后恢复：487 通过、5 跳过、0 失败。

第 1 步静态与专项验证：`dart format` 通过；新增生命周期测试 5/5 通过；`flutter analyze` 无 error/warning，只有两个与本步无关的既有 info。

### 第 2 步：真正按需启动 + 身份徽章只读

- [x] 删除 `main.dart` 全局延迟预热。
- [x] 所有主动链入口统一等待 `ensureStarted()`；finalized 读取、提交和链事件订阅必须等待同步完成。
- [x] `ChainEventSubscription.connect()` 等待真实启动/同步结果；交易监控失败后单定时器重试，不假报连接成功、不堆叠并发连接。
- [x] 广场和“我的”身份徽章只读账户维度持久快照，未启动轻节点时不得拉起链；快照不参与发布、投票或权限判断。
- [x] 轻节点进入 operational 后由可取消监听按当前钱包触发一次后台刷新，不轮询。
- [x] 新增账户隔离、损坏快照、非正式身份档拒绝、我的页不启动、广场浏览不读链、发布页真实读链和订阅失败测试。
- [x] Pixel 8a profile 验收：广场冷启动 20 秒及“我的”停留 10 秒均无 bootstrap/client 创建；再次冷启动广场稳定期 10 秒 CPU 为 0.3%，进程中无 smoldot/Tokio 线程。进入交易后才唯一启动，约 5 秒同步至 best/finalized #31，同步期间连续输入无 ANR、输入超时、SIGABRT 或崩溃。

第 2 步静态与自动化验证：`dart format` 通过；专项测试 15/15 通过；全量 Flutter 测试 487 通过、5 跳过、0 失败；`flutter analyze` 无 error/warning，只有两个无关既有 info。

新增待第 3 步分层验证的观测：交易页显示“轻节点已就绪”后的两次 10 秒进程 CPU 为 4.7% 和 6.9%；线程瞬时采样中 smoldot/Tokio 为 0%，主要活动落在 Flutter main/raster。该数据未复现 ANR，也不能据此归因 smoldot；第 3 步需用同一窗口的进程、线程与原生采样拆分来源。

### 第 3 步：未来新安装用户快速接入设计与现网能力审计

- [x] 用户确认正式网络只有一条链，2026-07-10 节点链高为 `#32`；不再用另一条受控长链代替正式网络，也不清理现有用户数据制造测试条件。
- [x] 解码当前 App 资产：`light_sync_state.json` 的 finalized header 是创世块 `#0`。
- [x] 历史核对结果：上游默认门槛曾为 32；第 10 步已按唯一目标状态改为 0，当前规则是 `F > H` 必须 warp、`F == H` 不发 warp。
- [x] 核对客户端完整路径：优先请求并验证 GRANDPA warp fragments，随后获取目标 finalized runtime/必要 storage proof，再转回普通同步追近头；历史成本主要随 authority set 变更与 proof 体积增长，不随普通块高线性增长。
- [x] 核对节点服务：所有节点注册 GRANDPA 协议并挂载 `warp_proof::NetworkProvider`；每次 finalized 都覆盖保存最新 justification，authority set 切换块 justification 强制持久化。
- [x] 核对长期数据条件：当前只允许归档节点模式，生产 service 使用 `--state-pruning archive`，默认 block pruning 保留 finalized 正典链，可提供 warp 所需 header、justification 和目标 state。
- [x] 固化唯一目标架构：正式签名 App 永久内置同 genesis 的 `#0` finalized checkpoint；Cloudflare 只更新经链身份校验的 bootnodes。新用户 `#0 → warp → F`，已安装用户 `本机 H → warp → F`（仅当 `F > H`），完成后保存本机 finalized database。
- [x] 正式链 finalized 到达 `#33` 后已用独立 Android managed test profile 执行真实 warp；确认 fragments 阶段、启动 `#0`、warp 目标 `#33`、peer finalized `#33` 和缓存落盘。第 8 步发现的首次会话完成判定与计数可观测性已在第 9 步修复；由于正式链仍只有 `#33`，修复后的两次干净重跑均被普通短链同步抢先，真实 warp 修复回归需在链高进一步增长后继续验收。

第 3 步历史结论已由第 10 步收口：新安装用户依靠 GRANDPA warp，不逐块验证全链，也不下载全节点数据库；不再存在“等待明显大于 32 块高度差”的门禁。

### 第 4 步：原生资源约束

- [x] 修复 `AllSync::status()` 长期硬编码为普通同步的问题；当前已进一步细分为 fragments 下载/验证、目标状态下载、runtime 构建、chain information 构建和 regular，不得用高度差推测 warp。
- [x] 原生状态快照补齐启动 finalized、最高 peer finalized、warp finalized、warp 请求数和已验证 fragment 数；Dart 严格解析枚举和计数，未知模式直接报格式错误。
- [x] 交易页按真实阶段展示“快速验证最终性 / 加载最新链状态 / 同步尾部区块”，warp 尚未结束时不得误报“轻节点已就绪”。
- [x] Tokio runtime 固定为 2 个 worker；Android 原生工作线程统一设置 `nice=5`，降低同步与 Flutter 主线程争抢 CPU 的优先级。
- [x] 原生 capability 从“每次请求新建线程”改为每 client 固定 2 个 worker、容量 64 的有界队列；队列满时返回 `native_capability_queue_full`，不得无限创建线程或堆积任务。
- [x] Android/Rust 日志严格服从 Dart `maxLogLevel`，删除硬编码 trace；删除 Rust 未读取的 `maxChains / cpuRateLimit / wasmCpuMetering` 假配置，避免形成无效资源承诺。
- [x] 不新增独立 Dart isolate：同步工作本身在受约束的 Rust/Tokio 线程中执行，FFI 通过异步 callback 返回；当前证据不支持再引入 isolate 生命周期和消息复制成本。
- [x] 自动化验证：`smoldot-light cargo check`、FFI Rust `cargo check`、2 个资源约束单测、Dart 状态 codec/config 测试、8 个 Flutter 生命周期/交易页专项测试、macOS/Android 双 ABI 原生构建、arm64 profile APK 构建均通过；`flutter analyze` 只有两个既有无关 info。
- [x] Pixel 8a / Android 16 profile 真机验收：进程中恰好存在 `cit-smol-0/1` 与 `cit-cap-0/1` 四个线程，nice 均为 5；空闲 10 秒四线程 CPU tick 增量均为 0；连续 60 秒滚动/切页后由“同步尾部区块”进入“轻节点已就绪”，无 ANR、输入超时、崩溃、panic 或队列溢出。

第 4 步结论：资源约束和同步阶段类型已经落地。第 8 步已取得真实 fragments 请求证据并确认四个原生线程 nice 均为 5；第 9 步已统一完成语义，并把 request/fragment 精确计数写入 profile 结构化日志。正式链仍为 `#33` 时的修复版重跑均走普通同步，计数诚实输出为 0/0；真实 warp 的非零精确计数留待更高链高复验。

全量 Flutter 套件复跑说明：本轮单并发执行到 229 项后卡在既有 `widget_test.dart`“无钱包启动页”用例；单独运行确认其缺少 `org.citizenapp/security.isDeviceRooted` plugin mock，并非 smoldot 回归。第 4 步相关专项测试均通过，本任务未越界修改该既有测试。

### 第 5 步：缓存单调推进

- [x] `smoldot_db_cache` 改为唯一严格信封 `citizenapp.smoldot.database.v1`，记录 genesis hash、finalized 高度/哈希和 database 正文；旧裸格式不兼容，首次读取直接删除。
- [x] genesis hash 从安装包固定 `#0` 的 `finalizedBlockHeader` 计算，并强制 checkpoint block number 为 0；缓存链身份不依赖 Cloudflare 返回值。
- [x] 启动时严格拒绝损坏 JSON、未知/多余字段、错误类型、超限正文和跨 genesis 数据；smoldot 拒绝有效信封正文时同样清理并回退固定 checkpoint。
- [x] 真机确认 `addChain(databaseContent)` 返回瞬间仍可能显示安装包 `#0`；恢复校验改为最多等待 5 秒，低于声明高度继续等待，同高度必须 hash 一致，超过声明高度直接接受，持续低于或同高异 hash 才清理，避免误删有效异步恢复缓存。
- [x] finalized database 导出前后分别读取高度和哈希，只有锚点完全一致才组成候选；导出期间推进时丢弃并最多重试一次。
- [x] 所有导出和 SharedPreferences 写入进入单一 Future 队列；dispose 先失效代际并等待队尾收口，旧 chain/client 任务不得写入新生命周期。
- [x] 单调覆盖规则：更低 finalized 丢弃；同高度同 hash 不重写；同高度不同 hash 清除旧值后写入当前轻节点稳定候选；只有更高 finalized 正常覆盖。
- [x] 现有生命周期测试文件新增 7 个缓存场景，总计 13/13 通过；全 RPC 测试 31/31 通过，连同交易页专项为 33/33。`flutter analyze` 仍只有两个既有无关 info，`git diff --check` 通过。
- [x] Pixel 8a / Android 16 profile 保留数据覆盖安装验收：检测并删除旧裸缓存，从固定 `#0` 同步到 finalized `#31`，写入 schema/genesis/finalized hash 完整的约 7 KB 信封；最终实现再次覆盖安装并冷启动，真实接受 6,973-byte `#31` 信封，异步恢复后进入“轻节点已就绪”，无 ANR、输入超时或崩溃。

第 5 步结论：本机同步缓存现在只有一个受 genesis 绑定、finalized 单调推进的持久化真相。未来链高增加时，已安装用户从本机最高 finalized 恢复；新安装用户仍从固定 `#0` 进入 GRANDPA warp，两条路径最终都只写同一信封，不存在远端 checkpoint 或旧裸缓存双轨。

### 第 6 步：远端 checkpoint 残留清理

- [x] bootstrap schema 从旧版直接升级为 `citizenapp.chain.bootstrap.v2`；Worker 和 Dart 只接受 v2，不保留旧 schema 兼容。
- [x] Worker `light_client` 删除整个远端 checkpoint 字段树，只保留 `mode / truth_source / api_is_truth / bundled_assets_required`；删除相关常量、URL/摘要规范化函数和生成逻辑。
- [x] Cloudflare `Env` 与根、staging、production 三套 Wrangler vars 删除远端轻同步资产配置；启动清单只治理冻结链身份、公开 bootnodes 和服务发现。
- [x] Dart model 删除远端 checkpoint 属性和摘要解析器，严格校验 `light_client` 精确字段与两个签名安装包资产；任何 checkpoint 字段递归出现都直接拒绝。
- [x] Worker/Dart 测试更新为 v2，新增旧 schema 与远端 checkpoint 拒绝断言；不保留动态 checkpoint、HTTP RPC、全节点数据库下载或双轨分支。
- [x] 本地验收：Worker `tsc --noEmit` 通过、全量 Vitest 103/103 通过；全 RPC 与交易页 Flutter 测试 34/34 通过；profile arm64 APK 构建通过；`flutter analyze` 只有两个既有无关 info。
- [x] 真实 Worker 运行态验收：Wrangler 4.107.0 本地启动成功，`GET /v1/chain/bootstrap` 返回 200、schema v2、6 个 bootnodes，`light_client` 只有四个允许字段；旧 checkpoint 字段不存在，`/v1/chain/rpc` 返回 404。

第 6 步结论：代码、配置、测试和协议文档中只剩单一 bootstrap v2 契约。Worker 无法下发 checkpoint，App 也不会接受携带旧字段的响应；轻节点信任锚唯一来自签名安装包固定 `#0`，Cloudflare 只提供经过本地链身份校验后才使用的 bootnodes。第 6 步先完成本地实现与真实本地 HTTP 验收，远端发布记录见第 7 步。

### 第 7 步：Bootstrap v2 远端发布

- [x] Wrangler 4.107.0 对 staging、production 分别执行远端部署前 dry-run，确认入口、绑定和三套环境变量均可解析，旧 checkpoint 变量未进入部署绑定。
- [x] staging `citizenapp-square-api-staging` 从版本 `cb89091f-c393-4e66-a3ad-703bdca91844` 发布到 `ff19bc46-dc17-4f77-a53f-aed2739142a0`，100% 流量；真实 HTTPS 返回 schema v2、6 个 bootnodes、四字段 `light_client`，不存在远端 checkpoint/RPC URL，`/v1/chain/rpc` 为 404。
- [x] Pixel 8a 使用 staging base URL 的 arm64 profile 包保留数据覆盖安装：读取 v2 启动清单并注入 6 个 bootnodes，接受 finalized `#31` 的本机严格缓存信封，进入“轻节点已就绪”，无 CitizenApp ANR、输入超时或崩溃。
- [x] production `citizenapp-square-api` 从版本 `6bf9ecd1-4a35-4bba-b8d2-418ff657b529` 发布到 `00d836aa-9c43-4561-ba33-8730d780c1a0`，100% 流量；真实 HTTPS 契约与安全断言全部通过，通用链 RPC 路由仍为 404。
- [x] 重新构建不带 staging Dart define 的生产 arm64 profile 包并保留数据覆盖安装，确认设备已经恢复生产配置；真实读取 6 个 bootnodes、恢复 genesis 绑定的 finalized `#31` / 6,973-byte 信封并进入“轻节点已就绪”。日志中的一次 `FATAL EXCEPTION` 已定位为独立 `uiautomator dump` 进程重复注册，不属于 `org.citizenapp`；CitizenApp 无 ANR、fatal signal 或进程崩溃。
- [x] 本步骤没有执行 D1 migration、GitHub push、PR 或 CI/CD，也没有修改 `citizenchain/runtime/`；发布前版本均已留档，可按环境独立回滚。

第 7 步结论：bootstrap v2 已在 staging 和 production 全量生效，生产手机也已恢复到无测试 base URL 的正式 profile 配置。远端只下发公开 bootnodes 和服务发现，安装包固定 `#0` 仍是新安装用户唯一信任锚。正式链当前 finalized 仍为 `#31`，因此本轮只能证明 v2 发布和短尾/缓存恢复，不能冒充 `#33` 以后真实 warp 已验收。

### 第 8 步：正式链 `#33` 真实 Warp 验收

- [x] 正式链实时状态确认 `best #33 / finalized #33`，hash 为 `0xe3985a35f8668d74f1552be80e1e4c5c01fcce7f7c757cc0cf254ec21a1d2d9c`，满足固定 `#0` 锚点严格大于 32 的 warp 门槛。
- [x] Pixel 8a / Android 16 创建独立 managed test profile，单独安装同一生产 arm64 profile 包并创建临时测试钱包；机主 CitizenApp 和 Private Space 数据均未读取、清除或覆盖。
- [x] 为排除钱包创建后的 baseline 读链抢先写缓存，在测试钱包已持久化、首次 `[ChainRpc]` 刚启动而尚未保存缓存时停止测试进程；重启后广场无 client 创建，再由交易 Tab 唯一触发从固定 `#0` 启动。
- [x] 真实阶段证据：交易页显示“轻节点正在快速验证最终性”，结构化内容为 `peer 5 / 启动 #0 / warp 目标 #33 / peer finalized #33`；原生 `warp_request_count > 0` 已经证明发起了真实 GRANDPA warp request，而非按高度差推测。
- [x] 时间与缓存：`18:36:58.996` 轻节点启动，`18:37:03.043` Dart `waitUntilSynced()` 返回，约 4.05 秒；`18:37:03.120` 保存 genesis 绑定的 finalized `#33` / 5,111-byte database 信封。另一轮干净 profile 同样从 `#0` 到 `#33` 并保存 7,071-byte 信封，证明结果可重复。
- [x] 资源与响应：进程中只有 `cit-smol-0/1`、`cit-cap-0/1` 四个目标线程，nice 全为 5；采样窗口进程 CPU 最高约为单核 38%，收口后四线程 tick 不再增长、瞬时 CPU 为 0%；ADB 输入命令分别在 155/168 ms 返回，页面可继续切换，无 CitizenApp ANR、Input dispatch timeout、fatal signal、panic 或崩溃。
- [x] 缓存可恢复：第二次启动严格验证 finalized `#33` 信封，约 0.80 秒应用 5,111-byte database，最终显示“轻节点已就绪，peer 5，best/finalized #33”。
- [x] 历史首次会话完成判定已再次收口：第 11 步删除服务层人工 warp 布尔状态，改由 warp 内核根据 peer finalized、fragment 工作、目标状态、runtime 和完整 chain-information 状态直接决定精确阶段。
- [x] 历史 fragment 计数歧义已修复：当前分别记录 received / verified / rejected，并记录稳定枚举的最后失败原因。

第 8 步结论：固定 `#0` 的新安装用户确实会在链 finalized `#33` 时进入 GRANDPA warp，并能在约 4 秒内形成可在下一次启动恢复的 `#33` 本机 database；“所有新用户从创世逐块追链”的担忧已经排除。但首次会话的完成条件错误，当前用户必须重启 App 才能从 warp 状态进入可用状态，因此完整验收失败，任务不能关闭。后续已要求同步完成同时满足原生 `isUsable=true / syncPhase=regular`，并把 request/fragment 计数纳入诊断输出。

### 第 9 步：首次 Warp 完成语义与缓存持续推进

- [x] 原生 `ChainStatusSnapshot.is_syncing` 不再只等于 runtime near-head heuristic：只要 `syncPhase != regular` 或 runtime 尚未近头都保持 syncing，任一 warp 阶段不得提前完成。
- [x] `smoldot-dart` 统一以原生 `LightClientStatusSnapshot.isUsable` 映射 `ChainStatus` 和结束 `waitUntilSynced()`；阶段、syncing 与可用性相互冲突的快照直接拒绝。
- [x] App 在 Dart wait 返回后再次读取完整快照，只有 `regular + !isSyncing + peer/hash/高度完整` 才设置 operational、开放业务并导出 database；warp 阶段即使 runtime 已近头也不得写缓存。
- [x] `ChainProgressBanner` 按 `isUsable` 持续轮询到 ready，并在阶段变化时输出 `phase/usable/source/startup/peer_finalized/current_verified/warp_target/active_requests/requests/received/verified/rejected/last_failure/best/finalized` 精确结构化诊断；widget 测试覆盖 fragment 验证、chain information 构建和 regular，ready 后停止轮询。
- [x] 本机缓存增加单实例一分钟低频刷新：只在链可用且 finalized 严格高于已持久化高度时进入原有串行稳定导出路径；同高度不导出，dispose 会取消定时器并等待进行中的刷新/写队列收口。
- [x] 历史自动化验证：`smoldot-light`、CitizenApp Rust、Flutter 和 `chain_info_test.dart` 当时均通过；历史 Android 多 ABI 构建记录已经失效，当前发布规则只允许 `arm64-v8a`，必须在第 5 阶段重新构建并验收 ARM64 APK。
- [x] `smoldot-dart` 全量测试中的本地新增用例全部通过；两项既有 Westend 公开网络用例因默认 30 秒超时失败，实测 Westend 约 51 秒后才 ready，属于外部公开网络时序，不是 CitizenChain 完成语义回归。
- [x] Pixel 8a / Android 16 在独立 managed profile 保留生物识别、清空 App 数据后两次制造“钱包已持久化但无同步缓存”的干净状态。第一次从 `#0` 到 `#33` 约 6.1 秒并在同一会话显示 ready，随后保存 7,071-byte 信封；第二次在约 12 秒时 best/finalized 已到 `#33`，但门禁继续等待到约 75 秒才记录 `regular + syncing=false`、显示 ready 并保存 9,031-byte 信封，证明高度追上不会再提前开放业务。
- [x] 第二次启动严格验证 finalized `#33` 信封并恢复 7,071-byte database，约 3 秒完成链状态确认；运行超过一分钟且 finalized 未推进时没有重复导出或重写缓存。自动化测试另覆盖 `#31 → #33` 时只刷新一次。
- [x] 真机结构化日志完整输出精确字段；正式链仍只有 `#33`，两次修复版干净重跑均由普通同步抢先，故真实值为 `phase=regular, requests=0, fragments=0`，未伪造非零 warp 计数。第 8 步的真实 fragments 请求证据继续有效，但修复版非零计数和真实 warp 首次会话回归必须等链高进一步增加后补验。
- [x] 真机仅有 `cit-smol-0/1`、`cit-cap-0/1` 四个目标线程，nice 全为 5；稳定期进程瞬时 CPU 0.0%，页面切换后 MainActivity 仍 resumed，无 CitizenApp ANR、Input dispatch timeout、fatal signal、panic 或崩溃。

第 9 步历史结论中的 32 块门槛已由第 10 步废弃；完成语义和缓存持续推进仍保留。

### 第 10 步：统一 `#0 / 本机 finalized → warp → 最新 finalized`

- [x] 唯一策略改为 `warp_sync_minimum_gap=0`：启动锚点 `H` 与 peer finalized `F` 满足 `F > H` 时必须 warp，`F == H` 不发 warp。
- [x] 新用户启动锚点固定为签名安装包 `#0`；已安装用户启动锚点只能来自原生实际采用的本机 finalized database，并在快照中输出来源、高度和 hash。
- [x] GRANDPA neighbor 到达前禁止普通 block request 抢跑；warp 活跃期间只调度 warp 请求，解决普通同步先改变锚点导致有效 fragment 被判定为 `blockNumberNotIncrementing` 的竞态。
- [x] 历史实现曾用服务层人工布尔状态和绝对 10 秒 watchdog 收口；后续真实 `H → F` 验收证明 watchdog 会在 fragment 已验证后误杀合法后续阶段，该设计已在第 11 步第 1 阶段删除。
- [x] proof 可观测性拆为 request、received、verified、rejected 与稳定枚举的 last failure；删除含义不实的单一 fragment 计数。
- [x] 缓存恢复改为即时强校验：必须为 `localDatabase` 且启动高度/hash 等于信封；不再等待网络实时 finalized 追到信封高度。
- [x] 修复 PoW finalized database 无法恢复的根因：内部链信息格式升级为 v2，显式编码 `consensus=pow`；旧无标记正文不兼容，清理一次后从当前客户端重建。新增同文件 PoW round-trip 测试。
- [x] Pixel 8a / Android 16 profile 实测新用户等价路径：`#0 → warp → #36` 在约 2–3 秒内完成，精确结果为 `requests=1 / received=1 / verified=1 / rejected=0 / lastFailure=null`，同一会话进入 ready 并保存 5,120-byte database。
- [x] 同机冷启动实测已安装用户路径：原生 `Database starting at #36`，快照 `source=localDatabase / startup=#36 / requests=0`，peer finalized 同为 `#36`，没有重复 warp，最终 `best/finalized #36` ready。
- [x] 自动化回归通过：smoldot-light 单测/doctest、warp 零门槛测试、PoW database round-trip、Rust FFI 4 测试、Flutter RPC/交易页/banner 36 测试、smoldot-dart 10 测试；目标 Flutter analyze 无问题。smoldot-dart 仅保留仓库既有 info 级 lint。
- [ ] 正式链推进到 `#37` 后补验已安装用户 `#36 → warp → #37`；当前链仍为 `#36`，本步不伪造未来高度证据。

第 10 步的“实现完成”结论已作废。后续真实 `H → F` 复验稳定重现 fragment 已验证但被绝对 watchdog 中断，任务继续保持 open。

### 第 11 步：严格统一 `H → warp → F`

- [x] 第 1 阶段：warp 内核新增唯一活动判定。发现 peer finalized 高于当前 warp header 时立即进入 fragments；最终 fragment 验证后进入 chain-information；只有完整可信 chain information 切换完成且没有更高 peer finalized 时才返回 idle/regular。
- [x] 第 1 阶段：删除服务层人工 warp 状态、全局最近 peer、绝对 10 秒计时器、错误的无进展断连和请求创建前的 regular 窗口。
- [x] 第 1 阶段：warp required/active 时继续只调度 warp 所需请求，普通 finalized block request 不得抢跑；warp 完成后才恢复普通 best/non-finalized 尾部跟随。
- [x] 第 1 阶段自动化验证：零门槛策略与活动状态 2 项测试、smoldot-light 2 项单测和 2 项 doctest、Rust FFI 4 项测试全部通过；rustfmt、`git diff --check` 通过且无新增编译警告。
- [x] 第 2 阶段：按外层 request id 分别记录 fragment/storage/call-proof 的真实 peer、请求类型和启动时间；成功、失败、取消、peer 断开只清理命中的请求，禁止一个 peer 代表整轮 warp。
- [x] 第 2 阶段：fragment/storage/call-proof 失败分别使用自身网络超时和失败枚举，处罚实际请求 peer；`remove_request/remove_source` 恢复对应状态后由其他合格 peer 重新调度。
- [x] 第 2 阶段：虚假的“warp 无进展”失败枚举已从原生、FFI、Dart 和源码残留中全部删除；新增请求隔离与取消后换 peer 的同文件测试。
- [x] 第 2 阶段自动化验证：smoldot-light 4 项单测和 2 项 doctest、Rust FFI 4 项测试、smoldot-dart 10 项测试通过；格式与残留检查通过。
- [x] 第 3 阶段：原生阶段扩展为 fragments 下载、fragments 验证、目标状态下载、runtime 构建、chain information 构建和 regular；FFI/Dart 使用同名严格 `syncPhase`，未知阶段或相互冲突的原生字段直接拒绝。
- [x] 第 3 阶段：新增 Rust 原生唯一 `isUsable`；Dart `ChainStatus/wait`、RPC manager、缓存导出和 Banner 只消费该字段，不再根据 best/finalized、请求历史或 UI 计时推导完成。
- [x] 第 3 阶段：普通订阅 `finalizedBlock` 与 `currentVerifiedFinalizedBlockNumber/hash`、`warpTargetFinalizedBlockNumber/hash` 已彻底分离；缓存稳定导出、恢复校验、低频推进和 finalized 业务读取统一使用完整验证 finalized。
- [x] 第 3 阶段：状态快照新增 fragment/storage/call-proof 当前活动请求数；Banner 与结构化日志按真实阶段显示和记录，`isUsable=true` 后停止轮询。
- [x] 第 3 阶段自动化验证：smoldot-light 5 项单测和 2 项 doctest、full-node 编译检查、Rust FFI 4 项测试通过；本机 Rust FFI release 动态库重建后，smoldot-dart 11 项测试、Flutter RPC/Banner 15 项测试及 4 个目标 Flutter 文件分析通过。Android 原生库和 APK 未在本阶段构建，不能作为真机完成证据。
- [x] 第 4 阶段：无有效本机 database 时，addChain 第一份原生快照必须精确证明 `bundledCheckpoint / #0 / genesisHash`；合法本机 database 仍必须精确证明 `localDatabase / 信封高度 / 信封 hash`，坏缓存只允许清理后回退一次。
- [x] 第 4 阶段：删除同步失败和后台重试未完成时“保存部分进度”的旧调用；原生 warp 活跃时关闭 chain-information 序列化，App 仅在 `isUsable=true` 且导出前后 `currentVerifiedFinalized` 完全一致时落盘。
- [x] 第 4 阶段：Dart `_synced` 改为每次业务入口重新确认原生 `isUsable`；运行期间 peer F 推进并重新进入 warp 时立即撤销 ready，完成后的 F 持久化为下一次 H。
- [x] 第 4 阶段：`ChainRpc` finalized 缓存命名空间、runtime API 锚点和钱包确认高度全部切到 `currentVerifiedFinalized`；普通订阅 finalized 只保留为 `surface_finalized` 诊断字段。
- [x] 第 4 阶段自动化验证：固定 #0 启动来源、高度和 hash，表面 F/完整验证 H 隔离，F 落盘后恢复为下一次 H，单调写入、冲突 hash、生命周期取消等现有测试通过；smoldot-light 5 项单测和 2 项 doctest、Rust FFI 4 项测试、smoldot-dart 11 项测试、Flutter RPC/cache/Banner 24 项测试及 5 个目标文件分析通过，本机 release FFI 库已重建验证。
- [x] 第 5 阶段：彻底删除 CitizenApp 的非 ARM64 构建目标、Rust target、脚本分支、原生库、文档、注释和任务记录；Android 发布面唯一允许 `aarch64-linux-android → arm64-v8a`，Gradle 同时强制排除插件携带的所有非 ARM64 native 库。
- [x] 第 5 阶段：重新构建 ARM64 smoldot native 库。源码产物为 5,798,976 bytes，SHA-256 `31f239d6bbf28fbb94c38461d7a3e680e431ed63ab3a6387fcaba56c56cd0beb`；验证为 ELF64/AArch64、16 KiB LOAD 对齐，并包含状态快照等关键 FFI 导出符号。
- [x] 第 5 阶段：以 `--target-platform android-arm64` 重建 profile APK。产物 `build/app/outputs/flutter-apk/app-profile.apk` 的 SHA-256 为 `92f790fa274874bc75d7a2c17ddb88782d3aebdd6d6ebcf4575f0e2f3c706d1d`；APK 内所有 native entry 均位于 `lib/arm64-v8a/`，`zipalign -c -P 16 4` 与 APK v2 签名校验通过。
- [x] 第 5 阶段：核验 APK 内 `light_sync_state.json` 非空、包含 `finalizedBlockHeader` 和 `grandpaAuthoritySet`，内置 header 的 SCALE 区块号为固定 `#0`。本阶段仅完成构建物静态验收，未把 APK 安装到设备，不能冒充真机 H/F 验收。
- [x] 第 6 阶段真机新安装：Pixel 8a 私密测试空间清除 App 数据后安装 ARM64-only profile APK；非链首屏冷启动约 1.07 秒且不创建 smoldot。进入交易链入口后，原生精确证明 `source=bundledCheckpoint / startup=#0`，只发起 1 次 warp 请求，proof `received=1 / verified=1 / rejected=0`，最终 `phase=regular / usable=true / currentVerifiedFinalized=peer F`。
- [x] 第 6 阶段真机本机恢复：新安装完成后保存 5,120-byte finalized database 信封；强停冷启动后精确证明 `source=localDatabase / startup=H`，peer F 与 H 相等时 `requests=0`，不重复 warp，连接 peer 后恢复 `regular + usable=true`。
- [x] 第 6 阶段真机断网恢复：移动数据关闭后冷启动仍严格验证本机 H，但 `peer=0 / peer_finalized=null / usable=false`，交易按钮保持禁用；Cloudflare 启动清单不可用时只使用本地链规格，不存在 HTTP 链状态回退。恢复移动数据并再次冷启动后，P2P 在约 5 秒内重新达到 `regular + usable=true`。
- [x] 第 6 阶段首轮资源与故障检查：首次 warp 观察窗口进程 CPU 约 0.3%，稳定后为 0%；`cit-smol-0/1` 与 `cit-cap-0/1` 四个原生线程 nice 均为 5；该窗口未发现 CitizenApp ANR、输入分发超时、fatal signal、崩溃或 capability 队列溢出。该历史结果不覆盖后续真实正高度差暴露的 native crash。
- [ ] 第 6 阶段运行中推进：正式链已产生真实 `F > H`。原进程运行中追高时收到 3 份 warp proof，但均以 `blockNumberNotIncrementing` 拒绝；随后交易服务验证 future 在 finalized 推进期间触发 native `SIGABRT`。Android 重启进程后，原生从已保存本机 H 以 1 次请求成功追到 peer F，proof `received=1 / verified=1 / rejected=0`，进入 `regular / usable=true` 并保存新 F。进程重启后的成功不能抵消运行中崩溃，因此本项仍失败且不得关闭。
- [x] 第 6 阶段崩溃根因定位：设备 crash buffer 为 `smoldot-light-6` 上的 `internal error: entered unreachable code`。使用与安装包 `.text` 逐字节一致的 ARM64 release 重链地址映射，调用栈精确落到 `transactions_service::validate_transaction()` 的 `transactions_service.rs:1493`。该行把 `PinPinnedBlockRuntimeError::BlockNotPinned` 写成 `unreachable!()`；同文件此前已经明确记录“选中的 best block 可能在验证 future 真正运行前被裁剪并解除 pin”的竞态。finalized 推进命中此竞态后，release 的 `panic=abort` 终止整个 App。这是本次 native crash 的确定根因，不是推测。
- [ ] 第 6 阶段竞态修复与复验：`BlockNotPinned` 必须作为过期验证锚的瞬态结果处理，清理本次 validation future 并对仍待处理的交易改用当前 best block 重试；禁止升级成 panic，也不得把单块过期误判为整条 runtime subscription 失效。需新增确定性回归、重建 ARM64 APK，并在有待处理交易时等待下一次 finalized 自然推进，证明 App 不退出、交易状态不被错误丢弃、节点从本机 H 追到新 F 且新 F 单调落盘。
- [x] 第 7 阶段自动化回归：smoldot-light 5 项单测和 2 项 doctest、Rust FFI 4 项测试、CitizenApp RPC/交易页 38 项测试、smoldot-dart 全量 51 项测试全部通过；5 个目标 Dart 文件 analyze 无问题，Dart 与本次 Rust 改动文件格式检查通过。
- [ ] 第 7 阶段最终收口复验：此前 ARM64-only APK、16 KiB 对齐、签名、固定 `#0` 资产、脚本语法和旧 ABI 零残留检查均已通过，但后续真实正高度差暴露的 native crash 已使“最终无故障收口”结论失效。完成竞态修复后必须重跑全部自动化、APK 静态检查和真实 finalized 推进验收；`citizenchain/runtime/` 仍保持零改动。

第 11 步当前结论：第 1～5 阶段、新安装 `#0 → F`、本机 H 等于 peer F 时零 warp 恢复、断网 fail-closed、P2P 恢复和首轮 CPU/线程验收均已有真实证据。真实 `F > H` 已经发生，但运行中追高暴露交易验证与 finalized 解除 pin 的确定竞态，并以 native `SIGABRT` 终止 App；进程重启后的追高成功不算通过。当前必须先修复 `BlockNotPinned → unreachable!()`，再完成有待处理交易条件下的下一次 finalized 推进与全量收口复验，任务继续保持 open。

## 完成标准

- 广场/信息等非链页面不创建 smoldot 线程。
- 并发链入口只创建一个 client，失败、dispose、重启均可恢复。
- 真实长链首次同步和已有缓存增量同步均有量化 CPU/耗时记录。
- 分叉压力下 Flutter 主线程仍可响应，不出现 ANR。
- 临时诊断、旧错误结论、旧注释和重复启动路径全部清理。
- `flutter analyze`、全量 Flutter 测试、Android profile/release 真机验收完成。
