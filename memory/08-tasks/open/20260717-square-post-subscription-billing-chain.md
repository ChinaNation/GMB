# 任务卡：第1步 · square-post 会员订阅自动扣费（链端·平台会员资金核心）

> 状态：**第1步链端完成并通过验证**（2026-07-17）。`cargo test -p square-post` 15 用例全绿；`cargo check -p citizenchain` 全 runtime 编译通过。创作者写入=第2步、平台改价治理=第3步、金标向量/权重基准=第4步。
> 架构真源＝`memory/01-architecture/gmb/subscription-part1-tech.md`（**旧版仍描述 on_initialize 桶扫/双轨/genesis，与本卡 as-built 冲突，以本卡为准，真源待整体改写单轨版**）。

## 需求（定稿）
在现有 `square-post`（会员订阅**唯一模块**，零新增 pallet）实现公民币**单轨**订阅自动扣费核：
- **平台会员**（自由/民主/薪火三档）：扣公民币 → 技术公司**费用账户**（OP_FEE）。
- **创作者会员**（任意钱包账户自设档价）：扣公民币 → 创作者本人钱包**全额**（第2步接入写入）。
- 唯一公民币结算，**彻底删银行卡、无兼容、无双轨**。
- **runtime 只做「钱的流动」**：`subscribe`（首扣）/`cancel`/`charge_due`（续扣）；**不做任何调度/日历/到期判断**。
  - 自动续费"何时扣"＝ Cloudflare（读国储会 RPC 链时间戳算日历，触发 `charge_due`）。
  - 权益计算 ＝ CitizenApp 本设备（读 Cloudflare 喂的链数据自行算年月日/到期）。

## as-built 链上设计（实际落地，已验证）
**文件**：`square-post/src/{lib,subscription,billing,weights}.rs` ＋ `tests/mod.rs`；`runtime/src/configs.rs`（Config 装配＋费率路由）。**不碰 node/chain_spec、无 genesis、无价格护栏、无周期常量**。

**类型（`subscription.rs`，零 primitives）**
```
enum MembershipLevel { Freedom=0, Democracy=1, Spark=2 }
enum IssuerKey<AccountId> { Platform, Creator(AccountId) }       // SCALE tag1B[+32B]
enum SubscriptionPlan { Level(MembershipLevel), Tier(u8) }
enum SubscriptionStatus { Active, PastDue, Cancelled }
struct SubscriptionState { plan, price_fen:u128, last_charged_at:u64/*unix ms 原始戳*/, status }
struct CreatorTier { tier_code:u8, price_fen:u128 }              // 第1步定义、第2步写入
```
**Storage**：`Subscriptions:(AccountId,IssuerKey)->State` / `PlatformPrice:Level->u128` / `PlatformCidNumber:Value<CidNumber>` / `CreatorPlans:AccountId->BoundedVec<CreatorTier,MaxCreatorTiers=16>` / `BillingKeeper:Value<AccountId>`。全部从空开始，无 genesis。

**Calls**：`0 publish_post`(既有) `1 subscribe(issuer,plan)`[热] `2 cancel(issuer)`[热] `4 charge_due(subscriber,issuer)`[**仅 BillingKeeper**·Free 费轨]。`3 set_creator_plans` / `5 propose_set_platform_price` 预留。

**扣款核（`billing.rs`）**：`try_charge`（`with_storage_layer` 原子，首扣=续扣共用）→ `resolve_price_and_payee`（Platform→`PlatformPrice`+`PlatformCidNumber` 派生 `RESERVED_NAME_FEE` 费用账户；Creator→本人全额）→ `transfer(KeepAlive)` → 写 `{price_fen, last_charged_at:now, Active}`。`charge_due` 收到 keeper 触发即扣一次，**链上零到期判断**；扣款失败→回滚外写 `PastDue`（欠费即停，返回 Ok）。首扣失败冒泡不写状态。

**configs.rs**：`Config` 装 `Currency=Balances`/`TimeProvider=crate::Timestamp`/`InstitutionAccountQuery=RuntimeInstitutionQuery`/`MaxCreatorTiers=ConstU32<16>`；费率路由 `subscribe|cancel→signer_onchain_route(who,0)`、`charge_due→FeeRoute::Free`。`RuntimeCallFilter` 尾 `_=>true` 放行热签，无需改。回调元组**本步未改**（无治理）。

## 死规则（贯穿后续步）
- 零 primitives 业务类型；零兼容/迁移/spec_version；不碰 node/chain_spec。
- 收款方＝技术公司**费用账户**（OP_FEE），非主账户（纠真源原 `RESERVED_NAME_MAIN`）。
- 链上**不做日历/周期/到期计算**；`last_charged_at` 只是给本机读的原始戳。
- `charge_due` **仅 keeper 可调**（`BillingKeeper`）；收款方/金额现读不可改；欠费即停。
- 定价现读现算，`price_fen` 只是展示快照，绝不第二价源。
- 平台/创作者共用一套引擎（`IssuerKey` 区分）。
- 五端字节：`call_index 0/1/2/4` + SCALE 类型布局本步锁死，App/BFF/CitizenWallet 对齐。

## 验收（已达成）
- [x] `subscribe(Platform,Spark)` 首扣 5999900 分转费用账户；订阅态 Active＋`last_charged_at`。
- [x] 幂等重订不双扣；`cancel` 保记录翻 Cancelled；续订翻 Active 不重扣。
- [x] 价未设 `PlatformPriceNotSet`、CID 未绑 `PlatformNotBound`、首扣失败零状态。
- [x] `charge_due` 非 keeper `NotBillingKeeper`；keeper 续扣转账；余额不足 `PastDue` 无转账。
- [x] `publish_post` 回归不破；无新增 pallet；`cargo test -p square-post`=15 passed；`cargo check -p citizenchain` 绿。

## 下一步
- **第2步 创作者会员**：`set_creator_plans`(call_index 3) + `CreatorPlans` 写入 + `subscribe(Creator)` 复用扣费引擎 + 用例。
- 第3步 平台改价治理（`propose_set_platform_price` call_index 5 + `InternalVoteExecutor` + votingengine 6-arity 核实）。
- 第4步 金标 SCALE 向量 + 权重基准。
- 文档：真源 `subscription-part1-tech.md` 整体改写为单轨＋charge_due keeper 版（跨所有端，另立文档任务）。

影响范围：`citizenchain`（square-post 扩展 ＋ configs 装配）。下游 Cloudflare（自动续费 Cron＋权益＋删 Stripe）/App/onchina/CitizenWallet/官网各自后续卡。
