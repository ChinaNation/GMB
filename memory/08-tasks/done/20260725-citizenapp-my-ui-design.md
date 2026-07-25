# CitizenApp“我的”页面 UI 设计

## 任务需求

- 为移动端“公民”`CitizenApp` 重新设计“我的”页面。
- 第一轮只生成一张 `390 × 844` 高保真 UI 效果图供用户确认。
- 暂不修改 Flutter 代码，不改变现有业务逻辑、数据契约或模块边界。

## 所属模块

- `citizenapp`
- 页面边界：`citizenapp/lib/my/user/`

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/05-modules/citizenapp/user/USER_TECHNICAL.md`
- `memory/05-modules/citizenapp/8964/PROFILE_TECHNICAL.md`
- `memory/07-ai/module-checklists/citizenapp.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`

## 设计边界

- 产品中文名使用“公民”，英文名使用 `CitizenApp`。
- 延续现有浅色主题、翠绿色品牌色、照片型资料头部、圆角方形头像与身份徽章。
- 保留现有入口：钱包、会员｜订阅、创作者、通讯录、电子护照、设置。
- 保留底部导航：广场、公民、聊天、交易、我的。
- 不展示真实 `account_id`、CID、余额或其他隐私数据。
- 不新增业务入口，不改变身份、会员、钱包或电子护照的业务含义。
- 效果图只输出 App 内容，不包含手机外框、系统状态栏、浏览器栏或系统底部指示器。

## 设计目标

- 首屏优先表达当前用户身份和个人资料入口。
- 对钱包、身份与个人服务进行清晰分组，避免六个同质卡片连续堆叠。
- 保证背景图明暗变化时，标题、昵称和身份徽章仍有足够对比度。
- 在 `390 × 844` 视口内完整容纳主要入口和底部导航，不裁切内容。
- 用户昵称上边缘与头像上边缘齐平，资料文字区不相对头像垂直居中。
- 钱包、电子护照两张主入口卡保持紧凑等高。
- 头像右下角只使用统一扇贝身份勋章：访客金、投票蓝、竞选红；有效会员显示白色对勾，否则显示白色小人。
- 底部“广场”只使用现有 `assets/icons/tank.svg` 坦克 V2 语义，不得替换为建筑、社区、指南针或其它通用图标。

## 预计修改目录

- `memory/08-tasks/open/`：新增本任务卡，记录设计目标、边界和用户反馈；仅涉及文档。
- `citizenapp/`：本轮仅只读现有主题、页面结构和素材，不修改代码、不新增文件、不产生残留。

## 输出物

- 一张会话内展示的 CitizenApp“我的”页面高保真 UI 效果图。
- 用户对视觉方向的反馈与后续是否进入实现的决定。

## 验收标准

- 输出尺寸与移动端页面比例匹配。
- 中文文案清楚，无明显乱码、裁切或重叠。
- 现有功能入口和底部导航完整。
- 视觉语言与 CitizenApp 现有主题一致。
- 未修改 CitizenApp 代码、链端、后端、协议或数据结构。

## 当前状态

- 需求分析：已完成。
- 新建任务卡：已获用户继续信号并完成。
- UI 效果图：已生成并在当前会话展示。
- 设计稿采用班夫湖默认背景、海鸥默认头像、翠绿色品牌体系，以及“身份头部 → 钱包/电子护照主入口 → 个人服务分组 → 设置 → 五栏导航”的层级。
- 用户第一轮反馈已完成视觉修订：昵称与头像顶部对齐、两张主入口卡降低高度、徽章恢复蓝色 8 花瓣扇贝加白勾、广场恢复炮管朝左的坦克图标。
- 用户第二轮反馈已完成视觉修订：钱包、电子护照和个人服务内容组整体轻微上移。
- 第二轮生成式预览曾错误重绘底部 Tab 图标，该稿已撤回，不得作为设计基线；修正版已
  直接从 `citizenapp/lib/main.dart`、`citizenapp/assets/icons/tank.svg`、
  `citizenapp/assets/icons/scale.svg` 和 Flutter `MaterialIcons` 提取五个现有图标，
  未使用生成式或近似图标。
- 用户已于 2026-07-25 明确确认修正版设计稿，当前视觉方案正式定稿；设计图保留为
  会话交付物，未写入仓库资产目录。
- CitizenApp UI 设计标准：已确认现有全局文档真源为
  `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`，已在该文件 §4.1.1
  基础上补齐并更新；未新建第二份标准文件。
- AI 必读入口与 CitizenApp 模块执行清单已同步，今后 CitizenApp UI 设计、实现与评审
  必须读取并遵守 §4.1.1。
- Flutter 实现：未授权，不在本轮范围。
- 任务状态：已完成并归档。
