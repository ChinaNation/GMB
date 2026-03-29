# 任务卡：全面仔细的检查一遍 admins-origin-gov 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-112311
- 状态：done
- 所属模块：citizenchain/governance
- 当前负责人：Codex
- 创建时间：2026-03-28 11:23:11

## 任务需求

全面仔细的检查一遍 admins-origin-gov 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、治理核心规则


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`nodeui` 或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


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
- 已读取启动协议要求文档、模块技术文档、`admins-origin-gov` 代码、tests、benchmarks、weights、`voting-engine-system` 联动实现、runtime 配置与 `nodeui` 治理侧读取代码。
- 已执行验证：
  - `cargo test -p admins-origin-gov`
  - `cargo test -p voting-engine-system`
  - `cargo check -p citizenchain`

## 审查结论

### 总体判断

- `admins-origin-gov` 当前未发现新的高危越权投票、跨机构提案/投票绕过或管理员人数破坏漏洞。
- 核心安全边界基本成立：`org` / `institution` 对齐校验、本机构管理员发起与投票、人数恒定约束、通过后公开执行入口都在。
- 但模块当前实现明显弱于技术文档声明的生命周期能力，且文档、weights、残留字段没有同步收口，已经影响“功能需求是否严格实现”和“审计可信度”。

### 主要发现

1. `admins-origin-gov` 没有实现文档所要求的“同一机构同一时间只允许一个管理员替换提案”以及“已通过但执行失败时保留补救窗口并继续阻塞”的状态机。当前 `propose_admin_replacement` 只校验机构归属和管理员身份，然后直接调用 `InternalVoteEngine::create_internal_proposal`；而投票引擎真实提供的是“每机构最多 10 个活跃提案”的全局限额，不区分事项类型。更关键的是，投票引擎在提案一进入 `PASSED` 就立即释放活跃提案名额，所以管理员替换提案一旦投票通过，即使自动执行失败，也不会继续阻塞该机构发起新的替换提案。这与模块技术文档中“单机构单活跃提案”“执行失败保留补救窗口”“普通 stale 清理不可取消已通过失败提案”的要求不一致。
2. 模块技术文档已经严重过期，描述了一整套当前代码里不存在的存储与接口，包括 `ProposalActions`、`ProposalCreatedAt`、`ProposalPassedAt`、`ActiveProposalByInstitution`、`cancel_stale_proposal`、`StaleProposalLifetime`、`InactiveProposalAutoCleaned` 等；runtime 配置文档也仍写着 `type StaleProposalLifetime`，但当前 `impl admins_origin_gov::Config for Runtime` 根本没有这个配置项。现在这份文档不能再作为该模块的真实实现说明。
3. `admins-origin-gov/src/weights.rs` 明显停留在旧状态机口径：`WeightInfo` 只有 3 个 extrinsic，但注释和 proof 说明仍多次引用当前代码并不存在的 `AdminsOriginGov::ProposalActions / ProposalCreatedAt / ProposalPassedAt / ActiveProposalByInstitution` 等存储。这意味着这份 benchmark 产物至少已经不能证明“现实现状已重新量测”，属于需要清理的残留。
4. `AdminReplacementAction.executed` 字段和 `ProposalAlreadyExecuted` 错误基本处于死路径残留状态。代码在提案创建时始终写入 `executed: false`，后续执行成功时并不会回写该字段，而是仅靠投票引擎状态从 `PASSED` 改成 `EXECUTED` 来阻止再次执行。因此重复执行实际返回的是 `ProposalNotPassed`，测试也按这个结果断言；`executed` 字段和对应错误定义会误导维护者对真实防重机制的理解。

### 功能实现判断

- 当前代码已实现的部分：
  - 仅支持管理员替换，不支持增删人数。
  - 仅本机构当前管理员可提案、可投票。
  - 达阈值后自动尝试执行，失败不回滚已通过投票。
  - 可通过公开 `execute_admin_replacement` 补执行。
  - `CurrentAdmins` 会被 runtime 权限检查与 `nodeui` 查询路径使用。
- 当前未严格实现或已与文档偏离的部分：
  - 单机构同一时间只允许 1 个管理员替换提案。
  - 已通过但执行失败的补救窗口阻塞语义。
  - `cancel_stale_proposal` 及其相关生命周期清理。
  - 文档声明的专属动作存储与机构活跃索引。

### 注释、文档与残留判断

- 代码内中文注释总体够用，关键权限和执行路径可读性尚可。
- 技术文档不完整且和现实现有系统性偏差，不能视作可信真源。
- `weights.rs` 是明显残留。
- `executed` 字段、`ProposalAlreadyExecuted` 错误和相关注释也属于需要收口的残留。

## 完成信息

- 完成时间：2026-03-28 11:39:00
- 完成摘要：完成 admins-origin-gov 审查；未发现新的高危越权漏洞，但确认存在 4 类问题：模块未实现文档要求的“单机构单活跃提案 / 失败补救窗口”状态机、技术文档严重过期、weights 注释与 proof 口径停留在旧实现、`executed` 字段与 `ProposalAlreadyExecuted` 属于死路径残留。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
