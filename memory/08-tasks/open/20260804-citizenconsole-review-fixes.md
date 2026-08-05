# 公民控制台全面复查整改：GitHub 令牌断链、编译互斥、换包原子性、按需读取

任务需求：修复 2026-08-04 对 `citizenconsole/` 全面复查发现的 2 个 HIGH、3 个 MEDIUM、
2 个 LOW 问题，并同步文档、注释、测试与残留清理。

所属模块：citizenconsole（不在 Git 版本库内）、memory/01-architecture

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 问题清单与定性

| 编号 | 等级 | 问题 | 根因 |
|---|---|---|---|
| HIGH-1 | 高 | `gh secret list` 未认证，失败被压成空集，GitHub 密钥在面板上恒显「未配置」 | `gh` 认证从登录态改成 `GH_TOKEN` 注入时，`server.mjs` 三处 `gh secret` 直调没跟上 |
| HIGH-2 | 高 | `GMB_APP_KEY` 写入半成功：Keychain 已改、GitHub 投影未改 | 同 HIGH-1 |
| MEDIUM-1 | 中 | 「编译」换包可与其它动作并发，旧包被删导致混版执行 | 取消全局串行后未给编译补互斥 |
| MEDIUM-2 | 中 | 换包两次 `mv` 之间被 SIGTERM 打断 → `.runtime` 既无 app 也无 launcher，控制台起不来 | 换包段落未屏蔽中断 |
| MEDIUM-3 | 中 | 进「公民云」页立刻弹 Touch ID | `renderCloudflareView()` 无条件读对账开关 |
| LOW-1 | 低 | 死 CSS：`.pill.go`、`.tp-form` 系列 | 历史残留 |
| LOW-2 | 低 | `GMB_TECHNICAL.md` 写「Developer ID + 公证」，与 `start.sh` 实现相反 | 文档未随实现更新 |

## 定稿口径

1. **失败与「不存在」必须可区分**：远程查询失败返回 `null`，前端渲染「未知」，
   绝不压成 `false`（即「未配置」）。这是仓库禁止的静默失败类型。
2. **读令牌即授权**：`gh secret` 写/删路径改为读 `github:GH_TOKEN`（本身强制 Touch ID），
   不再额外调 `authorizeProduction()`。沿用 `server.mjs:889` 既有约定
   ——「动作已读密钥就不再多弹一次指纹」。
3. **编译是唯一的全局串行点**：它替换的是自己脚下的应用包，必须与全体动作互斥。
4. **换包必须原子**：要么没换、要么换完，中间不接受中断。
5. **状态轮询路径永不注入令牌**：`/api/status` 每几秒一次，注入令牌等于每次轮询弹指纹。

## 执行结果

全部 7 项已修复，测试已补，文档与注释已同步，死 CSS 已删。
