# 任务卡：第1步 · square-post 会员订阅自动扣费（链端最小实现）

> 状态：**开工授权**（用户 2026-07-17 允许建卡）。架构真源＝`memory/01-architecture/gmb/subscription-part1-tech.md`（单轨版，待本卡同步改写）＋`membership-tax.md`＋ADR-037。本卡**只做链端**；Cloudflare 自动续费调度 / 权益计算、App 订阅编排、onchina 改价、CitizenWallet 登记、官网退化各自后续卡。

任务需求：
在现有 `square-post`（会员订阅**唯一模块**，零新增 pallet）内实现两类订阅的**公民币自动扣费核**：
- **平台会员**（自由/民主/薪火三档）：扣公民币 → 技术公司**费用账户**（OP_FEE）。
- **创作者会员**（创作者=任意钱包账户自设档价）：扣公民币 → 创作者本人钱包**全额**。
- 结算货币**唯一公民币**，彻底无银行卡、无兼容、无双轨。
- **runtime 只做"扣费本体"**：一次被触发即原子完成的 `charge_due` + 首扣 `subscribe`；**不做调度扫描**（自动续费的"何时扣"与权益计算全在 Cloudflare/软件端）。

所属模块：`citizenchain/runtime/misc/square-post`（扩展）＋`runtime/src/configs.rs`（装配）。

输入文档：
- memory/01-architecture/gmb/subscription-part1-tech.md（架构真源，单轨改写中）
- memory/01-architecture/gmb/membership-tax.md（分层/单向解耦）
- memory/01-architecture/citizenchain/ORACLE_TECHNICAL.md（国储会=首期唯一 Worker 链上游）
- ADR-037（会员业务模型）
- citizenchain/runtime/misc/square-post/src/lib.rs（发帖基线，publish_post=call_index 0）
- citizenchain/runtime/primitives/src/account_derive.rs（RESERVED_NAME_FEE / OP_FEE=0x01 已核实）

必须遵守（死规则）：
- **零 primitives 业务类型**：`MembershipLevel`/护栏常量/周期常量全在 `square-post` 模块内声明，绝不进 `primitives`（作废任务卡草案 §39 的 `primitives/membership_price.rs`）。
- **零兼容、零残留、统一切换**：不搞过渡/迁移/spec_version；链开发期一次改到位。
- **收款方=技术公司费用账户**：平台轨 payee＝`PlatformCidNumber` 派生 `RESERVED_NAME_FEE`（不是主账户，纠正真源文档原 `RESERVED_NAME_MAIN`）。
- **时间权威只在链上**：`charge_due` 到期门必用 `pallet_timestamp::now()` 重校验 `now >= next_charge_at`；**绝不**用调用者传入或 RPC 读数给转账放行。国储会 RPC 时间戳只是 Cloudflare 调度的触发时钟（下游卡），链是最终裁决。
- **`charge_due` 无权限但安全**：收款方现读不可改向、金额现读价不可抬高、未到期即拒不可提前/重复；调用者只能触发已到期扣款。走 Free 费轨免手续费。
- **定价现读现算**：`price_fen` 只是展示快照，扣款永远现读 `PlatformPrice`/`CreatorPlans`，绝不做第二价源。
- **平台/创作者共用一套**引擎（`IssuerKey` 区分），不分叉两套。
- **五端字节锁死**：`call_index` + SCALE 布局本卡定稿后不改（`IssuerKey`/`SubscriptionPlan`/`SubscriptionState`/`CreatorTier` 逐字节，供 App/BFF/CitizenWallet 对齐）。
- 不清楚先沟通。

---

## 链上定稿设计

### 类型（`subscription.rs` 模块内，零 primitives）
```
enum MembershipLevel { Freedom=0, Democracy=1, Spark=2 }
enum IssuerKey<AccountId> { Platform, Creator(AccountId) }        // SCALE tag1B [+32B]
enum SubscriptionPlan { Level(MembershipLevel), Tier(u8) }        // Platform用Level / Creator用Tier
enum SubscriptionStatus { Active, PastDue, Cancelled }
struct SubscriptionState<Moment> { plan, price_fen:u128, next_charge_at:u64(unix ms), status }
struct CreatorTier { tier_code:u8, price_fen:u128 }               // 17B含tier_code, 不存展示名
// 常量: PLATFORM_PRICE_MIN/MAX_FEN, CREATOR_PRICE_MIN/MAX_FEN, MaxCreatorTiers, BILLING_PERIOD_MS(30天)
// 默认价(分): Freedom 199900 / Democracy 599900 / Spark 5999900【拍板确认】
```

### Storage
```
Subscriptions:     StorageMap<(AccountId, IssuerKey), SubscriptionState, OptionQuery>  // 单 Blake2_128Concat 元组键(对齐BFF)
PlatformPrice:     StorageMap<MembershipLevel, u128, OptionQuery>                       // 技术公司经internal-vote写
PlatformCidNumber: StorageValue<CidNumber, OptionQuery>                                 // 创世绑技术公司CID→派生费用账户
CreatorPlans:      StorageMap<AccountId, BoundedVec<CreatorTier, MaxCreatorTiers>, ValueQuery>
// 删: DueQueue / PendingDueBucket（调度移到 Cloudflare）
```

### Calls（`#[pallet::call]`，逻辑体在对应文件）
```
0 publish_post(..)                                   既有零改动
1 subscribe(issuer, plan)              [热·生物识别]  建订阅 + 首扣即时(try_charge)
2 cancel(issuer)                       [热]           写 Cancelled, 不删记录(支持resume), 幂等
3 set_creator_plans(tiers)             [热]           创作者覆盖式写自己档价, 无CID限制, 护栏校验
4 charge_due(subscriber, issuer)       [keeper·Free]  续扣: 到期门+原子扣款; 未到期NotDueYet拒; 失败PastDue
5 propose_set_platform_price(actor_cid, level, price_fen) [冷·internal-vote]  技术公司改平台价
```

### 扣费核（`billing.rs`，首扣=续扣共用；无 on_initialize）
```
try_charge(subscriber, issuer, now) -> DispatchResult:      // subscribe首扣 & charge_due续扣唯一路径
  with_storage_layer:
    (price_fen, payee) = resolve_price_and_payee(issuer, plan)      // 现读现算
      Platform      → PlatformPrice[level] + PlatformCidNumber 派生 RESERVED_NAME_FEE 费用账户
      Creator(acct) → CreatorPlans[acct] 找 tier.price_fen; payee = acct 本人全额
    Currency::transfer(subscriber → payee, price_fen, KeepAlive)    // 低于ED即拒
    next = now + BILLING_PERIOD_MS
    Subscriptions.insert{ price_fen快照, next_charge_at:next, Active }
    Event::Charged{..}
charge_due 门: ensure Timestamp::now() >= state.next_charge_at, else NotDueYet(no-op拒)
失败语义: transfer失败 → with_storage_layer整笔回滚; charge_due侧写 status=PastDue(回滚外,欠费即停不重试);
          subscribe首扣失败 → 冒泡dispatch,不写任何状态(杜绝已订阅未扣款悬空态)
```

### 治理（`proposal.rs`，链上只做授权写值）
`MODULE_TAG=b"sub-scr"` → `propose_set_platform_price` 校验发起人∈技术公司管理员（`InternalAdminProvider::is_institution_admin`）→ `InternalVoteEngine` 建机构提案 → 表决通过 → `InternalVoteExecutor` 按 MODULE_TAG 认领 → `PlatformPrice::insert`。

### configs.rs 装配
- `impl square_post::Config`：`+Currency=Balances` `+TimeProvider=Timestamp`（到期门）`+InstitutionAccountQuery`（派生费用账户）`+InternalVoteEngine=InternalVote` `+votingengine::Config` 超trait ＋护栏常量。
- **回调元组第 6 槽**：`configs.rs:2311` 现 5 槽 `(A,(B,C),(D,E),F,G)` → 追加 `square_post::InternalVoteExecutor`；**开工先核实 votingengine 宏支持 6-arity**，否则嵌套 pair 规避。
- **费率路由**（`configs.rs:762` `SquarePost(_)=>Reject` 拆开）：`subscribe|cancel|set_creator_plans → signer_onchain_route`；`charge_due → Free`；`propose_set_platform_price → institution_onchain_route`；`publish_post` 不变。
- `RuntimeCallFilter`：`charge_due` 显式放行（无权限可入块）。

---

输出物：
- 代码：`subscription.rs`/`billing.rs`/`proposal.rs` 新增 ＋ `lib.rs` 聚合（post 逻辑抽 `post.rs`）＋ configs 装配 ＋ chain_spec 创世（PlatformPrice 三档 ＋ PlatformCidNumber）。
- 中文注释：轻量、口径说明（BILLING_PERIOD_MS/费用账户/到期门权威）。
- 测试：`tests/{mod,cases,billing}.rs`——首扣/续扣/未到期拒/欠费PastDue/幂等cancel-resume/换档/护栏/治理写价；**金标 SCALE 向量**（IssuerKey/SubscriptionPlan/SubscriptionState/CreatorTier/propose call 逐字节，供五端对齐）。
- 文档更新：`subscription-part1-tech.md` 同步改写为**单轨 + charge_due keeper 模型**（删 on_initialize/DueQueue 段）。
- 残留清理：不涉及银行卡（那在 BFF 卡）；本卡确保 publish_post 回归不破。

验收标准：
- 选薪火档热签 `subscribe(Platform,Spark)` → `Subscriptions[(self,Platform)]` Active，扣 5999900 分转技术公司**费用账户**。
- `charge_due` 未到期 → `NotDueYet` 拒；到期 → 续扣成功顺延 `next_charge_at`；余额不足 → 整笔回滚 ＋ `PastDue`，无悬空态。
- `set_creator_plans` 覆盖式写 `CreatorPlans[self]`，任意钱包账户可开；`subscribe(Creator(x),tier)` 全额转 x。
- `propose_set_platform_price` 经 internal-vote 通过后 `PlatformPrice` 更新；非技术公司管理员拒。
- `publish_post` 回归不破；**无新增 pallet**；金标 SCALE 向量单测通过。
- `cargo build` / `cargo test -p square-post` 绿；clippy 无警告。

拍板点（开工即定）：
1. **`call_index`**：`1 subscribe / 2 cancel / 3 set_creator_plans / 4 charge_due / 5 propose_set_platform_price`（0 既有）——五端同批锁死，允许永久留洞。
2. **护栏常量值**：`PLATFORM_PRICE_MIN/MAX_FEN`、`CREATOR_PRICE_MIN/MAX_FEN`、`MaxCreatorTiers`、默认三档价。
3. **回调元组 6-arity**：实现首步核实。

影响范围：`citizenchain`（square-post 扩展 ＋ configs ＋ chain_spec）。链改须用户显式确认后动手。**后续卡**：Cloudflare（自动续费 Cron ＋ 权益计算 ＋ 删 Stripe 全套）、App（订阅编排 ＋ 创作者页）、onchina（改价冷签）、CitizenWallet（三处登记）、官网（退化引导）。
