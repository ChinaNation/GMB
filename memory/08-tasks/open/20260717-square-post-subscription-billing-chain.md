# 任务卡：square-post 会员订阅链端（公民币单轨 · 分步实施）

> 状态：**第1、2、4步链端完成并验证**（2026-07-17）。`cargo test -p square-post` **24 用例全绿**；`cargo check -p citizenchain` 全 runtime 编译通过、无告警。**第3步=平台改价治理，推迟到技术公司在链上中国注册后再做**（技术公司 CID 由注册流程生成并绑 `PlatformCidNumber`）。权重基准=开发期占位、后续 polish。
> 架构真源 `memory/01-architecture/gmb/subscription-part1-tech.md` **旧版描述 on_initialize桶扫/双轨/genesis/链上创作者档，与本卡 as-built 冲突——以本卡为准**（真源待整体改写单轨版）。

## 需求（定稿）
`square-post`（会员订阅**唯一模块**，零新增 pallet）实现公民币**单轨**订阅：彻底删银行卡、无兼容双轨。
- **平台会员**（自由/民主/薪火）：定义+价格在链上，扣公民币→技术公司**费用账户**（OP_FEE），按月自动扣。
- **创作者会员**（任意钱包账户）：档定义（名称/档种类≤10/月季年周期/价格）**全部链下**（App 本地设 + Cloudflare 存），链上只做订阅付款；**创作者必须是有效平台会员才能被订阅（链上强制）**。
- runtime 只做「钱的流动」：`subscribe`(首扣)/`cancel`/`charge_due`(续扣)；**不做调度/日历/到期判断**（Cloudflare 读国储会 RPC 链时间戳算日历触发；权益计算在设备）。

## as-built 链上设计（第1+2步，已验证）
**文件**：`square-post/src/{lib,subscription,billing,weights}.rs` ＋ `tests/mod.rs`；`runtime/src/configs.rs`。**不碰 node/chain_spec、无 genesis、无价格护栏、无周期常量、无链上创作者档**。

**类型（`subscription.rs`，零 primitives）**
```
enum MembershipLevel { Freedom=0, Democracy=1, Spark=2 }
enum IssuerKey<AccountId> { Platform, Creator(AccountId) }        // SCALE tag1B[+32B]
enum SubscriptionPlan { Level(MembershipLevel), CreatorPrice(u128) } // 平台带档位/创作者带价
enum SubscriptionStatus { Active, PastDue, Cancelled }
struct SubscriptionState { plan, price_fen:u128, last_charged_at:u64/*unix ms 原始戳*/, status }
```
**Storage**：`Subscriptions:(AccountId,IssuerKey)->State`（平台+创作者共用）/ `PlatformPrice:Level->u128` / `PlatformCidNumber:Value<CidNumber>` / `BillingKeeper:Value<AccountId>`。全部从空开始。**无 CreatorPlans**（创作者档全链下）。

**Calls**：`0 publish_post`(既有) `1 subscribe(issuer,plan)`[热] `2 cancel(issuer)`[热] `4 charge_due(subscriber,issuer,amount:Option<u128>)`[**仅 BillingKeeper**·Free]。`3 set_creator_plans`(**永不上链**，创作者设档全链下) / `5 propose_set_platform_price`(第3步) 预留。

**逻辑**
- `subscribe`：平台→读 `PlatformPrice[level]`+费用账户；创作者→订阅者签名授权当前价 `CreatorPrice(a)`（a>0）、收款方=创作者全额、**门禁 `ensure 创作者是 Active 平台会员 else CreatorNotPlatformMember`**、不能订自己。幂等（同档 Active no-op / Cancelled 翻 Active 不重扣）。
- `charge_due`（仅 keeper）：平台→`amount=None` 现读 `PlatformPrice`；创作者→`amount=Some(当前价)` keeper 带入（**改价后续扣走新价**），`None`→`CreatorPriceRequired`、平台传 `Some`→`PlanIssuerMismatch`。失败→回滚外写 `PastDue`（欠费即停）。
- `try_charge`：首扣=续扣共用 `with_storage_layer` 原子路径；`transfer(KeepAlive)`。首扣失败冒泡不写状态。

**configs.rs**：`Config` 装 `Currency=Balances`/`TimeProvider=crate::Timestamp`/`InstitutionAccountQuery=RuntimeInstitutionQuery`；费率路由 `subscribe|cancel→signer_onchain`、`charge_due→FeeRoute::Free`。回调元组未改（无治理）。

## 死规则（贯穿后续）
零 primitives 业务类型；零兼容/迁移；不碰 node/chain_spec；收款方=费用账户 OP_FEE；链上不算日历/周期/到期；`charge_due` 仅 keeper；平台价链上真源现读、创作者价链下真源(keeper带)；创作者门禁链上强制；创作者档定义全链下；`call_index 0/1/2/4` + SCALE 布局锁死。

## 验收（第1+2步已达成）
- [x] 平台：订阅首扣转费用账户、幂等不双扣、价未设/CID未绑 fail-closed、首扣失败零状态、续扣现读价、余额不足 PastDue、`charge_due(Platform,Some)` 拒。
- [x] 创作者：非平台会员创作者被订阅→拒；平台会员创作者被订阅→全额转其钱包；`charge_due(Creator,Some(新价))` 走新价；`None`→拒；订阅自己→拒；`a=0`→拒；余额不足→PastDue。
- [x] 发帖回归不破；无新增 pallet；`cargo test -p square-post`=23 passed；`cargo check -p citizenchain` 绿；无 `CreatorPlans/CreatorTier/MaxCreatorTiers` 残留。

## 第4步 金标 SCALE 向量（已完成）
- fixture：`runtime/misc/square-post/tests/fixtures/subscription_scale_vectors.json`（`pallet_index=34` + 18 条向量，五端逐字节对齐**唯一真源**）。
- 回填/校验：`SUBSCRIPTION_VECTORS_UPDATE=1 cargo test -p square-post subscription_scale_vectors` 重算写回；默认 `cargo test` 断言链端编码==fixture，类型/call 布局漂移即红。
- 字节规范（五端契约）：`pallet=34=0x22`；call `subscribe=1/cancel=2/charge_due=4`；`IssuerKey Platform=00 / Creator=01+32B`；`Plan Level(l)=00+l / CreatorPrice(a)=01+u128LE`；`State=plan‖price_fen(u128LE)‖last_charged_at(u64LE)‖status`；`Option<u128> None=00 / Some=01+16B`。例：订平台薪火 calldata=`2201000002`。

## 下一步
- **第3步 平台改价治理（推迟）**：待技术公司注册后做——`propose_set_platform_price`(call_index 5) + `InternalVoteExecutor`（MODULE_TAG）+ configs 回调元组第6槽（**先核实 votingengine 6-arity**）+ `Config: +votingengine::Config`/`InternalVoteEngine` + institution 费率路由。
- 文档：真源 `subscription-part1-tech.md` 整体改写单轨＋charge_due keeper＋创作者链下版（跨所有端，另立文档任务）。
- 下游卡（可照金标向量并行开工）：Cloudflare（自动续费 Cron＋权益＋删 Stripe＋创作者档存储/≤10校验/平台会员门禁）、App（订阅编排＋创作者本地设档）、onchina（改价冷签）、CitizenWallet（登记）、官网（退化）。

影响范围：`citizenchain`（square-post 扩展 ＋ configs 装配）。
