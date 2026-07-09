# 官网手续费卡链上/链下标签 + Substrate 文案调整

## 任务需求（均在 /Users/rhett/GMB）
1. 交易费用分配机制 4 卡左上角加黄色小字：全节点/国储会/安全基金=链上交易，清算行=链下交易。
2. Technology 标题「基于 Substrate 的主权区块链」→「基于 PoW共识 的主权区块链」。
3. 删页脚右下「基于 Substrate 构建的主权区块链」。

## 实现
- Tokenomics.tsx：feeDistribution 加 tx 字段；卡片顶部加 `text-left text-xs text-gold-400` 小字（安全基金也是链上 8:1:1 一员故标链上交易）。
- Technology.tsx L116 标题改；框架标签「Substrate / Polkadot SDK」(L6,事实)保留未动。
- Footer.tsx：删底部第二个 `<p>`（Substrate 行），版权行保留。

## 验证（2026-07-08）
- build/lint 通过；浏览器(main 5205) DOM 核对：4 卡 tx 标签(链上/链下,gold rgb229,176,32,左对齐)、Technology 新标题、页脚无 Substrate、版权保留；控制台无 error。

## 结论
- 3 项完成并验证。
