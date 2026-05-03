# 任务卡：检查 voting-engine benchmark 权重和 admins-change duoqian 双入口风险

- 任务编号：20260502-191731
- 状态：done
- 所属模块：citizenchain/runtime/governance
- 当前负责人：Codex
- 创建时间：2026-05-02 19:17:31

## 任务需求

只读检查 voting-engine benchmark 签名脱节、internal_vote 免费权重 DoS 风险、admins-change 与 duoqian-manage 双治理入口风险是否存在，并给出修复建议。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 只读检查结论
- 修复建议
- 验证命令记录

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已核验 `voting-engine/src/benchmarks.rs` 与 `citizen_vote.rs` 签名不一致，`cargo check -p voting-engine --features runtime-benchmarks` 在 `citizenchain/` 子工作区确认触发 E0061。
- 已核验 `joint_vote` benchmark 直接写入 `Proposals`，未走 `do_create_joint_proposal` 的真实提案创建、互斥锁和全量快照路径。
- 已核验 `internal_vote` 权重仍为占位值，runtime 金额提取策略将其标记为 `NoAmount`；终态分支会进入统一内部投票回调并遍历 5 个业务 executor。
- 已核验 `admins-change::propose_admin_replacement` 未限制 `org`，可覆盖已激活的 `ORG_DUOQIAN` 管理员主体；`duoqian-manage` 同时负责该类主体的创建、激活、关闭生命周期，存在治理边界泄漏。

## 检查结论

1. benchmark 与真实函数签名脱节：存在，且会导致 `runtime-benchmarks` 编译失败。
2. `internal_vote` 免费策略叠加占位权重：存在，属于高风险权重/费用不匹配问题；DoS 影响需要结合区块 weight、交易池和提案构造成本做进一步压测，但风险链路成立。
3. `propose_admin_replacement` 未排除 `ORG_DUOQIAN`：存在；“双入口”更准确地说是 `admins-change` 对 `ORG_DUOQIAN` 管理员替换开放了通用入口，而 `duoqian-manage` 负责这类主体生命周期，模块边界不干净。

## 建议修复

- `voting-engine` benchmark：补齐 `province` 与 `signer_admin_pubkey` 参数；把 `joint_vote` benchmark 改成通过真实创建提案路径准备状态，或抽出共享 benchmark fixture，避免直接伪造与生产语义不一致的 `Proposal`。
- `internal_vote` 权重和费用：先取消免费或增加可验证的限流/深度约束，再用 benchmark 重新生成包含终态回调分支的权重；最终权重文件不得保留“占位”注释。
- `admins-change` 边界：在 `propose_admin_replacement` 入口增加 `org` 白名单，仅允许 `ORG_NRC | ORG_PRC | ORG_PRB`；补充 `ORG_DUOQIAN` 调用被拒绝的单测，并同步更新管理员治理文档。
