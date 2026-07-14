# CitizenApp Cloudflare 双环境部署（已迁入本地控制台）

当前状态：
- 原根脚本和明文 Secret 文件已删除；当前唯一入口为根 `deploy/` 本地控制台，其源码可追踪、私密运行数据被精确忽略。
- staging/production Secret 已迁入 macOS Keychain，网页只显示配置状态和中文用途。

任务需求：
- 为 CitizenApp 聊天、广场及其共用 Cloudflare Worker 建立 staging 与 production 分离的一键部署入口。
- 部署脚本负责本地门禁、远端 Secret 同步、D1 状态检查、Worker 部署和真实 HTTP 健康检查。

所属模块：
- `citizenapp/cloudflare`
- 根目录本机私密 `scripts`

输入文档：
- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`

必须遵守：
- Secret 只保存在 macOS Keychain，不得进入仓库配置、日志、浏览器或命令行参数。
- production 与 staging 使用不同 Keychain 条目、Worker、D1、KV、R2 和路由。
- production 发布必须通过 Touch ID，Stripe 密钥必须是 live 模式；staging 必须是 test 模式。
- 不重复执行 `0001_square_core.sql`。该文件是清空后重建基线，不是可重复增量迁移。

输出物：
- 根 `deploy/` 本地部署控制台中的 Cloudflare 动作
- staging/production macOS Keychain 条目
- 架构文档更新、中文注释和残留检查

验收标准：
- 两个脚本通过 Bash 语法检查。
- Keychain 两套各 16 项完整，网页和日志不暴露明文。
- staging 脚本能够在不回显 Secret 的前提下通过本地门禁、远端检查、部署和真实 `/api-staging/health` 验收。
- production 脚本在没有 live Stripe 配置或没有显式确认时拒绝发布。
- 不修改或重建 production D1，不触碰 production 远端直到用户明确执行生产脚本。

当前进度：
- [x] 需求与新增文件已确认。
- [x] 当前 Cloudflare 双环境、远端 Secret 名称和 D1 迁移登记已只读核对。
- [x] 创建并验证 staging/production 私密部署入口。
- [x] 更新文档、完善注释、清理残留。
- [ ] 完成真实 staging 部署验收。

执行记录：
- 2026-07-13：两个脚本 Bash 语法检查通过；生产错误确认会退出，staging 缺少本机 Secret 时会退出。
- 2026-07-14：部署控制台和动作脚本经泄密审计后改为Git追踪；运行记录、编译产物和私密材料由根 `.gitignore` 精确忽略。
- 2026-07-13：staging/production 远端 D1 均可列出，但 Wrangler 把清库重建基线 `0001_square_core.sql` 显示为待迁移；脚本已明确忽略该基线且阻止未知增量迁移，避免重复建表。
- 2026-07-13：Cloudflare 远端已有 Secret 只能列出名称、不能读取原值；本轮已将新值迁入 macOS Keychain并逐项校验。
- 2026-07-13：旧根脚本和明文 Secret 文件已彻底删除，部署入口收敛为根 `deploy/` 本地网页。
