# 公民控制台「编译」按钮：源码改动免手工构建

任务需求：
在公民控制台右上角「关闭」按钮左侧新增「编译」按钮。点击后用当前源码重新编译签名包，
只有两种结局：

- **成功** → 换包 → 控制台退出 → 浏览器重连由 launchd 用新包拉起 → 进入新控制台
- **失败** → 不换包、不退出，日志留在页面上，去修复代码

绝不允许出现「编译失败却进了旧包的控制台」这第三种状态。

所属模块：citizenconsole（不在 Git 版本库内）

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 背景

`consoleDir = dirname(import.meta.url)`（server.mjs），服务端读的是**签名 App 包内的拷贝**，
不是源码。改了 `citizenconsole/` 下任何文件，不重新构建就不生效；浏览器刷新、重启、
换标签一律无效。静态资源虽然是 `cache-control: no-store` 且每请求 `readFile`，
但读的仍是包内路径。

## 关键约束

1. **不能原地重编译**：控制台正跑在 `.runtime/CitizenConsoleSecurity.app` 里，
   `codesign --force` 原地重写主二进制，运行中的进程缺页时会被 AMFI SIGKILL。
   → 构建到 `.runtime/next/`，全部校验通过后整目录 `mv` 换包。
   `mv` 换的是新 inode，运行中的进程仍持有旧 inode，不受影响。
2. **不能先退出再编译**：失败时旧包完好，launchd 会把旧控制台拉起来，
   等于「失败也进控制台」，违反需求。→ 编译期间控制台保持存活，成功才退出。
3. **必须过 Touch ID**：写签名包等同于替换整条安全边界。没有 Touch ID，
   「能写仓库」就直接等于「能把任意代码装进签名包」。
4. **编译进程不得 detached**：必须与控制台同生共死，
   否则关掉控制台后会留一个脱缰进程在背后把包换掉。
5. `ERR` trap 默认不被函数继承，`set -euo pipefail` 必须加 `-E`，
   否则 `build_production_into` 里失败时 `next/` 半成品不会被清理。

## 落地

| 文件 | 改动 |
|---|---|
| `start.sh` | `set -Eeuo pipefail`；`build_production` / `verify_production` 参数化为 `*_into` / `*_at <dir>`；新增 `build_staged`（构建 next/ → 校验 next/ → 换包）；新增 `build-staged` 分支 |
| `server.mjs` | 新增 `startRebuildRun()` + `POST /api/rebuild`（`validOrigin` + `authorizeProduction` + 跑源码目录的 `start.sh build-staged`，退出码 0 才 `closeConsole()`） |
| `web/index.html` | `#closeConsole` 左侧插 `#rebuildConsole`「编译」按钮 |
| `web/app.js` | `openRunTab` 增加 `onDone` 回调并在 `done` 事件里触发；编译按钮 handler；`awaitRebuiltConsole()` 轮询 8888 等 launchd 拉起新包 |

跑的是**源码目录**的 `start.sh`（`stateDir`），不是包内拷贝——包里根本没有 `start.sh`，
且改了 `start.sh` 也要立刻生效。

## 首次生效

按钮本身在包里，所以**首次必须手工跑一次**：

```
cd citizenconsole && ./start.sh build-production
```

之后即自举，改任何代码点「编译」即可。

## 未验证项

`build-staged` 的真实执行未在本任务内跑过：它需要 `security find-identity` 读签名证书、
`xcodebuild`、`codesign --sign`，属会弹 GUI / Touch ID 的链路，AI 不得代跑。
语法、分支、用法输出已验证；`npm test` 48/49（唯一失败是既有的 `citizenapp.sql`
缺 `SCHEMA VERSION`，与本任务无关）。

验收标准：
- 首次手工 `build-production` 后，点「编译」能走完整条链路
- 故意写坏 `server.mjs` 再点「编译」，必须停在失败态、不换包、不退出
- 改对后再点「编译」，必须自动进入新控制台且改动生效
