# 20260727 node admin_unlock 测试隔离竞态修复

任务需求：
修复 `citizenchain/node` 单测隔离竞态：`admin_unlock.rs` 内 `list_decrypted_admins_filters_by_cid`
与 `lock_decrypted_admin_removes_entry` 两个用例共享全局 `static DECRYPTED_ADMINS`
(`Mutex<HashMap>`)，并行 `cargo test --workspace` 偶发失败。目标：并行跑稳定通过，零行为回归。

所属模块：citizenchain/node（Blockchain Agent；仅测试代码，不碰 runtime、不碰生产逻辑）

根因（已读码复核）：
- `decrypted_map()` / `DECRYPTED_ADMINS` 全仓仅本文件引用；直接 insert 该表的测试仅这两个用例。
- 测试 A `list_decrypted_admins_filters_by_cid` 结尾 `decrypted_map().lock().unwrap().clear()`
  清的是**整张表**（admin_unlock.rs:390）。
- 测试 B `lock_decrypted_admin_removes_entry` 在 insert `cc…`（:395）与
  `assert!(lock_decrypted_admin(cc…).is_ok())`（:402）之间，若被 A 的 clear() 插入，
  `cc…` 被抹掉 → lock 返回「该公钥未在解密状态」→ 断言失败。
- 两用例 key（aa/bb/cc）与 cid 互不相同；去掉全表 clear() 后并行无交叉。
- `--test-threads=1` 6/6 通过，印证是并行竞态而非逻辑 bug。

采用方案（方案①，最小改动、零新依赖、长期免疫）：
- 去掉测试 A 结尾的全表 `clear()`，改为只 `remove` 本用例插入的 `aa…`/`bb…` 两个 key。
- 测试 B 无需改（`lock_decrypted_admin` 已自动移除 `cc…`）。
- 未选方案②（serial_test 需新增第三方依赖）与方案③（仍保留全表清空语义、将来复发）。

输入文档：
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/requirement-analysis-template.md

必须遵守：
- 不可突破模块边界（仅 citizenchain/node 测试代码）
- 不碰 citizenchain/runtime，不改生产逻辑
- 不引入第三方依赖
- 一切改动落在主检出 /Users/rhett/GMB，不碰 worktree

预计修改目录：
- citizenchain/node/src/transaction/offchain/settlement/：改 admin_unlock.rs 测试清理逻辑
  （clear() → 只删自身 key）。仅测试代码，不涉文档、无残留。

输出物：
- 代码：admin_unlock.rs 测试块清理逻辑修改（含中文注释）
- 测试：复用现有 6 个用例
- 文档更新：本任务卡 + memory 记忆
- 残留清理：无遗留全表 clear()

验收标准：
- 主检出默认并行 `cargo test -p node admin_unlock` 稳定通过
- 多轮并行不再偶发失败
- 无 runtime diff、无依赖变更
- 残留已清理
