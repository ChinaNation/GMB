# CitizenConsole 编译软件卡「无效本机会话」修复（runAction 补 403 整页刷新自愈）

任务需求：控制台点 CitizenApp/CitizenWallet「编译软件」，进程 idle 重启后旧会话 cookie 失效，
`/api/run` 返 403「无效本机会话」，但 `runAction` 未像其它 fetch 那样整页刷新重种 cookie，导致卡在
「等待执行部署任务…\n无效本机会话」。补上与其它三处一致的 403 自愈。
所属模块：citizenconsole（公民控制台前端）

## 根因（已只读诊断确认）

- `sessionToken` 每次进程启动重生（`server.mjs:21`），cookie 只在壳页 GET `/` 时种（`server.mjs:715`），
  所有 `/api/*` 过会话校验（`server.mjs:718`，含 `/api/run` @ `server.mjs:778`）。
- 旧标签页对上一个进程种的 cookie，进程重启后失效 → 403「无效本机会话」。
- `loadStatus`(`app.js:706`)、`loadReconcileFlags`(`app.js:496`)、`postJson`(`app.js:638`) 都有
  `403 → location.reload()` 自愈；**唯独 `runAction`(`app.js:689`) 没有** → 卡住。

## 存在性确认

- CitizenApp 编译软件 = `server.mjs:119`（module id `citizenapp`, action `build-install`）。
- CitizenWallet 编译软件 = `server.mjs:128`（module id `citizenwallet`, action `build-install`）。**存在。**
- 两者都由 `app.js:304` 绑到同一个 `runAction` → **一处修复覆盖两者及全部部署动作**。

## 改动（唯一）

- `citizenconsole/web/app.js` `runAction` 的 `!response.ok` 分支最前面补：
  `if (response.status === 403 && result.error === '无效本机会话') { location.reload(); return; }`
  与 `loadReconcileFlags`/`postJson`/`loadStatus` 三处逐字一致。

## 不在本次范围（已向用户标注）

- `stopRun`(`app.js:181`)、`hotReload`(`app.js:189`) 有相同缺口，但属「停止/热载」按钮、非「编译软件」，
  用户未要求，暂不动（用户点头再一并补）。
- 不动 server 端会话机制（cookie/token 设计本身是有意的，见 `server.mjs:712-713` 注释）。

## 验收标准

- `node --check citizenconsole/web/app.js` 通过（该目录为原生 JS，无构建步骤）。
- 四处 fetch（loadStatus/loadReconcileFlags/postJson/runAction）403「无效本机会话」处理一致。
- 逻辑：会话失效时「编译软件」自动整页刷新重种 cookie，而非卡在报错。

## 执行结果（2026-07-21）

- `citizenconsole/web/app.js` `runAction` 的 `!response.ok` 分支补入
  `if (response.status === 403 && result.error === '无效本机会话') { location.reload(); return; }`
  （`app.js:692`），与 `loadReconcileFlags`(496)/`postJson`(638)/`citizenconsole.js`(33) 逐字一致。
- `node --check web/app.js` 通过；四处 403「无效本机会话」自愈已对齐。
- 覆盖 CitizenApp + CitizenWallet 两个「编译软件」及所有经 `runAction` 的部署动作（一处改，全覆盖）。
- 未跑浏览器预览：CitizenConsole 是 127.0.0.1 本机部署控制台（server.mjs 触碰 keychain/TouchID/launchd
  与真实编译部署），复现 idle 重启的失效会话不安全也不实际；改动是纯 403 分支、镜像三处既有自愈，
  以语法检查+跨点一致性验证为准。
- 未动 `stopRun`/`hotReload` 同类缺口（非「编译软件」，待用户确认）。

### 追加（2026-07-21 续，用户「都修复」）

用户要求把同类缺口全部补齐。审计 app.js 所有直连 `fetch(` 与经 `postJson` 的会话守卫路径，逐一对齐 403 自愈：

- **新增自愈**：`stopRun`(`app.js:185`)、`hotReload`(`app.js:195`)、`saveNode`(`app.js:676` 节点配置 POST)。
- **已覆盖（本次确认无需再改）**：`runAction`(698)、`loadReconcileFlags`(500)、`postJson`(642，覆盖
  secret/set·secret/delete·rtupg latest-wasm/build-request/submit·reconcile POST)、`loadStatus`(713 catalog 403→reload)、`citizenconsole.js`(33 充值发币页)。
- **有意排除**：`#closeConsole` 的 `/api/shutdown`(`app.js:754`)——该处**故意忽略响应**（注释「连接中断属预期,忽略」），且「关闭控制台」动作上做 reload 自愈属错误 UX；如需让它在失效会话下提示未关闭，另议。
- 验证：`node --check web/app.js` 通过；app.js 共 6 处 + citizenconsole.js 1 处 403「无效本机会话」自愈，模式逐字一致。
