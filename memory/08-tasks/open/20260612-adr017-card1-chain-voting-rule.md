# 任务卡:ADR-017 卡1 链端出块即固化(投票规则无约束)

## 方案
`node/src/core/service.rs:723` voting_rule 从 `VotingRulesBuilder::default().build()`(BeforeBestBlockBy(2)+3/4,最多投到 best−2)改为 `()`(官方无约束实现,允许投到链尾)。不动 runtime/创世;节点二进制重建后滚动重启生效(国储会权威节点优先)。

## 验收
- [ ] cargo check + node 测试通过
- [ ] 部署后实测:发一笔交易数秒内 finalized==best;静默期尾块被固化(user 部署验证)

## 完工记录(2026-06-12,代码完工,待部署)

- `node/src/core/service.rs:723` voting_rule → `()`(官方无约束实现),带 ADR-017 中文注释。
- cargo check -p node 通过(0 warning);cargo test -p node 153 passed / 1 failed(唯一失败 compact_u128_big_integer 为既有测试取值错误,与本卡无关,已另立卡)。
- **部署待 user**:重建节点二进制 → 全网滚动重启(国储会权威节点优先)→ 实测发一笔交易数秒内 finalized==best。
