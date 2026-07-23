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
    Suspended = 3,
    CreatorPaused = 4,
}

enum SuspendReason {
    NeedReconsent = 0,
    InsufficientBalance = 1,
}

struct SubscriptionState {
    plan: SubscriptionPlan,
    started_at: u64,
    last_charged_at: u64,
    last_charged_price_fen: u128,
    paid_until: u64,
    subscription_status: SubscriptionStatus,
    authorized_price_fen: u128,
    suspend_reason: Option<SuspendReason>,
}
```

所有时间字段均为 unix 毫秒时间戳。字段顺序即固定 SCALE 顺序，Dart、TypeScript、JSON 金标和 runtime 必须逐字节一致。

- `Active`：授权有效且在续费调度内。
- `Cancelled`：用户签名取消，保留当前已付权益至 `paid_until`。
- `Suspended`：暂停续扣、保留粉丝关系、退出调度，等用户动作恢复；`suspend_reason` 说明原因（创作者改价待再签名 / 余额不足待充值再签）。
- `CreatorPaused`：创作者掉平台会员，粉丝暂停扣费但**仍留调度**，创作者恢复即自动续。
- `Terminated`：仅显式关闭/清档，不由余额不足或掉会员自动进入。
- `authorized_price_fen`：订阅者已授权用于自动续费的价格（创作者改价重签检测与换挡折算基准）。

## 5. Storage

```rust
Subscriptions<(AccountId, IssuerKey)> -> SubscriptionState
PlatformPrice<MembershipLevel> -> u128        // 创世 genesis_build 播种，仅内部投票可改
CreatorPlans<AccountId> -> CreatorTiers
RenewalSchedule<(due_at_be, AccountId, IssuerKey)> -> ()
RenewalIndex<(AccountId, IssuerKey)> -> due_at
```

平台机构 CID 永久固定为**创世常量**（公民链基金会 `CITIZENCHAIN_FOUNDATION`），不是可写存储；无 `PlatformCidNumber` 存储、无迁移（开发期零用户、重新创世）。`RenewalSchedule` 用大端时间戳键保持到期顺序，`RenewalIndex` 保证每个订阅只有一个当前到期项。不保存下次扣款区块、外部续费账户、设备状态、链下扣款密钥或第二份展示套餐。

## 6. Call 契约

| index | 调用 | 签名者 |
|---:|---|---|
| `0` | `publish_post(...)` | 发帖账户 |
| `1` | `subscribe(issuer, plan, expected_price_fen)` | 订阅者 |
| `2` | `cancel(issuer)` | 订阅者 |
| `3` | `set_creator_plans(tiers)` | 创作者 |
| `4` | `change_subscription_plan(issuer, new_plan, expected_price_fen)` | 订阅者 |
| `5` | `propose_set_platform_price(actor_cid_number, membership_level, new_price_fen)` | 公民链基金会治理岗位任职账户 |
旧 call、旧 SCALE tag 和旧交易载荷不兼容；不存在外部 `renew` 或周期确认 call。

## 7. 首次订阅与自动调度

1. CitizenApp 从 finalized storage 读取当前价格。
2. CitizenApp 提交 `subscribe`；runtime 重新读取当前价格并校验 `expected_price_fen`。
3. runtime 在同一 storage layer 完成转账，以当前区块唯一 `Timestamp.Now` 写入 `started_at`、`last_charged_at` 和审计价格。
4. runtime 按 UTC 真实公历计算 `paid_until`，写入 `Active`，并把该到期时间登记到调度索引。
5. CitizenApp 等待交易 finalized 后读取并显示链上 `started_at` 与 `paid_until`；不再提交第二笔确认交易。

平台收款账户从**创世常量** CID + `RESERVED_NAME_FEE` 派生（公民链基金会费用账户，创世已播种）。创作者订阅款全额进入创作者钱包，不能订阅自己，且创作者必须在扣款时拥有有效平台订阅。

## 8. runtime 自动续费

1. 续费在 `on_idle(remaining_weight)` 执行（Timestamp inherent 已于其前写入，可读本块共识时间戳）；不再用 `on_finalize` 固定处理量，也不静态预留最坏权重。
2. runtime 从最早到期项开始处理 `due_at <= now`，单块处理量按**当块剩余权重**动态排空（`limit = min(remaining/单笔权重, MaxSubscriptionRenewalsPerBlock backstop)`）；超大同刻促发多块分摊。
3. 每个周期扣款均读取当时最新链上价格；平台治理改价自动按新价续。创作者改价（当前价 ≠ `authorized_price_fen`）则**不自动续**，转 `Suspended(NeedReconsent)` 待订阅者再签名。
4. 停链期间无法发生状态变更；恢复出块后按到期顺序补扣所有已到期周期，未完成部分在后续区块继续。
5. 续费失败按原因分流（不再一律 `Terminated`）：余额不足 → `Suspended(InsufficientBalance)`（离调度）；创作者掉平台会员 → `CreatorPaused`（留调度、下周期重试、恢复即续）；档位/周期删除 → `Suspended(NeedReconsent)`；公历换算失效等 → `Terminated`。

续费不需要账户再次签名，不依赖 CitizenApp、设备或 Cloudflare 在线，也不存在任何外部续费提交者。

## 9. 挂起、取消和换套餐

- 挂起恢复：`Suspended` 由订阅者再签名（改价场景）/充值后再签（缺钱场景）恢复，走 `subscribe` 落到首扣路径；`CreatorPaused` 随创作者恢复平台会员在下周期重试时自动续。
- 取消：写 `Cancelled`，保留已确认的 `paid_until`，到期前已付权益仍有效。
- 换套餐（`change_subscription_plan`）**立即生效并折算**：剩余权益 `y = 已授权价 × (paid_until−now) ÷ (paid_until−last_charged_at)`；升档补扣 `新价−y`、新周期从现在起算；降档不扣、余额按新档单价折算成延长时长。
- 再订阅（同计划未过期的 `Cancelled`）：恢复原调度继续扣费，不重扣。
- 不退款、不补差价、不按日折算。

## 10. 创作者套餐与平台调价

- `set_creator_plans` 覆盖式写入创作者自己的链上付款字段。
- 新订阅和下一次真实续费读取最新价格；当前已付周期不变。
- 创作者在 CitizenApp 同一次业务提交中填写档位标识、名称和周期价格，并只签名一次 `set_creator_plans` 交易；finalized 后 App 把交易哈希、区块哈希和完整已签名 extrinsic 连同展示资料提交给 Cloudflare，Worker 严格复核交易包含关系和同一区块链上状态后保存镜像。
- Cloudflare 展示资料必须引用 finalized 的 `creator_account_id + tier_id`，不得保存第二份扣款真源价格；finalized 后的镜像只用 Bearer 会话和链读复核，不生成设备请求签名。边缘保存失败只能重试镜像 HTTP，不得再次签名或重复提交链上交易。
- `propose_set_platform_price` 只调用统一内部投票引擎；人口快照、资格、计票和状态推进不进入业务 pallet。

## 11. 信任边界

订阅授权由订阅者第一次签名建立，并持续有效到订阅者签名取消。公历到期时间、自动扣款和状态变更由 runtime 唯一决定；CitizenApp 与 Cloudflare 都不能伪造、延长或触发续费。共识时间戳仍受区块生产与 Timestamp pallet 规则约束，但在同一链状态下所有节点执行完全相同的整数公历结果。

## 12. Cloudflare/D1

- 钱包账户是所有镜像的业务主键；平台订阅主键为 `account_id`，创作者档位主键为 `(creator_account_id, tier_id)`，创作者订阅主键为 `(subscriber_account_id, creator_account_id)`。
- confirm 请求固定携带 `tx_hash`、`block_hash`、`signed_extrinsic_hex` 和业务动作；订阅或换档携带目标档位，创作者订阅同时携带 `creator_account_id`、`tier_id`、`billing_period`，创作者套餐保存同时携带展示档位数组。
- Worker 重新计算 extrinsic 哈希，严格解码签名者、pallet/call index 与 SCALE 参数，校验签名者等于 Bearer 会话钱包、指定区块属于 finalized 主链且确实包含该完整 extrinsic，再读取同一区块 `Timestamp.Now`、`Subscriptions` 或 `CreatorPlans`。请求中的价格、状态和期限从不作为真源。
- `chain_transaction_confirmations` 将一笔 finalized 交易首次绑定到钱包、区块、extrinsic 序号、动作和规范化请求哈希；完全相同的 HTTP 重试幂等成功，同一交易换钱包、换动作或换展示资料一律冲突拒绝。
- `square_memberships` 和 `square_creator_subscriptions` 镜像完整链上状态、finalized 锚点及最近一次交易哈希；`last_charged_price_fen` 只是已发生扣款的审计镜像，不能作为下一次扣款价格真源。
- `square_creator_tiers` 按档位规范化保存展示名称和 finalized `CreatorPlans` 镜像；覆盖保存使用 D1 batch 原子替换，不保留退役档位残行。
- `chain_clock` 只接受更高 finalized 区块，保存同一区块链时间戳和本地观测时刻。门禁统一要求状态为 `Active` 或尚在已付期内的 `Cancelled`、`chain_timestamp < paid_until` 且链时钟未陈旧；`Terminated`、未知状态、缺时钟、未来观测、陈旧时钟和到期全部 fail-closed。
- Cron 每轮只读取一次 finalized 头和时间戳，只查询 `Active AND paid_until <= chain_timestamp` 的到期候选，按固定上限逐行纠偏；不扫描未到期全表，不计算公历，不触发扣款或续费。
- 平台发布、上传预留、平台用量与创作者管理等 Cloudflare 资源入口都调用同一平台门禁；创作者订阅专属资源必须在签发数据或短效资源地址前调用创作者订阅门禁。仓库当前没有创作者专属内容路由，因此本步骤不虚构该产品功能。
- CitizenApp 按钱包保存最近 finalized 证明和有界镜像待重试队列；App 再次运行时只重试 Bearer HTTP，不再次签名或提交链上交易。Cloudflare 不可用不阻断链上订阅操作。
- 直接端到端 P2P 媒体不占用 Cloudflare 存储或中转，App 的发送端与接收端仍执行本地大小门禁；这类端到端数据不应被表述为 Cloudflare 可集中强制的订阅权益。Cloudflare 承载的上传、存储、中转和签名 URL 则全部由服务端门禁强制执行。

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
- 第四步已把 finalized 证明扩展为交易哈希、区块哈希和完整已签名 extrinsic；App 按钱包持久化有限证明历史与待重试队列，镜像失败不重复签名。
- 页面显示使用 `DateTime.fromMillisecondsSinceEpoch(...).toLocal()` 展示真实日期和时间，不显示区块高度或“固定天数”。
- Cloudflare 暂时不可用时，App 仍以 finalized 链上价格、档位和订阅状态工作；展示名称可使用本地兜底。

## 14.1 交易收费

- `subscribe`、`cancel`、`change_subscription_plan`、`set_creator_plans` 都是账户签名的非系统链上交易，统一经过 runtime 交易支付扩展和签名账户收费路由。
- 业务转账金额为零时仍收取最低链上交易费；不得把取消订阅或只改状态误判成免费操作。
- runtime 到期自动扣款在区块执行阶段内部运行，不是外部交易，因此不追加用户交易费。

## 14.2 OnChina 平台调价与机构工作台

- 所有机构管理员都从链上中国统一入口扫码登录。登录态必须携带节点绑定的准确 `institution_cid_number`，工作台由后端根据准确 CID、机构类型和链上权限下发；前端不得根据机构码猜测工作台。
- 注册局、私权、司法、立法、其它公权和非法人机构使用不同工作台。私权机构只查看本机构信息、链上 `admins` 和被授权模块，不复用注册局的公民、机构目录或登记页面。
- 平台会员价格模块是实例级授权：只有当前绑定 CID 与**创世常量**平台机构 CID（公民链基金会）精确相等时才下发。OnChina 不在 PostgreSQL 保存平台价格或平台 CID 副本。
- 调价 API 为 `GET /api/v1/membership/platform-prices` 与 `POST /api/v1/membership/platform-prices/propose`。prepare 和 submit 都重新检查节点绑定、准确平台 CID 和链上 active `admins`，任何无法确认都 fail-closed。
- 所有 OnChina 链交易共用 `POST /api/v1/admin/chain/submit` 与同一 core 提交器。流程固定为：OnChina 展示请求二维码，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫后验签、dry-run、提交并等待进块。禁止业务模块另建提交 URL、二维码协议或签名流程。
- 平台调价动作在唯一 QR registry 中为 `propose_set_platform_price`；CitizenWallet 必须中文展示公民链基金会 CID、目标平台档位和新价格，未知或不完整载荷直接拒签。
- `propose_set_platform_price` 只创建统一内部投票提案。资格、计票、推进和终态执行归投票引擎，OnChina 和 SquarePost 不实现第二套投票。

## 15. 真实验收

- runtime 单元测试、金标 SCALE、benchmark 编译、完整 runtime 测试和 WASM 构建通过。
- 在真实链数据库副本确认旧 StorageVersion、订阅相关前缀为空及迁移 pre/post 不变量。
- runtime 覆盖月末、闰年、跨年、季和年周期计算、自动续费、停链后补扣和余额不足终止。
- Cloudflare 严格解码新状态，拒绝尾随字节和非法标签，且不包含任何日期计算。
- Cloudflare 必须验证 finalized 主链中完整已签名 extrinsic、同一区块状态和首次请求绑定；旧区块证明不能刷新链时钟。
- 本地 Worker、D1 与 HTTP 必须实测缺设备证明的 finalized 镜像请求可进入业务校验，而其它受保护写请求仍保持设备证明门禁。
- OnChina 已在隔离本地 PostgreSQL 上连接真实本地链并完成链投影同步；平台价格、调价提案和统一提交接口在无登录态时均 fail-closed，旧公民专属提交入口已移除。
- OnChina、CitizenWallet 与统一二维码注册表已完成编译、静态分析和自动测试；最终跨端调价交易、内部投票终态及完整订阅生命周期纳入第 6 步总验收。
- 后续步骤必须完成真机、真实本地链、真实 Worker/D1/HTTP 的端到端验收。

## 16. 禁止事项

- 禁止用区块高度、固定天数或固定毫秒表示订阅周期。
- 禁止在 CitizenApp 或 Cloudflare 计算并提交订阅到期时间。
- 禁止外部 renew、周期确认、设备签名或 keeper。
- 禁止跳过停链期间已经到期的周期或恢复旧订阅协议。
- 禁止把 D1 镜像当作订阅真源。
- 禁止在业务模块实现投票流程。
- 禁止保留旧字段、旧注释、旧 UI 文案或兼容分支。
