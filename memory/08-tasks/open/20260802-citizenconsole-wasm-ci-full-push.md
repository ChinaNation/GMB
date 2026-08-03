# CitizenConsole WASM CI 整仓推送与闭环运行

## 任务需求

修复 CitizenConsole「CitizenChain WASM → 运行 WASM CI」按钮的安全标识、运行反馈和
执行顺序。按钮必须只验证一次 Touch ID，鉴权后读取正式链当前 `spec_version` 与
`genesis_hash`，按链上版本与仓库版本关系决定是否把仓库 `spec_version` 提高一版，随后
提交并推送本机全部 Git 可见代码，最后启动并等待准确提交对应的 WASM CI 完成。

## 用户确认边界

- 保留 Touch ID，按钮必须明确显示需要 Touch ID。
- 同一按钮完成正式链校验、整仓提交、推送、WASM CI 启动和结果等待，不增加第二套入口。
- 链上与仓库版本相同时，仓库版本提高一版后构建新的升级候选。
- 仓库版本已比链上高一版时，不再次提高版本，直接重试该版本的 WASM CI。
- 其他版本差值一律失败关闭，禁止自动覆盖。
- WASM CI 最终成功后流程才成功；CI 失败则保留原版本供下次重试。
- `citizenconsole/` 继续整目录由 Git 忽略，不得上传 GitHub。
- 本任务卡的新建路径、用途和 Git 跟踪已经用户明确确认。
- 涉及 `citizenchain/runtime/` 的真实运行验收必须另行取得 runtime 二次确认。

## 已核实的当前事实

- WASM CI 动作配置为 `production:false`，前端因此错误显示“无需密码”。
- 服务端在创建运行任务之前读取 Keychain 并连接目标链；Touch ID 和 RPC 预检期间没有
  运行日志标签，错误只写入被模块弹窗遮住的系统日志。
- 当前脚本只提交 `citizenchain/runtime/`、`citizenchain/Cargo.toml` 和
  `citizenchain/Cargo.lock`，不符合整仓提交要求。
- 当前 `gh` 没有可用的 GitHub API 登录状态；CitizenConsole 已有 GitHub `SSH_KEY`，
  用户明确禁止因此登录 GitHub 或新增第二个 GitHub 密钥。
- 2026-08-02 检查时，本机 `main` 与 `origin/main` 一致，仓库 `spec_version` 未被按钮改变，
  本次异常点击没有产生提交或启动 WASM CI。

## 技术方案

### 第一步：统一安全动作和可见运行状态

- 将 WASM CI 定义为生产动作，按钮显示“需要 Touch ID”。
- 点击后立即创建运行任务并返回 run id，再在同一运行任务中执行生物识别和预检。
- 所有阶段和失败原因写入当前 WASM 日志标签；前端补齐网络与 JSON 解析异常处理。

### 第二步：一次鉴权读取全部所需配置

- 一次 Touch ID 原子读取 `NODE_WS`、`CHAIN_GENESIS_HASH` 和已有 GitHub `SSH_KEY`。
- GitHub 卡片只保留 `SSH_KEY`，禁止新增 API 凭证或转去浏览器登录 GitHub。
- Secret 只经原生安全代理进入目标子进程，不写源码、日志、命令行参数或持久明文文件。

### 第三步：版本判断和整仓推送

- 强制当前分支为 `main`，读取链上、本地 HEAD 和 `origin/main` 的 `spec_version`。
- 链上版本等于仓库版本：同步提高 runtime 真源和现有测试断言一版。
- 仓库版本等于链上版本加一：复用已存在的升级候选，不重复提高。
- 其他差值、创世哈希不一致或远端版本分叉全部停止。
- `git add -A` 提交全部 Git 可见改动，固定提交信息带 `[skip ci]`，避免整仓推送误触发
  其他 push CI；使用临时 SSH 文件推送 `origin/main`，退出即删除临时文件。

### 第四步：准确启动并等待 WASM CI

- 用户已否决额外 GitHub API 凭证方案，原方案撤销。
- 新方案必须只复用已有 `SSH_KEY`，且仍需保证只启动准确 HEAD 的 WASM CI、支持
  同版本重试并等待最终结果。该方案尚未确认，不得先行实施。

### 第五步：文档、测试和残留清理

- 增加按钮安全标识、单次鉴权、版本关系、整仓推送、失败重试和等待 CI 的回归测试。
- 删除 `ensure_runtime_pushed`、`nowatch` 和“只提交 runtime 范围”等旧代码、注释与文档。
- 更新 CI 路由、Runtime 升级和 CitizenConsole 技术文档，记录真实验收证据。

## 预计修改目录

- `/Users/rhett/GMB/citizenconsole/`
  - 修改本机私有控制台的按钮、服务端编排、原生 Keychain 白名单、动作脚本和测试；涉及代码、
    中文注释和旧流程残留清理，整目录继续由 Git 忽略。
- `/Users/rhett/GMB/citizenchain/runtime/`
  - 真实按钮运行时仅按已确认版本规则修改 `src/lib.rs` 和 `src/tests/cases.rs`；涉及 runtime
    代码，执行前必须单独二次确认，本阶段不触碰。
- `/Users/rhett/GMB/memory/07-ai/`
  - 更新 CI 路由和执行边界；涉及现有文档及“只提交 runtime”旧口径清理。
- `/Users/rhett/GMB/memory/05-modules/citizenchain/`
  - 更新 WASM 构建和 Runtime 升级技术文档；只修改现有文档并清理旧流程描述。
- `/Users/rhett/GMB/memory/08-tasks/open/`
  - 维护本任务卡、执行状态和验收证据；涉及 Git 跟踪文档，不记录任何 Secret 明文。

## 禁止事项

- 未经 runtime 二次确认不得修改或由脚本修改 `citizenchain/runtime/`。
- 未经单独授权不得推送 `origin/main` 或触发 GitHub Actions。
- 不读取、回显或记录任何 Secret 明文。
- 不创建非 `main` 分支，不恢复只提交局部路径或触发即退出的旧流程。
- 不覆盖或提交其他线程尚未完成的改动。

## 执行状态

- [x] 需求分析与版本规则确认
- [x] 任务卡创建
- [x] 第一步：统一安全动作和可见运行状态
- [x] 第二步：一次鉴权读取全部所需配置
- [x] 第三步：版本判断和整仓推送
- [ ] 第四步：仅用已有 SSH 私钥启动并等待 WASM CI
- [x] 第五步：文档、测试和残留清理
- [ ] Runtime 二次确认与真实按钮验收

## 执行记录

- 2026-08-02：用户确认版本判断规则和“等待 WASM CI 最终结束”的完整流程。本阶段只实现
  CitizenConsole 与文档，不运行按钮、不修改 runtime、不推送 GitHub、不触发远端 CI。
- 2026-08-02：本机私有 CitizenConsole 已完成按钮生产标识、可见 run 预检、单次 Touch ID
  正式链与 SSH 配置原子读取、`main` 整仓 SSH 推送和 `spec_version` 同版提高/高一版复用。
  旧的 `ensure_runtime_pushed`、`nowatch`、局部提交代码和当前技术文档口径已清理。
- 2026-08-02：Node/浏览器/Bash 语法、ShellCheck、diff whitespace 和 WASM 专项 9 项测试
  通过；版本测试在隔离临时 Git 仓库真实覆盖“同版提高一版、高一版原版本重试、漂移拒绝”。
  同时修正旧 D1 安全测试与“唯一创世基线禁止 schema 版本第二真源”的反向断言，CitizenConsole
  全套 52 项全部通过。当前未运行按钮、未修改 runtime、未提交推送、未触发 CI。
- 2026-08-02：已用 `build-staged` 完成原生安全代理、Node、Socket Launcher 的签名换包，
  `start.sh verify` 完整性验收通过；运行包中 `server.mjs`、`web/app.js` 和三个 Git/WASM
  动作脚本均与源码逐字节一致。换包后无旧 CitizenConsole 进程驻留，下次打开将加载新签名包。
- 2026-08-02：用户否决新增 GitHub API 凭证与浏览器登录方案。相关字段、原生白名单、
  读取/注入、脚本要求、测试和文档口径已删除；GitHub 卡片只保留原有 `SSH_KEY`。
- 2026-08-02：删除后 CitizenConsole 52 项测试全部通过，原生安全代理与控制台已重新
  编译、签名和原子换包，完整性验收通过；源码与签名运行包已不含新增 GitHub API 凭证口径。
