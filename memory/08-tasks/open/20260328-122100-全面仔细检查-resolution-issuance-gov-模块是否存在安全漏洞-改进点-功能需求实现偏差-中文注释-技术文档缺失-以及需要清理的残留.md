# 任务卡：全面仔细检查 resolution-issuance-gov 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

- 任务编号：20260328-122100
- 状态：open
- 所属模块：citizenchain/runtime/governance/resolution-issuance-gov
- 当前负责人：Codex
- 创建时间：2026-03-28 12:21:00

## 任务需求

全面仔细检查 resolution-issuance-gov 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/resolution-issuance-gov/RESOLUTIONISSUANCEGOV_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 审查结论
- 风险点
- 改进建议
- 文档/残留清单

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已读取模块代码、benchmark、weights、runtime 配置与模块技术文档
- 已核对 `resolution-issuance-iss` 执行模块与 `voting-engine-system` 回调路径
- 已执行 `cargo test -p resolution-issuance-gov`
- 已执行 `cargo check -p resolution-issuance-gov`
- 已执行 `cargo check -p resolution-issuance-gov --features runtime-benchmarks`
- 已执行 `cargo check -p citizenchain`
- 已执行模块级 `rustfmt --check`

## 审查结论

### 已确认

- 未发现可由普通用户直接利用的高危安全漏洞
- 提案创建、联合投票回调、执行失败状态覆盖、事务回滚、治理期间禁止切换收款集合这些主路径已实现
- 单测、模块编译、runtime-benchmarks 编译和 runtime 总体编译均通过

### 已确认问题与改进点

1. 功能需求未严格实现：`set_allowed_recipients` 没有落实“合法收款账户必须对应 CHINA_CB 省储会固定多签地址”“省储会集合只允许新增、不允许删除”。
   - 文档把这两条列成明确业务要求
   - 代码当前只校验：Root 权限、非空、无重复、当前没有 Voting 中提案
   - 结果是：Root 可以把名单替换成任意账户，也可以删除既有省储会账户；这不属于普通权限绕过，但确实不满足文档要求

2. `weights.rs` 明显过期，仍保留已删除旧存储/旧模型：
   - 文件头已直接承认生成自旧代码
   - 仍引用 `NextProposalId`、`Proposals`、`GovToJointVote`、`JointVoteToGov`、`RetryCount`
   - 当前代码已切到投票引擎统一 ID + ProposalData 模型，weight 可信度不足

3. 技术文档不完整：
   - 内容上基本跟现代码一致，但文件存在大量乱码，已经影响可读性和审计
   - 代码位置写成了错误路径：`runtime/issuance/resolution-issuance-gov/src/lib.rs`，实际在 `runtime/governance/...`
   - 文档里关于 “weights.rs 在旧代码上生成” 的说明是对的，但正文其它位置仍有乱码导致信息质量下降

4. 残留与可维护性问题：
   - `ProposalNotVoting` 错误码当前未被任何路径使用，属于旧实现残留
   - `IssuanceWeightInfo` 配置项当前未被代码使用，属于死接线
   - 模块源码 `rustfmt --check` 未完全通过，只有一处可自动格式化的小差异
   - `memory/scripts/load-context.sh` 未登记 `resolution-issuance-gov`

5. 测试覆盖还有缺口：
   - 当前测试没有直接约束 “Root 只能新增合法省储会地址、不能删除既有地址”
   - 当前测试没有直接覆盖技术文档乱码/路径漂移这类文档质量问题

### 说明

- 当前模块代码主逻辑是可用的，最大的实质问题不是权限绕过，而是“收款名单治理约束没有严格落地”，再加上 `weights` 和技术文档质量明显落后于实现
