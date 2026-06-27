# GMB AI 开发流程

## 1. 标准流程

```text
你提出任务需求
→ Codex 先做需求分析
→ Codex 读取 memory/
→ Codex 读取对应模块文档
→ Codex 判断边界与影响范围
→ 如逻辑不清先沟通
→ 分析确认后先创建任务卡
→ 技术方案必须包含预计修改目录(逐项中文注释)、更新文档、完善注释、清理残留
→ 再进入实现
→ Codex 改代码并补中文注释
→ Codex 更新 memory/ 或技术文档
→ Codex 完善注释
→ Codex 清理残留
→ GitHub PR 触发门禁
→ Claude 自动审查
→ Codex 修复问题
→ GitHub Actions 测试、构建、发布
```

## 2. 强制门禁

- 改代码后必须更新文档
- 真实开发任务必须创建任务卡
- 改代码后必须清理残留
- 逻辑不清时必须先沟通
- 关键逻辑必须补中文注释
- 禁止兼容硬规则：除非用户在当前任务中明确要求兼容，否则不得保留旧流程、旧格式、旧数据、旧命名、旧目录、旧注释、旧文案、旧交易载荷、旧接口分支、过渡兼容、双轨兼容或影子旧流程
- 彻底改造硬规则：发现旧代码、旧注释、旧文档、旧配置、旧数据、旧目录、旧测试、旧任务描述或旧 UI 文案残留时，必须在同一任务内清理
- 真实验收硬规则：涉及 API、数据库、登录、权限、扫码或页面展示的任务，必须使用真实本地服务、真实数据库、真实 HTTP 接口或真实页面验收；编译、类型检查和 build 只能作为基础门禁
- 每次输出技术方案都必须包含更新文档、完善注释、清理残留
- 每次输出技术方案都必须包含预计修改目录清单；每个目录必须附中文注释，说明修改用途、边界和是否涉及代码、文档或残留清理
- 每次执行技术方案后都必须更新文档、完善注释、清理残留
- `memory/08-tasks/` 下的任务卡文件名（含 `.md` 扩展名）不得超过 160 个 UTF-8 字节
- 相同功能必须在前后端创建相同文件夹；功能不大时直接在对应文件夹下创建相同文件，功能过大时再按需下钻一级同名子文件夹；不确定边界时必须先询问用户
- 用户对外只提供任务需求，不强制手工拆标题和目标
- 只读报错诊断例外：输入包含 `检查为什么报错` 时，直接检查并输出报错原因，不创建任务卡、不修改代码；后续要求修复时再创建任务卡

## 3. GitHub 自动化入口

- `.github/workflows/ai-guardrails.yml`
- `.github/workflows/claude-pr-review.yml`
- `.github/workflows/claude-on-comment.yml`
- `.github/workflows/citizenchain-wasm.yml`
- `.github/workflows/citizenchain.yml`
- `.github/workflows/citizencode-ci.yml`
- `.github/workflows/citizenpassport-ci.yml`
- `.github/workflows/citizenapp-ci.yml`

## 4. 路径分流执行原则

- `citizenchain/runtime/**` 通过独立 WASM CI 产出链上升级 wasm，`citizenchain/node/**` 通过统一 `citizenchain.yml` matrix 构建 Linux / Windows / macOS 桌面安装包
- 共享 Rust 目录变更时，允许多侧联动执行
- `cid`、`citizenapp` 分别独立执行
- 纯文档、Pages 等目录按各自规则触发
- 目录路由细则统一记录在 `memory/07-ai/ci-path-routing.md`

## 5. 本地执行入口

- `bash scripts/analyze-requirement.sh --requirement "..."`
- `bash scripts/check-startup-acceptance.sh`
- `bash scripts/architect-entry.sh --requirement "..." --execute`
- `bash scripts/start-task.sh --requirement "..."`
- `bash scripts/new-task.sh --module "<模块>" --requirement "..."`
- `bash scripts/load-context.sh <模块>`
- `bash scripts/complete-task.sh memory/08-tasks/open/<任务卡>.md "完成摘要"`

## 6. 当前执行方式

- 你只使用 Codex 主窗口
- Claude 在 GitHub PR 与评论中承担审查与辅助角色
- 项目长期记忆统一保存在 `memory/`
