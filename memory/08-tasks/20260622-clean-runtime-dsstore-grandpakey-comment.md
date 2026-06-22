# 清理 runtime .DS_Store 与 grandpakey-change 过期注释

## 状态

完成

## 任务需求

- 删除 `citizenchain/runtime/` 下的 `.DS_Store` 二进制残留。
- 修复 `grandpakey-change` 中引用已删除 wrapper 的过期注释。

## 修改范围

- `citizenchain/runtime/.DS_Store`：删除 macOS 残留文件。
- `citizenchain/runtime/primitives/.DS_Store`：删除 macOS 残留文件。
- `citizenchain/runtime/governance/grandpakey-change/src/lib.rs`：仅修正注释中的当前重试/取消入口名称。
- `citizenchain/runtime/governance/grandpakey-change/src/benchmarks.rs`：仅修正 benchmark 注释中的旧 wrapper 名称残留。
- `citizenchain/runtime/governance/grandpakey-change/src/tests/cases.rs`：仅重命名测试函数，去掉旧 wrapper 名称残留，测试逻辑不变。

## 风险边界

- 本任务只做残留清理和注释修正。
- 不修改 runtime 业务逻辑、storage、call index、权重或接口。
- 不引入兼容旧 wrapper 名称。

## 验收记录

- `find citizenchain/runtime -name .DS_Store -print`：无输出。
- `rg "execute_replace_grandpa_key|cancel_failed_replace_grandpa_key" citizenchain/runtime/governance/grandpakey-change`：无输出。
- `git diff --check -- citizenchain/runtime/governance/grandpakey-change/src/lib.rs citizenchain/runtime/governance/grandpakey-change/src/benchmarks.rs citizenchain/runtime/governance/grandpakey-change/src/tests/cases.rs`：通过，无空白错误。
- `rg -n "[ \t]+$" memory/08-tasks/20260622-clean-runtime-dsstore-grandpakey-comment.md`：无输出。
- 根 `.gitignore` 已包含 `.DS_Store` 与 `**/.DS_Store`，无需新增忽略规则。
