# 任务卡：分析 duoqian-transfer-pow 与 duoqian-transaction-pow 并记录 Pending 残留风险

- 任务编号：20260322-141500
- 状态：open
- 所属模块：citizenchain,runtime,transaction,memory
- 当前负责人：Codex
- 创建时间：2026-03-22 14:15:00

## 任务需求

基于源码全面分析 `duoqian-transfer-pow` 与 `duoqian-transaction-pow` 的实际功能实现，澄清它们与内部投票引擎的关系，并把 `propose_create` 预写 `Pending` 状态但缺少拒绝/超时清理路径的问题固定到任务卡和模块文档中。

## 必读上下文

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/05-modules/citizenchain/runtime/transaction/duoqian-transaction-pow/DUOQIAN_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer-pow/DUOQIAN_TRANSFER_TECHNICAL.md`
- `citizenchain/runtime/transaction/duoqian-transaction-pow/src/lib.rs`
- `citizenchain/runtime/transaction/duoqian-transfer-pow/src/lib.rs`
- `citizenchain/runtime/governance/voting-engine-system/src/internal_vote.rs`
- `citizenchain/runtime/src/configs/mod.rs`

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 源码级分析结论
- 模块文档更新
- 风险写入任务卡

## 待确认问题

- 新补充的内置机构 `ZF / LF / JC / JY / SF` 后续是全部接入 `duoqian-transfer-pow`，还是只接其中一部分。

## 实施记录

- 已确认：`duoqian-transfer-pow` 当前只支持内置治理机构 `NRC / PRC / PRB` 转账，不支持注册型 `ORG_DUOQIAN`，也尚未接入 `ZF / LF / JC / JY / SF`。
- 已确认：`duoqian-transfer-pow` 的转账审批实际就是复用 `voting-engine-system` 的内部投票能力；只要机构类型被本模块识别，并接入 runtime 管理员/阈值提供器，就可以走内部投票转账，不需要新增转账模块。
- 已确认：`duoqian-transaction-pow` 当前真实职责是 `sfid` 机构登记、注册型多签机构创建、注册型多签机构关闭；创建/关闭当前都走 `ORG_DUOQIAN` 内部投票，不是旧文档里的离线 M-of-N 一次性签名提交。
- 已记录关键问题：在 `propose_create`（`citizenchain/runtime/transaction/duoqian-transaction-pow/src/lib.rs`，约第 490 行）里，模块会先把 `DuoqianAccounts` 写成 `Pending`，然后才创建内部投票提案。
- 已进一步确认：当前模块中可以看到 `execute_create` 把 `Pending` 改成 `Active`，`execute_close` 删除 `Active` 账户，但未看到“创建提案被拒绝 / 超时后清理 Pending 记录”的路径。
- 风险结论：若创建提案未通过，链上可能残留一条 `Pending` 的 `DuoqianAccounts`，后续再次对同一 `sfid_id / duoqian_address` 发起创建时，可能命中 `AddressAlreadyExists`。
- 已把上述口径同步回模块技术文档，避免继续沿用旧版离线签名描述。
