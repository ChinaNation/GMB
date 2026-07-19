# 任务卡：公民币平台订阅与创作者订阅统一改造

> 状态：执行中
> 当前步骤：第 5 步已完成；等待确认第 6 步
> 唯一架构文档：`memory/01-architecture/gmb/subscription-part1-tech.md`
> 决策记录：`memory/04-decisions/ADR-037-citizen-coin-native-membership.md`

## 1. 任务目标

在保留现有 CitizenChain 及全部无关链上状态的前提下，把平台订阅与创作者订阅统一为一套公民币原生订阅系统，并贯通 CitizenApp、CitizenChain、Cloudflare/D1、OnChina 和 CitizenWallet。

本任务禁止重新创世、替换 chainspec、清空链数据库或恢复 Stripe、USDC、外部计费 keeper。现有正式链已确认没有订阅数据；runtime 升级仍必须使用 StorageVersion 原地升级并在发现任何订阅记录时中止。

## 2. 最终业务规则

### 2.1 平台订阅

- 三档平台会员为 `freedom`、`democracy`、`spark`。
- 档位价格唯一真源为链上 `PlatformPrice`。
- 用户在 CitizenApp 对订阅、取消、换档签名。
- 首次扣款和每次 runtime 自动续费均读取扣款时最新链上价格。
- 订阅款全额进入技术公司费用账户。
- 平台调价只允许通过统一投票引擎，不得在业务模块内实现投票。

### 2.2 创作者订阅

- 成为创作者的唯一资格是当前拥有有效平台订阅。
- 链上只保存扣款必需的 `tier_id`、`billing_period` 和 `price_fen`。
- 档位名称、说明、权益文案和媒体资料只保存在 Cloudflare/D1。
- 创作者修改套餐扣款字段必须签名上链。
- 创作者改价对全部存量订阅的下一次扣款生效；当前已付周期不补差价。
- 到期后由 runtime 根据当前区块共识时间戳自动扣款，不需要续费签名或周期确认。
- App、设备和 Cloudflare 在线状态不影响续费；停链期间到期周期在恢复出块后依次补扣。
- 创作者订阅款全额进入创作者钱包。
- 创作者在 CitizenApp 一次提交自己的档位标识、名称与各真实公历周期价格；同一业务操作只签名一次。链上交易 finalized 后，Cloudflare 接收交易哈希、严格读取 finalized 链上付款字段并保存展示名称，不得再要求账户签名或设备请求签名。

### 2.3 换套餐与生命周期

- Active 且未到期：换套餐写入 `pending_plan`，下一次续费时生效。
- 已取消或已终止：换套餐作为新的签名授权，立即按当前价格首扣并恢复 Active。
- 不退款、不补差价、不按天折算。
- `Cancelled` 只停止续费，已付权益保留至 `paid_until`。
- `Terminated` 表示余额不足、真实转账失败或套餐失效，自动续费永久停止且不重试。
- 订阅周期只使用时间戳和真实公历，与区块高度无关。
- 自然公历只在 runtime 按 UTC 确定性计算；CitizenApp 和 Cloudflare 都不得提交到期时间。

### 2.4 签名与交易收费

- 同一个业务操作只允许一次签名，不得把链上提交、finalized 后镜像、重试镜像拆成账户签名或设备请求签名。
- 订阅、取消、换套餐、创作者设置套餐分别是一笔用户发起的链上交易，各自按统一链上交易费规则收费。
- 所有产生区块且不属于系统内部执行的交易都必须收费；不能因为业务金额为零而免除交易费。
- runtime 内部自动续费不是外部用户交易，不追加用户交易费，也不要求签名。

## 3. 链上固定契约

`SquarePost` pallet index 固定为 `34 / 0x22`，不新建 pallet。

| call index | 调用 |
|---:|---|
| `0` | `publish_post` |
| `1` | `subscribe(issuer, plan, expected_price_fen)` |
| `2` | `cancel(issuer)` |
| `3` | `set_creator_plans(tiers)` |
| `4` | `change_subscription_plan(issuer, new_plan, expected_price_fen)` |
| `5` | `propose_set_platform_price(...)` |
不存在外部续费或周期确认 call。订阅签名持续授权 runtime 自动扣款，直到订阅者签名取消。

核心 storage：

- `Subscriptions`
- `PlatformPrice`
- `PlatformCidNumber`
- `CreatorPlans`
- `RenewalSchedule`
- `RenewalIndex`
- `MigrationBlocked`

## 4. 模块边界

- `citizenchain/runtime/misc/square-post/`：价格、扣款、确定性 UTC 公历、到期调度、最小订阅状态、迁移和平台调价回调。
- `citizenapp/`：订阅、取消、换套餐热签，finalized 状态读取和真实日期展示；无续费提交。
- `citizenapp/cloudflare/`：创作者展示资料、finalized 镜像、低频对账和内容权益门禁；无扣款权、无日期计算。
- `citizenchain/onchina/`：技术公司平台调价治理入口。
- `citizenwallet/`：平台调价载荷中文识别、一次签名和响应二维码展示；普通订阅不进入 CitizenWallet。
- `memory/`：任务、协议、命名、ADR、架构和验收事实真源。

## 5. 分步骤执行

- [x] 第 1 步：统一任务卡、ADR、协议、命名与技术架构。
- [x] 第 2 步：完成 square-post runtime 原地升级与链上订阅状态机。
- [x] 第 3 步：完成 CitizenApp 平台/创作者订阅和换套餐流程。
- [x] 第 4 步：完成 Cloudflare/D1 finalized 镜像、对账和权益门禁。
- [x] 第 5 步：完成 OnChina 平台调价、机构工作台隔离与 CitizenWallet 一次签名响应回扫。
- [ ] 第 6 步：完成跨端真实运行态验收、残留总清理和任务归档。

每一步都必须先输出技术方案并取得确认；执行后立即更新文档、完善中文注释、清理残留，再输出下一步技术方案。

## 6. 文件与新增限制

- 继续使用本任务卡，不新建第二张执行任务卡。
- 任何 runtime 修改必须单独取得二次确认。
- 任何新增文件或目录必须列出完整路径、用途、原因、Git 跟踪状态并取得确认。
- 禁止 Git push、PR、远端 workflow 和生产部署，除非当前任务另行明确授权。

## 7. 完成标准

- 平台和创作者首次扣款、runtime 自动续费、停链后补扣、取消、换套餐、余额不足终止全部按目标语义运行。
- 创作者平台订阅资格在 App、runtime 和 Cloudflare 三层一致执行。
- 创作者改价能作用于全部存量订阅的下一次扣款。
- D1 只作为 finalized 链状态镜像，任何外部请求不得触发续费或延长权益。
- runtime 在现有链状态副本上完成真实原地升级验收，无关状态不变。
- CitizenApp 真机、真实本地 Worker/D1、真实 HTTP、真实链上交易完成端到端验收。
- 代码、测试、中文注释、协议、ADR、架构、任务卡和 UI 文案无旧订阅路线残留。

## 8. 当前已知工作区状态

仓库仍有多个并行任务留下的未提交修改。当前任务只收敛订阅相关目标文件，保留 OnChina、CitizenConsole、CitizenWeb、钱包及其他任务的无关改动，不擅自暂存、提交、回退或清理用户工作。

## 9. 第 1 步执行记录（2026-07-18）

- 重写本任务卡，固定六步执行顺序和每步确认门禁。
- 重写 ADR-037 和订阅完整技术架构。
- 在统一协议登记 P-TX-014 和 P-STORAGE-006。
- 在统一命名登记订阅跨端字段并删除会员支付 Stripe/USDC 旧命名。
- 将旧链端专项任务卡移入 `done/`，清除其可执行 keeper 和永久锁价旧方案。
- 更新仓库技术总览的公民币订阅主流程。
- `git diff --check` 通过；`bash scripts/check-startup-acceptance.sh --ci` 通过。
- 本步骤没有修改代码、runtime、数据库、链状态或远端。

## 10. 第 2 步执行记录（2026-07-18）

- `SquarePost` 由订阅签名建立持续授权；删除外部 `renew`、周期确认及对应签名入口。
- 首次订阅立即读取链上当前价格并转账；runtime 从当前区块唯一共识时间戳确定性计算下一个真实 UTC 公历到期时间。
- 新增 `RenewalSchedule` 与 `RenewalIndex` 有界调度；到期自动读取最新价格扣款，停链恢复后按到期顺序补扣全部应付周期。
- 余额不足、真实转账失败、平台收款账户失效、创作者资格或套餐失效时写 `Terminated`、移除调度且不重试；签名取消写 `Cancelled` 并保留当前已付权益。
- 未到期签名换套餐写 `pending_plan`，在下一次自动扣款原子生效；已取消或已终止后换套餐建立新的自动扣款授权。
- CitizenApp 已删除公历计算、续费和周期确认 RPC，只保留订阅、取消、换套餐及创作者套餐管理签名，并展示链上时间戳。
- Cloudflare 严格镜像 finalized 状态，不计算公历、不触发续费、不保存第二份价格真源；完整镜像、对账和门禁仍归第 4 步。
- StorageVersion 原地迁移和 try-runtime pre/post 校验继续保留；发现旧订阅或调度数据时阻断，不转换、不兼容。
- 回归结果：`square-post` 单元测试 14 项、完整 runtime 单元测试 44 项、CitizenApp 目标测试 17 项通过；CitizenApp 目标静态分析无问题；Cloudflare 目标测试 42 项和 TypeScript 类型检查通过；runtime benchmark 与 try-runtime 特性编译通过。
- 在开发链数据库只读副本完成真实 try-runtime：原链 runtime 版本 1、新 runtime 版本 2，完整状态解码、迁移 pre/post、全部 pallet try-state、第二次迁移幂等性和权重安全检查均通过；原数据库未写入，验收副本已移入系统废纸篓。
- 已用 `WASM_BUILD_FROM_SOURCE=1` 恢复普通生产 release WASM；未执行链上升级、原数据库写入、Git 提交、Git 推送、远端 workflow 或生产部署。

## 11. 第 3 步技术方案（已执行）

### 11.1 目标

完成 CitizenApp 平台订阅和创作者订阅的用户流程。App 只对订阅、取消、换套餐以及创作者管理自己套餐的操作签名；启动、恢复前台和设备更换只刷新 finalized 链状态，绝不提交续费、周期确认或其它签名。

### 11.2 实现范围

1. 在 `subscription_rpc.dart` 增加新 `SubscriptionState` SCALE 严格解码和 finalized storage 读取，字段顺序与 P-STORAGE-006 完全一致；继续只保留 call index 1—4 的 App 业务调用。
2. 平台会员页直接读取 finalized `PlatformPrice` 与当前账户平台订阅，完成订阅、取消、换档、等待 finalized、刷新状态和防重复点击。
3. 创作者订阅页读取 finalized `CreatorPlans` 与当前账户对该创作者的订阅，按月、季、年展示链上当前价格，完成订阅、取消、换档和 finalized 刷新。
4. 创作者套餐管理使用创作者账户签名覆盖式提交 `set_creator_plans`；名称、说明、权益和媒体仍只走现有 Cloudflare 展示资料接口，不复制链上付款价格。
5. 所有页面仅把链上 unix 毫秒时间戳转换为本地真实日期显示；`Active` 显示下次链上自动扣款时间，`Cancelled` 显示已付权益截止时间，`Terminated` 显示订阅已终止并允许用户重新签名订阅或换套餐。
6. App 生命周期监听只做 finalized 状态刷新和 UI 更新，禁止产生任何自动交易、设备授权、续费签名或周期确认。

### 11.3 预计修改目录

- `citizenapp/lib/rpc/`：代码；实现订阅 storage key、SCALE 严格解码和 finalized 链读，边界是不新增续费 call。
- `citizenapp/lib/my/membership/`：代码；完成平台订阅、取消、换档、状态刷新和真实日期展示。
- `citizenapp/lib/8964/subscribe/`、`citizenapp/lib/my/creator/`：代码；完成创作者订阅与创作者套餐管理，展示字段仍与链上付款字段分离。
- `citizenapp/test/`：测试；补齐跨端金标、RPC、service、widget、生命周期只读刷新和错误态测试；预计复用现有测试文件，不新增文件。
- `memory/01-architecture/gmb/`、`memory/07-ai/`、本任务卡：文档与残留清理；同步最终 App 行为、字段、测试和真实验收事实，不修改 runtime 方案。

### 11.4 不修改范围

- 不修改 `citizenchain/runtime/`，因此第 3 步不触发 runtime 二次确认。
- 不实现 Cloudflare 完整对账和门禁；该范围保留到第 4 步。
- 不修改 OnChina、CitizenWallet、链数据库、chainspec 或部署配置。
- 不新增外部续费服务、keeper、App 定时扣款、周期确认、设备签名或兼容旧协议。

### 11.5 验收

- Dart 与 runtime 金标 SCALE 逐字节一致，非法标签、截断和尾随字节 fail-closed。
- 平台与创作者订阅、取消、换套餐分别完成真实本地链 signed extrinsic → finalized → App 状态刷新。
- App 重启、恢复前台和更换设备仅产生链读，不产生交易或签名请求。
- 页面显示真实本地日期，且 Active、Cancelled、Terminated 三态和按钮行为与链上一致。
- Flutter 单元、service、widget、静态分析和真机/模拟器真实运行态验收通过；随后更新文档、完善中文注释并清理旧文案与旧入口。

## 12. 第 3 步执行记录（2026-07-18）

- 保留 CitizenApp 已有会员三卡 UI、创作者页和广场创作者订阅按钮，只替换其数据真源与操作编排。
- `subscription_rpc.dart` 已接入 `Subscriptions`、`CreatorPlans` 和同一 finalized 区块的 `Timestamp.Now` 严格读取；非法枚举、截断及尾随字节全部 fail-closed。
- 平台订阅和创作者订阅均已接入订阅、取消、换档的一次账户签名交易；页面直接使用 finalized 链状态显示真实本地日期，不计算或提交到期日期。
- 创作者设置档位采用一次 `set_creator_plans` 签名。链上 finalized 后，Cloudflare 接收交易哈希、区块哈希和完整已签名 extrinsic，严格复核签名钱包、调用参数、区块包含关系及同一区块 finalized 付款字段，只保存可重建镜像和展示字段；这些镜像请求只用 Bearer 会话，不生成设备请求签名。边缘保存失败会在 CitizenApp 下次运行时只重试 HTTP 镜像，不再次签名或提交链上交易。
- Cloudflare 暂时不可用时，CitizenApp 仍可读取链上价格、档位和订阅真态；展示名称使用现有本地兜底，边缘服务不再成为链上功能前置条件。
- 已审计 runtime 交易收费路由：`subscribe`、`cancel`、`change_subscription_plan`、`set_creator_plans` 均进入签名账户收费路由，并由统一交易支付扩展扣费；零业务金额仍适用最低链上交易费。runtime 自动续费属于系统内部执行，不生成外部收费交易。
- CitizenApp 目标静态分析通过，目标测试 29 项通过；Cloudflare TypeScript 类型检查、目标测试 45 项和完整测试 178 项通过；`git diff --check` 通过。
- 真实本地 Worker 已启动并通过 `/health` HTTP 检查；CitizenApp 已在真实 Android 设备安装、启动并检查现有会员与创作者 UI，创作者平台会员门禁按 finalized 真态显示。
- 当前运行中的本地链仍是旧 runtime，尚未安装第 2 步产物；因此不能虚构本步骤的真实签名交易 finalized 验收。未执行 runtime 升级、链数据库写入、远端部署、Git 提交或推送；真实跨端交易验收在获得单独部署授权后纳入第 6 步。

## 13. 第 4 步技术方案（已执行）

### 13.1 目标

完成 Cloudflare/D1 的 finalized 订阅镜像、低资源对账和内容权益门禁。Cloudflare 只加速读取与保存展示资料，任何 HTTP 请求都不能扣款、续费、延长权益、决定价格或要求第二次签名。

### 13.2 实现范围

1. 平台与创作者订阅镜像统一保存 finalized 链字段：订阅者、收款主体、当前套餐、待生效套餐、开始时间、最近扣款时间、最近扣款价格、已付权益截止时间、状态、finalized 区块号和区块哈希。
2. 首次订阅、取消、换档及创作者套餐保存的 HTTP 确认接收交易哈希、区块哈希、完整已签名 extrinsic、动作和必要展示字段，只使用现有 Bearer 会话，不生成设备请求签名；Worker 复算哈希、严格解码签名者与调用参数、确认交易位于 finalized 主链区块，并从同一区块状态复核后幂等写 D1，不信任客户端提交的状态或时间。
3. 权益有效口径统一为：`Active` 且链上当前时间早于 `paid_until`，或 `Cancelled` 且链上当前时间仍早于 `paid_until`；`Terminated`、过期、未知状态、解码失败和链读失败全部拒绝。取消不能立即剥夺已经付款的权益。
4. 创作者资格门禁使用同一 finalized 平台订阅有效口径；创作者内容门禁使用订阅者对该创作者的 finalized 订阅有效口径，不采信 D1 自报状态。
5. 对账采用有界、低频、分批游标方式，只纠正已有镜像；不全链高频扫描、不运行外部续费任务、不在 Worker 计算公历。确认路径优先精确链读，D1 索引只服务查询与有限批次。
6. 创作者档位名称等展示资料继续保存在 D1；付款套餐和价格从 finalized `CreatorPlans` 复核并镜像，链上始终是唯一扣款真源。
7. 删除旧状态判断、旧确认载荷、已移除路由、二次业务签名和把 D1 当真源的残留，不保留兼容分支。

### 13.3 预计修改目录

- `citizenapp/cloudflare/src/chain/`：代码；统一 finalized storage 读取、严格 SCALE 解码、同一区块时间戳与交易复核，边界是不发起任何扣款交易。
- `citizenapp/cloudflare/src/membership/`：代码与残留清理；实现平台/创作者镜像、低频分批对账及统一有效权益门禁，删除旧状态和二次签名路径。
- `citizenapp/cloudflare/migrations/`：数据库文档或现有迁移调整；统一镜像字段、复合唯一键和对账索引，不保存第二份价格真源，不新增兼容表。
- `citizenapp/cloudflare/src/routes.ts`、`src/limits/`、`src/types.ts`：代码与残留清理；收敛确认路由、请求类型、限流和 fail-closed 边界。
- `citizenapp/cloudflare/test/`：测试；覆盖 finalized 复核、幂等、取消后已付权益、终止拒绝、待生效换档、链读失败和有界对账；优先复用现有测试文件。
- `citizenapp/lib/8964/`、`citizenapp/lib/my/`：仅在真实 HTTP 契约需要时修改现有 API 调用与错误展示，不改变已经保留的 UI，也不增加签名。
- `memory/01-architecture/gmb/` 与本任务卡：文档和残留清理；记录最终 Worker/D1 契约、资源边界、测试及真实验收事实。

本步骤预计不新建文件或目录；如检查后确需新增，必须先列出完整路径、用途、原因及 Git 跟踪状态并再次取得确认。

### 13.4 不修改范围

- 不修改 `citizenchain/runtime/`、OnChina、CitizenWallet、chainspec 或链数据库。
- 不部署 Cloudflare、不写生产 D1、不推送 GitHub，也不触发远端 workflow。
- 不实现外部续费、周期确认、新的账户或设备签名、日期计算、全链扫描或旧接口兼容。

### 13.5 验收

- Worker 类型检查和目标测试通过；严格解码、幂等和 fail-closed 测试覆盖完整。
- 使用真实本地 Worker、真实本地 D1 和真实 HTTP 验证确认、查询、门禁与有界对账。
- 验证同一个创作者套餐业务操作从一次链上签名到边缘镜像完成全程没有第二次签名；镜像失败重试只产生 HTTP 请求。
- 验证 `Cancelled` 在 `paid_until` 前仍有权益、到期后拒绝，`Terminated` 始终拒绝。
- 更新文档、完善中文注释并清除旧路由、旧字段、旧签名和旧状态判断残留后，输出第 5 步完整技术方案。

## 14. 第 4 步执行记录（2026-07-18）

- finalized 镜像已从弱交易引用收紧为完整交易证明：Worker 复算 `tx_hash`，严格解码 signed extrinsic 的签名钱包、pallet/call 和参数，确认 `block_hash` 属于 finalized 主链、完整 extrinsic 确实包含在该区块，再读取同一区块 `Timestamp.Now`、`Subscriptions` 或 `CreatorPlans`。
- 新增交易首次绑定约束：同一 `tx_hash` 只能绑定一个钱包、区块、extrinsic 序号、动作和规范化请求哈希；相同 HTTP 重试幂等，改绑钱包、动作或展示资料返回冲突。
- D1 基线已统一使用钱包账户键：平台订阅主键为 `owner_account`，创作者档位复合主键为 `(creator_account,tier_id)`，创作者订阅复合主键为 `(subscriber_account,creator_account)`；旧的 `0004_creator.sql` 迁移已删除，未保留兼容表或双轨字段。
- `chain_clock` 只接受更高 finalized 区块。统一门禁接受未到期的 `Active` 和 `Cancelled`，拒绝 `Terminated`、到期、缺失、未来观测或陈旧链时钟；平台发布/上传/用量、创作者套餐管理与创作者统计共用该 fail-closed 口径。
- 对账 Cron 每轮只读取一次 finalized 头与时间戳，只查询已到期的 Active 平台/创作者镜像候选，按有界批次逐行纠偏；不扫描未到期全表、不计算公历、不触发续费，单行链读失败不阻断同批其它记录。
- CitizenApp 已按钱包持久化 finalized 交易证明历史和有界待镜像队列；App 再次运行时只重试 Bearer HTTP，不再次签名或重复提交链上交易，原有会员与创作者 UI 页面保持不变。
- Worker TypeScript 类型检查通过；26 个测试文件、154 项测试通过；Wrangler 本地启动阶段构建与分析通过。CitizenApp 全仓静态分析通过，全量 Flutter 测试 738 项通过，5 项因纯 Dart 宿主缺少原生库按既有条件跳过，没有失败。
- 真实本地 D1 基线迁移执行 51 条命令成功并确认五张订阅相关表存在；真实本地 Worker `/health` 返回 200。finalized 镜像路由在 Bearer 且无设备证明时进入业务参数校验，普通受保护写路由在缺设备证明时仍返回 401，证明“镜像不二次签名”没有放宽其它写入口。
- 本步骤没有修改 `citizenchain/runtime/`，没有写生产 D1、没有部署 Cloudflare、没有升级链数据库，也没有提交或推送 Git。

## 15. 第 5 步执行记录（2026-07-18）

- 登录、扫码登录轮询、鉴权检查和工作台清单统一携带节点绑定的准确 `institution_cid_number`；前端不再根据机构码猜测工作台，后端未返回工作台时直接拒绝加载。
- 工作台类型已拆分为注册局、私权、司法、立法、其它公权和非法人机构。私权机构管理员仍从链上中国统一入口扫码登录，但只进入本机构信息、`admins` 和经授权模块，不复用注册局的公民、机构目录或登记 UI。
- 平台会员模块只在当前绑定 CID 与 finalized `PlatformCidNumber` 精确一致时下发；查询与发起前读取同一 finalized 区块的三档 `PlatformPrice`，链读、CID、节点绑定或链上 `admins` 任一无法确认均 fail-closed，PostgreSQL 不保存价格副本。
- 调价提案严格构造 `SquarePost::propose_set_platform_price`，仅调用现有统一内部投票引擎。OnChina 不实现投票资格、计票、推进或执行，也不保存投票表。
- 链签流程已收口到唯一 core 实现和统一 `POST /api/v1/admin/chain/submit`：OnChina 展示请求二维码，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫响应后统一验签、dry-run、提交并等待进块；旧公民专属提交 URL 和业务专属提交实现已移除。
- prepare 与 submit 都重新核对当前节点绑定、准确平台 CID 和链上 active `admins`，防止 prepare 后撤权继续提交。调价外部交易继续经过统一交易收费；投票引擎系统执行不追加签名或外部交易。
- 唯一二维码 registry 已登记 `propose_set_platform_price` 与中文字段。CitizenWallet 严格显示技术公司 CID、目标档位和新价格，并拒绝未知档位、零价、截断、尾随字节、call/action 不一致及非标准载荷。
- OnChina 140 项测试、CitizenWallet signer 139 项测试、二维码 registry 一致性和单注册表守卫全部通过；前端生产构建和 Dart 静态分析通过。
- 真实本地验收使用隔离 OnChina 与临时 PostgreSQL 连接真实本地 CitizenChain，完成链投影同步并达到健康状态 UP；未登录访问平台价格、调价提案和统一提交均返回拒绝，旧提交入口已失效。隔离进程与临时数据已停止并清理。
- 本步骤未修改 `citizenchain/runtime/`，未部署、未写生产数据库、未提交或推送 Git。

## 16. 第 6 步完整技术方案（等待确认）

### 16.1 目标

在不增加任何签名、不新建业务流程的前提下，完成平台订阅、创作者订阅、自动续费、Cloudflare 镜像、机构工作台和平台调价的一次跨端真实运行态总验收；清除全仓旧接口、旧字段、旧文案和重复实现，所有验收事实回写文档后归档本任务。

### 16.2 执行顺序

1. 固定验收环境：使用独立本地链状态、独立 OnChina/PostgreSQL、真实本地 Worker/D1 和 CitizenApp/CitizenWallet 调试运行；先记录 genesis、finalized 区块和测试账户，禁止触碰生产状态。
2. 验收 CitizenApp 平台订阅：订阅、取消、换档各只签名一次；finalized 后显示真实本地日期，Cloudflare 镜像失败重试不再签名；取消后已付期内权益有效、到期后拒绝。
3. 验收创作者链路：有效平台会员才能设置创作者套餐；一次签名同时确认链上付款字段，finalized 后 Cloudflare 保存展示资料；订阅、换档和取消分别只签名一次，款项进入创作者钱包。
4. 验收 runtime 自动续费：推进真实共识时间到到期点，确认无需 App、设备、Cloudflare 或外部提交即可自动扣款；余额不足、套餐失效和创作者平台资格失效均终止且不重试。此项只运行现有代码；若发现必须修改 runtime，立即停止并列出完整路径、改动和原因，另行取得 runtime 二次确认。
5. 验收 OnChina 调价：平台机构管理员从统一登录入口进入专属工作台，OnChina 出请求二维码，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫统一提交；确认链上只创建统一内部投票提案，业务模块没有第二套投票。
6. 执行全仓残留审计：清理旧订阅 API、旧提交 URL、第二次签名、设备确认、外部续费、区块高度周期、固定天数文案、重复 action registry 和错误产品命名；复跑各端全量测试与真实 HTTP/页面检查。
7. 更新架构、ADR、统一协议、模块文档和任务卡；全部完成后把本任务卡从 `open` 移至仓库既有完成态目录，不新建归档文件。

### 16.3 预计修改目录

- `citizenapp/lib/`：代码、注释与残留清理；只修复跨端验收发现的订阅、finalized 展示、一次签名或镜像重试问题，保留既有会员和创作者 UI。
- `citizenapp/test/`：现有测试与残留清理；补足一次签名、真实日期展示、镜像重试和门禁回归，不新增测试文件。
- `citizenapp/cloudflare/src/`、`citizenapp/cloudflare/test/`、`citizenapp/cloudflare/migrations/`：Worker/D1 代码、测试、基线和残留清理；只处理 finalized 镜像、幂等和权益门禁，禁止承担扣款、续费或日期计算。
- `citizenchain/onchina/src/`、`citizenchain/onchina/frontend/`：代码、页面、中文注释和残留清理；验收准确 CID 工作台、平台调价与唯一响应二维码回扫提交，不新增投票或第二套提交器。
- `citizenwallet/lib/`、`citizenwallet/test/`：代码、中文显示、严格拒签测试和残留清理；验收一次签名及响应二维码，不接入普通用户订阅。
- `citizenchain/crates/qr-protocol/registry/`：协议与生成物一致性清理；保持唯一动作注册表，不另建扫码协议。
- `citizenchain/runtime/`：默认只读验收，禁止产生任何 diff；如真实验收发现 runtime 缺陷，必须先暂停并取得逐路径二次确认后才能修改。
- `memory/01-architecture/`、`memory/04-decisions/`、`memory/05-modules/`、`memory/07-ai/`、`memory/08-tasks/`：文档、任务归档和残留清理；记录真实证据、最终接口和禁止边界，不创建新文档文件。

### 16.4 不修改与停止条件

- 不部署 Cloudflare、OnChina 或链升级，不写生产 D1/PostgreSQL，不推送 GitHub或触发远端 workflow。
- 不新增目录或文件；如验收确需新增，先列完整路径、用途、原因和 Git 跟踪状态并等待确认。
- 不兼容旧订阅、旧二维码、旧 URL 或旧数据格式。
- 任何无法从真实运行输出确认的签名次数、扣款、分账、门禁或投票结果都不得推断；立即停下并沟通。

### 16.5 完成标准

- 平台与创作者的订阅、取消、换档和创作者设置套餐均证明同一业务操作只有一次签名；自动续费证明没有签名和外部提交。
- 真实共识时间下的首次扣款、到期自动扣款、停链恢复补扣、余额不足终止、已付取消权益和下一周期换档全部取得真实链上证据。
- CitizenWallet 平台调价只签名一次并生成响应二维码，OnChina 回扫后由唯一入口提交并得到 finalized 交易或明确的真实环境阻断证据。
- Cloudflare/D1 只保存可重建 finalized 镜像和展示资料，所有平台/创作者资源门禁不能通过直接调用绕过。
- 全量测试、真实本地服务、真实 HTTP 和相关页面验收通过；文档、注释、生成物与代码一致，无旧流程残留，任务卡完成归档。
