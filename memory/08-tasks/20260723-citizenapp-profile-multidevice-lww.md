# CitizenApp 多设备资料一致性：钱包名（昵称）改为云端为真源

任务需求：同一助记词在多台设备上使用时，用户资料以「最新」为准，不再「本机永远赢」。
所属模块：citizenapp（Mobile）+ citizenapp/cloudflare（Worker，Step B）。

## 缘起

Isar 属性改名事故中，钱包「旅行者」的 Isar 行丢失，靠助记词重建后名字变成默认名「钱包1」。用户追问：名字不是也存在 Cloudflare 吗？核实结果：

- 云端**确有** `display_name`，但 `NicknamePublisher.publishDefault()` 是**只写不读**的单向镜像，全仓**没有任何**从 `displayName` 回写本机 `walletName` 的路径。
- 而且它**只推默认钱包**那一个名字，非默认钱包的名字云端根本没有。
- 更糟：本机被声明为「真源」，重建后的默认名一旦成为默认钱包并触发改名，会**反向覆盖**云端那份正确的名字。

## 冲突面只有一个字段

签名 / 头像 / 背景 / 动态 / 文章 **本来就只存云端一份**（按 accountId 取），两台设备读写同一条记录，服务端天然最后写入生效，不存在两个真源。**钱包名是唯一的双真源例外**，故工作量集中于此。

## 定稿（用户确认）

1. **模型翻转**：钱包名由「本机为真源 + 单向推云端」改为「**云端为真源 + 本机为缓存**」，与其他资料统一。这是消除冲突，而不是发明仲裁机制。
2. **零 Isar schema 变更**：同步元数据（已同步的云端 `updated_at`、待同步项）存进**已有的 `AppKvEntity`** KV 表，按 accountId 建键 —— 与通讯录 `contacts:<accountId>` / `contact_pending_ops:<accountId>` 同一范式。今日刚因 schema 变更丢过数据，本次刻意绕开。
3. **每个钱包推自己账户**：取代只推默认钱包，否则云端存不全。
4. **冷钱包**无设备子钥、云端无资料，名字保持纯本机 —— 无法消除的边界，写入文档。
5. **绝不懒注册**：按指定钱包换会话时，子钥未注册即返回 null 按不可用处理，死契约不破。

## 分步

- **Step A（App 侧，本卡执行）**：会话可按指定钱包换取；钱包名 pull/push + 待同步队列；三处改名入口与导入入口接入。
- **Step B（Worker 侧，用户明确推迟）**：`updateProfile` 增加 `edited_at`，服务端取 `max(现存, min(edited_at, now))` 并丢弃更旧写入 —— **只有做完 Step B，语义才从「最后到达者赢」变成「最新编辑者赢」**。`min(..., now)` 夹取必须有，否则系统时间被设到未来的设备会把该值永久钉死。Worker 需部署生产，属对外发布动作，由用户单独授权。

## Step A 落点

- `lib/8964/profile/services/square_session_provider.dart`：新增 `ensureSessionFor(wallet)`（冷钱包直接 null）；`ensureSession()` 改为取默认钱包后委托它。
- `lib/8964/profile/services/nickname_publisher.dart`：补 `resolveRemote(accountId)` 拉取、`publishFor(wallet, name)` 按钱包推自己账户、待同步队列（AppKvEntity）与 `syncWalletName(wallet)` 编排。
- `lib/wallet/pages/wallet_page.dart`：两处改名入口改走新流程；进页对热钱包 best-effort 同步。
- `lib/wallet/pages/import_wallet_page.dart`：导入成功后拉云端名。
- `lib/8964/profile/profile_edit_page.dart`：改名后同步缓存元数据，避免与新流程打架。
- 测试：`test/wallet/` 补 pull/push/待同步重放/云端更旧不覆盖。

## 风险

- **推翻既有定稿**（「本机钱包名是真源」），必须同步改写 `CITIZENAPP_TECHNICAL.md`，否则后人按旧注释改回单向推。
- **不得形成改名回环**：从云端拉回写本机时直接调 `WalletManager.renameWallet`，**不触发推送**（推送只由 UI 改名入口发起）。
- 仅做 Step A 时语义仍是「最后到达者赢」；离线久的设备上线后可能用旧编辑覆盖新编辑 —— 待 Step B 修正。
- 命名债：`nickname_publisher.dart` 在承担双向同步后名字偏窄，后续可考虑更名，本次按已批准路径不动。

## 验收

- `flutter analyze` 0 问题；全量 `flutter test` 通过。
- 真机：导入钱包后取回云端昵称；本机改名后云端可见；断网改名进队列、联网后重放。

## 执行结果

### Step A（2026-07-23，完成）

- **会话按钱包换取**：`SquareSessionProvider` 新增 `ensureSessionFor(wallet)`（冷钱包直接 null），`ensureSession()` 改为取默认钱包后委托它。**死契约保持**：绝不懒注册 / 绝不弹 Turnstile / 绝不读 seed，未注册子钥即按不可用返回。
- **同步器**：`NicknamePublisher` 由「只写不读的单向镜像」改为双向同步器 ——
  - `onLocalRename(wallet, name)`：入待同步队列 → 推到**该钱包自己 accountId** 的 `display_name`（旧实现只推默认钱包，云端存不全）。
  - `syncWalletName(wallet)`：先重放待同步项；**队列非空则直接返回**（本机改动未上云，绝不能被云端旧值覆盖）；再拉云端，`updated_at` 比已同步值新且名字不同才回写本机。
  - **防回环**：回写走 `WalletManager.renameWallet`，不触发推送；推送只由 UI 改名入口发起。
  - `resolveRemote(accountId)`：按账户读云端昵称。
  - 旧 `publishDefault()` **已删除**，两处调用点全部迁走，无残桩。
- **零 Isar schema 变更**：元数据存已有 `AppKvEntity` —— `wallet_name_synced_at:<accountId>`（intValue）、`wallet_name_pending:<accountId>`（stringValue + 编辑时刻 intValue）。今日刚因 schema 变更丢过数据，刻意绕开。
- **接线**：`wallet_page` 两处改名入口改走 `onLocalRename`；`_reload()` 增 `_syncWalletNames()`（逐钱包隔离 best-effort，单个失败只跳过并 `debugPrint`，同步后重读一次列表）；`import_wallet_page` 导入成功后 `syncWalletName(profile)` 取回云端昵称。
- **测试**：新建 `test/wallet/nickname_sync_test.dart`（4 例：云端更新回写本机 / 云端更旧不覆盖 / 推送失败入队且待推送期间不被云端覆盖、恢复后重放 / 冷钱包跳过）。
- **文档**：`USER_TECHNICAL.md` 改写「展示规则」旧表述，并新增「钱包名同步模型（2026-07-23 翻转：云端为真源）」小节，含防回环、待推送不覆盖、零 schema、会话契约、冷钱包边界与「最后到达者赢」的已知限制。（`CITIZENAPP_TECHNICAL.md` 未提及钱包名/昵称，无需改。）
- **验收**：`flutter analyze`(lib+test) 0 问题；全量 `flutter test` **792 通过 / 5 跳过 / 0 失败**（+4）。
- **真机验收未做**：设备中途 USB 断开，`adb devices` 为空。待重连后验证：导入取回云端昵称、改名后云端可见、断网改名入队并在联网后重放。
