# 公民币平台订阅与创作者订阅技术架构

> 状态：统一目标架构
> 任务卡：`memory/08-tasks/open/20260716-citizen-coin-subscription.md`
> 决策：`memory/04-decisions/ADR-037-citizen-coin-native-membership.md`

## 1. 目标

平台订阅和创作者订阅统一使用公民币付款，并复用现有 `SquarePost` pallet。订阅期限和自动扣款都以区块唯一的共识 unix 毫秒时间戳为依据；runtime 使用确定性的 UTC 公历算法计算月、季、年，绝不使用区块高度、固定天数或固定毫秒替代真实日期。

现有链和全部无关状态必须保留。禁止重新创世、替换 chainspec、清空链数据库、恢复旧订阅格式或建立双轨兼容。

## 2. 系统边界

| 能力 | 唯一负责方 | 边界 |
|---|---|---|
| 平台价格、创作者付款套餐 | CitizenChain | 每次真实扣款读取当前链上价格 |
| 扣款、收款、订阅状态 | CitizenChain | 首扣和到期自动扣款均在 runtime 原子执行 |
| 真实公历到期时间 | CitizenChain | 从当前区块共识时间戳确定性计算 |
| 自动续费调度 | CitizenChain | 到期后按时间顺序处理；停链期间到期周期在恢复出块后补扣 |
| 创作者展示资料 | Cloudflare/D1 | 只保存名称、说明、权益和媒体资料 |
| finalized 订阅镜像 | Cloudflare/D1 | 只做低频缓存、展示和门禁加速，可从链重建 |
| 平台调价 | 统一投票引擎 | `SquarePost` 只创建提案并接收终态回调 |

CitizenApp 对订阅、取消、换套餐以及创作者覆盖设置自己的套餐分别只签名一次，并展示链上时间戳。Cloudflare 不计算日期、不发起扣款、不扫描全链订阅、不决定价格或到期时间；App、设备和 Cloudflare 是否在线均不影响续费。

## 3. 链上模块

- 路径：`citizenchain/runtime/misc/square-post/`
- pallet：`SquarePost`
- pallet index：`34 / 0x22`
- 发帖和订阅共享 pallet，但类型、storage、扣款、迁移和治理代码分文件维护。
- 不新建订阅 pallet；使用有界到期索引和确定性整数公历算法，不引入外部日历服务。

## 4. 类型与 SCALE 契约

### 4.1 基础类型

```rust
enum MembershipLevel { Freedom = 0, Democracy = 1, Spark = 2 }
enum IssuerKey<AccountId> { Platform = 0, Creator(AccountId) = 1 }
enum BillingPeriod { Monthly = 0, Quarterly = 1, Yearly = 2 }
```

- 平台计划固定使用 `Monthly`。
- 创作者计划可使用月、季、年。
- `BillingPeriod` 表达业务公历周期；runtime 分别增加一个月、三个月或十二个月，并在目标月份没有原日期时使用该月最后一个有效日期，保留 UTC 时分秒和毫秒。

### 4.2 创作者付款套餐

```rust
struct PeriodPrice {
    billing_period: BillingPeriod,
    price_fen: u128,
}

struct CreatorTier {
    tier_id: TierId,
    prices_fen: PeriodPrices,
}
```

- `tier_id` 在同一创作者下唯一。
- 每档至少包含一个不重复周期，价格必须大于零。
- 链上不保存档位名称、说明、权益文案或媒体。

### 4.3 订阅计划与状态

```rust
enum SubscriptionPlan {
    Platform { membership_level: MembershipLevel },
    Creator { tier_id: TierId, billing_period: BillingPeriod },
}

enum SubscriptionStatus {
    Active = 0,
    Cancelled = 1,
    Terminated = 2,
}

struct SubscriptionState {
    plan: SubscriptionPlan,
    pending_plan: Option<SubscriptionPlan>,
    started_at: u64,
    last_charged_at: u64,
    last_charged_price_fen: u128,
    paid_until: u64,
    subscription_status: SubscriptionStatus,
}
```

所有时间字段均为 unix 毫秒时间戳。字段顺序即固定 SCALE 顺序，Dart、TypeScript、JSON 金标和 runtime 必须逐字节一致。

`Active` 表示授权仍有效并已进入自动续费索引；`Cancelled` 表示用户已签名取消并保留当前已付权益；`Terminated` 表示到期扣款失败或目标套餐失效，自动续费已永久停止。

## 5. Storage

```rust
Subscriptions<(AccountId, IssuerKey)> -> SubscriptionState
PlatformPrice<MembershipLevel> -> u128
PlatformCidNumber -> Option<CidNumber>
CreatorPlans<AccountId> -> CreatorTiers
RenewalSchedule<(due_at_be, AccountId, IssuerKey)> -> ()
RenewalIndex<(AccountId, IssuerKey)> -> due_at
MigrationBlocked -> bool
```

`RenewalSchedule` 用大端时间戳键保持到期顺序，`RenewalIndex` 保证每个订阅只有一个当前到期项。不保存下次扣款区块、外部续费账户、设备状态、链下扣款密钥或第二份展示套餐。

## 6. Call 契约

| index | 调用 | 签名者 |
|---:|---|---|
| `0` | `publish_post(...)` | 发帖账户 |
| `1` | `subscribe(issuer, plan, expected_price_fen)` | 订阅者 |
| `2` | `cancel(issuer)` | 订阅者 |
| `3` | `set_creator_plans(tiers)` | 创作者 |
| `4` | `change_subscription_plan(issuer, new_plan, expected_price_fen)` | 订阅者 |
| `5` | `propose_set_platform_price(actor_cid_number, membership_level, new_price_fen)` | 技术公司治理账户 |
旧 call、旧 SCALE tag 和旧交易载荷不兼容；不存在外部 `renew` 或周期确认 call。

## 7. 首次订阅与自动调度

1. CitizenApp 从 finalized storage 读取当前价格。
2. CitizenApp 提交 `subscribe`；runtime 重新读取当前价格并校验 `expected_price_fen`。
3. runtime 在同一 storage layer 完成转账，以当前区块唯一 `Timestamp.Now` 写入 `started_at`、`last_charged_at` 和审计价格。
4. runtime 按 UTC 真实公历计算 `paid_until`，写入 `Active`，并把该到期时间登记到调度索引。
5. CitizenApp 等待交易 finalized 后读取并显示链上 `started_at` 与 `paid_until`；不再提交第二笔确认交易。

平台收款账户从真实 `PlatformCidNumber` 派生。平台 CID 或费用账户缺失时 fail-closed。创作者订阅款全额进入创作者钱包，不能订阅自己，且创作者必须在扣款时拥有有效平台订阅。

## 8. runtime 自动续费

1. `on_initialize` 只预留本块最大处理权重；`on_finalize` 在 Timestamp inherent 写入后读取本块共识时间戳。
2. runtime 从最早到期项开始，处理所有 `due_at <= now` 的 Active 订阅，单块处理量受 `MaxSubscriptionRenewalsPerBlock` 约束。
3. 每个周期扣款均读取当时最新链上价格，成功后从该周期原到期时间增加一个真实公历周期并更新索引。
4. 停链期间无法发生状态变更；恢复出块后按到期顺序补扣所有已到期周期，未完成部分在后续区块继续。
5. 任一周期余额不足、转账失败、平台收款账户失效、创作者资格失效或套餐失效时，写 `Terminated` 并移除调度，不重试。

续费不需要账户再次签名，不依赖 CitizenApp、设备或 Cloudflare 在线，也不存在任何外部续费提交者。

## 9. 扣款失败、取消和换套餐

- 真实转账失败：写 `Terminated`，不延长权益、不重试。
- 创作者资格失效、档位删除或周期删除：不扣款，停止续费并写 `Terminated`。
- 取消：写 `Cancelled`，保留已经确认的 `paid_until`，到期前已付权益仍有效。
- 未到期换套餐：写 `pending_plan`，当前已付周期不变。
- 已取消或已终止后换套餐：作为新的签名授权，按目标计划当前价格立即扣款并重新进入 `Active` 调度。
- 不退款、不补差价、不按日折算。

## 10. 创作者套餐与平台调价

- `set_creator_plans` 覆盖式写入创作者自己的链上付款字段。
- 新订阅和下一次真实续费读取最新价格；当前已付周期不变。
- 创作者在 CitizenApp 同一次业务提交中填写档位标识、名称和周期价格，并只签名一次 `set_creator_plans` 交易；finalized 后 Cloudflare 接收交易哈希并严格读取链上付款字段，仅保存档位名称等展示资料。
- Cloudflare 展示资料必须引用 finalized 的 `creator_account + tier_id`，不得保存第二份扣款真源价格；finalized 后的镜像只用 Bearer 会话和链读复核，不生成设备请求签名。边缘保存失败只能重试镜像 HTTP，不得再次签名或重复提交链上交易。
- `propose_set_platform_price` 只调用统一内部投票引擎；人口快照、资格、计票和状态推进不进入业务 pallet。

## 11. 信任边界

订阅授权由订阅者第一次签名建立，并持续有效到订阅者签名取消。公历到期时间、自动扣款和状态变更由 runtime 唯一决定；CitizenApp 与 Cloudflare 都不能伪造、延长或触发续费。共识时间戳仍受区块生产与 Timestamp pallet 规则约束，但在同一链状态下所有节点执行完全相同的整数公历结果。

## 12. Cloudflare/D1

- 只严格解码 finalized `Subscriptions` 并镜像链上字段。
- confirm 只做链读核实后的快速镜像，不信任请求中的价格、状态或到期值，也不代表周期确认。
- 低频对账用于纠偏，不进行高频全表扫描。
- 门禁使用链上 `paid_until` 与当前链时间戳比较；状态未知、镜像过期或链读失败时 fail-closed。
- 不计算公历、不提交续费、不持有扣款密钥、不保存第二份价格真源。
- 同一业务操作的链上交易 finalized 后，所有镜像与重试都不再产生账户签名或设备请求签名。

## 13. 原地 runtime 升级

### pre-upgrade

- 在正式链数据库副本读取当前 runtime 和 SquarePost StorageVersion。
- 断言旧 `Subscriptions`、退役到期索引和相关订阅前缀为空。
- 快照帖子、发布计数、账户、余额、总发行量、平台价格和平台 CID。
- 发现任何旧订阅数据即阻断，不转换、不删除。

### on-runtime-upgrade

- 仅在旧订阅相关前缀全部为空时执行。
- 精确清理退役 keeper 单值。
- 仅对缺失的平台价格做缺省回填，已有值不覆盖。
- 保留平台 CID；缺失时继续 fail-closed。
- 写入目标 StorageVersion，并清除迁移阻断标志。

### post-upgrade

- 校验目标 StorageVersion、空订阅状态和未阻断状态。
- 校验帖子、发布计数、账户、总发行量、平台 CID 与升级前一致。
- 校验平台价格仅按“缺失回填、已有保留”变化。

正式升级前必须在链数据库副本完成 TryRuntime pre/post；不得把编译通过等同于升级验收。

## 14. CitizenApp 与签名体验

- 订阅、取消、换套餐和创作者设置套餐各自使用一笔热钱包标准 extrinsic，并等待 finalized；同一业务操作不得追加第二次账户签名，自动续费没有用户交易或签名。
- 第三步已经接入完整页面流程、finalized 状态读取、真实日期展示和创作者一次签名后边缘镜像重试，不实现续费编排。
- 页面显示使用 `DateTime.fromMillisecondsSinceEpoch(...).toLocal()` 展示真实日期和时间，不显示区块高度或“固定天数”。
- Cloudflare 暂时不可用时，App 仍以 finalized 链上价格、档位和订阅状态工作；展示名称可使用本地兜底。

## 14.1 交易收费

- `subscribe`、`cancel`、`change_subscription_plan`、`set_creator_plans` 都是账户签名的非系统链上交易，统一经过 runtime 交易支付扩展和签名账户收费路由。
- 业务转账金额为零时仍收取最低链上交易费；不得把取消订阅或只改状态误判成免费操作。
- runtime 到期自动扣款在区块执行阶段内部运行，不是外部交易，因此不追加用户交易费。

## 15. 真实验收

- runtime 单元测试、金标 SCALE、benchmark 编译、完整 runtime 测试和 WASM 构建通过。
- 在真实链数据库副本确认旧 StorageVersion、订阅相关前缀为空及迁移 pre/post 不变量。
- runtime 覆盖月末、闰年、跨年、季和年周期计算、自动续费、停链后补扣和余额不足终止。
- Cloudflare 严格解码新状态，拒绝尾随字节和非法标签，且不包含任何日期计算。
- 后续步骤必须完成真机、真实本地链、真实 Worker/D1/HTTP 的端到端验收。

## 16. 禁止事项

- 禁止用区块高度、固定天数或固定毫秒表示订阅周期。
- 禁止在 CitizenApp 或 Cloudflare 计算并提交订阅到期时间。
- 禁止外部 renew、周期确认、设备签名或 keeper。
- 禁止跳过停链期间已经到期的周期或恢复旧订阅协议。
- 禁止把 D1 镜像当作订阅真源。
- 禁止在业务模块实现投票流程。
- 禁止保留旧字段、旧注释、旧 UI 文案或兼容分支。
