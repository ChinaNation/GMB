# 交易页扫码改造：扫码收进收款地址框、多签账户独立卡片、扫码填地址抽共用组件

任务需求：交易页「扫一扫」入口删除，改为收款地址输入框内右侧的扫码图标（专用于填入
收款地址）；顶部只保留「多签账户」并改为独立卡片、右侧加 chevron；「扫码填地址」这套
在多签两页已重复两份的写法抽成全仓唯一共用组件。

所属模块：citizenapp/lib/transaction、citizenapp/lib/qr

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 定稿口径

1. **签名请求全部收归聊天页扫一扫**。零迁移工作量 —— `transaction_tab_page` 与
   `chat_tab` 调用的是同一个 `openScanDispatchFlow(paymentWallet:)`（都不传
   `signingAccount`），三条签名分支（广场动作 a=9 / 公民身份 a=2 / 占号换绑 a=10,11）
   在聊天页已完整可用。删交易页入口=零能力损失。
2. **交易页扫码专用填地址**，走 `QrScanMode.transfer`，与多签三页同款。
3. **聊天页扫一扫零改动**，继续走链下支付；其地址类码死路（k=3/k=5/裸地址必弹
   「该收款码不支持扫码支付」）本轮**知情保留**，待链下支付上线一并处理。
4. **k=4 收款码零改动**，待链下支付上线。
5. 右箭头按用户字面：`Icons.chevron_right, size: 22`，不设颜色（照抄链上交易-失败
   右侧那颗）。与邻近多签转账入口卡片（20/textTertiary）的差异为已知项。
6. 提示文案**必须短**。

## 账户选择口径（澄清，本轮不做）

| 分支 | 码里 `u` | 账户来源 | 要不要选 |
|---|---|---|---|
| 广场动作 a=9 | 带 | 按 `u` 找本机账户 | **不能选**（发起方已指定，选错=签错账户） |
| 公民身份 a=2 | 带 | 按 `u` 找本机账户 | **不能选** |
| 占号/换绑 a=10/11 | 留空 | `_pickBindingAccount` 底部选择器 | 已实现 |

用户设想的「单账户直接签、多账户扫完选」只对第三类成立，且选择器已在。**唯一剩余
缺口**：只有一个账户时仍会弹只有一项的选择器，可优化为直接用。列为后续项，本轮不做。

## 改动清单

| # | 文件 | 改动 |
|---|---|---|
| 1 | `lib/qr/widgets/address_scan_button.dart` | **新增** 全仓唯一「扫码填地址」组件 |
| 2 | `lib/transaction/personal-manage/personal_account_entry.dart` | **新增** 多签账户独立卡片（`onTap` 可注入，可单测） |
| 3 | `lib/transaction/multisig-transfer/multisig_transfer_page.dart` | 接组件，删 `_scanToAddress` |
| 4 | `lib/transaction/multisig-transfer/safety_fund_transfer_page.dart` | 同上 |
| 5 | `lib/transaction/transaction_tab_page.dart` | 删扫一扫入口与双层 Group/Tile，改用卡片 |
| 6 | `lib/transaction/onchain-transaction/onchain_payment_page.dart` | decoration 加可选 `suffixIcon`；收款地址框接组件；typedef 删 `currentWallet` 参数 |
| 7 | `lib/qr/pages/qr_scan_page.dart` | `transfer` 模式扫到 signRequest 给短提示，不再落「无法识别」 |

## 残留清理

- `_TransactionEntryGroup` / `_TransactionEntryTile` / `_dividerHeight` /
  `ValueKey('transaction-entry-divider')`
- `transaction_tab_page` 的 `_openScan` 与 `scan_dispatch_flow` / `flutter_svg` import
- 多签两页各自的 `_scanToAddress()` 与随之失效的 import
- `OnchainPaymentExtraEntriesBuilder` 的 `currentWallet` 参数（唯一实现方不再需要）

## 主要风险

`suffixIcon` 撑高收款地址框：现装饰 `isDense: true` + `contentPadding vertical 14`
（内容高约 48），`IconButton` 默认最小尺寸也是 48，很可能正好不撑高。策略：先按最简
写法接入（与多签两页渲染逐字节一致），**实测截图比对地址框与金额框等高**；只有确认
撑高才给组件加 `dense` 开关，且多签两页不传、保持零视觉回归。不为假想问题预先加参数。

## 输出物

代码、中文注释、测试、文档更新、残留清理。

## 验收标准

- `flutter analyze` 无新增问题
- `flutter test test/qr test/transaction` 全绿
- 地址框与金额框等高（实测）
- 文档已更新，残留已清理

## 执行结果（2026-08-07 完成）

### 代码

| 文件 | 改动 |
|---|---|
| `lib/qr/widgets/address_scan_button.dart` | 新增。全仓唯一「扫码填地址」组件：固定 `QrScanMode.transfer`、只回传 SS58、空结果不回调。回调而非代写 controller —— 代写会吞掉调用方的 `setState` |
| `lib/transaction/personal-manage/personal_account_entry.dart` | 新增 `PersonalAccountEntryCard`。左图标 + 标题 + 右 `chevron_right(22, 无色)`，整卡点击区，`onTap` 可注入 |
| `lib/transaction/transaction_tab_page.dart` | 149 行 → 22 行。删扫一扫入口与 Group/Tile 双层结构 |
| `lib/transaction/onchain-transaction/onchain_payment_page.dart` | `_transactionFieldDecoration` 提为类外 `@visibleForTesting transactionFieldDecoration`（不依赖实例状态，且要被布局测试调用）+ 可选 `suffixIcon`；收款地址框接组件；typedef 删 `currentWallet` 参数 |
| `lib/transaction/multisig-transfer/{multisig_transfer,safety_fund_transfer}_page.dart` | 接组件，各删一份 `_scanToAddress()` 与失效 import |
| `lib/transaction/personal-manage/personal_account_close_page.dart` | 接组件（框外 IconButton → 框内 suffixIcon），删 `Row`/`Expanded`/`SizedBox` 与失效 import |
| `lib/qr/pages/qr_scan_page.dart` | `transfer` 模式扫到 signRequest → 「这是签名请求 / 此处只扫收款地址。请到「聊天 → 扫一扫」。」 |

### 关键取舍

- **签名请求零迁移**：交易页与聊天页此前调的就是同一个
  `openScanDispatchFlow(paymentWallet:)`（都不传 `signingAccount`），三条签名分支在
  聊天页早已可用。删交易页入口=零能力损失，不需要搬任何代码。
- **未加 `dense` 参数**：方案里预留的「若 suffixIcon 撑高地址框就加尺寸开关」经实测
  不需要 —— 内容高与 suffix 盒子均为 48，天然等高。不为假想问题加参数。

### 测试（此前该链路零覆盖）

| 文件 | 覆盖 |
|---|---|
| `test/qr/address_scan_button_test.dart`（新，4 例） | tooltip；进 `transfer` 模式与标题；收款码结果**只取地址、丢弃金额/备注/清算行**；取消不回调 |
| `test/qr/qr_scan_page_sign_hint_test.dart`（新，1 例） | 扫到签名请求出指路提示、不与「无法识别」并存、不带结果返回 |
| `test/transaction/personal_account_entry_test.dart`（新，2 例） | 标题与 chevron（22/无色）；点标题与点箭头都命中同一点击区 |
| `test/transaction/transaction_field_decoration_test.dart`（新，2 例） | **地址框与金额框等高**；不传 `suffixIcon` 不渲染尾部图标 |
| `test/ui/transaction_tab_page_test.dart`（改） | 断言由「扫一扫 + 竖线 + 双入口」改为「只剩多签卡片 + chevron + 地址框内扫码 tooltip」 |

等高测试做过**反证**：把 `contentPadding` 的 vertical 由 14 改成 4，测试如实失败
（`Expected: <32.0> Actual: <48.0>`），确认不是空测试。

### 文档

- `memory/05-modules/citizenapp/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`：
  重写入口与扫码边界，新增装饰共用与等高约束条款
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`：交易页入口结构（两处）
- `memory/05-modules/citizenapp/signer/SIGNER_TECHNICAL.md`：签名入口改「聊天 → 扫一扫」
- `memory/01-architecture/qr/qr-action-registry.md`：a=9 签名端入口改聊天 tab
- `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`：新增签名请求收归条目

### 残留清理

`_TransactionEntryGroup` / `_TransactionEntryTile` / `_dividerHeight` /
`transaction-entry-divider` key / 两份 `_scanToAddress()` / typedef 的 `currentWallet`
参数 / 四处失效 import；另订正两处过时注释（`offchain_scan_flow.dart`、
`scan_dispatch_flow.dart` 仍写「交易 tab 扫一扫」）。全仓 grep 零命中。

### 验证

- `flutter analyze lib test` → No issues found
- `flutter test --concurrency=1`（全量，CI 同款命令）→ **1124 通过 / 48 跳过 / 0 失败**
- `flutter test --concurrency=1 test/qr test/transaction test/ui` → 83 通过

**踩坑记录**：本轮先用裸 `flutter test` 跑全量，进程 0% CPU、无子进程、无网络连接干挂
55 分钟一个用例都没跑。原因是本仓 Isar 用 `isar_community` 分支，并发 isolate 会死锁；
CI 用的一直是 `flutter test --concurrency=1`（`.github/workflows/citizenapp-ci.yml:87`）。
**本地全量验证必须用 CI 同款命令**，否则不是慢，是挂死。
- 无宿主可运行目标（无 macOS desktop 平台、无已启动模拟器），故未做真机截图；
  等高这条唯一的视觉风险已转成上表的自动化断言，比一次性目测更强。

## 后续项（本轮不做）

1. **占号/换绑单账户免选**：`_pickBindingAccount` 在只有一个热账户时仍会弹只有一项的
   选择器，可优化为直接用。广场动作与公民身份两类**不需要**选择器 —— 码里 `u` 已指定
   账户，让用户选反而会签错账户。
2. **聊天页扫一扫地址类死路**：扫用户码/账户码/裸地址仍被「未绑定清算行」挡回，
   待链下支付上线一并处理（用户知情保留）。
（原第 3 项「个人多签关闭页扫码未并入」已于本轮补做，见下。）

## 追加：第 4 处扫码填地址并入组件（2026-08-07，用户指示「都统一」）

`personal_account_close_page.dart` 的「扫码填入受益人地址」原是输入框**外**的独立
IconButton（`Row` + `Expanded(TextField)` + `SizedBox(8)` + 22×22 IconButton），与其余
三处「框内 suffixIcon」形态不一致。现改为 `TextField.decoration.suffixIcon:
AddressScanButton`，删掉整层 `Row`/`Expanded`/`SizedBox` 与自带的扫码闭包，
并清掉失效的 `flutter_svg`、`qr_scan_page` import。

至此**全仓「扫码填地址」四处形态与实现完全统一**：

| # | 页面 | 输入框 |
|---|---|---|
| 1 | 链上支付（交易 Tab 与独立页共用） | 收款地址 |
| 2 | 多签转账 | 受益人地址 |
| 3 | 安全基金转账 | 受益人地址 |
| 4 | 个人多签账户关闭 | 受益人地址 |

复核判据：`grep -rn "AddressScanButton\|QrScanMode.transfer" lib/` 除组件与扫码页自身外
只剩这四处 `suffixIcon:`，无任何裸 `QrScanMode.transfer` 调用。

**已知差异（未强行拉平）**：四处输入框各自的 `InputDecoration` 边框、圆角、
`contentPadding` 本就不同（交易页 `isDense` + radiusSm，多签页 radius 8/10），组件只统一
**扫码行为与图标**，不接管各页装饰 —— 强行统一装饰会改动三个已上线页面的视觉。
