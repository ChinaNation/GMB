# 任务卡：清链重启前的全仓库彻底审计

- 任务编号：20260506-230100
- 状态：open（审计中）
- 负责人：当前主聊天入口（多 Explore subagent 并行扫描,结果汇总）
- 关联前置：[20260506-001500-windows-console-fix-and-chainspec-freeze.md](20260506-001500-windows-console-fix-and-chainspec-freeze.md)
- 关联后续：根据本次审计产出的"待办清单"逐条修完后,执行 `fuwuqi.sh q` 6 台清链重启

## 1. 任务目标

清链重启之前(`fuwuqi.sh q` 重新创世),把仓库内**今天 3 个重构 PR 没收尾干净**的所有残留全部找出来,生成一份「彻底重构清单」,逐条修完再创世,确保新创世的链一开始就处在干净一致的状态。

不留下:
- OLD pallet 名称残留（DuoqianManage / GrandpakeyChange / dq-mgmt 等）
- 已删除概念的引用（清算行 / KEY_ADMIN / finalize_create / sfid_finalized / DuoqianAccounts mirror 等）
- OLD storage 路径硬编码（Institutions / 47B raw subject_id 等）
- 跨产品命名/编码不一致
- 死代码 / 死分支 / 不再触达的 migration / 未使用的常量
- 文档/memory 与代码状态脱节

## 2. 审计范围（6 区并行）

| # | 区域 | 子目录 |
|---|---|---|
| 1 | 链 runtime + 链端 RPC node 后端 | `citizenchain/runtime/` + `citizenchain/node/src/` |
| 2 | 节点 Tauri 前端 | `citizenchain/node/frontend/` |
| 3 | wumin 冷钱包 | `wumin/lib/` + `wumin/test/` |
| 4 | wuminapp 热钱包 | `wuminapp/lib/` + `wuminapp/test/` |
| 5 | 服务端（sfid + cpms + website） | `sfid/` + `cpms/` + `website/` |
| 6 | 文档 / memory | `memory/` |

每个 Explore agent 按统一标记规则输出:

| 标记 | 含义 |
|---|---|
| `[RENAME]` | OLD 名称还在(DuoqianManage / dq-mgmt / Grandpakeychange / 等) |
| `[DEAD]` | 引用已删除概念(清算行 / KEY_ADMIN / finalize_create / 等) |
| `[STORAGE]` | OLD storage 路径 / 47B subject_id / pallet index 错配 |
| `[DOC]` | 注释或文档与现状脱节 |
| `[TEST]` | 测试 fixture / 用例用了 OLD 形态 |
| `[TODO]` | 可执行的 TODO/FIXME |
| `[CONSISTENCY]` | 跨产品/跨 crate 不一致 |
| `[DEAD-CODE]` | 已无调用方的函数/import/常量 |

输出格式：`[标记] file:line — 一句描述` ,每 agent 上限 80 条最关键。

## 3. 汇总

各 agent 产出收回后,我把全量发现汇总到 [memory/08-tasks/open/20260506-230100-pre-regenesis-audit-report.md](20260506-230100-pre-regenesis-audit-report.md),按区+标记分类。user 据此排修复优先级与拆 PR。

## 4. 验收

- 所有 [RENAME] / [DEAD] / [STORAGE] 类发现修完
- 所有 [TEST] 用例同步更新
- memory/ 内 OBSOLETE 项实际删除或归档
- `cargo check -p node`(citizenchain) / `flutter analyze`(wumin & wuminapp) / sfid / cpms 全部 0 警告
- 6 台 `fuwuqi.sh q` 清链重启成功,新链在 3 个 block 内能从主网导出 chainspec 与本地源码 diff 为零
