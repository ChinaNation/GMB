# CitizenApp 广场 tab 顶部重设计（删标题/空态字 + 头像移左他人视角 + 坦克水印 + 分类精修）

任务需求：重设计公民App 广场 tab 顶部区与分类区，用户已逐轮确认设计稿。
所属模块：citizenapp / 8964（广场）

## 定稿设计（用户逐轮确认）

1. **删除**：左上「广场」大号字 + 下方「推荐/今日剩余 N 条」小号字整列；中间空态「图标 + 暂无XX动态」两行字。
2. **头像移左**：左上角放**默认用户真实头像**（本地 avatarPath，与「我的」tab 同源）+ **右下角扇贝身份徽章**（用户确认要徽章）。
   点击 → `UserProfilePage(ownerAccount: 默认用户, isSelf: false)` = **他人视角**看自己主页（区别于「我的-背景图」的 `isSelf:true` 本人可编辑视角）。右上角保留「发布」按钮。
3. **坦克水印**：`assets/icons/tank.svg` 放大居中，青色 primary(#007A74)，透明度约 **0.05**（用户要「再淡一点点」，比稿更淡）+ 轻微高斯模糊，`IgnorePointer` 常驻背景；有动态时卡片浮其上，无动态时只见水印。
4. **分类精修**：`推荐/关注/竞选` 由默认 `SegmentedButton` 换成青色圆角分段 pill（每段图标+文字：推荐=罗盘/关注=人群/竞选=喇叭；选中=白底青字，未选=灰字）。保持 `SquareFeedKind` 与 `onChanged` API 不变。

## 落点

- `lib/8964/pages/square_home_page.dart`
  - 头部 Row（L244-297）：删左标题列 + 删右侧身份徽章 IconButton；左放头像（可点→他人视角）、右留发布按钮。
  - feed 容器（L315-347）：`Expanded` 内包 `Stack`，底层 `IgnorePointer` 坦克水印居中，上层原 FutureBuilder+RefreshIndicator。
  - `_openSelfAsOthers()`：push `UserProfilePage(isSelf:false)`。
  - 新增 `_avatarPath` 状态，`initState` 经 `UserProfileService().getState()` 载入（与「我的」同源）。
  - **清理**：删 `_browseLeft` 字段 + L369 setState（删标题后变死码）——**副作用：不再显示「今日剩余 N 条」浏览配额**（用户要求删小标题，配额消耗逻辑在 Worker 端不受影响，仅隐藏计数）。
- `lib/8964/widgets/square_feed_tabs.dart`：`SegmentedButton` → 自定义 pill 分段控件；`SquareFeedKind` 加每档图标（或图标在组件内 switch）。
- `lib/8964/widgets/square_empty_state.dart`：**删除整个文件**（仅 square_home_page 引用，删空态后孤儿）。
- `lib/8964/pages/square_home_page.dart` `_FeedBody`：`posts.isEmpty` 分支不再返回 `SquareEmptyState`，改为可下拉刷新的空滚动区（错误横幅保留）；删 `_emptyIcon/_emptyTitle/_emptyMessage`。
- **新增共享组件** `lib/8964/profile/widgets/local_identity_avatar.dart`：把 `my/user/user.dart` 私有 `_SquareAvatar`（本地文件头像+扇贝徽章）抽成公有 `LocalIdentityAvatar`（加可选 badgeSize），广场头像与「我的」tab 共用，单一视觉身份不漂移。同步改 `user.dart` 复用它（行为不变）。

## 必须遵守

- 视角区分：头像点击固定 `isSelf:false`（他人视角）；不得动「我的-背景图」的 `isSelf:true` 入口。
- 头像与「我的」tab 同源（本地 avatarPath + ProfilePresentation 兜底），避免同一用户两套头像。
- 不动 feed 卡片、发布闭环、Worker、链端；分类过滤逻辑（following 空/campaign 过滤）不变。
- 8964→my 引用已有先例（profile_edit_page 等），沿用不新增边界破坏。

## 输出物

- 代码 + 中文注释；共享头像组件；删空态文件。
- widget 测试：头部无「广场/推荐」文案、头像存在且点击 push 他人视角 `isSelf:false`、分类三段可切换、空态无「暂无X动态」文字、水印存在。
- 残留清理：`_browseLeft`、`SquareEmptyState`、旧标题/空态零残留。

## 验收标准

- `flutter analyze lib/8964` 0 问题；广场与「我的」相关测试通过。
- 顶部无大小标题；左头像点击进他人视角主页；发布按钮保留。
- 分类为青色 pill 三段；空态只见坦克水印无文字；水印若隐若现。

## 执行结果（2026-07-21）

- **新增共享组件** `lib/8964/profile/widgets/local_identity_avatar.dart`：`LocalIdentityAvatar`（本地文件头像+扇贝身份徽章，加可选 `badgeSize`）。`my/user/user.dart` 私有 `_SquareAvatar` 删除并改用它（行为不变，「我的」头像 badgeSize 默认 24），同时移除 user.dart 里因此变空的 `ui/identity_badge.dart` import。
- **头部**（`square_home_page.dart` build）：删「广场」大字+「推荐/今日剩余 N 条」小字整列；删右侧身份徽章 IconButton。左上=`LocalIdentityAvatar(size:46, badgeSize:18)`，外包 `Tooltip(accountLabel)`+`GestureDetector`（account 空则不可点）；点击 `_openSelfProfileAsOthers` → `UserProfilePage(isSelf:false)` 他人视角。右上保留发布按钮，中间 `Spacer`。
- **头像数据**：新增 `_avatarPath` 状态，`initState` 经 `UserProfileService().getState()` 载入（与「我的」同源，设备级）。
- **坦克水印**：feed `Expanded` 内改 `Stack`，底层 `Positioned.fill>IgnorePointer>Center>Opacity(0.05)>ImageFiltered(blur 2.2)>SvgPicture.asset('assets/icons/tank.svg', primary青, key: square-tank-watermark)`，动态浮其上。
- **分类精修**（`square_feed_tabs.dart`）：`SegmentedButton` → 青色圆角分段 pill（`_SquareFeedTab`），每段图标（推荐罗盘/关注人群/竞选喇叭）+文字，选中白底青字带 1px 青边、未选透明边防抖动；API 不变。
- **空态清理**：`_FeedBody` `posts.isEmpty` 改为可下拉刷新的空滚动区（错误横幅抽 `_errorBanner` 复用），删 `_emptyIcon/_emptyTitle/_emptyMessage` 与 `feedKind` 字段；**删除 `lib/8964/widgets/square_empty_state.dart` 整个文件**。
- **死码清理**：删 `_browseLeft` 字段 + `_loadFeed` 里的 setState（**副作用：不再显示「今日剩余 N 条」浏览配额**，Worker 端消耗逻辑不受影响）。
- **测试**：`square_home_page_test.dart` 重写 test1（水印存在/头像存在/发布按钮存在/旧标题空态零残留/加 `_RecordingFeedSource` 断言三分类按 feedKind 切换）；test2/3 保持（发布 tooltip、身份 accountLabel tooltip 仍在）。`flutter analyze lib/8964 + user.dart` 0 问题；广场 3/3、「我的」lazy-chain、user_service、8964/profile 共 57 测试全过。
- **边界**：8964→my 引用沿用既有先例；未碰 feed 卡片、发布闭环、Worker、链端、`isSelf:true` 本人入口。
