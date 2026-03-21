# GMB Claude 协作规则

本文件定义 Claude 在 GitHub Actions 与代码审查场景中的项目级规则。

## 1. 总原则

- 以中文进行说明、评论和问题描述
- 优先识别 bug、安全风险、行为回归和缺失测试
- 不要猜测关键业务逻辑，不清楚时要明确指出疑问
- 必须遵守仓库中的 `memory/` 文档约束

## 2. 项目记忆入口

在审查或实现任务前，优先阅读以下文档：

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/agent-rules.md`

## 3. 审查重点

- 是否突破了 CPMS / SFID / CitizenChain / WuMinApp 的边界
- 是否修改了关键安全规则但没有更新文档
- 是否缺少必要中文注释
- 是否留下调试残留、TODO、FIXME、`console.log`、`dbg!`、`todo!`
- 是否修改了代码却没有同步更新 `memory/` 或对应技术文档

## 4. CitizenChain 特别规则

- `citizenchain` 是一个完整产品，节点程序和节点 UI 对外仍是一个桌面软件
- 旧版可运行桌面壳在 `citizenchain/nodeuitauri`
- 新版 Flutter Desktop 节点 UI 在 `citizenchain/nodeui`
- `citizenchain/runtime/` 是目标结构，当前仍处于迁移期

## 5. 输出要求

- 先给出问题，再给出总结
- 说明风险等级和影响范围
- 修复建议尽量具体到模块或文件
- 如缺少上下文，要明确写出“基于当前 diff 的判断”
