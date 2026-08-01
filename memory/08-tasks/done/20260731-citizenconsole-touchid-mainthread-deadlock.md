# CitizenConsole Touch ID 主线程死锁修复

状态：done（2026-07-31 方案 A 已落地并随控制台一同验证通过）

> 订正：本卡修的主线程死锁是真实缺陷（同步阻塞 + 无超时），但**不是**「点 Touch ID
> 转圈」的直接原因。真正挡在最后的是私有通道读取语义 bug，见
> [20260731-citizenconsole-socketpair-read-semantics.md](20260731-citizenconsole-socketpair-read-semantics.md)。
> 通道修好后请求才第一次送达原生进程，本卡的 run loop 等待与 120 秒超时同批生效。

## 缺陷

点「使用 Touch ID 打开」后一直转圈，指纹对话框始终不出现，随后整个控制台不响应。

`sample` 抓取原生进程主线程栈实证：

```
com.apple.main-thread
  main → CitizenConsoleSecurityApp.main() AppMain.swift:50
    → AuthenticatedChannel.run() :68
      → AuthenticatedChannel.serve(on:) :78
        → read (libsystem_kernel)
```

主线程即 `serve` 的 `while true` 同步读 fd 循环，`AppMain.swift` 只 `import Foundation`
与 `Security`，无 NSApplication、无 run loop。请求到达后 `catalog.handle` 就在该主线程
执行，进入 `KeychainVault.authenticate`：

```swift
let semaphore = DispatchSemaphore(value: 0)
context.evaluatePolicy(...) { success, _ in accepted = success; semaphore.signal() }
semaphore.wait()
```

死锁链：主线程被 `semaphore.wait()` 阻塞 → Touch ID 对话框需主线程 run loop 才能呈现
→ 对话框弹不出 → completion 永不回调 → semaphore 永不 signal → 主线程永久卡死 →
`securityRequest`（server.mjs:279 同步 `writeSync`+`readSync`）随之冻结 Node 事件循环 →
端口不再 accept、已有连接无响应 → 表现为「控制台打不开」。

`canEvaluatePolicy` 与 `biometryType == .touchID` 均通过（否则会直接抛「当前设备无法使用
Touch ID」而非转圈），Touch ID 硬件正常。

## 缺陷来源

2026-07-26 `20260726-citizenconsole-production-hardening.md` 把「可由任意同用户进程调用的
通用密钥 CLI（security-broker）」改为「常驻签名 app + 匿名 socketpair 私有通道」时引入。
该卡执行清单最后一项 `[ ] 完成 Developer ID 正式签名、公证、真实 Touch ID 和篡改拒绝验收`
从未打勾——`.runtime/` 一直没有构建产物，此死锁从未被执行到，直到 2026-07-31 首次构建成功
才暴露。

## 方案选择

- **A（选定）**：`authenticate()` 内改用 run loop 等待并加超时护栏。改动一个函数，
  请求处理保持严格串行，等待期间不读 fd，天然无重入；不触碰任何安全边界。
- B：fd 读循环移至后台线程、主线程跑 `RunLoop.main.run()`。同样能修，但把严格串行
  单线程变成并发模型，需额外保证串行化，出错面大于 A。
- C（否决）：退回独立 broker 进程。等于把 7/26 修掉的「任意同用户进程可调用、
  可指定任意提示文案读取任意密钥」漏洞放回来。

第四条路（由签名 app spawn 短命子进程弹 Touch ID）不成立：`authenticate()` 返回的
`LAContext` 需供后续 `kSecUseAuthenticationContext` 使用，而 LAContext 无法跨进程传递；
要走通必须把全部 Keychain 操作搬进子进程，即退化为 C。

## 改动

`citizenconsole/security-app/Sources/CitizenConsoleSecurity/KeychainVault.swift`

1. 信号量等待改为 run loop 等待，使系统能在主线程呈现 Touch ID 对话框。
2. 新增 120 秒超时护栏：对话框未被响应时只让本次请求失败，不再拖垮整个安全进程。
   这是本次故障的放大器——原实现同步阻塞且无超时，单次未响应即冻结全部服务。
3. 跨线程标志用 `NSLock` 保护，不裸读写 Bool。

安全边界全部保持：匿名 socketpair 私有通道、9 项固定操作目录、提示文案写死在程序内、
签名密封资源、发币私钥不进普通 Node 进程。

## 执行结果（2026-07-31）

`KeychainVault.swift` 三处改动已落地：新增 `AuthenticationOutcome`（NSLock 保护跨线程标志）、
`authenticationTimeoutSeconds = 120`、等待改 `RunLoop.current.run(mode:before:)`。
`biometric-security.test.mjs` 加断言钉住：必须含 `RunLoop.current.run(mode:`、
必须含 `authenticationTimeoutSeconds`、禁止出现 `DispatchSemaphore`。

验收：`npm run check` 通过（含 `swiftc -warnings-as-errors` 类型检查）；
`npm test` 37 passed / 0 failed；`build-production` 签名与完整性验收通过；
launchd 重载后 socket 激活正常，首页 200，`/api/auth/status` 返回 `{"unlocked":false}`。

真实 Touch ID 弹窗验收待用户执行——该步骤 Claude 不得代做。

## 待用户执行

`./start.sh build-production` 由 Claude 完成；**启动与按指纹由用户执行**
（[[never-launch-touchid-apps-for-user]]）。
