# CitizenApp 广场：删顶部头像入口 + 发布改右下角 FAB + 分类栏上移

任务需求：用户确认的广场顶部精简。
所属模块：citizenapp / 8964 广场

## 定稿（用户确认）

1. 删广场左上角头像及其「点击进他人视角看自己」功能。进自己主页只保留「我的-背景图」。
2. 身份真源 = 我的-我的钱包-钱包列表的「默认用户标签」；广场顶部不再显示身份。
3. feed 里点任意作者头像（含自己发的帖）仍进用户主页（`_openAuthor` 不改，isSelf 自动判定）；
   自看场景由上一轮「防御性置灰治理」兜底（保留，不回退）。
4. 发布按钮 → 右下角圆形悬浮 `FloatingActionButton`（primary 色），`endFloat`=我的 tab 上方。
5. 分类栏（6 段）整体上移到顶部，省下 header 约 60px。

## 落点（`square_home_page.dart`）

- build：外包 `Scaffold(body: SafeArea>Column[Padding>SquareFeedTabs, Expanded>Stack(水印+feed)],
  floatingActionButton: 发布 FAB, floatingActionButtonLocation: endFloat)`。删整个顶部 header `Row`。
  顶部 padding 收到 `fromLTRB(16,8,16,8)` 仅给 tabs。
- FAB：`FloatingActionButton(onPressed:_openCompose, tooltip:'发布动态', backgroundColor:AppTheme.primary,
  foregroundColor:白, child: Icon(edit_rounded))`。
- feed 底部 padding 20→~88，避免 FAB 盖住末条互动区。
- **删死码**：`_openSelfProfileAsOthers`、`_loadAvatarPath`、`_avatarPath` 字段、initState `unawaited(_loadAvatarPath())`；
  `_membership` 字段（仅原头像徽章读）+ `_refreshMembership` 里的 setState 写 + `_loadIdentity` 里的
  `unawaited(_refreshMembership())` 预取（无展示对象）；import `local_identity_avatar.dart`、`my/user/user_service.dart`。
- **保留**：`_identityFuture` 与 `_onWalletsChanged/_onChainHealthChanged` 重载（`_openAuthor` 的 isSelf 判定仍需跟随默认用户）；
  `_refreshMembership` 方法本身（`_openCompose` 发布门禁仍调，用返回值）。

## 必须遵守 / 边界

- 不碰 UserProfilePage 的自看防御治理（保留）、私信 self 守卫（保留）、关注/通知/订阅内部实现（独立会话）。
- 不动主壳 main.dart（FAB 走广场内层 Scaffold，主壳无 FAB、body 可容纳，已核实）。

## 测试影响（`square_home_page_test.dart`）

- test1：`find.byType(LocalIdentityAvatar)` → `findsNothing`；加 `find.byType(FloatingActionButton) findsOneWidget`；`find.byTooltip('发布动态')` 保留（FAB 带此 tooltip）。
- test2：点 `find.byTooltip('发布动态')`（现为 FAB）→ 会员门禁弹窗，不变。
- test3「walletsRevision→身份重载」原靠头像账户 tooltip 断言，头像删后无可视身份元素 → **删该用例**及仅其用的
  `_SwitchableWalletManager`/`_hotWallet` 辅助（重载逻辑保留在代码中服务 `_openAuthor`，非可视特性）。

## 验收

- `flutter analyze` 0 问题（无未用字段/ import 残留）。
- 广场页无头像、发布为右下角圆形 primary FAB、分类栏在顶部；点作者头像仍进主页。
- 广场相关 widget 测试通过。

## 执行结果（2026-07-21）

- **删头像入口 + 分类上移**：`square_home_page.dart` build 删整个顶部 `Row`（头像 FutureBuilder + 发布 IconButton）；`SquareFeedTabs` 提到顶部，`Padding` 顶部 `fromLTRB(16,8,16,8)`。
- **发布改 FAB**：build 外包 `Scaffold(floatingActionButton: FloatingActionButton(onPressed:_openCompose, tooltip:'发布动态', backgroundColor:AppTheme.primary, foregroundColor:白, child:Icon(edit_rounded)), floatingActionButtonLocation: endFloat, body: SafeArea>Column[...])`。feed `ListView.separated` 底 padding 20→88 给 FAB 让位。
- **删死码**：`_openSelfProfileAsOthers`、`_loadAvatarPath`、`_avatarPath`、`_membership` 字段、initState 头像预取、`_loadIdentity` 里会员预取、`_refreshMembership` 的 setState 写；import `local_identity_avatar.dart`、`my/user/user_service.dart`。保留 `_identityFuture`+`_onWalletsChanged`（服务 `_openAuthor` isSelf）、`_refreshMembership` 方法（发布门禁用）。
- **保留（防御）**：UserProfilePage 自看治理、私信 self 守卫，均不回退。
- **测试**：`square_home_page_test.dart` test1 头像断言→`findsNothing` + 加 `FloatingActionButton findsOneWidget`（发布 tooltip 保留）；删 test3（靠头像 tooltip）及仅其用的 `_SwitchableWalletManager`/`_hotWallet`。`flutter analyze lib/8964 + 测试` 0 问题；广场 3/3 通过。
- **验证限制**：Flutter 移动端未起真机；FAB 相对底部导航「我的」tab 的实际位置、分类栏上移观感建议真机再核。
