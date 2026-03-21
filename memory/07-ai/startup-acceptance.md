# GMB AI 启动协议验收标准

## 1. 目标

本标准用于确认：

- 新开的 Codex 线程是否已经接入 GMB AI 编程系统
- 根目录入口是否仍然正确指向 `memory/`
- 新线程是否会先进入需求分析，而不是直接写代码
- Codex 是否仍承担主入口与总调度器角色

## 2. 仓库级验收条件

以下条件必须全部成立：

- 根目录 `AGENTS.md`、`CODEX.md`、`CLAUDE.md` 必须存在
- 根目录这 3 个入口必须指向 `memory/` 下对应文件
- `memory/AGENTS.md` 必须保留“第一轮必须先做需求分析”
- `memory/AGENTS.md` 必须保留“Codex 主窗口按需自动调度专业工作线程”
- `memory/CODEX.md` 必须保留“进入真实开发前必须创建任务卡”
- `memory/07-ai/chat-protocol.md` 必须保留“以 `需求分析` 开头”

## 3. 手工验收步骤

1. 在 `GMB` 工作区中新开一个 Codex 线程
2. 直接输入真实任务需求
3. 观察第一轮正式回复

## 4. 通过标准

第一轮正式回复必须同时满足：

- 以 `需求分析` 开头
- 先概括任务需求
- 输出建议模块
- 输出影响范围
- 输出主要风险点
- 输出是否需要先沟通
- 输出建议下一步
- 不直接进入实现
- 不要求用户手工指定应该交给哪个 Agent

## 5. 继续执行标准

在继续执行真实开发前，必须满足：

- 已完成需求分析
- 已得到继续信号或边界已明确
- 已创建 `memory/08-tasks/` 下的任务卡

## 6. 自动检查入口

- 仓库检查：`bash memory/scripts/check-startup-acceptance.sh --ci`
- 本地人工检查：`bash memory/scripts/check-startup-acceptance.sh`
