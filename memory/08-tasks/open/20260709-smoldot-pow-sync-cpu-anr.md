# 20260709 公民 App「公民没有响应」ANR —— smoldot PoW 全量同步烧 CPU（诊断+修复卡）

## 症状
citizenapp（Flutter）反复出现 Android **ANR「公民没有响应」**（`Input dispatching timed out, Waited 5000ms for MotionEvent/FocusEvent`）。发生在**广场（落地页）**、feed 空、屏幕基本无内容时也会 ANR。用户报「又出现」= 反复复现。真机 Pixel 8a（adb `3C071JEKB09000`，Android 16，profile 包，DEBUGGABLE 可 simpleperf `--app`）。

## 根因（已逐层实证，非推测）——按证据链

1. **ANR = 平台主线程被饿死**：ANR 原因 `Input dispatching timed out`；ANR trace 里 app 占 **81% CPU（68% user）+ 118929 次 minor 缺页**，主线程 `utm=1038`（10.38s CPU/34s）。不是死锁（进程自愈、事后 0% CPU）。
2. **Dart 应用代码空转**：连 Flutter DevTools/VM Service（`setFlag profiler=true` + `getCpuSamples`）采样 25s → **182 样本 = 0.18% CPU**，全是框架 drawFrame/microtask/socket 收尾，**无 app 热点**。→ 排除 Dart 代码。
3. **gfxinfo：整窗口只渲染 3 帧** → 排除渲染/动画转圈。
4. **Isar** 每线程增量 10ms → 排除。
5. **simpleperf `--app` DSO 级归因（6451 样本/22s）**：`libflutter.so 49%`（内嵌 Dart VM 运行时：GC/端口/FFI，**非渲染**）+ `libc 21%`（`sendto/writev/write/__memmove` 网络+拷贝）+ `libsmoldot.so 12%`（Rust）+ `libart 2.8%`。
6. **隔离实验（决定性）**：临时注释 `main.dart` 的 smoldot 启动、重编重装 → smoldot 线程 0 条、**22s 总样本 6451→283（↓96%）**，`libflutter 2285→21`、`libc 2499→174`、`libsmoldot→0`，录制期无 ANR。→ **CPU 100% 由 smoldot 链同步管线引起**。
7. **为什么 smoldot 这么烧（本该轻）——checkpoint 解码 + fork 结构**：
   - `assets/light_sync_state.json` 的 `finalizedBlockHeader` 解出来是**创世块**（parent_hash 32B 全 0、number=0、state_root=`6a380e96…`=创世 state root）。`grandpaAuthoritySet` 是真数据（GRANDPA **确实接入了**）。
   - **`smoldot-pow` fork 没实现 warp sync**（`smoldot-pow/lib/src` 搜不到任何 warp 代码）。
   - `citizenchain/scripts/bake-chainspec.sh` **故意烤创世**（注释：「物化块 0 → 读块 0 头生成轻形态 → 生成 lightSyncState」）。
   - **合起来**：无 warp → smoldot 从 checkpoint（=创世）**逐块下载+验证追到当前链头**，每次启动重来；成本 **O(链高度)**，链越长越慢、无上限恶化。每验一块又经 **smoldot↔Dart FFI 桥**塞给 Dart VM（分配/GC/memmove），桥把 Rust 的活放大约 **4 倍**（libflutter+libc 70% vs libsmoldot 12%）。
   - **∴ smoldot 本该轻（GRANDPA warp 一步跳 finalized），但 fork 没 warp + checkpoint 是创世 → 退化成从创世全量同步 → CPU 爆 → 主线程饿死 → ANR。**

## 用户已定 / 关键结论
- **静态烘焙一个"近期 checkpoint"治不了本**：没 warp,checkpoint 必随链增长变旧,catch-up = (head − checkpoint) 块,O(高度)。用户亲自点破。
- 底部 tab 已懒建（只有广场 0 落地即建）；但 smoldot 是 `main.dart:74` **全局定时器无脑启**（延迟 1.5s），连不需链的广场也启 → 落地页 ANR。
- 链使用地图（按流程非按 tab）：广场**浏览** feed=后端不需链；广场**发动态/文章**=需链（`SquareChainService`→`ChainRpc`→smoldot：建签名交易+提交+读事件）；公民 tab（提案）、交易 tab（余额）=进 tab 即需；信息=后端 mailbox+MLS（独立 native，非链客户端）；我的=仅身份徽章（MyId `ChainRpc` 读链）。全工程 `initialize()` **只有 main.dart 一处**；消费者都是 `isReady` 才用、自己不启。**无任何真后台/跨页链需求**（徽章/待投票数都只在进过对应 tab 后才活）。

## 修复方案（待走 A 还是 B，用户拍）
- **短期止血：懒启动**。删 `main.dart` 全局定时 `initialize()`，改**「首次真正访问链时自动启动」**（把幂等 `initialize()` 埋进 `SmoldotClientManager`/`ChainRpc` 的查询·订阅·提交入口）。广场浏览/信息永不启 → 落地 ANR 消失。
  - **我的身份徽章走 A（只读不启）**：`isReady` 才查、否则显缓存/占位，**绝不为徽章触发 smoldot 启动**（否则我的落地又卡）。← 已定。
- **中期治本（务实，推荐）方案 A — 服务端刷新的「近头 checkpoint」+ 本地缓存**：
  - 基建已半成：worker `/v1/chain/bootstrap` 下发 `light_sync_state_url`（现 wrangler `CITIZEN_CHAIN_LIGHT_SYNC_STATE_URL` 为空）+ `chain_bootstrap_api.dart` 已读；但 `smoldot_client.dart:436` 只用烘焙 asset、没接拉来的。
  - 补齐：server 起定时烘焙任务（每几分钟从冻结节点导出**最新 finalized 块**的 light_sync_state 放 URL）→ app 启动拉**近头** checkpoint → smoldot 只补最后几块。叠加已有 `_saveDatabaseCache`（上次同步点持久化）增量补。checkpoint 必须是**真 finalized 块（非创世）**。不用写 Rust。
- **长期理想 方案 B — 给 smoldot-pow fork 补 warp sync**：移植上游 GRANDPA warp；checkpoint 变信任锚,增长由 warp proof 处理(成本 ∝ 权威集变更,与块高无关);还需节点服务 warp proof。Rust 大活。

## 现场/证据文件
- 关键源码：`citizenapp/lib/main.dart:66-87`（smoldot 启动）、`citizenapp/lib/rpc/smoldot_client.dart`（同步/checkpoint/缓存,`_injectLightSyncState`@428、`_saveDatabaseCache`@474、3min 超时+重试）、`citizenapp/assets/light_sync_state.json`（**创世 header,是病灶**）、`citizenapp/lib/rpc/chain_bootstrap_api.dart`（拉 manifest,已读 `light_sync_state_url`）、`citizenapp/cloudflare/src/chain/bootstrap.ts`、`citizenchain/scripts/bake-chainspec.sh`（烤创世）、`smoldot-pow/`（无 warp）、`smoldot-dart`（Dart 绑定,pubspec `smoldot: path: smoldot-dart`）。
- 诊断工具可复用：simpleperf `--app org.citizenapp`（本包 DEBUGGABLE 可用,非 root）；VM Service `setFlag('profiler','true')`+`getCpuSamples`（脚本见本会话 scratchpad `cpu_profile.dart`）；`/proc/PID/task/*/stat` 每线程 CPU 增量（maps/`-p PID` 被 perf_harden 挡）。

## ⚠️ 设备/代码当前状态
- `main.dart` 隔离改动**已还原**（smoldot 启动恢复,源码干净）。
- **真机上还装着「smoldot 关闭」的隔离测试包**（很流畅但链功能不可用）→ 需重编正常 profile 包装回。
- 未改任何生产逻辑（全程只读诊断 + 一次已还原的隔离实验）。
