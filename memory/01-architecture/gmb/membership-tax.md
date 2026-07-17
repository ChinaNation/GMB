# 工程架构：公民币会员 + 收入台账 + 链上税务

> **架构修订（2026-07-16，用户定）**：会员订阅是 CitizenApp 内容功能，**并入现有 `misc/square-post`（保持此名、零新增 pallet）**，非独立 pallet；下文凡「subscription pallet」一律指 square-post 扩展（新增 `subscription.rs`/`billing.rs`/`proposal.rs` 子文件 + storage + call，复用其 `pallet_index`）。App「我的」页会员与通讯录之间加「创作者」栏。第1部分落地工作内容以 `memory/08-tasks/open/20260716-citizen-coin-subscription.md` 为准。

> 范围：**工程架构**（分层 / 目录结构 / 统一命名 / 端到端闭合），不改业务模型（业务决策见 ADR-037 会员、ADR-038 税务）。已并入架构探源（5 工程范式）+ 对抗审查（1 high + 5 medium + 3 low）修正，关键事实经磁盘核对。`[拍板]` 处待用户确认，其余为推荐终态。

## 一、系统分层

### 1.1 全局四层 + 数据流

```
┌───────────────────────────────────────────────────────────────────┐
│ 链层  citizenchain/runtime  —— 唯一真源(状态机 + 资金动作 + 台账)     │
│   subscription pallet(idx 35) / tax-registry pallet(idx 36·第3期)  │
│   Storage 是全系统唯一权威值;其余端只读镜像 / 构造 call,不另存定价    │
└──────────────▲──────────────────────────────▲─────────────────────┘
   QR(冷签)     │ SCALE call / storage read     │ 标准 extrinsic(热签)
┌──────────────┴──────────┐        ┌───────────┴─────────────────────┐
│ 机构治理层 onchina       │        │ 边缘 BFF 层 cloudflare Worker     │
│ domains/membership 调价  │        │ membership/citizen_coin.ts       │
│ domains/tax      设税率   │        │ 链读确认 + D1 幂等镜像(非真源)    │
│ 冷签 internal-vote       │        │ requireCreatorSubscription 门禁   │
└──────────────▲──────────┘        └───────────▲─────────────────────┘
   QR 载荷      │                    HTTP(snake_case JSON)│
┌──────────────┴─────────────────────────────┴─────────────────────┐
│ 客户端层                                                          │
│  CitizenApp(热钱包:subscribe/cancel/set_creator_plans + 生物识别) │
│  CitizenWallet(冷钱包:onchina 机构写 QR 三处登记,严格两色)         │
│  citizenweb(无私钥:卡轨 Stripe 闭环 + 公民币"把手"引导入 App)      │
└───────────────────────────────────────────────────────────────────┘
```

层间接口（严格四类，不新造）：Rust `trait`（configs.rs 装配终态回调/台账写入）｜ JSON-RPC（`chain/rpc.ts`、App `lib/rpc/ChainRpc` 链读，唯一网关）｜ HTTP snake_case（Worker 路由，镜像/门禁）｜ QR（SCALE+签名尾，onchina↔CitizenWallet 冷签）。

**核心数据流不变量**：定价/税率**单向从链上 storage 流出**——onchina 经治理写入、BFF/App 只读镜像，任一端不得反向回写或另建第二定价源。

### 1.2 pallet 内部分层（照 onchain-issuance 成熟范本）

| 文件 | 承载 | subscription | tax-registry |
|---|---|---|---|
| `lib.rs` | `#[pallet]` 本体 Config/Storage/Event/Error/GenesisConfig/`#[pallet::call]` | `subscribe`/`cancel`/`set_creator_plans` + `propose_set_platform_price` | `propose_set_tax_rate` |
| `types.rs` | 纯数据结构 | `IssuerKey`/`SubscriptionState`/`CreatorTier` | `AuthorizationRecord`/`TaxRule`（`TaxPeriod` 不在此，见闭合5） |
| `proposal.rs` | `MODULE_TAG` + 4B `ACTION_*` + 提案体 | `MODULE_TAG=b"sub-scr"` + `ACTION_SUB_SET_PLATFORM_PRICE=*b"SPPR"` | `MODULE_TAG=b"tax-reg"` + `ACTION_TAX_SET_RATE=*b"TXRT"` |
| `validation.rs` | 入参校验纯函数 + `#[cfg(test)] mod tests` | 档位/价护栏 | 税率 Perbill 护栏 |
| `execution.rs` | `InternalVoteExecutor<T>` 按 `MODULE_TAG` 认领 | 落 `PlatformPrice` | 落 `TaxRules` |
| `billing.rs`/`settlement.rs` | 跨块有界桶扫 `on_initialize` | `billing.rs` DueQueue 桶扫扣款 | `settlement.rs` tax_period 桶扫结算 |
| `authorization.rs`（仅 tax） | 接 legislation-yuan 下游推送 | — | `TaxAuthorizationWriter` 写 `TaxAuthorization` |
| `weights.rs`/`benchmarks.rs`/`tests/{mod,cases,…}.rs` | 权重/基准/测试 | `tests/{mod,cases,billing}` | `tests/{mod,cases,settlement,authorization}` |

### 1.3 subscription ↔ tax 解耦（单向，无环）

铁律：**两 pallet 互不 import**，只经 `configs.rs` 装配的 trait 连接、数据单向。

```rust
// primitives/src/income_ledger.rs [新增] —— 唯一记账接口 + TaxPeriod 单源
pub trait IncomeLedgerWriter<AccountId, Balance> {   // 收款方=钱包账户;税务侧内部 account→CID(有则记台账,无则跳过=自行申报)
    fn record_income(payee: AccountId, period: TaxPeriod, amount: Balance) -> DispatchResult;
}
impl<C, B> IncomeLedgerWriter<C, B> for () {   // 第1/2期空实现(记0),subscription 侧不变即可先行
    fn record_income(_: C, _: TaxPeriod, _: B) -> DispatchResult { Ok(()) }
}
```

- subscription `billing.rs` 扣款成功 → `T::IncomeLedger::record_income(...)`（关联类型）。
- **分期相位（并入审查 M）**：第 1/2 期 `type IncomeLedger = ()`（空实现记 0），**tax pallet(36) 不入 construct_runtime**；第 3 期才 `type IncomeLedger = TaxRegistry` 并挂 idx 36。全文以此为准，无"接真实 TaxRegistry 又记 0"的自相矛盾。
- 单向：subscription 依赖 trait 抽象不依赖 tax crate；税务纯下游，结算只读 `IncomeLedger`，永不回调 subscription。

### 1.4 核心 vs 非核心边界（runtime 只放核心，其余全链下）

原则：**runtime pallet 只承载"权威真源存储"和"不可篡改的资金/治理动作"；凡能在链下计算、展示、编排、缓存、门禁的，一律不进 runtime。** pallet 越薄越好。

| 功能 | 在哪 | 理由 |
|---|---|---|
| **① 权威状态** `Subscriptions`/`PlatformPrice`/`CreatorPlans`/`IncomeLedger`/`TaxRules`/`TaxAuthorization`/`DueQueue` | **runtime** | 资金授权与税基的唯一真源,必须链上不可篡改 |
| **② 资金动作** 按月扣款转账(`billing`)、税结算征收(`settlement`) | **runtime** | 真金转移,必须链上原子执行 |
| **③ 治理写入** `propose_set_platform_price`/`propose_set_tax_rate` 执行落库 | **runtime** | 定价/税率权威值,经治理写 |
| **④ 调度 + 最小安全校验** `on_initialize` 桶扫;档位合法/价护栏区间/授权区间校验 | **runtime** | 防非法状态入链;调度必须链上驱动 |
| 订阅编排(构造 extrinsic/生物识别/提交/轮询确认) | App `subscription_service` | 无需上链,客户端编排 |
| 换档/降档折算、金额预览、异档守卫 | App/BFF | 链下算好最终参数再提交;链上只接受结果不重算 |
| 门禁资格判断 `requireCreatorSubscription` | BFF(链读) | 链上订阅关系为真源,判断在边缘,不进 pallet |
| D1 镜像/缓存/幂等去重 | BFF `citizen_coin.ts` | 加速读,非真源 |
| 收入/申报/历史展示 | App(读链+Isar) | 纯展示,读链渲染 |
| 价格换算/UI/文案 | App/web | 表现层 |

要点:`validation.rs` **只做安全不变量校验**(防止非法状态写入),复杂业务规则(异档守卫、比例折算、预览)一律链下先算；创作者门禁、镜像、编排、展示**零行进 pallet**。第 1/2 期税务未接入时,subscription 连 `IncomeLedger` 记账都是空实现——pallet 只在真正需要链上强制时才承载逻辑。

## 二、目录结构（五端，标 `[新增]/[改动]/[删除]`）

### 2.1 citizenchain/runtime

```
Cargo.toml                                   [改动] members 追加两行(带中文尾注释)
runtime/primitives/src/
  ├── membership_price.rs                    [新增] MembershipLevel 枚举+单位+护栏(不放可调价)
  ├── tax_policy.rs                          [新增] Perbill 上限等硬顶(不放税率值)
  ├── income_ledger.rs                       [新增] IncomeLedgerWriter trait + TaxPeriod + impl for ()
  └── lib.rs                                 [改动] pub mod membership_price/tax_policy/income_ledger;
runtime/misc/square-post/                    [改动] 扩展(保持名·零新增 pallet):原广场发帖索引 + 会员订阅
  ├── lib.rs/post.rs         原发帖(publish_post + SquarePosts);发帖逻辑抽 post.rs
  ├── subscription.rs [新增] IssuerKey{Platform,Creator(AccountId)}(创作者=任意钱包账户,无CID)/SubscriptionState/CreatorTier + subscribe/cancel/set_creator_plans
  ├── billing.rs      [新增] on_initialize 按月扣款桶扫(DueQueue,原子,欠费即停)
  └── proposal.rs     [新增] MODULE_TAG + propose_set_platform_price 治理 + InternalVoteExecutor
     新增 Storage: Subscriptions/PlatformPrice/CreatorPlans/DueQueue;Config +Currency+治理+护栏;Genesis PlatformPrice 三档
runtime/public/tax-registry/                 [新增] idx 36(第3期入表) name="tax-registry"
  └── src/{lib,types,proposal,validation,execution,authorization,settlement,weights,benchmarks}.rs + tests/{mod,cases,settlement,authorization}.rs
```

`[改动] runtime/src/lib.rs` construct_runtime（索引取当前最大 34+1 顺延；tax 第3期才加）：
```rust
// 会员订阅并入现有 square-post,不新增 pallet index(square-post 已在 misc 组注册)
#[runtime::pallet_index(36)] pub type TaxRegistry = tax_registry;    // 公权-税务台账与申报期结算(第3期)
```

`[改动] runtime/src/configs.rs` 四处装配：
1. `impl subscription::Config`（关联类型全列：`RuntimeEvent`/`WeightInfo`/`Currency`/`IncomeLedger`(第1/2期`=()`)/护栏常量绑定）；第3期 `impl tax_registry::Config`（含 `TaxAuthorizationWriter`）。
2. **回调元组追加第 6 顶层槽（并入审查 M——现 5 槽未达 (A..F) 上限，不重打包已有）**：
   `type InternalVoteResultCallback = ( multisig, (pub_manage,priv_manage), (per_manage,per_admins), resolution_destroy, grandpakey_change, (subscription::Executor, /*第3期*/ tax_registry::Executor) );`
3. **`RuntimeCallFilter::contains` 与 `RuntimeFeeRouter::fee_route` 拆开写（并入审查 H1，二者语义相反）**：
   - `fee_route` 穷尽匹配（编译器强制）：`subscribe`/`cancel`/`set_creator_plans` → `signer_onchain_route`；`propose_*` → `institution_onchain_route`；`billing` 内部触发 → `FeeRoute::Free`。
   - `contains` 结尾是 `_ => true`（fail-open），故**未启用/仅内部驱动的 call 必须显式 `RuntimeCall::TaxRegistry(_) => false`**（仿 `OnchainIssuance(_) => false`），防治理入口被静默暴露为可入块；热轨 `Subscription(subscribe/cancel/set_creator_plans)` 显式列出放行，防误入 deny。
4. `legislation-yuan::Config` 加 `type TaxAuthorizationWriter = tax_registry::Pallet<Runtime>`（第3期）。

`[改动] citizenchain/node/src/core/chain_spec.rs`（**遗漏补**）：为 subscription（初始 `IssuerKey=Platform` 绑技术公司 CID）写 GenesisConfig；tax 第3期同理。chainspec 创世冻结，GenesisConfig 非空必须创世写入。

### 2.2 citizenchain/onchina（两新域，冷签）

```
domains/membership/                          [新增] 技术公司调平台价
  └── {mod,model,chain_call,handler}.rs  chain_call: MEMBERSHIP_PALLET_INDEX=35 + CALL_PROPOSE_SET_PLATFORM_PRICE + build_*()
domains/tax/                                 [新增] 税务机构设税率(域名=tax,对齐 tax-registry;不用 fiscal 防命名分裂)
  └── {mod,model,chain_call,handler}.rs  chain_call: TAX_REGISTRY_PALLET_INDEX=36 + CALL_PROPOSE_SET_TAX_RATE + build_*()
auth/operation_auth.rs   [改动] AdminActionType::SetPlatformPrice/SetTaxRate 四 arm:enum + as_str + auth_type()=PasskeyColdSign + parse()
                                (parse 字符串→enum 是唯一非编译器强制项,漏则运行期静默 400,须与前端串逐字一致)
platform/capability.rs   [改动] can_set_platform_price(技术公司唯一 CID 精确匹配)/can_set_tax_rate(税务机构唯一 CID)
                                —— 对 capabilities_for(institution_code) 类型码范式的显式偏离,注释标注(审查 H1:类型码=提权)
workspace/manifest.rs    [改动] workspace_action: set_platform_price / set_tax_rate
```

### 2.3 citizenapp/cloudflare（公民币轨 + 删 USDC 残桩）

```
membership/citizen_coin.ts   [新增] citizenCoinConfirmRoute(POST /v1/square/membership/citizen-coin/confirm)
                                    + decodeSubscription()(仿 identity.ts,复用 storage_key.ts,链读确认→D1 幂等镜像)
membership/service.ts        [改动] +requireCreatorSubscription(fail-closed 链读不缓存) +upsertCitizenCoinMembership
                                    subscriptionIsActive 加 'citizen_coin' 分支
membership/prepaid.ts        [删除] 整文件
membership/subscribe.ts      [改动] 删 trialEnd/payment_switch(保 Stripe 卡轨)
membership/webhook.ts        [改动] 删 processPrepaidCheckout + checkout.session.completed 分派
membership/plans.ts          [改动] 档位集合与 App _fallbackMembershipPlans 对齐(法币价仍此单源,见闭合3)
account/service.ts           [改动] 删 usdc 分支
routes.ts                    [改动] 注册 citizen-coin/confirm
limits/catalog.ts            [改动] routeLimits 共享正则 (subscribe|cancel|prepaid) 去掉 |prepaid、删 /prepaid/change(审查 M:改正则非删行)
types.ts                     [改动] +CitizenCoinConfirmBody/Row;source 收窄 'stripe'|'citizen_coin';删 prepaid_payment_ref 字段(审查 M)
migrations/0001_square_core.sql [改动] 建 square_citizen_coin_payments(tx_hash PK,owner_account,membership_level,granted_at);删 prepaid_payment_ref 列
```
残留归零判据：`grep -rn 'prepaid\|usdc' citizenapp/cloudflare/src` 收敛到 0（`asset_balance_tile.dart` 的 "USDC" 是资产币种符号，非会员，勿误删）。

### 2.4 citizenapp/lib（App 热钱包）

```
my/membership/membership_page.dart      [改动] 直调 SubscriptionService.subscribe;banner 加 PastDue(欠费即停);删 USDC 入口
my/membership/subscription_service.dart [新增] 编排:校验+SubscriptionRpc+生物识别;平台/创作者共用靠 IssuerKey 区分
my/membership/creator_income_ledger_*.dart [新增·第3期] 链上活跃+Isar 历史双轨(仿 personal_proposal_history)
rpc/subscription_rpc.dart               [新增] 照 transfer_rpc.dart:_subscriptionPalletIndex/_*CallIndex + _build*Call + SignedExtrinsicBuilder
8964/creator/{creator_plan_service,creator_plan_edit_page,creator_content_gate}.dart [新增·第2期] 创作者档读写 + 门禁 fail-closed
8964/profile/widgets/creator_subscribe_button.dart [新增·第2期] 调同一 SubscriptionService.subscribe(IssuerKey.creator(cid))
```
铁律：公民币订阅一律 `SignedExtrinsicBuilder` 标准 extrinsic + 生物识别，**禁分配 op_tag、禁冷钱包盲签分支**（不保留热/冷双分支）。

### 2.5 citizenweb（官网，无私钥）

```
pages/Membership.tsx            [改动] 删 crypto-prepaid/usdc-* 全部预付 UI 与逻辑;保留 card-subscribe/cancel 全 Stripe 闭环
components/CitizenCoinHandle.tsx [新增] 照 DownloadButton.tsx,独立"把手"引导入 App,零签名逻辑
```

### 2.6 CitizenWallet 三处登记（仅 onchina 冷签动作 propose_*，热轨不进）

```
signer/pallet_registry.dart   [改动] // ---- Subscription(35) ---- / // ---- TaxRegistry(36) ---- + call_index 常量
signer/payload_decoder.dart   [改动] decode() 加 35/36 路由 + _decodeProposeSetPlatformPrice/_decodeProposeSetTaxRate
qr/qr_protocols.dart          [改动] action=(pallet<<8)|call + fromDecodedAction 'propose_set_platform_price'/'propose_set_tax_rate'
test/signer/payload_decoder_test.dart [改动] 裸 call_data + 带签名尾 各一用例
```

## 三、统一命名（跨五端对照表）

`★` = 必须逐字节/逐字一致（漏则冷签红拒或静默漂移）。命名硬规则：目录分隔符层级封顶（crate ≤1 分隔、字段 ≤2 且最短）｜顶级单词无分隔符、子级 kebab-case｜禁 `name/label/type/status` 泛化承载业务语义｜禁 `v2/tmp/new/old`｜跨语言同名只允许 snake↔camel 差异、手工映射。

| 对象 | runtime(Rust) | onchina(Rust) | Worker(TS/SQL) | CitizenApp(Dart) | CitizenWallet(Dart) | 一致性 |
|---|---|---|---|---|---|---|
| pallet_index | `35`/`36` | `MEMBERSHIP_PALLET_INDEX=35`/`TAX_REGISTRY_PALLET_INDEX=36` | — | `_subscriptionPalletIndex=35` | `subscriptionPallet=35` | ★ 五端相等 |
| MODULE_TAG | `b"sub-scr"`/`b"tax-reg"` | — | — | — | — | ★ 全仓唯一(连字符缩写风格) |
| 订阅 storage | `Subscriptions:StorageMap<(AccountId,IssuerKey),SubscriptionState>` | 读键 | 读 storageMapKey | 读 | — | PascalCase |
| 平台价 storage | `PlatformPrice:StorageMap<MembershipLevel,u128>` | 写目标 | 镜像 | 读 | — | ★ 键名 |
| 创作者档 | `CreatorPlans:StorageMap<AccountId,BoundedVec<CreatorTier>>`(键=创作者钱包账户,非CID) | — | — | 读写 | — | PascalCase |
| 收入台账 | `IncomeLedger:StorageMap<(CidNumber,TaxPeriod),u128>` | — | — | 第3期读 | — | TaxPeriod 源=primitives |
| 税授权/规则 | `TaxAuthorization`/`TaxRules` | 写目标 | — | — | — | PascalCase |
| 扣款队列 | `DueQueue:StorageMap<BlockNumber,BoundedVec<SubKey,MaxPerBucket>>` | — | — | — | — | 桶扫单源 |
| extrinsic 热签 | `subscribe`/`cancel`/`set_creator_plans` | — | — | `subscribe`/`cancel`/`setCreatorPlans` | — | ★ 动作串 |
| extrinsic 冷签 | `propose_set_platform_price`/`propose_set_tax_rate` | build 构造 | — | — | decode 解析 | ★ fn 名=锚点串 |
| Event | `Subscribed`/`Cancelled`/`Charged`/`ChargeFailed`/`PlatformPriceSet`/`IncomeRecorded`/`TaxRateSet`/`TaxSettled` | — | — | — | — | PascalCase 过去分词 |
| ACTION 码 | `*b"SPPR"`/`*b"TXRT"` | — | — | — | — | 4B 全大写 |
| 动作枚举 | — | `SetPlatformPrice`→`"SET_PLATFORM_PRICE"` | — | — | — | 枚举 PascalCase |
| 鉴权档 | — | `PasskeyColdSign` | — | — | — | ★ 冷签必属此 |
| 能力位 | — | `can_set_platform_price`/`can_set_tax_rate`(精确 CID) | 前端 capabilityMap 同名 | — | — | camelCase |
| 会员档 | `MembershipLevel{Freedom,Democracy,Spark}` | — | `membership_level` | `membershipLevel` | — | ★ 语义同名 |
| 来源字段 | — | — | `subscription_source∈{stripe,citizen_coin}` | `subscriptionSource` | — | ★ 值域 |
| 欠费态 | `SubscriptionState.past_due` | — | `past_due` | `pastDue` | — | ★ |
| 路由 | — | — | `/v1/square/membership/citizen-coin/confirm` | — | — | kebab 段 |
| D1 表 | — | — | `square_citizen_coin_payments(tx_hash PK,…)` | — | — | `{domain}_{noun}` |
| 目录 | `misc/subscription`、`public/tax-registry` | `domains/membership`、`domains/tax` | `membership/` | `8964/creator/` | — | kebab |

## 四、端到端闭合链

**闭合1 公民币订阅（热钱包）**：`membership_page` 选档 → `subscription_service.subscribe(IssuerKey.platform,level)` 校验 → `subscription_rpc._buildSubscribeCall(35,idx,SCALE(issuer,plan))` → `WalletManager.signWithWallet`（local_auth 生物识别） → `SignedExtrinsicBuilder.signAndSubmit` 标准 extrinsic → 链上写 `Subscriptions[(acct,Platform)]=Active` → onWatch(inBlock) → App `POST …/citizen-coin/confirm{tx_hash}` → `citizen_coin.ts` 链读 `Subscriptions` 确认 → `upsertCitizenCoinMembership`（`tx_hash` PK 幂等）镜像 D1。不变量：链上唯一真源、D1 仅镜像、禁 op_tag/两段式、confirm 只读不定价。

**闭合2 onchina 调价（冷签）**：`domains/membership/handler` → `capability.can_set_platform_price`（技术公司唯一 CID）→ `operation_auth` `PasskeyColdSign` → `chain_call.build_*` 构造 `propose_set_platform_price` 裸 SCALE(pallet=35) → QR → CitizenWallet `payload_decoder` 命中 35/idx → 三处登记齐→两色核对签 / 缺任一→`decodeFailed` 红拒 → `chain_submit` → 链上建 InternalVote → 机构表决通过 → `InternalVoteResultCallback` 命中 `subscription::Executor` 按 `MODULE_TAG=b"sub-scr"` 认领 → `execution` 写 `PlatformPrice[level]` → onchina 回读确认。**逐字节相等项（并入审查 M）=数字动作码 `(pallet<<8)|call_index` + SCALE 字段序**；**方法名闭合三元组 = Rust `#[pallet::call]` fn 名 ↔ `payload_decoder` action 串 ↔ `qr_protocols` switch key**（`review_title` 是中文展示串，**不参与字节校验**）。

**闭合3 单源闭合（无双写漂移）**：`primitives::membership_price`（仅枚举/单位/护栏 `[拍板]`）→ runtime `PlatformPrice`/`CreatorPlans`/`TaxRules` storage（可调值唯一真源，治理写）→ 链读（唯一网关）→ Worker/App 只读镜像。**公民币轨定价单源 = runtime `PlatformPrice`**；**法币轨价单源 = `plans.ts`/Stripe（第二轨，非漂移）**——两轨同档价各自单源、不跨折算，档位集合三处（`plans.ts`↔App `_fallbackMembershipPlans`↔runtime `MembershipLevel`）需人工对齐（建议后补 CI 断言，属已知开口）。

**闭合4 命名闭合（`subscribe` 逐端零漂移）**：runtime fn `subscribe`/call_index=N ↔ App `_subscribeCallIndex=N`+`SubscriptionService.subscribe` ↔ Worker `subscription_source='citizen_coin'`；`IssuerKey::Platform|Creator(AccountId)` 单源 primitives(创作者=任意钱包账户,无CID)。防漂移三机制：①金标 fixture 逐字节断言（`primitives/tests/fixtures/*.json` + CitizenWallet 同构对同一 JSON）②onchina 编码器单测反向对拍 Rust `.encode()` ③冷签严格两色白名单（未登记默认拒）。新协议先入 `memory/07-ai/unified-protocols.md` + `unified-naming.md`。

**闭合5 收入台账→税务（单向无环）**：subscription `billing` 扣款成功（全额转收款方，不预扣）→ `T::IncomeLedger::record_income(payee_cid,period,amount)`（trait 抽象）→ tax-registry `IncomeLedger[(cid,period)]+=amount` 幂等；单向到此，tax 永不回调 subscription → tax `settlement.on_initialize` 按 `tax_period` 到期桶有界结算，复核 `TaxAuthorization`(有效期/范围)+读 `TaxRules`。不变量：subscription 依赖 trait 不依赖 tax crate（无编译环）；第1/2期 `IncomeLedger=()` 空实现记 0，subscription 不变先行；征税权两级治理（legislation 授权经 `authorization.rs` 下游 push 写 `TaxAuthorization` → 税务机构 internal-vote 设 `TaxRules`），subscription 不感知税务。

## 拍板点（落地前）
1. `[已定·分组]` 会员订阅**并入 `misc/square-post`**（保持名、零新增 pallet）；tax-registry→`runtime/public/`（公权壳+投票外包）。
2. `[护栏常量]` `membership_price.rs`/`tax_policy.rs` 是否设 `PLATFORM_PRICE_MIN/MAX_FEN`、Perbill 上限硬护栏。
3. `[call_index 定稿]` 各 extrinsic `#[pallet::call_index(N)]` 具体号（五端同批，允许永久留洞不复用）。

关键路径引用：`runtime/src/configs.rs`（回调元组第6槽 + CallFilter/FeeRoute 拆写）、`runtime/src/lib.rs`（construct_runtime 35/36）、`onchina/src/domains/address/chain_call.rs`（chain_call 范本）、`cloudflare/src/membership/service.ts`（门禁落点）、`citizenapp/lib/rpc/transfer_rpc.dart`（subscription_rpc 骨架范本）、`node/src/core/chain_spec.rs`（创世装配）。
