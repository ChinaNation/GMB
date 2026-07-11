# 任务卡：会员模型精确匹配 + 访客双档（自由/民主）

- 建卡日期：2026-07-10
- 归属：CID/后端（worker+Stripe）+ Mobile（App 卡片）+ 官网（citizenweb）
- 状态：**已完成编码 + 测试 + 官网浏览器验证（2026-07-10）**；待部署（Stripe 建价 + 上线）
- 前置：承接 [[20260710-citizenapp-identity-membership-3cards]]（层叠三档卡已完成）
- 非 citizenchain runtime，无重新创世

## 1. 需求（用户定稿）

1. 会员改精确匹配：什么身份只能订本身份对应会员，**禁止降档**。
2. 访客身份加一档会员：$2.99（现有）+ $9.99（新，权益=投票公民）。两个 9.99 权益相同，差别=身份（民主匿名 / 投票公民认证）。
3. 发帖额度按会员套餐（不按身份）。
4. 非本档订阅按钮置灰。
5. Stripe 价格由用户自建。
6. 访客会员：公民 App 与官网都用**一张卡内左右两档切换**（左自由 / 右民主，默认自由，点击切换）。
7. 文案：卡片分区「链上身份信息」→「链上公开的身份信息」。

## 2. 模型（membership_level 与身份解耦）

| level | 显示名 | 价(cents) | required_identity | 权益 |
|---|---|---|---|---|
| `visitor` | 自由会员 | 299 | visitor | 标清/1分钟视频512MB/文20000字50图标清 |
| `visitor_pro`(新) | 民主会员 | 999 | visitor | =voting：HD/30分钟视频2GB/文30000字100图HD |
| `voting` | 投票公民会员 | 999 | voting | HD/30分钟2GB/文30000字100图HD |
| `candidate` | 竞选公民会员 | 9999 | candidate | HD/3小时10GB/文30000字100图HD |

## 3. Step1 — worker（citizenapp/cloudflare/src/membership + types + wrangler）

- `plans.ts`：`MembershipLevel` 加 `'visitor_pro'`；`membershipPlans` 加该套餐（quota 复制 voting，required_identity_level='visitor'，display_name='民主会员'，visitor 的 display 改 '自由会员'）；`membershipPlanList()` 返回 4（顺序 visitor, visitor_pro, voting, candidate）；`assertMembershipLevel`/`membershipPlan` 认新值。
- **精确匹配**：`assertCheckoutEligibility`（checkout.ts）删掉 `if(required==='visitor') return` 捷径，改为读链身份后 `plan.required_identity_level === identity.identity_level` 才放行（含 visitor 也要确认确无 voting/candidate 身份）。`eligibleMembershipLevels`（service.ts）由 `identitySatisfies(≥)` 改精确 `required===identity`。
- Stripe：`types.ts` 加 `STRIPE_PRICE_VISITOR_PRO?`；`priceIdForMembership`（checkout.ts）加 visitor_pro→该 env 分支；webhook `stripe.ts` price→level 反查加 visitor_pro 分支；`wrangler.toml` 三处 env 段加 `STRIPE_PRICE_VISITOR_PRO=""`。
- `resolveMembershipEntitlement` **不动**（按套餐；visitor_pro 走 visitor 分支免身份校验得 voting 额度）。
- 测试：`cloudflare/test/membership.test.ts` 补 visitor_pro 精确匹配 + 降档拒绝用例。

## 4. Step2 — App 卡片（citizenapp/lib/my/membership/membership_page.dart）

- `_orderedPlans` 改为按身份档聚合：访客档含 [visitor, visitor_pro] 两套餐，voting/candidate 各一。三张身份卡不变（访客/投票/竞选）。
- 访客卡内加**自由/民主分段切换**（默认自由=visitor，点击切民主=visitor_pro），切换价格/权益/订阅按钮。
- **置灰非本档订阅**：订阅按钮仅当 `plan.requiredIdentityLevel === state.identityLevel`（本人身份档）可点，否则 disabled 灰态。
- 分区标题文案改「链上公开的身份信息」。
- 徽章「降档染色」分支变死代码（无降档），本卡不依赖，暂不清理。
- 测试：补访客卡双档切换 + 非本档按钮置灰用例。

## 5. Step3 — 官网（citizenweb/src/pages/Membership.tsx）

- 访客卡同样自由/民主左右切换（默认自由，点击切换）。
- 订阅 challenge 传对应 membership_level（visitor / visitor_pro）。
- 精确匹配由 worker 强校验，官网 UI 跟随。

## 6. 待部署（用户侧）

- Stripe 后台新建 $9.99 民主 price，配 `STRIPE_PRICE_VISITOR_PRO`（wrangler secret/vars）。
- 重新生成官网机构/构建部署。

## 7. DoD

- [x] worker 4 档 + 精确匹配 + 新价格映射，`tsc` 干净、106 vitest 全过（含新增 visitor_pro 资格 / 降档拒绝用例）。
- [x] App 访客卡自由/民主切换、非本档置灰、文案「链上公开的身份信息」，`dart analyze` 干净、7 widget 测试全过。
- [x] 官网访客卡自由/民主切换，build/lint/tsc 干净 + 本地浏览器验证（点民主→$9.99+高清30分钟权益+面板同步）。

## 8. 实际落点（已改文件）

- worker：`plans.ts`(加 visitor_pro + IdentityLevel 解耦 + identityEligibleForPlan)、`checkout.ts`(精确匹配 + visitor_pro 价格)、`stripe.ts`(webhook 反查)、`service.ts`(eligibleMembershipLevels 精确)、`types.ts`(STRIPE_PRICE_VISITOR_PRO + membership_level 联合放宽)、`uploads/service.ts`(normalizeMembershipLevel 保 visitor_pro)、`social/author_signals.ts`(identity_level→IdentityLevel)、`wrangler.toml`(env)。测试 `membership.test.ts` / `membership_checkout.test.ts`。
- App：`lib/my/membership/membership_page.dart`(_plansByTier 聚合 + _PlanToggle + 置灰 _SubscribeButton + 文案) + `test/my/membership/membership_page_test.dart`。
- 官网：`citizenweb/src/pages/Membership.tsx`(4 档 plans + 身份档聚合渲染 + 访客自由/民主 toggle)。**2026-07-10 追加**：官网卡片整套复刻 App 身份卡设计(白卡+档色顶带「身份·X」在顶+扇贝徽章+「链上公开的身份信息」字段区+会员权益+档色价签)，两端统一;selectedLevel 改 nullable——选中卡自动切「订阅会员」tab、点「取消订阅」释放所有卡选中(取消无需选档);tsc/lint/build 干净+浏览器验证(卡片渲染/自由民主切换/取消deselect/选卡→订阅tab全通过)。

## 9. 待部署（用户侧，未做）

- Stripe 建 $9.99 民主 price → 配 `STRIPE_PRICE_VISITOR_PRO`（wrangler secret/vars，三环境）。
- worker deploy + 官网 build 部署。
