# 会员订阅生命周期：单订阅 + 换档 proration + 身份绑定冻结

任务需求：
按 ADR-033 实现会员订阅生命周期五规则。分两期：第 1 期单订阅 + 换档结算 + 防重订；第 2 期身份绑定冻结 + 暂停收款。

所属模块：membership（Cloudflare Worker 主导）+ 官网 citizenweb + CitizenApp（配合）

输入文档：
- memory/04-decisions/ADR-033-membership-subscription-lifecycle.md
- citizenapp/cloudflare/src/membership/*（subscribe/service/plans/stripe_api/webhook）
- citizenapp/cloudflare/src/account/action_challenge.ts（签名挑战）
- citizenweb/src/pages/Membership.tsx（官网订阅页）
- citizenapp/lib/my/membership/membership_page.dart（App 会员卡）

必须遵守：
- 不可突破模块边界（权益真源=subscription webhook，唯一写入 upsertStripeMembership）
- 不可绕过既有契约（op_tag 0x1D / QR_V1 / owner_account 单行存储不变）
- 一钱包一订阅：有活跃订阅时任何「新建」路径改走 update-or-reject，绝不新建第二个
- 换档改同一订阅对象（subscription_id 稳定），升档 pending（付成功才生效），降档进信用余额不退现金
- USDC/加密钱包不能 off-session 静默扣款，升档补差价走一次主动付款
- 不清楚 Stripe 行为先查官方文档/API 参考，不猜

输出物：
- 代码（Worker 换档分派器 + Stripe 助手 + 官网/App 态展示）
- 中文注释
- 测试（Worker vitest 覆盖 新订阅/同档/resume/升档(卡·待付)/降档 各分支）
- 文档更新（memory/05-modules 会员模块）
- 残留清理

验收标准：
- 功能可运行（tsc + vitest 全绿；官网 tsc/eslint；App analyze/test）
- 有活跃订阅时不再产生第二个 Stripe 订阅
- 升/降档在同一订阅上按 proration 结算，升档付成功才生效
- 文档已更新、残留已清理、Review 问题已处理

## 进度

- [x] 第 1 期后端：Stripe 换档/resume 助手（stripe_api.ts）
- [x] 第 1 期后端：subscribeConfirmRoute 改分派器（新订阅/同档/resume/升/降）+ 防重订
- [x] 第 1 期后端：subscribeChallengeRoute 返回当前订阅态
- [x] 第 1 期后端：vitest 分支测试（131 全绿，含 5 换档用例）
- [x] 第 1 期官网：Membership.tsx submitSignature 处理换档 action（checkout_url/payment_url/即时生效文案）——功能端到端可用
- [x] 第 1 期 App：无需改动（App 只 launchUrl 打开官网，不直连 confirm）
- [x] 第 1 期官网增强：挑战响应带本地估算 preview（升补/降转金额），签名弹窗展示
- [x] 第 2 期后端：resolveMembershipEntitlement 精确匹配双向 + frozen；webhook 对齐精确匹配
- [x] 第 2 期后端：pause/resume collection 助手 + 懒判定双向同步（冻结暂停 / 匹配恢复，原子占位）
- [x] 第 2 期后端：身份读取安全回退（回链失败退上次已知身份，绝不误冻）
- [x] 第 2 期后端：D1 加 frozen_at + collection_paused（基线重建）
- [x] 第 2 期后端：vitest（升/降双向冻结 + 读失败不误冻；Worker 133 全绿）
- [x] 第 2 期 App：会员卡冻结横幅（SquareMembershipState.frozen）
- [x] 文档：ADR-033 + CITIZENWEB_TECHNICAL 3.2 + 死码清理（identitySatisfies/identityLevelRank）
- [x] Stripe 测试模式冒烟(冻结→暂停→换档→恢复)通过:暂停不改 status、换档 proration $7 与预览一致、换档不自动恢复、pause_collection= 清空恢复
- [x] test clock 冒烟:暂停期间 status 仍 active、current_period_end 照常前移一个周期、续费账单 void(不扣费)
- [x] P0 修复:webhook current_period_end 改 item 层优先+顶层兜底(新版 API 顶层为 null 会致订阅事件 400→用户付款拿不到权益);priceValue 复用 firstItem;+测试(item 层周期事件)
- [x] P1 修复:changeStripeSubscriptionTier 升档遇"无可扣款方式"400→抛 membership_upgrade_needs_payment 可操作提示(非裸 502);+测试
- [ ] USDC 订阅独立路线(用户思路:按季/年、不自动续、按时长授权、换挡重算补钱/补时长)——待专项分析设计
- [ ] 待覆盖(非核心):无卡但"有 PM 收不到款"的升档 open 账单→payment_url 分支(需 3DS 卡冒烟)
