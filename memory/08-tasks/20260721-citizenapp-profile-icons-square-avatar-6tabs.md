# CitizenApp 三处 UI 调整：主页头部按钮底衬 + 广场头像圆形上移 + 广场 6 分类

任务需求：用户逐条确认的三处调整。
所属模块：citizenapp（8964 广场 + profile 用户主页）

## Q1 · 用户主页返回/三点按钮在背景图上看不见 → 每个图标加保底底衬

- 根因：返回箭头+三点均白（`user_profile_page.dart:443` SliverAppBar `foregroundColor:white`），
  唯一对比来自很弱且**顶弱底强**的压暗渐变（`collapsible_header.dart:78` `[0x22→0x38]`），
  按钮在最顶、默认 banner 是随机风景照（顶部常亮天空）→ 白图标洗白。
- 方案（用户选定）：给返回与三点各加一枚半透明深色圆形底衬（`Colors.black.withValues(alpha:0.32)`），
  背景无论明暗都可读。**不改压暗渐变**。
- 落点：`user_profile_page.dart` 返回 `IconButton` 加 `style: IconButton.styleFrom(backgroundColor, foregroundColor:white)`；
  `profile_kebab_menu.dart` 的 `icon` 由裸 `more_vert` 换成 `CircleAvatar(深色底 + 白 more_vert)`，`padding:zero`。
  ProfileKebabMenu 仅用户主页用，改它安全。

## Q2 · 广场头部头像方形 vs 发布按钮圆形 → 仅此头像改圆 + 整体上移

- 现状：头像 `LocalIdentityAvatar(size:46)` 内部 `BorderRadius.circular(10)`（方角）；发布 `IconButton.filled`（圆）。
  头部 padding `fromLTRB(16,14,16,10)`（`square_home_page.dart:263`）。
- 方案：`LocalIdentityAvatar` 加可选 `circular=false`（圆时 radius=size/2、扇贝徽章偏移 -4→0 贴圆边）；
  **只在广场头部传 `circular:true`**，「我的」tab / feed 仍方角圆角（app 身份视觉不变）。
  头部 padding 顶部 14→6，整块（头像+发布+分类）上移一点。

## Q3 · 广场分类 3→6：推荐、关注、竞选、文章、照片、视频

- `SquareFeedKind` 加 `article('文章','recommended')`、`photos('照片','recommended')`、`videos('视频','recommended')`
  （内容型三档 workerValue 复用推荐服务端流，**不动 Worker**；顺序即 tab 顺序）。
- `_filterPosts` 客户端过滤（与主页 5 页签同一 `contentFormat/mediaKind` 口径）：
  article=contentFormat==article；photos=normal且有图无视频；videos=normal且有视频。加 `_hasImage/_hasVideo`。
- `SquareFeedTabs`：等宽分段 6 段，**图标在上、文字在下**竖排（横排图标+文字在 320px 会溢出），
  `iconFor` 补 article=article_outlined/photos=photo_library_outlined/videos=videocam_outlined。
- 穷举 switch 同步：`_filterPosts`、`iconFor`（全仓仅这两处 switch SquareFeedKind）。

## 必须遵守 / 边界

- 不动 Worker、链端、feed 卡片渲染、发布闭环；不动「我的-背景图」本人视角入口。
- LocalIdentityAvatar 圆形只作用于广场头部；共享组件默认行为不变。
- 「把 X 改 Y」字面执行，不附带子任务。

## 输出物 / 验收

- 代码 + 中文注释；`flutter analyze` 0 问题。
- 测试：广场 6 分类可切换（新增 article/photos/videos tap→lastFeedKind）、分类过滤（seedPosts 校验 article/photos/videos）；
  主页 kebab/返回按钮测试仍通过；「我的」头像测试仍通过。
- 残留清理：无。

## 执行结果（2026-07-21）

- **Q1**：`user_profile_page.dart` 返回 `IconButton` 加 `IconButton.styleFrom(backgroundColor: Colors.black.withValues(alpha:0.32), foregroundColor: white)`；`profile_kebab_menu.dart` 三点 `icon` 换成 `CircleAvatar(半透明黑底 + 白 more_vert)` + `padding: zero`。两枚白图标现在无论 banner 明暗都有深色圆形底衬保证可读。
- **Q2**：`LocalIdentityAvatar` 加 `circular=false`（圆时 radius=size/2、徽章偏移 -4→0）；广场头部头像传 `circular:true`（仅此处圆形，「我的」/feed 仍圆角方形）；头部 padding 顶部 14→6，头像+发布+分类整块上移一点。
- **Q3**：`SquareFeedKind` 加 article/photos/videos（workerValue 均 `recommended`，不动 Worker）；`_filterPosts` 客户端过滤（article=文章；photos=普通图文有图无视频；videos=普通有视频）+ `_hasMedia`；`SquareFeedTabs` 6 段等宽改**图标在上文字在下**竖排（12px 文字，适配 320px），`iconFor` 补三档图标。两处 SquareFeedKind 穷举 switch（`_filterPosts`/`iconFor`）已同步。
- **测试**：`square_home_page_test.dart` test1 扩为 6 分类切换断言；新增「按内容分类过滤」测试（seedPosts 图/视频/文章，高视口避免懒加载，互不串档）。`flutter analyze lib/8964 + 测试` 0 问题；广场 4/4、profile + 我的 49 全过。
- **验证限制**：Flutter 移动端，未起真机；以 widget 测试 + analyze 为准。Q1 的底衬观感、Q2 圆形头像与徽章贴合、Q3 6 段竖排在窄屏的实际效果建议真机再核。
