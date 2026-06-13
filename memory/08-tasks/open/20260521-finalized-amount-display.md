# wuminapp 与区块链软件金额展示统一 finalized 口径

## 任务需求

除交易状态进度外，wuminapp 和区块链软件中对用户展示为“余额、金额、总额、收益”的链上金额统一使用 finalized 高度读取。交易状态仍保留 pending / inBlock / finalized 三段，inBlock 只作为交易进度反馈，不作为确定金额展示口径。

## 影响范围

- `wuminapp/rust/`：补 finalized storage 查询能力。
- `wuminapp/smoldot-dart/`：暴露 finalized storage / account 读取 binding。
- `wuminapp/lib/rpc/`：新增 finalized 余额 API，保留 best API 供交易进度与诊断使用。
- `wuminapp/lib/wallet/`：钱包列表和钱包详情余额改为 finalized。
- `wuminapp/lib/governance/`：治理机构详情、更多账户、多签详情和相关余额提示改为 finalized。
- `wuminapp/lib/transaction/`：转账页、多签转账、安全基金、手续费划转的金额展示改为 finalized，交易状态三段不改。
- `citizenchain/node/src/`：区块链软件后端金额展示入口改为 finalized block hash 查询。
- `citizenchain/node/frontend/`：必要时同步金额文案与页面数据口径。
- `memory/`：更新技术文档，清理“金额展示按 best/latest”残留描述。

## 规则

- 确定金额展示 = finalized。
- 交易进度 = pending / inBlock / finalized。
- best/latest 仅用于交易进度、诊断、提交前快速预校验，不作为确定金额展示。
- 缓存金额必须标明 finalized 来源，不允许 best 金额写入同一个展示缓存。

## 执行记录

- 2026-05-21：创建任务卡，准备进入底层 finalized storage 能力改造。
- 2026-05-21：wuminapp 轻节点新增 finalized storage / System.Account 读取能力，`ChainRpc` 新增 finalized 余额、批量余额和 total 余额 API。
- 2026-05-21：wuminapp 钱包、治理机构、多签、链上转账页面的金额展示和余额提示切换为 finalized；交易状态仍保留 pending / inBlock / finalized。
- 2026-05-21：区块链软件钱包余额、发行总额、质押总额、挖矿收益、机构账户余额和提案金额详情切换为 finalized 读取。
- 2026-05-21：更新 wuminapp 与 CitizenChain node 技术文档，清理金额展示按 best/latest 的残留描述。

## 2026-06-12 升级备注(ADR-017)

本卡"确定金额=finalized"规则已由 ADR-017 扩展为**全端 finalized 单一口径**(余额/提案/交易记录/索引扫描/事件全量,豁免区仅交易提交管线),并配套链端"出块即固化"投票规则。本卡规则继续有效,作为 ADR-017 的子集。
