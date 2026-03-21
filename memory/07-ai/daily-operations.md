# GMB AI 系统日常操作手册

## 1. 你的日常使用方式

你只需要做两件事：

1. 在 Codex 主窗口里用中文输入任务需求
2. 在 GitHub PR 里查看 Claude 的审查意见

## 2. 标准操作流程

### 2.1 需求进入

- 你在 Codex 中提出任务需求
- Codex 先做需求分析
- Codex 再读取 `memory/`
- 如果逻辑不清，Codex 先和你确认

### 2.2 开发执行

- Architect 先建任务卡
- Codex 先装载模块上下文
- Codex 改代码
- Codex 补中文注释
- Codex 更新文档
- Codex 清理残留
- Codex 提交 PR

### 2.3 自动检查

GitHub 自动运行：

- `AI Guardrails`
- `Claude PR Review`
- 目录命中的模块级 CI

### 2.4 审查修复

- 你查看 PR 中的 Claude 意见
- 在 Codex 中下达修复指令
- Codex 根据审查意见继续修复并补文档

### 2.5 发布

- 通过 PR 后，进入现有 GitHub 构建或发布流程

## 3. 常用指令建议

### 3.1 给 Codex 的常用中文指令

- `实现这个功能，并同步更新 memory 和技术文档。`
- `先分析影响范围，不清楚的地方先问我。`
- `根据 Claude 的 review 评论修复，并补测试。`
- `清理这次改动留下的无用代码和残留。`

### 3.2 本地 AI 任务入口

- `bash memory/scripts/analyze-requirement.sh --requirement "任务需求"`
- `bash memory/scripts/architect-entry.sh --requirement "任务需求" --execute`
- `bash memory/scripts/start-task.sh --requirement "任务需求"`
- `bash memory/scripts/new-task.sh --module "sfid/backend" --requirement "任务需求"`
- `bash memory/scripts/load-context.sh citizenchain/runtime`
- `bash memory/scripts/complete-task.sh memory/08-tasks/open/<task>.md "完成摘要"`

### 3.3 给 Claude 的常用 PR 评论

```text
@claude 请检查这个 PR 的 bug、回归风险、安全问题、文档遗漏和中文注释是否充分。
```

```text
@claude 请重点检查这个 PR 是否突破了 CPMS、SFID、CitizenChain、WuMinApp 的边界。
```

## 4. 遇到问题时的判断顺序

如果 AI 开发流程没有按预期工作，按这个顺序排查：

1. `memory/` 文档是否完整
2. PR 是否更新了文档
3. GitHub Secret 是否已配置
4. Claude GitHub App 是否已安装
5. Workflow 是否真的被触发

补充判断：

- 如果 `Claude PR Review` 报 `Credit balance is too low`，先检查 API key 是否来自有余额的 workspace
- 如果 `Claude PR Review` 报 `error_max_turns`，说明额度已通，优先检查 workflow 里的 `--max-turns` 是否过小
- 如果 `Benchmark Weights` 报 lockfile 相关错误，先确认 benchmark 脚本是否在用默认构建模式，而不是强制锁定 lockfile
- 如果你怀疑“改了一个目录却跑了另一个目录”，先对照 `memory/07-ai/ci-path-routing.md`

## 5. 当前阶段结论

当前阶段已经形成一套最小可用模式：

- Codex 负责开发
- Claude 负责审查
- GitHub 负责门禁和自动化
- `memory/` 负责长期记忆
