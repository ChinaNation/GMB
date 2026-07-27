# citizenwallet 账户详情页 UI(删派生路径 + 账户名可改)

状态:done(需求 1、2 落地并通过验收 2026-07-26;需求 3 暂缓)
所属模块:Mobile(citizenwallet 冷钱包)
工作根:`/Users/rhett/GMB/citizenwallet/`(只改主检出)

## 需求
1. 账户详情**不再显示派生路径**:删头部副标题里的"· 派生路径 X"(留钱包名)、删公开信息卡的 `派生路径` 行(含分隔线+复制图标);连带删除删账户确认弹窗里的"按路径 //N"措辞(honor"账户详情中不显示派生路径")。
2. 头部账户名**点击可改名**:新增 `WalletManager.renameAccount(accountId, name)`;账户名旁加编辑图标,点击弹重命名框;改后本页刷新 + 返回时父页 `_load` 回刷。
3. (暂缓)SS58 下方加"私钥"(点击验证后显示)。**本轮不做**;概念已澄清:账户 N≥1 私钥独立可单显,账户 0=钱包根特殊。

## 逐文件(需求 1、2)
- `lib/wallet/wallet_manager.dart`:新增 `maxAccountNameLength` + `renameAccount`(按 accountId 改 accountName,写事务;不动密钥)。
- `lib/ui/account_detail_page.dart`:头部名→可点击+编辑图标+用状态 `_accountName`;副标题去派生路径;删公开卡 `派生路径` 行;删账户弹窗去路径措辞;`_renameAccount` 方法;QR/删账户文案改用 `_accountName`。
- `test/ui/account_detail_page_test.dart`:去"派生路径/`//1`"断言,加"派生路径 findsNothing + 编辑图标在场";保留 C-1 的"不展私钥"断言(需求 3 未做)。

## 验收
- `dart analyze` 0 + `flutter test` 全绿;账户详情全页 grep 无 `派生路径` 文案显示。

## 进度(2026-07-26 完成 需求 1、2)

- `wallet_manager.dart`:新增 `maxAccountNameLength=5` + `renameAccount(accountId, name)`(改 accountName 写事务,不动密钥)。
- `account_detail_page.dart`:
  - 需求1:头部副标题去"· 派生路径 X"(留钱包名);删公开信息卡 `派生路径` 行+分隔线+复制图标;删账户确认弹窗去"按路径 //N"措辞;类注释同步。
  - 需求2:头部账户名改为可点击(旁加 `edit_outlined` 图标)→ 弹重命名框 → `renameAccount` → 本页 `_accountName` 状态刷新;QR/删账户文案改用 `_accountName`。
- `account_detail_page_test.dart`:去"派生路径/`//1`"断言 → 改为 `findsNothing`;加"账户名在场 + 编辑图标在场";保留 C-1"不展私钥"三断言。

终验:`dart analyze` **0** + `flutter test` **209 passed**;`account_detail_page.dart` grep `派生路径`/`derivationPath` = 0。

需求 3(SS58 下方加私钥)本轮未做。概念已澄清:账户 N≥1 私钥独立、单向不可逆,可只暴露单账户;账户 0=钱包根,暴露=整钱包。待用户定表示形式(原始私钥 hex / 密钥URI)+ 核实 polkadart_keyring 能否取原始 secret 后再落地。

**更新(2026-07-27)**:已授权 model B(全 `//index`,账户0 改 `//0` 无 bare 根)——账户0 也成叶子,每账户私钥独立隔离,展示单账户私钥安全。req3 移交 `20260727-citizenwallet-modelb-index-derivation.md` **Step 1.2** 落地(展示该账户 child mini-secret `0x<64hex>`)。
