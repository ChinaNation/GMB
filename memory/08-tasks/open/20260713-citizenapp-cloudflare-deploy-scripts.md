# CitizenApp Cloudflare 双环境私密部署脚本

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
- Secret 只保存在被 Git 忽略的根目录 `scripts/`，权限固定为 `600`，不得进入模块脚本、仓库配置、日志或命令行参数。
- production 与 staging 使用不同 Secret 文件、Worker、D1、KV、R2 和路由。
- production 发布必须显式输入 `PRODUCTION`，Stripe 密钥必须是 live 模式；staging 必须是 test 模式。
- 不重复执行 `0001_square_core.sql`。该文件是清空后重建基线，不是可重复增量迁移。

输出物：
- `scripts/cloudflare.sh`（已由仓库级统一部署入口替代两个分散脚本）
- 两个本机 Secret 文件
- 架构文档更新、中文注释和残留检查

验收标准：
- 两个脚本通过 Bash 语法检查。
- Secret 文件权限为 `600` 且被 Git 忽略。
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
- 2026-07-13：脚本权限为 `700`，两个 Secret 文件权限为 `600`，四个文件均由根 `.gitignore` 的 `/scripts/` 规则忽略。
- 2026-07-13：staging/production 远端 D1 均可列出，但 Wrangler 把清库重建基线 `0001_square_core.sql` 显示为待迁移；脚本已明确忽略该基线且阻止未知增量迁移，避免重复建表。
- 2026-07-13：Cloudflare 远端已有 Secret 只能列出名称、不能读取原值；首次真实运行仍需负责人向两个本机 Secret 文件安全录入原始值。录入完成前不能执行完整 staging 发布验收。
- 2026-07-13：仓库级部署入口任务确认后，两个分散脚本已删除并收敛为 `scripts/cloudflare.sh`；输入 `c/s` 直接选择 staging/production。
