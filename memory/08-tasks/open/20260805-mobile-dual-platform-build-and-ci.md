# 移动端双平台：编译按钮拆分 iOS/Android + CI 与 Release 双端产物

当前状态：已完成（2026-08-07，四端统一为保留数据的覆盖升级）。

任务需求：把公民与公民钱包的「编译软件」各拆成「编译iOS端」「编译Android端」两个按钮，
点哪个编哪个；并让「运行 CI」与「正式 Release」同时产出 iOS 与 Android 两端产物。

所属模块：citizenconsole（不在 Git 版本库内）、citizenapp、citizenwallet、.github/workflows

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 已定口径

1. **入口即语义**：点「编译iOS端」就是 iOS，自动判定与回落**整段删除**。
   探测不到目标平台的设备就明确报错退出，绝不改编另一端。
   （现行 `citizenapp-run.sh` 的 `except: print('android')` 正是「以为编了 iOS、
   实际编的 Android」的来源。）
2. **不沉淀本地产物**：两端一律不再往 `<product>/target/` 复制 APK，
   产物只在 GitHub。连带删掉只为产出本地 APK 而存在的那次重复 `flutter build apk`。
3. **iOS 走「甲」方案**：CI 与 Release 产出**未签名** iOS 产物，
   付费 Apple Developer Program 到位后再补签名正式版。
   实测阻塞：本机只有 `Apple Development` 一个签名身份，分发证书 0 个、描述文件 0 个；
   团队 `7QJXLLBA6J` 是免费个人团队，开不出分发证书与 Ad Hoc/App Store 描述文件。
4. **官网下载链接绝不提前加**：固定资产名规则下，挂不存在的 iOS 资产必然 404。
5. **不造 iOS 更新清单**：`citizenapp-android-update.json` 是 Android 侧载专用，
   iOS 走 App Store，没有对等物。
6. **原生库照旧在 CI 里重新构建**，不信任入库的 `.a` / `.so`（沿用 Android job 惯例）。

## 分步

- 第 1 步：本地编译拆四个按钮 + 彻底删本地沉淀
- 第 2 步：两端 CI 加 iOS job（`macos-latest`，`--no-codesign`）
- 第 3 步：两端 Release 同时挂 iOS 产物

## 执行结果

### 第 1 步（2026-08-05 完成）

| 落点 | 改动 |
|---|---|
| `citizenconsole/server.mjs` | 2 个动作 → 4 个（`{product}-build-ios` / `-build-android`）；`actionSupportsHotReload` 覆盖两个新 mode |
| `citizenconsole/web/app.js` | 前端热载判定同步（原来也钉在 `build-install` 上，属同一处漏改） |
| `citizenconsole/actions/common.sh` | `launch_local_terminal` 收第三个参数 `platform`，白名单校验后 `exec bash "$script_rel" "$platform"` |
| `citizenconsole/actions/{citizenapp,citizenwallet}.sh` | `build-install` 分支拆成 `build-ios` / `build-android`，各自传死平台 |
| `citizenapp/scripts/citizenapp-run.sh` | 删自动判定与回落、删本地沉淀、收必填平台参数、`flutter run -d`；adb 自检只在 android 下做 |
| `citizenwallet/scripts/citizenwallet-run.sh` | 同上（原来把 `android` 写死传给 `build-signer-native.sh`） |
| `citizenconsole/web/styles.css` | `.actions-mobile` 三列 → 四列，两处规则（基础 + 媒体查询）同步；四列排不下 2/3 宽度，改为占满整行 |

### 清理

- 删 `citizenapp/target/`（181MB）与 `citizenwallet/target/`（98MB），共 279MB
- `build-install`、`TARGET_DIR`、`TARGET_APK`、`sync_android_artifact`、`DEVICE_LINE`、
  `ANDROID_TARGET_PLATFORMS`、`repeat(3, minmax(145px…))` 全仓零残留

### 验证

- 两端 iOS 原生库真跑通过：`libsmoldot.a` 36 符号（smoldot 21 / sr25519 4 / chat_mls 11）、
  `libcitizenwallet_signer.a` 4 符号。公民钱包的 iOS 分支此前从未被任何自动化调用过，本次首跑通过。
- 参数契约实测：缺参数、非法平台在**控制台层与脚本层各拦一次**；旧 `build-install` mode 退出码 2。
- 测试 84 个全绿；新增 4 组用例，逐项注入回归验证（改回旧写法必红，共 4 红）。

### 文档

`GMB_TECHNICAL.md`（详情页四动作 + 入口即语义 + 不沉淀）、`WALLET_TECHNICAL.md`（删本地产物约定）、
`ci-path-routing.md` 第 3 节（两个移动产品共同的本地编译口径）；
`20260802-citizenconsole-wasm-ci-full-push.md` 的 08-03 决策记录保留原文并追加指向，不改写历史口径。

### 待用户执行

`bash ./citizenconsole/start.sh build-production` 重新编译控制台后按钮才会变成四个。

### 第 2 步：四端 CI/Release 统一 + iOS 双端产物（2026-08-05 完成）

用户口径：三个甲全收，且**公民、公民钱包、公民链、公民链wasm 各端都要统一**。

| 维度 | 改动 |
|---|---|
| CI 模式产出产物 | 两个移动端补上传不签名产物（`公民-未签名.apk` / `公民钱包-未签名.apk`）。此前全部 upload 都带 release 守卫，`mode=ci` 跑完什么都不留，是四端仅有的例外 |
| runner 标签钉死 | 四端 10 处 `-latest` 全部钉死：`ubuntu-24.04` / `windows-2025` / `macos-15`。钉的就是当时的解析结果，行为不变 |
| 原生库从源码重建 | `citizenwallet-ci.yml` 补 Rust 工具链 + `build-signer-native.sh android`；此前整条流水线没有 Rust，直接吃入库的 `.so` |
| iOS job | 两端各加一个 `macos-15` job，`--no-codesign`，产物 CI 与 Release 都出 |

### 顺带收敛（不做就会新增一份副本）

`citizenwallet` 的 pallet/call 索引同步此前有两份实现且覆盖范围不同（CI 同步 3 个 pallet、
`citizenwallet-run.sh` 同步 20 个），加 iOS job 会变成三份。抽取为唯一脚本
`citizenwallet/scripts/sync-pallet-registry.sh`（20 pallet + 3 call 全集），三处共用。
实测抽取后 `pallet_registry.dart` 零差异，是等价替换。`citizenwallet-run.sh` 118 → 59 行。

### 有意保留的不对称

- **CI 上传严、Release 上传宽**：CI 的 artifact 是该次运行唯一产出，故 `if-no-files-found: error`；
  Release 的 artifact 只是 Release 资产的 30 天便利副本，保留 `continue-on-error`——
  副本没传上不该让已发布的 Release 判失败。
- **`citizenchain-wasm.yml` 不动**：它的单一模式（无 `inputs`）、走 `spec_version` 而非
  语义化版本、成功即发 `runtime-v<n>` 不可变档案，都是白纸黑字定过的设计，不是欠账。

### iOS 阻塞（实测确认，非配置问题）

本机签名身份只有 `Apple Development: joy_rhett@icloud.com`；分发证书 **0 个**、
描述文件 **0 个**；团队 `7QJXLLBA6J` 是免费个人团队（`Runner.entitlements` 已写明它不支持
Push 权能）。免费团队开不出分发证书与 Ad Hoc/App Store 描述文件，其开发描述文件 7 天过期。
故可安装 `.ipa` 在付费 Apple Developer Program 到位前不可能产出，iOS 只出未签名 `Runner.app`。

`continue-on-error: true` 的唯一存在条件即此。**出现真签名步骤时必须同时删掉**，
由 production-security 的互斥用例反向钉住。

### 测试

90 个全绿（本步新增 6 组）。逐项注入回归验证 4 处，其中**第 2 处暴露了一个假测试**：
iOS 三步顺序断言直接在 workflow 原文里找下标，命中的是注释里按 1)2)3) 写的说明
（天然正确顺序），无论真实步骤怎么排都恒真。已改为剥掉注释行后再定位，重新注入确认必红。
同类问题还修了签名判定——`--no-codesign` 含 `codesign` 三字却语义相反，且说明注释里
就列着这些词，两类噪声都会让测试测的不是它声称的东西。

### 待用户执行

`bash ./citizenconsole/start.sh build-production` 后跑一次公民链 CI，验证 Windows 标签变更。

## 第 1 步的回归修复（2026-08-05）

拆分编译按钮后用户连撞两次失败，两次现象不同、根因同一个。

### 现象与机制

| 报错 | 机制 |
|---|---|
| `Killed: 9 ... flutter clean` | `citizenapp-run.sh:50` 的 `pkill -9 -f flutter_tools.snapshot` 是**机器级**的，`-f` 匹配全命令行，把公民钱包正在跑的 flutter 一起 SIGKILL |
| `package_config.json does not exist`（`pub get` 明明刚成功） | 同产品另一端的 `flutter clean` 删掉了本次刚 `pub get` 出来的 `.dart_tool/` |

根因：**同一产品的两个编译端共用同一份 Flutter 工程目录**——`.dart_tool/`、`build/`、
`rust/target/`、原生库产物、`ios/Pods/` 都只有一份。拆分前每产品只有一个编译动作，
`duplicateRunActive` 天然挡着；拆成两个 actionId 后这条路开了。是第 1 步引入的。

### 修复

1. **删掉机器级 pkill**（`citizenapp-run.sh`）。残留进程由控制台的独立进程组 +
   按组终止承接，这一枪不需要，且会误伤其它产品乃至用户手敲的 flutter。
   `adb.*fork-server` 那个定向重置保留——它只在 adb 自己卡死时重启 adb server。
2. **同产品两端互斥**（`server.mjs` `conflictingProductBuild`），跨产品不受限：
   两个产品各自独立 `[workspace]` 与 `target/`，共用的 `citizen-signer` 是只读 path 依赖。
3. **被拒绝时当场弹窗**（`web/app.js` `showNotice`），点名是哪一端在占用；
   同时仍写系统日志留痕。只写日志不够——「点了没反应」用户不会去翻日志标签。

### 结束即放行

互斥判据**只看实时 run 状态**（`running` / `starting`）。`finishRun` 在正常退出、
非零退出、手动停止、子进程 error 四条路径上都会改写 `state`，且由 `finishedAt` 保证只执行一次。
因此另一端一结束就立刻放行，成功失败一视同仁；不落锁文件、不留粘滞标记，
崩溃或强杀也不会留下解不开的锁。

### 测试

93 个全绿（本次新增 3 组）。其中互斥那组**不是源码正则断言**——把守卫函数从 `server.mjs`
抽出来真跑 10 个场景（同产品挡、跨产品放行、starting 也算占用、success/failed/stopped
三种终态各自放行、CI 不参与、同按钮交给 `duplicateRunActive`）。
「结束即放行」是行为契约，正则断言不出来：写成 `state === 'idle'` 之类的错判据同样能
通过任何形状匹配，却会让另一端永远启动不了。
逐项注入回归验证 3 处，全部必红。

### 顺带收敛

`isProductBuildMode(mode)` 成为「什么算编译端动作」的唯一谓词，热载判定与互斥判定共用。

## 标签收场入口合并为「×」（2026-08-05）

用户现象：`CitizenWallet · 编译Android端` 标签关不掉，关了刷新又回来；同时控制台「编译」
按钮一直报「有动作正在运行」。

根因是同一个：**`flutter run` 装机后常驻等待热载，永不自行退出**，该 run 永远是 `running`。

- `renderTabs` 对 `supportsHotReload` 的标签只渲染「热载 + ×」，**漏了「停止」**，
  `canStop` 明明是 true 却没有按钮 → UI 上没有任何终止入口
- `closeTab` 的 × 只把 id 加进 `detachedRunIds`（内存 Set）并断开 SSE，刷新即失忆，
  `loadStatus` 看到 run 仍在 running 就重新接管 → 「关不掉」
- 该 run 一直占着 `anyRunActive()` → `startRebuildRun` 永久拒绝 → 「编译」被挡死

### 修复

先补了「停止」按钮，用户否决——**一步能解决的事不拆两步**。最终形态：
删掉「停止」按钮，**× = 关闭标签 = 终止任务**，`closeTab` 在任务还活着时先 `stopRun`
再摘标签。编译标签的按钮顺序固定为「热载（就绪后）/ 编译中·失败（未就绪）→ ×」，
热载是功能按钮、不是收场按钮。

`detachedRunIds` 保留，但语义变了：不再是「本页不再接管」，而是**短期竞态挡板**——
停止请求只发 SIGTERM 就返回、不等子进程真的死，抢在状态翻成终态前摘掉标签会被
下一轮 `loadStatus` 接管回来。状态一变终态由 `loadStatus` 自动清除。

### 清理

- 删掉 `renderTabs` 里那句「×（还在跑则先终止再关）」——`closeTab` 从来没调过 `stopRun`，
  注释描述的是一个没实现的行为
- 两个分支各写一遍的「停止 / ×」合成一处，去掉重复实现
- `openRunTab` 调用处多传的第 6 个实参（函数只有 5 个形参，会被静默丢弃）已删

### 测试

94 个全绿。新增一组：用计数判定去重（`停止` 与 `×` 各只有一处实现，再分支出去必然出现第二份）
＋ 按下标断言「停止在热载分支之后」＋ 热载分支内不得嵌入停止/关闭。
注入回归（把停止改回只给非编译标签）确认必红。

### 尚未处理，待用户决定

「编译」对 `anyRunActive()` 的互斥是否收窄。核查发现：`launch_local_terminal` 最后
`exec bash "$GMB_ROOT/$script_rel"`，**exec 之后进程已完全离开签名包**，后续不再碰包内文件；
换包用 `mv`（新 inode），运行中的进程持有旧 inode。所以编译端任务其实是这条互斥里
风险最小的一类；真正有混版风险的是全程留在包内 `actions/*.sh` 的 CI / Release / 部署。

## 日志标签统一为「名称 · 状态 · ×」（2026-08-05）

需求：名称 = 软件名 + 动作（如 `公民iOS端`），状态 = 动作状态（编译中 / 热载），× = 关闭即停止，名称尽量精简。

### 实现

- 服务端新增 `runTabName(module, action, nodeLabel)` 作为标签名唯一来源，
  动作精简名走 `ACTION_SHORT_NAMES` 按 **mode** 取（`build-ios`→iOS端、`ci`→CI、
  `wasm-ci`→WASM CI、`release`→Release、`deploy`→节点名 …）。
  **刻意不对 `action.title` 做字符串裁剪**：按钮文案带着与状态重复的动词
  （「编译iOS端」「运行 CI」），裁剪规则会随文案改动悄悄失效。
- 软件名取中文 `cardTitle`，不用 `CitizenWallet` 这类英文正式名——标签栏要短，且全站中文。
- `/api/run` 一并返回 `title`，前端不再自拼。前端拿不到 44 节点的节点名，自拼必然漂移。
- 前端 `renderTabs` 去掉 `标题 · 状态` 后缀，名称/状态/× 三段固定；编译任务就绪后
  状态栏就地变成可点的「热载」，不额外占位。
- 日志首行保留完整称谓（产品正式名 + 按钮原文案），与标签栏精简名各司其职。
- `rebuild` 动作补 `mode`，走同一张精简名表。

### 实测名称（14 个动作全覆盖）

`公民iOS端` `公民Android端` `公民CI` `公民Release` `公民钱包iOS端` `公民钱包Android端`
`公民链CI` `公民链WASM CI` `公民链启动节点` `公民链Release` `公民链01 北京`
`公民云生产部署` `公民云官网部署` `公民控制台编译`

### 测试

95 个全绿。命名那组把 `runTabName` 抽出来真跑 14 个动作，逐条钉住最终显示的字符串——
正则只能证明代码长什么样，证明不了「公民 + 编译iOS端」最终会显示成「公民iOS端」。
另钉住：名称不含英文产品名、不含任何状态字。注入 2 处回归（软件名退回英文、状态拼回名称）确认必红。

## 删除热载（2026-08-05）

编译改成 `build + 原位覆盖`（iOS release + `devicectl device install app`，Android debug +
`adb install -r`）之后，热载已不可能触发：检测靠日志里的 `Flutter run key commands` / `A Dart VM Service on`，
只有 `flutter run` 才输出。更硬的一层是 **iOS 上物理不可能**——iOS 必须 release 才能从桌面点开，
而 release 关闭了 Dart VM Service，`flutter attach` 也连不上。按两端一致铁律，四端一并删除。

### 删除范围（前后端 + 样式，零残留）

| 位置 | 删除内容 |
|---|---|
| `server.mjs` | `actionSupportsHotReload`、`run.supportsHotReload`、`run.hotReloadable`、就绪检测特征串、`emit(run,'status')`、`POST /api/runs/:id/hot-reload` |
| `web/app.js` | 热载按钮、`hotReload()`、`status` SSE 监听器（唯一发送点已删，成死码）、`tab.hotReloadable` |
| `web/styles.css` | `.log-tab-hot` 两条规则 |

### 取而代之

`isBuildRun(module, action)` → `run.isBuild`，**只决定标签文案**（运行中显示「编译中」而非「运行中」）。
终态一律走 `stateLabel`：成功 / 失败 / 已停止，不存在停在「编译中」的终态。

### 验证

97 个全绿。新增一组钉住：前后端与样式里 `hotReload|热载|log-tab-hot|hot-reload` 全无
（判定只看代码行、剥掉注释——说明为什么删的注释本身含这些词），`status` 事件两端都不留死码，
`isBuild` 前后端同口径。注入 2 处回归（恢复 `hotReloadable` 字段、文案改成无条件「编译中」）确认必红。
另用真实 `styles.css` 起独立页面截图验证：编译中 / 成功 / 失败 / 运行中 / 已停止 五种状态显示正常。

### 仍未处理，需你决定

四个编译动作**没有任何超时**（`timeoutMs: 0`，全仓只有 Cloudflare 生产部署声明了 30 分钟）。
`flutter build ios --release` 卡在签名环节时进程不生不死，标签会永远停在「编译中」——
这正是「一直显示编译中」最可能的真实成因，删热载并不能解决它。

## 四端覆盖升级保留钱包数据（2026-08-07）

### 根因

公民与公民钱包的 iOS 编译脚本曾调用 Flutter 安装命令。本机 Flutter 3.41.0 的该命令默认
`uninstall = true`，每次发现同 Bundle ID 的旧 App 都先执行 `device.uninstallApp()`；iOS 随之
删除 Application Support 中的 Isar 数据库，所以每次编译安装后钱包与账户列表消失。Android
一直使用 `adb install -r`，属于保留数据的覆盖升级，没有同类问题。

### 统一目标态

- 公民与公民钱包、iOS 与 Android 四端统一为“编译最新软件并覆盖升级”，禁止卸载旧软件。
- iOS 直接调用 Xcode `devicectl device install app`；其 `--device` 接受 Flutter 返回的硬件
  UDID，无需转换为 CoreDevice 内部 Identifier。
- iOS 覆盖前校验固定 Bundle ID 和有效 Team ID。系统更新 App 时允许迁移数据容器并改变
  `dataContainerPath`，所以不得用绝对路径相等判断是否保留数据；已有钱包 Isar 文件时，覆盖后
  必须复读到同名、同非零大小的数据库。无法读取或安装失败一律失败关闭，绝不回退到卸载重装。
- Android 保持 `adb install -r`；`-r` 是替换现有应用并保留数据。任何安装失败只允许停止。
- 两款产品共用字节级相同的 iOS 安全安装函数，仅 Bundle ID 和构建步骤不同；控制台回归测试
  强制两份函数一致，并反向禁止四端出现任何卸载命令。

### 修改范围

- `citizenapp/scripts/`：公民 iOS 原位覆盖、身份校验和钱包数据库守卫；Android 保持 `-r`。
- `citizenwallet/scripts/`：公民钱包采用同一实现；不改钱包业务和密钥模型。
- `citizenconsole/test/`：删除对旧 iOS 安装方式的错误要求，增加四端禁止卸载、数据容器守卫、
  两款产品函数一致性和固定 Bundle ID 断言。
- `memory/05-modules/citizenapp/wallet/`、`memory/07-ai/`：统一当前技术口径并清理错误注释。

### 验收口径

- Bash 语法、ShellCheck、CitizenConsole 全量测试通过。
- 全仓可执行脚本不存在 Flutter 卸载式安装、`devicectl ... uninstall` 或 `adb uninstall`。
- 已连接 iPhone 上两款 App 覆盖前后钱包 Isar 文件保持同名、同非零大小，App 仍可直接启动。
- 覆盖安装不得修改 runtime、订阅、Cloudflare 或链上逻辑。

### 真实运行态验收

- 公民 iOS：完整 release 构建成功；直接调用与控制台脚本相同的安全安装函数再次覆盖成功，
  `org.citizenapp`、Team ID `7QJXLLBA6J` 校验通过，`citizenapp.isar` 覆盖前后均为
  41,943,040 字节。首次验收同时确认 iOS 可能迁移容器 UUID，任务据此删除了错误的绝对路径
  相等判断；新容器中的原钱包数据库实际存在，不是空容器。
- 公民钱包 iOS：完整脚本一次通过，`org.citizenwallet`、同一 Team ID 校验通过，
  `citizenwallet.isar` 覆盖前后均为 1,048,576 字节。
- 公民 Android：完整脚本 `adb install -r` 成功，`citizenapp.isar` 覆盖前后均为
  36,700,160 字节，证明既有钱包数据库保留。
- 公民钱包 Android：目标 Pixel 原先未安装该产品，本次首次安装成功并可启动；启动后生成
  1,048,576 字节 `citizenwallet.isar`。后续编译固定走同一 `adb install -r` 覆盖路径。
- Bash 语法、ShellCheck、本任务回归测试及 CitizenConsole 编译检查全部通过。CitizenConsole
  全量测试当前 98/99；唯一失败是既存节点部署测试仍断言旧冻结 CI run id `30724462739`，而
  当前节点部署脚本已是 `31164684013`，与本任务无关且未越界修改。
- iPhone 两款 App 完成安装与数据库复读后进入 CoreDevice 不可调试状态，后续自动 launch 被
  iOS 拒绝；该状态发生在覆盖和数据连续性验收完成之后，不改变安装结果。
