# citizenwallet 删分组 + 钱包详情页 UI 改造

状态:done(需求 1、2 落地并通过验收 2026-07-26;需求 3 暂缓,另起卡)
所属模块:Mobile(citizenwallet 冷钱包)
工作根:`/Users/rhett/GMB/citizenwallet/`(仓库顶层 `/Users/rhett/GMB`,只改主检出)

## 需求

1. **彻底删除分组功能**(全仓无残留)。
2. **钱包详情页 UI**:助记词功能**原样不动、只挪位置**——顶部卡片改为「钱包图标(去外框、缩小、上移)+ 名称」在上、「助记词区域(= 现有助记词区原样:点击查看→确认→就地展开→隐藏)」在下;删「N 个账户」;不加任何「查看」按钮;删除下方独立的助记词卡与分组卡。
3. (暂缓)每钱包卡片独立扫码,只扫本钱包账户。**本轮不做。**

## 需求 1 —— 删分组(逐文件)

| 文件 | 删除内容 |
|---|---|
| `lib/isar/wallet_isar.dart` | `allGroup` 常量、`WalletEntity.groupNames` 字段、`WalletGroupEntity` 集合、schema 列表里的 `WalletGroupEntitySchema`、`_defaultGroupNames` + `_ensureDefaultGroups`(及其调用) |
| `lib/isar/wallet_isar.g.dart` | build_runner 重生成 |
| `lib/wallet/wallet_manager.dart` | `Wallet.groupNames` 字段 + `inGroup()` + 构造默认值;`_toWallet`/`_appendWalletAtomic` 的 groupNames |
| `lib/ui/group_management_page.dart` | 整文件删除(含上轮加的 `groupNameFormatError`/`groupNameMaxRunes`) |
| `lib/ui/home_page.dart` | group import、`_groups`/`_selectedGroup`/`_filteredWallets`/`_buildGroupRow`/`_openGroupManagement`、`_loadAll` 分组加载、列表顶部筛选行;清理不再用的 isar imports |
| `lib/ui/wallet_detail_page.dart` | group 字段/`_toggleGroup`/`_buildGroupSection`(与需求 2 合并处理) |
| `test/ui/group_name_validation_test.dart` | 整文件删除 |
| `test/wallet/wallet_model_test.dart` | 删 `Wallet.inGroup` 测试组 + `wallet()` helper 的 groupNames |

数据模型变更(删字段+删集合):[[in-development-zero-users]] 零用户 → 删库重建,无 migration(沿用 wallet_isar 既定策略)。

## 需求 2 —— 详情页(仅 `lib/ui/wallet_detail_page.dart`)

- 新「身份卡」:`cardDecoration` 单卡,上排 [小图标(无外框,primaryLight)+ 名称(贴顶)],下方 = 现有助记词区(label + 点击查看/展开/隐藏)**原样**。
- 删 `_buildHeader`(渐变卡 + 「N 个账户」);助记词渲染从独立卡剥离(去外框 Container),并入身份卡;删分组卡。
- 助记词交互与安全边界(`getMasterMnemonic` 生物识别 + `ScreenshotGuard` 防截屏 + 不复制)**一字不改**。

## 验收
- `dart analyze` 0 + `flutter test` 全绿;分组残留复扫(`WalletGroupEntity`/`groupNames`/`allGroup`/`group_management`/`inGroup`)全 0。

## 进度(2026-07-26 完成 需求 1、2)

需求 1(删分组):
- `wallet_isar.dart`:删 `allGroup`、`WalletEntity.groupNames`、`WalletGroupEntity` 集合、schema 列表项、`_defaultGroupNames`+`_ensureDefaultGroups`(及调用);`wallet_isar.g.dart` build_runner 重生成(WalletGroupEntity/groupNames 已消失)。
- `wallet_manager.dart`:删 `Wallet.groupNames`+`inGroup()`+构造默认值;`_appendWalletAtomic`/`_toWallet` 去 groupNames。
- `home_page.dart`:删 group_management/wallet_isar/isar imports、`_groups`/`_selectedGroup`/`_filteredWallets`/`_buildGroupRow`/`_openGroupManagement`、`_loadAll` 分组加载;`_buildWalletList` 直接平铺 `_wallets`(去筛选行/分组空态)。
- 删整文件 `lib/ui/group_management_page.dart`、`test/ui/group_name_validation_test.dart`;`wallet_model_test.dart` 删 `Wallet.inGroup` 组 + `wallet()` helper。

需求 2(详情页 `wallet_detail_page.dart`):
- 新 `_buildIdentityCard`:`cardDecoration` 单卡,上排 [22px 无外框图标 + 名称贴顶],下方 `_buildMnemonicArea`。
- `_buildMnemonicSection`→`_buildMnemonicArea`:剥掉外层卡容器,助记词区(点击查看→确认→就地展开→隐藏)交互/安全边界一字不改。
- 删 `_buildHeader`(渐变卡 + 「N 个账户」)、`_buildGroupSection`、`_toggleGroup`、group 字段;清理 isar imports。

终验:`dart analyze` **0** + `flutter test` **209 passed**(= 前 215 − 删除的 6 条分组测试,无回归);分组残留复扫(`WalletGroupEntity`/`groupNames`/`allGroup`/`group_management`/`inGroup`)全 0。

需求 3(每钱包独立扫码)按用户要求本轮未做,后续另起卡。
