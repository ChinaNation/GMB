# CitizenApp 四个主 Tab UI 设计

## 任务需求

- 按已定稿的 CitizenApp“我的”页面设计语言，继续设计交易、聊天、公民、广场四个主
  Tab 页面。
- 每个页面输出一张 `390 × 844` 高保真移动端 UI 效果图。
- 本轮只交付设计稿，不修改 Flutter 代码、业务逻辑、数据契约或模块边界。

## 所属模块

- `citizenapp`
- 广场：`citizenapp/lib/8964/`
- 公民：`citizenapp/lib/citizen/`
- 聊天：`citizenapp/lib/chat/`
- 交易：`citizenapp/lib/transaction/`

## 输入与标准

- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` §4.1.1
- `memory/07-ai/module-checklists/citizenapp.md`
- 已确认的 CitizenApp“我的”页面设计稿
- 四个目标页面的当前 Flutter 代码、主题和实际图标资源

## 设计边界

- 统一使用 `AppTheme` 的浅色背景、翠绿色品牌色、紧凑卡片、分组列表和字体层级。
- 不新增当前代码中不存在的入口、字段、状态、数据或导航。
- 底部导航顺序固定为“广场 / 公民 / 聊天 / 交易 / 我的”。
- 底部导航必须直接使用仓库 SVG 和 Flutter `MaterialIcons` 原字形，不允许生成式重绘、
  临摹、风格化或替换。
- 广场首稿展示推荐内容流；公民首稿展示提案子 Tab；聊天首稿展示会话列表；交易首稿
  展示链上支付主流程。
- 设计稿不包含设备外框、系统状态栏或浏览器栏。

## 页面内容

### 广场

- 推荐、关注、竞选、文章、照片、视频六个现有分类。
- 作者、正文、媒体和互动信息组成的内容流。
- 右下角现有发布按钮。

### 公民

- 提案、立法、选举、治理、公权五个二级 Tab。
- 提案编号、机构、提案摘要、状态和待投票提示组成的提案列表。

### 聊天

- 页面标题、右上角加号、搜索入口和会话列表。
- 加号只对应现有扫一扫、收付款、发私信、发群聊、加好友五个动作，本张主页面稿不展开
  弹层。

### 交易

- 顶部通讯录、标题和交易钱包入口。
- 链同步状态、扫一扫、多签账户、链上支付表单和交易状态入口。

## 预计修改目录

- `memory/08-tasks/open/`：新增并更新本任务卡；仅涉及文档，会被 Git 跟踪。
- `citizenapp/`：只读页面代码、主题和图标，不修改代码、资产或配置。
- 仓库外会话生成目录：保存四张设计效果图，不被 Git 跟踪。

## 验收标准

- 四张页面均为同一套 CitizenApp 视觉语言。
- 页面内容与当前代码和技术文档一致。
- 五个底部 Tab 图标、顺序、标签、颜色和选中态符合 UI 标准。
- 中文文案清楚，无乱码、裁切、重叠或不真实的业务数据。
- 未修改 CitizenApp 代码和图标资源。

## 当前状态

- 用户已明确允许创建本任务卡。
- 页面代码与 UI 标准只读检查：已完成。
- 四张 UI 效果图：已完成并通过同画布视觉一致性检查，用户已确认。
- 交易稿：仓库外会话文件 `citizenapp-transaction-ui-final.png`。
- 聊天稿：仓库外会话文件 `citizenapp-chat-ui-final.png`。
- 公民稿：仓库外会话文件 `citizenapp-citizen-ui-final.png`。
- 广场稿：仓库外会话文件 `citizenapp-square-ui-final.png`。
- 四张稿的页面主体以已确认“我的”稿为参考生成；底部导航随后单独覆盖，直接使用
  `tank.svg`、`scale.svg` 和 Flutter `MaterialIcons-Regular.otf` 原字形。生成式预览
  不参与底部五个 Tab 图标绘制。
- 已核对四张稿当前 Tab 选中态分别为交易、聊天、公民、广场，其余四项保持未选中态。
- `citizenapp/lib/main.dart`、`citizenapp/assets/icons/tank.svg` 和
  `citizenapp/assets/icons/scale.svg` 未修改。
- 后续 Flutter 实现已由独立任务卡
  `20260725-citizenapp-approved-ui-flutter-implementation.md` 承接；用户明确要求本次只
  更新“我的 / 交易 / 聊天 / 公民”，广场设计稿保留但广场代码暂不更新。
