# 任务卡：CitizenApp「我的 → 身份 ｜ 会员」三档竖卡改版

- 建卡日期：2026-07-10
- 归属 Agent：Mobile Agent（纯 CitizenApp 客户端 UI）
- 状态：已完成编码 + 测试（2026-07-10）；待真机 e2e 与官网主域最终确认
- 关联：[[project_citizen_identity_strict_auth_vote_gates]]、[[project_public_institution_feature_2026_06_13]]、徽章设计 `citizenapp/lib/ui/identity_badge.dart`

## 1. 任务背景

用户要求改造 CitizenApp「我的」Tab 的会员入口与会员页：

- 入口「会员」→ 改名「身份 ｜ 会员」。
- 会员页现为 4 张竖排卡片（1 张 `_MembershipStatusCard` 状态卡 + 3 张 `_MembershipPlanCard` 套餐卡，纯只读）。
- 改为 3 张可横向滑动的竖形卡片，一卡 = 身份档 + 会员等级 + 等级参数 + 右上角徽章。
- 进入页面默认定位到「默认用户当前身份档」对应卡。

## 2. 目标状态

### 2.1 入口与标题

- `citizenapp/lib/my/user/user.dart` 内 `_buildEntryCard(title: '会员')` → `'身份 ｜ 会员'`。
- `MembershipPage` `AppBar` 标题 `'会员'` → `'身份 ｜ 会员'`。

### 2.2 三档竖卡（PageView 横向滑动）

- 三档固定顺序：`visitor` 访客轻节点 / `voting` 公民轻节点 · 投票 / `candidate` 公民轻节点 · 竞选。
- 每卡结构：档色顶带 + 右上角徽章 + 大字档名 + 身份/会员状态行 + 等级参数（动态/文章额度取现有 `SquareMembershipPlan` 字段）+ 底部价格 + 订阅按钮。
- 进入默认 `PageController(initialPage = 身份档 index)`：visitor=0 / voting=1 / candidate=2。身份档单源取默认钱包链上身份（`identityLevel`，本地徽章快照先出、链查询后补，初始 index 用首帧可得值，链查询回来后不强制跳页，避免闪跳）。

### 2.3 徽章（复用现成组件，不重画）

- 复用 `lib/ui/identity_badge.dart` 的 `CitizenBadge` + `identityBadgeStyle()`。
- 铁律：一个钱包只有一个身份 + 一个会员态。**只有「你的身份卡」按真实会员态显示勾/小人**，另两张恒显示小人（它们只是档位介绍，不代表本人）。
- 访客 = 所有无链上身份的用户；访客订阅了 → 勾，未订阅 → 小人。
- 「你的身份卡」徽章参数：`identityLevel = 该卡档`、`membershipLevel = 用户实际会员档`、`membershipActive = 用户会员生效态`（沿用现有 `state.active` 语义，与「我的」头像徽章一致；含高档身份买低档会员的 checkColor 染色由现成函数处理）。

### 2.4 订阅按钮三态（点击都拉起设备浏览器进官网）

| 会员态 | 按钮文案 |
|---|---|
| 未订阅 / 订阅未生效 | 订阅 |
| 已订阅 · 自动续费中（未取消） | 取消订阅 |
| 已取消 · 但付费期未到期 | 续订会员 |

- 判定数据源（worker `GET /v1/square/membership` 已返回，见 `cloudflare/src/membership/service.ts::membershipRoute`）：
  - `subscription_active`（bool）：是否已订阅且未过期。
  - `membership.cancel_at_period_end`（0/1）：是否已取消到期终止。
- 状态机：
  - `!subscription_active` → 订阅
  - `subscription_active && !cancel_at_period_end` → 取消订阅
  - `subscription_active && cancel_at_period_end` → 续订会员
- 点击行为：`url_launcher` 拉起系统浏览器打开官网会员页（URL 待用户确认，落单一常量）。App 侧不做任何 Stripe/取消/续订本地逻辑，全部在官网完成。

## 3. 影响范围与修改边界

- 主改：`citizenapp/lib/my/user/user.dart`（入口文案 + `MembershipPage` 及子卡片重写为 pager）。视文件体量决定是否拆出 `citizenapp/lib/my/membership/` 子目录（`MembershipPage` 现内联在 user.dart，约 500-940 行，建议拆分降耦）。
- 数据模型：`citizenapp/lib/8964/services/square_api_client.dart` 的 `SquareMembershipState` 补两字段：`subscriptionActive`（读 `subscription_active`）、`cancelAtPeriodEnd`（读 `membership.cancel_at_period_end`）。
- 依赖：`citizenapp/pubspec.yaml` 新增 `url_launcher`（当前无外部浏览器打开能力）。
- 复用不改：`lib/ui/identity_badge.dart`。
- 测试：`citizenapp/test/` 补 `MembershipPage` widget 测试（默认档定位、3 卡渲染、按钮三态文案）；`test/ui/identity_badge_test.dart` 不受影响。
- **不碰**：`citizenchain/runtime`、扫码签名/验签、worker（数据已够）。无双端联动风险。

## 4. 待确认 / 风险

- [ ] 官网会员页确切 URL（仓库无记录，官网由 `64.181.239.233` Nginx 提供，主域未登记）——唯一硬阻塞，须用户提供。
- [ ] 是否给官网 URL 附带档位 hint（如 `?level=voting`）——默认只开 `/membership`，官网当前未读该参数。
- 徽章勾采用 `state.active`（entitlement）而非纯 `subscription_active`：正常同档场景两者等价；仅「订阅有效但链上身份被撤销」极端场景不同，采用与头像徽章一致口径。

## 5. 完成定义（DoD）

- [x] 入口 + 标题改名生效（`user.dart` 入口 `身份 ｜ 会员`，页面 AppBar 同名）。
- [x] 3 竖卡 pager，进入默认停在默认用户身份档卡（`PageController.initialPage = _tierIndex(identityLevel)`）。
- [x] 徽章复用现成 `CitizenBadge`/`identityBadgeStyle`，只有本人身份卡反映真实勾/小人。
- [x] 订阅按钮三态文案正确（订阅/取消订阅/续订会员），点击均 `launchUrl` 拉起系统浏览器进官网。
- [x] `dart analyze` 无新增告警；新增 6 条 widget 测试全通过（`test/my/membership/membership_page_test.dart`）。

## 6. 实现落点（已改文件）

- `citizenapp/lib/my/membership/membership_page.dart`（新建，pager + 三卡 + 三态按钮 + 官网 URL 配置）。
- `citizenapp/lib/my/user/user.dart`（入口文案改名；删除旧内联 `MembershipPage` 及子类；改导入）。
- `citizenapp/lib/8964/services/square_api_client.dart`（`SquareMembershipState` 补 `subscriptionActive`/`cancelAtPeriodEnd` 解析）。
- `citizenapp/pubspec.yaml`（新增 `url_launcher: ^6.3.1`）。
- `citizenapp/android/app/src/main/AndroidManifest.xml`（补 `<queries>` https VIEW，Android 11+ 打开浏览器）。
- `citizenapp/test/my/membership/membership_page_test.dart`（新建 widget 测试）。

## 7. 遗留 / 待用户确认

- 官网会员页主域：默认 `https://crcfrcn.com/membership`（`String.fromEnvironment('CITIZENAPP_MEMBERSHIP_SITE_URL')` 可覆盖）。仓库未登记官网主域，若非裸 `crcfrcn.com` 需用户给准域名或部署时 `--dart-define` 覆盖。
- 真机 e2e：iOS/Android 点击按钮实际跳官网、pager 滑动与默认定位需真机验收（widget 测试已覆盖逻辑）。
