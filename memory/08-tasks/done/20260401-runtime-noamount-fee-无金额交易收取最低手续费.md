# 付费调用交易改造

## 状态：done

## 任务目标

将所有用户/管理员主动发起的非资金交易（提案、投票、绑定等）统一改为收取 1 元/次的付费调用费。同时将交易分为 5 类：链上资金交易、链下资金交易、付费调用交易、免费调用交易、拒绝交易。

## 实现方案

在 `citizenchain/runtime/src/configs/mod.rs` 的 `OnchainTxAmountExtractor` 中，把付费调用的虚拟金额设为 `Amount(100000)`（虚拟金额 1000 元 = 100000 分，100000 × 0.1% = 1 元）。

## 付费调用交易（1 元/次，共 37 笔）

- DuoqianManage：vote_create、vote_close、propose_create_personal、cleanup_rejected_proposal、兜底
- DuoqianTransfer：propose_transfer、vote_transfer、propose_safety_fund_transfer、vote_safety_fund_transfer、propose_sweep_to_main、vote_sweep_to_main
- VotingEngine：joint_vote、citizen_vote
- SfidSystem：bind_sfid、unbind_sfid、rotate_sfid_keys
- AdminsChange：propose_admin_replacement、vote_admin_replacement
- ResolutionIssuanceGov：propose_resolution_issuance
- RuntimeUpgrade：propose_runtime_upgrade、developer_direct_upgrade
- ResolutionDestro：propose_destroy、vote_destroy
- GrandpaKeyChange：propose_replace_grandpa_key、vote_replace_grandpa_key
- FullnodeIssuance：bind_reward_wallet、rebind_reward_wallet
- OffchainTransaction 兜底：bind_clearing_institution、费率提案/投票、运维操作等

## 免费调用交易

- System、Timestamp（系统内部）
- ShengBankInterest（Root 运维）
- ResolutionIssuanceIss（治理执行 + 运维）
- ResolutionIssuanceGov：finalize_joint_vote、set_allowed_recipients
- VotingEngine：create_internal_proposal、create_joint_proposal、internal_vote、finalize_proposal
- RuntimeUpgrade：finalize_joint_vote
- DuoqianManage：register_sfid_institution
- DuoqianTransfer：execute_transfer
- AdminsChange：execute_admin_replacement
- ResolutionDestro：execute_destroy
- GrandpaKeyChange：execute_replace_grandpa_key、cancel_failed_replace_grandpa_key
- CitizenIssuance（无 extrinsic）

## 涉及文件

- `citizenchain/runtime/src/configs/mod.rs` — OnchainTxAmountExtractor 分类逻辑

## 验收标准

- [x] 付费调用交易扣除 1 元
- [x] 免费调用交易不受影响
- [x] 手续费正确分配（80% 矿工 + 10% 国储会手续费账户 + 10% 安全基金）
- [x] execute_* 类执行操作保持免费
- [x] 系统内部调用保持免费
