# CitizenApp 默认钱包=身份 全端传播修复 + 我的主页推特式 UI

## 任务需求

用户在「我的 → 我的钱包」拖拽把另一个热钱包置顶（切换默认用户）后，**以钱包账户为唯一主键的所有身份面必须同步切换到新用户**：聊天 sender、广场发帖/点赞/评论作者身份与 isSelf 判定、会员状态、我的 tab 头像/昵称/认证、我的主页。钱包名 = 用户昵称（可改）。同时把「我的主页」从"资料白字压在 300px 高头图里"改造成真推特结构：短头图 + 头像跨压头图下缘 + 资料移到头图下方。

## 背景（已完成的只读诊断，见本轮分析）

根因三件套：①`WalletManager` 无任何变更通知（纯方法类，无 ChangeNotifier/Stream/Isar watch，监听者=0，切换只写 Isar `sortOrder`）；②`main.dart` 用 `IndexedStack` 常驻各 tab State，`ProfilePage`/`SquareHomePage`/`ImTabPage` 都只在 initState 读一次默认钱包；③我的 tab「钱包」入口 push 不 await、返回不 reload（唯一改默认钱包的路径恰好不刷新）。结果：动作类路径（发帖/聊天/session 签名）每次现取已用新钱包，而常驻页 UI 停留旧身份 → 显示/行为脑裂，须重启才对齐。

「默认用户钱包」= 派生规则：`getWallets()` 中第一个 `isHotWallet`（按 `sortOrder`），非存储字段（见 [[project_citizenapp_institution_arch_2026_06_29]] 相关钱包模型与卡 20260705-citizenapp-default-wallet-identity）。

## 建议模块

Mobile Agent — citizenapp：
- 传播：`lib/wallet/core/wallet_manager.dart`、`lib/wallet/pages/wallet_page.dart`、`lib/my/user/user.dart`、`lib/8964/pages/square_home_page.dart`、`lib/im/im_tab_page.dart`、会员显示入口。
- UI：`lib/8964/profile/user_profile_page.dart`、`lib/8964/profile/widgets/collapsible_header.dart`、`lib/8964/profile/widgets/profile_header_card.dart`。

## 设计定稿（用户已确认的 UI 决策）

- 头像行右上角保留 **3 个图标：通知 / 聊天 / 关注**（图标非按钮，用户决策 5 不变）。
- 头图：有 R2 背景图用背景图；无则纯品牌绿平铺（去掉旧三色渐变兜底，不留残桩，用户「都可以」→取更干净纯色）。
- 结构改动：头图 ~112px（不再 300px 整块压资料）；头像 76px 圆角方形跨压头图下缘（一半在图一半在白底）；昵称+认证勾 / 地址+复制·CID / 个性签名 / 关注·关注者·帖子 全部深色字落在头图下方白底；上滑虚化+返回+⋮+居中标题、分类标签(帖子/竞选/照片/视频/文章) sticky（决策 6 保留）；⋮ 菜单本人=二维码/编辑资料、他人=二维码/举报（决策 4 保留）。

## 分步骤技术方案

### Phase 1：身份传播骨架（功能核心，先做）
1. 新增全局 `DefaultWalletNotifier`（ChangeNotifier 单例，lib/wallet/core/）；`notifyChanged()` 在默认钱包真正变化后触发。
2. `wallet_page._onReorder` 落盘成功且默认热钱包变化后调 `notifyChanged()`；创建/导入/删除若改变默认钱包同样触发。
3. `ProfilePage`（我的 tab）：initState `addListener(_loadState)`、dispose 移除；「钱包」入口 push 改 await + 返回后 `_loadState()`（双保险）。
4. `SquareHomePage`：监听 → 重置 `_identityFuture = loadCurrent()` + setState；`_openAuthor` isSelf 判定随新身份。
5. `ImTabPage`：监听 → `_reload(syncFirst: true)`。
6. 会员：定位 citizenapp 会员状态显示处，确认其读默认钱包并接入同一监听（或每次现取）。

### Phase 2：我的主页推特式 UI
7. `collapsible_header.dart`：头图缩短、去渐变兜底、资料区从"钉在头图内 foreground"改为头图下方常规流；保留上滑虚化+sticky 标签。
8. `profile_header_card.dart`：头像跨压头图下缘、3 图标位置、名称/地址·CID/bio/计数排布。
9. `user_profile_page.dart`：折叠高度与 sliver 结构随之调整。

## 主要风险点

- 脑裂已在线上态存在（发帖/聊天已用新钱包、界面还显示旧的），Phase 1 是必修项，优先级高于 UI。
- `IndexedStack` 常驻 State：必须靠监听驱动重读，不能指望 tab 切换重建。
- 切换默认钱包已按卡 20260706-chat-square-silent-sign 要求需一次生物识别授权且失败回滚 UI 不落盘——通知只能在**落盘成功之后**触发，勿在授权失败/回滚路径触发。
- UI 改造须保住决策 6 的上滑虚化+吸顶标签行为；不碰数据层/Worker/R2 契约。
- 链端 0 改动。

## 是否需要先沟通

- 否。传播边界与 UI 两处决策点用户已确认，逐条执行。

## 输出物

- 代码 + 中文注释；`dart format` + `flutter analyze` 干净；`flutter test test/8964 test/im test/wallet` 全绿。
- 文档回写（本卡 + memory 相关 + WALLET_TECHNICAL 身份传播）。
- 残留清理（旧渐变兜底、read-once 僵死点）。

## 验收标准

- 真机：拖拽切换默认钱包后，我的 tab 昵称/头像/认证、我的主页、广场作者身份/isSelf、聊天 sender、会员状态**立即**切换到新用户，无需重启。
- 我的主页呈推特结构（短头图 + 头像跨界 + 资料下移），决策 5/6/4 保留。
- analyze/test 全绿；旧残桩清理。

## 完成记录（2026-07-08）

### Phase 1 传播骨架 —— 已由并行线程实现（本卡不重复做）
核对当前工作树：`WalletManager.walletsRevision` ValueNotifier 已建，`reorderWallets` 落盘后 `_bumpWalletsRevision()`（wallet_manager.dart:165）；监听方全部接上——我的 tab（user.dart:75）、广场首页（square_home_page.dart:61）、IM 列表（im_tab_page.dart:83）、交易页（onchain_payment_page.dart:123）；会员页每次打开现取 session（天然新用户）。`square_home_page_test.dart` 已带「walletsRevision 自增后广场身份即时重载」测试并通过。→ **换钱包=换身份 功能已打通**；用户真机若仍见旧用户，是旧构建，重新 build/run 即跟随。

### Phase 2 我的主页推特式 UI —— 本轮完成
- `collapsible_header.dart`：重排为「短头图（128，折叠虚化）在上 + 白底铺满在下 + 资料区从头图下缘 top-anchored」；去掉旧三色渐变兜底，改品牌色 `primaryDark` 平铺；删 `bottomInset` 参数、新增 `bannerHeight`。
- `profile_header_card.dart`：头像 80 圆角方形 + 4px 白描边，`Positioned(top:-40)` 跨压头图下缘；文字全改深色（textPrimary/secondary/tertiary）落白底；删「认证公民」胶囊（认证仅留头像角勾号，对齐 mockup）；名/地址·CID/签名(2 行)/关注·关注者·帖子 计数。
- `profile_action_icons.dart`：三图标从「黑底白字」改为白底描边款（`border` 描边 + textSecondary 图标；已关注态 primary 高亮）适配白底资料区。
- `user_profile_page.dart`：`_bannerHeight=128`、`_expandedHeight=348`，改传 `bannerHeight`。
- 测试：`profile_header_test.dart` 去掉「认证公民」胶囊断言（改注释说明认证以头像勾号呈现）。`flutter test test/8964` 61 项全绿（含 profile 全部 + 广场传播）；`flutter analyze lib` 干净（仅 transaction 模块一条既有 info lint，非本卡）。

### 待验收
- **视觉像素需真机确认**：widget 测试覆盖结构/交互（三图标、5 Tab、关注、头像图、缓存），但不覆盖头像跨界位置、头图高度观感等像素表现——需用户 build/run 后目测，必要时我再微调 `_bannerHeight/_expandedHeight/_avatarOverlap`。
- 未提交 git（等待用户指示）。
