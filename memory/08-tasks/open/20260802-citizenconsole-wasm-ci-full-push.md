# CitizenConsole WASM CI 整仓推送与闭环运行

## 任务需求

修复 CitizenConsole「CitizenChain → 运行 WASM CI」按钮的安全标识、运行反馈和
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

## 修复前核实的事实

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
- 独立 GitHub 卡片已经删除；同一个 `SSH_KEY` 归入 CitizenApp、CitizenChain、CitizenWallet
  三张产品卡片，禁止复制密钥、回显私钥、新增 API 凭证或转去浏览器登录 GitHub。
- Secret 只经原生安全代理进入目标子进程，不写源码、日志、命令行参数或持久明文文件。

### 第三步：版本判断和整仓推送

- 强制当前分支为 `main`，读取链上、本地 HEAD 和 `origin/main` 的 `spec_version`。
- 链上版本等于仓库版本：同步提高 runtime 真源和现有测试断言一版。
- 仓库版本等于链上版本加一：复用已存在的升级候选，不重复提高。
- 其他差值、创世哈希不一致或远端版本分叉全部停止。
- `git add -A` 提交全部 Git 可见改动，完整提交消息固定为 `CitizenChain WASM`，不得附加
  跳过自动流程的标记；使用临时 SSH 文件推送 `origin/main`，退出即删除临时文件。

### 第四步：准确启动并等待 WASM CI

- `.github/workflows/citizenchain-wasm.yml` 只接受完整消息为 `CitizenChain WASM` 的
  `main` push，不保留手动触发入口；无路径过滤，允许同一代码树用空提交重试。
- CitizenChain、CitizenApp、CitizenWallet 的 push workflow 在该固定消息下于根 job
  分配 runner 前跳过，并使用独立 concurrency 组，避免整仓推送执行无关 CI 或取消同分支
  已经运行的正常 CI。
- 推送后不登录 GitHub、不读取第二凭证；用公开 Actions API 按精确 40 位 `head_sha` 和
  `event=push` 查找任务，等待到 `completed/success` 才结束。限流按响应头自动等待。

### 第五步：文档、测试和残留清理

- 增加按钮安全标识、单次鉴权、版本关系、整仓推送、失败重试和等待 CI 的回归测试。
- 删除局部提交、触发即退出和“只提交 runtime 范围”等旧代码、注释与文档。
- 更新 CI 路由、Runtime 升级和 CitizenConsole 技术文档，记录真实验收证据。

## 预计修改目录

- `/Users/rhett/GMB/.github/workflows/`
  - 修改四条现有 CI 的 push 路由、固定消息门禁和并发隔离；涉及 Git 跟踪配置与旧手动
    WASM 入口清理，不新增 workflow 文件。
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
- [x] 第四步：仅用已有 SSH 私钥启动并等待 WASM CI
- [x] 第五步：文档、测试和残留清理
- [x] CitizenChain WASM 与 GitHub 独立卡片归并清理
- [ ] Runtime 二次确认与真实按钮验收

## 执行记录

- 2026-08-02：用户确认版本判断规则和“等待 WASM CI 最终结束”的完整流程。本阶段只实现
  CitizenConsole 与文档，不运行按钮、不修改 runtime、不推送 GitHub、不触发远端 CI。
- 2026-08-02：本机私有 CitizenConsole 已完成按钮生产标识、可见 run 预检、单次 Touch ID
  正式链与 SSH 配置原子读取、`main` 整仓 SSH 推送和 `spec_version` 同版提高/高一版复用。
  旧的局部提交、触发即退出代码和当前技术文档口径已清理。
- 2026-08-02：Node/浏览器/Bash 语法、ShellCheck、diff whitespace 和 WASM 专项 9 项测试
  通过；版本测试在隔离临时 Git 仓库真实覆盖“同版提高一版、高一版原版本重试、漂移拒绝”。
  同时修正旧 D1 安全测试与“唯一创世基线禁止 schema 版本第二真源”的反向断言，CitizenConsole
  全套 52 项全部通过。当前未运行按钮、未修改 runtime、未提交推送、未触发 CI。
- 2026-08-02：已用 `build-staged` 完成原生安全代理、Node、Socket Launcher 的签名换包，
  `start.sh verify` 完整性验收通过；运行包中 `server.mjs`、`web/app.js` 和三个 Git/WASM
  动作脚本均与源码逐字节一致。换包后无旧 CitizenConsole 进程驻留，下次打开将加载新签名包。
- 2026-08-02：用户否决新增 GitHub API 凭证与浏览器登录方案。相关字段、原生白名单、
  读取/注入、脚本要求、测试和文档口径已删除，只保留原有 `SSH_KEY`。
- 2026-08-02：删除后 CitizenConsole 52 项测试全部通过，原生安全代理与控制台已重新
  编译、签名和原子换包，完整性验收通过；源码与签名运行包已不含新增 GitHub API 凭证口径。
- 2026-08-02：用户确认固定消息 push 方案。本地已改为 `CitizenChain WASM` 精确提交消息
  触发唯一 WASM push workflow，并以公开 API 按精确 SHA 等待；其它三条产品 workflow
  对该消息在 runner 分配前跳过且不干扰已有正常 CI。公开 API 已做无凭证真实只读检查，
  返回字段满足精确 SHA 等待需求；53 项测试、ShellCheck、Actionlint、签名换包与完整性校验
  全部通过。实现阶段未修改 runtime、未提交推送、未触发远端 CI。
- 2026-08-02：按用户确认删除 CitizenChain WASM 与 GitHub 两张独立卡片。WASM CI 和
  「开发升级」归入 CitizenChain；`SSH_KEY` 作为唯一共享 Keychain 项归入 CitizenApp、
  CitizenChain、CitizenWallet，三款产品 CI/Release 均在一次 Touch ID 中读取。旧模块 ID、
  独立脚本、通用推送按钮和旧 UI 文案已清理；GitHub workflow 名称保持不变。
- 2026-08-02：CitizenConsole 54 项测试、Node/Bash/Swift 检查、ShellCheck 与 diff whitespace
  全部通过。已通过控制台自身的 Touch ID「编译」入口重新构建、签名并原子换包，
  `start.sh verify` 通过，签名包内 `server.mjs`、`web/app.js`、`citizenchain.sh` 与源码一致。
  真实页面确认首页只剩 6 张模块卡片；WASM CI 在节点 CI 正下方，开发升级在启动节点正下方；
  三张产品卡片均显示同一个 `SSH_KEY`，未触发任何产品 CI、未提交或推送 GitHub。
- 2026-08-02：用户确认首页六张模块卡片改为固定一行中文小卡片，顺序为「发币、公民、
  公民链、公民云、公民网、公民钱包」。首页卡片只显示名称，不再显示图标、简介、生产状态
  或进入标识；技术模块名、详情页、密钥状态和业务动作保持不变。窄窗口保持单行并允许横向
  滚动，不恢复两列或单列卡片布局。
- 2026-08-02：首页小卡片改造完成。CitizenConsole 55 项测试及 Node/Bash/Swift、ShellCheck
  全部通过；已通过 Touch ID 重新签名换包并通过完整性校验。真实页面确认六张卡片均只有
  一个名称节点，桌面端高度统一为 52px 且纵坐标一致；520px 窄窗口下仍保持同一行，卡片区
  以横向滚动承载。未执行任何产品动作、未修改 runtime、未推送 GitHub。
- 2026-08-02：用户确认充值发币页收敛为顶部订单台账和下方配置两张业务卡片。页头左侧改为
  无箭头的「返回」按钮，标题严格居中；「刷新台账」与「拉取并发币」统一在订单台账按钮行，
  独立“结算 · 拉取并发币”卡片删除。结算状态与逐单结果仍保留在订单台账内，返回锁定、发币
  API、会话生命周期和台账逻辑保持单源不变。
- 2026-08-02：充值发币页布局改造完成。CitizenConsole 56 项测试及 Node/Bash/Swift、
  ShellCheck、diff whitespace 全部通过；已通过控制台自身的 Touch ID「编译」入口重新签名
  换包，`start.sh verify` 与页面源码逐字节一致性检查通过。真实页面确认标题中心误差不足
  0.01px，订单台账位于配置上方，两项台账操作同一行且顺序正确，旧返回文案与旧结算卡片
  均已清理；点击「返回」已回到六模块首页。验收未解锁发币密钥、未拉取订单、未执行发币、
  未修改 runtime、未提交或推送 GitHub。
