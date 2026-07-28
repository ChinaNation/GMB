# CitizenApp 身份门禁文案统改 + 账户详情按 account_id 找回钱包功能

## 任务状态

- 状态：进行中
- 创建日期：2026-07-28
- 模块：Mobile Agent（`citizenapp`），不涉及 `citizenchain/runtime`、不涉及扫码签名协议

## 任务需求（用户 6 项）

1. 身份门禁标题 `注册身份后使用{X}` → 聊天=`注册后开始聊天`、其余=`注册后使用{X}`，并删掉标题下那行小字。
2. 警示条标题 `需要已注册身份` → `需要注册身份`；正文改为
   `注册后可使用广场、聊天、通讯录等功能，未注册用户只能使用钱包、交易等功能。`
3. 门禁按钮 `去注册身份` → `注册`。需求 1/2/3 同样覆盖 会员｜订阅 / 创作者 / 通讯录页
   （五页共用同一个 `IdentityRegistrationGate`，一处改动即全覆盖）。
4. 「我的 → 身份」页访客卡标题 `注册身份·访客` → `身份·访客`。
5. 「我的钱包」右上角「＋」弹三项：`添加下一个账户` / `添加指定账户` / `导入冷钱包`
   （添加两项在上、导入冷钱包在下）；把「账户详情」右上角误放的「添加账户」删掉，
   拆成钱包列表「＋」里的两个添加入口。对齐 CitizenWallet 已实现做法。
6. 找回单钱包多账户改造中从热钱包详情丢失的 充值 / 提现 / 零钱包 / 交易记录 / 清算行，
   装回「账户详情」；一律按 `account_id`（Substrate 官方口径，不缩写）键控与签名。

## 已检查依据

- `citizenapp/lib/my/myid/widgets/identity_registration_gate.dart`：五页共用门禁，文案单源。
- `citizenapp/lib/my/creator/widgets/creator_gate_view.dart`：**平台会员**门禁，另一套文案，不在本次范围。
- `citizenapp/lib/my/myid/myid_page.dart:374`：访客卡标题 `注册身份·访客`。
- `citizenapp/lib/wallet/pages/wallet_page.dart`：`WalletEntryChooserSheet` 现仅「导入冷钱包」；
  热钱包账户走 `AccountDetailPage`，冷钱包走 `WalletDetailPage`（含完整功能）。
- `citizenapp/lib/wallet/pages/account_detail_page.dart`：精简版（SS58+私钥+删除），右上角误置「添加账户」。
- `citizenapp/lib/wallet/widgets/wallet_action_card.dart`：充值/提现/零钱包，现按 `WalletProfile.walletIndex` 键控。
- `citizenapp/lib/transaction/offchain-transaction/`：提现/零钱包/存入/绑定/清算设置均按 `walletIndex` 键控+签名。
- `citizenapp/lib/wallet/core/wallet_manager.dart`：`signForAccountId(accountId,payload)` 已提供按账户签名。
- `citizenapp/lib/transaction/offchain-transaction/services/clearing_bank_prefs.dart`：缓存键为 `walletIndex`。
- CitizenWallet `citizenwallet/lib/ui/wallet_detail_page.dart`：三项添加 UX 参考。

## 落地边界

- 需求 1/2/3：只改 `identity_registration_gate.dart` 文案，不动 `CreatorGateView`。
- 需求 5：「＋」三项菜单；`add_account_sheet` 支持固定初始模式；删账户详情右上角添加。
- 需求 6：`ClearingBankPrefs` 重键 `walletIndex`→`account_id`；`wallet_action_card` 与
  `withdraw/petty_wallet/deposit/bind_clearing_bank/clearing_bank_settings` 六处改吃
  `accountId+ss58Address`，签名统一 `signForAccountId`；`account_detail_page` 装回五项功能；
  冷钱包 `WalletDetailPage` 传 `wallet.accountId/ss58`，不破冷路径、不扩范围。
- 同步测试：`clearing_bank_prefs_test` / `wallet_action_card_test` / `wallet_multi_account_ui_test`
  / `identity_registration_gate_test` / `clearing_bank_settings_page_test`。
- 更新文档注释、清残留；不触碰 runtime；未获允许不 push。

## 预计修改目录

- `citizenapp/lib/my/myid/widgets/`：门禁文案（需求 1/2/3）。
- `citizenapp/lib/my/myid/`：访客卡标题（需求 4）。
- `citizenapp/lib/wallet/pages/`：「＋」三项菜单、账户详情找回功能、冷钱包详情传参（需求 5/6）。
- `citizenapp/lib/wallet/widgets/`：动作卡按账户键控（需求 6）。
- `citizenapp/lib/transaction/offchain-transaction/`：清算全链路按 `account_id` 键控与签名（需求 6）。
- `citizenapp/test/`：同步 5 个既有测试，不新增测试文件。
- `memory/08-tasks/open/`：本任务卡。

## 当前进度

- [x] 需求 1/2/3：门禁文案统改（`identity_registration_gate.dart` 单源覆盖五页；聊天=注册后开始聊天、其余=注册后使用{X}；删小字；需要注册身份 + 新警示正文；按钮=注册；`_GateView.body` 改可空）
- [x] 需求 4：访客卡标题 `注册身份·访客`→`身份·访客`（`myid_page.dart`）
- [x] 需求 6：`ClearingBankPrefs` 重键 `walletIndex`→`account_id`
- [x] 需求 6：`wallet_action_card` + `withdraw`/`petty_wallet`/`deposit`/`bind_clearing_bank`/`clearing_bank_settings` 六处改吃 `accountId+ss58Address`，签名统一 `signForAccountId`；冷钱包 `WalletDetailPage` 传 `wallet.accountId/ss58`
- [x] 需求 5+6：`account_detail_page.dart` 删右上角「添加账户」、装回 充值/提现/零钱包 + 清算行 + 交易记录（按 account_id）；去 ChainTxMonitor 实时监听改初始化加载 + 下拉刷新
- [x] 需求 5：`wallet_page.dart` 列表「＋」三项菜单（添加下一个账户/添加指定账户/导入冷钱包，有热钱包才显前两项）+ `add_account_sheet.dart` 固定初始模式（去面板内切换器）
- [x] 同步 5 个测试（clearing_bank_prefs / clearing_bank_settings_page / identity_registration_gate / wallet_action_card / wallet_multi_account_ui）
- [x] dart analyze 干净（lib 全域 + 5 测试）
- [x] 定向测试运行通过（5 文件全绿；wallet_multi_account_ui 14/14 All tests passed）
- [x] 清残留（全仓无旧构造/旧键/旧文案；wallet_gate 两处注释为动作描述非 UI 文案，保留）
- [ ] 真机验收（Pixel/模拟器）——未做,待用户确认是否需要

## 测试修复记录（wallet_multi_account_ui_test）

- AccountDetailPage 内容变高(加了充值/提现/零钱包/清算行/交易记录),底部删除按钮落在
  默认测试视口(800×600)+缓存范围外 → 惰性 ListView 未构建 → `findsOneWidget` 落空。
  修法:两个账户详情测试放大测试视口 `tester.view.physicalSize = 1200×3200` + `addTearDown(reset)`。
- 账户详情测试需 Isar(读交易记录)。初版在 AccountDetailPage 组与 AddAccountSheet 组
  **各调一次** `useIsolatedIsar()` → 两次开 IsarCore,第二个 group 的 setUpAll 挂死(12 分钟超时)。
  修法:按 `isar_test_env.dart` 注释,`useIsolatedIsar()` 只在 `main()` 顶部调一次(文件级),
  两个 group 内的调用全删。
