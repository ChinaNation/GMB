# 第1部分 · 公民币订阅完整技术架构（可直接实现）

> **对抗审查已并入（3 CRITICAL + 2 HIGH）**：① `Subscriptions` 改**单 hasher** `StorageMap<(AccountId,IssuerKey)>` 匹配 BFF storageMapKey；② `SubscriptionPlan` 是 **2 字节枚举** `[tag][value]`（非 1B）；③ `CreatorTier` 是 **17 字节含 tier_code**（非 16B）；④ 桶满**不误判欠费**（只真实余额失败才 PastDue）；⑤ `do_subscribe` **幂等防双扣** + `SubscriptionStatus` 补 `Cancelled`。⑥ **订阅所有类型/常量/trait 全在 `square-post` 模块内声明，零 `primitives`（区块链核心常量库）改动**——`MembershipLevel`/护栏常量/按月周期/`IncomeLedgerWriter` trait 均为 square-post 私有，不得写入 primitives（§2.2/§2.3 的 primitives 写法为错误残留，以模块内声明为准）。★`pallet_index=34`（复用 square-post，非 35）。call_index 待拍板。

真源依据：`memory/01-architecture/gmb/membership-tax.md` + `memory/08-tasks/open/20260716-citizen-coin-subscription.md` + ADR-037；文档与代码冲突处一律以**任务卡 + 实际代码基线**为准（下文★逐字项已核实）。

---

## 一、总览

### 1.1 分层与职责

| 层 | 位置 | 职责 | 签名/信任边界 |
|---|---|---|---|
| 链上 | `citizenchain/runtime/misc/square-post`（pallet_index=**34**，零新增 pallet） | 订阅状态/平台价/创作者档/按月扣款调度/最小安全校验 | 唯一真源 |
| BFF | `citizenapp/cloudflare/src/membership/citizen_coin.ts` | 链读确认 + D1 幂等镜像 + 创作者门禁 | 只读链，不代管签名 |
| App | `citizenapp/lib/{my,rpc,8964}` | 热钱包标准 extrinsic + 生物识别，UI 编排 | 热签，禁 op_tag/禁冷钱包 |
| onchina | `citizenchain/onchina/src/domains/membership` | 技术公司冷签调平台价 | PasskeyColdSign + internal-vote |
| CitizenWallet | `citizenwallet/lib/signer` | 冷签调价的两色核对（三处登记） | 仅 propose_set_platform_price 一条 |
| 官网 | `citizenweb/src/pages/Membership.tsx` | 保留 Stripe 卡轨，删 USDC 预付轨 | 零私钥 |

### 1.2 核心 vs 非核心边界（死规则）

- **链上只放**：订阅状态、平台三档价、创作者档位、按月扣款桶扫、最小安全校验。
- **全部链下**：换档折算、预览、异档守卫、门禁、镜像、展示名、时间折算。
- 创作者档位链上**只存 `price_fen`**，展示名/介绍全链下（D1/Isar）。

### 1.3 两条必要功能闭环（详见第六节）

- ① 订阅平台会员：App 选档 → 热签 subscribe(Platform, Level) → 上链扣款转技术公司账户 → worker 链读确认 → D1 镜像 → UI。
- ② 订阅创作者会员：创作者 set_creator_plans → 订阅者 subscribe(Creator(account), Tier) → 全额转创作者钱包 → 按月续扣 → 门禁实时链读解锁。

### 1.4 数据流总图

```
App(热签) ──extrinsic──▶ square-post 链上
                            │ Subscriptions / PlatformPrice / CreatorPlans / DueQueue
                            │ on_initialize 桶扫按月扣款
onchina(冷签) ─QR─▶ CitizenWallet ─extrinsic─▶ propose_set_platform_price ─internal-vote─▶ PlatformPrice
                            │
Worker BFF ◀─state_getStorage─┘  幂等镜像 D1 ◀── App confirm
```

---

## 二、链上 square-post 扩展（核心）

依据：维度1（链上设计）+ 维度3（BFF 侧对 SCALE 布局的反向约束）。

### 2.1 文件拆分（单 pallet 内四文件 + 一个 lib.rs 聚合）

```
runtime/misc/square-post/src/
├── lib.rs        顶层类型 re-export + #[frame_support::pallet] 唯一 mod
│                 （Config/Storage/Event/Error/GenesisConfig/#[pallet::call]/#[pallet::hooks] 各仅一次）
├── post.rs       原 publish_post 业务体（do_publish_post，搬迁零逻辑变化）
├── subscription.rs  IssuerKey/SubscriptionPlan/SubscriptionState/CreatorTier 类型
│                    + do_subscribe/do_cancel/do_set_creator_plans + resolve_price_and_payee + enqueue/remove_due
├── billing.rs    process_due_bucket_scan(on_initialize) + charge_due_bucket(桶扫) + try_charge_and_reschedule(原子扣款)
└── proposal.rs   MODULE_TAG=b"sub-scr" + PlatformPriceUpdateAction + do_propose_set_platform_price + InternalVoteExecutor
```

**FRAME 硬约束**：`#[pallet::storage/event/error/genesis_config/genesis_build/hooks/call]` 只能各出现一次且必须直接写在 lib.rs 的 `pub mod pallet` 内。四个业务文件只放**普通 `impl<T: Config> Pallet<T>`** 块（Rust 允许跨文件为同一 struct 追加 impl）+ 纯数据类型。5 个 extrinsic 的 `#[pallet::call]` 函数体全是一行转发。

**Cargo.toml**（仿 `grammar/grandpakey-change`）新增 path 依赖：`primitives`、`votingengine`、`entity-primitives`，均透传 `/std` feature；`[dev-dependencies]` 加 `internal-vote` 供治理路径测试。

### 2.2 类型与常量（★全在 square-post 模块内，零 primitives 改动）

> 全部声明在 **`square-post/src/subscription.rs`** 模块内（pallet 私有），**不进 `primitives`**（核心常量库只放全链共享基础设施，订阅业务不得写入）。第1部分**不含 IncomeLedger**（税务第2部分再在 square-post 加 Config 关联类型 + billing 调用点）。

```rust
// ── square-post/src/subscription.rs 内（模块私有，非 primitives） ──
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum MembershipLevel { Freedom = 0, Democracy = 1, Spark = 2 }

pub const PLATFORM_PRICE_DEFAULT_FEN: [(MembershipLevel, u128); 3] = [
    (MembershipLevel::Freedom,   199_900),
    (MembershipLevel::Democracy, 599_900),
    (MembershipLevel::Spark,     5_999_900),
];
pub const PLATFORM_PRICE_MIN_FEN: u128 = 100;            // 【推荐·拍板】1 元
pub const PLATFORM_PRICE_MAX_FEN: u128 = 100_000_000;   // 【推荐·拍板】100 万元
pub const CREATOR_PRICE_MIN_FEN: u128 = 100;            // 【推荐·拍板】1 元
pub const CREATOR_PRICE_MAX_FEN: u128 = 10_000_000;     // 【推荐·拍板】10 万元/月

/// 按月扣款周期（区块）。30 天@6 分钟/块 = 7200；模块内 const，不读也不改 pow_const。
pub const BILLING_PERIOD_BLOCKS: u32 = 7_200;

// IncomeLedgerWriter trait + TaxPeriod：第1部分不建，税务第2部分在本模块内加（下游 tax pallet 实现）。
```

> ★零 primitives：`BILLING_PERIOD_BLOCKS` 是订阅业务常量，声明在 square-post 模块内；不往 `primitives::pow_const` 加、也不必读它（7200 直接写死并注释口径）。

### 2.3 Config（完整关联类型）

```rust
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[pallet::config]
pub trait Config: frame_system::Config + votingengine::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type CitizenIdentity: SquarePostCitizenIdentityProvider<Self::AccountId>;   // 既有，发帖用
    type Currency: Currency<Self::AccountId>;                                    // 扣款/收款共用
    type InstitutionAccountQuery: entity_primitives::InstitutionMultisigQuery<Self::AccountId>;
    // ★第1部分不含以下两项（属税务，第2部分再加）；MembershipLevel/护栏/trait 均在 square-post 模块内，不进 primitives：
    // type IncomeLedger: IncomeLedgerWriter<Self::AccountId, BalanceOf<Self>>;  // 模块内 trait，第2部分税务 pallet 实现
    // type TimeProvider: UnixTime;                                              // 第2部分记账期年/月
    type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

    #[pallet::constant] type MaxSquarePostIdLen: Get<u32>;
    #[pallet::constant] type MaxSquareCidNumberLen: Get<u32>;
    #[pallet::constant] type MaxSquareStorageReceiptIdLen: Get<u32>;
    #[pallet::constant] type MaxCreatorTiers: Get<u32>;         // 单创作者档位上限
    #[pallet::constant] type MaxDueQueuePerBlock: Get<u32>;     // 单桶订阅笔数硬护栏
    #[pallet::constant] type MaxBillingPerBlock: Get<u32>;      // 单块扣款笔数上限
    type MaxBillingWeightPerBlock: Get<Weight>;                 // 扣款管线独立权重预算

    type WeightInfo: crate::weights::WeightInfo;
}
```

> `Config: + votingengine::Config` 必需：`propose_set_platform_price` 要经 `<T as votingengine::Config>::InternalAdminProvider::is_institution_admin` 校验发起人，逐字对照 grandpakey-change。

### 2.4 Storage（精确类型）

```rust
pub type CidNumberOf<T>       = BoundedVec<u8, <T as Config>::MaxSquareCidNumberLen>;
pub type IssuerKeyOf<T>       = subscription::IssuerKey<<T as frame_system::Config>::AccountId>;
pub type SubscriptionStateOf<T> = subscription::SubscriptionState<BlockNumberFor<T>>;
pub type SubKeyOf<T>         = (<T as frame_system::Config>::AccountId, IssuerKeyOf<T>);

// 既有（逻辑搬 post.rs，storage 仍在 lib.rs，append-only 不重排既有 variant）
#[pallet::storage] pub type SquarePosts<T> = StorageMap<_, Blake2_128Concat, PostIdOf<T>, SquarePostOf<T>, OptionQuery>;
#[pallet::storage] pub type PublishedPostCountByAccount<T> = StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

// 订阅关系：★审查CRITICAL① —— 单 hasher StorageMap 对元组 (AccountId,IssuerKey) 做一次 blake2_128_concat,
// 必须与 BFF storageMapKey('SquarePost','Subscriptions', subscriber++issuer) 逐字节一致(DoubleMap 双hasher 与 BFF 单hasher 永不匹配→confirm恒404/门禁恒402)。
#[pallet::storage] pub type Subscriptions<T> = StorageMap<
    _, Blake2_128Concat, SubKeyOf<T>, SubscriptionStateOf<T>, OptionQuery>;
// 按订阅者枚举「谁订阅了我」改由 BFF D1 镜像 square_creator_subscriptions 承担,链上不需 clear_prefix。

// 平台三档价（分），技术公司经 internal-vote 写
#[pallet::storage] pub type PlatformPrice<T> =
    StorageMap<_, Twox64Concat, primitives::membership_price::MembershipLevel, u128, OptionQuery>;

// 技术公司 CID 绑定；None=未注册→平台轨 fail-closed 挂起
#[pallet::storage] pub type PlatformCidNumber<T> = StorageValue<_, CidNumberOf<T>, OptionQuery>;

// 创作者档位；键=创作者钱包账户（非 CID）
#[pallet::storage] pub type CreatorPlans<T> =
    StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<subscription::CreatorTier, T::MaxCreatorTiers>, ValueQuery>;

// 到期区块分桶待扣款队列（有界），on_initialize 只弹当前桶
#[pallet::storage] pub type DueQueue<T> =
    StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, BoundedVec<SubKeyOf<T>, T::MaxDueQueuePerBlock>, ValueQuery>;

// 自动扣款游标，仿 votingengine::PendingExpiryBucket
#[pallet::storage] pub type PendingDueBucket<T> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;
```

**subscription.rs 类型**（价格字段统一裸 `u128` 分，只在 `transfer` 前一步 `saturated_into()`，杜绝二套价格类型漂移）：

```rust
pub enum IssuerKey<AccountId> { Platform, Creator(AccountId) }        // SCALE: tag(1B) [+32B AccountId]
pub enum SubscriptionPlan { Level(MembershipLevel), Tier(u8) }         // Platform用Level / Creator用Tier
pub enum SubscriptionStatus { Active, PastDue, Cancelled }  // ★审查HIGH⑤:补 Cancelled;cancel 写此态而非删记录,支持 Cancelled→resume 只翻 Active 不重扣
pub struct SubscriptionState<BlockNumber> {
    pub plan: SubscriptionPlan,
    pub price_fen: u128,             // 最近成功扣款快照，仅展示/审计，不作下次扣款依据
    pub next_charge_at: BlockNumber, // 下次扣款区块 = DueQueue 桶 key
    pub status: SubscriptionStatus,
}
pub struct CreatorTier { pub tier_code: u8, pub price_fen: u128 }      // 不存展示名
```

> ★关键不变量：`price_fen` **只是展示快照**。每次扣款永远从 `PlatformPrice`/`CreatorPlans` **现读现算**，绝不按 `price_fen` 收费——否则调价后老订阅永远按旧价扣，变成第二套价格真源。

### 2.5 Event / Error

```rust
#[pallet::event]
pub enum Event<T: Config> {
    // 既有 SquarePostPublished 保留…
    Subscribed { subscriber: T::AccountId, issuer: IssuerKeyOf<T>, plan: SubscriptionPlan },
    Charged { subscriber: T::AccountId, issuer: IssuerKeyOf<T>, amount: BalanceOf<T>, next_charge_at: BlockNumberFor<T> },
    ChargeFailed { subscriber: T::AccountId, issuer: IssuerKeyOf<T> },
    Cancelled { subscriber: T::AccountId, issuer: IssuerKeyOf<T> },
    CreatorPlansSet { creator: T::AccountId, tier_count: u32 },
    PlatformPriceProposed { proposal_id: u64, level: MembershipLevel, new_price_fen: u128, proposer: T::AccountId },
    PlatformPriceSet { level: MembershipLevel, price_fen: u128 },
}

#[pallet::error]
pub enum Error<T> {
    // 既有 EmptyPostId/FieldTooLong/… 保留append-only…
    CannotSubscribeSelf, SubscriptionNotFound, PlanIssuerMismatch,
    PlatformPriceNotSet, PlatformNotBound, CreatorTierNotFound,
    TooManyCreatorTiers, DuplicateTierCode, PriceOutOfRange,
    DueQueueBucketFull, NotPlatformInstitution, InvalidInstitution,
    UnauthorizedAdmin, ProposalActionNotFound,
}
```

### 2.6 extrinsic（lib.rs `#[pallet::call]`，逻辑体在对应文件）

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)] // 既有，零改动，函数体搬 post.rs::do_publish_post
    pub fn publish_post(origin, post_id, post_category, content_hash, storage_receipt_id, storage_until) -> DispatchResult { .. }

    // 公民热钱包标准 extrinsic + 生物识别；禁 op_tag、禁冷钱包。首扣与本调用原子完成
    #[pallet::call_index(1)]
    pub fn subscribe(origin, issuer: IssuerKeyOf<T>, plan: SubscriptionPlan) -> DispatchResult {
        let who = ensure_signed(origin)?; Self::do_subscribe(who, issuer, plan)
    }
    #[pallet::call_index(2)]
    pub fn cancel(origin, issuer: IssuerKeyOf<T>) -> DispatchResult {
        let who = ensure_signed(origin)?; Self::do_cancel(who, issuer)
    }
    // 创作者=签名者本人，任意钱包账户，无 CID 限制；覆盖式整表替换
    #[pallet::call_index(3)]
    pub fn set_creator_plans(origin, tiers: Vec<CreatorTier>) -> DispatchResult {
        let who = ensure_signed(origin)?; Self::do_set_creator_plans(who, tiers)
    }
    // 技术公司冷签 PasskeyColdSign + internal-vote；actor_cid_number 必逐字节==PlatformCidNumber
    #[pallet::call_index(4)]
    pub fn propose_set_platform_price(origin, actor_cid_number: Vec<u8>, level: MembershipLevel, new_price_fen: u128) -> DispatchResult {
        let who = ensure_signed(origin)?; Self::do_propose_set_platform_price(who, actor_cid_number, level, new_price_fen)
    }
}
```

**核心逻辑（subscription.rs）**：

```rust
pub(crate) fn do_subscribe(who, issuer, plan) -> DispatchResult {
    if let IssuerKey::Creator(ref c) = issuer { ensure!(c != &who, Error::CannotSubscribeSelf); }
    let now = frame_system::Pallet::<T>::block_number();
    // ★审查HIGH⑤:幂等分支,防重复 subscribe/双击/resume 立即双扣。
    if let Some(old) = Subscriptions::<T>::get(&who, &issuer) {
        // 同档 Active 未到期 → 幂等 no-op,直接返回不扣款
        if old.plan == plan && old.status == Active && old.next_charge_at > now { return Ok(()); }
        // 同档 Cancelled 未到期 → 只翻 Active 重入队,不重扣(续订不二次收费)
        if old.plan == plan && old.status == Cancelled && old.next_charge_at > now {
            Subscriptions::<T>::mutate(&who,&issuer,|m| if let Some(s)=m { s.status = Active; });
            Self::enqueue_due(old.next_charge_at, (who.clone(), issuer.clone()))?;
            Self::deposit_event(Event::Subscribed { subscriber: who, issuer, plan });
            return Ok(());
        }
        // 换档 / PastDue / 已过期 → 重新开单:先摘旧桶残留再立即首扣
        Self::remove_due_entry(old.next_charge_at, &who, &issuer);
    }
    let seed = SubscriptionState { plan, price_fen: 0, next_charge_at: now, status: Active };
    Self::try_charge_and_reschedule(&who, &issuer, &seed, now)?;         // 首扣=续扣共用原子函数
    Self::deposit_event(Event::Subscribed { subscriber: who, issuer, plan });
    Ok(())
}
// do_cancel:写 status=Cancelled + remove_due_entry(停止续扣),不删记录(保留到 next_charge_at 供 resume);已 Cancelled 幂等。

// 唯一定价/收款方解析入口，do_subscribe 与 billing 共用，杜绝两套定价漂移
pub(crate) fn resolve_price_and_payee(issuer, plan) -> Result<(u128, T::AccountId), DispatchError> {
    match (issuer, plan) {
        (Platform, Level(level)) => {
            let price = PlatformPrice::<T>::get(level).ok_or(Error::PlatformPriceNotSet)?;
            let cid = PlatformCidNumber::<T>::get().ok_or(Error::PlatformNotBound)?;
            let payee = T::InstitutionAccountQuery::lookup_institution_account(
                cid.as_slice(), primitives::account_derive::RESERVED_NAME_MAIN,
            ).ok_or(Error::PlatformNotBound)?;
            Ok((price, payee))
        }
        (Creator(creator), Tier(tier_code)) => {
            let tier = CreatorPlans::<T>::get(creator).iter()
                .find(|t| t.tier_code == *tier_code).cloned().ok_or(Error::CreatorTierNotFound)?;
            Ok((tier.price_fen, creator.clone()))   // 收款方=creator 本人钱包，全额转零折算
        }
        _ => Err(Error::PlanIssuerMismatch.into()),
    }
}
```

### 2.7 on_initialize 有界桶扫扣款（billing.rs，逐字复刻 votingengine::expiry）

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight { Self::process_due_bucket_scan(n) }
}

// 游标裁切：老桶没清完本块不抢跑到 n（等价 votingengine pending_has_remaining 分支）
pub(crate) fn process_due_bucket_scan(n) -> Weight {
    if T::MaxBillingPerBlock::get() == 0 { return Weight::zero(); }
    // 先处理 PendingDueBucket（若 <= n），未清完则 put 回并 return；清完 kill
    // 再处理当前块 n 的桶，未清完 put(n)
}

fn charge_due_bucket(bucket, now, max_count, max_weight) -> (usize, bool, Weight) {
    let mut items = DueQueue::<T>::take(bucket);
    // item_weight 预算裁切 → process_count；drain(..process_count) 逐笔：
    for (subscriber, issuer) in items.drain(..process_count) {
        let Some(state) = Subscriptions::<T>::get(&subscriber, &issuer) else { continue; }; // cancel已摘除
        if state.next_charge_at != bucket { continue; }                                       // 换档旧桶残留
        match Self::try_charge_and_reschedule(&subscriber, &issuer, &state, now) {
            Ok(()) => {}
            // ★审查HIGH④:区分失败种类,不把「调度容量耗尽」误判成「欠费」永久掉订阅。
            // 根治:enqueue_due 满桶时自动线性探测落到下一非满桶(有界步数),故扣款不因桶满 Err;
            //      charge 侧此处的 Err 只剩真实资金失败(余额<ED/InsufficientBalance)→ PastDue 欠费即停。
            Err(_) => {                                                                       // 真实欠费即停
                Subscriptions::<T>::mutate(&subscriber, &issuer, |m| { if let Some(s)=m { s.status = PastDue; } });
                Self::deposit_event(Event::ChargeFailed { subscriber, issuer });
                // 不重排 next_charge_at、不重新入队；用户须重新 subscribe 恢复 Active
            }
        }
    }
    let has_remaining = !items.is_empty();
    if has_remaining { DueQueue::<T>::insert(bucket, items); }  // 剩余 re-insert 回同桶
    (process_count, has_remaining, weight)
}

// 原子{解析定价/收款方→扣款→全额转→记台账→顺延周期→重新入队}；首扣/续扣唯一执行路径
pub(crate) fn try_charge_and_reschedule(subscriber, issuer, state, now) -> DispatchResult {
    frame_support::storage::with_storage_layer(|| {
        let (price_fen, payee) = Self::resolve_price_and_payee(issuer, &state.plan)?;
        let amount: BalanceOf<T> = price_fen.saturated_into();
        T::Currency::transfer(subscriber, &payee, amount, ExistenceRequirement::KeepAlive)?;  // 低于ED即拒
        let period = primitives::income_ledger::tax_period_at(T::TimeProvider::now());
        T::IncomeLedger::record_income(payee.clone(), period, amount)?;  // 第1/2期 () 恒 Ok，不破坏原子边界
        let next = now.saturating_add(primitives::pow_const::BLOCKS_PER_MONTH.saturated_into());
        Subscriptions::<T>::insert(subscriber, issuer, SubscriptionState {
            plan: state.plan, price_fen, next_charge_at: next, status: Active });
        Self::enqueue_due(next, (subscriber.clone(), issuer.clone()))?;
        Self::deposit_event(Event::Charged { subscriber: subscriber.clone(), issuer: issuer.clone(), amount, next_charge_at: next });
        Ok(())
    })
}
```

**失败语义（核心不变量）**：
- `KeepAlive` 承载“扣额使付款人低于 ED 即拒”；`resolve_price_and_payee` 找不到价格/档位/绑定返回 Err。
- 两类失败被 `with_storage_layer` 整体回滚（已转账/已插入 Subscriptions/DueQueue 全撤销）。
- `status=PastDue` 写在 `with_storage_layer` **之外**，不受回滚影响——欠费即停不重试、不重排。
- 首扣失败 Err 直接冒泡给 dispatch，不写任何状态，杜绝“已订阅未扣款”悬空态。

### 2.8 治理（proposal.rs，逐字照抄 grandpakey-change 范本）

```rust
pub const MODULE_TAG: &[u8] = b"sub-scr";
pub struct PlatformPriceUpdateAction { pub level: MembershipLevel, pub new_price_fen: u128 }

pub(crate) fn do_propose_set_platform_price(who, actor_cid_number, level, new_price_fen) -> DispatchResult {
    ensure!(new_price_fen >= PLATFORM_PRICE_MIN_FEN && new_price_fen <= PLATFORM_PRICE_MAX_FEN, Error::PriceOutOfRange);
    let bounded_cid = CidNumberOf::<T>::try_from(actor_cid_number.clone()).map_err(|_| Error::FieldTooLong)?;
    let platform_cid = PlatformCidNumber::<T>::get().ok_or(Error::PlatformNotBound)?;
    ensure!(bounded_cid == platform_cid, Error::NotPlatformInstitution);   // 绑定唯一CID非类别码
    let cid_text = core::str::from_utf8(&actor_cid_number).map_err(|_| Error::InvalidInstitution)?;
    let institution_code = votingengine::types::institution_code_from_cid_number(cid_text).ok_or(Error::InvalidInstitution)?;
    ensure!(<T as votingengine::Config>::InternalAdminProvider::is_institution_admin(institution_code, &actor_cid_number, &who), Error::UnauthorizedAdmin);

    let action = PlatformPriceUpdateAction { level, new_price_fen };
    let mut encoded = Vec::from(MODULE_TAG); encoded.extend_from_slice(&action.encode());
    let proposal_id = T::InternalVoteEngine::create_institution_proposal_with_data(
        who.clone(), institution_code, actor_cid_number.clone(), None,
        Vec::from([actor_cid_number]), MODULE_TAG, encoded)?;
    Self::deposit_event(Event::PlatformPriceProposed { proposal_id, level, new_price_fen, proposer: who });
    Ok(())
}

// InternalVoteExecutor 按 MODULE_TAG 互斥认领，approved 时纯 storage insert，始终 Executed
impl<T: pallet::Config> votingengine::InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(proposal_id, approved) -> Result<ProposalExecutionOutcome, DispatchError> {
        // 校验 is_proposal_owner + starts_with(MODULE_TAG)；approved 则 PlatformPrice::insert + PlatformPriceSet 事件
    }
    fn can_cancel_passed_proposal(_) -> Result<ProposalCancelDecision, DispatchError> { Ok(Ignored) }
}
```

### 2.9 GenesisConfig（重新创世，无 migration）

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub platform_prices: Vec<(MembershipLevel, u128)>,   // 默认取 PLATFORM_PRICE_DEFAULT_FEN
    pub platform_cid_number: Option<Vec<u8>>,            // 公司未注册留 None，平台轨挂起
}
// build 时逐档 assert 护栏区间后 insert；Some(cid) 则 put。
```

`node/src/core/chain_spec.rs`：`platform_cid_number: None`（技术公司注册后须整链重新创世补入）。`subscribe(Platform,_)` 与 `propose_set_platform_price` 在 `None` 时统一 `Error::PlatformNotBound` fail-closed。

### 2.10 IncomeLedger=() hook 与第3期切换

`configs.rs` 本期唯一改动点 `type IncomeLedger = ();`。调用点唯一：`try_charge_and_reschedule` 内 transfer 成功后、insert 前。第3期只需换成 `type IncomeLedger = tax_registry::Pallet<Runtime>;`，square-post 内部零改动（单向依赖 trait）。

> ★依据维度1/3 risk：`record_income` 的 payee 是 **AccountId 不是 CID**。第2期税务侧若按 CID 归集需自己做 account→CID 反查（有 CID 则记，无则视为自行申报跳过），trait 签名不改成 CID。

### 2.11 与 publish_post 共存（configs.rs 装配）

```rust
impl square_post::Config for Runtime {
    type CitizenIdentity = RuntimeSquarePostCitizenIdentity;  // 既有不变
    type Currency = Balances;
    type InstitutionAccountQuery = RuntimeInstitutionQuery;   // 复用既有聚合
    type IncomeLedger = ();
    type TimeProvider = pallet_timestamp::Pallet<Runtime>;
    type InternalVoteEngine = InternalVote;
    type MaxCreatorTiers = ConstU32<8>;
    type MaxDueQueuePerBlock = ConstU32<2_048>;
    type MaxBillingPerBlock = ConstU32<2_048>;
    type MaxBillingWeightPerBlock = votingengine::BlockWeightFraction<Runtime, 4>;
    // …其余既有 ConstU32 保留
}
```

- `InternalVoteResultCallback` 元组追加第 6 槽 `square_post::InternalVoteExecutor<Runtime>`。**⚠ 实现前须核实** `votingengine/src/traits/callbacks.rs` 宏支持 6 元组 arity。
- `RuntimeFeeRouter`：`publish_post` 分支不动（`signer_onchain_route(who,0)`）；新增 `subscribe|cancel|set_creator_plans => signer_onchain_route(who,0)`；`propose_set_platform_price{actor_cid_number,..} => institution_onchain_route(who, actor_cid_number.as_slice())`。第 762 行 `SquarePost(_) => Reject` **保持原样**（只兜底 `__Ignore`）。
- `RuntimeCallFilter::contains` 尾部 `_ => true` fail-open，**不需改动**。

---

## 三、App（citizenapp/lib）

依据：维度2 + 维度4。

### 3.1 前置：SCALE 工具收敛（唯一“顺手重构”）

新增 `citizenapp/lib/rpc/scale_bytes.dart`：把 `SquareChainService` 里 `@visibleForTesting` 的 `storageMapKey/blake2128Concat/writeCompactBytes/hexEncode/hexDecode/u64LittleEndian` 提升为正式公开静态工具，新增 `u128LittleEndian`，收敛 `transfer_rpc.dart` 私有 `_u128LittleEndian`。`SquareChainService/TransferRpc/SubscriptionRpc` 三处改 import 这一份，删私有重复实现（DRY）。

### 3.2 user.dart 创作者栏（★精确位置：会员卡之后、通讯录卡之前）

`user.dart:441-457` 间，会员卡（435-444）之后、通讯录卡（446）之前插入：

```dart
const SizedBox(height: 12),
Padding(padding: const EdgeInsets.symmetric(horizontal: 16),
  child: _buildEntryCard(
    leading: const Icon(Icons.storefront_outlined, color: AppTheme.info, size: 22),
    title: '创作者', onTap: _openCreator)),
```

`_openCreator` push `CreatorPage`，**不回读 `_loadState()`**（跟随“电子护照/设置”零 reload 惯例，创作者档位/收入与 MyTab 头部展示无关）。图标 `Icons.storefront_outlined`（不新增 SVG），色 `AppTheme.info`（未被占用，五入口色互斥）。

### 3.3 subscription_rpc.dart（传输层，新增）

```dart
class SubscriptionIssuer {
  const SubscriptionIssuer.platform() : this._(isCreator: false);
  const SubscriptionIssuer.creator(String creatorAccount) : this._(isCreator: true, creatorAccount: creatorAccount);
  final bool isCreator; final String? creatorAccount;
}
class SubscriptionRpc {
  static const int palletIndex = SquareChainService.palletIndex; // =34，单源，禁另开常量
  static const int subscribeCallIndex = 1;        // 【待拍板】
  static const int cancelCallIndex = 2;            // 【待拍板】
  static const int setCreatorPlansCallIndex = 3;   // 【待拍板】
  static const int maxCreatorTiers = 8;            // 客户端镜像 Config::MaxCreatorTiers，拍板后核对

  // SCALE 手工 ByteOutput 拼（不走 registry.encode，同 transfer_rpc 风格）：
  // subscribe = [34][1][issuer_tag][creator_account?32B][plan_variant_tag:1B][plan_value:1B]  ★plan=SubscriptionPlan枚举=2字节
  //             Platform→plan=[0x00,level] ; Creator→plan=[0x01,tier_code]
  // cancel     = [34][2][issuer_tag][creator_account?32B]
  // set_creator_plans = [34][3][compact(len)][ ([tier_code:1B][price_fen u128LE 16B])=17B × N ]  ★每档17字节含tier_code
  static void _writeIssuer(ByteOutput out, SubscriptionIssuer i) {
    if (!i.isCreator) { out.pushByte(0); return; }              // IssuerKey::Platform tag=0
    out.pushByte(1); out.write(Keyring().decodeAddress(i.creatorAccount!)); // Creator tag=1 + AccountId32
  }

  Future<({String txHash, int usedNonce, String blockHashHex})> subscribe({required String fromAddress, required Uint8List signerPubkey, required SubscriptionIssuer issuer, required int plan, required Future<Uint8List> Function(Uint8List) sign, TxPoolWatchCallback? onWatchEvent})
    => SignedExtrinsicBuilder(...).signAndSubmitInBlock(callData: buildSubscribeCall(issuer, plan), ...);
  // cancel / setCreatorPlans 同构

  Future<Uint8List?> fetchSubscriptionRaw({required String subscriberAddress, required SubscriptionIssuer issuer}); // 读 Subscriptions[(sub,issuer)]
  Future<Uint8List?> fetchCreatorPlansRaw(String creatorAddress); // 读 CreatorPlans[creator]，任意账户可读
}
```

> 编码字节序总表（★五端逐字节一致，审查CRITICAL②③已修正）：`IssuerKey`：`Platform=0x00`｜`Creator=0x01+32B AccountId`。`SubscriptionPlan`(枚举**2字节**)：Platform→`[0x00,level]`(Level变体)、Creator→`[0x01,tier_code]`(Tier变体)。`CreatorTier`(**17字节**)：`[tier_code:1B][price_fen:16B u128LE]`。`Vec<CreatorTier>=Compact<u32>len + 项×17B`。tier_code 由 App 连续赋号(0..N-1)且与 §六 tierIndex 同义。

### 3.4 subscription_service.dart（编排层，平台/创作者共用，新增）

```dart
class SubscriptionService {
  Future<SubscriptionResult> subscribe({required SubscriptionIssuer issuer, required int plan, TxPoolWatchCallback? onWatchEvent}) async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) throw const SubscriptionException('请先在「我的 → 我的钱包」创建热钱包');
    if (!wallet.isHotWallet) throw const SubscriptionException('公民币订阅仅支持热钱包');   // fail-fast，禁冷签双分支
    try {
      final result = await _rpc.subscribe(
        fromAddress: wallet.address, signerPubkey: hexDecode(wallet.pubkeyHex),
        issuer: issuer, plan: plan,
        sign: (payload) => _walletManager.signWithWallet(wallet.walletIndex, payload), // 硬件金库读seed触发一次生物识别
        onWatchEvent: onWatchEvent);
      await _confirm(issuer: issuer, txHash: result.txHash);
      return SubscriptionResult(txHash: result.txHash, blockHashHex: result.blockHashHex);
    } on SecureSeedException catch (e) { throw SubscriptionException(seedSignErrorMessage(e)); }  // ★三段式catch
      on WalletAuthException catch (e) { throw SubscriptionException(e.message); }
      on Exception catch (e) { throw SubscriptionException('订阅签名或提交失败：$e'); }
  }
  // cancel/setCreatorPlans 同构（setCreatorPlans 不调 _confirm，非计费关系）
  Future<void> _confirm({required SubscriptionIssuer issuer, required String txHash}) async {
    final session = await _sessionProvider.ensureSession(); if (session == null) return;
    try {
      await _apiClient.confirmCitizenCoinSubscription(session: session, txHash: txHash,
        issuerKind: issuer.isCreator ? 'creator' : 'platform', creatorAccount: issuer.creatorAccount);
    } on SquareApiException catch (e) { debugPrint('confirm失败(链上已成功,可重试): $e'); } // best-effort
  }
}
```

> ★依据 `project_wallet_sign_catch_secureseed.md`：`SecureSeedException` 是 sealed，与 `WalletAuthException` 无继承关系，漏写会导致生物识别取消/无锁屏**彻底静默**。文案单源 `seedSignErrorMessage(e)`。

### 3.5 membership_page.dart 平台轨改造

- 主动作从“打开官网”改为 App 内 `SubscriptionService.subscribe/cancel`；Stripe 卡轨降级为页面底部次要文字链接“或访问官网用银行卡订阅”（保留 `MembershipSiteConfig`/`_openMembershipSite`）。
- `SquareMembershipState`：删 `isPrepaid`；`subscriptionSource` 值域收窄 `citizen_coin|stripe|null`；新增 `bool get pastDue => subscriptionStatus == 'past_due';`。
- `_MembershipTierCard` 从 `onTapAction: VoidCallback` 改为 `onAction: ValueChanged<_SubscribeAction>`（四态：subscribe/cancel/resume/resubscribe）。subscribe/resume/resubscribe 统一走 `subscribe(Platform, levelIndex)`（链上 subscribe 是幂等“确保 Active”语义，见第六节不变量）。

### 3.6 my/creator 管理页（新增）

- `my/creator/creator_page.dart`：三卡（我的档位 / 谁订阅了我 / 我的公民币收入）。`_load` 并行 `fetchTiers(self) + fetchMyIncomeSummary`；`_openEditPlans` 返回后 `_load`。
- `my/creator/creator_plan_edit_page.dart`：每档 `¥价格/月` 输入框 + 删除，`+新增档位`（达 `maxCreatorTiers` 禁用），保存调 `SubscriptionService.setCreatorPlans(tierPriceFen)`，客户端护栏预校验 + 链上 validation 兜底。
- `my/creator/creator_plan_service.dart`：`fetchTiers(creatorAccount)` 链读 `CreatorPlans`（`CreatorTierView.label='创作者档 N'` 通用文案，无展示名表）；`fetchMyIncomeSummary/fetchMySubscribers` 走 BFF D1。
- 「我的公民币收入」文案必须标注“**预计月流水(MRR)**·实际结算台账将在税务功能上线后提供”（第1/2期非权威账本）。

### 3.7 8964 他人主页「订阅 TA」按钮（新增）

- `8964/profile/widgets/subscribe_button.dart`：`SubscribeCreatorButton(creatorAccount)`，初始化 `fetchTiers(target)` + 自身订阅态 `fetchSubscriptionRaw`；`_tiers.isEmpty` → `SizedBox.shrink()`（未开店主页不显示）；点击 bottomSheet 选档 → `subscribe(Creator(target), tierIndex)`。
- 与 `ProfileActionIcons` 的 🔔“订阅动态”（免费关注）是**两个功能，不复用同图标**；用 `Icons.workspace_premium_outlined`。
- `profile_header_card.dart` 加可选 `Widget? creatorSubscribeButton`（默认 null 向后兼容）；`user_profile_page.dart` 调用处 `creatorSubscribeButton: widget.isSelf ? null : SubscribeCreatorButton(creatorAccount: widget.ownerAccount)`。

### 3.8 热钱包签名闭合时序（★submit-only vs inBlock 选择）

维度2 与维度4 对提交策略存在两种主张，**采纳维度4（`signAndSubmitInBlock`）**——因为订阅是创建类交易需拿 `blockHashHex` 供 confirm，与 `square_publish_service` 范本同构；失败核实责任仍交 Worker 独立链读，App 不做同步阻塞 `ExtrinsicFailed` 判定。

```
UI选档 → SubscriptionService.subscribe(issuer,plan)
  → getDefaultWallet()（非热钱包 fail-fast）
  → _buildSubscribeCall 拼 [34][1][issuer][plan]
  → SignedExtrinsicBuilder.signAndSubmitInBlock:
        fetchMetadata ∥ fetchGenesisHash
        → 并行 fetchRuntimeVersion + fetchNonce（nonce 绝不本地缓存/自增）
        → buildImmortalSigningPayload
        → sign(payload)=signWithWallet（硬件金库读seed，一次生物识别）→64B sr25519
        → buildImmortalExtrinsicPayload.encode(registry, sr25519)
        → ChainRpc.submitExtrinsic（拿 txHash + blockHashHex）
  → _confirm：unawaited POST /v1/square/membership/citizen-coin/confirm {tx_hash, issuer_kind, creator_account?}
  → Worker state_getStorage 链读 Subscriptions 核实 + D1 幂等镜像
  → MembershipStatusNotifier.refresh 回刷 UI（绝不在提交时本地乐观置位）
```

> ★依据 `feedback-unawaited-bg-sync-needs-completion-refresh.md`：unawaited confirm 完成后必回刷 UI，兜底值不入缓存。生物识别不是独立 `local_auth`，而是 `HardwareBoundSeedVault` 读 seed 的硬件原子解锁（`wallet_manager.dart:126`），不要再叠一层 local_auth。

---

## 四、BFF（citizenapp/cloudflare/src/membership）

依据：维度3。

### 4.1 citizen_coin.ts（新增，链读确认 + D1 幂等镜像）

**storage key 构造**：`Subscriptions` 是 `StorageMap<(AccountId, IssuerKey), _, Blake2_128Concat>`（单 hasher 覆盖整段元组编码），与既有 `storageMapKey(pallet, storage, keyData)` 对齐。

```ts
const SQUARE_POST_PALLET_NAME = 'SquarePost';   // ★construct_runtime 类型名（lib.rs:400），非字符串化标识
const SUBSCRIPTIONS_STORAGE_NAME = 'Subscriptions';
type IssuerKeyInput = { kind: 'platform' } | { kind: 'creator'; creatorAccount: string };

function encodeIssuerKey(i: IssuerKeyInput): Uint8Array {  // Platform=[0x00]; Creator=[0x01,...32B]
  return i.kind === 'platform' ? new Uint8Array([0]) : concat(new Uint8Array([1]), decodeOwnerAccount(i.creatorAccount));
}
export function subscriptionStorageKey(subscriber: string, issuer: IssuerKeyInput): Uint8Array {
  return storageMapKey(SQUARE_POST_PALLET_NAME, SUBSCRIPTIONS_STORAGE_NAME,
    concat(decodeOwnerAccount(subscriber), encodeIssuerKey(issuer)));
}
```

**SubscriptionState 定长解码（22 字节，长度不符抛 502）**：

```ts
// [level_or_tier:u8][price_fen:u128LE 16B][next_charge_at:u32LE 4B][status:u8] = 22B
function decodeSubscriptionState(bytes: Uint8Array): OnchainSubscriptionState {
  if (bytes.length !== 22) throw new HttpError(502, 'citizen_coin_decode_failed', ...);
  return { level_or_tier: bytes[0], price_fen: readU128Le(bytes,1), next_charge_at: readU32Le(bytes,17),
           status: bytes[21] === 1 ? 'past_due' : 'active' };
}
export async function fetchOnchainSubscription(env, subscriber, issuer): Promise<OnchainSubscriptionState | null>;
```

**citizenCoinConfirmRoute**（★owner 来自 `requireSession` 派生，body 不携带 owner，仿 `posts/confirm.ts`）：

```
1. session = requireSession(request, env)
2. assertTxHash(/^0x[0-9a-f]{64}$/) + assertIssuerInput(issuer_kind/creator_account)
3. 始终活读链 fetchOnchainSubscription(session.owner_account, issuer)；null → 404（刚广播未上链，客户端重试轮询）
4. recordCitizenCoinPaymentOnce：SELECT tx_hash
     命中 owner≠session → 409 tx_hash_owner_mismatch（防冒领，relay.ts 无此校验因无权益语义）
     命中 issuer 不一致 → 409 tx_hash_issuer_mismatch
     命中一致 → granted=false 跳过；未命中 → INSERT（主键竞态回读兜底）
5. issuer=platform → citizenCoinMembershipStatement(...).run() 覆盖刷新 square_memberships 镜像
   issuer=creator  → upsertCreatorSubscription 写 square_creator_subscriptions（不写 memberships）
6. 返回解码后最新链上状态
```

> ★关键设计（对任务描述的必要精化）：**payments 台账一次性幂等** 与 **memberships 镜像每次覆盖刷新** 拆成两个独立操作。原因：链上按月扣款失败翻 PastDue，Worker 无任何 webhook 得知；若镜像死绑 tx_hash 首现，D1 会在欠费停权后仍长期显示 active。拆开后 App 可用同一 tx_hash 反复 confirm（前台唤起/进页前）低成本刷新镜像，不违反台账幂等。

### 4.2 requireCreatorSubscription 门禁（fail-closed，不缓存，实时链读）

```ts
// 物理位置 citizen_coin.ts，由 service.ts 具名重导出（避免顶层模块环）
export async function requireCreatorSubscription(env, subscriberAccount, creatorAccount): Promise<void> {
  // fetchOnchainSubscription(creator issuer)；RPC失败→502；未订阅/PastDue/null→402；active才resolve
}
```

与 `requireActiveMembership`（读 D1 缓存镜像）语义**刻意不同**：创作者门禁挂“解锁专属内容”一次性授权，ADR-037 §5② 要求实时链上真源，失败拒绝而非软降级（与 `identity.ts` 读链失败降级为访客相反，不可复用后者）。

### 4.3 subscriptionIsActive 分支

```ts
export function subscriptionIsActive(m: MembershipRow): boolean {
  if (m.subscription_source === 'citizen_coin') return m.subscription_status === 'active'; // 只看status不看expires_at
  const s = m.subscription_status || 'active';
  return (s === 'active' || s === 'trialing') && m.expires_at > nowMs();
}
```

citizen_coin 无到期日概念（欠费即停无宽限），`expires_at/next_charge_block` 仅展示排序，生效只信 `status`。

### 4.4 D1 表

```sql
-- 一次性支付台账，tx_hash 主键幂等
CREATE TABLE square_citizen_coin_payments (
  tx_hash TEXT PRIMARY KEY CHECK(length(tx_hash)=66 AND tx_hash GLOB '0x*' AND substr(tx_hash,3) NOT GLOB '*[^0-9a-f]*'),
  owner_account TEXT NOT NULL, issuer_kind TEXT NOT NULL CHECK(issuer_kind IN ('platform','creator')),
  creator_account TEXT, membership_level TEXT, tier_code INTEGER,
  price_fen INTEGER NOT NULL, next_charge_block INTEGER NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active','past_due')), granted_at INTEGER NOT NULL);
CREATE INDEX idx_ccp_owner ON square_citizen_coin_payments(owner_account, granted_at);
CREATE INDEX idx_ccp_creator ON square_citizen_coin_payments(creator_account, granted_at) WHERE creator_account IS NOT NULL;

-- 创作者当前态镜像（活跃集合，非流水；供"谁订阅我/收入"）
CREATE TABLE square_creator_subscriptions (
  subscriber_account TEXT NOT NULL, creator_account TEXT NOT NULL,
  tier INTEGER NOT NULL, price_fen INTEGER NOT NULL, status TEXT NOT NULL,
  granted_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, last_tx_hash TEXT NOT NULL,
  PRIMARY KEY (subscriber_account, creator_account));
CREATE INDEX idx_scs_creator ON square_creator_subscriptions(creator_account, status);

-- square_memberships 改：删 prepaid_payment_ref；subscription_source 加 CHECK IN('stripe','citizen_coin')；加 next_charge_block INTEGER
-- 删整表 square_stripe_payments 及其索引
```

### 4.5 删 USDC 残桩清单（归零判据 `grep -rn 'prepaid\|usdc' cloudflare/src` = 0）

| 文件 | 动作 |
|---|---|
| `membership/prepaid.ts` | 整文件删（423行 USDC Stripe-Crypto Checkout） |
| `membership/webhook.ts` | 删 `checkout.session.completed` 分派 + `processPrepaidCheckout` + `prepaidMonthsFromMeta` + `allowPrepaidSwitch` metadata |
| `membership/service.ts` | 删 `upsertPrepaidMembership`/`applyPrepaidTierChange`/`stripePaymentAlreadyGranted`/`stripePaymentInsert`；`upsertStripeMembership` 去 `WHERE...OR ?=1` + `allowPrepaidSwitch` 形参改无条件 UPSERT；`subscriptionIsActive` 删 usdc_prepaid 分支换 citizen_coin |
| `membership/subscribe.ts` | 删 `trialEndSeconds` 整块 + `createStripeCheckoutSession` 的 trial/payment_switch metadata |
| `account/service.ts` | `cancelMembershipRoute` 删 usdc_prepaid 分支，换 `citizen_coin → 409 citizen_coin_cancel_onchain_only`（引导 App 链上 cancel，BFF 无签名能力代管） |
| `types.ts` | `MembershipRow` 删 `prepaid_payment_ref`，`subscription_source: 'stripe'\|'citizen_coin'`，加 `next_charge_block`；新增 `CitizenCoinConfirmBody`/`CitizenCoinPaymentRow` |
| `routes.ts` | 删 4 个 prepaid 路由，注册 `POST /v1/square/membership/citizen-coin/confirm`、`GET creator/subscribers`、`GET creator/income` |
| `limits/catalog.ts` | `(subscribe\|cancel\|prepaid)` → `(subscribe\|cancel)`；删 `/prepaid/change` 行；加 `citizen-coin/confirm`(api_json_small) + creator 两路由 |
| `migrations/0001_square_core.sql` | 按 4.4 改表 |

> `asset_balance_tile.dart` 的 "USDC" 是资产币种符号，不在 cloudflare/src 内，**勿误删**。

---

## 五、onchina 调价冷签 + CitizenWallet 三处登记 + 官网

依据：维度5。范式=**legislation propose_enact_law 单轮冷签**（membership-tax.md 闭合2 只有一轮 QR，非 CitizenOnchainPush 双轮 grant）。

### 5.1 onchina domains/membership（新建域，仿 domains/address 四文件）

- `chain_call.rs`：`SQUARE_POST_PALLET_INDEX=34` / `CALL_PROPOSE_SET_PLATFORM_PRICE=4`；`build_set_platform_price_chain_call(actor_cid_number, input)` 编码 `[34,4] + push_vec(actor_cid_number) + level(1B) + price_fen(u128 LE 16B)`；`level_tag`(freedom=0/democracy=1/spark=2) **必补逐字节 golden 测试**对齐链端 `MembershipLevel.encode()`。
- `handler.rs`：`prepare_set_platform_price` = `require_admin_any(ctx)` → `chain_identity::active_registry_cid_number(&state)` 取 actor_cid_number → `capability::can_set_platform_price(actor_cid_number)`（非技术公司 403）→ `build_set_platform_price_chain_call` → `core::qr::build_sign_request_bytes`（TTL=120s，actor_pubkey=ctx.admin_account）。路由 `POST /api/v1/admin/membership/chain-call`。

### 5.2 auth/operation_auth.rs 四 arm（唯一强制改动，穷尽 match 漏改即编译失败）

1. enum 变体 `SetPlatformPrice`
2. `as_str` → `"SET_PLATFORM_PRICE"`
3. `auth_type` 并入 `PasskeyColdSign` 的 `|` 链
4. `parse_action_type` → `Ok(SetPlatformPrice)`

`is_governance()/requires_governing_capability()` 是 `matches!` 非穷尽，不加即默认 false（正确）。补一条仿 legislation 的往返单测。

### 5.3 platform/capability.rs（显式偏离类型码范式，H1）

```rust
const TECH_COMPANY_CID_ENV: &str = "ONCHINA_TECH_COMPANY_CID";
/// H1：类型码=提权。技术公司唯一法人，必锁精确 CID，不进 CapabilitySet/capabilities_for。
pub(crate) fn can_set_platform_price(actor_cid_number: &str) -> bool {
    let Ok(cid) = std::env::var(TECH_COMPANY_CID_ENV) else { return false; };  // 未配置 fail-closed
    !cid.trim().is_empty() && cid.trim().eq_ignore_ascii_case(actor_cid_number.trim())
}
```

### 5.4 CitizenWallet 三处登记（仅 propose_set_platform_price 冷签需要）

1. `pallet_registry.dart`：`squarePostPallet=34` / `proposeSetPlatformPriceCall=4`。
2. `payload_decoder.dart`：加路由 + `_decodeProposeSetPlatformPrice`（复用既有 `_readCidNumber/_readU128Le/_hasValidSigningTail/_fenToYuan`），格式 `[0x22][0x04][actor_cid_number][level:u8][price_fen:u128LE]`，`levelNames=['自由会员','民主会员','薪火会员']`，越界/尾无效 → null → decodeFailed 红拒。
3. `qr_protocols.dart`：`proposeSetPlatformPrice = 0x2204`（chain(34,4)）+ `fromDecodedAction` 加 `'propose_set_platform_price' => proposeSetPlatformPrice`。

> subscribe/cancel/set_creator_plans 是 App 热签**永不进 CitizenWallet**。

### 5.5 官网 Membership.tsx

删加密预付轨（`PrepaidDuration`/`prepaidTotalCents`/`parsePrepaidChangePreview`/`usdc-purchase`/`usdc-change` 等，`ActionKind`/`SigningKind` 收窄为 `'card-subscribe'|'cancel'`，删时长选择器 JSX + `prepaid_tier_change_required` 重试逻辑），保留 Stripe 卡轨闭环。新增 `components/CitizenCoinHandle.tsx`（照 `DownloadButton.tsx` 零签名独立引导组件，引导“打开 CitizenApp → 会员 → 公民币订阅”），零 fetch/零签名依赖。

### 5.6 调价闭环时序（闭合2 唯一权威）

```
onchina 表单 {level, price_fen} → POST /api/v1/admin/membership/chain-call
  → require_admin_any → active_registry_cid_number → can_set_platform_price(唯一CID精确匹配,非则403)
  → build_set_platform_price_chain_call([34,4]+cid+level+price_fen) → build_sign_request_bytes(QR_V1/k=1,TTL120s)
  → 渲染QR → CitizenWallet 扫码 → payload_decoder 命中34/4逐字段解 + qr_protocols 0x2204交叉核对(两色)
  → 核对无误冷签(signer∈技术公司链上Active管理员集,链端校验)
  → chain_submit propose_set_platform_price
  → 链上 internal-vote 技术公司多签表决 → InternalVoteResultCallback 按 MODULE_TAG=b"sub-scr" 认领 → PlatformPrice::insert
  → onchina/App/Worker 只读回链 PlatformPrice 镜像展示
```

---

## 六、端到端闭合（逐跳经手文件 + 不变量）

### ① 订阅平台会员

```
membership_page 选档
 → subscription_service.subscribe(Platform, levelIndex)
 → subscription_rpc._buildSubscribeCall [34][1][0x00][level]
 → wallet_manager.signWithWallet（生物识别）
 → signed_extrinsic_builder.signAndSubmitInBlock → chain_rpc.submitExtrinsic
 → 链上 do_subscribe → resolve_price_and_payee(PlatformPrice[level]+PlatformCidNumber派生OP_MAIN)
   → try_charge_and_reschedule(with_storage_layer: transfer→record_income()→insert Subscriptions{next=now+7200,Active}→enqueue_due)
 → subscription_service._confirm → square_api_client.confirmCitizenCoinSubscription
 → citizen_coin.ts citizenCoinConfirmRoute → fetchOnchainSubscription 链读核实
   → recordCitizenCoinPaymentOnce(tx_hash幂等) → citizenCoinMembershipStatement 覆盖 square_memberships
 → MembershipStatusNotifier.refresh 回刷
后续每月：on_initialize → process_due_bucket_scan → charge_due_bucket(bucket) → try_charge_and_reschedule
   成功顺延重入队；失败整笔回滚 + status=PastDue（不重试不重排）
```

**不变量**：链上 subscribe 是幂等“确保 Active”语义——无记录开新单（立即扣款）；同档已有记录且 Cancelled 且未到期则只翻 Active（不重复扣款，“续订”不二次收费）；PastDue 或换档视为重新开单（立即扣款）。App 只管调同一 subscribe，不关心分支。

### ② 订阅创作者会员

```
创作者 creator_plan_edit_page._save
 → subscription_service.setCreatorPlans(tierPriceFen)
 → subscription_rpc [34][3][compact(len)][price_fen×N] → 生物识别 → 链上 CreatorPlans[self]=BoundedVec（覆盖式，无CID校验）
订阅者 他人主页 subscribe_button
 → fetchTiers(target) 链读 CreatorPlans[target] 非空渲染按钮
 → subscribe(Creator(target), tierIndex)
 → resolve_price_and_payee(CreatorPlans[creator] 找 tier) → try_charge_and_reschedule 全额转 creator_account（签名账户本身）
 → confirm(issuer_kind='creator') → upsertCreatorSubscription 写 square_creator_subscriptions
按月：billing 管线同 ① 续扣，全额转创作者
门禁解锁：requireCreatorSubscription 每次实时链读 Subscriptions[(subscriber,Creator(creator))] active 才放行
创作者查看：creator_page → GET creator/subscribers、creator/income（按 creator_account=session.owner_account 聚合 D1）
```

**不变量**：`subscribe(plan=tierIndex)` 的 tierIndex 是订阅发起时刻 CreatorPlans 的下标；链上 validation 须校验 `tierIndex < len`（无法防“下标含义变了”，故 App 用最新链读现算 + 编辑优先改价而非删中间档重排）。CreatorTier 只存 price_fen，展示名全链下。

---

## 七、统一命名（★逐字一致项，五端对照）

| 维度 | 值 | ★强一致端 |
|---|---|---|
| pallet_index | **34**（复用 square-post，零新增） | runtime lib.rs / configs.rs 费率路由 / onchina SQUARE_POST_PALLET_INDEX / CitizenWallet squarePostPallet / BFF `'SquarePost'` |
| pallet 类型名 | `SquarePost`（construct_runtime） | BFF storageMapKey 前缀源 |
| call_index | 0=publish_post 1=subscribe 2=cancel 3=set_creator_plans 4=propose_set_platform_price【待拍板】 | Rust `#[pallet::call_index]` / App subscription_rpc / onchina chain_call / CitizenWallet payload_decoder |
| storage | `Subscriptions` `PlatformPrice` `PlatformCidNumber` `CreatorPlans` `DueQueue` `PendingDueBucket` | 链上 + BFF SUBSCRIPTIONS_STORAGE_NAME |
| MODULE_TAG | `b"sub-scr"` | proposal.rs / InternalVoteExecutor 认领 |
| MembershipLevel | Freedom=0 Democracy=1 Spark=2 | 链 primitives / App tierOrder / onchina level_tag / CitizenWallet levelNames |
| 默认价(分) | 199900 / 599900 / 5999900 | primitives PLATFORM_PRICE_DEFAULT_FEN |
| IssuerKey SCALE | Platform=0x00 / Creator=0x01+32B | 链 / App _writeIssuer / BFF encodeIssuerKey / CitizenWallet |
| BLOCKS_PER_MONTH | 7200 (=BLOCKS_PER_DAY×30) | pow_const.rs |
| QR 动作码 | 0x2204 = chain(34,4) | qr_protocols proposeSetPlatformPrice |
| 环境变量 | `ONCHINA_TECH_COMPANY_CID` | onchina capability |
| HTTP 路由 | `POST /v1/square/membership/citizen-coin/confirm`、`GET .../creator/{subscribers,income}`、`POST /api/v1/admin/membership/chain-call` | routes.ts / main.rs / limits catalog |
| D1 表 | `square_citizen_coin_payments`、`square_creator_subscriptions`、`square_memberships`(改) | migrations |
| Dart 文件 | `rpc/subscription_rpc.dart` `rpc/scale_bytes.dart` `my/membership/subscription_service.dart` `my/creator/{creator_page,creator_plan_edit_page,creator_plan_service}.dart` `8964/profile/widgets/subscribe_button.dart` | App |

---

## 八、分阶段与验收

### 阶段 A — 平台会员

链：primitives(membership_price/income_ledger/BLOCKS_PER_MONTH) → subscription.rs 类型 + Subscriptions/PlatformPrice/PlatformCidNumber → subscribe/cancel + billing 桶扫 + GenesisConfig + configs 装配 + 费率路由。
BFF：citizen_coin.ts confirm(platform) + subscriptionIsActive 分支 + citizenCoinMembershipStatement + 删 USDC + D1 表。
App：scale_bytes 收敛 → subscription_rpc(subscribe/cancel) → subscription_service → membership_page 改造。
onchina：domains/membership + operation_auth 四arm + capability + CitizenWallet 三处登记 + Membership.tsx。

**验收 A**：
- [ ] 选 Spark 档热签 → 链上 Subscriptions[(self,Platform)] Active，扣 5999900 分转技术公司 OP_MAIN。
- [ ] confirm 后 GET /membership 显示 active；同 tx_hash 重放 confirm 不重复入账、刷新镜像。
- [ ] 余额不足扣款整笔回滚，status=PastDue，无悬空态。
- [ ] on_initialize 到期桶自动续扣，桶满 DueQueueBucketFull。
- [ ] onchina 冷签调价 → CitizenWallet 两色核对 → PlatformPrice 更新；非技术公司 CID 403。
- [ ] `grep -rn 'prepaid\|usdc' cloudflare/src` = 0。
- [ ] primitives/tests/fixtures 金标向量：IssuerKey/SubscriptionState/CreatorTier/propose call SCALE 五端逐字节断言。

### 阶段 B — 创作者会员

链：set_creator_plans + CreatorPlans + resolve_price_and_payee 的 Creator 分支（复用 billing）。
BFF：confirm(creator) + upsertCreatorSubscription + requireCreatorSubscription + creator/subscribers、creator/income 路由。
App：my/creator 三页 + subscribe_button + profile_header_card 插槽。

**验收 B**：
- [ ] set_creator_plans 覆盖式写 CreatorPlans[self]，无 CID 校验，任意钱包账户可开。
- [ ] 他人主页有档才显示“订阅 TA”，订阅后全额转创作者钱包本人账户。
- [ ] requireCreatorSubscription active 放行、PastDue/未订阅 402、RPC 失败 502（fail-closed，不缓存）。
- [ ] creator_page 显示订阅数/预计 MRR（标注“预计·非最终结算”）。
- [ ] IncomeLedger hook 空实现记 0，不影响扣款原子性。

---

## 关键风险与拍板点（实现前必须解决）

1. **【拍板点】call_index 具体号**（1/2/3/4 为推荐值，任务卡列为未定稿）——五端漂移即冷签红拒或静默错扣，落地前先拍板。
2. **【拍板点】护栏常量** `PLATFORM_PRICE_MIN/MAX_FEN`、`CREATOR_PRICE_MIN/MAX_FEN`、`MaxCreatorTiers`——App 侧当前为客户端镜像占位，链上 validation 兜底。
3. **【必核实】InternalVoteResultCallback 6元组 arity** — `votingengine/src/traits/callbacks.rs` 宏须支持第 6 槽，否则编译报 trait not implemented。
4. **【必测】MembershipLevel/SubscriptionState/CreatorTier 逐字节 golden 向量** — 定长解码无自描述边界，长度断言不能替代金标测试。
5. **【文档漂移】ADR-037 原文 IssuerKey=Creator(cid)** 已被任务卡/prompt 覆盖为 `Creator(AccountId)`，ADR 正文未回写——以任务卡为准，勿反向改回 CID 键。ADR-037/membership-tax.md 的 pallet_index=35 是并入前残留，**实际 34**。
6. **【运维】D1 镜像 staleness** — 链上自动续扣无 webhook，欠费翻 PastDue 后 D1 可能滞后；缓解=App 主动重放 confirm + 可选 Worker cron 批量回链核对（第1部分不实现，需 App 任务卡显式列出）。
7. **【安全】`ONCHINA_TECH_COMPANY_CID` 部署前必配且保密**，建议生产启动校验存在且格式合法否则拒绝启动（fail-closed）。
8. **【安全洼地】单轮直发 vs 双轮 grant** — 遵照闭合2选 legislation 单轮冷签，安全强度弱于走 auth/actions.rs 两步 grant 的 PasskeyColdSign 动作；调价是资金相关高危，建议二次确认是否补 `require_admin_security_grant`（技术零成本，preview_action_conn 通配已兼容）。