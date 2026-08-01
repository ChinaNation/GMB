# CitizenConsole 解除 Developer ID 与公证依赖

状态：done（2026-07-31 已构建、启动并验证通过）

## 起因

`joy_rhett@icloud.com`（887WZMJ6Q3）付费会员有效，但 Apple 推送了新版 Program License
Agreement 未接受，账号全部会员资源被锁：

```
Unable to process request - PLA Update available
You currently don't have access to this membership resource.
```

无法签发 Developer ID Application 证书 → 控制台无法构建 → Cloudflare 生产部署
（[20260731-cloudflare-redeploy-current-genesis.md](20260731-cloudflare-redeploy-current-genesis.md)）持续阻塞。

用户决定：把控制台改成不依赖正式开发者会员的证书。

## 方案

签名身份从 **Developer ID Application + Apple 公证** 降级为 **Apple Development**
（本机已有 `Apple Development: joy_rhett@icloud.com (887WZMJ6Q3)`，有效，不受 PLA 阻塞）。

Team 仍是 887WZMJ6Q3，`keychain-access-groups` 不变 → **Keychain 19 项无需重录**。

## 保留 / 移除

保留（防篡改主体不动）：

- 源码密封：`server.mjs`、`actions/`、`topup/`、`rtupg/`、`web/`、`node_modules`、`node`
  仍打进签名 app，改脚本仍须重新构建
- `codesign --verify --deep --strict` 完整嵌套资源校验
- Hardened Runtime（`ENABLE_HARDENED_RUNTIME = YES` + 运行时 `signatureFlags & 0x0001_0000`）
- 拒绝 `get-task-allow`（`CODE_SIGN_INJECT_BASE_ENTITLEMENTS = NO` + entitlements 不声明 + 运行时复验）
- `AppMain.swift` 运行时自校验：`anchor apple generic` + 固定 `identifier` + 固定 `subject.OU`
- `keychain-access-groups = 887WZMJ6Q3.com.gmb.citizenconsole.security`

移除（按 [[no-remnants]] 彻底删除，不留注释桩）：

- `xcrun notarytool submit --keychain-profile CitizenConsoleNotary --wait`
- `xcrun stapler staple` / `stapler validate`
- 公证打包（`CitizenConsole-notarize.zip`、`notary-payload`）
- `spctl --assess`（Apple Development 签名未公证，Gatekeeper 评估必然 rejected）
- `AppMain.swift` requirement 中两条 Developer ID 专属 OID
  （`1.2.840.113635.100.6.2.6`、`1.2.840.113635.100.6.1.13`）

OID 两条直接删而非替换为 Apple Development 对应 OID：`anchor apple generic` 已保证证书链
锚定 Apple，叠加固定 `identifier` 与 `subject.OU = 887WZMJ6Q3` 后约束已足够，无需赌具体 OID 值。

## 安全代价（用户已知悉并决定）

- 失去 Apple 公证背书与 Gatekeeper 放行，产物只在本机可信
- Apple Development 证书有效期 1 年，到期需重新签发（免费 Apple ID 亦可签发此类证书）
- 开发期零用户（[[in-development-zero-users]]），控制台为本机运维工具，风险可控

## 改动清单

| 文件 | 改动 |
| --- | --- |
| `citizenconsole/start.sh` | `developer_id_identity` → `signing_identity` 找 Apple Development；删公证全段；`verify_production` 去掉 stapler/spctl、authority 断言改 Apple Development |
| `citizenconsole/security-app/Sources/CitizenConsoleSecurity/AppMain.swift` | requirement 删两条 Developer ID OID；注释同步 |
| `citizenconsole/security-app/CitizenConsoleSecurity.xcodeproj/project.pbxproj` | `CODE_SIGN_IDENTITY` → `"Apple Development"` |
| `citizenconsole/test/production-security.test.mjs` | 首个用例断言改写，并反向断言 notarytool/stapler/spctl 已清零 |
| `citizenconsole/test/biometric-security.test.mjs` | 33/34 行签名断言同步 |

`CitizenConsoleSecurity.entitlements` 不改。`citizenapp/ios/` 不改。

## 执行结果（2026-07-31，已构建并跑通）

控制台已启动：`http://127.0.0.1:8888` HTTP 200，标题「公民控制台」。
签名实测 `Identifier=com.gmb.citizenconsole.security`、`TeamIdentifier=7QJXLLBA6J`、
`flags=0x10000(runtime)`、`Authority=Apple Development: joy_rhett@icloud.com (887WZMJ6Q3)`。
`npm run check` 通过，`npm test` 37 passed / 0 failed。

落地过程中撞到四个必须解决的实测问题，逐一记录以免重踩：

**① Team ID 一度被改错。** 证书 subject 为
`UID=XTXKBD344F, CN=Apple Development: joy_rhett@icloud.com (887WZMJ6Q3), OU=7QJXLLBA6J`。
Team ID 是 **OU 字段的 `7QJXLLBA6J`**，CN 括号内的 `887WZMJ6Q3` 只是证书标识。
先前误把后者当 Team ID 并改了 4 处，已全部改回。**Keychain 访问组因此从未变化，
19 项 Secret 无需重录**，[20260731-citizenconsole-team-id-switch.md](20260731-citizenconsole-team-id-switch.md)
基于误判而立，应作废。

**② Xcode 拒绝用 Apple Development 证书手动签名**（`requires a provisioning profile`），
而描述文件依赖被 PLA 锁住的会员资源。改为 `CODE_SIGNING_ALLOWED=NO` 构建 +
构建后 `codesign` 直接签名：本机运行不校验描述文件。

**③ 声明 `keychain-access-groups` 导致 AMFI 在 exec 阶段静默 SIGKILL**
（returncode -9，无任何 stdout/stderr，系统日志亦无记录）。对照实验确认：
去掉 entitlements 后进程正常启动。根因是 Apple Development 签名的 entitlement
需要描述文件授权。**故 Keychain 改用签名标识默认访问组，已存 19 项需重录一次。**
`verify_production` 已加正向拦截，声明该键即拒绝启动。

**④ Hardened Runtime 下内嵌 Node 的 V8 无法保留 CodeRange 虚拟内存**
（`Fatal process out of memory`）。`com.apple.security.cs.*` 属 Hardened Runtime 例外，
**不需要描述文件授权**，与 ③ 不是一类，故声明 `allow-jit` 与
`allow-unsigned-executable-memory` 即可，Hardened Runtime 得以保留。
两项必须签在 `Contents/Helpers/node` 自己身上——node 是独立进程，签在 app 本体无效。

另修 `verify_production` 中新增检查的写法：`set -e` 下 `plutil` 找不到键会直接终止
脚本，必须用 `if` 包裹而非 `cmd && { ... }`。

## 追加：Touch ID 弹不出来的根因（2026-07-31）

点「使用 Touch ID 打开」一直转圈、控制台随后整个打不开，根因是
`INFOPLIST_KEY_LSBackgroundOnly = YES`。

该键声明本程序不显示任何 UI，macOS 据此阻止它呈现任何对话框，**Touch ID 对话框也在内**。
`securityRequest` 是完全同步的（`writeSync` + 死循环 `readSync`，server.mjs:279），
`evaluatePolicy` 不返回就把 Node 单线程事件循环整个冻住，于是端口不再 accept、
已有连接也不响应，表现为「控制台打不开」。

已从 Debug/Release 两处删除该键，只保留 `INFOPLIST_KEY_LSUIElement = YES`
（无 Dock 图标、无菜单栏，但允许弹对话框），并加测试断言禁止它被加回。

同批清理 `~/Library/LaunchAgents/com.gmb.deploy-console.plist` 的 `ProgramArguments`
残桩：原本还带着 `1 => nvm 的 node`、`2 => 工作区未密封的 server.mjs`。
`socket-launcher.swift:5` 已明写这两个参数会被忽略（它用 `arguments[0]` 推导同目录的
签名二进制再 execv），故直接删至只剩 socket-launcher 一项。备份在本次会话 scratchpad。

**Claude 不得代为启动控制台**：用 Bash 工具起的实例不在用户登录 GUI 会话里，
Touch ID 同样弹不出来，且会抢占 8888 让 launchd 的 socket 激活失效。
正确方式是由 `com.gmb.deploy-console`（`RunAtLoad=false` + `ProcessType=Interactive`
+ Sockets 8888）在用户访问时按需拉起。

## 后续（2026-07-31 当日完成）

本卡把签名从 Developer ID 降级为 Apple Development 后，`keychain-access-groups`
仍需描述文件授权，等于把密钥绑在一张 8/3 到期的 profile 上。该依赖已由
[20260731-citizenconsole-keychain-migration.md](20260731-citizenconsole-keychain-migration.md)
彻底解除：密钥迁至传统钥匙串，entitlements 只剩两项 `cs.*`，构建不再嵌入描述文件，
不依赖任何 Apple 开发者账号资源。

本卡遗留的「PLA 接受后可反向恢复 Developer ID + 公证」不再建议执行——恢复
`keychain-access-groups` 会重新引入账号绑定，启动守卫现已反向拦截该配置。

## 待用户执行

```
cd /Users/rhett/GMB/citizenconsole && ./start.sh build-production && ./start.sh
```

## 待实测点

Apple Development 签名 + `CODE_SIGN_STYLE = Manual` + 空 `PROVISIONING_PROFILE_SPECIFIER`
搭配 `keychain-access-groups` 是否被 codesign 接受。macOS 非沙箱应用的该 entitlement
不属受限项，签名时只写入不校验描述文件，运行时 Keychain 仅比对签名 team 前缀，判断可行；
若 xcodebuild 报描述文件相关错误，改为 `CODE_SIGN_STYLE = Automatic` 让 Xcode 自动生成
Mac Development 描述文件。

## 后续

PLA 接受后若要回到 Developer ID + 公证，按本卡改动清单反向恢复即可；本卡不保留任何过渡开关
（[[no-compatibility]]）。
