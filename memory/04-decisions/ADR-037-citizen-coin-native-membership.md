# ADR-037 公民币原生会员订阅：按月自动扣 + 双边订阅市场（接 ADR-036 / 取代 ADR-034 加密路线 / 税务见 ADR-038）

- 状态：**Accepted（2026-07-17 定稿）**
- 决议日期：2026-07-16 草案 → 2026-07-17 定稿
- **定稿修订（2026-07-17，覆盖草案「只留卡」口径）**：**Stripe 美元支付全面下线，银行卡轨一并清除**。平台会员与创作者会员**统一走公民币链上订阅，无第二支付轨**。原草案「银行卡仍走 Stripe（只留卡）」作废；USD 定价、Stripe checkout/webhook、USDC 预付、官网会员页全部删除，价格唯一真源＝链上 `PlatformPrice`。下方正文凡提「只留卡 / 卡轨 / USD 价 / 官网卡档」均按本修订理解为**已删除**。
- 关联：ADR-036（会员身份解耦三档）、ADR-034（USDC 预付——本决策**取代其加密路线**）、ADR-033（生命周期）、ADR-011、**ADR-038（创作者收入所得税，本 ADR 税务侧）**、ADR-030（onchina 控制台）、ADR-028/ADR-020（广场/聊天门禁）、`primitives::fee_policy`/`account_derive`/`cid`。

## 标题

会员订阅直接用公民币【原生 GMB Balances】**按月自动扣**，钱包账户即唯一身份。落**双边订阅市场**：① 用户向"技术公司"订阅（平台会员）② 任意用户开自己会员体系被别人订阅赚公民币（创作者会员，扣所得税后归创作者，税务见 ADR-038）。**支付轨唯一＝公民币，Stripe（含银行卡）与 USDC 预付全面下线**（2026-07-17 定稿修订）。

## 背景

- ADR-034 的 USDC 路线依赖 Stripe 加密能力；LIVE 账户被判 `crypto_payments=inactive`，生产不可用。用户定调：平台+创作者订阅统一用公民币按月扣；**Stripe（含银行卡）全面下线**（2026-07-17 定稿）。
- 独特优势：GMB 自有 runtime，可做外链做不到的原生订阅 pallet 按月自动扣。
- 事实核对：公民币＝原生 GMB（2 位精度元/分、ED=111 分）；链上暂无订阅 pallet；worker 已能链读 + 交易广播兜底；现"订阅签名"`op_tag 0x1D` 是链下 BFF 授权（只留给卡轨），公民币要真金上链不复用 0x1D。

## 决策

**1. 两种订阅、一套机制（issuer 参数化，键钉稳定 CID）**
- 一个 `subscription` pallet：`Subscription: StorageMap<(subscriber, IssuerKey), SubscriptionState>`。
- `IssuerKey = Platform | Creator(cid)`：平台轨 issuer 固定为技术公司账户；创作者轨 issuer **钉稳定 CID**（并入审查 H4/M8——不用 `cid_or_account` 联合，个人↔机构转变表现为"同一 CID 属性变化"而非换键，防存量订阅成孤儿）。
- 平台轨 vs 创作者轨共用同一状态机与按月扣款循环，差异只在：issuer 是谁、价从哪张表取、扣款是否过税。

**2. 按月自动扣（订阅授权 + 有界扫单 + 原子 + 欠费即停）**
- 用户签一次 `subscribe(issuer, plan)` ＝开一条限额授权；`cancel(issuer)` 关闭。
- **有界扫单（并入审查 M5）**：建时间桶到期索引 `DueQueue: StorageMap<due_bucket, BoundedVec<SubKey, MaxPerBucket>>`；`on_initialize` 只弹当前桶、**O(有界且与总订阅量无关)**；桶满溢出策略显式（拒新订阅入桶或均摊相邻桶），**禁遍历全表**。
- **原子（并入审查 H2）**：每条订阅"扣款 → 全额转收款方 → 记收入台账 → 顺延 `next_charge_at`"整体包 `with_storage_layer`，任一步失败整条回滚不推进。**全额到收款方**（平台→技术公司账户；创作者→创作者钱包），**订阅侧不预扣税、不分账**；收款成功时向 ADR-038 `IncomeLedger` 按收款方 CID 记账（记账不扣钱）。**扣额 < ED 或不可收款**：拒扣本期 → **欠费即停**。
- **欠费即停**：一次扣款失败 → `PastDue` 立即停权，不重试、不宽限；用户重新 `subscribe` 才回 `Active`。
- 自动续费走 `fee_policy::Free`（系统触发无协议交易费）；用户主动 `subscribe/cancel/set_creator_plans` 收标准链上费。

**3. 定价：直接公民币、平台价链上可写 storage（对 ADR-037 上一版的显式修订）**
- **平台三档公民币月价＝链上可写 `PlatformPrice: StorageMap<MembershipLevel, u128 /*分*/>`**，默认 `freedom=199900 / democracy=599900 / spark=5999900` 分（＝1999/5999/59999 元）。
- 修订理由：需求 3"技术公司在 onchina 控制台自助调价"——常量改动要重新创世，与自助调价冲突（探源 gaps）。故价格**下沉链上 storage**，由技术公司经 `internal-vote` 写入（见 7）。`primitives::membership_price` 只留 `MembershipLevel` 枚举 + 单位 + 硬上下限护栏（是否设护栏＝拍板点），**不放可调数值**。
- 创作者档：`CreatorPlans: StorageMap<Creator(cid), BoundedVec<CreatorTier{price_fen, tier_code}>>`，价格/权益由创作者自设，`set_creator_plans` caller 须解析为**已闭合 CID 纳税主体**（ADR-038 第 2 节）否则拒当创作者。
- 不引预言机；无 USD 价、无第二支付轨；两种订阅均不跨折算。
- **价唯一真源（定稿修订）**：`MembershipLevel` 档位集合以 `primitives` 枚举为唯一形状源；**价格唯一真源＝链上 `PlatformPrice`（分）**，worker `plans.ts` 只存配额（发帖/媒体/聊天上限）不存价，App 直接链读 `PlatformPrice` 展示。原「卡轨 USD 价 vs 币轨分价双源一致性断言」随卡轨删除而作废。

**4. 资金去向：全额到收款方，税走后置申报期结算（ADR-038）**
- ① 平台会员：全额进 `PLATFORM_MEMBERSHIP_ACCOUNT` ＝「中国公民链技术有限公司」私权法人机构 `OP_MAIN`（**公司后期注册，地址注册后再填单源常量**；填入前平台公民币轨挂起）。
- ② 创作者会员：**全额进创作者钱包**。
- 税**不逐笔预扣**：所有收款方（技术公司/创作者/任何盈利主体）收全额毛收入，税是其作为纳税主体的**后置周期义务**——收款成功向 ADR-038 `IncomeLedger` 按 CID 记账，由其管辖税务机构在**申报期结算征收**（税率/征收方式税务机构运行期设定）。订阅侧不碰税。
- 均不进两和基金（OP_HE 专用）。

**5. 权益（用量）体系分层：链上订阅关系为唯一真源**
- ① 平台档用量（ADR-036 发帖/媒体/聊天文件上限）：资格＝链上 `Subscription[(user, Platform)]`（真源）；权益**值**＝Worker `plans.ts`/`limits/catalog.ts`（与支付无关，链上不重复存值）；强制点保持现状三层。
- ② 创作者"解锁专属内容"：资格＝链上 `Subscription[(subscriber, Creator(cid))]`；BFF 新增 `requireCreatorSubscription`（与 `requireActiveMembership` 同构），挂广场私密帖/聊天群加入/信封解密授权点；依赖 ADR-028/ADR-020，属净新建。**门禁链读失败一律 fail-closed 拒绝**（并入审查 L，防 smoldot 抖动泄漏）。

**6. 无退款契约（并入审查遗漏项）**
- 订阅设计为**欠费即停、无退款入口**（链上支付无退单，简洁）。税走后置申报期结算（ADR-038），与订阅退款无关。

**7. 签名 + 治理入口（onchina）**

| 动作 | 谁签 | 通道 |
|---|---|---|
| `subscribe`/`cancel`（平台/创作者，公民） | 订阅者本人 | **热钱包标准 extrinsic + 生物识别**（禁 op_tag 0x1D、禁冷钱包盲签） |
| `set_creator_plans`（个人创作者自设价） | 创作者本人 | 热钱包标准 extrinsic + 生物识别 |
| `set_creator_plans`（机构创作者） | 机构 admin | onchina `internal-vote` + 冷签 |
| 技术公司调平台价 `set_platform_price` | 技术公司 admin 集合 | onchina 冷签（PasskeyColdSign）+ `internal-vote`；能力位 `can_set_platform_price` **绑技术公司唯一 CID**（非类别码，并入审查 H1） |

- 前置：技术公司链上不存在，须先 `PrivateManage.propose_create_private_institution` 创建为私权法人 + 补管理员，方可登录 onchina 调价。入口挂 `domains/membership/chain_call.rs`（仿 `AddressRegistry.set_catalog_version` 范式）。
- 公民订阅热钱包通道与 onchina 机构写冷签是**两条独立签名通道**，不混用。

## 边界

- 公民币轨天生 App-only（官网无私钥）；官网会员页已整页删除，会员订阅只在 App 内完成。
- 公民币获取途径（转账/充值/交易）是前置依赖，不在本卡；无币者需先获取公民币再订阅（无卡轨兜底）。
- 创作者所得税＝ADR-038；本 ADR 只在扣款处调用 `TaxQuery`。

## 影响

- **链（重大，重新创世无 migration）**：新 `subscription` pallet（`PlatformPrice`/`CreatorPlans`/`Subscription`/`DueQueue` + `subscribe`/`cancel`/`set_creator_plans`/`set_platform_price` + `on_initialize` 桶扫，全额转收款方）；收款成功向 ADR-038 `IncomeLedger` 记账（订阅侧不预扣税）。
- **Worker（as-built 2026-07-17）**：删 `membership/{subscribe,webhook,prepaid,stripe_api}.ts` 四文件；加 `membership/citizen_coin.ts`（`platformSubscriptionConfirmRoute`：上链后镜像确认 + `last_tx_hash` 幂等，路由 `POST /v1/square/membership/confirm`）；`service.ts` `subscriptionIsActive` 收敛为按 `subscription_status='active'`（与创作者门禁同口径）；`types.ts` Env 删全部 Stripe 变量、`MembershipRow` 删 `subscription_source/stripe_*/cancel_at_period_end/prepaid_payment_ref`；`routes.ts`/`limits/catalog.ts`/`security/request_guard.ts` 删 subscribe/cancel/prepaid/webhook 路由与豁免；`account/service.ts` 删 Stripe 取消端点（订阅取消改走链上 extrinsic）；`account/purge.ts` 删注销退订（订阅与注销解耦）；`action_challenge.ts` `SignedAction` 删 `cancel_membership`/`subscribe_membership`；D1 `0001` 重建 `square_memberships`（新 schema）+ 删 `square_stripe_webhook_events`/`square_stripe_payments` 两表 + 删 `0002` 迁移。
- **App（as-built）**：`rpc/subscription_rpc.dart` 加平台档 `subscribePlatform`/`cancelPlatform`（SCALE 对齐金标向量 `subscription_scale_vectors.json`）；`my/membership/subscription_service.dart`（平台订阅编排：热签+生物识别+confirm）；`membership_page.dart` 由官网跳转壳重建为 App 内公民币订阅；价格链读 `PlatformPrice`（`8964/chain/square_chain_service.dart`）；`8964/services/square_api_client.dart` 删 `priceUsdMonthly/isPrepaid/cancelAtPeriodEnd/subscriptionSource` + 加 `confirmPlatformSubscription`。创作者轨（已公民币）不动。
- **官网（as-built）**：`citizenweb` 会员页 `Membership.tsx` **整页删除**（含路由/导航/专属组件），官网不再承载会员支付。
- **控制台 / 配置（as-built）**：`citizenconsole` 删 `membership-test`/`membership-e2e` 动作 + `STRIPE_API_KEY`/`STRIPE_HOOK_SECRET` secret + `cloudflare-membership-e2e.mjs`；`wrangler.toml` 删 `FREEDOM/DEMOCRACY/SPARK_PRICE_ID`/`CHECKOUT_*`/`STRIPE_DEV_PROXY` 及 Stripe secret 说明（`TOPUP_*` 边界保留）。

## 备选方案

- 预付/托管：否（用户定按月自动扣）；平台价用 primitives 常量：否（须 storage 自助调价）；平台价能力位按类别码：否（须绑唯一 CID）；分账用 fee_policy/烧毁兜底：否（须原子事务）；创作者键用 cid_or_account：否（须钉稳定 CID）；退款入口：否（破坏税终局）。

## 后续动作

- 任务卡：`memory/08-tasks/open/20260716-citizen-coin-subscription.md`（第1部分订阅:平台会员+创作者会员,含框架目录方案）；税务=`20260716-onchain-tax-settlement.md`（第2部分,ADR-038）。
- **已定稿 Accepted**（2026-07-17）。链端 `square-post` 订阅 pallet + 创作者公民币轨已实建；本次（2026-07-17）完成平台会员改公民币 + Stripe/USDC 全系下线（Worker/App/官网/控制台/配置，as-built 见「影响」）。剩余：技术公司注册后补 `PLATFORM_MEMBERSHIP_ACCOUNT` 并接 onchina 调价；平台/创作者 confirm 的链读硬化（当前信任上链 tx，与创作者同一 TODO）。
