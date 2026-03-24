# 任务卡：将目录 wumin copy 重命名为 wumincopy，并同步修正脚本入口与任务卡中的路径引用

- 任务编号：20260323-183617
- 状态：open
- 所属模块：wumin
- 当前负责人：Codex
- 创建时间：2026-03-23 18:36:17

## 任务需求

将目录 wumin copy 重命名为 wumincopy，并同步修正脚本入口与任务卡中的路径引用

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wumincopy/scripts/wumincopy-run.sh
- memory/08-tasks/open/20260323-182300-在-wumin-copy-中做最小改动-只增加粘贴助记词导入钱包功能-不修改包名-存储结构或其他流程.md
- memory/08-tasks/open/20260323-182718-新增脚本-cd-gmb-wuminapp-scripts-wumin-run-sh-用于运行-安装-wumin-冷钱包-参考现有-wuminapp-scripts-app-run-sh.md

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
- 已将目录 `wumin copy/` 物理重命名为 `wumincopy/`
- 已同步修正脚本路径与相关任务卡中的目录引用
- `wumincopy/scripts/wumincopy-run.sh` 保持可执行，入口命令为 `cd ~/GMB && ./wumincopy/scripts/wumincopy-run.sh`

## 验证记录

- `bash -n wumincopy/scripts/wumincopy-run.sh`
  - 结果：通过
