# 任务卡:卡③全部N+1循环逐条改批量(余额/storage/投票)

属 ADR-018(memory/04-decisions/ADR-018-citizenapp-unified-query-low-load.md)。

卡③全部N+1循环逐条改批量(余额/storage/投票)

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] 旧代码/文档/注释清理无残留

## 进度(2026-06-12)
- [x] institution_detail_page `_loadExtraAccounts`:更多账户余额循环逐条 `fetchFinalizedBalance` → 收集未命中地址一次 `fetchFinalizedBalances` 批量(analyze0/test196)。
- [ ] vote_view `_loadItemsForIds`:每提案调一次 hasUnvotedWallet → 需新增跨提案批量投票查询接口(InternalVoteQueryService.fetchAdminVotesForProposals)。
- [ ] institution_manage_service / personal_manage_service:账户详情多次 fetchStorage → fetchStorageBatch(与卡②listCidAccounts 整表化一并)。
- [ ] chain_tx_monitor:每事件/钱包余额 → 同块去重后批量(N 小,低优先)。

## 附带修复:广场重复卡片(2026-06-13)
现象:每次进广场,同一提案显示两张卡(一张带红点 needsVote=true,一张无)。
根因:vote_view newBlock 刷新 `_items = [...newItems, ..._items]`(:134)prepend 无去重;本地缓存 `_items` 与 `_allIds` 索引口径不同步(卡①把 _fetchAllGovernanceIds 改 org 过滤后,与旧本地索引口径不一致),某提案不在 _allIds 却在 _items → 被当新提案重查 prepend → 同一 id 两份(fresh + stale)。
修复:新增 `_dedupById`(按 proposalId 保留首次/ fresh 优先),prepend(:134)与翻页 append(:261)各过一遍。analyze0 / test196。待真机确认广场每提案单卡。
