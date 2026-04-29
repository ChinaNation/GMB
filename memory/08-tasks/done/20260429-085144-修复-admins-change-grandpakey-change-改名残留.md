# 任务卡：修复 admins-change 与 grandpakey-change 改名残留

- 任务编号：20260429-085144
- 状态：done
- 所属模块：citizenchain/governance
- 当前负责人：Codex
- 创建时间：2026-04-29 08:51:44

## 任务需求

修复上一轮全仓库复查发现的两个模块改名残留，仅处理 `admins-change` 与 `grandpakey-change` 相关的旧命名、旧 storage 口径和任务索引残留，不处理其他模块的并行变更。

## 改动范围

- `citizenchain/runtime/transaction/duoqian-manage/src/lib.rs`
- `memory/08-tasks/done/20260408-sfid-三角色命名统一-任务卡0.5.md`
- `memory/08-tasks/done/20260328-112311-审查-admins-change-模块安全与质量.md`
- `memory/08-tasks/open/20260429-runtime-step2-admins-change-unified-subjects.md`
- `memory/08-tasks/index.md`

## 实施记录

- 任务卡已创建。
- 已修复 `duoqian-manage` 测试提示文案中的旧命名。
- 已修复历史任务文档中的旧管理员 storage 口径。
- 已将 step2 任务卡文件名和任务索引统一到 `admins-change`。
- 已复扫旧模块名、旧 crate 标识和旧管理员 storage 口径，当前活跃仓库范围未再命中。
- 已执行验证：`cargo test -p duoqian-manage --lib`，21 个测试全部通过。

## 完成信息

- 完成时间：2026-04-29
- 完成摘要：修复上一轮复查发现的两个治理模块改名残留，未处理其他模块并行变更。
