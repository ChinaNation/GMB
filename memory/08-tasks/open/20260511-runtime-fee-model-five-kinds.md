# runtime 五类费用模型修复

## 任务需求

- 修复 runtime 当前把 `VOTE_FLAT_FEE` 当金额再套链上费率的问题。
- 全 runtime 统一为五类费用模型：投票交易费、链上交易费、链下交易费、免费、未知拒绝。
- 完成后更新文档、完善注释、清理残留，并检查是否有交易不按五类模型执行。

## 建议模块

- `citizenchain/runtime/transaction/onchain-transaction`：费用类别与扣费适配器。
- `citizenchain/runtime/src`：RuntimeCall 到五类费用的归类。
- `citizenchain/runtime/transaction/duoqian-transfer`：执行阶段链上手续费核对。
- `memory/05-modules/citizenchain/runtime`：费用模型文档。

## 影响范围

- runtime 交易手续费预检、入池、扣费、事件记录。
- 投票/提案类交易从错误的 0.10 元恢复为 1 元。
- 链上资金交易继续按金额 0.1% 且最低 0.10 元。
- 链下清算手续费继续由清算结算逻辑处理，不混入链上 80/10/10 分账。

## 主要风险点

- 不能把执行阶段的真实链上手续费误改成 1 元。
- 不能让新增 RuntimeCall 通过兜底免费漏收费。
- 不能破坏现有脏工作区里的其他模块改动。

## 执行状态

- 状态：已完成

## 完成记录

- 已将 `onchain-transaction` 从旧的金额/无金额/未知三类模型改为单一 `FeeRoute` 协议。
- runtime `RuntimeCall` 的最终费用类型和付款方统一由 `FeeRoute::{Free, Onchain, Offchain, Vote, Reject}` 同时表达，不存在独立付款方真源。
- 已修复多签转账提案、个人/机构多签提案、决议提案等提案交易本身按 1 元收费，不再按提案金额套 0.1%。
- 已保留执行阶段真实链上手续费：多签转账、创建/关闭多签账户等实际资金移动仍按 `amount × 0.1%` 且最低 0.1 元扣取。
- 已更新 runtime 费用模型文档并清理旧金额提取器、金额结果类型与无金额分支残留。

## 验证记录

- `cargo test -p onchain-transaction --manifest-path citizenchain/Cargo.toml`：20 项通过。
- `cargo test -p citizenchain runtime_fee_kind_classifier --manifest-path citizenchain/Cargo.toml`：2 项通过。
- 旧三类金额提取模型在 runtime 与费用文档中的残留扫描为 0 命中。
- `cargo test -p citizenchain --manifest-path citizenchain/Cargo.toml`：36 项通过，1 项既有测试失败；失败项为 `resolution_destro_internal_vote_flow_executes_destroy_and_reduces_issuance`，错误是 `AlreadyVoted`，属于投票流程测试状态问题，不是本次费用分类路径。
