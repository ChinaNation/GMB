# GMB AI 编程系统操作手册

## 1. 日常操作顺序

### 1.1 提需求

你只需要在 Codex 主窗口里用中文说清楚：

- 任务需求
- 明确限制

### 1.2 先分析

推荐入口：

```bash
bash memory/scripts/analyze-requirement.sh --requirement "任务需求"
```

### 1.3 分析确认后继续执行

```bash
bash memory/scripts/architect-entry.sh --requirement "任务需求" --execute
```

说明：

- 进入真实开发前，必须先创建任务卡
- 任务卡创建后，才能开始改代码、补文档和清理残留

### 1.4 提交前收口

```bash
cat memory/07-ai/pre-submit-checklist.md
```

### 1.5 新线程验收

```bash
bash memory/scripts/check-startup-acceptance.sh
```

## 2. 一句话用法

先分析任务需求，再建任务卡，再写代码，再按清单收口。
