# GMB Agent 规则

## 1. 统一交互规则

- 你只在 Codex 主窗口中使用中文提出需求
- Codex 是唯一主开发入口
- Claude 不作为第二个主聊天窗口，而是后台代码审查角色

## 2. Agent 角色

### Architect Agent

- 负责读取 `memory/`
- 负责任务拆解
- 负责边界控制
- 负责发现需求歧义并及时沟通

### Blockchain Agent

- 负责 `citizenchain` 全域
- 包括 `node/`
- 包括 `nodeuitauri/`
- 包括 `nodeui/`
- 包括 `runtime/`
- 包括区块链相关文档和打包流程

### SFID Agent

- 负责 `sfid` 后端、前端、数据库与文档

### CPMS Agent

- 负责 `cpms` 后端、前端、数据库与文档

### Mobile Agent

- 负责 `wuminapp`
- 负责 Flutter 移动端与 Isar 本地存储

### Review Agent

- 由 Claude 承担
- 负责检查代码、指出问题、给出修复建议

### Release Agent

- 由 GitHub Actions 承担
- 负责自动测试、构建、打包、发布

## 3. 强制规则

- 逻辑不清必须先沟通
- 代码必须补中文注释
- 代码更新后必须更新文档
- 代码更新后必须清理残留
- 不允许擅自突破模块边界
- 不允许跳过契约直接扩展系统规则

## 4. 开发闭环

```text
读文档
→ 读契约
→ 生成计划
→ 写代码
→ 跑测试
→ 更新文档
→ 清理残留
→ 提交审查
→ 修复问题
```
