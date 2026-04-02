# 无金额交易收取最低手续费

## 状态：open

## 任务目标

当前所有 NoAmount 类型的 extrinsic（提案、投票、绑定清算行等）不收手续费，存在被滥用刷交易池的风险。需要对有滥用风险的无金额调用收取最低手续费 0.1 元。

## 实现方案

在 `citizenchain/runtime/src/configs/mod.rs` 的 `PowTxAmountExtractor` 中，把需要收费的调用从 `NoAmount` 改为 `Amount(10000)`（虚拟金额 100 元 = 10000 分），利用现有费率公式 `max(金额 × 0.1%, 0.1元)` 自动算出最低手续费 0.1 元。

## 需要改为收费的调用

- VotingEngineSystem（治理提案、投票）
- AdminsOriginGov（管理员治理）
- RuntimeRootUpgrade（升级提案）
- ResolutionDestroGov（销毁治理）
- ResolutionIssuanceGov（发行治理）
- GrandpaKeyGov（GRANDPA 密钥治理）
- SfidCodeAuth（SFID 授权码）
- CitizenLightnodeIssuance（公民轻节点发行）
- DuoqianManagePow 的 vote_create、vote_close
- OffchainTransactionPos 的 bind_clearing_institution

## 保持免费的调用

- System、Timestamp（系统内部）
- FullnodePowReward（节点出块奖励）
- ShengBankStakeInterest（省储行质押利息）
- ResolutionIssuanceIss（发行执行，系统行为）
- DuoqianTransferPow（内部已有独立扣费机制）
- OffchainTransactionPos 的 submit/enqueue/process_offchain_batch（relay submitter 白名单限制）

## 涉及文件

- `citizenchain/runtime/src/configs/mod.rs` — PowTxAmountExtractor 分类逻辑

## 验收标准

- [ ] 需要收费的调用提交后扣除 0.1 元手续费
- [ ] 保持免费的调用不受影响
- [ ] 手续费正确分配（80% 矿工 + 10% 国储会 + 10% 销毁）
