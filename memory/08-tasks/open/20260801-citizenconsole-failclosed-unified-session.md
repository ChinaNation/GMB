# CitizenConsole 故障可见 + 会话语义统一 + 验证失败即关闭

状态：open（2026-08-01）

## 背景

用户报告「公民控制台一直转圈打不开」，刷新 / 重开标签 / 重启浏览器 / 重启电脑全部无效。

### 已诊断根因（本次故障，已由用户跑 `start.sh build-production` 修复）

`.runtime/CitizenConsoleSecurity.app` 的密封资源被破坏。逐项校验 8874 项，**恰好 1 项**不匹配：

```
Resources/console/actions/cloudflare.sh
  签名时 sha256: 58466abab282d56bc52f629b068c75322a50517a4dfac86edb181c664e9b7a0f
  当前   sha256: 3be260d89728fbc3fc696e04f3a7f120bbca66e71f5fecdc3c2aa8fe8eb686dd
```

时间线：Jul 31 18:53 构建签名 → Aug 1 00:23 工作区改该文件 → Aug 1 00:36 **同一份新内容被直接拷进已签名 bundle**。
两份文件字节完全相同，但都不等于签名时封入的那份 → 密封破裂 → `AppMain.swift` 的
`SecStaticCodeCheckValidity`（带 `kSecCSCheckNestedCode | kSecCSStrictValidate`）必然失败 → `exit(1)`。

### 为什么表现是「转圈」而非「打不开」

plist 是 launchd **socket 激活**（`Sockets.Listeners` 127.0.0.1:8888，`RunAtLoad=false`）。
**监听 socket 由 launchd 持有**，于是浏览器一连上，TCP 三次握手就由 launchd 完成；app 自检失败
直接退出，**没人 accept 那条连接、没人写一个字节的 HTTP 响应** → 浏览器无限等待。

实测：`curl -m 6 http://127.0.0.1:8888/` → `http_code=000`，6.005 秒零字节；
同时 Chrome 挂着 3 条 ESTABLISHED 僵死连接。`launchd-error.log` 累积 52 次同一报错。

重启电脑同样无效：LaunchAgent 开机自动加载，socket 照旧监听，每次连接重跑一次注定失败的启动。

## 改动清单

### 一、故障可见（治「静默转圈」）

1. 新增 `security-app/Sources/CitizenConsoleSecurity/LaunchdFailureResponder.swift`
   —— 守卫失败时从 launchd 移交的 fd 排空 pending 连接，各回 HTTP 503 诊断页。
   - fd 0 设 `O_NONBLOCK`，循环 `accept` 到 `EAGAIN`，总时限 2 秒（刷新 N 次就有 N 条排队）
   - 写响应 → `shutdown(SHUT_WR)` → drain 到 EOF → `close`（不 drain 直接 close 会触发 RST，
     浏览器丢弃响应仍显示失败）
   - **零副作用**：不读请求内容、不执行任何东西、不碰钥匙串。此刻本程序签名已不可信，
     唯一被允许的动作是把失败原因告诉浏览器
2. `AppMain.swift` catch 分支接入
3. `socket-launcher.swift` 改 `@main enum` 并接入（多文件编译要求无 top-level code）
4. `start.sh` swiftc 行加 `-parse-as-library` 与共享文件；`package.json` check 加 socket-launcher

### 二、会话语义统一（六个动作收敛成一个结果）

5. `GET /` 作废 `consoleSessionToken` —— 刷新 / 关标签重开 / 重启浏览器 / 重启电脑
   走的都是这一条路径。只认 `GET /`，站内跳 `/citizenconsole.html` 不受影响
6. **生物验证不通过（失败或取消）即关闭控制台**（用户指令）：`/api/auth/unlock` 的
   `securityRequest` 抛错 → 回统一文案 → `killAllLocalRuns()` + `process.exit(0)`。
   不给重试窗口，把暴力尝试成本从「点一下按钮」提升到「重新触发 launchd 拉起进程」
7. `/api/shutdown` 补 `killAllLocalRuns()` —— 现在 `process.exit(0)` **不触发** SIGTERM handler，
   点「关闭」会把正在跑的编译变成孤儿，与 launchd 停服路径不一致
8. 删除「刷新状态」按钮（用户指令：浏览器 F5 已是同一效果，功能精简）

### 三、模块计数修正

9. `deployCount` 与 `ready` 去掉 `page` 排除 → 8 个（`pageIds` 整个删除）。
   `citizenconsole` 模块 `secrets` 全空，`configured([])` 返回 true → 天然计入就绪，「8/8」自洽

### 四、防复发

10. `start.sh` 加 `verify` 子命令 —— 现在 `verify_production` 只在 `run` 路径跑，
    失败信息全进看不见的 launchd log

## 已知代价（用户已确认）

- 刷新即需重新 Touch ID。对管密钥与发币的控制台是正确姿态，且正是用户要的「刷新 = 重启控制台」
- 单会话：多标签互踢。`consoleSessionToken` 是进程内单变量，刷新任一标签其余立即失效
- 主动「刷新状态」的需求本就不高：运行状态与日志由 SSE 自动同步

### 五、删除统一部署，只保留 44 台各自的独立部署（用户指令）

11. 删 `modules.citizenchain.actions` 里的 `{ id: 'deploy-all', title: '部署服务器' }`
12. 删 `startRun` 开头的批量分发点 `if (... action.id === 'deploy-all') return startBatchRun(...)`
13. 删 `startBatchRun`（59 行）与 `nodeChildEnv`（20 行）—— 两者都是批量独占：
    `startBatchRun` 只被那一处分发点调用，`nodeChildEnv` 只被 `startBatchRun` 调用。
    单节点部署在 `startRun` 里自行组装 `GMB_NODE_*`（`selectedNode` 分支），两条路径本就独立，
    删批量不触及要保留的独立部署
14. 动作脚本 `citizenchain.sh` 的 mode 白名单**本就不含 `deploy-all`**（第 7 行只放行 7 个 mode），
    无需改动，天然 fail-closed；只订正一处与被删按钮同名的注释措辞（「部署服务器」→「节点部署」）

保留的唯一部署路径：节点卡片 → `runAction('citizenchain','deploy',{nodeId})` → `startRun`，
必须选中有效节点才读密钥；`deploy` 为 `production: true`，受 `anyProductionRunActive()` 串行保护，
**44 台一次只能部署一台**。这正是删批量的目的：一次误操作不再能同时改动全部权威引导节点。

新增回归测试锁死该约束，全仓禁止再出现 `deploy-all` / `startBatchRun` / `nodeChildEnv`。

### 六、本机数据清理三按钮重整（用户指令）

| 原 | 现 |
| --- | --- |
| 清空链数据 `clear-chain-data` | **删除**（按钮 + `clear_local_chain_data` + mode 白名单 + case 分支） |
| 清空节点数据 `clear-node-data` | **清空非链数据** `clear-non-chain-data`（行为不变：只清 OnChina 内嵌 PG） |
| 删除本机全部旧数据 `reset-formal-local-data` | **删除全部数据** `delete-all-data`（改为保留 TLS 的逐项删除） |

标题改了 id/mode 也一并改：留 `clear-node-data` 这种与显示名不符的内部名就是命名残留。

#### 关键冲突：TLS 证书在 gmb.dev 里面

`citizenchain/scripts/run.sh:61` 把 `ONCHINA_TLS_DIR` 设为
`$HOME/Library/Application Support/gmb.dev/onchina-tls`。原实现是 `rm -rf` 整个 `gmb.dev`，
**会连 TLS 一起删掉**，与「保留 TLS 证书」的新要求直接冲突。故当前目录改为逐项遍历删除。

`gmb` 与 `gmb.dev` 下**各有两个** TLS 目录，性质不同，均保留：

- `onchina-tls/` —— OnChina 的 CA 根证书与私钥（`onchina-org-root-ca.key`）。
  **删掉不可逆**：重新生成即是另一个 CA，已签发的机构证书全部作废。
- `tls/` —— 节点 libp2p 传输层自签证书（`cert.der`/`key.der`）。
  `node/src/core/tls_cert.rs` 写明「TLS 层只负责传输加密，身份认证由 Noise 协议通过 peer ID 完成」，
  缺失时自动重建。

两者都不含链状态、不含节点身份。节点身份在 `chains/citizenchain/network/secret_ed25519`，
属区块链数据，**随删除全部数据一并清除**（PeerId 会变，本机开发节点可接受）。

`china.sqlite` 在仓库内（`citizenchain/onchina/src/cid/china/china.sqlite`），属冻结资产，天然不受影响。

**本机并行两套节点数据**，由 `node/src/shared/security.rs` 按构建 profile 分目录：

| 目录 | 归属 | 常量 |
| --- | --- | --- |
| `gmb` | 正式安装版 | `PROD_APP_DATA_DIR_NAME` |
| `gmb.dev` | `scripts/run.sh` 开发版 | `DEV_APP_DATA_DIR_NAME` |

两者都是**当前**目录，同规则处理：保留 TLS、其余逐项删。

**已修正的一处误判**：初版把 `gmb` 写进了「旧产品遗留目录整体删除」清单，注释还写着
「当前产品不读取它们」。实际 `lsof` 抓到正式版进程正在读写 `gmb/chains`——它是活的。
后果是点「删除全部数据」会连正式版的 `tls/` 一并删掉，与用户明确要求的「两个 TLS 目录
都不许删」直接冲突。当时只看到 `gmb` 与 `gmb.dev` 并存、后者时间戳更新就下了结论，
没有查 `security.rs` 的 profile 常量。

旧产品目录现在只剩两个真正废弃的旧 bundle id（`org.chinanation.citizenchain.desktop`、
`org.citizenappchain.desktop` 的 Application Support 与 Containers），**整体删除**。

#### 安全设计

- 遍历根**硬编码**两条（`gmb` 与 `gmb.dev`），不用可被 `GMB_APP_DATA_DIR` 覆盖的
  `$app_data_dir`：那个变量一旦被改写，整目录遍历就会指向任意位置
- `shopt -s dotglob` 让隐藏项也进入遍历，否则点开头的残留数据被静默留下；`nullglob` 防空目录产生字面 `*`
- 每项删除后复验 `[[ ! -e ]]`，未删净立即 exit 1
- 旧目录仍走 case 白名单 + Finder 兜底（不清空整个废纸篓）

#### 充值台账绝不删除（用户质疑「账本不能删除」→ 查实为资金安全漏洞）

原实现（沿用自 `reset_formal_local_data`）会删 `.runtime/topup-ledger.json`。用户质疑后查实，
问题比「审计记录丢失」严重得多：

**台账是防重复发币的唯一依据。** `topup/routes.mjs:358` 幂等①：

```js
const prior = await getLedgerEntry(ctx.consoleDir, order.order_id);
if (prior?.gmb_tx_hash && prior?.gmb_block_hash
    && Number.isSafeInteger(prior?.gmb_extrinsic_index) && prior?.signed_extrinsic_hex) {
  // 已记完整发币证明 → 只重试回写，绝不重复发币
```

台账一删，已发币订单 `prior` 读回 null，幂等判定失效，**会重新发一次币**。

`topup/ledger.mjs:24` 的注释早已警告过这件事——「损坏、权限错误和 IO 错误必须 fail-closed，
否则把既有发币记录误当作空表会造成重复发币」。作者对**文件损坏**做了 fail-closed，
但**删除文件**走的是 `ENOENT → return {}` 这条「首次运行」正常路径，正好从旁边绕过了那道保护。

处置：删除动作里删台账的整段代码移除；`assert_deletable` 随之**收紧为仓库内一律拒绝**
（原本为台账开的 `.runtime` 例外不再需要），守卫因此没有任何例外，更简洁也更安全。

#### 代码不可删守卫 `assert_deletable`（用户指令：不允许按钮删掉任何代码和代码里的数据库）

先查清事实：`baseChildEnv()` 是严格白名单（`HOME/USER/LOGNAME/SHELL/TMPDIR/LANG/LC_ALL/SSH_AUTH_SOCK`
+ PATH + `GMB_ROOT` + node 路径），**`GMB_APP_DATA_DIR` 与 `ONCHINA_PG_DATA_DIR` 都不在其中**，
子进程拿不到，故点按钮这条路径上 `app_data_dir` 恒为 `gmb.dev`，环境变量污染不可能发生；
`china.sqlite` 也从来不在任何删除目标里。

但「恰好没删」不等于「不可能删」。新增 `assert_deletable`，清理路径上**每个 rm 之前必过**（4 处）：

- 拒绝仓库内任何路径（`$GMB_ROOT` 及其子路径），含 `citizenchain/onchina/src/cid/china/china.sqlite`
- 拒绝 `/`、`$HOME`、`$HOME/Library`、`Application Support` 根、`Containers` 根
- 先 `cd … && pwd -P` 解析符号链接再比对前缀，防止用软链把仓库挂进清理目标绕过前缀判断
- **无任何例外**：`citizenconsole/.runtime` 也在拒绝之列——充值台账就在那里

#### 实测验证

**保留逻辑**：scratchpad 造假目录跑同一段遍历（绝不碰真实 `gmb.dev`）——
`.hidden-state`、`chains`、`cold-wallets.json`、`onchina-pgdata`、`security-audit.log` 全删，
`onchina-tls` 与 `tls` 保留且内容完好。

**守卫**：抽出函数单独跑用例，全部符合预期。拒绝 `china.sqlite`、仓库根、仓库内源码目录、
**软链穿透进仓库**、`$HOME`、`Application Support` 根、`.runtime/topup-ledger.json` 与
`.runtime` 内任何路径；放行 `gmb/chains` 与 `gmb.dev/chains`。

**测试有效性（变异验证）**：「每个 rm 前必须有守卫」这条断言，在原文件检出 0 漏网；
分别去掉 `entry` 守卫、去掉 `pg_data_dir` 守卫、新增一个裸 `rm` —— 三种变异全部被检出，
证明不是永远通过的假断言。

#### 文档同步

两处描述现状的技术文档已订正（`done/` 归档属历史记录不动；其余「不部署服务器」是泛指 CI 行为，与按钮无关）：

- `05-modules/citizenchain/node/NODE_TECHNICAL.md` —— 原文描述「部署服务器会并发启动所有配置齐全节点」
- `01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md` —— 原文以「部署服务器」指代生产入口

## 埋雷点（差点让 build-production 白跑一次）

Xcode 工程用的是**显式文件列表**（`PBXSourcesBuildPhase.files` 逐个列出），
**不是** Xcode 16 的文件系统同步组（`PBXFileSystemSynchronizedRootGroup` 计数为 0）。
新增的 `.swift` 放进 `Sources/` 不会被自动编译，`AppMain` 引用它会报
`cannot find 'LaunchdFailureResponder' in scope`。已手工注册 4 处：
`PBXBuildFile`（`B10000000000000000000008`）、`PBXFileReference`（`B10000000000000000000021`）、
`PBXGroup(Sources).children`、`PBXSourcesBuildPhase.files`。
**今后本工程新增 Swift 文件必须同样手工注册。**

## 验证结果（2026-08-01）

| 项 | 结果 |
| --- | --- |
| `npm run check`（含新增的启动器 typecheck） | 通过 |
| `npm test` | 40 passed / 0 failed（新增 2 条安全回归） |
| 启动器真实编译（`@main` + 多文件链接） | 通过，产出 105KB 二进制 |
| Xcode 真实构建（`CODE_SIGNING_ALLOWED=NO`，产物落 scratchpad） | **BUILD SUCCEEDED**，`LaunchdFailureResponder.swift` 出现在 SwiftCompile 列表 |
| 构建期间正在服役的签名 app | 未受影响：密封 8874 项零异常、HTTP 200 / 0.9ms |
| `process.exit(0)` 出现处 | 恰 4 处：idle 退出、SIGTERM、SIGINT、closeConsole |

Xcode 验证特意把 `CONFIGURATION_BUILD_DIR` 指向 scratchpad —— 用默认值会直接覆盖
`.runtime/CitizenConsoleSecurity.app`，那是刚签好、正在服役的 app，覆盖后未签名即当场再次故障。

## 生效方式

改动只在工作区，app 内仍是旧代码。必须执行：

```
cd ~/GMB/citizenconsole && bash start.sh build-production
```

**该命令需钥匙串授权，只能由用户在终端执行**（死规则：绝不代启动需 Touch ID/GUI 的程序）。
