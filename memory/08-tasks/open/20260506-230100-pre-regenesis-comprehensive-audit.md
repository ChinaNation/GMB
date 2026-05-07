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

2026-05-07 当前线程补充：已在审计报告顶部新增 v3 当前核验报告。v3 重新核验了 v2 中已过期的 P0 结论，并把当前仍需在重新创世前处理的真实问题分为 PR-A 残留清仓、PR-B 协议真源统一、PR-C runtime 清理、PR-D memory 创世冻结四批。

2026-05-07 第 1 步冻结：已新增 [20260506-230100-pre-regenesis-step1-freeze.md](20260506-230100-pre-regenesis-step1-freeze.md)，作为“重新创世前总审计”的第一步交付物。该文件冻结了当前必须先处理的 P0/P1/P2 清单和第 2 步执行顺序。

2026-05-07 本轮复核：已按用户要求重新执行第 1 步基线采集与关键协议扫描；修正任务卡文件名长度统计口径，当前按“文件名本身含 `.md`”计算没有超过 160 UTF-8 字节的任务卡。

2026-05-07 P0-1 执行：已清理 tracked 本地链状态、网络密钥、前端 dist、Python pycache 与 spec backup，并补 `.gitignore` 防止复发。`secret_ed25519` 按已泄露处理，重新创世后禁止复用。

2026-05-07 协议治理入口：已新增 [memory/07-ai/unified-protocols.md](../../07-ai/unified-protocols.md)。后续 P0-2 不再表述为“新增扫码协议”，而是统一 `WUMIN_QR_V1 / sign_request / payload_hex` 内层的 `OrganizationManage.propose_create_institution` 交易载荷格式。

2026-05-07 命名治理入口：已新增 [memory/07-ai/unified-naming.md](../../07-ai/unified-naming.md)。后续所有新建或重命名目录、文件、字段、变量、类、模块、API 字段、storage 字段、QR display 字段、任务卡文件名、文档文件名，都先按统一命名文件执行；不确定命名必须先报告确认。

2026-05-07 必读治理入口：已新增 [memory/07-ai/unified-required-reading.md](../../07-ai/unified-required-reading.md)。后续每次设计、编程、改协议、改命名、改文档、改流程前，先按统一必读文件确认必读清单。

2026-05-07 P0-2 执行：已按 `memory/07-ai/unified-protocols.md` 的 `P-TX-001` 统一 `OrganizationManage(17).propose_create_institution(5)`；`wuminapp` 改为读取 `/registration-info` 并按 10 字段编码，`wumin` 冷钱包 decoder 拒绝旧尾字段，`citizenchain node` display 补齐签发省份和签发管理员字段。

2026-05-07 P0-3 执行：已清理 `wuminapp` 对旧 `OrganizationManage::DuoqianAccounts` 的活跃读取；多钱账户信息改按注册机构 `AddressRegisteredSfid -> Institutions + InstitutionAccounts`、个人多签 `PersonalManage::PersonalDuoqians`、管理员阈值 `AdminsChange::Subjects` 三条真源读取，并补回归测试。

2026-05-07 P0-4 执行：已统一 `wuminapp` 在线 signed extrinsic 为 immortal era；新增 `SignedExtrinsicBuilder`，所有热钱包在线签名路径固定 `eraPeriod = 0 / era = 0x00 / blockNumber = 0 / blockHash = genesisHash`，不再用最新块 hash/number 构造签名 payload。

2026-05-07 P0-5 执行：已统一 Step2D fixture 真源到 `memory/06-quality/fixtures/step2d_credential_payload.json`；`cast_referendum` 固定为 `JointVote(23).cast_referendum(1)` 与 `0x1701` 前缀；删除 `wumin` / `wuminapp` 两份重复 fixture，并让两端测试读取同一份 fixture。

2026-05-07 P0-6 执行：已修正 CPMS 编译脚本默认读取的 SFID 目录为 `sfid/backend/sfid/`，同步省市码注释和统一命名文件；已删除 Wumin 本地脚本与 CI 中已失效的 `supportedSpecVersions` 写源码残留。

## 4. 验收

- 所有 [RENAME] / [DEAD] / [STORAGE] 类发现修完
- 所有 [TEST] 用例同步更新
- memory/ 内 OBSOLETE 项实际删除或归档
- `cargo check -p node`(citizenchain) / `flutter analyze`(wumin & wuminapp) / sfid / cpms 全部 0 警告
- 6 台 `fuwuqi.sh q` 清链重启成功,新链在 3 个 block 内能从主网导出 chainspec 与本地源码 diff 为零
