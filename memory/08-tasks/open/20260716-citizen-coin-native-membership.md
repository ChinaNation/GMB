# 任务卡（草案·待确认）：公民币原生会员订阅（链上直付 + 原生订阅 pallet）

> 状态：**草案，待用户确认**。确认前不改链码、不改产品码。依据 ADR-037。

任务需求：把会员「加密货币预付」从 Stripe-Crypto(USDC) 迁到 GMB 自有链上代币【公民币】直付，CitizenWallet 直接扣公民币开通/续订，绕过 Stripe。银行卡仍留 Stripe。双轨「Stripe 管卡 + 公民币管原生」。

所属模块：citizenchain（新 membership pallet + primitives 单源）、citizenapp/cloudflare（会员后端）、citizenapp（App 会员页）、citizenweb（官网会员页）。跨产品，含重大链改。

输入文档：
- memory/04-decisions/ADR-037-citizen-coin-native-membership.md（本卡依据）
- memory/04-decisions/ADR-036 / ADR-034 / ADR-033 / ADR-011
- memory/07-ai/chat-protocol.md、agent-rules.md、task-card-template.md
- 记忆：feedback_signing_layer_selection_rule、feedback_unified_signing_optag_only、project_qr_signing_two_color、project_seed_biometric_binding_design、project_chain_fee_model_and_payment_diagnosis、feedback_chain_dev_never_ask_migration、feedback_in_development_zero_users、reference_worker_chain_rpc_secret_names

必须遵守：
- 不突破模块边界；不绕既有契约（`fee_policy` 单源、`account_derive` 派生、签名分层）。
- 链开发期：重新创世即可，**无 migration / 无兼容 / 无 spec_version 提问 / 无残留**。
- 签名走热钱包标准 extrinsic + 生物识别；禁 0x1D、禁冷钱包盲签。
- 定价直接公民币（`primitives::membership_price` 单源），不引预言机。
- 收入进指定运营机构账户（`MEMBERSHIP_REVENUE_ACCOUNT`），不进两和基金。
- 不清楚逻辑先沟通。

---

## 第 1 期（MVP）：公民币预付多期

输出物：
- **链**：`citizenchain/runtime/membership/` 新 pallet（新 `pallet_index`）：
  - 存储 `MembershipOf: AccountId -> {level, expires_at}`。
  - extrinsic `prepay(level, periods)`：校验 → 按 `membership_price[level] × periods` 转公民币入 `MEMBERSHIP_REVENUE_ACCOUNT` → 写/延 `MembershipOf`（`expires_at = max(now, 现expires) + periods 月`）→ 抛 `MembershipPaid` 事件。
  - `primitives::membership_price`（每档每月固定分，单源）+ `MEMBERSHIP_REVENUE_ACCOUNT`（运营机构 cid_number 派生账户，单源）。
  - 接 `fee_policy` 收标准链上交易费。
- **Worker（`citizenapp/cloudflare/src/membership/`）**：删 `prepaid.ts`；加 `citizen_coin.ts`（`state_getStorage` 读 `MembershipOf` 确认 → 镜像 D1 `subscription_source='citizen_coin'`，按 `tx_hash` 幂等去重）；`service.ts` `subscriptionIsActive` 加 `citizen_coin` 分支（仅看 `expires_at`）；`types.ts` source 联合改；D1 基线 `0001_square_core.sql` 重建（`usdc_prepaid`→`citizen_coin`）。
- **App（`membership_page.dart`）**：公民币轨——选档+期数 → 构 `prepay` extrinsic → 生物识别 → 热钱包签 → 提交（RPC/relay）→ 轮询 worker 同步 → 显有效 + 起止 + 到期提醒；删 USDC 入口。
- **官网（`Membership.tsx`）**：删加密预付卡；加「用 App 公民币订阅」QR/深链把手；卡档不变。
- 中文注释、测试（pallet 单测 + worker 测 + App 测）、文档更新、残留清理。

验收标准：
- 公民币轨全链跑通：签→付→落块→worker 确认→权益生效；`tx_hash` 重放不重复延时长。
- 卡轨回归不破；USDC 路线与 `usdc_prepaid` 残桩零残留。
- 手续费叠加正确、UX 明示；两色识别显绿名。
- 重新创世一次成功，无 migration 链。

## 第 2 期（旗舰）：托管计量自动续订 escrow-and-meter

输出物：
- pallet 扩：`deposit_and_subscribe(level, deposit)` 存押托管 + `on_initialize` 到期扫单自动扣一期/延窗口/转运营账户；`cancel` 停扣 + 提未计量余额；升/降档改档期费率 + 结算当期。
- 调度扫单 O(有界)/块（`BoundedVec` 到期集或滚动游标），防区块权重 DoS。
- Worker/App：托管余额展示、续订状态、取消提余额。

验收标准：
- 托管有钱自动续、无需逐期签名；托管耗尽自然停。
- 取消退未计量余额准确；已计量期不可退。
- 扫单权重有界，压测无 DoS。

---

影响范围说明：跨 4 产品；含重大链改（新 pallet + runtime + 重新创世）。链改须用户显式确认后方可动手。
