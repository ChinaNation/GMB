# 任务卡（草案·待确认）：链上税务·申报期结算（第2部分）

> 状态：**草案，待确认**。依据 ADR-038。第1部分订阅见 `20260716-citizen-coin-subscription.md`；税务**串行在订阅之后**落地，不阻塞订阅上线。

任务需求：创作者及盈利主体订阅收入的所得税，**链上申报期结算（非逐笔预扣）**。订阅款全额到账，收入按纳税主体 CID 记台账，税率/征收方式由税务机构后期运行期设定，征税权两级治理（立法院授权→税务机构设）。

所属模块：citizenchain（tax-registry pallet + primitives + legislation 回调）、onchina（domains/tax）、cloudflare/citizenapp（读链展示，链下）。

输入文档：**memory/01-architecture/gmb/membership-tax.md（工程架构:分层/目录/命名/闭合/核心边界）**、ADR-038、ADR-037、ADR-027/030/025；memory/07-ai/*。

必须遵守：
- 税**不逐笔预扣**；订阅款全额到账，税走后置申报期结算。
- 税率/征收方式/申报期/落账**税务机构运行期设定，架构不定值**（只建机制容器）。
- 纳税主体钉**稳定 CID**（不跑地址 union），CID 定管辖；能力位绑税务机构**唯一 CID**（非类别码）。
- 核心 vs 非核心：pallet 只放台账/授权/税率 storage + 结算资金动作 + 治理写入 + 调度；展示/申报编排链下。
- 链开发期重新创世无 migration/兼容/spec_version；签名分层（onchina 机构写冷签 internal-vote）；不清楚先沟通。

前置串行：a. 税务机构（财税部等）补管理员；b. 《税法》经 legislation-yuan 立法→回调写 `TaxAuthorization`（机器可读授权，或 ADR-039）；c. tax-registry pallet；d. subscription 接 `IncomeLedger`（第1部分空实现→此处接真实）。

输出物：
- **链**：`runtime/public/tax-registry/`（idx 36）
  - storage：`IncomeLedger:(taxpayer_cid,tax_period)->收入`、`TaxAuthorization:InstitutionCode->AuthEntry`、税率/征收规则可配置 storage。
  - `RuntimeTaxSubjectQuery`（钉稳定 CID 反查个人/机构类型，不跑地址 union）。
  - `propose_set_tax_rate`（税务机构 internal-vote 写规则；写前+执行+**征税时点**三重复核 `TaxAuthorization` 区间/有效期 + `caller_cid∩scope` 双向）。
  - `settlement.rs`（按 `tax_period` 到期桶有界结算征收，结算时点复核授权）。
  - `authorization.rs`（接 legislation-yuan 下游 push 写 `TaxAuthorization`）。
  - `primitives::tax_policy`（只放硬顶护栏）、`primitives::income_ledger`（`IncomeLedgerWriter` trait + `TaxPeriod` 单源，第1部分已建）。
  - `configs.rs`：`impl tax_registry::Config`、回调元组第6槽接 `tax_registry::Executor`、`legislation-yuan::Config` 加 `TaxAuthorizationWriter`、CallFilter `TaxRegistry(_)=>false`(仅治理/内部驱动)。
  - subscription `billing.rs` 的 `T::IncomeLedger=TaxRegistry`（收款成功记账，全额不预扣）。
- **onchina**：`domains/tax/`（`propose_set_tax_rate` chain_call+handler）+ `AdminActionType::SetTaxRate`（PasskeyColdSign 四 arm）+ `can_set_tax_rate`（唯一税务机构 CID）+ workspace + CitizenWallet 三处登记。
- **worker/app（链下）**：收入台账/申报状态读链展示。
- 中文注释、测试（结算/征税时点校验/纳税主体判定/授权失效）、文档、残留清理。

验收标准：
- 征税权 fail-closed：未授权不征、越界/过期回落 0 税；纳税主体钉 CID 不可伪造；结算原子且权重有界；能力位绑唯一 CID。
- 收款成功记 `IncomeLedger` 正确；重新创世一次成功。

税务政策拍板点（**税务机构后期运行期设定，不在本架构定值**）：税额落账层级、税率粒度、无税率放行 vs 禁收款、税基口径、机构类型变更生效口径。

影响范围：citizenchain（新 pallet + 立法/治理集成 + 重新创世）+ onchina + 链下展示。链改须用户显式确认后动手；依赖第1部分 subscription 就绪。
