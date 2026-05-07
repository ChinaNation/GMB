# 清链重启前彻底审计 · v3 当前线程核验报告（2026-05-07）

- 任务关联：[20260506-230100-pre-regenesis-comprehensive-audit.md](20260506-230100-pre-regenesis-comprehensive-audit.md)
- 第 1 步冻结清单：[20260506-230100-pre-regenesis-step1-freeze.md](20260506-230100-pre-regenesis-step1-freeze.md)
- v3 基于当前工作区 `git ls-files`、全仓库 `rg` 扫描、关键源码/文档逐项回原文核验产出
- 当前第 1 步执行以冻结清单为准；若本报告与冻结清单存在细节差异，优先执行冻结清单
- 下方 v2 报告保留为历史记录；v3 已明确撤回/更新的条目，不再作为当前修复依据
- 2026-05-07 已新增统一协议入口：`memory/07-ai/unified-protocols.md`；后续所有协议、载荷格式、接口契约、字段顺序和签名验签规则先登记到该文件
- 2026-05-07 已新增统一命名入口：`memory/07-ai/unified-naming.md`；后续所有目录、文件、字段、变量、类、模块、API 字段、storage 字段、QR display 字段、任务卡文件名、文档文件名先按该文件登记或确认
- 2026-05-07 已新增统一必读入口：`memory/07-ai/unified-required-reading.md`；后续每次设计、编程、改协议、改命名、改文档、改流程前，先按该文件确认必读清单

---

## 0. v2 结论更新与撤回

| v2 条目 | 当前核验结论 | 证据 |
|---|---|---|
| P0-1：`citizenchain/node` 三处仍使用 mortal era | **已修复，不再成立** | [benchmarking.rs:125](../../../citizenchain/node/src/core/benchmarking.rs:125)、[rpc.rs:141](../../../citizenchain/node/src/core/rpc.rs:141)、[submitter.rs:232](../../../citizenchain/node/src/offchain/settlement/submitter.rs:232) 当前均为 `Era::Immortal` |
| P0-2：`wumin` 冷钱包 `supportedSpecVersions = {0}` | **已修复，不再成立** | [pallet_registry.dart:7](../../../wumin/lib/signer/pallet_registry.dart:7) 说明 `supportedSpecVersions / isSupported` 已物理移除；CI 与本地脚本旧写源码残留已在 P0-6 同批清理 |
| P0-3：`sfid` 后端/前端 `spec_version: 0` 硬编码 | **当前活跃代码未命中，不再按 v2 修** | 当前 `rg "spec_version"` 未在 v2 指定活跃文件命中 `spec_version: 0`；后续只需保持动态读取规则 |

---

## 1. 当前真实阻塞项（必须在重新创世前处理）

### P0-1：已 tracked 的本地链数据库、网络密钥和构建产物必须清出仓库（已执行）

原始冻结时 `git ls-files` 中存在本地链状态和生成物：

- `citizenchain/.local-node/chains/citizenchain/db/full/*`：本地 RocksDB 链数据被纳入版本库
- `citizenchain/.local-node/chains/citizenchain/network/secret_ed25519`：本地网络密钥被纳入版本库，属于高危残留
- `citizenchain/node/frontend/dist/*`：前端构建产物被 tracked
- `tools/__pycache__/fill_china_admins.cpython-314.pyc`：Python 字节码被 tracked
- `wuminapp/assets/light_sync_state.json.spec9-backup`：旧 light sync 状态备份被 tracked

执行结果（2026-05-07）：

- 已从 Git 索引和工作区移除上述本地状态文件、生成物、缓存与备份
- 已补 `.gitignore` 对 `citizenchain/.local-node/`、`citizenchain/node/frontend/dist/`、`**/__pycache__/`、`*.pyc`、`*.spec*-backup` 的明确规则
- `secret_ed25519` 已经进入仓库历史，按已泄露密钥处理；重新创世后不得复用
- 验收已通过：`git ls-files` 不再命中本地链状态、dist、pycache、pyc、spec backup 或 `secret_ed25519`

### P0-2：`wuminapp` 仍读取已删除的 `OrganizationManage::DuoqianAccounts`（已执行，见冻结清单 P0-3）

执行结果（2026-05-07）：

- `wuminapp` 已新增统一 storage codec，集中构造和解码 `AddressRegisteredSfid`、`Institutions`、`InstitutionAccounts`、`PersonalDuoqians`、`PersonalDuoqianInfo`、`AdminsChange::Subjects`。
- 注册机构多签信息已改走 `AddressRegisteredSfid -> Institutions + InstitutionAccounts`。
- 个人多签信息已改走 `PersonalManage::PersonalDuoqians`。
- 管理员与阈值已统一走 `AdminsChange::Subjects`。
- `wuminapp/lib` 与 `wuminapp/test` 已无旧 `DuoqianAccounts` 活跃读取。

以下为执行前审计证据，保留用于追溯：

Runtime 当前已经删除 `DuoqianAccounts mirror`，注册机构账户应走 `OrganizationManage::InstitutionAccounts`，个人多签应走 `PersonalManage::PersonalDuoqians`。但 `wuminapp` 活跃代码仍在读旧 storage：

- [duoqian_manage_service.dart:348](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:348) 注释仍称“从 `DuoqianAccounts` 存储解码”
- [duoqian_manage_service.dart:359](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:359) 构造 `DuoqianAccounts` storage key
- [institution_admin_service.dart:41](../../../wuminapp/lib/institution/institution_admin_service.dart:41) 注册型机构阈值来源仍写 `DuoqianAccounts.threshold`
- [institution_admin_service.dart:76](../../../wuminapp/lib/institution/institution_admin_service.dart:76) 注释称注册多签和个人多签都走 `DuoqianAccounts`
- [institution_admin_service.dart:143](../../../wuminapp/lib/institution/institution_admin_service.dart:143) 构造 `OrganizationManage::DuoqianAccounts(duoqian_address)` storage key

处理建议：

- 注册型机构主账户、状态、阈值改读 `OrganizationManage::InstitutionAccounts`
- 个人多签改读 `PersonalManage::PersonalDuoqians`
- 管理员与治理主体信息继续以 `admins-change::Subjects` 为真源
- 同步更新 `wuminapp` 相关模型、页面注释、测试 fixture 和 memory 文档

### P0-3：`wuminapp` 多条签名路径仍使用 mortal era，需要统一 era 协议（已执行，见冻结清单 P0-4）

执行结果（2026-05-07）：

- 已登记统一签名协议 `P-SIGN-001：Citizenchain signed extrinsic era`。
- `wuminapp` 已新增 `SignedExtrinsicBuilder`，统一构造 signed extrinsic。
- 在线签名 extrinsic 已固定 immortal era：`eraPeriod = 0`、`era = 0x00`、`blockNumber = 0`、`SigningPayload.blockHash = genesisHash`。
- 已替换 `OnchainRpc`、`InternalVoteService`、`RuntimeUpgradeService`、`TransferProposalService`、`DuoqianManageService`、`OnchainClearingBankRpc` 六条路径。
- signed extrinsic 构造路径不再调用 `fetchLatestBlock()`。

以下为执行前审计证据，保留用于追溯：

链端后端已经统一为 `Immortal`，但 `wuminapp` 热钱包仍有 `_eraPeriod = 64` 并把 latest block hash/number 写入 payload：

- [onchain.dart:28](../../../wuminapp/lib/rpc/onchain.dart:28)、[onchain.dart:86](../../../wuminapp/lib/rpc/onchain.dart:86)、[onchain.dart:100](../../../wuminapp/lib/rpc/onchain.dart:100)
- [internal_vote_service.dart:37](../../../wuminapp/lib/proposal/shared/internal_vote_service.dart:37)、[internal_vote_service.dart:113](../../../wuminapp/lib/proposal/shared/internal_vote_service.dart:113)、[internal_vote_service.dart:128](../../../wuminapp/lib/proposal/shared/internal_vote_service.dart:128)
- [runtime_upgrade_service.dart:36](../../../wuminapp/lib/proposal/runtime_upgrade/runtime_upgrade_service.dart:36)、[runtime_upgrade_service.dart:472](../../../wuminapp/lib/proposal/runtime_upgrade/runtime_upgrade_service.dart:472)、[runtime_upgrade_service.dart:487](../../../wuminapp/lib/proposal/runtime_upgrade/runtime_upgrade_service.dart:487)
- [transfer_proposal_service.dart:46](../../../wuminapp/lib/proposal/transfer/transfer_proposal_service.dart:46)、[transfer_proposal_service.dart:1124](../../../wuminapp/lib/proposal/transfer/transfer_proposal_service.dart:1124)、[transfer_proposal_service.dart:1139](../../../wuminapp/lib/proposal/transfer/transfer_proposal_service.dart:1139)
- [duoqian_manage_service.dart:50](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:50)、[duoqian_manage_service.dart:549](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:549)、[duoqian_manage_service.dart:564](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:564)
- [onchain_clearing_bank_rpc.dart:25](../../../wuminapp/lib/offchain/rpc/onchain_clearing_bank_rpc.dart:25)、[onchain_clearing_bank_rpc.dart:182](../../../wuminapp/lib/offchain/rpc/onchain_clearing_bank_rpc.dart:182)、[onchain_clearing_bank_rpc.dart:193](../../../wuminapp/lib/offchain/rpc/onchain_clearing_bank_rpc.dart:193)

处理建议：

- 先确认“PoW 链一律 immortal era”是否覆盖热钱包在线签名；若覆盖，则上述路径统一移除 `_eraPeriod` 和 latest block 依赖
- 若热钱包允许 mortal，必须在协议文档明确区分：后端/冷钱包/离线签名 immortal，热钱包在线签名 mortal，并给出安全边界

### P0-4：`wumin` / `wuminapp` Step2D fixture 漂移（已执行，见冻结清单 P0-5）

执行结果（2026-05-07）：

- 已登记统一交易载荷协议 `P-TX-002：JointVote.cast_referendum`。
- 已新增 `memory/06-quality/fixtures/step2d_credential_payload.json` 作为 Step2D 唯一 fixture 真源。
- `cast_referendum` 已统一到 `JointVote(23).cast_referendum(1)`，fixture call data 前缀固定 `0x1701`。
- 已删除 `wumin/test/fixtures/step2d_credential_payload.json` 与 `wuminapp/test/fixtures/step2d_credential_payload.json` 两份重复副本。
- `wumin` / `wuminapp` 测试都改读统一 fixture，并补 metadata/prefix 断言。

以下为执行前审计证据，保留用于追溯：

`wumin/test/fixtures/step2d_credential_payload.json` 与 `wuminapp/test/fixtures/step2d_credential_payload.json` 对 `cast_referendum` 的 expected call data 不一致：

- `wumin` fixture 的 `expected_call_data` 已是 `0x1701...`，对应当前 `JointVote.cast_referendum`
- `wuminapp` fixture 的 `expected_call_data` 仍是 `0x0902...`，疑似旧 `VotingEngine` 路由
- 两边 fixture metadata 仍写 `"pallet_index": 9`、`"call_index": 2`，与 `0x1701...` 不一致

处理建议：

- 统一 fixture 到当前 runtime 路由
- `expected_call_data`、metadata、payload decoder 测试同时更新，避免冷钱包和热钱包对同一凭证产生不同解读

---

## 2. 高优先级清理项（建议与 P0 同批或紧随其后）

### P1-1：`organization-manage` 遗留 `finalize_create` 物理残留

当前仍有已删除流程的实体类型、事件、错误枚举或注释：

- [organization-manage/src/lib.rs:175](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs:175) `AdminSignaturesOf<T>` 标注为 legacy finalize_create 类型
- [organization-manage/src/lib.rs:350](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs:350) `CreateFinalized` 事件已无 emit 路径
- [organization-manage/src/lib.rs:465](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs:465) legacy 错误说明仍引用已删路径
- [organization-manage/src/lib.rs:476](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs:476) `MalformedSignature` 仍被 [organization-manage/src/lib.rs:727](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs:727) 使用，不能整块误删

处理建议：

- 只删除确无引用的 legacy 类型/事件/错误
- 保留仍被活跃逻辑使用的 `MalformedSignature`
- 删除前后跑 `cargo check` 和 runtime 相关测试

### P1-2：`wumin/scripts/wumin-run.sh` 仍修改已移除的 `supportedSpecVersions`（已执行）

[wumin-run.sh](../../../wumin/scripts/wumin-run.sh) 原先仍会按链上 spec_version 回写冷钱包源码；但 [pallet_registry.dart:7](../../../wumin/lib/signer/pallet_registry.dart:7) 已说明 `supportedSpecVersions / isSupported` 被物理移除。

执行结果（2026-05-07）：

- 已删除 `wumin/scripts/wumin-run.sh` 中读取链上 spec_version 并写源码的旧逻辑
- 已删除 `.github/workflows/wumin-ci.yml` 中同类写源码逻辑
- 两处仍保留 pallet/call index 同步，后续若要继续精简，应改成“校验不改源码”

### P1-3：`organization-manage/src/institution/close.rs` 是空壳且注释边界已错

[close.rs:1](../../../citizenchain/runtime/governance/organization-manage/src/institution/close.rs:1) 当前只是目录占位说明，且称“个人多签和机构多签共用同一条关闭逻辑”。现状已经不是这样：机构关闭在 `organization-manage`，个人关闭在 `personal-manage`。

处理建议：

- 若无实际引用，删除该占位模块和 `mod` 导出
- 若保留目录边界，必须改成当前真实边界说明，避免误导后续模块拆分

### P1-4：链端和 memory 文档仍有 `DuoqianAccounts` / 旧机构真源叙述

代表性残留：

- [fee_config.rs:37](../../../citizenchain/runtime/transaction/offchain-transaction/src/fee_config.rs:37) 仍称清算行管理员通过 `duoqian-manage` 的 `DuoqianAccounts` 校验
- [ORGANIZATION_MANAGE_TECHNICAL.md:43](../../05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md:43) 仍列 `DuoqianAccounts<main_address, DuoqianAccount>`
- [DUOQIAN_TRANSFER_TECHNICAL.md:84](../../05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md:84) 仍称资金账户从 `organization-manage::DuoqianAccounts` 校验
- [GOVERNANCE_TECHNICAL.md:78](../../05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md:78) 仍称阈值读取 `DuoqianAccounts.threshold`
- [GOVERNANCE_TECHNICAL.md:601](../../05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md:601) 仍同时使用 `AdminsChange.Institutions` 与 `DuoqianManage.DuoqianAccounts`

处理建议：

- 注册机构：统一写 `OrganizationManage::InstitutionAccounts`
- 个人多签：统一写 `PersonalManage::PersonalDuoqians`
- 管理员/阈值/治理主体：统一写 `AdminsChange::Subjects`
- 旧 `DuoqianAccounts` 只允许出现在明确标注为 legacy/history 的文档段落

---

## 3. 结构与编号统一项

### P2-1：`memory/05-architecture/` 与正式文档编号体系不一致

`memory/01-architecture/repo-map.md` 定义的正式文档目录没有 `memory/05-architecture/`，但当前该目录内有 QR 协议与并发框架文档，例如：

- `memory/05-architecture/qr-protocol-spec.md`
- `memory/05-architecture/qr-action-registry.md`
- `memory/05-architecture/qr-signing-recognition.md`
- `memory/05-architecture/20260409-sfid-50k-concurrent-framework.md`

处理建议：

- 架构级协议迁入 `memory/01-architecture/`
- 模块实现细节迁入 `memory/05-modules/<module>/`
- 迁移后更新引用，避免出现两个“05”语义

### P2-2：`memory/tasks/` 旧任务目录与 `memory/08-tasks/` 制度冲突

当前 tracked 旧任务文件：

- `memory/tasks/smoldot-checkpoint-plan.md`
- `memory/tasks/smoldot-kbuckets-dht.md`
- `memory/tasks/smoldot-stability-plan.md`

处理建议：

- 未完成项迁入 `memory/08-tasks/open/`
- 已完成或废弃项迁入 `memory/08-tasks/done/` 或模块技术文档
- 迁移后删除 `memory/tasks/` 空目录入口

### P2-3：根目录 `docs/` 的定位未写入 repo-map

当前 tracked：

- `docs/index.html`
- `docs/FRC_README.html`
- `docs/GMB_README.html`
- `docs/logo.png`
- `docs/logo.svg`

处理建议：

- 若它是 GitHub Pages 发布目录，在 repo-map 中明确边界：只放发布静态页，不放系统权威文档
- 若不是发布目录，则迁入 `website/` 或 `memory/`，并删除重复生成物

### P2-4：`memory/08-tasks/open/` 长期 open 任务过多

当前 `memory/08-tasks/open/` 有 90 个 `.md` 文件。重新创世前应做一次任务卡冻结：

- 已完成：移动到 `memory/08-tasks/done/`
- 被新任务取代：标记 `OBSOLETE` 后归档
- 仍未完成：保留 open，并补“下一步阻塞点”

---

## 4. 启动协议一致性

当前核验：

- 根 `AGENTS.md` 与 `memory/AGENTS.md` 一致
- 根 `CODEX.md` 与 `memory/CODEX.md` 一致
- 根 `CLAUDE.md` 与 `memory/CLAUDE.md` 一致
- `bash memory/scripts/check-startup-acceptance.sh` 通过

仍建议统一：

- `CODEX.md` 已显式包含“检查为什么报错”只读诊断例外
- `CLAUDE.md` 入口说明中未同等显式展开该例外，虽然下层 `AGENTS.md` 已覆盖

处理建议：

- 在 `CLAUDE.md` / `memory/CLAUDE.md` 中补齐同一句只读诊断例外，确保 Codex / Claude 两个入口的首屏规则读感一致

---

## 5. 预计修改目录

| 目录 | 用途、边界与类型 |
|---|---|
| `citizenchain/.local-node/` | 清理本地链数据库和网络密钥残留；只做版本库索引移除和 ignore 规则，不作为源码目录保留 |
| `citizenchain/node/frontend/dist/` | 清理前端构建产物；不改业务代码 |
| `citizenchain/runtime/governance/organization-manage/` | 删除 `finalize_create` 物理残留、修正机构关闭边界；涉及 runtime 代码与中文注释 |
| `citizenchain/runtime/transaction/offchain-transaction/` | 修正清算行/费率注释中的旧 storage 真源；主要涉及代码注释和技术文档 |
| `wumin/scripts/` | 删除已失效的 `supportedSpecVersions` 写源码逻辑；涉及脚本和中文说明 |
| `wumin/test/fixtures/` | 对齐冷钱包 Step2D fixture；涉及测试数据 |
| `wuminapp/lib/` | 修正旧 `DuoqianAccounts` 查询、统一 era 签名协议；涉及热钱包业务代码与中文注释 |
| `wuminapp/test/fixtures/` | 对齐热钱包 Step2D fixture；涉及测试数据 |
| `memory/01-architecture/` | 承接架构级协议文档迁移；只放跨模块协议和仓库结构规则 |
| `memory/05-modules/` | 修正文档中的 storage 真源、模块名和协议描述；只放模块技术文档 |
| `memory/07-ai/` | 如需补审计方法论或入口规则，只改 AI 协议文档 |
| `memory/08-tasks/` | 归档旧任务卡、保留本次审计与后续修复任务；只改任务卡状态与索引 |
| `docs/` / `website/` | 决定静态发布目录归属；涉及文档/发布产物边界，不承载系统权威记忆 |

---

## 6. 推荐执行批次

| 批次 | 内容 | 验收 |
|---|---|---|
| PR-A：残留清仓 | P0-1 已完成：移除 tracked 本地链数据、网络密钥、dist、pyc、backup；补 `.gitignore`；标记密钥不可复用 | `git ls-files` 已不再出现本地状态/生成物；ignore 规则已命中 |
| PR-B：协议真源统一 | P0-2 已完成 `propose_create_institution(17.5)` 三端统一；P0-3 已完成 `wuminapp` 旧 `DuoqianAccounts` 查询清理；P0-4 已完成热钱包 immortal era 统一；P0-5 已完成 Step2D fixture 真源统一；P0-6 已完成 `wumin` 旧 spec sed 与 CPMS 旧 SFID 路径清理 | P0-2/P0-3/P0-4/P0-5 相关 Flutter 测试、wumin decoder 测试和目标 analyze 已通过；P0-6 `cargo check` / `bash -n` / 旧残留 rg 扫描已通过，CPMS 仍有既有 warning 待后续清理；`cargo check -p node` 被 `WASM_FILE` 硬规则阻断 |
| PR-C：runtime 清理 | 清 `organization-manage` legacy finalize_create 物理残留；修 close 模块边界；修链端旧 storage 注释 | `cargo check` 通过；无误删仍活跃的 error/event |
| PR-D：memory 创世冻结 | 迁移 `memory/05-architecture`、`memory/tasks`；明确 `docs/` 定位；归档 90 个 open 任务；补齐 Claude/Codex 入口一致性 | repo-map 与实际目录一致；open 任务只剩真实未完成项 |

---

## 7. 下一步建议

**PR-A / P0-1 残留清仓已执行；PR-B / P0-2 机构创建载荷统一、P0-3 旧 `DuoqianAccounts` 查询清理、P0-4 热钱包 immortal era 统一、P0-5 Step2D fixture 真源统一、P0-6 旧路径与 spec 脚本残留清理均已执行**。下一步进入 PR-C：清理 `organization-manage` legacy `finalize_create` 物理残留，并复核仍活跃的 `MalformedSignature` 不被误删。

---

# 历史报告：清链重启前彻底审计 · 修正版总清单（v2）

- 任务关联：[20260506-230100-pre-regenesis-comprehensive-audit.md](20260506-230100-pre-regenesis-comprehensive-audit.md)
- v1 报告基于 6 个 Explore agent 输出未经原文核验,大量误判已删除
- v2 每条都标 **证据来源**(原文行号 + ADR/项目记忆),只列**已核验的真实问题**

---

# 一、v1 报告误判撤回（避免再次踩坑）

| v1 标记 | 实际状态 | 证据 |
|---|---|---|
| ❌ "P0-1：删 ClearingBank 整套" | **保留**,清算行体系完全活跃 | [ADR-006:35-38](../04-decisions/ADR-006-扫码支付-step1-同行MVP.md) L0-L3 四层架构里 L2=清算行 ; [ADR-007](../04-decisions/ADR-007-clearing-bank-three-phase.md) Step 1+2 done, Step 3 待 ; [project_institution_create_rules:澄清块](../../../.claude/projects/-Users-rhett-GMB/memory/project_institution_create_rules.md)「废除指 is_clearing_bank 字段模型,并非业务概念废除」 |
| ❌ "P0-2：删 SafetyFund / Sweep 提案 UI" | **保留**,extrinsic 真实存在 | [duoqian-transfer/lib.rs:443](../../citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs:443) `propose_safety_fund_transfer` ; [:527](../../citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs:527) `propose_sweep_to_main` ; [configs/mod.rs:289](../../citizenchain/runtime/src/configs/mod.rs:289) `safety_fund_account()` ; 前端 NrcSection/PrcSection 路由都正常 |
| ❌ "P0-2：删 RuntimeUpgradeProposalPage" | **保留**,joint vote 升级路径仍是合法路径 | [runtime-upgrade/lib.rs:163](../../citizenchain/runtime/governance/runtime-upgrade/src/lib.rs:163) `propose_runtime_upgrade` 与 [:217](../../citizenchain/runtime/governance/runtime-upgrade/src/lib.rs:217) `developer_direct_upgrade` 同时活跃 |
| ❌ "wuminapp ClearingBank 是死代码" | **是 ADR-007 Step 3 in-progress 工作** | [ADR-007 进度行](../04-decisions/ADR-007-clearing-bank-three-phase.md) "Step 3 待启动" |
| ❌ "wumin clearing_bank 5 条 op 死代码" | **是 ADR-007 Step 3 已就位的 wumin 部分** | wumin 的 ClearingBank decoder 与 action_labels 已为 Step 3 准备 |
| ❌ "ADR-007 整 ADR 失效需归档" | **保留**,Step 3 待实施时仍是权威依据 | 同上 |

---

# 二、已核验的真实问题（按优先级）

每条标 **证据**(文件:行 + 实际查到的代码片段)。

## 🔴 P0-1：Era 不对称(确认 mortal/immortal 共存)

[feedback_sfid_pow_chain_recipe.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_sfid_pow_chain_recipe.md) 立铁律「PoW 链冷钱包/后端推链一律 immortal era」,但当前代码三处仍 mortal:

| 文件 | 行 | 现状 |
|---|---|---|
| [citizenchain/node/src/core/benchmarking.rs:130](../../citizenchain/node/src/core/benchmarking.rs:130) | 130 | `Era::mortal(period, best_block.saturated_into())` |
| [citizenchain/node/src/core/rpc.rs:146](../../citizenchain/node/src/core/rpc.rs:146) | 146 | `Era::mortal(period, best_number.saturated_into())` |
| [citizenchain/node/src/offchain/settlement/submitter.rs:240](../../citizenchain/node/src/offchain/settlement/submitter.rs:240) | 240 | `Era::mortal(period, best_block.saturated_into())` |

[citizenchain/node/src/governance/signing.rs](../../citizenchain/node/src/governance/signing.rs) 已 immortal。这三处需对齐。

修法：`Era::Immortal`(Substrate API 直接有该枚举)替换。

## 🔴 P0-2：wumin 冷钱包 supportedSpecVersions 锁死 spec=0

[wumin/lib/signer/pallet_registry.dart:23](../../wumin/lib/signer/pallet_registry.dart:23):
```dart
static const Set<int> supportedSpecVersions = {0};
```

清链重启后 spec 回到 0,但下次 setCode 立刻 spec=1。提前加 `1` 进集合(或后续每次升级都要再发版冷钱包)。

[wumin/test/signer/payload_decoder_test.dart:603](../../wumin/test/signer/payload_decoder_test.dart:603) 等多处用 `supportedSpecVersions.first` 作 spec 默认值,加 1 后测试需同步。

## 🔴 P0-3：sfid 后端/前端 spec_version 硬编码 0

| 文件 | 行 | 代码 |
|---|---|---|
| [sfid/backend/citizens/binding.rs:51](../../sfid/backend/citizens/binding.rs:51) | 51 | `"spec_version": 0,` 硬编码 |
| [sfid/frontend/sheng_admins/SuperAdminSubTab.tsx:426](../../sfid/frontend/sheng_admins/SuperAdminSubTab.tsx:426) | 426 | `spec_version: 0,` 硬编码 |

清链后 0→1 立坏,且违反 [feedback_no_chain_changes.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_no_chain_changes.md) 精神(应从链动态读)。

修法:从 `state_getRuntimeVersion` 读取(后端缓存,前端拉一次)。

## 🟠 P1-1：MORTAL_ERA_PERIOD 死常量 + encode_mortal_era 死函数

[citizenchain/node/src/governance/signing.rs:17](../../citizenchain/node/src/governance/signing.rs:17) `MORTAL_ERA_PERIOD: u64 = 64` 与 [signing.rs:1286](../../citizenchain/node/src/governance/signing.rs:1286) `encode_mortal_era` 函数 —— 我前几小时改 immortal 后 lib 里无引用。但单元测试 `encode_mortal_era_period64` 还在测它(line 1460+)。

修法:删常量 + 函数 + 对应单元测试,一并清理。

## 🟠 P1-2：wumin payload_decoder.dart 注释头与实际不符

[wumin/lib/signer/payload_decoder.dart:858-859](../../wumin/lib/signer/payload_decoder.dart:858):
```dart
// OrganizationManage(17) / propose_create_personal(4)
// 格式：[17][4][BoundedVec account_name]...
```

实际路由是 `PersonalManage(7) call_index=0`(B 阶段拆分,2026-05-06 已落地)。注释头还停在拆分前的旧形态。函数体逻辑正确,**只是文档头错**。

修法:改注释为 `PersonalManage(7) / propose_create(0)`,格式行 `[7][0][...]`。

## 🟠 P1-3：memory 文档里 AdminsChange::Institutions 旧 storage 名残留

C 阶段 rename `Institutions → Subjects` 已落地,但下面 9 处文档仍写旧名:

| 文件 | 位置 |
|---|---|
| [citizenchain/runtime/governance/grandpakey-change/src/weights.rs](../../citizenchain/runtime/governance/grandpakey-change/src/weights.rs) | 51, 84(weights proof 注释) |
| [citizenchain/runtime/governance/resolution-destro/src/weights.rs](../../citizenchain/runtime/governance/resolution-destro/src/weights.rs) | 58, 85 |
| [citizenchain/runtime/governance/runtime-upgrade/src/weights.rs](../../citizenchain/runtime/governance/runtime-upgrade/src/weights.rs) | 56, 87 |
| [memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md](../05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md) | 28, 112-145 |
| [memory/05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md](../05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md) | 126 |
| [memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md](../05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md) | 247 |
| [memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md](../05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md) | 84 |
| [memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md](../05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md) | 22 |
| [memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_2_A_SUBMITTER.md](../05-modules/citizenchain/node/offchain/STEP2B_II_B_2_A_SUBMITTER.md) | 52 |
| [citizenchain/node/frontend/offchain/api.ts](../../citizenchain/node/frontend/offchain/api.ts) | 152, 155, 158, 168(注释) |
| [citizenchain/node/frontend/offchain/organization-manage/institution-detail.tsx](../../citizenchain/node/frontend/offchain/organization-manage/institution-detail.tsx) | 1(注释) |

修法:全局替换 `AdminsChange::Institutions` → `AdminsChange::Subjects`(注释/文档/weights proof)。

## 🟠 P1-4：runtime 注释里 OBSOLETE concept 残留

| 文件 | 行 | 残留概念 |
|---|---|---|
| [organization-manage/src/lib.rs](../../citizenchain/runtime/governance/organization-manage/src/lib.rs) | 100, 172, 347, 1027 | `finalize_create`(2026-05-02 Phase 3 已删) |
| [organization-manage/src/lib.rs](../../citizenchain/runtime/governance/organization-manage/src/lib.rs) | 477, 488, 840 | `ACTION_CREATE_PERSONAL` / 单账户机构 call=0 / call=3 hole |
| [organization-manage/src/institution/create.rs:4](../../citizenchain/runtime/governance/organization-manage/src/institution/create.rs:4) | 4 | 同上 |
| [organization-manage/src/benchmarks.rs:187](../../citizenchain/runtime/governance/organization-manage/src/benchmarks.rs:187) | 187 | 同上 |
| [runtime/src/configs/mod.rs](../../citizenchain/runtime/src/configs/mod.rs) | 351, 352, 1730 | 同上 |
| [node/src/governance/signing.rs:122](../../citizenchain/node/src/governance/signing.rs:122) | 122 | `vote_X` wrapper extrinsics(Phase 4 已删) |

修法:注释更新为「Phase 3/4 unified voting 已删」或直接清掉过时叙述。

## 🟡 P2-1：sfid chain_client 仅文档化 SfidSystem(10),其它 pallet 缺

[sfid/backend/app_core/chain_client.rs:48-57](../../sfid/backend/app_core/chain_client.rs:48):
```rust
pub(crate) const SFID_SYSTEM_PALLET_INDEX: u8 = 10;
pub(crate) const CALL_INDEX_ADD_SHENG_ADMIN_BACKUP: u8 = 2;
...
```

只列 SfidSystem(10) 的 pallet+call。**实际 sfid backend 推链只发 sfid_system 的 extrinsic**(根据这些常量名),所以**严格说不算 bug**。但如果未来要让 sfid 后端推 organization-manage 或 personal-manage 的 extrinsic(比如帮机构创建多签),需要补 pallet_index=17 / 7。

修法:**清链重启前不动**。等 sfid 后端确实需要发其它 pallet 的 extrinsic 时再补。

## 🟡 P2-2：sfid backend `#[allow(dead_code)]` 占位多处

| 文件 | 行 | 内容 |
|---|---|---|
| [sfid/backend/main.rs](../../sfid/backend/main.rs) | 32-36, 55-56, 62-64 | qr / cpms_register_inflight / sharded_store transition 占位 |
| [citizenchain/runtime/transaction/offchain-transaction/src/solvency.rs:62](../../citizenchain/runtime/transaction/offchain-transaction/src/solvency.rs:62) | 62 | `emit_warning_if_low` 死函数(Step 3 占位) |
| [citizenchain/runtime/governance/organization-manage/src/common.rs:43](../../citizenchain/runtime/governance/organization-manage/src/common.rs:43) | 43 | `_unused_dispatch_result_anchor` 占位 |

修法:清链前不动(占位的功能本就在等 Step 3 等后续阶段),只在 PR-5 文档清扫顺手验证一遍是否还需要。

## 🟡 P2-3：wumin test fixture step2d_credential_payload.json 含已删用例

[wumin/test/fixtures/step2d_credential_payload.json](../../wumin/test/fixtures/step2d_credential_payload.json) 含 `propose_runtime_upgrade` 用例,但 [payload_decoder_test.dart:603](../../wumin/test/signer/payload_decoder_test.dart:603) 注释明说该 case 已删。

修法:从 fixture 删该 entry。

## 🟡 P2-4：wuminapp test 用 wuminapp 自己的 ClearingBank* 测试可保留

[wuminapp/test/trade/clearing_bank_settings_page_test.dart](../../wuminapp/test/trade/clearing_bank_settings_page_test.dart)、[clearing_bank_prefs_test.dart](../../wuminapp/test/trade/clearing_bank_prefs_test.dart) —— ADR-007 Step 3 in-progress,**保留**(不算死代码)。但要确认这些测试是否真覆盖了当前 wuminapp 实现(Step 3 还没完整落地,可能测试断言错的对象)。

修法:清链前不动,Step 3 落地时一并 review。

## 🔵 P3-1：InstitutionPalletId 已改名 SubjectId,memory 文档未同步

| 文件 | 位置 |
|---|---|
| [memory/05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md](../05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md) | 39, 49, 50 |
| [memory/05-modules/citizenchain/runtime/STEP2_D_LAYER_B_PALLET_INTEGRATION.md](../05-modules/citizenchain/runtime/STEP2_D_LAYER_B_PALLET_INTEGRATION.md) | multiple |

修法:全局替换 `InstitutionPalletId → SubjectId`。

## 🔵 P3-2：memory 老 task card 长期 open 未关闭

| 文件 | 状态 |
|---|---|
| [memory/08-tasks/open/20260322-公民投票完整实现.md](20260322-公民投票完整实现.md) | 2026-03 任务,已超期 |
| [memory/08-tasks/open/20260325-133531-统一-duoqian-manage-命名.md](20260325-133531-统一-duoqian-manage-命名.md) | 已被 [20260505-215047 rename](20260505-215047-rename-org-manage-to-organization-manage.md) 取代 |
| [memory/08-tasks/open/20260331-step2-node-offchain-rpc-链下待结算账本.md](20260331-step2-node-offchain-rpc-链下待结算账本.md) | 2026-03 开,无后续 |
| [memory/08-tasks/open/20260401-step2a-offchain-pallet-简化密钥机制.md](20260401-step2a-offchain-pallet-简化密钥机制.md) | 同上 |
| [memory/08-tasks/open/20260404-offchain-packer-worker-接入.md](20260404-offchain-packer-worker-接入.md) | 同上 |
| [memory/08-tasks/open/20260502-sfid-step2b-duoqian-manage-credential.md](20260502-sfid-step2b-duoqian-manage-credential.md) | 标题旧名 |

修法:逐一确认是否真的完成 → 移到 `done/` 或者用 OBSOLETE 注脚标记。

## 🔵 P3-3：ADR-010 时间逆序

[memory/04-decisions/ADR-010-subject-id-protocol.md:11](../04-decisions/ADR-010-subject-id-protocol.md:11) 写「A 阶段(2026-05-04)前的派生协议」但 ADR 自身日期标 2026-05-06。

修法:核对实际拆分时间,统一时间线。

---

# 三、清链重启路径(精简版)

按 v2 报告分 3 个 PR(原 v1 的 5 个 PR 中 PR-1/2 整体撤回):

| PR | 内容 | 估时 |
|---|---|---|
| **PR-A** | P0-1 / P0-2 / P0-3:Era 切 immortal 三处 + wumin spec 集合加 1 + sfid spec_version 动态读 | 1-2 h |
| **PR-B** | P1-1 / P1-2 / P1-4:删 MORTAL_ERA_PERIOD 死常量 + 改 wumin 注释头 + 清 OBSOLETE 注释 | 1 h |
| **PR-C** | P1-3 / P3-1 / P3-2 / P3-3:文档/memory 全局清扫 + 老 task card 整理 | 2-3 h |

3 个 PR 全部合并 + cargo/flutter check 0 警告 + 测试通过 → **6 台 `fuwuqi.sh q` 清链重启** → 重新导出 chainspec → 删客户端 Institutions 临时 patch → 重 build → UI 应该正常(因为重启后所有数据都在 NEW 路径,无需绕过)。

---

# 四、本次审计的方法论教训(立条铁律)

**新铁律建议**:

> Explore subagent 报告**不是结论**,只是**线索池**。每条 finding 都必须回原文(代码 `file:line` + 权威 ADR/项目记忆)核验后才能进入正式报告。Agent 输出可能基于 prompt 过度简化或对历史叙述误读。

应在 [memory/07-ai/agent-rules.md](../07-ai/agent-rules.md) 或新建 `memory/07-ai/audit-recipe.md` 立条:**审计类任务,subagent 输出仅作 leads,正式报告以 `file:line + ADR 锚点`形式给证据**。

---

# 五、对比 v1 报告的核心修正

| v1 类别 | v1 数量 | v2 数量 | 删减/重定 |
|---|---:|---:|---|
| P0 阻塞 | 7 大类 | **3 项** | 删 P0-1(ClearingBank)、P0-2(Safety/Sweep/Upgrade UI)、P0-6(CPMS QR1 待用户确认)、P0-7 合入 P0-3 |
| P1 命名残留 | 5 大类 | **4 项** | 删 P1-3(wumin PersonalManage 二重身改为只是 stale comment),P1-5(memory OBSOLETE 归档撤回:ADR-007 不归档) |
| P2 死代码 | 8 处 | **4 项** | 死代码确认 4 处真死,4 处 in-progress 占位保留 |
| P3 文档漂移 | 5 大类 | **3 项** | 大部分合并到 P1-3 |
| P4 推荐改进 | 4 项 | **1 项** | 只剩"audit 方法论铁律" |

**v1 → v2 删除约 60% 误判项**,v2 保留 **~25 条已核验真实问题**,均以 ADR / 代码 file:line 为证据。
