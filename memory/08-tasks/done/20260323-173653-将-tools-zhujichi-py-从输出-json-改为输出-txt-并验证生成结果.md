# 任务卡：将 tools/zhujichi.py 从输出 json 改为输出 txt，并验证生成结果

- 任务编号：20260323-173653
- 状态：done
- 所属模块：ai-system
- 当前负责人：Codex
- 创建时间：2026-03-23 17:36:53

## 任务需求

将 tools/zhujichi.py 从输出 json 改为输出 txt，并验证生成结果

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建

## 完成信息

- 完成时间：2026-03-23 17:40:19
- 完成摘要：已将 zhujichi.py 默认输出改为 vault_without_salt.txt，新增纯文本格式化输出与更明确的 subkey 错误提示，并更新仓库 tools 文档说明；已通过 py_compile 和临时 txt 生成验证。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
