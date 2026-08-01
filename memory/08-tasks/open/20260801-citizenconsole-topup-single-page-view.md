# CitizenConsole 充值发币改同页视图（消除整页跳转与返回重验证）

状态：open（2026-08-01）

## 背景：一个由本轮改动引入的回归

同日新增规则 `GET / → consoleSessionToken = null`（刷新即重启控制台）落地后，出现行为不一致：

- 其余 7 个模块（Cloudflare / CitizenWeb / CitizenChain WASM / CitizenApp / CitizenWallet /
  CitizenChain / GitHub）走 `dialog.showModal()` 弹窗，**从不离开首页、一个请求都不发**；
- 只有 `citizenconsole`（充值发币）带 `page: '/citizenconsole.html'`，是**唯一的整页模块**。
  从它返回首页会发一次 `GET /` → 清会话 → 锁屏 → **强制重新生物验证**。

设计时误判：当时认为「站内跳 `/citizenconsole.html` 不受影响」，只考虑了去程没考虑回程；
又因看到 `citizenconsole.js` 底部「优先 `history.back()` 让主页从 bfcache 原样恢复」的注释，
误以为返回不发文档请求。**实际充值发币页解锁后持有活跃流式连接（页面生命周期锁），
该页面不合格进入 bfcache**，`history.back()` 退化为真实导航，必然发出 `GET /`。

从子页返回首页并非新会话，重新验证无任何安全收益，只有摩擦。

## 方案（用户已确认方案 A）

把充值发币从独立文档改为首页内的**全屏视图**，hash 路由切换，站内进出零请求。

```
/          → 控制台首页
/#topup    → 充值发币视图（同一份文档）
```

**会话规则一行不改**：刷新 `/#topup` 仍发 `GET /` → 清会话 → 锁屏，「刷新 = 重启控制台」
语义完整保留；站内切换不发请求，自然不触发。不给规则打补丁，而是消除产生例外的结构。

### 可行性实测结论（动手前）

- **id 零冲突**：`index.html`（`consoleLock/consoleMain/cards/moduleDialog/...`）与
  `citizenconsole.html`（`unlock/configTable/ledgerBody/...`）完全不重叠，无需重命名
- 规模小：76 行 HTML（含 24 行 `<style>`）+ 388 行 JS
- 样式自带 `.tp-*` 前缀，并入 `styles.css` 无冲突

### 改动清单

1. `index.html` 增 `#topupView`（默认 hidden），搬入三个 `.tp-section`；
   `backToConsole` 由 `<a href="/">` 改 `<button>`（不再是导航）
2. `styles.css` 收编 24 行 `.tp-*`，删内联 `<style>`
3. `app.js` 增 hash 路由 + 动态 `import('/citizenconsole.js')`（19KB 按需加载，首页体积不变）
4. `citizenconsole.js` 顶层自执行 `loadConfig(); loadLedger();` 改导出 `mount()/unmount()`
5. `server.mjs`：`page: '/citizenconsole.html'` → `view: 'topup'`；serveStatic 删
   `/citizenconsole.html`（**保留** `/citizenconsole.js`，动态 import 要用）；
   删「未登录访问该页 → 303 回 `/`」分支
6. 删 `web/citizenconsole.html`

## 最高风险点：切回首页必须清空内存密钥（用户明确强调）

现有生命周期锁只绑 `pagehide`。改同页后**离开视图不再触发 `pagehide`**，若不处理会造成：
解锁（后端持内存密钥）→ 切回首页 → 视图隐藏但流式连接仍在 → **密钥继续留在后端内存**。
这是实打实的安全回归。

### 后端释放路径（已查实，无需改动）

```
前端 abort 连接 → res.on('close') → requestSessionLock() → clearSession()
   clearSession: holdResponse.end() + nativeLock()(原生托管时) + seedBytes.fill(0) + topupSession=null
```

例外：`settleInFlight`（正在发币）时只置 `lockRequested=true` 延后到结算结束再清——
发币过程中不能抽走密钥，这是既有正确行为，不改。

### 处置

生命周期锁由「页面级」改为**双绑定**，两条路径调用同一个 `releaseSession()`，不写两份：

| 触发 | 说明 |
| --- | --- |
| `unmount()`（离开视图） | **新增** |
| `pagehide`（关标签 / 刷新 / 关浏览器） | 保留 |

### 验收结果（2026-08-01，已实测）

新增 `test/topup-session-release.test.mjs`。**第一条是真实行为测试**，不是源码断言：
用 `ctx.topupUnlock` 桩走生产的原生托管分支建立真实会话（`handleTopup` 驱动），
拿解锁响应里的真实 `session_id`，`emit('close')` 模拟前端断连，断言：

- `nativeLock()` 恰好被调用 1 次 —— **生产路径下「清空内存密钥」的真实动作**
  （生产 server 始终注入 `topupUnlock`，`nativeManaged` 恒真、`seedBytes` 恒为 null，
  密钥在原生安全进程里，Node 侧从来没有明文）
- 持**同一个真实 session_id** 再调发币入口仍被 403 拒 —— 会话确已作废

配套断言 `clearSession` 同时覆盖 `nativeLock()` 与 `seedBytes.fill(0)` 两条形态，
以及 `unmount` / `pagehide` 共用同一个 `releaseSession`。

**变异验证（证明测试不是永远通过）**：分别注入三种回归——后端 `res.on('close')`
不再触发释放、前端 `unmount` 不再释放、路由离开视图不调 `unmount`——**三种全部被检出**，
源文件逐一恢复并逐字节比对一致。

### 实现过程中补掉的一个自造漏洞

`hashchange` 可由用户手工改写 URL 触发。初版 `renderView` 未判锁屏状态，
未解锁时把 hash 改成 `#topup` 会让充值发币界面直接显示出来（API 侧有会话门 fail-closed，
但界面不该先露出）。已在 `renderView` 开头加锁屏判定：`consoleLock` 未隐藏时
两个视图一律 hidden。该门同样做了变异验证（删掉即被测试检出）。

## 附带收益

- 8 个模块行为一致：「回到首页」永不需要重新验证
- 消灭 `/citizenconsole.html` 的 303 特殊分支
- 消灭对 bfcache 的隐式依赖（该机制在持有活跃连接时本就不成立，正是本次故障成因）

## 生效方式

改完必须 `bash start.sh build-production`（源码构建期封进 app）。
**该命令需钥匙串授权，只能由用户在终端执行。**
