# CitizenApp multisig-transfer 技术说明

## 模块边界

`citizenapp/lib/transaction/multisig-transfer/` 是 CitizenApp 端多签转账唯一实现目录。

- `multisig_transfer_page.dart`：创建多签转账提案。
- `multisig_transfer_detail_page.dart`：展示多签转账详情、投票状态和投票操作。
- `multisig_transfer_service.dart`：构造 `MultisigTransfer::propose_*` call data，读取转账提案详情。
- `multisig_transfer_models.dart`：多签转账、 安全基金转账、手续费划转提案模型。
- `multisig_transfer_balance_guard.dart`：检查管理员钱包是否足以支付提案/投票交易费。
- `multisig_transfer_entry.dart`：多签转账入口卡片和页面跳转。
- `multisig_transfer_proposal_adapter.dart`：给机构页、投票页和账户页使用的列表展示、详情跳转、缓存清理和数据源适配。

提案通用元数据、上下文、缓存放在 `citizenapp/lib/governance/shared/proposal/`；投票引擎共享能力放在 `citizenapp/lib/votingengine/internal-vote/`。多签转账业务详情通过 `ProposalWithDetail.businessDetails` 的不透明键值挂载，键名由本模块定义。

`citizenapp/lib/governance/organization-manage/` 不再实现多签转账按钮和跳转；多签账户详情页只允许挂载 `multisig-transfer` 提供的 `MultisigTransferEntryCard`，入口自身逻辑在 `multisig-transfer` 内部。

`citizenapp/lib/governance/` 和 `citizenapp/lib/citizen/vote/` 只允许调用 `MultisigTransferProposalAdapter` / `MultisigTransferProposalFeed`，不得直接判断 `TransferProposalInfo`、`SafetyFundProposalInfo`、`SweepProposalInfo`，也不得直接打开 `MultisigTransferDetailPage`。

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
- CitizenApp 必须等待交易进入区块，并读取该区块 `System.Events`。
- 只有同一区块存在匹配本次发起人、机构主账户、收款人和金额的 `MultisigTransfer::TransferProposed` 事件，才允许提示“提案创建成功”并写入本地个人多签提案历史。
- 如果交易已入块但没有 `TransferProposed`，视为提案创建失败，不写本地历史。

三类提案统一标准（2026-06-09 静默失败整改）：

- 安全基金转账（`propose_safety_fund_transfer`）与手续费划转（`propose_sweep_to_main`）与普通转账提案完全同标准：`_signAndSubmitInBlock` 等真正入块 → 读 `System.Events` 核对 `SafetyFundTransferProposed` / `SweepToMainProposed` 事件 → 返回事件中的 `proposalId`。
- 事件核对共用 `_confirmProposalEvent` + `_findProposalIdInEvents` 单一扫描骨架，事件字段解码各自实现（字段顺序必须与 runtime Event enum 严格一致；事件变体序号按声明顺序：TransferProposed=0、SafetyFundTransferProposed=3、SweepToMainProposed=6）。
- submit-only 的 `_signAndSubmit` 已从本服务删除：提案类业务成功必须拿到 proposalId，submit-only 在原理上给不出。普通钱包余额转账仍走 builder 层 `signAndSubmit`（submit-only + 20 分钟后台 watch），两档标准见 `signed_extrinsic_builder.dart` 注释。
- 错误处理铁律：提交/解码/查询失败一律留痕（debugPrint）或上抛，禁止裸 `catch (_) {}` 吞错；链上余额刷新失败而展示缓存时，UI 必须标注“可能已过期”。

投票提交和确认：

- 投票成功真源是 `InternalVote::InternalVotesByAccount(proposal_id, admin)`，不是 txHash、交易池 watch 或本地 nonce。
- `InternalVoteService.submit()` 必须等待交易 `inBlock / finalized`，随后回读 `InternalVote::InternalVotesByAccount`，确认该管理员投票已经写入 runtime。
- CitizenApp 不缓存、不预占、不回滚交易 nonce；每次签名前实时读取 runtime `frame_system::Account.nonce`。
- 新成功流程不再写本地 pending；确认成功后只清理旧残留 pending，并立即把该管理员显示为已投票。
- `timeout / finalityTimeout / retracted / future / error`：保留本地 pending，并提示用户刷新后以链上投票记录为准。
- `invalid / dropped / usurped`：先复核链上投票记录；如果仍没有投票记录，清除本地 pending，并提示交易未出块原因。
- `inBlock / finalized` 只代表交易进块；仍必须以 `InternalVotesByAccount` 回读结果为准。
- runtime 无投票记录且 pending 超过 20 分钟时，视为本地提交没有形成有效投票，清除 pending 后允许重新提交，避免管理员明细无限显示“投票中”。
- 服务层完成入块和 runtime 投票记录确认后，按钮 `submitting` 结束；详情页 `_load(showSpinner: false)` 只负责后台同步最新展示状态。
