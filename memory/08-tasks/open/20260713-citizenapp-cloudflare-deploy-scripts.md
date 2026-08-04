# CitizenApp Cloudflare 唯一生产安全部署

当前状态：已完成（2026-08-03）。

## 任务需求

- CitizenConsole“公民云 → 生产部署”是聊天、广场、会员及其共用 Cloudflare Worker
  的唯一生产发布入口。
- 用户只点击一次按钮并完成一次 Touch ID；控制台随后自动完成类型生成、静态检查、测试、
  候选版本上传、零流量验收、正式切换、真实健康检查和失败回滚。
- 正式用户数据已经存在，日常控制台彻底删除“清空并重建全部数据”按钮、动作、脚本和文案。
- 普通代码部署不得取得数据管理令牌，不得修改、删除或重建 D1、KV、R2、Queue 数据。

## 所属模块

- `citizenconsole/`
- `citizenapp/cloudflare/`
- `memory/01-architecture/citizenapp/`
- `memory/01-architecture/gmb/`

## 安全硬约束

- 生产部署必须通过一次 Touch ID，禁止 Mac 登录密码回退。
- 本机 Data Protection Keychain 是生产 Secret 真源；Secret 不得写入仓库、普通临时文件、
  日志或命令行参数。
- `CF_DEPLOY_TOKEN` 只用于 Worker 版本上传与部署；`CF_DATA_TOKEN` 不得进入生产部署进程。
- 禁止逐项执行会立即发布中间版本的 `wrangler secret put`；代码、配置和全部 Worker Secret
  必须归入同一个不可变候选版本。
- 候选版本先以 0% 流量加入部署，通过 Version Override 对正式路由做只读健康检查；普通用户
  在验收完成前始终由旧版本 100% 服务。
- 验收通过后候选版本一次切至 100%；切换后健康检查失败必须自动恢复旧版本 100%。
- D1 schema、Durable Object migration、Route 或持久资源拓扑变化不得混入普通生产代码发布；
  检测到这类变化必须失败关闭并要求单独方案。
- 所有子进程必须有界运行；超时或控制台退出必须回收整棵进程树并释放生产任务占用。
- 成功日志必须包含源码哈希、旧/新 Worker version id 和线上健康检查结果，不得包含 Secret
  或用户数据。

## 验收标准

- 控制台和动作脚本中不存在 `reset-formal-data`、“清空并重建全部数据”、D1 DROP、KV/R2
  批量删除和 Queue purge 的可执行入口。
- 一次 Touch ID 后自动运行完整发布链；不出现第二次确认或手工类型生成步骤。
- 类型生成正常退出；异常时在限定时间内明确失败并回收 Wrangler/esbuild 子进程。
- `npm audit --audit-level=low` 必须为零漏洞；任何级别依赖漏洞或审计失败均在读取生产
  Secret、连接 Cloudflare 前失败关闭。
- Worker 自动化测试、CitizenConsole 生产安全测试和 Bash/Node 语法检查全部通过。
- 候选版本在 0% 流量下使用 Version Override 取得真实健康响应后才允许切换 100%。
- 任一发布后检查失败时，旧版本自动恢复为 100%。
- 真实 CitizenConsole 运行态中任务具有明确成功或失败终态，不再无限卡住。

## 执行进度

- [x] 读取仓库实现、运行进程和 Cloudflare 当前官方版本/Secret/回滚规则。
- [x] 确认卡死发生在 `wrangler types` 已输出完成信息但进程未退出，尚未进入远端部署。
- [x] 用户确认彻底删除生产数据清空能力，并按推荐安全架构实施。
- [x] 删除清空重建按钮、服务端动作和脚本实现。
- [x] 完成原子候选版本、0%验收、100%切换与自动回滚。
- [x] 完成超时回收、测试、文档、残留清理和真实运行态验收。

## 执行记录

- 2026-08-03：旧生产部署进程在类型生成阶段卡住约三小时；已按精确 PID 从叶子到根终止，
  未执行 Secret 同步、D1 远端检查或 Worker 发布。
- 2026-08-03：Cloudflare 当前 Wrangler 4.114.0 的 `secret put` 会创建并立即部署版本；现有
  逐项同步 Secret 的流程不满足原子发布要求，必须删除。
- 2026-08-03：已从 CitizenConsole 删除生产数据清空按钮、服务端动作及 D1 DROP、KV/R2
  批量删除、Queue purge 脚本；本机 Keychain 中旧 `R2_ACCESS_ID`、`R2_SECRET_KEY` 已通过
  CitizenConsole 的真实 Touch ID 流程删除，正式 Worker 中对应旧 Secret 也已原子删除。
- 2026-08-03：首次真实发布在接触 Cloudflare 前被依赖审计拦截；已将 Wrangler 更新至
  4.118.0，并锁定修复后的 `undici` 7.29.0，`npm audit --audit-level=low` 为零漏洞。
- 2026-08-03：已修复签名应用中错误推导 `npx` 相邻路径的问题，改为启动时解析并验证
  唯一绝对路径；已增加回归测试。
- 2026-08-03：第一次候选版本零流量验收因 Cloudflare 边缘尚未传播而失败，系统自动回滚至
  旧版本 `9a8b0bad-3363-430b-989e-7e946a7a2cd9`；随后按 Cloudflare 官方传播语义加入
  有界重试，仍严格校验候选版本 id。
- 2026-08-03：真实生产发布完成。源码哈希为
  `aaf370792db26aca480377ab460926bc56bbe4946cb42accc4a762e639e46f90`；新版本
  `9748b7e8-c25f-40e6-a9c6-16baf26be75e` 已占 100% 流量，正式 `/api/health` 返回同一
  Worker version id。发布全过程未取得 `CF_DATA_TOKEN`，未修改 D1、KV、R2 或 Queue 数据。
