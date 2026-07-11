# 20260709 citizenapp 启动懒加载改造（打开即启动 → 后台按需加载）

## 背景 / 病根（真机 logcat + 代码双证）

Pixel 8a / Android 16，debug 包打开公民 App 反复弹「公民没有响应」(ANR)。真机定位：

- **本质是 ANR**（`Input dispatching timed out. Waited 5002ms`），不是弹窗、不是生物识别、不是死循环（App ~12s 后落回空闲）。
- **debug 包 JIT 慢 5-10 倍** → 启动那段重活把 UI 线程饿死 >5s → ANR。同代码 **profile(AOT) 包无 ANR**（已真机验证）。
- 但**启动确实太重**（profile 也 100-217% CPU ~14s），三个根因：
  - **A. 顶层 5 tab 全量预建**：`main.dart` `IndexedStack(children: _pages)` 一次建广场/公民/信息/交易/我的全部。
  - **B. 公民默认子页 ProposalView 一建就同步 4.2 万条行政区字典**：`proposal_view.dart:185` `_loadInstitutionScope()` → `ensureSynced()` → Isar 灌 42k 条（代码注释「秒级~十几秒」，`public_page.dart:101`）。因 A 在**启动**就跑。
  - **C. smoldot 轻节点**：已 `addPostFrameCallback` 首帧后后台 init（`main.dart:69`），不阻塞首帧但仍吃 CPU。

用户拍板：**打开 App 时能不启动的都别启动，能后台慢慢加载的全挪后台。** ①②③ 一起做。

## 目标

顶层 tab 懒建；行政区/机构大 JSON 只在用到时后台加载；smoldot 不与首屏抢 CPU。首屏只建落地页（广场），其余点到才建。

## 任务分解

### ① 顶层 tab 懒建（main.dart _AppShellState）
- `IndexedStack` 保留（保活已建页），但 children 改**按访问懒建**：仅当前 index 或曾访问过的 index 建真页，未访问的用 `SizedBox.shrink()` 占位；页实例缓存一次建成后由 IndexedStack 保活。
- 广场(0)=落地页启动即建；公民(1)/信息(2)/交易(3)/我的(4) 点击才建。
- ProfilePage 更新红点：`_handleUpdateStateChanged` 里失效 index 4 缓存令其重建带新红点。
- **权衡（记录）**：公民待办票数徽标(`onPendingVoteCountChanged`)、Chat 后台收信 在首次访问该 tab 前不触发。Chat 若需后台收信，另起轻量后台邮箱服务（本卡不做，标注）。

### ② 行政区/机构 sync 移出启动 + UI 关键路径
- ①落地后 ProposalView 不在启动建 → `ensureSynced` 自然延到进「公民」才跑。
- 进一步：ProposalView 里 `ensureSynced` 改 fire-and-forget（不 `await` 阻塞提案流首屏），字典就绪后回刷名称；未就绪先按机构码兜底显示（代码已容忍 sync 失败）。
- 42k 解析/灌库若在主 isolate 卡顿，改 chunk 让路 / compute isolate。机构详情（公民→机构→xx机构）确认按省分包懒加载（`public_institution_bundle_loader` 已按省 `<省>.json`）。

### ③ smoldot 不与首屏抢 CPU
- 保持首帧后后台 init，但延到落地页渲染稳定后再启动（idle/短延迟），或改由首个链上消费方 `ensureInitialized()` 懒触发（落地=广场走 Worker 后端不需链，可延到进交易/钱包）。
- 权衡：首次看余额会等同步。

## 验收
- profile/release 包冷启动 CPU 峰值与高占用时长明显下降；停广场时公民/Chat/交易/我的 不 initState。
- 点「公民」才触发 42k 同步且不卡 UI；点「机构」按省懒加载。
- 全端 `flutter analyze` 干净；真机冷启动无 ANR、无「请新增钱包」误报。

## 完成情况（2026-07-09，已真机 + 单测验证）
- ①②③ 全部实现，`flutter analyze` 干净。
- **真机验证（Pixel 8a profile 包）**：停广场时 logcat 搜不到任何 admin_division/ensureSynced（42k 同步启动**不触发**）；**无 ANR**；App 落回空闲；点「公民」tab 秒开不冻结（②后台化生效）；点公民也无 ANR。
- **单测**：`flutter test --concurrency=1` → **+433 通过 / ~5 跳过(native 库环境跳过) / 0 失败**。踩坑：本轮给 `signWithWallet/verifyWalletAccess` 加了 local_auth 后，`wallet_manager_test.dart` 6 条挂在 `MissingPluginException(local_auth authenticate)`；修法=setUp 打桩 `MethodChannel('plugins.flutter.io/local_auth')` authenticate→true（已修，全绿）。**教训：给 WalletManager 加 local_auth 会波及所有走签名/验证的单测，必须在 setUp 打桩该 channel。**
- 遗留（另议）：release 包未出；iOS 未验；②可再拆「区域名字典延到机构详情」；Chat 后台收信 / 公民待办徽标 懒建后首访前不触发。

## 备注
- debug 包已被 profile 包覆盖（同包名），手机上只剩一个；日常测/发布用 **release** 包。
- 关联 [[project_seed_biometric_binding_design]]（同 App，seed 加固 + 昵称=钱包名统一同批）。
