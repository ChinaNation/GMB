# ADR-036 会员与身份彻底解耦：三档订阅、任意身份任意档、聊天文件上限按档

## 标题

把「会员（付费订阅）」与「身份（电子护照：访客/投票/竞选）」解耦成两个独立轴；会员收敛为三档订阅，任意身份可订任意档；聊天文件大小上限成为按档会员权益。取代 ADR-033 规则5（身份绑定 + 冻结）。

## 背景

现状（截至 2026-07-15，ADR-033 / `project_membership_visitor_two_tier_exact_match` 上线 production）会员四档 `freedom/democracy/voting/candidate` 与身份档强绑：

- `plans.ts` 每档带 `required_identity_level`，`identityEligibleForPlan` 精确匹配禁降档；
- `subscribe.ts:assertCheckoutEligibility` 下单前读链身份，不符 403；
- `service.ts:resolveMembershipEntitlement` + `syncCollectionState` 在身份≠会员档时冻结权益并暂停 Stripe 收款；
- `webhook.ts` 有 `identity_required` 状态与 `VOTING_PRICE_ID` 反查。

`democracy` 与 `voting` 权益完全相同，唯一差别是身份（民主匿名 / 投票认证）——两档并存只为区分身份。聊天文件上限现状（`chat_media_limits.dart`）为固定值、不分档、设备直连零中转。

用户 2026-07-15 决策：会员与身份是两件事，应自由组合；订阅收敛三档并把文件上限做成会员权益。

## 决策

1. **两轴解耦**：`MembershipLevel = freedom | democracy | spark`（去掉 `required_identity_level`）；`IdentityLevel = visitor | voting | candidate` 归电子护照。任意身份 × 任意档全组合可订。
2. **三档月费**：`freedom` $2.99、`democracy` $9.99、`spark`（薪火）$99.99。`spark` 为新内部键（`candidate` 是身份名，解耦后不再作会员键）。
3. **四档→三档映射**（零用户直接重建，无迁移）：freedom/democracy 保留去绑；`voting` 删除（与 democracy 权益重复）；`candidate` $99.99 改名 `spark` 并复用其 Stripe price 值改挂 `SPARK_PRICE_ID`。
4. **权益按套餐不按身份**：发帖/文章/媒体质量额度随 `membership_level`；**新增聊天文件上限权益** `chat_file_max_bytes`（freedom 10MB、democracy 100MB、spark 5GB），单源置于 `MembershipPlan` quota，无会员行按 freedom 10MB。客户端强制（媒体走 WebRTC P2P，服务端不在字节路径）。
5. **取代 ADR-033 规则5**：删除身份绑定 + 冻结 + 暂停收款整套机制（`resolveMembershipEntitlement` 冻结分支、`syncCollectionState`、`identityLevelForFreeze`、`identity_required` 状态、D1 `frozen_at`/`collection_paused`/`identity_level` 列）。ADR-033 规则1–4（一钱包一订阅、换档 proration、动钱动权签名）保留。

## 边界

- **本决策不实现** >100MB（仅薪火）R2 瞬时群密钥密文中转 transport——归卡2 私密小群 `20260715-citizenapp-chat-group-private-e2e` 阶段3（见 `project_chat_media_tiered_relay_2026_07_15`）。本决策只定「薪火可发 ≤5GB」的档位与限额值。
- **会员页纯订阅**：`membership_page` 由三身份卡改三档订阅卡，移除卡内「链上公开身份字段」，身份字段并入电子护照 `myid`；入口/标题「身份 ｜ 会员」→「会员」。身份与会员完全解耦。
- 不改 `citizenchain`（链端）。会员侧不再读链身份。

## 影响

- Worker：`membership/{plans,subscribe,service,webhook,prepaid}.ts`、`types.ts`、`wrangler.toml`、`limits/catalog.ts`、`migrations/0001_square_core.sql`（列删）、`test/`。
- App：`my/membership/membership_page.dart`、`my/user/user.dart`、`8964/services/square_api_client.dart`、`ui/identity_badge.dart`、`chat/chat_media_limits.dart`、`8964/profile`、`my/myid/`（承接身份字段）、`test/`。
- 官网：`citizenweb/src/pages/Membership.tsx`。
- Stripe：三价（删 voting、$99.99 改挂 SPARK）。

## 落点任务卡

`memory/08-tasks/open/20260716-citizenapp-membership-identity-decouple.md`
