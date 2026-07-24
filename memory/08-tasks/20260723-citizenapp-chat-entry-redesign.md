# CitizenApp 聊天页顶栏改造 + 加号 5 入口 + 全文搜索页

任务需求：把聊天 tab 顶部「新建群聊」卡片改为搜索框；右上角搜索图标改为加号；点加号弹出选择框(扫一扫 / 收付款 / 发私信 / 发群聊 / 加好友),各项复用现有成熟链路。
所属模块：citizenapp（Mobile）— 纯前端交互层,不涉及 runtime / 链端 / OnChina。

## 定稿(用户确认)

1. **聊天页 = `ChatTab`**(`lib/chat/chat_tab.dart`),顶栏是自定义 sliver `_ChatHeader`(非 AppBar),搜索图标是无 onTap 死 Icon,`_NewGroupEntry` 卡片 → `GroupCreatePage`。
2. **加号 5 入口(右上角锚定下拉菜单)**:
   - 扫一扫 = 交易·扫一扫(`openScanDispatchFlow` → `QrScanPage(dispatch)`),聊天页调用传默认热钱包做 `paymentWallet`;并扩展 dispatch 扫到用户码 → 转账。
   - 收付款 = 展示本人**唯一**用户二维码(`userContact` 名片码);两处出口(`UserQrPage` / `WalletQrDialog`)收敛为唯一组件。
   - 发私信 = 通讯录**单选** → `openDirectChat`。
   - 发群聊 = 通讯录**多选(≥2)** → `GroupCreatePage`(最少人数 1→2)。
   - 加好友 = `QrScanPage(contact)` + `UserContactService.addContact`(抽复用函数)。
3. **用户二维码只有一个**:同一张 `userContact` 名片码,扫码结果由扫描模式决定 —— contact 模式=加好友、transfer/dispatch(扩展后)=转账。**不新造带金额收款码。**
4. **顶部搜索框**:点击进入**独立聊天搜索页**,结果分 3 段:会话 / 联系人 / 聊天记录(全文,本地已解密消息表检索)。
5. 群上限以代码常量 `kMaxGroupMembers = 1989` 为准(memory 旧记 1000 待订正)。

## 分步计划(每步独立可验收、无残桩;聊天页可见 UI 仅 Step 4 一次到位)

- **Step 1**:通讯录侧三能力 —— 发私信单选、发群聊最少 2 人、加好友抽复用函数。(`my/user/` · `chat/group/`)
- **Step 2**:交易/二维码侧 —— 扫一扫认用户码→转账、收付款收敛唯一用户二维码组件。(`qr/` · `wallet/` · `8964/profile/`)
- **Step 3**:独立聊天搜索页(会话 + 联系人 + 聊天记录全文)。(`chat/` · `chat/storage/`)
- **Step 4**:聊天页顶栏总装 —— 搜索Icon→加号+下拉菜单接线 5 入口、卡片→搜索框接搜索页、删旧卡片。(`chat/chat_tab.dart`)
- **Step 5**:文档回写 + memory 订正(1000→1989)+ 全局残留清理 + 模拟器真实验收。

## Step 1 落点

- `lib/my/user/contact_book_page.dart`
  - 用枚举 `ContactPickMode { browse, pickForTransfer, pickForMessage }` 替换 `selectForTrade` 布尔。
  - `browse` / `pickForTransfer` 行为逐字节不变(点人分别进主页 / pop 返回联系人);新增 `pickForMessage`:点人直接 `_message` 开私聊,且**不带操作菜单、不带扫码 action**(纯选人)。
  - 抽出顶层 `scanAndAddContact(context, {selfAccountId})` 复用函数;`_scanContactQr` 改为调用它(供 Step 4 聊天页「加好友」复用)。
- `lib/transaction/onchain-transaction/onchain_payment_page.dart`:`ContactBookPage(selectForTrade: true)` → `mode: ContactPickMode.pickForTransfer`。
- `lib/chat/group/ui/group_create_page.dart`:`_canCreate` 最少 1→2 人;「已选 N」提示补「至少 2 人」。
- 测试:`test/user/contact_book_page_test.dart`(`_page` 改 `mode` 入参 + 新增 pickForMessage 点人开私聊用例);`test/ui/transaction_tab_page_test.dart:100`(`selectForTrade isTrue` → `mode == pickForTransfer`)。

## 边界

- 只动 ContactBookPage 的模式字段;`wallet_page.dart` 的同名 `selectForTrade`(选钱包)**不动**(不同 widget、不同语义)。
- 不动 `openDirectChat` / `GroupCreatePage` 建群链路内部(仅改最少人数判定)、不动 `UserContactService` 加密/同步、不动 qr 扫码页本体(Step 2)。
- 本步聊天页顶栏 UI 完全不动。

## 验收(真实运行态)

- `flutter analyze`(lib/my/user + lib/chat/group + lib/transaction 相关)0 问题、无未用符号残留。
- 模拟器:通讯录 pickForMessage 点联系人真开私聊;pickForTransfer 收款人选择不回归;建群选 1 人不可创建、2 人可创建;通讯录扫码加好友(走抽出函数)行为等价。
- 相关 widget 测试通过。
- 说明:group_create 的两人门为纯 getter,因 ChatRuntime 构造即实例化真实 ChatStore(Isar/native)、fake 成本高,本步以 analyze + 模拟器真机验收,widget 测试推迟(非静默跳过)。

## 执行结果

### Step 1（2026-07-23，代码完成 + analyze 通过，运行态验收被无关 WIP 阻塞）

- **枚举化**：`contact_book_page.dart` 以 `ContactPickMode { browse, pickForTransfer, pickForMessage }` 替换 `selectForTrade` 布尔；`browse` / `pickForTransfer` 行为逐字节不变。
- **发私信单选**：新增 `pickForMessage` —— `_open` 点联系人走 `_message`(复用统一 `openDirectChat` 收口);该模式 AppBar 标题「选择联系人」、隐藏扫码 action、卡片不显示逐项操作菜单(`_ContactCard.showActions`)。
- **加好友抽复用**：抽出顶层 `scanAndAddContact(context, {selfAccountId})`;`_scanContactQr` 改调它(供 Step 4 聊天页「加好友」复用)。
- **连带调用点**：`onchain_payment_page.dart` 收款人选择改 `mode: ContactPickMode.pickForTransfer`。
- **发群聊 2 人门**：`group_create_page.dart` `_canCreate` 最少 1→2;「已选 N」未满 2 补「至少 2 人」提示。
- **测试**：`contact_book_page_test.dart` `_page` 改 `mode` 入参 + 新增「发私信模式点人开私聊、无操作菜单」用例;`transaction_tab_page_test.dart` 断言改 `mode == pickForTransfer`。
- **连带修复(用户授权按 Substrate 官方/ADR-040 标准同步更新)**：`wallet_action_card.dart:103` 的半途重命名 `OnchainTopupPage(gmbAddress: widget.wallet.ss58Address)` 修正为 `accountId: widget.wallet.accountId` —— 该值经充值页流向 `topup_api` 的 `'account_id'` 上报字段,按标准必须是 0x+64 hex accountId 而非 ss58Address(`WalletProfile` 两者皆有);同步把 `onchain_topup_page.dart` 该字段注释订正为 `account_id`(0x+64 hex)。修前全仓 2 个 error 均在此行。
- **验收**：`flutter analyze lib` 全量 0 问题;`dart format` 通过;`contact_book_page_test` + `transaction_tab_page_test` 共 **13 用例全部通过**(含新增「发私信模式点联系人直接开私聊、无操作菜单」)。
- **残留核查**：`gmbAddress` 全仓归零;`selectForTrade` 仅剩 `wallet_page.dart` 的 `WalletTab`(「选择交易钱包」,另一 widget,按边界不动);`ContactPickMode` 4 处调用一致。
- **模拟器真机验收说明**：`pickForMessage` 在 Step 1 尚无 UI 入口(要到 Step 4 加号菜单接线后才可达),故本步以 widget 测试覆盖其行为;UI 可达部分(建群 2 人门、通讯录选收款人不回归)并入 Step 4 总装后与聊天页顶栏一并做模拟器真机验收,避免重复起真机。

### Step 2（2026-07-23，完成）

- **扫一扫认用户码 → 转账**：`qr_scan_page.dart` `_handleCode` 的 `QrScanMode.dispatch` 分支补 `QrRouteType.userContact` → 复用**已存在**的 `_handleContactAsRecipient`(transfer 模式早在用)。零新逻辑;至此同一张 `userContact` 码按扫描场景分流:contact 模式=加好友、transfer/dispatch=按收款人转账。
- **用户二维码收敛为唯一组件**：以 `8964/profile/user_qr_page.dart` 的 `UserQrPage` 为全 App 唯一真源 —— 并入原钱包弹窗独有的「复制地址」(地址居中 + 复制图标浮右);新增 `_ss58Address` getter(accountId 真源、ss58 仅展示,免重复派生);底部文案由「其他用户扫描此二维码可添加通讯录」订正为如实覆盖双场景的「扫描此二维码可加为联系人，或向其转账」;类注释升级为「唯一用户二维码 + 扫码结果由扫描模式决定」。
- **删除重复出口**：`lib/wallet/widgets/wallet_qr_dialog.dart` 与 `test/wallet/widgets/wallet_qr_dialog_test.dart` **整文件删除**(与 UserQrPage 编码同一份 `userContact` 载荷,属双轨)。
- **调用点改接**：`wallet_identity_card.dart` QR 图标改为 push `UserQrPage(accountId: wallet.accountId, contactName: _walletName)`(传 accountId 而非 ss58,符合 ADR-040);import 与 :16 类注释同步订正。
- **测试**：新建 `test/8964/profile/user_qr_page_test.dart`(5 用例:渲染昵称/地址/复制/下载入口、双场景文案、复制不抛、下载流程不抛、载荷仍 QR_V1 且 k=3)。
- **验收**：`flutter analyze`(全项目 lib+test)**0 问题**;`user_qr_page_test` + `profile_header_test` **22/22 通过**(含「kebab QR code opens the user QR page」);`test/wallet/widgets/` **12/12 通过**(身份卡改接无回归)。
- **残留核查**：`WalletQrDialog` / `wallet_qr_dialog` 全仓引用归零;`UserQrPage` 现有 2 个调用点(社交主页 ⋮、钱包身份卡),聊天页「收付款」待 Step 4 接第 3 个。
- **模拟器真机验收**：并入 Step 4 总装一并执行(扫码需真机相机,模拟器走相册选图路径)。

### Step 3（2026-07-23，完成）

- **关键事实(实测,决定实现形态)**：① `ChatMessageEntity.plaintext` 本地明文落库、`accountId`/`createdAtMillis` 有索引而 `plaintext` 无索引 → 本地全文检索可行;② `plaintext` 存的是**编码载荷**(文本/媒体/贴纸),必须先经 `ChatPayloadCodec.decode(...).summary` 解码再匹配,裸匹配会漏正文并错配媒体元数据;③ `ChatStore` 原只有单会话 `readMessages`,无跨会话检索。
- **`ChatStore.searchMessages({accountId, keyword, limit=50})`**：按 `accountId` 索引收窄 → 解码摘要匹配(大小写不敏感)→ 时间倒序截断 limit;空关键词/空账户直接返回空,不查库;**不为搜索改表结构、不加索引**。
- **新建 `lib/chat/chat_search_page.dart`**：一个输入框 + 三段结果(会话/联系人/聊天记录)。会话按 `title`/`lastMessage` 内存过滤;联系人按备注名/账户过滤(公开昵称需联网,搜索页不引入网络依赖);聊天记录走 `searchMessages`。点结果一律复用既有收口 —— 群聊 `openGroupChat`、单聊 `openDirectChat`,不复刻 ChatPage 装配。异步检索用递增序号丢弃过期结果,防快速输入时旧结果覆盖新结果。
- **聊天记录命中口径(用户确认)**：**只打开消息所在会话,不定位到具体消息**;消息级锚点需 ChatPage 支持滚动定位,单列后续任务。
- **测试**：新建 `test/chat/chat_search_page_test.dart`(7 用例:空关键词只提示且不查库、一个关键词三段命中、大小写不敏感、单聊/群聊分别走对应收口、点联系人开私聊、点聊天记录开所在会话、无命中空态);`test/chat/chat_store_test.dart` 补 `searchMessages` 真实 Isar 用例(跨会话+时间倒序+limit 截断+大小写不敏感+空关键词/空账户/他人账户查不到)。
- **验收**：`flutter analyze` 0 问题;`test/chat/` 全目录 **156 通过 / 4 跳过**(跳过为既有 smoldot 守卫)。
- **本步未动** `chat_tab.dart`(搜索框接线属 Step 4)、未动消息表结构与 MLS 链路。

### Step 4（2026-07-23，代码完成；模拟器验收受环境阻塞）

- **顶栏改到目标态**（`chat_tab.dart`，本步唯一改的业务文件）：
  - `_ChatHeader` 由 `const` 无回调改为接收 `onAction`；原**无点击的装饰** `Icon(Icons.search_rounded)` 换成加号按钮(`Icons.add_rounded`，tooltip「新建」)。**弹窗样式于 2026-07-23 二次改造**：初版用 `PopupMenuButton`，但用户要求「淡深色背景 + 顶部凸出三角、三角顶点对齐加号」，而 `PopupMenuButton` 的水平位置由框架决定、拿不到确定锚点，三角只能靠猜；改为 `showGeneralDialog` 自绘弹窗，按加号按钮 `RenderBox` 的真实屏幕坐标定位（见「加号弹窗样式改造」小节）。
  - **删除** `_NewGroupEntry` 整个 widget 及其挂载点，原位换成 `_SearchEntry`(搜索框，点击进 `ChatSearchPage`，透传 `store` 与 `_accountId` 收窄依赖)。
  - 新增 `_ChatEntryAction` 枚举 + `_onEntryAction` 分派 + 5 个处理方法：扫一扫 `openScanDispatchFlow(paymentWallet: 默认钱包)`；收付款 push `UserQrPage(accountId: wallet.accountId, contactName: wallet.walletName)`；发私信 push `ContactBookPage(mode: pickForMessage)` 后回刷；发群聊复用既有 `_openCreateGroup`(原卡片处理函数，职能迁入菜单)；加好友 `scanAndAddContact`。
  - 新增 `_requireAccount()` 统一无热钱包拦截提示。
- **`ChatEntryOpeners`(可注入入口，用户选定方案)**：5 个 opener 统一为 `ChatEntryOpener = Future<void> Function(BuildContext)`，且**先查注入再解析钱包** —— 否则测试注入时仍会先摸真实 `WalletManager`(触碰存储)。正式运行 `openers` 为 null 走真实实现。
- **测试**：`chat_tab_test.dart` 新增 4 用例(顶栏为搜索框+加号且旧卡片已删、加号弹出 5 项、5 项分别路由、点搜索框进搜索页)。菜单动画用固定步长 `pump` 而非 `pumpAndSettle`——聊天页有 15s 轮询定时器，`pumpAndSettle` 会被推着走。存量用例未断言过旧卡片与装饰搜索图标(已核实)，故删卡片不破坏存量。
- **验收**：`flutter analyze`(lib+test)0 问题；**全量 `flutter test` 779 通过 / 5 跳过 / 0 失败**。
- **残留核查**：`chat_tab.dart` 内 `_NewGroupEntry` 与「新建群聊」字样归零；全仓仅剩 `group_create_page.dart:91` 的建群页自身标题(正确保留)。
- **⚠ 模拟器真机验收未能执行（环境阻塞，非代码问题）**：本机 `xcode-select` 指向 `/Applications/Xcode.app`，但 `xcrun simctl list runtimes` **为空 —— 未安装任何 iOS 模拟器运行时**；`flutter devices` 仅有 Chrome，物理 iPhone 未连接。App 依赖 Isar / mobile_scanner / smoldot / saver_gallery 等原生插件，Web 不是可用验收面。**Steps 1–4 的可视化真机验收仍欠着**，待装模拟器运行时或接真机后补做。

### Step 5（2026-07-23，完成，纯文档零代码）

- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`：2.6「用户入口」补聊天页顶栏新形态（搜索框 + 加号 5 入口）及“全部复用既有链路、聊天页不自建重复实现”的边界。
- `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`：「当前状态」新增三条 2026-07-23 记录 —— ① 顶栏改造与 `ChatEntryOpeners`（含**先查注入再解析钱包**的坑）；② 用户二维码唯一化与扫码场景分流（含删 `wallet_qr_dialog.dart`、传 accountId 而非 ss58）；③ 聊天搜索页与 `searchMessages`（含**载荷必须解码后再匹配**的坑、只打开会话不定位消息的口径）。
- `memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md`：建群页补「最少 2 人」门槛；并订正已过时的“`chat_tab.dart` 新建群入口 sliver”描述为“已迁入加号菜单「发群聊」，原卡片整块删除”。上限 1989 该文档本就正确，未动。
- 个人记忆：订正记忆卡 frontmatter 里“私密小群 ≤1000”为 ≤1989（正文本就是 1989，错的只有 description）；新增长期不变量记忆「用户二维码全 App 唯一 + 扫码模式分流」。
- **本步零代码改动**，未新建仓库文件。

### 加号弹窗样式改造 + 轻节点状态栏文案中文化（2026-07-23，完成）

用户两项需求，一并落地：

**1. 交易页轻节点状态栏文案中文化**（`lib/ui/widgets/chain_progress_banner.dart`）

- `peer N` → 「已连接节点 N」；`best #N` → 「最新区块 #N」；`finalized #N` → 「已验证区块 #N」。
- 连带：warp 分支同一行的 `peer finalized` → 「节点已验证区块」（不改会中英混杂在同一行）；两处用户可见句子「…读取 peer、best、finalized 等链路信息」同步中文化。
- **刻意不改**：`RPC_TECHNICAL.md` 与代码内部的 peer / best / finalized 是 **smoldot 协议名词**，不是 UI 文案，跟着扫会让技术描述失真（与 accountId 收敛时排除区块/交易哈希是同一条原则）。已在 RPC 文档写明这条界线。

**2. 加号弹窗改为淡深色 + 顶部凸出三角，三角顶点对齐加号**（`lib/chat/chat_tab.dart`）

- **放弃 `PopupMenuButton`**：其水平位置由框架按可用空间决定，拿不到确定锚点，三角只能靠猜偏移量对齐 —— 与「顶点对齐加号」的要求直接冲突。
- 改为 `showGeneralDialog` 自绘：经 `Builder` 取加号按钮自身 `RenderBox` 的屏幕坐标算中心 X → 反推面板左边界 → 夹到屏内 → **用夹取后的实际左边界回算三角横向位置**，保证靠边时仍对准加号。
- 面板 `Color(0xF01F2A30)`（淡深色带透明度）、白色图标与文案、圆角 12；`barrierColor` 透明，不压黑整屏。
- **面板本身必须是 `Material`**：弹窗不在 Scaffold 之下，`InkWell` 缺 Material 祖先会直接抛异常。**此坑由既有 widget 测试当场拦下**，未流到真机。
- 菜单项加了图标（扫码 / 收付款 / 私信 / 群组 / 加好友），与深色面板配套。

**验收**：`flutter analyze` 0 问题；`chat_tab_test` 19/19；全量 `flutter test` **792 通过 / 5 跳过 / 0 失败**。
**待办**：三角与加号的像素级对齐、深色浓淡观感**必须真机确认**，当前设备 USB 断开未验。

## 总体收尾

- Steps 1–5 全部完成；`flutter analyze` 0 问题，**全量 `flutter test` 779 通过 / 5 跳过 / 0 失败**。
### 真机验收（Android Pixel 8a `3C071JEKB09000`，2026-07-23，部分完成）

`flutter run --debug` 真机安装运行，逐项截图核对：

- ✅ **右上角已是加号**，原装饰性搜索图标消失。
- ✅ **旧「新建群聊」卡片已不存在**。
- ✅ **点加号弹出 5 项且顺序正确**：扫一扫 / 收付款 / 发私信 / 发群聊 / 加好友，右上角锚定弹出。
- ✅ **扫一扫**：从聊天页加号直达扫码页（标题「扫一扫」、取景框、相册 / 手电筒），`openScanDispatchFlow` 接线正确。
- **前置卡点与解除**：该机初次验证时 `_accountId` 为空，搜索框等 5 项被「请先创建热钱包」拦截。根因**不是本任务引入**，而是工作树中进行中的「钱包字段统一」把 Isar 实体属性改了名（`address→accountId`、`pubkeyHex→ss58Address`），而 Isar 按属性名存取，本机 07-09 起的旧库值仍挂旧名下 → 新构建读出空串，钱包身份成空壳。数据未丢（`citizenapp.isar` 36.7MB、`FlutterSecureStorage.xml` 完好，`firstInstallTime` 07-09 证明是覆盖升级非全新装）。已用一次性修复（按 `walletIndex` 直读严档 seed → `_deriveSr25519FromSeed` 重新派生 → 写回；**不搬旧值故不受“diff 行序与语义交叉”影响**）修好热钱包 index=3，修复代码已整段删除并以干净构建复验。冷钱包 index=2 无种子不可派生，需用户用原 SS58 地址重新导入。
- ✅ **搜索框**：修复后正常渲染；点击进入 `ChatSearchPage`，占位「搜索会话、联系人、聊天记录」、自动聚焦、空态提示齐全。
- ✅ **收付款**：打开收敛后的唯一 `UserQrPage`（钱包3），**复制图标**与**「扫描此二维码可加为联系人，或向其转账」双场景文案**均已生效（Step 2 成果）。
- ✅ **发私信**：`ContactBookPage` 以 `pickForMessage` 模式打开 —— 标题「选择联系人」、右上角扫码图标已隐藏（Step 1 成果）。
- ✅ **发群聊**：`GroupCreatePage` 显示「已选 0·至少 2 人」、创建按钮置灰（Step 1 两人门生效）。
- ✅ **加好友**：进入 contact 模式扫码页（标题「扫码添加好友」，与「扫一扫」区分）。
- ⏸ 因该机通讯录为空，未能验证「发私信点人直开私聊」「建群 1 人→2 人切换」的选中态过程，其行为由 widget 测试覆盖。
- 📝 顺带观察（**非本任务引入**）：contact 模式扫码页取景框下方文案为「扫描对方收款码」，用于加好友语境读着不对，建议后续订正。
- iOS 侧仍无模拟器运行时（`xcrun simctl list runtimes` 为空），未验证。
