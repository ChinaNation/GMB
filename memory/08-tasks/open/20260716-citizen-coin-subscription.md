# 任务卡（草案·待确认）：公民币订阅（第1部分 · 融入广场内容体系）

> 状态：**草案，待确认**。**完整技术架构真源（精确签名/扣款算法/闭合/命名/审查修正）= `memory/01-architecture/gmb/subscription-part1-tech.md`**；依据 ADR-037 + `memory/01-architecture/gmb/membership-tax.md`。第2部分税务见 `20260716-onchain-tax-settlement.md`（串行在后，不阻塞本卡）。

任务需求：会员订阅是 CitizenApp 内容功能，**融入现有广场/内容体系，不新建独立模块**。链上**并入 `square-post`（保持此名）做最小实现**；App"我的"页在**会员与通讯录之间加"创作者"栏**（用户管理自己创作者订阅的入口）；BFF 用现有 `membership/` 内容模块。必要功能只两条：**① 用户订阅平台会员 ② 用户订阅创作者会员**。Stripe 只留卡。

所属模块：citizenchain（`misc/square-post` 扩展 + `primitives`）、cloudflare（`membership/` 内容 BFF）、citizenapp（`my/user`+`my/membership`+`my/creator`+`8964`）、onchina（调价入口）、citizenweb、CitizenWallet。

必须遵守：
- **不新建 pallet**：会员订阅并入 `square-post`（发帖+会员同属广场内容域），复用其 `pallet_index`。
- **链上尽量最小**（核心 vs 非核心 §1.4）：pallet 只放订阅状态/平台价/创作者档/扣款调度；换档折算/预览/异档守卫/门禁/镜像/展示**全链下**。
- 签名分层：公民订阅热钱包标准 extrinsic + 生物识别（禁 op_tag 0x1D、禁冷钱包盲签）；技术公司调价冷签 internal-vote。
- 平台价链上 storage（技术公司 onchina 调）；`primitives::membership_price` 只放枚举/护栏。
- 链开发期重新创世无 migration/兼容/spec_version/残留；不清楚先沟通。

---

## 框架目录方案（融入版；`[新增]/[改动]/[删除]`；`A`=平台会员 `B`=创作者会员）

### ① 链层 citizenchain/runtime/misc/square-post/（扩展，**零新增 pallet，保持名**）

现状：广场发帖索引（`SquarePosts`/`PublishedPostCountByAccount` + `publish_post`，无调度无治理）。扩展为**广场内容业务 pallet**（发帖 + 会员订阅）：
```
runtime/misc/square-post/src/
├── lib.rs           [改动·A] 聚合 Config/Storage/Event/Error/GenesisConfig/call/#[pallet::hooks]
│     Config  +Currency(扣公民币) +调价治理接口 +护栏常量(MaxCreatorTiers 等)
│     Storage +Subscriptions:(subscriber,IssuerKey)->SubscriptionState  +PlatformPrice:MembershipLevel->u128分
│             +CreatorPlans:AccountId->BoundedVec<CreatorTier>(创作者=任意钱包账户,无CID要求)  +DueQueue:BlockNumber->BoundedVec<SubKey>
│     call    +subscribe(issuer,plan)/cancel(issuer)[热·A/B] +set_creator_plans(tiers)[热·B] +propose_set_platform_price[冷·A]
│     hooks   +on_initialize 桶扫 DueQueue:with_storage_layer 原子{扣款→全额转 issuer→顺延};失败→欠费即停
│     Genesis PlatformPrice 三档默认(199900/599900/5999900) + IssuerKey::Platform 绑技术公司 CID
├── post.rs          [改动·A] 原发帖逻辑抽出(publish_post + SquarePosts)
├── subscription.rs  [新增·A] IssuerKey{Platform,Creator(AccountId)}(创作者=任意钱包账户,无CID限制) · SubscriptionState{level_or_tier,price_fen,next_charge_at,status} · CreatorTier · subscribe/cancel/set_creator_plans
├── billing.rs       [新增·A] on_initialize 按月扣款桶扫(欠费即停)
├── proposal.rs      [新增·A] MODULE_TAG + propose_set_platform_price 治理提案 + InternalVoteExecutor
├── weights.rs / benchmarks.rs [改动]
└── tests/{mod,cases,billing}.rs [改动]

runtime/primitives/src/membership_price.rs  [新增·A] MembershipLevel{Freedom,Democracy,Spark} + 单位(分) + 护栏(不放可调价)
runtime/src/configs.rs  [改动·A] square-post Config 补 Currency/治理/护栏;回调元组接 square-post::InternalVoteExecutor;
                                CallFilter:subscribe/cancel/set_creator_plans 放行;FeeRoute:热签→signer_onchain·propose→institution_onchain·billing→Free
node/src/core/chain_spec.rs  [改动·A] square-post GenesisConfig 写 PlatformPrice 三档 + Platform 绑技术公司 CID
```
（`PLATFORM_MEMBERSHIP_ACCOUNT`=技术公司账户，公司后期注册后补；补入前平台轨挂起。IncomeLedger 记账 hook 留空，税务第2部分再接。）

### ② 客户端 App citizenapp/lib（融入现有内容目录）

```
my/user/user.dart                     [改动·A] 菜单「会员」与「通讯录」之间插入「创作者」栏:_openCreator→CreatorPage(user.dart:441-457 之间)
my/membership/membership_page.dart    [改动·A] 平台会员公民币轨:选三档→SubscriptionService.subscribe(Platform,level)→生物识别→上链→confirm;banner 加 PastDue;删 USDC
my/creator/creator_page.dart          [新增·B] 「创作者」栏落地:管理我的创作者订阅——设我的会员档/价·看谁订阅了我·我的公民币收入(读链/BFF)
my/creator/creator_plan_edit_page.dart[新增·B] 编辑创作者档位/月价(热签 set_creator_plans)
my/membership/subscription_service.dart[新增·A] 编排:校验+SubscriptionRpc+生物识别;平台/创作者共用靠 IssuerKey 区分
rpc/subscription_rpc.dart             [新增·A] square-post pallet_index + subscribe/cancel/set_creator_plans call_index + SignedExtrinsicBuilder
8964/profile/widgets/subscribe_button.dart [新增·B] 广场他人主页「订阅 TA」→ subscribe(Creator(creator_account),tier)
```

### ③ 边缘 BFF cloudflare/src/membership/（现有内容模块，加公民币轨）

```
membership/citizen_coin.ts   [新增·A] 链读 Subscriptions 确认 + D1 幂等镜像(tx_hash PK) + requireCreatorSubscription[B](fail-closed 门禁)
membership/service.ts        [改动·A/B] subscriptionIsActive 加 citizen_coin 分支;+upsertCitizenCoin;+requireCreatorSubscription
membership/prepaid.ts        [删除·A] 整文件(USDC)
membership/{subscribe,webhook,plans}.ts + account/service + routes + limits/catalog + types + migrations/0001_square_core.sql  [改动·A] 删 USDC 残桩;source 收窄 stripe|citizen_coin;建 square_citizen_coin_payments(tx_hash PK)
```
（残留归零：`grep -rn 'prepaid\|usdc' cloudflare/src` = 0。）

### ④ 机构治理层 onchina/src（技术公司调价·冷签，操作 square-post 治理 call）

```
domains/membership/  [新增·A] {mod,model,chain_call,handler}.rs  SQUARE_POST_PALLET_INDEX + CALL_PROPOSE_SET_PLATFORM_PRICE + build_*()
auth/operation_auth.rs  [改动·A] AdminActionType::SetPlatformPrice(PasskeyColdSign 四 arm)
platform/capability.rs  [改动·A] can_set_platform_price(技术公司唯一 CID 精确)
workspace/manifest.rs   [改动·A] set_platform_price action
```

### ⑤ 官网 citizenweb/src
```
pages/Membership.tsx              [改动·A] 删加密预付 UI;保留 Stripe 卡轨闭环
components/CitizenCoinHandle.tsx  [新增·A] 「用 App 公民币订阅」把手,零签名
```

### ⑥ 冷钱包 CitizenWallet lib（仅调价冷签）
```
signer/pallet_registry.dart / payload_decoder.dart / qr/qr_protocols.dart  [改动·A] square-post 的 propose_set_platform_price 三处登记
```

---

## 两条必要功能（最小闭环）
- **① 订阅平台会员**：`membership_page` 选三档 → `subscribe(Platform, level)` 热签上链 → 按月 `on_initialize` 自动扣公民币入技术公司账户 → 欠费即停；worker 链读确认镜像 D1 → 权益生效。
- **② 订阅创作者会员**：**创作者=任意钱包账户（无 CID 限制，谁都能开）**。创作者在「创作者」栏 `set_creator_plans` 设档/价 → 订阅者在广场他人主页 `subscribe(Creator(creator_account), tier)` → 按月扣款**全额转创作者钱包账户** → BFF `requireCreatorSubscription(subscriber, creator_account)` 门禁解锁专属内容。**收款不受 CID 限制**；CID 只在税务侧（第2部分）用——有 CID 走链上申报结算、无 CID 自行申报，均不阻塞收款。

**链上不做**（全链下）：换档/降档折算、金额预览、异档守卫、内容门禁、D1 镜像、收入/历史展示、UI。

## 分阶段
- **Phase A 平台会员**：square-post 扩订阅+扣款+调价 + primitives + BFF 公民币轨 + 删 USDC + App「会员」页轨 + `user.dart` 加「创作者」栏(占位) + onchina 调价 + 官网 + 冷签登记。
- **Phase B 创作者会员**：`CreatorPlans`/`set_creator_plans` + `subscribe(Creator)` + BFF 门禁 + `my/creator/` 管理页 + 广场「订阅 TA」。依赖广场/聊天权限(ADR-028/020)。

## 验收标准
- 平台会员全链跑通:签→首扣→上链→worker 确认→权益生效;按月自动续扣;余额不足即停可充值重订。
- 创作者可设档被订阅、款全额进钱包、订阅有效才解锁专属内容。
- `square-post` 扩展后发帖(publish_post)回归不破;卡轨不破;USDC 残桩零残留;**无新增 pallet**。
- 「创作者」栏在 user.dart 会员与通讯录之间;pallet 仅含核心(无编排/折算/门禁/展示行);重新创世一次成功。

影响范围：citizenchain(square-post 扩展+重新创世) + 5 端。链改须用户显式确认后动手;Phase B 依赖广场/聊天权限系统。
