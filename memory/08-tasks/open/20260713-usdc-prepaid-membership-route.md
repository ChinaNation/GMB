# USDC 预付会员路线 + 从到期日往后叠切换

任务需求：
按 ADR-034 实现 USDC 预付会员路线(季/年一次性购买、无自动续、按时长授权),与卡自动续订并存;一钱包一路线;切换支付统一「从当前到期日往后叠」。

所属模块：membership（Cloudflare Worker 主导）+ 官网 citizenweb + CitizenApp

输入文档：
- memory/04-decisions/ADR-034-usdc-prepaid-membership-route.md
- memory/04-decisions/ADR-033-membership-subscription-lifecycle.md
- citizenapp/cloudflare/src/membership/*（subscribe/service/plans/stripe_api/webhook）
- citizenapp/cloudflare/src/account/action_challenge.ts
- citizenweb/src/pages/Membership.tsx
- citizenapp/lib/my/membership/membership_page.dart

必须遵守：
- 不可突破模块边界（权益真源=会员表 expires_at + 身份匹配；USDC 授权益走 webhook checkout.session.completed）
- 不可绕过既有契约（op_tag 0x1D / QR_V1 / owner_account 单行 / 一钱包一订阅）
- USDC 无自动续、无自动扣款；无 stripe_subscription_id；有效性只看 expires_at>now
- 切换支付按「从 max(now,现expires_at) 往后叠」，零权益损失（不作废、不跨路线折算）
- context 扩为 level|duration 字符串，不改 0x1D payload 字节布局
- 定价无折扣（月数×月费，动态 price_data）
- 链零改动；不清楚 Stripe 行为先查文档/沙箱冒烟，不猜

输出物：
- 代码（Worker 预付购买/换挡/切换 + 官网/App）
- 中文注释
- 测试（vitest 覆盖 购买授时长/升补钱/降补时长/两向切换叠加）
- 文档更新（CITIZENWEB 会员章节 + memory 会员模块）
- 残留清理

验收标准：
- 功能可运行（Worker tsc+vitest 全绿；官网 tsc/eslint；App analyze/test）
- USDC 购买按季/年授对时长；换挡补钱/补时长正确
- 切换支付新路线从旧到期日起算、权益连续无损
- 冻结/身份对 USDC 复用生效、无 pause 副作用
- 文档已更新、残留已清理

## 分段

- [x] 段1 购买：D1 加 prepaid_payment_ref(基线重建)；prepaid.ts 购买路由(签名 context=level|duration → 一次性 Checkout price_data，月数×月费)；webhook checkout.session.completed 预付授时长(叠加 addMonths)；subscriptionIsActive 对 usdc_prepaid 只看 expires_at；upsertPrepaidMembership；限流目录注册 prepaid 路由。测试 149 全绿(challenge/confirm/时长校验/webhook 授时长/卡checkout不误授/subscriptionIsActive)
- [x] 段2 换挡：change/challenge + change 路由(签名 context=change|target)；降档/同价本地折算即时切档(新到期=now+折算天数)；升档补差价(实际年365折日费)建一次性 Checkout→webhook usdc_prepaid_upgrade 只切 level;applyPrepaidTierChange;createOneTimeCheckout 共用。测试 153 全绿(降档/升档/no_active_prepaid/webhook 升档)
- [x] 段3 切换支付：USDC→卡(subscribeConfirmRoute 新订阅带 subscription_data[trial_end]=USDC 到期日,试用满首扣)；卡→USDC(webhook 预付授权益前 cancelStripeSubscriptionAtPeriodEnd 卡订阅,upsert 从卡到期叠加+清 stripe_subscription_id 解耦;取消放付成功后不误退)。测试 155 全绿(USDC→卡 trial_end、卡→USDC 转 usdc_prepaid 叠加)
- [x] 段4 前端 + 取消识别：官网加支付方式(卡/USDC)切换、USDC 购买(季/年金额)、USDC 换档(预览补钱/补时长)；取消入口按支付方式出文案(卡=到期取消 cancel_kind=stripe / USDC=到期自然失效 cancel_kind=usdc_prepaid)。后端 cancelMembershipRoute 按 subscription_source 分派 + /prepaid 异档拒(prepaid_tier_change_required 引导换档)。App SquareMembershipState 加 currentPeriodStart+subscriptionSource,会员卡 _ActiveMembershipBanner 显示订阅起止+路线(预付/自动续)。测试:后端 vitest 160(+5:异档拒/同档续费/3 取消)、App flutter 10(+3:预付/卡横幅/getter)、官网 tsc+eslint(无测试框架,金额逻辑后端已覆盖)
- [x] 段4 对抗性审查(12 agent 工作流)4 项 CONFIRMED 全修:①webhook 结算 HIGH——异档并存(两档挑战在 confirm→webhook 窗口都过守卫)时旧便宜档时长被直贴贵档少收档差 → upsertPrepaidMembership 加**价值守恒折算**(异档按 剩余天×旧月费÷新月费 折成本档等值天数追加,同档续费仍日历月叠加);②官网 upgrade_pending 无 payment_url 误报成功 → 加显式报错分支;③App 冻结态与订阅横幅共存矛盾 → 横幅加 !frozen;④App 取消标签忽略 cancelAtPeriodEnd → 加"已取消·到期终止"。低级项(USDC 按钮走取消入口=用户明确设计/日期时区=起止同移非真差/cancel 顺序=旧码良性)按分析保留。测试补:后端 161(+折算兜底)、App 12(+取消标签/冻结隐藏)
- [x] 文档：CITIZENWEB 会员章节 + ADR-034 段4 + 任务卡；残留已清(无死码/未用导入,三端 tsc/analyze/eslint 净)

## 先决/待核
- 真实 USDC Checkout 是否落可用 payment_intent 凭证以入库绑定（沙箱定向验证）
