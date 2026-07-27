# citizenwallet 4 项 UI 改进(每钱包扫码 + 添加账户页 + 返回键 + 账户区上移)

状态:done(2026-07-27 全部落地并通过)
所属模块:Mobile(citizenwallet 冷钱包)
工作根:`/Users/rhett/GMB/citizenwallet/`(只改主检出)

## 需求(用户已拍板)
1. **每钱包扫码**:app bar 全局扫码图标移到每张钱包卡片 3 点左侧,`ScanPage(wallet)` 只扫本钱包账户,跨钱包签名请求拒绝。**反转 HD S1.3「全局扫码」。**
2. **添加账户专用页**:镜像导入页,用共享 `Bip39InputField`(BIP39 逐词候选自动补全)替代 model B 1.1 的朴素弹窗。
3. **返回键统一**:citizenwallet ThemeData 加 `actionIconTheme(backButtonIconBuilder → Icons.chevron_left)`,一处覆盖全部 AppBar 自动返回键(对齐 citizenapp)。
4. **账户区上移**:`_buildAccountsSection` 标题行 padding 顶部 14 → 5。

## 逐文件
- `lib/ui/home_page.dart`:删 app bar 扫码 IconButton + `_openScan`;`_buildWalletCard` 名称与 3 点间插扫码 IconButton(scan-line.svg)→ `_openWalletScan(wallet)` → `ScanPage(wallet)`。
- `lib/ui/scan_page.dart`:`ScanPage({required Wallet wallet})`;`_handleCode` 加 `account.masterId==wallet.masterId` 校验,跨钱包拒绝;页头文案。
- `lib/ui/add_account_page.dart`(新):镜像 import 页,`Bip39InputField` + `addAccount(masterId, mnemonic)` + `SensitivePageMixin` 防截屏。
- `lib/ui/wallet_detail_page.dart`:`_addAccount`→push `AddAccountPage`;删 `_promptMnemonic`;账户区 padding 14→5。
- `lib/ui/app_theme.dart`:ThemeData 加 `actionIconTheme`(chevron_left)。
- `test/ui/add_account_page_test.dart`(新):渲染 Bip39 输入 + 按钮在场(IO-free)。

## 验收
- `dart analyze` 0 + `flutter test` 全绿;残留清扫(`_openScan`/全局 ScanPage()/`_promptMnemonic` 全 0)。

## 进度(done 2026-07-27)
- 需求1:`home_page` 删 app bar 扫码 + `_openScan`;卡片 3 点左侧加 scan-line.svg 扫码 IconButton → `_openWalletScan` → `ScanPage(wallet)`;`scan_page` 加 `wallet` 参 + `account.masterId==wallet.masterId` 校验(跨钱包拒绝)+ 页头文案。**反转 HD S1.3 全局扫码。**
- 需求2:新增 `add_account_page.dart`(镜像导入页,`Bip39InputField` + `SensitivePageMixin` 防截屏);`wallet_detail._addAccount` 改 push 该页,删 `_promptMnemonic`/`_addingAccount`,按钮简化。
- 需求3:`app_theme` ThemeData 加 `actionIconTheme(backButtonIconBuilder→chevron_left)`,一处统一全部返回键(对齐 citizenapp)。
- 需求4:`_buildAccountsSection` 标题行顶部 padding 14→5。
- 新增 `test/ui/add_account_page_test.dart`;终验 `dart analyze` 0 + `flutter test` **210 passed**;残留(`_openScan`/`ScanPage()`/`_promptMnemonic`/`_addingAccount`)复扫全 0;改动只在主检出 citizenwallet。
