# 20260710 公民 App 底部导航栏图标更新

## 任务目标

调整 CitizenApp 底部导航栏（`citizenapp/lib/main.dart` 的 `NavigationBar`）：

1. `我的 / 交易 / 公民` 三个 tab 完全不变。
2. `信息` 改名为 `聊天`，图标换成"气泡 + 三点"（消息语义）。
3. `广场` 图标从指南针（`Icons.explore`）换成"坦克"线性图标，作为坦克人（8964 主题）的表达；本轮先落地坦克本体，人物版另议。

## 已确认事实

1. 现有导航顺序：`广场 / 公民 / 聊天(原信息) / 交易 / 我的`，见 `main.dart:439` 起的 `destinations`。
2. 自定义图标唯一先例是 `交易` tab：`SvgPicture.asset('assets/icons/scale.svg')` + `ColorFilter(BlendMode.srcIn)`，未选中 `AppTheme.textTertiary`、选中 `AppTheme.primary`，尺寸 22。设计语言 = Lucide 描边（`fill=none`、圆角 caps/joins）。
3. `assets/` 按单文件登记（非整目录），新增 SVG 必须在 `pubspec.yaml` assets 列表显式登记。
4. Flutter SDK 确认存在 `Icons.textsms_outlined` / `Icons.textsms_rounded`（气泡 + 三点，rounded 家族，与 bar 现有 `_rounded` 一致）。
5. 资产命名沿用"按图形本体"约定（`scale.svg` / `wallet.svg` / `bank.svg`），坦克图标命名 `tank.svg`。

## 设计定稿（用户逐轮确认）

- 聊天图标：方案 1 · 气泡三点。
- 广场图标：坦克 V2（极简，短炮管 + 内缩车体，履带两端露出），描边 **stroke 1.5**（比天平的 stroke 2 更细，用户明确选定）。
- 人物版（坦克人本体 P 系列）本轮不做，留待后续卡。

## 分步实施

### 第 1 步：新增 tank.svg 资产 ✅
- [x] 新建 `citizenapp/assets/icons/tank.svg`（viewBox 0 0 24 24，`fill=none`，stroke 1.5，圆角，V2 几何）。
- [x] `pubspec.yaml` assets 列表在 `scale.svg` 后登记 `assets/icons/tank.svg`。

### 第 2 步：改 main.dart 导航项 ✅
- [x] `广场`：`Icon(Icons.explore*)` → `SvgPicture.asset('assets/icons/tank.svg')`，两态 ColorFilter（textTertiary / primary），尺寸 22，仿 `scale.svg` 写法（`main.dart:440` 起）。
- [x] `信息` → `聊天`：label 改字；icon `Icons.chat_bubble_outline_rounded`→`Icons.textsms_outlined`，selectedIcon `Icons.chat_bubble_rounded`→`Icons.textsms_rounded`。

### 第 3 步：聊天页头部一致性（清残桩）✅
- [x] `lib/chat/chat_tab.dart:465` 大标题 `信息` → `聊天`（tab 改名后目的页头必须同步，否则半改）。
- [x] `test/chat/chat_tab_test.dart`：标题断言 `信息`→`聊天`；两个受影响用例描述同步改名。

### 第 4 步：验证 ✅
- [x] `dart format` 通过；`flutter analyze lib/main.dart lib/chat/chat_tab.dart` — No issues found。
- [x] `flutter test test/chat/chat_tab_test.dart --concurrency=1` — 10/10 通过（Isar 社区分支需 concurrency=1）。
- [x] `广场` label 未改（仅换图标），`test/widget_test.dart` 的 `find.text('广场')` 断言不受影响；`Icons.explore_outlined` 在 `square_home_page.dart:435` 是页内分类图标、非导航栏，未动。
- [ ] （建议）真机目视：坦克描边 1.5 清晰、聊天气泡三点正确、选中态变主题绿、三个不变 tab 无回归。

### 第 5 步：尺寸微调（用户反馈）✅
- [x] 聊天 `textsms` 图标略大 → `Icon(..., size: 22)`（默认 24 下调）。
- [x] 广场 `tank.svg` 略小 → SvgPicture `width/height: 22 → 26`（两态都改）。根因：自定义 SVG 上方留白多，同尺寸下视觉偏小。
- [x] `flutter analyze lib/main.dart` — No issues found。
- 备选（未做）：如仍偏小，可收紧 `tank.svg` viewBox 去掉坦克上方留白，让图形更满更居中，而非继续加大渲染尺寸。

## 完成状态

代码与静态/单测验证已完成，等真机目视确认后本卡可归档。剩余 `test/chat/chat_tab_test.dart` 内其余以「聊天 Tab …」命名的用例仅为测试描述（不断言标题文案），未改以免越界；如需全量统一可另行处理。

## 备注

- 坦克炮管一律朝左；如后续要"人 + 坦克对峙"或"坦克人本体"，另开卡不在本卡范围。
- 广场若日后统一改用比 2 更细的描边，`交易` 的 `scale.svg`（本卡不动）会显得略粗；是否统一需用户单独授权。
