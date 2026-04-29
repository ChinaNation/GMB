# 任务卡：全面仔细检查 duoqian-transfer 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

- 任务编号：20260328-130310
- 状态：open
- 所属模块：citizenchain/runtime/transaction/duoqian-transfer
- 当前负责人：Codex
- 创建时间：2026-03-28 13:03:10

## 任务需求

全面仔细检查 duoqian-transfer 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md

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
- 已审查代码文件：
  - citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs
  - citizenchain/runtime/transaction/duoqian-transfer/src/weights.rs
  - citizenchain/runtime/transaction/duoqian-transfer/src/benchmarks.rs
- 已审查关联模块与文档：
  - citizenchain/runtime/src/configs/mod.rs
  - citizenchain/runtime/transaction/duoqian-manage/src/lib.rs
  - memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md
  - memory/scripts/load-context.sh

## 审查结论

- 未发现普通外部用户可直接利用的高危安全漏洞。
- 核心转账治理流程基本可用，但“功能需求严格实现”不完全成立。
- 主要问题集中在：weights 仍是占位估算、`ProposalData` 缺少模块标签防护、测试基座已失配、技术文档存在接口漂移。

## 发现问题

1. `weights.rs` 仍是占位实现，未以 benchmark 结果收口。
   - 证据：`src/weights.rs` 文件头直接写明“占位 weights，后续由 benchmark 生成替换”，且 `execute_transfer` 明确标注为参考其他模块“估算”。

2. 提案数据直接以 `TransferAction.encode()` 裸写入 `ProposalData`，读取时也直接 `decode`，没有像其他治理模块那样增加 `MODULE_TAG` 前缀校验。
   - 证据：`src/lib.rs` 中 `store_proposal_data(proposal_id, data)` 直接写入编码后的 `TransferAction`；`vote_transfer` 与 `execute_transfer` 都直接对整段 `data` 做 `TransferAction::decode(&mut &data[..])`。

3. 模块单测当前无法通过编译，说明测试 mock 已和上游接口脱节。
   - 证据：`cargo test -p duoqian-transfer` 失败，报错包括：
     - `SfidInstitutionVerifier` 现在需要 3 个泛型参数，但测试只实现了 2 个。
     - `duoqian_manage::Config for Test` 缺少 `FeeRouter`、`MaxSfidNameLength`。

4. 技术文档的 `Config Trait` 描述仍是旧口径，和现代码不一致。
   - 证据：文档仍把 `Currency`、`InternalVoteEngine`、`ProtectedSourceChecker` 写成当前模块自身 `Config` 项；但现代码里的本模块 `Config` 只声明 `RuntimeEvent`、`MaxRemarkLen`、`FeeRouter`、`WeightInfo`，其余通过父 trait 继承。

5. `load-context.sh` 未登记 `duoqian-transfer`，属于上下文装载残留。

## 注释与残留

- 中文注释整体完整，主流程可读性较好。
- 未发现 `TODO`、调试打印或临时代码残留。
- 模块源码 `rustfmt --check` 通过。

## 验证记录

- `cargo check -p duoqian-transfer`：通过
- `cargo check -p duoqian-transfer --features runtime-benchmarks`：通过
- `cargo check -p citizenchain`：通过
- `cargo test -p duoqian-transfer`：失败，原因见“发现问题”第 3 条
- `rustfmt --check citizenchain/runtime/transaction/duoqian-transfer/src/{lib.rs,benchmarks.rs,weights.rs}`：通过
