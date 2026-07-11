# CitizenApp 默认钱包统一身份（废聊天账户，聊天+发动态同源）

## 任务需求

- 废除“聊天账户”概念，改为“**默认用户钱包 = 用户身份**”，同时用于聊天和发动态。
- 默认 = 钱包列表中**最靠前的热钱包**（冷钱包永不成为默认、拿不到徽标，但不拦拖动）。
- 用户可在「我的 → 我的钱包」拖拽置顶来改默认（拖拽/`sortOrder` 已存在）。
- 默认钱包卡片：三个竖点菜单**左侧**显示「默认用户」徽标。
- 收敛现有两套身份：聊天用 `communicationWalletIndex`、发动态用 `activeWalletIndex`，统一到默认钱包。

## 建议模块

- 钱包列表/卡片/拖拽：`citizenapp/lib/wallet/pages/wallet_page.dart`
- 默认判定：`citizenapp/lib/wallet/core/wallet_manager.dart`（基于 `sortOrder` + `isHotWallet` 派生，无新 Isar 字段）
- 用户资料（废聊天账户 UI/字段）：`citizenapp/lib/my/user/user.dart`、`user_service.dart`
- 聊天身份：`citizenapp/lib/chat/chat_runtime.dart`
- 广场身份：`citizenapp/lib/8964/services/square_identity_state.dart`

## 影响范围

- 新增“默认钱包”派生：`getDefaultWallet()` = `getWallets()` 中第一个 `isHotWallet`；不新增 Isar 字段（复用 `sortOrder` 排序）。
- `WalletListTile` 增 `isDefault` 入参 → 渲染「默认用户」徽标。
- `UserProfile` 三字段 `communicationWalletIndex/Address/Name` 下沉为“读默认钱包”，逐步废弃。
- 昵称 = 默认钱包名（“改昵称”即“改默认钱包名”，双向同步链路对齐）。
- 我的二维码 / 联系人自我地址 / 进聊天 sender 全改读默认钱包地址。
- 广场 `SquareIdentityService.loadCurrent` 从 `getWallet()`（活跃钱包）改为 `getDefaultWallet()`。
- `activeWalletIndex`（交易/付款活跃钱包）保持独立，不复用、不混淆。

## 主要风险点

- 无默认钱包场景（仅冷钱包 / 无钱包）：聊天与发动态入口需给清晰引导（“请先在我的钱包创建热钱包”），不得崩溃。
- 历史数据迁移：老用户 SharedPreferences 里已有 `communicationWalletIndex`，首启需对齐到默认钱包并可安全丢弃旧字段。
- 昵称双向同步（改钱包名 ↔ 改昵称）链路不能断。
- 属于“修改关键交互路径”，为 citizenapp 模板“先沟通”项——已获用户确认（默认落最靠前热钱包 / 聊天+广场统一）。
- 不改 Isar 结构（采用派生方案）；若后续要显式 `isDefault` 字段再单独评估迁移。

## 是否需要先沟通

- 否。三处关键决策已用户确认：①默认落最靠前热钱包；②聊天+广场统一到默认钱包；③配合 Worker 内置生产地址（另卡）。

## 预计修改目录

- `citizenapp/lib/wallet/`：默认派生 + 「默认用户」徽标 + 拖拽语义；涉及代码。
- `citizenapp/lib/my/user/`：废聊天账户 UI/字段，改读默认钱包；涉及代码。
- `citizenapp/lib/chat/`：聊天身份改默认钱包；涉及代码。
- `citizenapp/lib/8964/`：广场身份改默认钱包；涉及代码。
- `citizenapp/test/`：更新/新增默认钱包派生与身份重指向单测；涉及测试。
- `memory/05-modules/citizenapp/`：记录身份模型统一；涉及文档。

## 分步骤技术方案

### 步骤 1：默认钱包派生

- `WalletManager.getDefaultWallet()` / `getDefaultWalletIndex()` = `getWallets()` 首个 `isHotWallet`。
- 单测覆盖：全冷钱包、无钱包、多热钱包、拖拽后置顶变化。

### 步骤 2：「默认用户」徽标

- `WalletListTile` 加 `isDefault` 入参，默认卡三点竖点左侧渲染徽标。
- 列表在 build 时计算默认 `walletIndex` 下发给对应卡片。

### 步骤 3：聊天重指向

- `ChatRuntime._readCommunicationAccount` 改用 `getDefaultWallet()`（热钱包保证与现有 `isHotWallet` 断言一致）。
- 无默认时抛清晰引导错误，替换原“请先在用户资料中设置聊天账户”。

### 步骤 4：广场重指向

- `SquareIdentityService.loadCurrent` 由 `getWallet()` 改 `getDefaultWallet()`。
- 保持 `cidNumber` 链上读取逻辑不变。

### 步骤 5：用户资料收敛

- 删除 `_selectCommunicationWallet` 与「聊天账户」设置行。
- 昵称/二维码/自我地址改读默认钱包名与地址。
- 首启把老 `communicationWalletIndex` 对齐默认钱包后可丢弃。

### 步骤 6：清理与验收

- 废弃/迁移 `communicationWalletIndex/Address/Name` 及 `setCommunicationWallet`（先标注废弃并迁移，再删）。
- `flutter analyze` / `flutter test`。
- 人工验收：拖拽改默认 → 聊天 sender、发帖 owner_account 同步变化；冷钱包置顶不拿徽标、默认落到最靠前热钱包。

## 当前执行状态

- [x] 步骤 1：`WalletManager.getDefaultWallet()` / `getDefaultWalletIndex()` = `getWallets()` 首个热钱包。
- [x] 步骤 2：`WalletListTile` 加 `isDefault`，默认卡三点左侧渲染「默认用户」徽标；列表用 `defaultUserWalletIndex()` 计算并下发。
- [x] 步骤 3：`ChatRuntime._readCommunicationAccount` / `readCommunicationAddress` 改读 `getDefaultWallet()`；无默认时抛「请先创建热钱包」引导；清掉未用的 `profileService` 依赖。
- [x] 步骤 4：`SquareIdentityService.loadCurrent` 由 `getWallet()`（活跃钱包）改 `getDefaultWallet()`。
- [x] 步骤 5：删「聊天账户」设置行与 `_selectCommunicationWallet`；资料页昵称/二维码/自我地址、联系人「发消息」入口、`chat_tab` 兜底全部改读默认钱包。
- [x] 步骤 6：删 `UserProfileState` 三个通信字段 + `nickname` getter + `setCommunicationWallet`/`updateCommunicationWalletName` + 两处 wallet_page 双向同步残桩 + 各处未用导入；`MyWalletPage.bindPurposeLabel` 默认改中性词。
- [x] 验收：`dart analyze lib test` 干净（唯一 info 为未触及文件既有 lint）；`flutter test --concurrency=1` 覆盖 wallet/user/chat/8964 全绿（含新增默认钱包判定 4 例 + 徽标 2 例）。
- [ ] 待用户真机验收：拖拽改默认 → 聊天 sender、发帖 owner_account 同步变化；冷钱包置顶不拿徽标、默认落最靠前热钱包。
