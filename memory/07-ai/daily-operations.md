# GMB AI 系统日常操作手册

## 1. 你的日常使用方式

你只需要做两件事：

1. 在 Codex 主窗口里用中文下达需求
2. 在 GitHub PR 里查看 Claude 的审查意见

## 2. 标准操作流程

### 2.1 需求进入

- 你在 Codex 中提出中文需求
- Codex 先读取 `memory/`
- 如果逻辑不清，Codex 先和你确认

### 2.2 开发执行

- Codex 改代码
- Codex 补中文注释
- Codex 更新文档
- Codex 清理残留
- Codex 提交 PR

### 2.3 自动检查

GitHub 自动运行：

- `AI Guardrails`
- `Claude PR Review`

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

### 3.2 给 Claude 的常用 PR 评论

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

## 5. 当前阶段结论

当前阶段已经形成一套最小可用模式：

- Codex 负责开发
- Claude 负责审查
- GitHub 负责门禁和自动化
- `memory/` 负责长期记忆
