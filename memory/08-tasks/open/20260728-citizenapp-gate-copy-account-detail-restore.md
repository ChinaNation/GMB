# CitizenApp 身份门禁文案统改 + 账户详情按 account_id 找回钱包功能

## 任务状态

- 状态：已完成
- 创建日期：2026-07-28
- 模块：Mobile Agent（`citizenapp`），不涉及 `citizenchain/runtime`、不涉及扫码签名协议

## 任务需求（用户 6 项 + 账户页 UI 复核）

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
7. 按用户确认的“方案 1 修正版”重做「我的钱包」账户卡片与「账户详情」：
   - 账户卡片删右箭头，右侧改为「扫码签名 + 竖三点」；整卡和菜单“账户详情”进入同一页面。
   - 账户卡片竖三点固定“重命名 / 账户详情 / 删除钱包或删除账户”；账户0显示删除钱包，
     其余账户显示删除账户。
   - 账户详情 AppBar 最右侧放竖三点，菜单仅“清算行 / 查看私钥”；正文删除清算行独立卡、
     私钥卡和删除按钮。
   - 顶部账户卡完整显示 SS58 地址与复制按钮；删除派生路径小字；账户名称右侧增加用户二维码
     图标并复用全 App 唯一 `UserQrPage`。
   - 充值下显示该账户链上余额；零钱包下继续显示该账户清算行余额；交易记录继续复用现有
     `LocalTxStore` / `LocalTxRecordTile`。
8. 账户卡扫码必须锁定当前 `account_id`：只接受签名请求，使用
   `WalletManager.signForAccountId`，请求指定账户不一致时必须在读取私钥前拒绝；不得修改
   `qr_scan_page.dart` 的蓝色对准框、提示小字、相册、手电筒或任何视觉参数。
9. 删除规则：
   - 账户0：二次危险提示后，对一次性本地挑战签名并本地验签，验签成功才能删除整只钱包。
   - 非0账户：菜单点击后直接删除，不弹确认、不签名。
   - 钱包/账户删除必须清理对应账户行、硬件金库 child、交易记录、同步游标、清算行缓存、
     通讯录密钥和本机相关缓存；任一清理失败不得显示删除成功。
10. 2026-07-28 真机复核补充：
    - 账户二维码必须位于顶部账户卡片独立右上角，不能紧挨账户名称。
    - 修复查看私钥在 `BiometricPrompt` 前直接失败；单账户删除不得误删同钱包共享 KEK。
    - 查看私钥只能读取设备安全存储中的账户私钥并完成生物识别，禁止要求输入助记词；
      KEK 缺失时只能明确报告设备私钥不可用。
11. 2026-07-28 账户卡布局复核补充：
    - 用户二维码必须贴到顶部账户卡真正的右上角，不能被卡片内容 padding 二次向内挤。
    - SS58 地址改为独立第二行，只在右侧保留复制按钮；复制按钮靠齐内容右边界，不再让
      上一行二维码的预留位压缩地址显示空间。
12. 2026-07-28 “我的”页面个人服务顺序调整：
    - 仅把“会员｜订阅 / 创作者 / 通讯录”调整为“创作者 / 通讯录 / 会员｜订阅”。
    - 三个入口原有图标、文案、点击页面和身份门禁全部保持不变。

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
- UI 复核新增边界：
  - 账户名称重命名只写 `AccountEntity.accountName`，不联动钱包名或链上昵称。
  - 账户二维码统一复用 `openAccountQrPage(accountId, paymentDisplayName)`；2026-07-28
    起身份账户生成严格 `k=3`，其它账户生成五分钟 `k=4`，不新增二维码协议。
  - 账户扫码签名改造现有三个签名服务为账户维度，不保留误落账户0的双轨分支。
  - `qr_scan_page.dart` 基线 SHA-256：
    `8160c55fab80256615d3132a3ae7e6e221ae434124be24788025718f196d7303`。
- 更新文档注释、清残留；不触碰 runtime；未获允许不 push。

## 预计修改目录

- `citizenapp/lib/my/myid/widgets/`：门禁文案（需求 1/2/3）。
- `citizenapp/lib/my/myid/`：访客卡标题（需求 4）。
- `citizenapp/lib/my/user/`：“我的”页面个人服务三项仅调整展示顺序。
- `citizenapp/lib/wallet/pages/`：「＋」三项菜单、账户详情找回功能、冷钱包详情传参（需求 5/6）。
- `citizenapp/lib/wallet/widgets/`：动作卡按账户键控（需求 6）。
- `citizenapp/lib/wallet/core/`：账户重命名、账户0签名删除门禁、钱包/账户彻底清理。
- `citizenapp/lib/qr/`：只改扫码签名分派；`qr_scan_page.dart` 零修改。
- `citizenapp/lib/signer/`：公民身份、广场动作、占号/换绑统一按 `account_id` 签名。
- `citizenapp/lib/transaction/offchain-transaction/`：清算全链路按 `account_id` 键控与签名（需求 6）。
- `citizenapp/test/`：更新既有钱包 UI、钱包生命周期、签名服务与门禁测试，不新增测试文件。
- `memory/05-modules/citizenapp/wallet/`：更新账户卡、账户详情、删除与扫码签名当前实现说明。
- `memory/05-modules/citizenapp/signer/`：更新指定 `account_id` 扫码签名边界。
- `memory/08-tasks/open/`：更新本任务卡、验收记录与残留清理结论。

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
- [x] 账户卡片 UI：扫码图标 + 竖三点 + 动态三项菜单 + 账户重命名。
- [x] 账户详情 UI：AppBar 菜单、完整 SS58、账户二维码、双余额、交易记录，删除旧正文入口。
- [x] 指定账户扫码签名：三个签名服务改按 `account_id`，扫码页 UI 零改动。
- [x] 账户0签名后删除整钱包；非0账户直接删除；全部关联本机数据清理并复核。
- [x] 新增范围静态检查与回归：`dart analyze` 0 问题；`flutter test -r compact`
  全量 910 项通过、5 项按既有条件跳过；`git diff --check` 通过。
- [x] Pixel 8a（Android 16）真机验收：真实启动、smoldot 进入 regular；实看账户卡片、
  三项菜单、账户详情、用户二维码、双余额、交易记录、详情 AppBar 菜单与扫码签名页面。
- [x] 按已确认方案 1 设计图与 Pixel 8a 同屏比对：标题栏三点位置、渐变账户卡、
  账户名旁二维码、完整地址、三列操作区与交易记录卡层级一致；真机无交易时按真实数据
  显示“暂无交易记录”，未伪造设计图示例流水。
- [x] 扫码签名 UI 冻结复核：`qr_scan_page.dart` Git diff 为空，SHA-256 仍为
  `8160c55fab80256615d3132a3ae7e6e221ae434124be24788025718f196d7303`；真机蓝色对准框、
  提示小字、相册和手电筒均保持基线布局。
- [x] 文档、中文注释与残留清理：更新 Wallet/Signer 技术文档；旧访客卡测试注释、
  旧账户详情私钥/删除入口及账户0签名回退残留已清理。
- [x] 二次 UI 修正：账户二维码从账户名后方移至账户卡独立右上角；Widget 测试断言二维码
  与账户名之间保留明显横向间隔，Pixel 8a 实测语义边界为 `[880,363][985,468]`。
- [x] 私钥认证真因修复：Pixel 8a 原生日志确认共享 KEK
  已删除的旧 Keystore alias 缺失，旧代码在 `BiometricPrompt` 前把 `null` 强转为
  `PrivateKey`；现改为明确报告设备安全存储中的账户私钥不可用。
- [x] 共享 KEK 生命周期修复：`deleteAccountKey` 只删目标账户密文，
  `deleteWalletKey` 只在整钱包删除/回滚时调用；补充子账户删除后账户0仍可解锁、
  整钱包仅删一次 KEK 的测试。
- [x] 纠正错误恢复设计：彻底删除账户详情助记词输入弹窗、`restoreWalletKeys`、
  `WalletKeyRecoveryRequired`、原生 recovery 档与对应测试/文档；查看私钥恢复为
  “危险确认 → 设备安全存储 → 生物识别 → 显示私钥”唯一流程。
- [x] 账户卡最终布局：二维码贴到卡片真正右上角；地址独立第二行；复制按钮靠齐内容右侧。
  定向测试 18/18 通过、`dart analyze` 0 问题、`git diff --check` 通过；Pixel 8a 真机
  复核二维码边界 `[912,331][1017,436]`、复制按钮边界 `[880,489][985,594]`，
  两者分处首行右上角和第二行最右侧，完整 SS58 地址获得整行可用宽度。
- [x] “我的”页面个人服务调整为“创作者 / 通讯录 / 会员｜订阅”，三个入口仍调用原来的
  `_openCreator` / `_openContacts` / `_openMembership`；定向测试通过、`dart analyze`
  0 问题、`git diff --check` 通过。Pixel 8a 真机语义边界依次为
  `[45,1174][1035,1342]`、`[45,1344][1035,1512]`、`[45,1515][1035,1683]`。

## 最终验收记录

- `wallet_multi_account_ui_test.dart` 覆盖账户卡图标顺序、整卡与菜单详情入口、0/非0账户
  删除文案、账户详情完整 SS58/二维码/AppBar 菜单，以及正文无旧私钥卡和删除按钮。
- `wallet_multi_account_test.dart` 覆盖账户改名、账户0签名验签后删除、取消认证不删除，
  以及账户/钱包关联事实、交易、游标、通讯录缓存和安全存储清理。
- 三个 signer 测试覆盖目标 `account_id` 精确签名与错配拒绝；动作卡测试覆盖链上余额和
  清算行余额分离显示。
- 真机验收未执行破坏性删除，也未读取私钥；删除安全链路由隔离 Isar 与模拟硬件金库测试
  验收，避免删除用户 Pixel 8a 上的真实钱包。
- 设备私钥缺失/失效测试只断言 fail-closed 与零密钥重写，不构造助记词恢复路径。
