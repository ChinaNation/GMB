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
