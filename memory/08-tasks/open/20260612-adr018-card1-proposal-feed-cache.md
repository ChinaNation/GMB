# 任务卡:卡①ProposalFeedCache+提案三页统一(机构详情/广场/个人多签),收口本次bug+最大降载

属 ADR-018(memory/04-decisions/ADR-018-wuminapp-unified-query-low-load.md)。

卡①ProposalFeedCache+提案三页统一(机构详情/广场/个人多签),收口本次bug+最大降载

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] 旧代码/文档/注释清理无残留

## 完工记录(2026-06-12,代码完工 analyze0/test196,待真机验证)
- service `duoqian_transfer_service.dart`:新增 `fetchCurrentYearProposals()`(ProposalsByYear 短key取一次)+ `filterInstitutionVisible()` + `filterGovernanceIds()`(纯客户端过滤);`fetchInstitutionVisibleProposals` 重写为"按年取+过滤";删除 `fetchProposalIdsByOrg/ByInstitution/ByOwner`(长前缀/无用)。
- adapter `duoqian_transfer_proposal_adapter.dart`:新增进程内共享 `currentYearProposals()`(TTL20s)+ `fetchGovernanceProposalIds()`;机构详情走共享缓存;删除旧 per-sfid `_visibleProposalCache` 三件套。
- `vote_view.dart`:`_fetchAllGovernanceIds` 3次 ProposalsByOrg → 共享缓存 org 过滤。
- 效果:广场+机构详情同周期共用一份当前年提案,从 ≥4 次链查降到 1 次按年取+1 次批量详情。
- analyze 0;test 196/196;apk 已构建(23:36)。**待手机重连后装机验证机构详情显示提案**。
