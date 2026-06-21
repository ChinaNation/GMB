任务需求：
修复 citizenapp 投票提交后的等待状态：投票是否成功必须以 runtime 投票引擎链上记录为准，不能由 txHash、交易池 watch 或 nonce 超时直接判定。

所属模块：
citizenapp

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md
- memory/05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md
- memory/05-modules/citizenapp/transaction/duoqian-transfer/DUOQIAN_TRANSFER_APP_TECHNICAL.md

必须遵守：
- 投票成功真源是 runtime 投票引擎 storage：内部投票读 `InternalVote.InternalVotesByAccount`，联合投票读 `JointVote` 机构管理员投票记录。
- `author_submitExtrinsic` 返回 txHash 只代表已提交，不能代表投票成功。
- 交易池 watch 的 timeout/finalityTimeout/retracted/future 不能直接清除 pending。
- 只有链上已经写入投票记录，才把 pending 标记为 confirmed。
- nonce 已被消耗且链上仍没有投票记录时，才清除 pending 并允许用户重新提交。
- runtime 无投票记录、nonce 未推进且 pending 超过 20 分钟确认窗口时，必须清除 pending 并允许重新投票，不能无限显示“投票中”。
- 投票提交拿到 txHash 后，按钮必须立即停止转圈，链上确认走后台刷新。
- 改代码后必须补中文注释、更新文档并清理残留。

预计修改目录：
- citizenapp/lib/votingengine/internal-vote/：修复待确认投票存储的确认口径，属于 Flutter 投票引擎客户端代码。
- citizenapp/lib/rpc/：修正 nonce-only 交易状态注释，避免继续把 `checkTxStatus` 误读成 txHash 级成功确认，属于 Flutter 链 RPC 注释清理。
- citizenapp/lib/governance/：修复多签管理和 runtime 升级投票详情页的 pending / watch 状态处理，属于 Flutter 治理 UI 与服务调用代码。
- citizenapp/lib/transaction/duoqian-transfer/：修复多签转账提案投票详情页的 pending / watch 状态处理，属于 Flutter 交易治理代码。
- memory/01-architecture/citizenapp/：同步 citizenapp 总体投票状态真源规则，属于文档更新。
- memory/05-modules/citizenapp/：同步 citizenapp 治理和多签转账技术文档，属于文档更新。

输出物：
- runtime storage 优先的 pending 投票确认逻辑
- 多签管理投票 watch 失败兜底
- 多签转账投票 watch 失败兜底
- runtime 升级联合投票 pending 查询回调
- 中文注释
- 文档更新
- 残留搜索和静态检查结果

验收标准：
- 投票交易提交后，如果链上已写入管理员投票记录，页面必须显示已投票并清除 pending。
- 投票交易 watch 超时、最终化超时、回滚或 future 状态时，不得直接回到“未投票”，必须继续以链上投票记录为准。
- nonce 已被消耗但链上没有对应投票记录时，pending 可清除并允许用户重新提交。
- 内部投票和联合投票注册逻辑一致：都由 runtime 投票引擎记录决定投票状态。
- `dart analyze lib test` 通过或明确记录无法运行原因。
- `git diff --check` 通过。

新增命名说明：
- 中文名：待确认投票链上查询回调；English name：PendingVoteChainLookup；类型：Dart typedef；使用位置：`citizenapp/lib/votingengine/internal-vote/pending_vote_store.dart`；简介：允许不同投票类型传入对应的 runtime 投票记录查询逻辑。

执行记录：
- 已修复 `PendingVoteStore.confirmAllDetailed`：先查询 runtime 投票引擎 storage，只有链上存在管理员投票记录时才标记 confirmed。
- 已增加 `PendingVoteChainLookup`，允许 runtime 升级联合投票传入 `JointVote` 管理员投票查询，不再误读 `InternalVote`。
- 已修复多签管理详情页：交易池 watch 失败后先复核链上投票记录；非终态 watch 失败不再清除 pending。
- 已修复多签转账详情页：交易池 watch 失败后先复核 `InternalVotesByAccount`；非终态 watch 失败不再让管理员回到未投票。
- 已修复 runtime 升级详情页：pending 确认使用 `JointVote::JointVotesByAdmin` 作为真源。
- 已修正 `OnchainRpc.checkTxStatus` 注释：明确 confirmed 只代表 nonce 已推进，不代表投票类业务成功。
- 已同步 citizenapp 架构文档、治理技术文档和多签转账技术文档。
- 已补充 pending 20 分钟确认窗口：链上无投票记录、nonce 未推进且超时后清除 pending，避免管理员明细无限“投票中”。
- 已修复 runtime 升级联合投票 `AccountId` 编码：删除页面内裸 sfid `[u8;48]` 编码，统一走 `institutionIdentityToPalletId()`。
- 已修复多签转账和 runtime 升级投票提交后的按钮转圈：拿到 txHash 后后台刷新，不再 `await _load()`。

验证记录：
- `dart format citizenapp/lib/votingengine/internal-vote/pending_vote_store.dart citizenapp/lib/governance/duoqian_manage_detail_page.dart citizenapp/lib/transaction/duoqian-transfer/duoqian_transfer_detail_page.dart citizenapp/lib/governance/runtime-upgrade/runtime_upgrade_detail_page.dart citizenapp/lib/rpc/onchain.dart`：通过。
- `cd citizenapp && dart analyze lib test`：通过，No issues found。
- `cd citizenapp && flutter test`：通过，182 passed。
- `git diff --check`：通过。
- 残留搜索 `txFailureEvent / event.isFailure / PendingVoteStore.instance.remove / confirmAll / 投票成功 / 交易已出块 / 未出块，已清除`：未发现旧的“watch 失败直接恢复未投票”流程残留；保留的 remove 均在链上投票记录复核或 nonce 消耗后执行。
- 本次补修后复跑 `cd citizenapp && dart analyze lib test`：通过，No issues found。
- 本次补修后复跑 `cd citizenapp && flutter test`：通过，182 passed。
- 本次补修残留搜索 `await _load / _sfidNumberToFixed48 / _votePendingTimeout / institutionIdentityToPalletId`：投票提交路径已无 `await _load()`；runtime 升级页已无错误 `_sfidNumberToFixed48()`；pending 20 分钟确认窗口和统一 AccountId 编码已落地。
