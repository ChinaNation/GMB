# 任务卡：CitizenApp「我的-创作者」页（App 侧实现）

> 状态：**App 侧完成并验证**（2026-07-17）。创作者管理页 + **他人主页「订阅 TA / 取消」订阅侧**均已实现；`flutter analyze` 无问题；`flutter test test/my/creator test/rpc/subscription_rpc_test.dart` **13 用例全绿**（含 subscribe/cancel 字节逐字节对齐链端金标向量）。BFF（Cloudflare）创作者端点为依赖契约，另立卡实现；本卡按契约对接 + `FakeCreatorApi` 本地跑。
>
> **订阅侧（步骤2）已实现**：`rpc/subscription_rpc.dart`（subscribe/cancel(Creator) 上链，pallet=34/call 1、2，字节对齐金标）+ `8964/subscribe/creator_subscribe_service.dart`（订阅/取消**都热签+生物识别**，三段式 catch，best-effort confirm）+ `8964/profile/widgets/creator_subscribe_button.dart`（双态：有档才显示，未订阅→选档周期订阅/已订阅→取消）+ `profile_header_card.dart` 加 `creatorSubscribeButton` 槽 + `user_profile_page.dart` 挂入（isSelf 隐藏）。订阅签=授权按月扣款、取消签=撤销；按月续扣 keeper 拉取不逐月签。

## 需求（用户定稿）
- 「我的」页在**会员与通讯录之间**加「创作者」入口（`Icons.storefront_outlined` + `AppTheme.info`）。
- 创作者管理自己的会员：**档名 + 最多 10 档 + 每档月/季/年价（可只开部分周期）**，档定义全在**本机设置 + Cloudflare 存**（链上不存）。
- **门禁**：非平台会员先引导去开通平台会员（**链上校验**，复用现有平台会员态）。
- **编辑会员档保留生物识别**（离链但属核心操作）：用户定——更安全。
- 前端展示/输入**一律元**，模型/API/链**一律分**，只在 UI 边界换算。

## as-built（已落地，已验证）
**文件（`citizenapp/lib/my/creator/`）**
```
models/creator_plan.dart       BillingPeriod{monthly,quarterly,yearly} · CreatorTier(pricesFen:分) · CreatorPlan(≤10) + JSON
models/creator_overview.dart   CreatorOverview(订阅人数/预计MRR分/档位数)
creator_money.dart             元/分唯一换算边界(fenToYuanLabel / yuanTextToFen)
creator_api.dart               CreatorApi 接口 + CreatorApiHttp(自包含 http,复用会话鉴权与统一签名) + FakeCreatorApi(内存)
creator_service.dart           编排:门禁(fetchMembership)+读档位/概览+saveTiers(生物识别·三段式catch)
creator_page.dart              三态:加载/门禁/已开通(概览卡+档位列表+新增x/10+谁订阅了我入口)
creator_plan_edit_sheet.dart   编辑/新增底部弹窗(档名+月季年元输入+保存(生物识别)+删除)
widgets/creator_overview_card.dart · creator_tier_card.dart · creator_gate_view.dart
```
**改动**：`my/user/user.dart`（加「创作者」入口 + `_openCreator`，不回读 `_loadState`）。

## 关键决策（务必延续）
- **签名零新协议**：保存档位复用**现有广场账户动作统一签名 `OP_SIGN_SQUARE_ACTION`(0x1D)**——`signingMessage(0x1D)` → 主钥 `signWithWallet`（读硬件金库弹一次生物识别）→ challenge/sign/confirm。**不新增 op_tag / 域 / 协议**（纠正过我一版自造 `GMB-creator-plan`）。
- **生物识别机制**：App 已移除操作层 local_auth（`wallet_manager.dart:124`），生物识别只在 `signWithWallet` 读 seed 时触发；故"编辑保留生物识别"＝保存必签一次名（正好落在 0x1D 动作路）。
- **门禁数据源**：复用 `SquareApiClient.fetchMembership(session).active`（平台会员=链上态镜像），非新查询。
- **元/分边界**：`creator_money.dart` 唯一换算点；模型/api 全 `int 分`。
- **创作者档全链下**：链上无 `CreatorPlans`（见 [[project_subscription_chain_asbuilt_2026_07_17]]）；≤10 档由 App/Cloudflare 校验。

## 依赖：BFF 契约（Cloudflare 卡实现）
```
GET  /v1/square/creator/plan            → {plan|null}
GET  /v1/square/creator/overview        → {overview:{subscriber_count,month_income_fen,tier_count}}  # month_income_fen=本月真实扣款到账合计(分),非摊算/预计
POST /v1/square/creator/plan/challenge  {owner_account,tiers[]} → {signing_payload_hex,challenge_id}
POST /v1/square/creator/plan            {owner_account,challenge_id,signature,tiers[]} → {plan}
```
- `set_creator_plan` 挂进现有 `account/action_challenge` 框架（`buildActionScalePayload` 加该动作，context 绑 tiers 的 blake2 哈希防替换；验签后覆盖写 D1）。
- `tiers[]` 元素 = `{tier_id,name,prices_fen:{monthly?,quarterly?,yearly?}}`（分）。

## 验收（已达成）
- [x] `flutter analyze` 创作者代码 + user.dart 无问题；`dart format` 已过。
- [x] `flutter test test/my/creator` 10 绿（元/分换算、模型 JSON 过滤非法价、FakeApi 保存必调签名）。
- [x] 非平台会员 → 门禁态；已开通 → 概览+档位+新增(x/10)+谁订阅了我；编辑弹窗校验(空名/无周期/价≤0 拦截)。

## 下一步
- BFF Cloudflare 创作者端点（上方契约）+ `set_creator_plan` action_challenge。
- `8964` 他人主页「订阅 TA」按钮（订阅者侧，上链 `subscribe(Creator(x),CreatorPrice)`）。
- 「谁订阅了我」订阅者明细页（现为占位 SnackBar）。
- 创作者/订阅编排的 service/widget 层测试（需 WalletManager/SquareApiClient DI seam）。

影响范围：`citizenapp/lib/my/creator`（新增）+ `my/user/user.dart`（入口）。BFF/链端不在本卡。
