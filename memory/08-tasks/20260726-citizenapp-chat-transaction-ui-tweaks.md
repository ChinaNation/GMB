# 任务卡：CitizenApp 聊天页与交易页 UI 微调（加号弹窗 / 搜索页 / 箭头统一 / 顶部两栏对齐）

状态：实现完成（2026-07-26）。六处改动全部落地；`flutter analyze` 六文件 No issues；
受影响四测试文件（chat_search_page / chat_tab / chain_progress_banner / transaction_tab_page）
共 34 用例全绿；`Icons.arrow_back` 真实用法归零、旧底色 0xE83D4A52 与旧 `_width=126` 已清除。
待模拟器实机视觉复核（弹窗三角对齐 / 两栏竖线对齐）。

改动文件：
- lib/ui/app_theme.dart（actionIconTheme 全局 chevron_left）
- lib/8964/profile/user_profile_page.dart:505（显式 arrow_back→chevron_left）
- lib/chat/chat_tab.dart（加号 28/16、弹窗底色 0xFF66727D 实心、行距 6、上下留白 4、宽 116、右内边距 22、三角避圆角 clamp、圆圈左移 pad-right 24、弹窗偏移 24）
- lib/chat/chat_search_page.dart（搜索框下移导航栏下方、AppBar=返回+搜索+清除文字键）
- lib/ui/widgets/chain_progress_banner.dart（左段入 Expanded、竖线两侧等宽间距→居中）
- lib/transaction/transaction_tab_page.dart（扫一扫 18→22、多签 28→24）

所属模块：Mobile（citizenapp，纯前端 UI，无链上 / CID / 协议改动）

## 任务需求

### 一、聊天页（lib/chat）

1. 右上角圆圈加号与加号图标一起等比缩小：圆圈 40→28，描边 1.5→1.25，加号图标 24→16。
2. 加号点击弹出的入口菜单（扫一扫 / 收付款 / 发私信 / 发群聊 / 加好友）：
   - 五项之间间距缩小：每项上下内边距 vertical 12→6，面板上下留白 SizedBox 6→4。
   - 底色减淡但保持 100% 不透明（严禁透出页面）：`0xE83D4A52`（深板岩 91%）→ `0xFF66727D`（浅板岩实心不透）。三角 caret 同色跟随。
   - 面板不加宽、按内容自适应；左内边距 16、右内边距 22（右侧比左侧宽一点点）。
   - 凸出三角落在面板平边上、不压圆角（消除三角与圆角之间的空隙）；三角尖顶正对圆圈加号水平中心。
3. 搜索页（chat_search_page.dart）：
   - 搜索输入框从 AppBar 标题移到导航栏下方独立一行，套 `surfaceMuted` 圆角框 + 聚焦 `primary` 1.5px 描边 + autofocus。
   - 导航栏 = 返回键（左向 chevron）+「搜索」标题 +「清除」文字键（有输入才显示，`primary` 色）。
   - 结果三段（会话 / 联系人 / 聊天记录）逻辑与样式不变。
4. 全局返回箭头统一为左向线性 chevron（`Icons.chevron_left`），与卡片右向 `Icons.chevron_right` 成对；禁止带横杆的箭头（`Icons.arrow_back` 及系统默认返回键）。
   - 单点实现：`AppTheme.lightTheme` 新增 `actionIconTheme: ActionIconThemeData(backButtonIconBuilder: … chevron_left)`，覆盖全仓 61 个 AppBar 的默认返回键。
   - 换掉唯一 1 处显式 `Icons.arrow_back`（lib/8964/profile/user_profile_page.dart:505）。

### 二、交易页（lib/transaction）

> 2026-07-28：本节第 5 项的内容区链状态卡片及其竖线已经被
> `20260728-citizenapp-transaction-citizen-topbar.md` 的顶栏行内状态取代；
> 下述内容只记录当时验收结果，不再是当前 UI 目标。

5. 顶部链状态栏（ui/widgets/chain_progress_banner.dart 的 compactThreeState）与扫一扫/多签账户栏（transaction_tab_page.dart）中间的竖线对齐：
   - 扫一扫/多签栏竖线本在卡片正中，链状态栏竖线偏右约 8px（呼吸圆点 + 间距摆在左 Expanded 之外把分隔线顶右了）。
   - 把呼吸圆点 + 间距移进左 Expanded、分隔线两侧留等宽间距，使链状态栏竖线也落在卡片几何正中。两卡同宽同插边（同一 ListView，padding all(16)），居中后即上下同一垂直线。
   - 竖线维持原样短线居中、上下留空、不顶满卡片（链状态栏 height 24、扫一扫/多签栏 height 52，均不改）。
6. 多签账户左侧图标缩小：`Icons.share_outlined` size 28→24。
7. 扫一扫左侧图标加大：`assets/icons/scan-line.svg` 18×18→22×22（继续用 scan-line.svg，遵守扫码图标死规则）。

## 必须遵守 / 不变边界

- 不突破 Mobile 模块边界：本任务只改 citizenapp 前端 UI，绝不动 citizenchain / CID / 协议 / 后端。
- 扫码图标只用 `assets/icons/scan-line.svg`，禁 Material 二维码图标（既有死规则）。
- 展开折叠 / 返回指示器禁实心三角，只用线性 chevron（既有死规则）。
- 弹窗底色只调颜色深浅、绝不调透明度（必须 100% 不透明）。
- 不附加未要求的子任务（不改竖线高度 / 粗细 / 其他页面样式）。
- 无残桩、无兼容分支，干净更新。

## 输出物

- 代码（citizenapp Flutter）
- 中文注释
- 组件/交互测试（搜索页导航栏结构、加号菜单尺寸）
- 残留清理（旧 `_width=126`、旧底色常量、AppBar 内搜索框结构）

## 验收标准

- 加号缩小、弹窗实心不透、三角对齐圆心、间距收紧、搜索页导航栏与输入框布局符合确认稿。
- 全仓返回键均为 chevron_left，无 `arrow_back` 残留（grep 归零）。
- 交易页两条竖线上下同一垂直线；两图标尺寸符合规格。
- `flutter analyze` 无新增告警；相关 widget 测试通过。
- 模拟器实机视觉核对通过。
