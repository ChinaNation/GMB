# ADR-033 会员订阅生命周期：单订阅 + 换档 proration + 身份绑定冻结

## 标题

会员订阅生命周期统一：一钱包一订阅、换档在同一订阅上按比例结算、链上身份绑定与权益冻结。

## 背景

现状（截至 2026-07-13）订阅链路只会「新建 Stripe Checkout」，存在缺口：

- 订阅路径不查已有订阅（`subscribe.ts` 只做身份资格校验），同一钱包在官网再次订阅/换档会创建**第二个 Stripe 订阅**，两个都扣费；webhook 按 `owner_account` 单行覆盖，旧订阅变孤儿、用户无法在 App 停掉 → **潜在双重扣费**。
- 无换档能力：升档/降档只能重新下单，等于新订阅。
- 会员权益与链上身份「≥ 满足」即可用（`resolveMembershipEntitlement` 只对非访客档、且非精确匹配），身份变化后旧权益不会被真正拦停。

## 决策

五条硬规则：

1. **一钱包一订阅**：任意时刻同一钱包最多一个活跃 Stripe 订阅；换档改**同一订阅对象**（subscription_id 不变），绝不新建第二个。
2. **重新订阅/换档**作用在未过期订阅上，续用其计费周期。
3. **换档按比例结算**：升档=当期剩余天数 ×（高−低）费率差；降档=不退现金，剩余价值进 **Stripe 信用余额**抵后续账单。
4. 换档（升/降）**都需公民 App 钱包签名授权**（动钱动权，op_tag 0x1D 复用 subscribe_membership 动作，会员等级绑进 payload）。
5. **身份绑定 + 冻结**：链上身份=权益唯一真源；档位 ≠ 身份（升/降任一方向、精确匹配）→ 权益立即**冻结**（视同无会员、不可用）+ Stripe **暂停收款**；**懒判定**（用权益时回链校验、带短缓存）；用户签名换到匹配档才解冻（恢复权益 + 恢复收款 + 按规则 3 结算）。

技术落点：

- 换档走 Stripe `POST /subscriptions/{id}`：改 `items[0][price]`，升档 `proration_behavior=always_invoice` + `payment_behavior=pending_if_incomplete`（**差价账单付成功才应用变更**——卡 off-session 自动扣、USDC/无卡返回 `hosted_invoice_url` 引导付款）；降档/同价 `proration_behavior=create_prorations`（信用余额，不收款）。
- resume（撤销待取消）= `cancel_at_period_end=false`。
- 权益落库仍以 **subscription webhook 为唯一真源**（换档触发 `customer.subscription.updated`，subscription_id 稳定）。
- USDC/加密钱包无法 off-session 静默扣款（链上转账须本人签发）→ 其续费与升档补差价一律走一次主动付款。

分期：**第 1 期**=规则 1–4（单订阅 + 换档 + 防重订 + 官网发起时展示补/转金额）；**第 2 期**=规则 5（冻结 + 暂停收款 + 身份精确匹配拦截）。第 2 期依赖第 1 期的换档能力。

## 影响

- 影响产品：Cloudflare Worker（订阅路径重构、Stripe 换档/resume 助手、webhook 兼容）、官网 citizenweb（发起时取当前订阅态、展示升/降/续订与预览金额）、CitizenApp（会员卡态展示、冻结提示）。
- D1：`square_memberships` 第 2 期加冻结/暂停列；开发期零用户，直接改基线 `migrations/0001_square_core.sql` 重建，不建迁移链。
- 契约不破：op_tag 0x1D / QR_V1 / owner_account 单行存储不变。
- 精确匹配「禁降档越级」规则升级为**双向冻结**：档位必须恰等身份档，升级（会员低于身份）同样冻结待换档。

## 备选方案

- 降档退现金：否。规则 3 明确不退资金，转信用余额（Stripe 原生 credit balance）。
- 换档新建订阅+取消旧的：否。违反规则 1（会短暂存在两订阅、且丢失计费周期）。
- 身份升级不冻结、仅提示：否（已评估）。取严格精确匹配，升降都冻结，规则统一。
- 冻结用链上事件常驻监听：否。无常驻监听服务，取懒判定 + 短缓存，用时必拦。

## 后续动作

- 任务卡：`memory/08-tasks/open/20260713-membership-lifecycle-single-subscription-proration.md`（分第 1/2 期）。
- 第 1 期：Worker 订阅分派器 + Stripe 换档助手 + 防重订 + 测试（本 ADR 落地时先做）。
- 第 2 期：冻结状态 + `pause_collection` + 精确匹配拦截 + D1 列。
- 完成后同步：官网 Membership.tsx、App 会员卡、`memory/05-modules` 会员模块文档。
