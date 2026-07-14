# GMB 统一部署入口（已由本地控制台替代）

当前状态：
- 本卡原有六个根 `scripts/` 交互入口已被 `20260713-deploy-dashboard` 完整替代，不再作为当前部署方式。
- 当前唯一入口是根 `deploy/` 本地网页；不含密钥的源码由Git追踪，密钥已迁入 macOS Keychain，旧明文文件和旧脚本已删除。

任务需求：
- 使用六个根目录私密脚本统一触发 Cloudflare、CitizenWeb、CitizenApp、CitizenWallet、CitizenChain 和 Runtime WASM 的 CI、测试部署、正式发布与服务器部署。
- 用户选择 `c`、`s` 或 `b` 后立即自动执行，不再进入 GitHub 页面手动触发，也不做第二次确认。

所属模块：
- 仓库级发布流程
- CitizenApp / CitizenWallet / CitizenChain / CitizenWeb

必须遵守：
- `scripts/` 不进入 Git；部署密钥只保存在本机权限 `600` 的 Secret 文件或 GitHub Secrets。
- `c` 只执行 CI 或测试部署；`s` 才允许 production / release；`b` 只允许滚动部署本次成功构建的 Linux amd64 产物。
- 脚本触发 GitHub workflow 后必须等待完成并以 workflow 结果作为退出码。
- Runtime WASM 只构建和校验，不自动提交链上 `setCode`。

输出物：
- 六个私密部署入口。
- GitHub workflow 的 `ci/release/deploy` 显式输入与执行隔离。
- CitizenWeb 本地测试网站启动/关闭与现有 production Pages 发布入口。
- 文档、中文注释与残留清理。

验收标准：
- 六个脚本通过 Bash 语法检查并被 Git 忽略。
- workflow YAML 可解析，CI 模式不读取发布签名密钥、不创建 Release、不部署服务器。
- release 模式自动生成正式产物；deploy 模式只滚动部署本次成功构建的服务器产物。
- 本轮不自动触发远端 CI、Release 或部署；由用户后续执行对应脚本即视为授权。

当前进度：
- [x] 用户确认文件、按键和自动触发规则。
- [x] 原六个入口已完成历史目标，随后由本地部署控制台彻底替代。
- [x] 验证脚本、workflow、文档和残留。

执行记录：
- 2026-07-13：六个脚本均通过 `bash -n` 和 ShellCheck，权限为 `700`，根 `.gitignore` 确认全部忽略。
- 2026-07-13：四个 GitHub workflow 通过 YAML 解析与 Actionlint；CitizenApp/CitizenWallet 使用 `mode=ci/release`，CitizenChain 使用 `mode=ci/release/deploy`。
- 2026-07-13：CitizenChain 的 release 与服务器 deploy 已拆开，删除自动删除上一条 CI 记录的旧清理 job，保留审计历史。
- 2026-07-13：所有脚本的非法输入和 dirty worktree 保护已本地验收。本轮按约定未触发远端 CI、Release、Pages/Worker 部署或服务器更新。
- 2026-07-13：为六个新部署入口补齐中文用途、模式、密钥边界、提交保护、远端任务定位和真实健康检查注释。
- 2026-07-13：完成根 `scripts/` 用途审计，删除 8 个无继续用途或存在安全风险的私密工具：旧节点直部署脚本及旧公网 RPC service、三份硬编码奖励绑定脚本、明文助记词生成脚本、一次性管理员回填脚本、已失效的白皮书图片提取脚本。
- 2026-07-14：可视化改造已删除六个旧部署入口及两份明文 Secret；根 `scripts/` 只保留非部署仓库工具，所有部署能力收口到根 `deploy/`。控制台源码经泄密审计后改为Git追踪，私密运行数据仍被精确忽略。
- 2026-07-13：部署密钥盘点确认 GitHub Actions 所需 `GMB_APP_KEY`、`GMB_SSH_KEY`、`GMB_TOP_KEY`、`GMB_TOP_PUBKEY` 均已配置；Cloudflare staging/production 各 16 个远端 Worker Secret 名称齐全，但平台不允许读取明文回填本机。
- 2026-07-13：从 Wrangler 当前登录账户确认并写入两份本机 Secret 的 `CF_ACCOUNT_ID`；其余值没有可读取的本机真源，禁止从远端状态或历史公开内容猜测回填，等待用户提供或轮换。
- 2026-07-13：完成 Cloudflare Access、R2、Turnstile、Stream、Worker 运行时签名密钥、Stripe Webhook、Firebase Admin SDK 与 Stripe API 密钥轮换；新值已迁入 macOS Keychain 并同步对应 staging/production Worker，不在任务卡记录密钥明文。
- 2026-07-13：删除两把旧 Firebase 服务账号密钥，仅保留本轮新建的 Firebase Admin SDK 密钥；废止 Stripe 旧测试主密钥、旧生产主密钥以及旧的测试/生产 CLI 辅助密钥。
- 2026-07-13：新 Stripe staging/production 密钥分别通过 Stripe 真实 API 验证，均返回目标账户 HTTP 200；两份本机 Secret 均为 16/16 非空，Cloudflare staging/production 远端 Secret 名称均为准确的 16 项。
