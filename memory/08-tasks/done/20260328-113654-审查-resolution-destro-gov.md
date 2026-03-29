# 任务卡：全面仔细的检查一遍 resolution-destro-gov 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-113654
- 状态：done
- 所属模块：citizenchain/governance
- 当前负责人：Codex
- 创建时间：2026-03-28 11:36:54

## 任务需求

全面仔细的检查一遍 resolution-destro-gov 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

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
- 已读取启动协议要求文档、模块技术文档、`resolution-destro-gov` 代码、tests、benchmarks、weights、`voting-engine-system` / `grandpa-key-gov` / `admins-origin-gov` 联动实现、runtime 配置。
- 已执行验证：
  - `cargo test -p resolution-destro-gov`
  - `cargo test -p grandpa-key-gov`
  - `cargo check -p citizenchain`

## 审查结论

### 总体判断

- `resolution-destro-gov` 的机构边界、零金额拦截、ED 保护和公开补执行路径基本成立。
- 但当前实现存在 1 个需要优先处理的真实安全漏洞，以及 3 类明显的功能/文档残留问题。

### 主要发现

1. `resolution-destro-gov` 对通用 `proposal_data` 做了无类型前缀解码，存在跨模块 proposal 被误当成销毁动作执行的风险。该模块在 `vote_destroy` 和 `execute_destroy` 中直接把 `voting-engine-system` 里拿到的字节解码成 `DestroyAction { institution, amount }`，没有类型标签、长度校验或与 proposal 来源模块做绑定。由于 `DestroyAction` 结构很短，而 `admins-origin-gov`、`grandpa-key-gov` 这类内部投票模块写入的 proposal data 都以同样的 `institution` 前缀开头，按 SCALE 解码语义会被前缀成功解成一个伪造的 `amount`。其中 `execute_destroy` 又是公开触发入口，只要某个外部模块的内部提案停留在 `PASSED`（例如 `grandpa-key-gov` 自动执行失败后保留为通过状态），任意签名账户都可能把这个外部提案误执行成一次资金销毁。
2. 当前实现没有严格实现技术文档要求的“单机构同一时间只允许 1 个活跃销毁提案”和“stale 清理 / 通过失败补救窗口”状态机。`propose_destroy` 只依赖投票引擎的全局活跃提案上限，而投票引擎真实口径是“每机构最多 10 个活跃提案，不区分事项类型”；同时提案一旦进入终态就立即释放活跃名额，测试也明确验证了“被拒绝后可立刻发新提案”“已执行后可立刻发新提案”。因此模块文档中的专属活跃索引、stale 覆盖和 `cancel_stale_destroy` 并未在当前实现里存在。
3. 技术文档已经严重过期，仍把 `StaleProposalLifetime`、`ProposalActions`、`ProposalPassedAt`、`ActiveProposalByInstitution`、`cancel_stale_destroy`、`StaleDestroyCancelled`、14 以外的测试覆盖等内容写成“当前现实”，而 runtime 接线和模块代码都已经没有这些配置和接口。
4. `src/weights.rs` 也是明显残留：虽然 `WeightInfo` 只剩 3 个 extrinsic，但注释和 proof 说明仍反复引用当前代码中不存在的 `ResolutionDestroGov::ProposalActions / ProposalCreatedAt / ProposalPassedAt / ActiveProposalByInstitution` 等旧存储，已经不能作为现实现状的可信 benchmark 说明。

### 功能实现判断

- 当前已实现：
  - 仅有效机构、且 `org`/`institution` 匹配时可提案。
  - 仅内部管理员可发起和投票。
  - `amount > 0`、ED 保护、余额不足时自动执行失败但投票不回滚。
  - `execute_destroy` 可用于后续补执行，且对非管理员开放。
- 当前未严格实现或已与文档偏离：
  - 单机构单活跃销毁提案。
  - 模块自有 stale 清理与覆盖策略。
  - 文档中声明的专属存储模型与 `cancel_stale_destroy` 入口。

### 注释、文档与残留判断

- 代码中文注释基本够用，关键余额与 ED 路径可读性尚可。
- 技术文档存在系统性漂移，不能直接作为可信真源。
- `weights.rs` 属于明显旧实现残留。

## 完成信息

- 完成时间：2026-03-28 11:45:00
- 完成摘要：完成 resolution-destro-gov 审查；确认 1 个高优先级漏洞和 3 类实现/文档残留问题：通用 proposal_data 无类型解码可被外部已通过内部提案误触发销毁、单机构单活跃与 stale 状态机未按文档实现、技术文档严重过期、weights 注释仍停留在旧存储模型。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
