# wuminapp duoqian-transfer 技术说明

## 模块边界

`wuminapp/lib/transaction/duoqian-transfer/` 是 wuminapp 端多签转账唯一实现目录。

- `duoqian_transfer_page.dart`：创建多签转账提案。
- `duoqian_transfer_detail_page.dart`：展示多签转账详情、投票状态和投票操作。
- `duoqian_transfer_service.dart`：构造 `DuoqianTransfer::propose_*` call data，读取转账提案详情。
- `duoqian_transfer_models.dart`：多签转账、 安全基金转账、手续费划转提案模型。
- `duoqian_transfer_balance_guard.dart`：检查管理员钱包是否足以支付提案/投票交易费。
- `duoqian_transfer_entry.dart`：多签转账入口卡片和页面跳转。
- `duoqian_transfer_proposal_adapter.dart`：给机构页、投票页和账户页使用的列表展示、详情跳转、缓存清理和数据源适配。

提案通用元数据、上下文、缓存放在 `wuminapp/lib/governance/shared/proposal/`；投票引擎共享能力放在 `wuminapp/lib/votingengine/internal-vote/`。多签转账业务详情通过 `ProposalWithDetail.businessDetails` 的不透明键值挂载，键名由本模块定义。

`wuminapp/lib/governance/organization-manage/` 不再实现多签转账按钮和跳转；多签账户详情页只允许挂载 `duoqian-transfer` 提供的 `DuoqianTransferEntryCard`，入口自身逻辑在 `duoqian-transfer` 内部。

`wuminapp/lib/governance/` 和 `wuminapp/lib/citizen/vote/` 只允许调用 `DuoqianTransferProposalAdapter` / `DuoqianTransferProposalFeed`，不得直接判断 `TransferProposalInfo`、`SafetyFundProposalInfo`、`SweepProposalInfo`，也不得直接打开 `DuoqianTransferDetailPage`。

## 费用规则

- 发起多签转账提案：管理员钱包按转账金额计费。
- 多签转账投票：投票管理员钱包按 `VOTE_FLAT_FEE = 1 元` 计费。
- 多签资金账户：执行阶段需要满足转账金额、手续费和 ED 保留要求。

管理员钱包余额不足时，页面直接提示“管理员钱包余额不足”，不再让用户误以为“投票成功但一直转圈”。

## 投票进度

详情页优先读取：

- `VotingEngine::AdminSnapshot`
- `InternalVote::InternalThresholdSnapshot`
- `InternalVote::InternalTallies`
- `InternalVote::InternalVotesByAccount`

进度条使用提案创建时的阈值快照，避免管理员变更后旧提案进度显示错误。

## 交易状态

发起多签转账提案的成功判定：

- `author_submitExtrinsic` / `txHash` 返回不代表提案创建成功。
- wuminapp 必须等待交易进入区块，并读取该区块 `System.Events`。
- 只有同一区块存在匹配本次发起人、机构主账户、收款人和金额的 `DuoqianTransfer::TransferProposed` 事件，才允许提示“提案创建成功”并写入本地个人多签提案历史。
- 如果交易已入块但没有 `TransferProposed`，视为提案创建失败，不写本地历史。

投票提交后监听交易池状态：

- 投票成功真源是 `InternalVote::InternalVotesByAccount(proposal_id, admin)`，不是 txHash、交易池 watch 或本地 nonce。
- `timeout / finalityTimeout / retracted / future / error`：保留本地 pending，并提示用户刷新后以链上投票记录为准。
- `invalid / dropped / usurped`：先复核链上投票记录；如果仍没有投票记录，清除本地 pending，并提示交易未出块原因。
- `inBlock / finalized`：保留正常确认路径，等待链上投票记录刷新。
- nonce 已被消耗但 `InternalVotesByAccount` 仍无记录时，视为本次投票未被 runtime 接受，清除 pending 后允许重新提交。
- runtime 无投票记录、nonce 未推进且 pending 超过 20 分钟时，视为本地提交未进入链，清除 pending 后允许重新提交。
- 提交拿到 txHash 后，按钮 `submitting` 必须立即结束，链上确认走后台 `_load(showSpinner: false)`，不得等待整页详情重新加载。
