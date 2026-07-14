# CitizenApp Cloudflare 双环境部署（已迁入本地控制台）

当前状态：
- 原根脚本和明文 Secret 文件已删除；当前唯一入口为根 `deploy/` 本地控制台，其源码可追踪、私密运行数据被精确忽略。
- staging/production Secret 已迁入 macOS Keychain，网页只显示配置状态和中文用途。

任务需求：
- 为 CitizenApp 聊天、广场及其共用 Cloudflare Worker 建立 staging 与 production 分离的一键部署入口。
- 部署脚本负责本地门禁、远端 Secret 同步、D1 状态检查、Worker 部署和真实 HTTP 健康检查。
- 部署控制台同时提供独立的会员真实测试动作；该动作不部署 Worker，而是经 Access 用户批准后对现有 staging 运行真实会员矩阵并清理测试数据。

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
- 会员真实测试只允许 `sk_test_`，使用每轮随机临时钱包；不得输出 seed、Access JWT 或 Stripe 密钥，结束时必须验证 staging D1 零残留并清理 Stripe 测试资源。

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
- [x] 部署控制台新增 Access 交互登录及会员真实测试入口。
- [x] 修复 Cloudflare 525 后再次完成当前可执行会员矩阵的真实运行态验收；最终 PASS=22、BLOCKED=0、FAIL=0，卡取消、升档差价建单、USDC Checkout、卡订阅 Checkout、Stripe 测试资源清理与 staging D1 零残留均有真实响应验证。
- [x] 通过部署控制台的 fetch、原生 TCP+TLS、兼容标志和 Stripe test A/B 找到并修复 525 精确根因：Zone 的 `origin_tls_compliance_modes=["pqh"]` 强制后量子混合密钥交换，使不支持该约束的 GitHub/Stripe fetch 握手失败；现已清空为 `[]`。
- [ ] 完成真实 staging 部署验收。

执行记录：
- 2026-07-13：两个脚本 Bash 语法检查通过；生产错误确认会退出，staging 缺少本机 Secret 时会退出。
- 2026-07-14：部署控制台和动作脚本经泄密审计后改为Git追踪；运行记录、编译产物和私密材料由根 `.gitignore` 精确忽略。
- 2026-07-13：staging/production 远端 D1 均可列出，但 Wrangler 把清库重建基线 `0001_square_core.sql` 显示为待迁移；脚本已明确忽略该基线且阻止未知增量迁移，避免重复建表。
- 2026-07-13：Cloudflare 远端已有 Secret 只能列出名称、不能读取原值；本轮已将新值迁入 macOS Keychain并逐项校验。
- 2026-07-13：旧根脚本和明文 Secret 文件已彻底删除，部署入口收敛为根 `deploy/` 本地网页。
- 2026-07-14：真实测试入口使用 `cloudflared access login` 的具体受保护路径 `/api-staging/health` 发现路径型 Access 应用；无有效会话时打开 `Approve` 窗口，批准后 JWT 仅保留在子进程环境并随 API 请求发送，不进入网页或日志。
- 2026-07-14：部署控制台真实运行发现并补齐三项工具缺口：launchd 子进程继承 Node PATH；远程 D1 测试复用具备 D1 write 权限的本机 Wrangler OAuth，不再误用部署低权限令牌；Stripe 卡取消夹具使用官方 `tok_visa` 创建本轮隔离测试卡。最终随机钱包真实签名、API、远端 D1 与 Stripe test 全矩阵完成，业务逻辑无 FAIL，四个 Stripe Worker 出口请求稳定复现 525。
- 2026-07-14：用户要求先定位 525 再执行 Cloudflare 配置整改；部署控制台开始增加四组无副作用 Stripe 出口隔离探针，有效测试密钥请求故意缺少必填参数，只允许得到 Stripe 错误响应，不创建支付资源。
- 2026-07-14：同一临时 Worker 源码与 Stripe test 密钥在 `www.crcfrcn.com/queue-test/*` 和 `chain.crcfrcn.com/queue-test/*` 均稳定复现外部 GitHub/Stripe 525、Cloudflare trace 200；绕开 Zone Route 的 `workers.dev` 能取得 Stripe 401/400 与 Stripe Request-ID，且 `created_resources=0`。Full (strict) 前后结果不变，因此 Flexible 是已整改的安全配置错误，但不是本次 525 的直接触发原因。
- 2026-07-14：Cloudflare Zone 已改为 Full (strict)，Minimum TLS 已提高到 TLS 1.2，Always Use HTTPS 已启用；真实外部验收为 HTTP 301、HTTPS 200、TLS 1.0 握手拒绝、TLS 1.2 成功、staging Access 302、chain 404，无新的 525。
- 2026-07-14：只读核对 Origin Server 配置，当前 Zone 为 Free 计划且未启用 Advanced Certificate Manager，页面没有 Custom Origin Trust Store；Worker 配置使用 `compatibility_date=2026-07-08` 且未显式设置 COTS/global fetch 兼容标志。根据 Cloudflare 的兼容日期默认值，外部子请求已采用 `no_cots_on_external_fetch`，因此自定义信任库不是本次外部 GitHub/Stripe 525 的原因。
- 2026-07-14：已永久删除 staging 临时 IP bypass 策略、`www`/`chain` 两条 `queue-test` 临时路由、`gmb-queue-egress-test` Worker、`gmb-egress-test-q` 队列和临时 Stripe Secret；本地一次性探针文件及部署控制台诊断动作亦已删除。
- 2026-07-14：仍未验证付款完成到 webhook 授权益整链、voting/candidate 成功身份、卡与 USDC 完整切换、身份不匹配冻结；这些项目必须在具备相应真实状态后另行验收，不得并入本轮 PASS=22。
- 2026-07-14：新增原生 TCP+TLS 对照后确认 GitHub/Stripe 443 握手与真实 HTTP 响应均成功，只有 Zone Route 下的 `fetch()` 返回 525；example.com/Google fetch 同时正常。读取真实 Zone 设置发现 `origin_tls_compliance_modes=["pqh"]`（修改于 2026-07-13），它强制源站仅使用后量子混合密钥交换，完整解释了支持目标成功、不支持目标 525、原生 socket 成功、workers.dev 成功的全部现象。
- 2026-07-14：通过仅限 `crcfrcn.com` Zone Settings Edit 的临时用户令牌将 `origin_tls_compliance_modes` 从 `["pqh"]` 清空为 `[]`，写入和回读均成功。修复后同一 Zone Route 真实复测：GitHub 403（缺 User-Agent，已到达 GitHub）、Stripe 首页 200、无认证/假密钥 401、有效 test 密钥 400 `parameter_missing` 并取得 Stripe Request-ID，全程零 525、零创建资源。
- 2026-07-14：修复后部署控制台会员真实测试最终 PASS=22、BLOCKED=0、FAIL=0；B3 卡取消与 Stripe/D1 `cancel_at_period_end` 落库、D2 升档差价建单、E1 USDC Checkout、F1 卡订阅 Checkout 全部 HTTP 200，随后验证清理 5 个 Stripe test 资源和 staging D1 零残留。
- 2026-07-14：用于读取和修改该新设置的三个临时 Cloudflare 用户 API 令牌已从个人资料中永久删除；根因探针 Worker、Route、Secret、本地探针文件和部署控制台临时诊断动作再次清理完毕。
