# 任务卡：全面仔细检查 grandpakey-change 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

- 任务编号：20260328-112224
- 状态：open
- 所属模块：citizenchain/runtime/governance/grandpakey-change
- 当前负责人：Codex
- 创建时间：2026-03-28 11:22:24

## 任务需求

全面仔细检查 grandpakey-change 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/grandpakey-change/GRANDPAKEYCHANGE_TECHNICAL.md

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
- 已读取模块代码、benchmark、weights、runtime 配置与模块技术文档
- 已执行 `cargo test -p grandpakey-change`
- 已执行 `cargo check -p grandpakey-change`
- 已执行 `cargo check -p grandpakey-change --features runtime-benchmarks`
- 已执行 `cargo check -p citizenchain`
- 已执行模块源码 `rustfmt --check`

## 审查结论

### 已确认

- 未发现可直接绕过管理员权限或直接破坏 GRANDPA 换钥流程的高危漏洞
- 弱 ed25519 公钥拒绝、内部管理员权限检查、通过后自动执行、临时失败后手动重试、不可执行失败提案清理这几条主路径已实现

### 已确认问题与改进点

1. 功能需求未严格实现：代码没有落实“同一机构同一时间只允许一个活跃提案”与“同一把 new_key 同一时间只能被一个活跃提案占用”。
   - `grandpakey-change` 代码里没有 `ActiveProposalByInstitution`、`PendingProposalByNewKey`、`ProposalActions`、`ProposalCreatedAt` 这些文档和 weight 声称存在的本地存储
   - 当前真实限制来自 `voting-engine` 的全局活跃提案上限：每机构最多 10 个，而不是 1 个
   - `new_key` 只检查“当前是否已被其他机构使用”，不检查“是否已被其他活跃提案预占用”
   - 结果是：同机构可并行提出多个换钥提案，不同机构也可同时提出同一个未来 `new_key`，冲突会拖到执行阶段才暴露

2. `weights.rs` 明显失真，仍引用代码里不存在的存储与旧调用模型：
   - `ActiveProposalByInstitution`
   - `PendingProposalByNewKey`
   - `ProposalActions`
   - `ProposalCreatedAt`
   这说明 benchmark 结果没有跟上当前实现，weight 可信度不足。

3. 技术文档大面积漂移：
   - 文档描述了不存在的存储
   - 文档描述了不存在的 `cancel_stale_replace_grandpa_key` call index = 3
   - 文档描述的“下一次 propose 自动清理 rejected/stale 旧提案索引”当前代码没有对应实现
   - 文档测试覆盖与当前真实测试数量和内容不一致

4. 测试覆盖不足：
   - 没有直接测试“同机构并行提多个提案”
   - 没有直接测试“两家机构同时提出同一个新 key”
   - 没有直接测试文档里声称存在的 stale/rejected 自动清理行为

5. 工具链残留：
   - `memory/scripts/load-context.sh` 未登记 `citizenchain/runtime/governance/grandpakey-change`

### 说明

- 当前模块“能编译、能跑主路径测试”不代表功能需求已严格落地；主要偏差点在并发治理约束与文档/weight 同步
