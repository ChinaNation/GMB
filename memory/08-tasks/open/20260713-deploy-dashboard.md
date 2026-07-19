# 本地部署控制台

任务需求：
- 在仓库根目录建立 `deploy/`，集中承载全部部署网页、后端、脚本和密钥状态；源码由 Git 追踪，运行记录和私密材料精确忽略。
- 首页用图标展示 CitizenApp、CitizenWallet、CitizenChain、CitizenChain WASM、CitizenWeb 和 CitizenApp Cloudflare Worker。
- CitizenWeb 只保留“测试部署”和“生产部署”两个按钮卡片；测试部署在本地构建并启动 `127.0.0.1:41732`，生产部署只更新现有 `citizenweb` Pages 项目。
- 测试操作无需密码；生产、Release 和服务器部署必须先完成本机 Touch ID 指纹验证。
- CitizenApp Cloudflare 模块提供“会员真实测试”：由官方 `cloudflared` 打开 Access 批准窗口，通过后才访问真实 staging API、远端 D1、sr25519 签名和 Stripe 测试环境。
- 部署密钥迁入 macOS Keychain，网页只显示状态，不显示、返回或记录密钥明文。
- 完成后删除根 `scripts/` 中旧六个部署入口和两份明文 Secret 文件，不保留旧流程。

所属模块：
- 仓库级部署与发布流程
- CitizenApp / CitizenWallet / CitizenChain / CitizenWeb

用户当前任务特批：
- `deploy/` 中不含密钥的部署脚本和本地网页进入 Git；`.runtime/`、日志、编译产物和私密文件不进入 Git。
- 用户于 2026-07-14 明确允许新增 `deploy/.runtime/cloudflared`；仅安装官方 Darwin ARM64 二进制，继续由根 `.gitignore` 排除，不进入 Git。
- 本任务不沿用“密钥相关入口必须留在根 scripts”的旧规则；根 scripts 只保留 AI 与仓库工具。

安全边界：
- 服务只监听 `127.0.0.1`，浏览器通过随机会话令牌访问。
- Keychain 密钥不返回前端；日志必须脱敏。
- 生产类操作每次执行都强制 Touch ID，不允许密码降级。
- 本轮开发验收不得真实触发 CI、Release、生产发布或服务器部署。

预计修改目录：
- `deploy/`：新增本地网页、后端、部署动作、Touch ID 与 Keychain 管理；源码由 Git 追踪，运行数据与私密材料被 Git 忽略。
- `scripts/`：删除旧部署入口和明文 Secret，清理旧流程残留；保留非部署仓库工具。
- `memory/`：更新任务卡、仓库映射、安全规则和部署入口文档；进入 Git。
- `.gitignore`：精确忽略根 `deploy/.runtime/` 与私密文件；进入 Git。

验收标准：
- 本地网页真实启动并通过浏览器检查首页、模块详情、密钥状态和按钮行为。
- 测试动作不要求认证；生产动作在命令执行前调用 Touch ID，认证失败不得执行。
- Keychain 中 staging/production 各 16 个 Worker Secret 完整且与迁移前值一致。
- 旧六个部署脚本和两份明文 Secret 文件全部删除。
- 日志、API 和前端均不泄露密钥。

当前进度：
- [x] 用户确认目录、Git 忽略、Keychain、Touch ID 和残留删除方案。
- [x] 实现本地部署控制台及全部密钥中文用途说明。
- [x] 迁移并验证 Keychain 密钥。
- [x] 删除旧部署入口和明文 Secret。
- [x] 完成 CitizenWeb 本地测试网站启动、真实页面访问和关闭验收。
- [x] 增加 CitizenApp Cloudflare“会员真实测试”按钮、Access 交互登录和逐项 PASS/BLOCKED/FAIL 日志；测试完成文案不再冒充部署成功。
- [x] 完成会员真实测试的 Access 人工批准及全矩阵运行态验收；业务断言 14 项通过、4 项仅在 Worker→Stripe 525 阻断，环境与清理 3 项通过，FAIL=0。
- [ ] 完成生产动作 Touch ID 成功路径验收；不得在验收中执行真实远端部署。

执行记录：
- 2026-07-13：staging/production 各 16 项 Secret 已写入 macOS Keychain并与迁移前值逐项比对一致，旧明文 Secret 文件随后删除。
- 2026-07-13：真实本地服务展示六个部署图标、20 项密钥中文用途、Keychain/GitHub Secrets 状态和脱敏日志；测试动作不要求密码。
- 2026-07-13：定位 CitizenWeb 测试和生产按钮未执行的原因是控制台当时由验收专用 `GMB_DEPLOY_DRY_RUN=1` 启动，不是官网源码、Wrangler 或 Pages 故障。
- 2026-07-13：CitizenWeb `npm ci`、lint、build、Wrangler 登录、现有 production Pages 项目和线上健康地址前置检查通过；未触发真实 production 部署。
- 2026-07-13：CitizenWeb 测试部署改为本地构建后启动 `http://127.0.0.1:41732`，已通过网页按钮真实启动、页面访问和关闭验收。
- 2026-07-13：部署控制台删除英文眉题，缩小主标题和刷新按钮，六张卡片改为左图标、右内容的紧凑横向布局并缩短高度；真实页面截图确认部署日志区域明显上移。
- 2026-07-13：本地构建、检查、密钥同步、远端触发、发布和健康检查阶段均增加简短中文步骤日志；当时的对外停止测试动作确认 `[开始]`、`[步骤]`、`[完成]` 日志正常显示（该对外动作已在 2026-07-18 删除）。
- 2026-07-14：对 `deploy/` 源码执行明文密钥、私钥头、高熵长值、服务器IP和本机路径扫描，未发现密钥材料。目录改为Git追踪源码，`.runtime/`、日志、编译二进制、`.env`、`secrets/`和常见私钥文件继续忽略。
- 2026-07-13：顶部标题改为“部署控制台”并缩小字号、固定左对齐；部署模块、生产就绪、执行状态三个概览卡压缩后移入同一行，刷新按钮固定右侧；真实页面检查确认布局与文案生效。
- 2026-07-13：部署控制台增加macOS launchd socket按需唤醒：平时Node服务不常驻，浏览器访问 `127.0.0.1:41731` 时启动，空闲15分钟且没有部署任务时退出；手动启动仍保留自动打开浏览器。
- 2026-07-14：经用户明确授权，在被忽略的 `deploy/.runtime/` 安装官方 `cloudflared 2026.7.1` Darwin ARM64 二进制并校验发布 SHA256；控制台新增“会员真实测试”，首次网页点击已验证会打开可见的 Cloudflare Access `Approve` 窗口。
- 2026-07-14：用户在可见 Access 窗口批准后，从部署控制台真实执行 staging 全矩阵；修复 launchd Node PATH、Wrangler D1 登录态和 Stripe 测试夹具后，最终结果为 PASS=17、BLOCKED=4、FAIL=0。PASS 包含 14 个业务断言、真实 Stripe active 订阅夹具、Stripe 两项资源清理和 D1 零残留；B3/D2/E1/F1 均真实到达 Worker 的 Stripe 调用点并返回 525。
- 2026-07-17：CitizenChain 模块增加“清空链数据”和“清空节点数据”两个本机按钮，均需要 Touch ID。清空链数据只删除本机 `gmb.dev/chains/citizenchain/db`，不删除 keystore、node-key 或链上中国数据库；清空节点数据只删除本机 `gmb.dev/onchina-pgdata`，不删除区块链数据库、TLS 证书或 `china.sqlite`。两个按钮不检查本地提交、不触发 GitHub、不连接远端服务器。
- 2026-07-17：服务器部署动作移除旧远端清库分支；“部署服务器/部署该节点”只消费 GitHub main 最新成功 CitizenChain CI 产物，并保留远端链数据。
- 2026-07-17：使用 `/tmp/gmb-deploy-clear-test.*` 隔离目录真实执行 `clear-chain-data` 与 `clear-node-data`。链数据清理只删除 `chains/citizenchain/db` 并保留 `onchina-pgdata`、`onchina-tls`；节点数据清理只删除 `onchina-pgdata` 并保留链数据库和 TLS 标记。脚本语法、`server.mjs`、`web/app.js` 检查均通过。
- 2026-07-17：CitizenChain 模块增加“启动节点”本机按钮。该动作不需要 Touch ID、不检查 Git、不触发 GitHub、不连接远端服务器；已运行时直接返回成功，未运行时打开 Terminal 执行 `citizenchain/scripts/run.sh`，让本地区块链软件保持在真实终端会话中运行。
- 2026-07-17：CitizenApp、CitizenWallet 模块各增加“编译软件”本机按钮（`build-install`，无需密码）。分别 `exec bash` 复用现有 `citizenapp/scripts/citizenapp-run.sh`、`citizenwallet/scripts/citizenwallet-run.sh`，把 APK 编译并安装到已连接手机；不新建脚本、不检查 Git、不触发 GitHub、不连接远端。
- 2026-07-17：本地动作（`build-install`/`start-local-node`/`local-start`/`clear-*`，即 `localOnly:true`）全部改为**在部署日志框内联流式**，不再弹独立 Terminal.app。取代上一条“打开 Terminal”的行为：`start_local_node`、`launch_local_terminal` 由 osascript 改为前台 `exec bash <run.sh>`，stdout/stderr 直接经控制台捕获流入日志。
- 2026-07-17：部署控制台前端改为**多标签日志**：常驻「系统」标签承载节点配置反馈；每个 run 一个标签（标题 `模块·动作·状态`）+ 独立 `<pre>` pane + 独立 SSE；左右可滚动切换，完成后标签显示状态并可 `×` 关闭。后端 `server.mjs` 弃用全局单锁 `activeRunId`，改 `anyRunActive/anyProductionRunActive/duplicateRunActive`：本地动作可并发、生产动作仍串行、同模块同动作防重复。
- 2026-07-17：本地动作以 `spawn('bash', …, {detached:true})` 建独立进程组；新增 `POST /api/runs/:id/stop` 按 `process.kill(-pid,'SIGTERM')` 杀整棵编译树（`canStop=localOnly&&running` 时标签显示「停止」）；控制台收到 SIGTERM/SIGINT 时 `killAllLocalRuns` 中断全部本地任务（重启即中断，已与用户确认）。放弃 macOS `script` 伪 TTY：其在 launchd 无 TTY 环境 `tcgetattr` 失败，且无 TTY 时 flutter/gradle 自动关闭 ANSI 颜色、输出更干净（已用独立脚本实测流式与进程组 kill 均正常、无残留）。
- 2026-07-17：`/api/status` 由 `latestRun+activeRunId` 改返回 `runs[]`（含 `canStop`）+`anyRunActive`；前端 `loadStatus` 只自动接管仍在运行、本页尚未建标签的任务并重连 SSE，已结束任务不自动打开、已关闭标签刷新后不复活。浏览器真实验收：触发 `运行 CI`（dry-run）自动开标签并流式到「成功」、弹窗自动关闭、标签切换仅一个 pane 可见、关闭标签回落「系统」、刷新不复活完成任务、概览随 `anyRunActive` 显示空闲/运行中，`node --check`、`bash -n` 全通过。
- 2026-07-17（修回归）：内联流式上线后本地编译全部失败，根因是 **PATH 回归**。plist [com.gmb.deploy-console.plist] 无 `EnvironmentVariables`，launchd 按需唤醒 server 时 PATH 极简（`/usr/bin:/bin:…`）；`baseChildEnv` 只额外加了 node bin，缺 `~/flutter/bin`、`~/.cargo/bin`、Android SDK、`/opt/homebrew/bin`。旧 osascript 经 Terminal.app 加载登录 shell 隐式拿到完整 PATH，改内联后丢失，实测精简 PATH 下 flutter/cargo/adb 全 `command not found`（`gh` 亦然，CI 在唤醒态同样受影响）。
- 2026-07-18：CitizenWeb 删除对外停止测试按钮和对应脚本分支，官网卡片只保留“测试部署 / 生产部署”；“测试部署”内部仍先停止旧本地测试进程再启动新预览，避免端口残留。
- 2026-07-18：修复 CitizenWeb 生产部署项目检查误判：`citizenweb` 固定 `wrangler@4.112.0` 到 `package-lock.json`，生产脚本先 `npm ci` 再用 `npm exec -- wrangler`；Cloudflare Pages 项目存在性检查改为解析 `wrangler pages project list --json`，不再 grep 表格输出，失败时输出当前账号真实项目名。
- 2026-07-17（修回归）：修复 = `server.mjs` 启动时 `spawnSync(SHELL||'/bin/zsh', ['-l','-c','printf %s "$PATH"'])` 取用户登录 shell（加载 `~/.zprofile`）的完整 PATH，缓存进 `loginPath`，`baseChildEnv` 用 `node目录:${loginPath}` 注入所有子进程（含 5s 超时 + try/catch 兜底）。一处修复覆盖本地编译与远端 CI。验收：`env -i` 模拟 launchd 精简环境跑修复逻辑，子进程 flutter/cargo/adb/gh/node 全部命中；`node --check` 通过；杀旧 server 后经 socket 强制冷启动，launchd 干净启动、无报错、`loginPath` 未拖慢启动。
- 2026-07-17（热载功能）：CitizenApp/CitizenWallet 的「编译软件」标签加「热载」。仅这两个模块的 `build-install`（`actionSupportsHotReload`）适用——flutter run 装机后 attached 长驻，往子进程 stdin 写 `r` 即触发手机热重载。标签状态区：编译/安装中=`编译中`、就绪=可点绿钮`热载`、失败=`失败`，右侧统一 `×`（还在跑先按进程组 stop 再关）。实现：`server.mjs` run 加 `supportsHotReload`/`hotReloadable`；onData 检测 `Flutter run key commands`/`A Dart VM Service on` 置位并 SSE 推 `status`；新端点 `POST /api/runs/:id/hot-reload`→`run.child.stdin.write('r\n')`；`finishRun` 清 `hotReloadable`；`runSummary` 增补两字段。前端 `app.js`：tab 加两字段、SSE 监听 `status`、`renderTabs` 三态、`hotReload()`、`closeTab` 还在跑先 stop。样式 `.log-tab-hot`/`.log-tab-state`。
- 2026-07-17（热载验收）：机制真机验证——写 `r` 到 flutter run stdin，SM A156U 输出 `Performing hot reload... / Reloaded in 550ms` 成功。控制台端确认：`编译中`+`×` 渲染、就绪 SSE `status` 使 `hotReloadable=true`（/api/status 复核）。**遗留环境问题（非代码）**：本轮真机 USB/adb 连接反复掉线（`adb devices` 时有时无、`Lost connection to device`），导致 flutter run 掉设备早退、标签转 成功/失败 而非停留「热载」。设备链路稳定时全流程可用；citizenwallet-run.sh 无 citizenapp-run.sh 那套 adb 卡死自检，可后续补（但 adb 全空是物理断连，自检也救不回）。`node --check` 通过、launchd 冷启动干净。
- 2026-07-17（onchina PG locale——产品级 bug，非仅控制台）：经控制台启动链后设置页「启动」链上中国平台失败。根因=onchina 内嵌 PostgreSQL 启动时环境无有效 locale → macOS PG17 `FATAL: postmaster became multithreaded during startup`（[onchina/src/core/embedded_pg.rs](../../../GMB/citizenchain/onchina/src/core/embedded_pg.rs) 的 `run()` 拉 initdb/pg_ctl/postgres 完全不设环境，全链路无人设 LANG/LC_ALL）。**影响所有平台真实用户**：双击安装包启动（macOS LaunchServices）与 launchd 一样是无 `LANG` 精简环境，只有 Terminal 启动才带 locale。PATH 不是打包用户的坑（PG/onchina 随包绝对路径）。
- 2026-07-17（修复）：`embedded_pg.rs` 加 `apply_pg_locale(cmd)` 注入 `LC_ALL=C`/`LANG=C`，用于 `run()`（initdb/pg_ctl start/stop）+ `is_running()`（pg_ctl status），postmaster 由 pg_ctl 继承。选 `C`=三平台必带（macOS/Linux 无需生成、Windows 无害）、规避 macOS multithreaded、与集群 `--no-locale`(C 排序) 一致、UTF8 中文数据不受影响。一份共用 Rust 覆盖 4 个安装包。验收：`cargo build -p onchina` 通过；新二进制在 `env -i`（无 LANG）下内嵌 PG `ready to accept connections`（修前同场景 FATAL）；实测 `LC_ALL=C`/`en_US.UTF-8` 均 ready、无 locale 复现 FATAL。
