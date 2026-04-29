# 任务卡：全面仔细的检查一遍 onchain-transaction 这个模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-123709
- 状态：done
- 所属模块：citizenchain/runtime/transaction
- 当前负责人：Codex
- 创建时间：2026-03-28 12:37:09

## 任务需求

全面仔细的检查一遍 onchain-transaction 这个模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/requirement-analysis-template.md
- memory/07-ai/thread-model.md

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
- 修改提案、投票、发行核心规则

## 模块级完成标准

- 审查安全边界、功能实现、中文注释、技术文档、残留清理
- 关键结论回写任务卡
- 跑完必要验证命令

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已检查 `citizenchain/runtime/transaction/onchain-transaction/src/lib.rs`
- 已检查 `citizenchain/runtime/src/configs/mod.rs`
- 已检查 `citizenchain/runtime/transaction/onchain-transaction/benches/transaction_fee_paths.rs`
- 已检查 `memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`
- 已检查 `citizenchain/node/src/rpc.rs`
- 已检查 `citizenchain/nodeui/backend/src/mining/mining-dashboard/mod.rs`
- 已完成 `cargo test -p onchain-transaction`
- 已完成 `cargo check -p citizenchain`
- 已完成 `cargo bench -p onchain-transaction --bench transaction_fee_paths --no-run`

## 审查结论

1. `transfer_all` 的收费口径偏大
   - `OnchainTxAmountExtractor` 对 `Balances::transfer_all` 直接按“当前可转全部余额”提取金额
   - 但交易手续费会先在 `withdraw_fee` 阶段扣掉，`transfer_all` 真正执行时只会转出扣费后的剩余可转余额
   - 结果是手续费按“不会真正转出的金额”计费，不符合“按交易金额收费”的口径
   - 位置：
     - `citizenchain/runtime/src/configs/mod.rs:315`
     - `citizenchain/runtime/transaction/onchain-transaction/src/lib.rs:134`

2. `FeePaid` 事件不包含 tip，但外部链路把它当成“真实手续费”
   - 事件里写入的是 `base_fee = fee_with_tip - tip`
   - 但 runtime 注释、node RPC 和 nodeui 注释都把 `FeePaid` 当成“真实手续费”来源
   - 只要出现非零 tip，`fee_blockFees` 和依赖它的展示都会少算
   - 位置：
     - `citizenchain/runtime/transaction/onchain-transaction/src/lib.rs:164`
     - `citizenchain/runtime/src/lib.rs:284`
     - `citizenchain/node/src/rpc.rs:333`
     - `citizenchain/nodeui/backend/src/mining/mining-dashboard/mod.rs:595`

3. 全节点手续费分成继承了 PoW 作者身份信任风险
   - `OnchainFeeRouter` 通过 `FindAuthor` + `RewardWalletByMiner` 把全节点份额打给作者绑定钱包
   - runtime 当前的 `PowDigestAuthor` 只是从 pre-runtime digest 解码 `AccountId`
   - 这意味着本模块的全节点手续费分成继承了 `fullnode-issuance` 同一条作者身份信任边界
   - 位置：
     - `citizenchain/runtime/transaction/onchain-transaction/src/lib.rs:265`
     - `citizenchain/runtime/src/configs/mod.rs:410`

4. 技术文档对事件口径不完整
   - 文档没有写 `FeePaid` 事件只记录基础费，不含 tip
   - 文档也没有写 node RPC / nodeui 对 `FeePaid` 的依赖关系，难以支撑后续审计和客户端联调
   - 位置：
     - `memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`

## 验证记录

- `cargo test -p onchain-transaction`
  - 结果：通过，16 个测试全部通过
- `cargo check -p citizenchain`
  - 结果：通过
- `cargo bench -p onchain-transaction --bench transaction_fee_paths --no-run`
  - 结果：通过，专项 benchmark 入口可编译
