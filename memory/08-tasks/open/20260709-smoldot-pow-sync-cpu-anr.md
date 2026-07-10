# 20260709 公民 ANR：smoldot 生命周期、同步与资源约束

## 任务目标

Pixel 8a / Android 16 曾出现 `Input dispatching timed out` ANR。任务目标是先切断非链页面无条件启动轻节点，再验证 warp、分叉与原生资源占用的真实边界，最终让链同步不能持续挤占 Flutter 主线程所需资源。

本卡是当前 ANR 修复任务的唯一任务卡，不再按旧的“fork 没有 warp，因此必然从创世逐块同步”结论实施。

## 已确认事实

1. 历史 ANR trace 显示进程处于高 CPU 状态；临时关闭 `main.dart` 的 smoldot 启动后，采样量和 CPU 显著下降，证明 smoldot 启动/同步管线是当次 ANR 的必要触发因素。
2. 用户在约 31 块的当前短链上重新实测，profile 包约 `0.5% CPU / 10s`，ANR 未复现。因此当前没有正在持续复现的短链故障，后续验收必须覆盖长链和分叉压力，不能拿短链无 ANR 代替完成。
3. `citizenapp/smoldot-pow` 已完整接入 GRANDPA warp sync，`sync/all.rs` 会创建 warp 状态机，`sync/warp_sync.rs` 明确支持 PoW；默认 `warp_sync_minimum_gap=32`。
4. `citizenchain/node/src/core/service.rs` 已配置 GRANDPA `warp_proof::NetworkProvider`。原卡“fork 没实现 warp”的结论错误，已删除。
5. smoldot 的区块同步主要在 Rust/Tokio 内完成；现有证据不能证明“每验证一块都经 FFI 传给 Dart”，也不能仅按 DSO 百分比推导 FFI 放大倍数。FFI/回调是否构成额外压力需要单独采样。
6. 当前代码在首次同步超时和后台重试失败后都会调用 `_saveDatabaseCache()` 保存部分进度；“同步不完成导致缓存存不进”的结论错误。
7. smoldot 会比较本地 finalized database 与 chainspec checkpoint 的块高并使用更高者。缓存护栏应防异步写乱序，而不是假定启动必然回退到创世。
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
- [x] 核对 smoldot 精确门槛：`warp_sync_minimum_gap=32`，且远端 finalized 必须严格大于本地高度加 32；因此当前 `#32` 不触发 warp，从 `#0` 出发最早在 `#33` 触发。
- [x] 核对客户端完整路径：优先请求并验证 GRANDPA warp fragments，随后获取目标 finalized runtime/必要 storage proof，再转回普通同步追近头；历史成本主要随 authority set 变更与 proof 体积增长，不随普通块高线性增长。
- [x] 核对节点服务：所有节点注册 GRANDPA 协议并挂载 `warp_proof::NetworkProvider`；每次 finalized 都覆盖保存最新 justification，authority set 切换块 justification 强制持久化。
- [x] 核对长期数据条件：当前只允许归档节点模式，生产 service 使用 `--state-pruning archive`，默认 block pruning 保留 finalized 正典链，可提供 warp 所需 header、justification 和目标 state。
- [x] 固化唯一目标架构：正式签名 App 内置同 genesis 的发行 finalized checkpoint，Cloudflare 只更新经链身份校验的 bootnodes；高度差超过 32 自动 warp，完成后保存本机 finalized database。远端动态 checkpoint 不进入当前信任路径。
- [ ] 正式链 finalized 到达 `#33` 或更高后，用全新 App 数据/独立测试 profile 完成真实 warp request、fragment、高度跳转、CPU 与输入响应验收。当前高度不满足触发条件，该验收保留为本步骤的运行态门禁，不得伪造为已通过。

第 3 步结论：未来链高增长时，新安装用户依靠 GRANDPA warp 快速接近 finalized 近头，不从创世逐块验证全链，也不下载全节点数据库。现网客户端和节点端代码路径已经具备该能力；当前唯一缺失的是正式链超过门槛后的真实运行证据与同步模式可观测性。

### 第 4 步：原生资源约束

- 根据第 3 步采样决定 Tokio worker 数量、线程优先级、日志/回调限速和是否需要独立 Dart isolate。
- 目标是限制最坏情况下的 CPU 竞争，而不是只依赖链短、checkpoint 新或分叉少。

### 第 5 步：缓存单调推进

- 串行化 finalized database 导出与持久化。
- 缓存信封记录 genesis hash、finalized 高度和数据库正文，只允许更高 finalized 覆盖。
- 覆盖异步写乱序、损坏缓存和生命周期切换测试。

### 第 6 步：发行 checkpoint 与远端残留清理

- 为正式 App 发布建立发行 checkpoint 导出与验证入口：候选锚点必须绑定同一 genesis，经上一发行锚点启动的无缓存 smoldot 验证，并与至少 3 个独立归档节点 finalized hash 交叉一致后写入签名安装包。
- 删除 Cloudflare bootstrap、Dart model、配置和测试中未进入当前信任路径的远端 checkpoint URL / SHA-256 字段，只保留经本地链身份校验的 bootnodes 清单。
- 不保留动态 checkpoint、HTTP RPC、全节点数据库下载或双轨兼容分支。

## 完成标准

- 广场/信息等非链页面不创建 smoldot 线程。
- 并发链入口只创建一个 client，失败、dispose、重启均可恢复。
- 真实长链首次同步和已有缓存增量同步均有量化 CPU/耗时记录。
- 分叉压力下 Flutter 主线程仍可响应，不出现 ANR。
- 临时诊断、旧错误结论、旧注释和重复启动路径全部清理。
- `flutter analyze`、全量 Flutter 测试、Android profile/release 真机验收完成。
