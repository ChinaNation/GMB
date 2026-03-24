# 任务卡：新增脚本入口用于运行/安装 wumincopy 冷钱包

- 任务编号：20260323-182718
- 状态：open
- 所属模块：wumin
- 当前负责人：Codex
- 创建时间：2026-03-23 18:27:18

## 任务需求

新增脚本入口用于运行/安装 `wumincopy` 冷钱包，并将脚本放到 `wumincopy/scripts/wumincopy-run.sh`

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/scripts/app-run.sh
- wumincopy/scripts/app-clean-run.sh
- wumincopy/scripts/wumincopy-run.sh

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
- 已删除旧入口 `wuminapp/scripts/wumin-run.sh`
- 已新增新入口 `wumincopy/scripts/wumincopy-run.sh`
- 新脚本位于 `wumincopy/scripts/` 下，内部切换到 `wumincopy/` 项目根目录执行 `flutter run`
- 实际运行命令为 `cd ~/GMB && ./wumincopy/scripts/wumincopy-run.sh`
- 已为脚本补充中文注释并设置可执行权限

## 验证记录

- `bash -n wumincopy/scripts/wumincopy-run.sh`
  - 结果：通过
