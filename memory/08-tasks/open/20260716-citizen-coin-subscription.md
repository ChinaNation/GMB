# 任务卡：公民币平台订阅与创作者订阅统一改造

> 状态：执行中
> 当前步骤：链端 5.6a–5.6e 完成（18.9~18.13）；5.7 客户端解码同步完成——5.7a citizenapp（19.8）+ 5.7b cloudflare（19.9）；剩治理完整路径集成测试（votingengine 修复后已解锁）与第 6 步跨端真实运行态验收+归档
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
- 订阅款全额进入公民链基金会费用账户。
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
- ~~`PlatformCidNumber`~~（第 5.5 步删除：平台订阅机构永久固定为创世公民链基金会 `CITIZENCHAIN_FOUNDATION`，CID 收敛为创世常量单源，不再作为可写存储；详见第 17 节）
- `CreatorPlans`
- `RenewalSchedule`
- `RenewalIndex`
- `MigrationBlocked`

## 4. 模块边界

- `citizenchain/runtime/misc/square-post/`：价格、扣款、确定性 UTC 公历、到期调度、最小订阅状态、迁移和平台调价回调。
- `citizenapp/`：订阅、取消、换套餐热签，finalized 状态读取和真实日期展示；无续费提交。
- `citizenapp/cloudflare/`：创作者展示资料、finalized 镜像、低频对账和内容权益门禁；无扣款权、无日期计算。
- `citizenchain/onchina/`：公民链基金会平台调价治理入口。
- `citizenwallet/`：平台调价载荷中文识别、一次签名和响应二维码展示；普通订阅不进入 CitizenWallet。
- `memory/`：任务、协议、命名、ADR、架构和验收事实真源。

## 5. 分步骤执行

- [x] 第 1 步：统一任务卡、ADR、协议、命名与技术架构。
- [x] 第 2 步：完成 square-post runtime 原地升级与链上订阅状态机。
- [x] 第 3 步：完成 CitizenApp 平台/创作者订阅和换套餐流程。
- [x] 第 4 步：完成 Cloudflare/D1 finalized 镜像、对账和权益门禁。
- [x] 第 5 步：完成 OnChina 平台调价、机构工作台隔离与 CitizenWallet 一次签名响应回扫。
- [x] 第 5.5 步（runtime 二次确认）：平台订阅机构 CID 收敛为创世常量 `CITIZENCHAIN_FOUNDATION`，删除 `PlatformCidNumber` 死存储，创世（genesis_build）播种三档默认价；开发期零用户、重新创世模型，不做任何迁移，迁移机制整块直接删除；联动 OnChina 读取端（技术方案见第 17 节，执行记录见 17.8）。
- [ ] 第 5.6 步（runtime 二次确认）：订阅生命周期重构——挂起态（改价未再签名/余额不足）、换挡立即折算（升档扣差、降档延时）、创作者掉会员暂停粉丝续扣可恢复、续费扩容（on_idle 按块权重排空、删 64 固定上限）、补齐治理调价测试（技术方案见第 18 节）。
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
- 唯一二维码 registry 已登记 `propose_set_platform_price` 与中文字段。CitizenWallet 严格显示公民链基金会 CID、目标档位和新价格，并拒绝未知档位、零价、截断、尾随字节、call/action 不一致及非标准载荷。
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

## 17. 第 5.5 步技术方案（等待执行确认，runtime 二次确认）

### 17.1 背景与根因

复核发现：平台订阅收款方派生依赖存储 `PlatformCidNumber` 读取平台机构 CID，但全仓无任何写入方（pallet 无 `genesis_config`、创世 seeder 不写、无 extrinsic、pallet 也不读常量），导致 fresh 部署后平台轨恒 fail-closed（`PlatformNotBound`）；公民链基金会则由 `runtime/genesis/src/institution/seeder.rs::insert_citizenchain_foundation` 完整播种（身份、主账户、费用账户、一名管理员、三个固定岗位及三条兼任关系）。缺的是「创世常量 → pallet 存储」这根绑定桥。

因公民链基金会是**永久固定**的平台订阅机构，`PlatformCidNumber` 这层「可写存储」本身就是错误抽象。正确修复是把平台 CID 收敛为**创世常量单源**，而不是补一个可写存储的绑定入口。

> 部署模型更正（覆盖本卡第 1 节的「禁止重新创世 / StorageVersion 原地升级」约束，仅限本步）：GMB 全系开发期、零用户，本步按**重新创世**落地，**不做任何迁移**。原设想的 v2→v3 迁移已撤销；`square-post` 的整套迁移机制（`migration.rs`、`on_runtime_upgrade` 等 hooks、`MigrationBlocked`、`ensure_subscription_ready` 迁移门禁）在重新创世模型下均为死代码，作为残留**直接删除**。

### 17.2 目标态

- 平台订阅机构永久等于 `primitives::cid::china::citizenchain::CITIZENCHAIN_FOUNDATION`，与 `runtime/src/configs.rs`、node 治理骨架、onchina 审计同一单源。
- 平台三档价仍是链上可变真源（仅统一内部投票可改），但在缺省时有确定初值。
- 删除死存储 `PlatformCidNumber`，零残留，联动更新 OnChina 读取端。

### 17.3 实现范围

平台 CID 收敛为创世常量（采用直读常量方案，最小 diff，与 configs.rs:134 同口径，pallet 已依赖 `primitives`）：

1. `subscription.rs` `current_price_and_payee` Platform 分支：`PlatformCidNumber::<T>::get().ok_or(PlatformNotBound)` 改为读 `CITIZENCHAIN_FOUNDATION.cid_number.as_bytes()`；payee 仍按该 CID + `RESERVED_NAME_FEE` 派生；`PlatformNotBound` 语义收窄为「公民链基金会费用账户不可派生」（理论不发生，保留 fail-closed）。
2. `proposal.rs::propose_price_change` 与 `InternalVoteExecutor::on_internal_vote_finalized`：删除 `PlatformCidNumber::get`，平台机构判定改用 `primitives::cid::china::citizenchain::is_citizenchain_foundation_identity(code, actor_cid)`（同时校验机构码 SFGY + 准确 CID，比裸 CID 比对更硬）。
3. `lib.rs`：删除 `PlatformCidNumber` StorageValue 定义与相关类型引用；更新 `PlatformNotBound` 注释。

平台价格创世播种（重新创世模型，不做迁移）：

4. `lib.rs` 新增 `#[pallet::genesis_config]` + `#[pallet::genesis_build]`（空配置，`DefaultNoBound`），创世无条件写入三档默认价（`199_900 / 599_900 / 5_999_900`）；重新创世经 build-spec 烘焙进新 chainspec。

迁移机制整块删除（残留直接删，不保留）：

5. 删除整个 `migration.rs` 模块。
6. `lib.rs` 删除 `on_runtime_upgrade / pre_upgrade / post_upgrade / try_state` 四个 hooks、`MigrationBlocked` 存储、`STORAGE_VERSION` 迁移语义引用；删除 `subscription.rs::ensure_subscription_ready` 及 `do_subscribe / do_cancel / do_change_subscription_plan / do_set_creator_plans` 对它的调用（重新创世后无旧数据、无原地升级，全为死代码）。
7. `RenewalSchedule / RenewalIndex` 一致性属运行期不变量、非迁移专属：如需保留，把原 `try_state` 中该段校验移入 `lib.rs` 的 `#[cfg(feature = "try-runtime")] Hooks::try_state`；否则一并随 `migration.rs` 删除。默认保留该不变量、迁移相关校验全删。

OnChina 读取端联动（删存储的必然连带；不联动会使控制台平台价页 fail-closed）：

8. `onchina/src/core/chain_runtime.rs`：`fetch_platform_membership_snapshot` 不再读 `storage_value_key("SquarePost","PlatformCidNumber")`；`platform_cid_number` 直接取自创世常量 `CITIZENCHAIN_FOUNDATION.cid_number`（onchina 已在 `domains/gov/chain_audit.rs` 引用该常量）。`recheck_platform_admin`/`platform_prices` 逻辑不变，仅数据来源改常量。

测试：

9. `runtime/misc/square-post/src/tests/mod.rs`：删除 setup 中 `PlatformCidNumber::<Test>::put`（平台订阅测试直接生效，CID 来自常量）；删除迁移相关单测（v1→v2/回滚/阻断等）；补 `genesis_build` 创世价格播种回归。金标 SCALE 向量不受影响（`SubscriptionState` 布局未变）。
10. `onchina` 平台会员相关测试：随数据源改常量同步调整断言。

### 17.4 预计修改目录

- `citizenchain/runtime/misc/square-post/src/`：代码 + 中文注释；平台 CID 改读创世常量、删 `PlatformCidNumber` 存储、新增 `genesis_build` 播种三档价、**删除整个 `migration.rs` 与迁移 hooks/`MigrationBlocked`/`ensure_subscription_ready`**。边界——不改订阅状态机、不改 call index、不改 `SubscriptionState` SCALE 布局。
- `citizenchain/runtime/misc/square-post/src/tests/`：测试；去掉 CID 手工 `put`、删迁移单测、补创世播种回归。
- `citizenchain/onchina/src/core/`、`citizenchain/onchina/src/domains/membership/`：代码 + 残留清理；平台 CID 读取从存储键改创世常量，控制台平台价页维持 finalized 价格快照读取。
- `memory/01-architecture/gmb/`、`memory/04-decisions/ADR-037-citizen-coin-native-membership.md`、本任务卡：文档；记录平台 CID 单源=创世公民链基金会常量、存储契约删 `PlatformCidNumber`、价格创世播种、迁移机制删除。

### 17.5 不修改范围

- 不改订阅状态机、call index、`SubscriptionState` SCALE 布局、收费路由、创作者订阅逻辑。
- 本步按重新创世模型落地（覆盖第 1 节原地升级约束，见 17.1）；不写生产链数据库。
- 不新增 pallet、不新增 extrinsic、不改统一投票引擎、不引入兼容旧存储分支、不保留任何迁移代码。
- 不 Git push、不 PR、不远端 workflow、不生产部署。

### 17.6 验收

- 重新创世后平台 `subscribe` 不再报 `PlatformNotBound`；收款方正确派生到公民链基金会费用账户。
- 三档 `PlatformPrice` 创世后确定存在默认值；统一内部投票调价仍生效。
- 全仓无 `PlatformCidNumber` 读写残留（含 onchina）；`migration.rs`、迁移 hooks、`MigrationBlocked`、`ensure_subscription_ready` 全部删除，无编译引用残留。
- `square-post` 单元测试、`genesis_build` 播种回归、runtime 全量单测通过；无 try-runtime 迁移校验残留。
- onchina 平台价页在真实本地链读到常量 CID + finalized 三档价；未绑定态相关旧分支已删。
- 金标 SCALE 向量逐字节不变。

### 17.7 待确认点

- 平台 CID 采用「pallet 直读创世常量」而非「Config provider 注入」：前者最小 diff、与 configs.rs/onchina 同口径；如需保持 pallet 与具体机构解耦，可改 provider 注入（多一处 Config + runtime wiring）。默认按直读常量执行。

### 17.8 第 5.5 步执行记录（2026-07-19）

- 平台 CID 收敛为创世常量：`subscription.rs::current_price_and_payee` 与 `proposal.rs`（`propose_price_change` 用 `is_citizenchain_foundation_identity`，回调用常量 CID 断言）改读 `CITIZENCHAIN_FOUNDATION.cid_number`；`lib.rs` 删除 `PlatformCidNumber` StorageValue；`PlatformNotBound` 语义收窄为「费用账户不可派生」。
- 价格创世播种：`lib.rs` 新增 `#[pallet::genesis_config]` + `#[pallet::genesis_build]`，创世无条件写三档默认价；`square-post` 加 `serde` 依赖。
- 迁移机制整块删除：删 `migration.rs`；`lib.rs` 删 `on_runtime_upgrade/pre/post_upgrade` 三 hooks、`MigrationBlocked` 存储、`MigrationIncomplete` 错误；`subscription.rs` 删 `ensure_subscription_ready`；`billing.rs` 删四处调用；`RenewalSchedule/RenewalIndex` 一致性不变量保留并移入 `lib.rs` 的 `try_state`。`runtime/src/lib.rs`、`configs.rs` 迁移相关注释同步更新。
- OnChina 联动：`chain_runtime.rs::fetch_platform_membership_snapshot` 不再读 `PlatformCidNumber` 存储键，`platform_cid_number` 取自创世常量；仅批量读三档 finalized 价；删除随之无用的 `decode_scale_vec_string` 及其测试断言。
- 验收：`square-post` 15 项单测通过（含 `genesis_build_seeds_default_platform_prices` 与宏生成的 `test_genesis_config_builds`）；`try-runtime` 特性编译通过；`onchina` 编译 + `chain_runtime`/`membership` 测试通过；`citizenchain` 12 项创世测试通过（含 serde 往返 `genesis_json_deserializes_into_runtime_genesis_config`）；全仓 `PlatformCidNumber` 零 `.rs` 残留。金标 SCALE 向量（`SubscriptionState` 布局未变）不受影响。
- 未触碰的并行改动：`runtime/votingengine/` 有他人未提交修改（另一任务），其 `runtime-benchmarks` 编译报错属该任务，非本步引入；按任务卡第 8 节不暂存、不提交、不回退他人工作。
- 未执行：Git 提交/推送、链上升级、chainspec 重建、生产部署（重新创世由用户在部署阶段执行）。

## 18. 第 5.6 步技术方案：订阅生命周期重构（等待执行确认，runtime 二次确认）

Review 采纳的四项决策 + 出块参数确定后，对 square-post 订阅生命周期重构。

### 18.1 链参数事实（已核实）

- 出块目标 6 分钟（`POW_TARGET_BLOCK_TIME_MS=360_000`）。
- 块权重预算 = 60 秒执行时间（`60×WEIGHT_REF_TIME_PER_SECOND`），proof_size 无界。
- 块大小 100MB（`MAX_BLOCK_BYTES`）**只约束交易字节**；自动续费在区块钩子内部执行、不产交易字节，**不受块大小约束**，只受块权重约束。
- 单块续费吞吐经该权重预算约 2 万–5 万笔（基准待测），非现值 64。

### 18.2 状态模型（新）

- `Active`：正常自动续费，在续费调度内。
- `Cancelled`：停续费，权益至 `paid_until`。
- `Suspended`（挂起，新）：权益暂停、粉丝关系保留、**退出自动调度**；触发＝①所属创作者计划价格/周期变更且未再签名（`NeedReconsent`）②续费时余额不足（`Insufficient`）；恢复＝用户主动再签名 / 充值后再签一次。
- `CreatorPaused`（创作者停扣，新）：创作者掉平台会员→其全部粉丝暂停扣费，计划/粉丝/订阅全保留，退出调度；创作者恢复平台会员即自动恢复 `Active` 并重新入调度，**向后续扣、不补扣暂停期**。
- `Terminated`：仅保留给显式关闭/清档，**不再由余额不足或掉会员自动进入**。

### 18.3 换挡 = 立即折算（方案 A + 降档延时）

按毫秒精确：剩余权益 `y = 已授权价 × (paid_until − now) ÷ (paid_until − last_charged_at)`。

- 升档（新价 > y）：立即扣 `新价 − y`；`new paid_until = add_calendar_period(now, 新周期)`；下期按新价整周期。
- 降档（新价 ≤ y）：不扣不退；剩余信用 `(y − 新价)` 按新档单价折算成额外时长叠加：`new paid_until = add_calendar_period(now, 新周期) + 额外天数`，`额外天数 = (y−新价) × 新周期天数 ÷ 新价`（向下取整到毫秒）。
- 校验：目标计划存在、创作者为有效平台会员、`expected_price == 当前价`（签名防漂移）。

### 18.4 再签名恢复（re-consent，仅创作者计划）

- 计划价/周期变更后：订阅记录「已授权价/周期」与当前链上计划不一致 → 续费时判 `NeedReconsent` 挂起。
- 到期前再签名：更新已授权价、保持 `Active`、当前周期 honored、下期按新价扣（无即时扣款、无掉权）。
- 已挂起后再签名：按新价立即扣、新周期从现在起算（等同重新订阅）。
- **平台治理改价：不触发挂起，自动按新价续**（决策 a：治理可信，前端公示即可）。

### 18.5 续费扩容（#6）

- 续费排空从 `on_finalize` 固定 64 改为 `on_idle(n, remaining_weight)` 按当块剩余块权重尽量排空：每笔用基准权重估算，循环处理到期任务至剩余权重不足留安全边际；附高位硬 backstop 常量。
- 删除 `on_initialize` 静态最坏权重预留。
- `benchmarking.rs` 补单笔续费权重基准，落地 `SubstrateWeight`。
- 挂起/缺钱订阅已退出调度、不占每块预算；稳态每块仅数百，超大同刻促发按块权重数万/块、多块分摊（138 万同刻约数十块≈1–5 小时排空，期间未扣到者短暂掉权、扣到即恢复）。

### 18.6 预计修改目录

- `citizenchain/runtime/misc/square-post/src/`：代码＋中文注释；状态机/钩子/事件、换挡折算与挂起判定、续费/恢复、权重基准。核心。
- `citizenchain/runtime/src/configs.rs`：`MaxSubscriptionRenewalsPerBlock` backstop 值。
- `citizenchain/runtime/misc/square-post/src/tests/`：补齐全部新语义 + 治理调价执行路径测试（#3）。
- `citizenchain/runtime/misc/square-post/tests/fixtures/`：`SubscriptionState` 新字段 → 金标 SCALE 向量重生。
- 端侧（`citizenapp/`、`citizenapp/cloudflare/`、`citizenchain/onchina/`、`citizenwallet/`）：解码新状态/字段同步——**单独评估，可能纳入后续步**，本步先收敛链端。

### 18.7 风险与前置

- `SubscriptionState` 新增字段（`authorized_price_fen`、`suspend_reason` 等）→ SCALE 布局变更 → **五端金标向量重生 + 五端解码同步**。开发期零用户、重新创世，无迁移。
- `on_idle` 读时间戳：inherent 已于 `on_idle` 前写入，可用 `now_ms`。
- 平台/创作者换挡折算共用同一原语；平台周期恒月、创作者按所选月/季/年。

### 18.8 分步实施顺序（每子步：出方案 → 确认 → 执行 → 更文档/清残留 → 出下一子步方案）

- **5.6a 状态与字段模型（SCALE 契约定稿）**：扩 `SubscriptionState`（`authorized_price_fen`、`suspend_reason`）、加 `SubscriptionStatus::Suspended` 与 `SuspendReason`；行为不变，仅承载新字段；金标向量重生；测试保持全绿。
- **5.6b 换挡立即折算**：`do_change_subscription_plan` 改为升档扣差 / 降档延时的即时折算原语 + 测试。
- **5.6c 挂起与恢复**：续费判定改价未签名 / 余额不足 → `Suspended` 退出调度；再签名 / 充值再签恢复；平台治理改价自动续不挂起 + 测试。
- **5.6d 创作者掉会员暂停粉丝**：判定创作者非有效平台会员 → 粉丝 `CreatorPaused` 暂停、可恢复（含恢复触发机制设计）+ 测试。
- **5.6e 续费扩容 + 治理测试**：`on_idle` 按块权重排空、删固定 64 与静态预留、权重基准；补齐 `proposal.rs` 治理调价执行路径测试；残留清理。
- **5.7 五端解码同步（收尾）**：citizenapp / cloudflare / onchina / citizenwallet 按新 `SubscriptionState` 布局与状态同步；纳入第 6 步或单列。

### 18.9 子步 5.6a 执行记录（2026-07-19）

- `subscription.rs`：`SubscriptionStatus` 追加 `Suspended=3`；新增 `SuspendReason`（`NeedReconsent=0/InsufficientBalance=1/CreatorIneligible=2`）；`SubscriptionState` 末尾加 `authorized_price_fen:u128` + `suspend_reason:Option<SuspendReason>`。`lib.rs` 导出 `SuspendReason`，`try_state` 把 `Suspended` 并入「非 Active 不得在调度」不变量。
- `billing.rs`：所有写状态处补 `authorized_price_fen`（=扣款价）与 `suspend_reason`（首扣/续费/恢复/换挡/取消=None；三个 Terminated 分支=None）。行为等价，未引入任何 Suspended 转移（留 5.6c）；未删 `pending_plan`（留 5.6b）。
- `benchmarking.rs`、`tests/mod.rs`：状态字面量补两字段；新增断言「首扣后 authorized_price_fen==扣款价、suspend_reason==None」。
- 金标向量：`subscription_scale_vectors.json` 重生 `state_platform` 与 `state_platform_spark_active`（尾部 +16B 已授权价 +1B None）；补 `status_suspended` 与三个 `SuspendReason` 向量（供 5.7 端侧对齐）。
- 验收：`cargo test -p square-post` 15/15 绿；`cargo check -p citizenchain`、`--features try-runtime`、`-p onchina` 均通过。`runtime-benchmarks` 因 `votingengine` 他人未提交改动报错（非本步），未跑。
- 未删 `pending_plan`；SCALE 布局在 5.6b 换挡改即时后将再次调整并重生向量（开发期零用户、重新创世，无迁移）。

### 18.10 子步 5.6b 执行记录（2026-07-19）

- `subscription.rs`：删除 `SubscriptionState.pending_plan` 字段。
- `billing.rs`：`do_change_subscription_plan` 重写为即时折算（包 `with_storage_layer`）——剩余权益 `y = 已授权价 × (paid_until−now) ÷ (paid_until−last_charged_at)`（仅 Active/Cancelled 未到期时，否则 0）；升档扣 `新价−y`、`paid_until = add_calendar_period(now,新周期)`；降档不扣、`extra_ms = (y−新价) × 周期ms ÷ 新价`、`paid_until = base + extra_ms`；新增 `remaining_credit` 原语。同步清 `charge_and_schedule`/`process_one_due`/`do_cancel`/`do_subscribe` 恢复分支的 `pending_plan`；续费恒用 `state.plan`。
- `lib.rs`：事件 `SubscriptionPlanChangePending` → `SubscriptionPlanChanged{subscriber,issuer,new_plan,charged_now,paid_until}`。
- `benchmarking.rs`/`tests`：清 `pending_plan` 字面量；重写旧 pending 测试为 `change_plan_upgrade_charges_difference_immediately`、新增 `change_plan_downgrade_extends_duration`、`change_plan_prorates_partial_remaining_credit`。
- 金标向量：`state_platform` / `state_platform_spark_active` 去掉 `pending_plan` 的 Option None 字节后重生。
- 验收：`cargo test -p square-post` 17/17 绿；`cargo check -p citizenchain`、`--features try-runtime`、`-p onchina` 通过；全仓 `pending_plan`/`SubscriptionPlanChangePending` 零 `.rs` 残留。

### 18.11 子步 5.6c 执行记录（2026-07-19）

- `lib.rs`：事件增 `SubscriptionSuspended{subscriber,issuer,reason,suspended_at}`、`SubscriptionReconsented{subscriber,issuer,authorized_price_fen}`；删 `SubscriptionPaymentFailed`；保留 `SubscriptionRenewalStopped`（仅真终止）。
- `billing.rs`：新增 `suspend_subscription` 原语。`process_one_due` 重写——创作者 `price≠authorized_price_fen` 或 `Err(CreatorPlanNotFound)` → `Suspended(NeedReconsent)`；转账失败 → `Suspended(InsufficientBalance)`；`CreatorNotPlatformMember` 等其它 Err → `Terminated`+`SubscriptionRenewalStopped`（`CreatorPaused` 留 5.6d）；平台恒按当前价扣、`authorized` 更新（治理改价自动续、不挂起）。`do_subscribe` Active 分支加创作者「到期前再签名」：`当前价≠authorized` 时校验 `expected==当前价` 后仅更新 `authorized`、保持 Active、不扣、发 `SubscriptionReconsented`。挂起态天然经 `charge_and_schedule` 恢复（无需改主流程）。
- 测试：`payment_failure_terminates_...` → `payment_failure_suspends_and_removes_schedule`；旧 `creator_subscription_renews_..._current_chain_price`（曾断言创作者改价自动扣新价）改为 `creator_subscription_renews_at_authorized_price_when_unchanged`；新增 `creator_price_change_suspends_renewal_until_reconsent`、`creator_price_change_reconsent_before_lapse_keeps_active_without_charge`。
- 验收：`cargo test -p square-post` 19/19 绿；`cargo check -p citizenchain`、`--features try-runtime`、`-p onchina` 通过；`SubscriptionPaymentFailed` 零残留。`SubscriptionState` 布局未变、金标向量无需重生。

### 18.12 子步 5.6d 执行记录（2026-07-19，恢复机制＝保留调度周期重试）

- `subscription.rs`：加 `SubscriptionStatus::CreatorPaused=4`（留调度）；删未用的 `SuspendReason::CreatorIneligible`。
- `billing.rs`：`process_one_due` 守卫改 `matches!(Active|CreatorPaused)` 并去掉 `paid_until==due_at` 绑定；新增 `Err(CreatorNotPlatformMember)` 分支 → 置 `CreatorPaused`、不扣、不终止、不推进 `paid_until`、`schedule_renewal` 重排到下周期重试、发 `SubscriptionCreatorPaused`；创作者恢复后下次到期正常扣款回 `Active`（若同时改价则先 `NeedReconsent` 挂起）。
- `lib.rs`：加事件 `SubscriptionCreatorPaused{subscriber,issuer,paused_at}`；`try_state` 放宽为纯双向一致（`Active|CreatorPaused` ⇒ 有调度项且双向一致，不再绑定 `paid_until`）。
- 测试：`creator_loses_membership_pauses_fans_and_resumes`（掉会员→CreatorPaused 留调度未扣未终止；恢复→回 Active 扣款）、`creator_paused_and_repriced_suspends_for_reconsent_on_return`（暂停+改价恢复→先 NeedReconsent 挂起离调度）。
- 金标向量：`SubscriptionState`（Active+None）编码不变，`state_platform` 无需重生；仅枚举向量删 `suspend_reason_creator_ineligible`、加 `status_creator_paused`（`04`）。
- 验收：`cargo test -p square-post` 21/21 绿；`cargo check -p citizenchain`、`--features try-runtime`、`-p onchina` 通过；`CreatorIneligible` 零残留。

### 18.13 子步 5.6e 执行记录（2026-07-19）

- 续费扩容：`lib.rs` Hooks 删 `on_initialize`/`on_finalize`，改 `on_idle(n, remaining_weight)` 按当块剩余权重排空（`limit = min(remaining/per, backstop)`；`()` 未计量权重时按 backstop 排空，兼容测试）；`weights.rs` `on_initialize(renewals)` → `process_one_due()`（85M+8r/7w）；`configs.rs` `MaxSubscriptionRenewalsPerBlock` `64 → 50_000`；`benchmarking.rs` 加 `process_one_due` 基准（受并行 votingengine 阻断未跑）。
- 残留清理：`billing.rs` `charge_and_schedule` 删恒真的 `reset_started_at`，`started_at` 恒取 `now`。
- 治理测试（#3 可测部分）：`propose_platform_price_rejects_zero_price`、`propose_platform_price_rejects_non_foundation_institution`（验证 5.5 的 `is_citizenchain_foundation_identity` 断言）。**完整「投票通过→执行→改价」路径需 votingengine 集成夹具（测试 runtime impl `votingengine::Config`），当前受并行 votingengine 未提交改动阻断，列为跟进项。**
- 测试夹具：`finalize_at` 改调 `on_idle(block, Weight::MAX)`。
- 验收：`cargo test -p square-post` 23/23 绿；`cargo check -p citizenchain`、`--features try-runtime`、`-p onchina` 通过；`on_initialize`/`on_finalize`/`reset_started_at` 零残留。

**链端子步 5.6a–5.6e 全部完成。** 剩 5.7 客户端解码同步。

## 19. 第 5.7 步技术方案：客户端解码同步（等待执行确认）

### 19.1 范围收敛（勘察结论）
只有 **citizenapp(Dart) + cloudflare(TS)** 解码 `SubscriptionState`；`onchina/citizenwallet` 对其零解码引用（onchina 只读平台价快照，5.5 已改；citizenwallet 只冷签治理）→ 二者无需改。「五端」实为两端。

### 19.2 新 SCALE 布局（金标真源 `subscription_scale_vectors.json`）
`plan / started_at / last_charged_at / last_charged_price_fen(u128) / paid_until / subscription_status(u8 0-4) / authorized_price_fen(u128，新) / suspend_reason(Option<u8>，新)`；删 `pending_plan`；状态 `3=suspended、4=creatorPaused`；`suspend_reason` 值 `0=needReconsent、1=insufficientBalance`。

### 19.3 A. citizenapp（Dart）
- `lib/rpc/subscription_rpc.dart`：`ChainSubscriptionState` 删 `pendingPlan`、加 `authorizedPriceFen:BigInt` + `suspendReason:String?`；`decodeSubscriptionState` 删 pending 读、status switch 加 3/4、status 后读 `authorized_price_fen`(u128)+`suspend_reason`(Option)；`isEffectiveAt` 维持 active||cancelled。
- `lib/my/membership/`（`subscription_service.dart`/`membership_page.dart`）：删 pendingPlan 引用；suspended（提示重新签名/充值）、creatorPaused（创作者暂停恢复中）状态展示；换挡文案改「立即生效」。
- `test/rpc/subscription_rpc_test.dart`：金标向量按新布局逐字节对齐 + suspended/creatorPaused/authorized/suspend_reason 断言。

### 19.4 B. cloudflare（TS）
- `src/chain/subscription.ts`：`SubscriptionStatus` 加 `'suspended'|'creatorPaused'`、`STATUS_BY_BYTE` 加 3/4、删 pendingPlan、status 后消费 `authorized_price_fen`(u128)+`suspend_reason`(Option)。
- `migrations/0001_square_core.sql`：两处 `subscription_status CHECK(... IN)` 加 `'suspended','creatorPaused'`（开发期零用户，直接改建表/重建 D1；核对是否有 `pending_plan` 列需删）。
- `src/membership/reconcile.ts`：对账候选纳入 `suspended/creatorPaused`（CreatorPaused 链上自动恢复 Active，镜像须刷新，否则暂停订阅镜像永不回 active）。
- `src/membership/service.ts` 门禁：维持只认 active/cancelled（suspended/creatorPaused 自然拒绝），仅加注释。
- `test/chain_subscription.test.ts`：金标向量与断言同步。

### 19.5 C. onchina / citizenwallet
核对确认无 SubscriptionState 解码 → 无改动，仅在执行记录声明。

### 19.6 预计修改目录
- `citizenapp/lib/rpc/`、`citizenapp/lib/my/membership/`、`citizenapp/test/rpc/`
- `citizenapp/cloudflare/src/chain/`、`.../membership/`、`.../migrations/`、`.../test/`
- 本任务卡 5.7 执行记录

### 19.7 不修改 / 验收
- 不改链端（已完成）、不改 onchina/citizenwallet、不部署、不写生产 D1。
- citizenapp：`flutter test test/rpc/subscription_rpc_test.dart` 绿 + `flutter analyze` 无问题，金标向量与 runtime 逐字节一致。
- cloudflare：`npm test`（chain_subscription+membership）绿 + tsc 通过，真实本地 D1 迁移含新状态。
- 两端 `pending_plan` 零残留。
- 执行顺序：先 citizenapp（5.7a）验收，再 cloudflare（5.7b）验收。

### 19.8 子步 5.7a 执行记录（citizenapp，2026-07-19）

- `lib/rpc/subscription_rpc.dart`：`ChainSubscriptionState` 删 `pendingPlan`、加 `authorizedPriceFen:BigInt` + `suspendReason:String?`；`decodeSubscriptionState` 删 pending 读、status switch 加 `3→suspended/4→creatorPaused`、status 后读 `authorized_price_fen`(u128) + `suspend_reason`(Option:0→null,1→0=needReconsent/1=insufficientBalance)；`isEffectiveAt` 维持 active||cancelled。
- `lib/my/membership/membership_page.dart`：横幅状态标签 switch 加 `suspended`（已挂起·待重新签名或充值）、`creatorPaused`（创作者暂停·恢复后自动续）两条（仅此 2 行，未碰同文件并行的价格面板改动）。
- 测试：`test/rpc/subscription_rpc_test.dart` state 向量对齐 runtime 金标（新布局）+ 新增 suspended/creatorPaused 解码断言；`test/my/creator/creator_api_test.dart`、`test/my/membership/membership_page_test.dart` 的 `ChainSubscriptionState` 构造去 pendingPlan、补两字段。
- 验收：`flutter analyze`（订阅相关文件）无问题；`flutter test test/rpc/subscription_rpc_test.dart test/my/creator/creator_api_test.dart` 全绿；membership_page_test 状态标签相关用例通过。
- 已知非本步失败：`membership_page_test` 两条「公民币月价来自链读/兜底名称」断言 `299 公民币/月` 失败——`membership_page.dart` 有并行未提交大改（168+/57-）把价格显示改为「元/月两位小数」但未同步这两条断言；属并行任务遗留，非本步引入，未处理。

### 19.9 子步 5.7b 执行记录（cloudflare，2026-07-19）

- `src/chain/subscription.ts`：`SubscriptionStatus` 加 `suspended/creatorPaused`；新增 `SuspendReason` 类型；`STATUS_BY_BYTE` 加 3/4；`ChainSubscriptionState` 删 `pendingPlan`、加 `authorizedPriceFen:bigint` + `suspendReason:SuspendReason|null`；`decodeSubscriptionState` 删 pending 读、status 后消费 `authorized_price_fen`(u128)+`suspend_reason`(Option:0→null,1→0=needReconsent/1=insufficientBalance)，补截断/非法标记校验。
- `migrations/0001_square_core.sql`：两处 `subscription_status CHECK` 加 `'suspended','creatorPaused'`；**删除死列** `pending_membership_level`、`pending_tier_id`、`pending_billing_period`（换挡即时生效、无 pending；开发期零用户直接改建表）。
- `src/membership/reconcile.ts`：平台/创作者对账候选改 `subscription_status IN ('active','suspended','creatorPaused')`（CreatorPaused 链上自动恢复须刷新镜像）；两处 apply 删 pending 列写入。
- `src/membership/citizen_coin.ts`、`creator.ts`：删 `state.pendingPlan` 幂等判定（换挡即时→确认后 plan 已是目标档）、删镜像 INSERT/UPDATE 的 pending 列；`service.ts` SELECT 去 `pending_membership_level`；`types.ts` `MembershipRow` 去该字段。
- 测试：`chain_subscription.test.ts` 金标向量对齐 runtime 新布局 + suspended/creatorPaused/terminated/非法 suspend_reason 断言；`membership.test.ts`、`membership_reconcile.test.ts`、`creator_reconcile.test.ts`、`chain_confirm.test.ts` 去 pending 字段并修正因删列而位移的 mock bind 下标。
- onchina/citizenwallet：grep 确认无 `SubscriptionState` 解码引用 → 无改动。
- 验收：`npx tsc --noEmit` 通过；`npx vitest run` **26 文件 / 155 测试全绿**；两端源码 `pending*` 零残留。

**5.7（citizenapp + cloudflare）客户端解码同步完成。** 剩：治理完整路径集成测试（votingengine 修复后已解锁）、第 6 步跨端真实运行态验收 + 归档。

### 19.10 第 6 步（可自动化部分）执行记录（2026-07-19）

合并大量并行改动（投票引擎重构/修复、管理员统一、公民链基金会更名）后做全栈一致性验证与残留审计：

- **全栈全绿**：`cargo test -p square-post` 23/23；`cargo check -p square-post --features runtime-benchmarks` 通过（votingengine 修复已解锁，我加的 `process_one_due` 基准有效）；`cargo test -p citizenchain --lib genesis` 14/14（`genesis_build` 播种完好）；cloudflare `vitest run` 26 文件 155 测试全绿 + `tsc` 通过；citizenapp `membership_page_test` 15/15、`subscription_rpc_test`/`creator_api_test` 全绿 + `analyze` 无问题。
- **残留审计（全仓零命中）**：`BillingKeeper`/`charge_due`/外部 renew、`stripe`/`prepaid`/`usdc` 旧支付、`pending_plan`/`SubscriptionPaymentFailed`/`SubscriptionPlanChangePending`。
- **架构文档同步**：`memory/01-architecture/gmb/subscription-part1-tech.md` 更新为最终 as-built——状态枚举加 `Suspended/CreatorPaused` + `SuspendReason`、`SubscriptionState` 去 `pending_plan` 加 `authorized_price_fen`/`suspend_reason`；存储去 `PlatformCidNumber`/`MigrationBlocked`（平台 CID＝创世常量）；续费改 `on_idle` 按块权重排空；换套餐改即时折算；失败按原因分流为挂起/暂停/终止。
- **治理完整路径集成测试**：判定不宜在 square-post 单测搭重型 votingengine mock（Config ~74 关联类型 + 并行改为岗位授权/`VotePlanOf`）；应放 runtime 级集成套件（所有 pallet 真实）——列为跟进项，归属 votingengine 集成套件。

### 尚未完成（需运行环境/跨团队）

- 治理「投票通过→执行→改价」完整集成测试（runtime 级，跟进）。
- 跨端真实运行态验收（本地链 + Worker/D1 + CitizenApp/CitizenWallet 真机联调）——需人工运行环境，未在本窗口执行。
- 任务卡归档：待上述真实验收完成后由 `open/` 移入完成态。
