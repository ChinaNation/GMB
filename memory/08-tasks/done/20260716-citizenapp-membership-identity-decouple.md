# 任务卡：CitizenApp 会员与身份彻底解耦（四档→三档 + 聊天文件上限按档）

- 建卡日期：2026-07-16
- 归属 Agent：Mobile Agent（App 会员卡）+ CID/后端（Cloudflare Worker + Stripe）+ 官网（citizenweb）
- 状态：**已完成编码 + 测试（2026-07-16）**；待用户侧 Stripe/官网部署配置
- 决策来源：用户 2026-07-15 定稿 + 2026-07-16 边界拍板
- 关联 ADR：ADR-036（本任务新建，取代 ADR-033 规则5 身份绑定冻结）
- 关联记忆：[[project_membership_identity_decoupling_2026_07_15]]、[[project_chat_media_tiered_relay_2026_07_15]]、[[project_membership_visitor_two_tier_exact_match]]、[[project_citizenapp_four_gate_entry_failclosed]]、[[project_citizenapp_passport_default_user]]
- 双轨：与卡2私密小群（`20260715-citizenapp-chat-group-private-e2e`）互不干扰；R2 瞬时中转 transport 归卡2 阶段3，本卡不实现。

## 1. 任务需求（用户定稿）

1. 把「会员=付费订阅」与「身份=电子护照（访客/投票/竞选）」**彻底解耦**成两个独立轴，任意身份可选任意会员档（全组合）。
2. 订阅三档月费：`freedom` 自由会员 $2.99、`democracy` 民主会员 $9.99、`spark` 薪火会员 $99.99。
3. 会员权益之一 = 聊天文件大小上限：无订阅/自由 ≤10MB、民主 ≤100MB、薪火 ≤5GB。>100MB（仅薪火，100MB–5GB）走 Cloudflare 瞬时群密钥密文中转——**本卡不实现该中转，归卡2 阶段3**；本卡只定「薪火可发 ≤5GB」的档位与单源限额值。

## 2. 目标模型（两轴解耦）

| 轴 | 取值 | 归属 |
|---|---|---|
| 会员 `MembershipLevel` | `freedom` / `democracy` / `spark` | membership 模块，**去掉 required_identity** |
| 身份 `IdentityLevel` | `visitor` / `voting` / `candidate` | myid 电子护照，不变 |

- `spark` 为新内部键（`candidate` 是身份名，解耦后不再作会员键）。
- 任意身份 × 任意档，全组合可订。

## 3. 四档→三档映射（零用户，直接重建，无迁移/兼容/补偿）

| 旧档 | 新档 | 处置 |
|---|---|---|
| `freedom` $2.99 | `freedom` $2.99 | 保留，去身份绑定 |
| `democracy` $9.99 | `democracy` $9.99 | 保留，去身份绑定 |
| `voting` $9.99 | — | 删除（与 democracy 权益重复，仅身份不同，解耦后无存在理由） |
| `candidate` $99.99 | `spark` $99.99 | 改名 + 解耦；复用现有 $99.99 Stripe price 值改挂 `SPARK_PRICE_ID` |

## 4. 权益重挂（一律按套餐，不按身份）

- 发帖/文章额度 `limits/catalog.ts:usageLimits` 与 `plans.ts` quota：四档→三档（删 voting，candidate→spark）。
- 媒体质量档 `imageResource/videoResource`：`candidate`→`spark` 分支改名。
- **新增聊天文件上限权益**：在 `MembershipPlan` quota 加单源字段 `chat_file_max_bytes`（freedom 10MB、democracy 100MB、spark 5GB）；App `chat/chat_media_limits.dart` 由固定值改为**按用户当前会员档读值**；**无会员行 = 按 freedom 10MB**。客户端强制（媒体走 WebRTC P2P，服务端不在字节路径）。

## 5. 解耦影响面（一次性净替换，零残留）

### Worker（citizenapp/cloudflare/src）
- `membership/plans.ts`：删 `required_identity_level` / `RequiredIdentityLevel` / `identityEligibleForPlan`；`MembershipLevel` 改三档；删 `voting`、`candidate`→`spark`；加 `chat_file_max_bytes`；`assertMembershipLevel` / `membershipPlan` / `membershipPlanList` 改三档。
- `membership/subscribe.ts`：`assertCheckoutEligibility` 删身份门（改无条件放行或删函数）；`priceIdForMembership` 改 `SPARK_PRICE_ID`、删 voting 分支；`applyMembershipChange` 删「同价民主↔投票」分支。
- `membership/service.ts`：`eligibleMembershipLevels` 恒返回三档；**删身份冻结子系统** `resolveMembershipEntitlement` 冻结分支 / `syncCollectionState` / `identityLevelForFreeze` / `normalizeIdentityLevel`；`requireActiveMembership` 只判订阅有效；upsert/prepaid 落库删 identity 字段。
- `membership/webhook.ts`：删 `identity_required` 状态与 `required_identity_level==='visitor'` 分支、`VOTING_PRICE_ID` 反查、`fetchChainIdentityState` 会员侧调用；加 `SPARK_PRICE_ID` 反查。
- `membership/prepaid.ts`：删身份分支。
- `types.ts`：删 `VOTING_PRICE_ID`；`CANDIDATE_PRICE_ID`→`SPARK_PRICE_ID`；Env/Row 删身份/冻结字段。
- `wrangler.toml`：三处 env 段 price 变量改三档（删 VOTING、CANDIDATE→SPARK）。
- `limits/catalog.ts`：`usageLimits` 三档；`videoResource`/`imageResource` 的 candidate→spark；`square_video_candidate` 资源键改名 `square_video_spark`。
- `migrations/0001_square_core.sql`：`square_memberships` 基线重建，删 `identity_level` / `identity_checked_at` / `frozen_at` / `collection_paused` 列。
- `test/`：会员/订阅/webhook/limits 测试改三档、删身份匹配/冻结用例、加解耦全组合用例。

### App（citizenapp/lib）
- `my/membership/membership_page.dart`：**三身份卡 → 三档订阅卡**；删按身份置灰、访客自由/民主切换、卡内「链上公开身份字段」展示（身份字段移入 myid 电子护照）；入口与页面标题「身份 ｜ 会员」→「会员」（`my/user/user.dart` 入口文案）。
- `8964/services/square_api_client.dart`：`SquareMembershipState`/`SquareMembershipPlan` 删 `requiredIdentityLevel` / 冻结相关字段；`_fallbackMembershipPlans` 改三档。
- `ui/identity_badge.dart`：清解耦后「降档染色」死码。
- `chat/chat_media_limits.dart`：改按会员档读 `chat_file_max_bytes`。
- `8964/profile` 展示档改名（voting 档展示删、candidate→spark）。
- `test/my/membership/`、`test/chat/` 相应改。

### 官网（citizenweb/src/pages/Membership.tsx）
- 四档身份聚合 → 三档并列订阅卡；订阅 challenge 传三档 level；删身份聚合/访客双档切换。

### 电子护照（承接目标2 身份字段落点）
- 会员页移出的「链上公开身份字段」并入 `lib/my/myid/`（myid_page 已是护照三卡真源，确认字段完整，缺则补）。

## 6. Stripe / proration

- 价格环境变量三档 `FREEDOM_PRICE_ID` / `DEMOCRACY_PRICE_ID` / `SPARK_PRICE_ID`；删 `VOTING_PRICE_ID`。$99.99 复用现有 candidate price 值。
- 单订阅升/降档 proration 逻辑保留，仅档集变三档。
- **身份绑定冻结暂停收款整体删除**（ADR-033 规则5 被 ADR-036 取代）。
- Stripe 后台：删 voting price 使用、$99.99 price 改挂 `SPARK_PRICE_ID`（用户侧配置，price ID 非密钥可入库）。

## 7. 门禁校验改动

- 门禁2 发布闸 `posts/confirm→requireActiveMembership`：只判订阅有效，删身份精确匹配/冻结。
- 门禁3 用量：`usageLimits` 三档。
- 聊天：新增媒体按档限额（客户端强制）；>100MB 薪火中转的服务端档位闸 = 卡2 阶段3。

## 8. 必须遵守

- 不修改 `citizenchain/`（链端）。
- 只在主检出 `/Users/rhett/GMB` 操作，不碰 worktree。
- 开发期零用户：直接重建 D1 基线，不留旧档/旧列/兼容/迁移/补偿入口。
- 权益真源仍是 subscription webhook（`upsertStripeMembership`）与 `expires_at`；op_tag 0x1D / QR_V1 / owner_account 单行存储契约不变。
- 一钱包一订阅、升档 pending、降档进信用余额不退现金 —— 保留。
- 关键安全逻辑补中文注释；未获单独授权不推 GitHub / 不触发远端 CI。

## 9. 输出物

- 三档解耦会员模型 + 单源聊天文件上限权益 + 客户端按档限额。
- Worker / App / 官网全链路解耦代码 + 中文注释。
- Worker vitest（三档全组合订阅 + 删身份匹配/冻结）、App analyze/test、官网 tsc/eslint/build。
- ADR-036 + 会员模块文档更新 + 死码/残留清理。

## 10. 验收标准

- 任意身份可订任意三档（visitor 可订 spark、candidate 可订 freedom），无 `membership_identity_mismatch`。
- 仓库内无 `voting` 会员档、`required_identity_level`、`identityEligibleForPlan`、冻结/暂停收款、`frozen_at`/`collection_paused`/`identity_level` 残留。
- 聊天文件上限按档：无会员/freedom 10MB、democracy 100MB、spark 5GB，单源 `chat_file_max_bytes`，收发端一致。
- 会员页为纯三档订阅卡，无身份字段；身份字段在电子护照可见；入口/标题为「会员」。
- Stripe 三价映射 + proration 换档；`tsc`+vitest 全绿、App analyze/test 通过、官网 build 通过。
- 本地真实 Worker/D1 验收三档订阅 + 换档 proration + 发布闸。
- ADR-036 + 文档已更新、残留已清理。

## 11. 待用户侧配置

- Stripe：$99.99 price 改挂 `SPARK_PRICE_ID`；停用 voting $9.99 price 的会员映射（price ID 已在 wrangler.toml staging/production 段就位）。
- 官网重新构建部署（三档卡）。

## 12. 完成记录（2026-07-16）

- **Worker**：`plans.ts` 三档 + `chat_file_max_bytes`（删 required_identity/identityEligibleForPlan/voting，candidate→spark）；`service.ts` 删身份冻结子系统（resolveMembershipEntitlement 冻结/syncCollectionState/identityLevelForFreeze/eligibleMembershipLevels）+ upsert 去身份列；`subscribe.ts` 删 assertCheckoutEligibility 身份门 + priceId→SPARK + 删同价 switch 死分支；`webhook.ts` 删 identity_required + visitorIdentity + VOTING 反查、加 SPARK；`prepaid.ts` 去身份；`stripe_api.ts` 删 pause/resumeStripeCollection；`types.ts` Env 三档 price + MembershipRow 删 identity/frozen 列；`limits/catalog.ts` usageLimits 三档 + `square_video_spark`；`feeds/browse.ts`/`uploads/{quota,service,validation}.ts`/`limits/usage.ts`/`membership/archive.ts`/`social/author_signals.ts`/`chain/identity.ts`（IdentityLevel 本地化）随改；`wrangler.toml` 三处 env 段；`migrations/0001_square_core.sql` 基线删 4 列。**tsc 干净 + 164 vitest 全过**（membership/membership_subscribe/limits/uploads_quota/profiles 等测试改三档 + 删冻结/身份匹配用例）。
- **App**：`square_api_client.dart` `SquareMembershipState/Plan` 去身份/冻结 + 加 `chatFileMaxBytes` + fetchMembership 简化解析 + 接 `ChatMediaLimits.applyMembershipLevel`；`membership_page.dart` 三身份卡→三档订阅卡（删身份字段/置灰/切换/冻结横幅，入口/标题→「会员」）；`chat_media_limits.dart` 按档动态（`maxBytesForLevel` + fail-closed 自由档）；`square_upload_service.dart` 竞选闸门→spark；`identity_badge.dart` 注释更新；`user.dart` 入口文案。**flutter analyze 干净**（仅卡2 group 测 2 条既有 info）**+ membership/chat/ui/8964 全测通过**。
- **官网**：`Membership.tsx` 四档身份聚合→三档并列订阅卡（删身份卡/徽章/字段/切换/冻结）。**tsc/eslint/build 全过**。
- **文档**：CITIZENAPP_TECHNICAL / CITIZENWEB_TECHNICAL / CHAT_TECHNICAL / unified-protocols / unified-naming 更新三档解耦口径；ADR-036 新建；ADR-033 规则5 标注被取代。全库无 `voting`/`candidate` 会员档、`required_identity_level`、冻结、`VOTING/CANDIDATE_PRICE_ID`、`square_video_candidate` 代码残留。
## 13. 2026-07-16 用户三点追加（已落地）

1. **Stripe 后台设计（LIVE 账户 `acct_1Trr2fHSzSYWD2rF` 经 MCP 完成）**：产品 `prod_UraY9wtqb2XjcY`「竞选公民会员」→ 改名「薪火会员」（$99.99，`SPARK_PRICE_ID=price_1TrrNCHSzSYWD2rFrZTsmKhl`）；产品 `prod_UraY5SJQSS3i0e`「投票公民会员」归档（active=false）+ 其价 `price_1TrrN8…`（voting $9.99）停用（active=false, nickname `voting_deprecated`）；三档现价 nickname/metadata 统一为 freedom/democracy/spark。最终 LIVE 仅 3 个活跃订阅价：自由 $2.99 / 民主 $9.99 / 薪火 $99.99，与 wrangler.production 对齐。**注**：staging/sandbox 是另一账户 `acct_1Trr2qQlQZ1x0Cw8`，本 MCP 未连，其测试产品名保持旧值（如需同步须单独授权该账户）。
2. **权限二分（撤回原「顶档会员闸门」，改治本）**：**发帖分类权限按身份档**——竞选内容只有竞选身份（`identity_level==='candidate'`）可发。worker 删 `assertMembershipCanPublishCategory`（会员闸门），新增 `assertIdentityCanPublishCategory`（身份闸门），在 `uploads/service.ts:prepareUpload` 与 `posts/confirm.ts` 仅竞选帖读链身份校验；App 删 `square_upload_service` 会员闸门 + `isSparkMembership` getter，`SquarePublishService`/`compose` 由 `isCertified`(voting+) 收紧为 `isCandidate`(candidate)，`compose_type` `certified`→`canCampaign`。**用量额度仍按会员档**。任意身份可订任意档。测试改：`uploads_quota.test`（identity 闸门用例）、`compose_type_test`、`membership_page_test`（删 isSparkMembership 用例）。**worker 164 vitest + App 149 test + web build 全绿**。
3. **星火 → 薪火**：显示名全库统一（英文键 `spark` 不变；worker plans/App/web/docs/memory 全改）。**页面职责**：会员介绍 + 订阅/取消 = 「我的-会员」（membership_page）；身份介绍 = 「我的-电子护照」（myid_page，三身份卡已具备，无需改）。
