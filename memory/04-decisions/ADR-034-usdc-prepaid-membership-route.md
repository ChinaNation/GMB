# ADR-034 USDC 预付会员路线 + 统一「从到期日往后叠」切换规则

## 标题

虚拟货币(USDC)会员改为预付固定时长路线,与卡自动续订并存;一钱包一路线;切换支付按「从当前到期日往后叠」处理,零权益损失。

## 背景

ADR-033 的会员订阅按 Stripe subscription(自动续费 + proration 换档)建成,已过真实 Stripe 测试验证。但**加密钱包无法被 off-session 静默扣款**(链上转账须本人签发)——Stripe 冒烟坐实:无可自动扣款方式时 `subscriptions.update` 升档直接 400。故 USDC **无法自动续费、无法自动升/降档**,不能沿用卡的 subscription 模型。

## 决策

USDC 走一条**预付固定时长**路线,与卡路线并存,一钱包同时只一条路线、一个订阅、一个支付凭证。

**1. 数据模型(复用 `square_memberships` 一行/钱包)**
- 用 `subscription_source` 区分:`'stripe'`(卡自动续) / `'usdc_prepaid'`(预付)。
- USDC 行:无自动续、无 `cancel_at_period_end`;`expires_at` = 到期时刻即权益真源;`current_period_start` = 激活时刻(供展示起止);预付凭证 id(payment_intent)入库,`client_reference_id=钱包` 唯一绑定。
- **USDC 行「是否有效」只看 `expires_at > now`**,不进 Stripe 状态生命周期。

**2. 时长与定价(无折扣)**
- 档期:季 = 3 月、年 = 12 月。
- 价 = 月数 × 月费,用 Stripe 一次性 `price_data`(`unit_amount = 月数 × 月费分`)动态生成,不预建价、不用 coupon。

**3. 购买流程**
- 选档 + 时长 + USDC → 钱包签名授权(复用 `subscribe_membership` 签名动作,`context` 由 `level` 扩为 `level|duration` 字符串——**不改 0x1D payload 布局、不动三端字节**)。
- 验签 → Stripe 一次性 Checkout(`mode=payment`),`metadata={owner, level, duration, route:'usdc_prepaid'}`。
- 付成功 → **webhook `checkout.session.completed` 新增预付分支**授时长:`expires_at = max(now, 现expires_at) + N 月`、`membership_level=level`、`current_period_start=激活`。(现 webhook 显式忽略该事件,卡靠 subscription 事件;USDC 无 subscription,必须在此授权。)

**4. 换挡(同 USDC 路线内,均需签名;按实际日历天:剩余天数 = 实际 (expires_at − now) 天)**
- 升档:补差价 = `剩余天数 ×(新−旧)月费 × 12 ÷ 365`(实际年折日费)→ 一次性 USDC Checkout;付成功切档、`expires_at` 不变。
- 降档:不退钱 → `新天数 = 剩余天数 × 旧月费 ÷ 新月费`(比值,与年基无关)→ **新到期 = now + 新天数**(剩余高档时间整额换成低档时长,非在原到期上叠加),切低档。纯本地算,无 Stripe。

**5. 切换支付 = 从当前到期日往后叠(统一规则)**
两种支付「取消」都是留权益到到期,故新路线权益从 `max(now, 当前 expires_at)` 起算,旧的照用到到期、新的接后面,**零损失、无跨路线折算**:
- **USDC → 卡**:建卡订阅 `trial_end = 当前 expires_at`(试用期=USDC 剩余,试用满即首次扣费);该行归属卡订阅。
- **卡 → USDC**:卡订阅设 `cancel_at_period_end=true` + USDC `expires_at = 旧 expires_at + N 月`;**清该行 `stripe_subscription_id`**(免旧卡订阅到期的 `subscription.deleted` 事件误置该行失效)。

**6. 冻结 / 身份绑定(复用第 2 期,零改动兼容)**
- 身份精确匹配冻结逻辑两路线通用(真源 = `expires_at` + 身份匹配)。
- `syncCollectionState` 已 `if (!stripe_subscription_id) return` → USDC 无 sub id **天然跳过 pause/resume**;冻结即「权益不可用 + 到期不续」,无扣款可暂停。

**7. App 展示**:会员卡显示订阅起止 `current_period_start ~ expires_at`(两路线通用) + 路线标签(预付 / 自动续费);`hasSubscriptionWindow` 门控(已支付且起止齐备才显示)。

**8. 取消识别(段4)**:`/cancel` 一个入口,验签后按 `subscription_source` 分派回 `cancel_kind`——卡=`stripe`(`cancel_at_period_end` 到期取消)、USDC=`usdc_prepaid`(无自动续、到期自然失效,不动订阅);无活跃订阅 `no_active_subscription`。官网据 `cancel_kind` 出文案。

**9. 购买异档守卫(段4)**:`/prepaid` 购买对「已有活跃 USDC 且档不同」拒(`prepaid_tier_change_required`,挑战前+确认后两处),强制走换档折算,防旧时长被贴成新档漏收;新购 / 同档续费 / 跨支付切换(原为卡)放行。

**10. webhook 幂等与跨路线乱序保护**
- `square_stripe_webhook_events` 先按 `event_id` 原子占位，处理成功后写 `processed_at`；处理中失败释放占位，允许 Stripe 重试。`square_stripe_payments` 以 `stripe_payment_intent_id` 永久去重，一次性付款事件重放不得重复授时长。
- 卡→USDC 后，旧卡的 `customer.subscription.updated/deleted` 可能晚于 Crypto Checkout 到达。`square_memberships.subscription_source='usdc_prepaid'` 是当前真源；普通旧卡事件的 D1 upsert 必须原子拒绝并返回 `subscription_superseded`，不得覆盖预付行。
- USDC→卡只能由服务端创建的新订阅越过上述守卫：Checkout 写入 `subscription_data.metadata.payment_switch=usdc_to_stripe`，webhook 校验该精确标志后才允许把行切回 `stripe`。客户端不得自行提供此标志。

## 影响

- 影响产品:Cloudflare Worker(预付购买路由、webhook checkout 授时长分支、换挡重算、切换支付、subscriptionIsActive 对 usdc 分支)、官网(USDC 选时长 + 预付发起 + 换挡预览)、CitizenApp(会员卡起止时间 + USDC 入口)。
- D1:`square_memberships` 记预付凭证列(或复用现有列);开发期零用户,直接改基线 `0001_square_core.sql` 重建。
- 契约不破:op_tag 0x1D / QR_V1 / owner_account 单行 / 一钱包一订阅 均不变。
- 链:零改动。

## 备选方案

- USDC 也做 Stripe subscription 自动续:否。加密钱包不能 off-session 扣款,冒烟已证 400。
- 切换支付时旧路线剩余价值作废 / 跨路线折算:否。改用「从到期日往后叠」,更简单且零损失。
- 年/季打折:否(本次无折扣)。

## 后续动作

- 任务卡:`memory/08-tasks/open/20260713-usdc-prepaid-membership-route.md`(购买 / 换挡 / 切换 / 前端四段**全部完成**)。
- 真实验收:2026-07-14 已在独立 Stripe Sandbox + Sepolia USDC 完成 8.97 美元 Crypto Checkout，确认可取得 `payment_intent` 并落入付款去重表；真实 event 连续重放两次未延长 D1 时长。随后 USDC→卡 Checkout 以 122 天 trial 完成切换，真签名取消通过，整车日志为 `PASS=8 / BLOCKED=0 / FAIL=0`。
- 未覆盖:voting/candidate 成功身份与身份不匹配冻结/解冻仍需真实链上身份钱包，不得引用本次访客钱包结果替代。
- 已同步:CITIZENWEB 会员章节、App 会员卡、本 ADR。
- 部署:Worker `wrangler deploy`;D1 基线重建含 `prepaid_payment_ref`;官网 `wrangler pages deploy`。
