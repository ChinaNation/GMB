# GMB 统一部署入口

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
- CitizenWeb Cloudflare Pages staging/production 发布入口。
- 文档、中文注释与残留清理。

验收标准：
- 六个脚本通过 Bash 语法检查并被 Git 忽略。
- workflow YAML 可解析，CI 模式不读取发布签名密钥、不创建 Release、不部署服务器。
- release 模式自动生成正式产物；deploy 模式只滚动部署本次成功构建的服务器产物。
- 本轮不自动触发远端 CI、Release 或部署；由用户后续执行对应脚本即视为授权。

当前进度：
- [x] 用户确认文件、按键和自动触发规则。
- [x] 实现六个入口与 workflow 模式隔离。
- [x] 验证脚本、workflow、文档和残留。

执行记录：
- 2026-07-13：六个脚本均通过 `bash -n` 和 ShellCheck，权限为 `700`，根 `.gitignore` 确认全部忽略。
- 2026-07-13：四个 GitHub workflow 通过 YAML 解析与 Actionlint；CitizenApp/CitizenWallet 使用 `mode=ci/release`，CitizenChain 使用 `mode=ci/release/deploy`。
- 2026-07-13：CitizenChain 的 release 与服务器 deploy 已拆开，删除自动删除上一条 CI 记录的旧清理 job，保留审计历史。
- 2026-07-13：所有脚本的非法输入和 dirty worktree 保护已本地验收。本轮按约定未触发远端 CI、Release、Pages/Worker 部署或服务器更新。
