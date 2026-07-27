# citizenwallet 基线七项修复

状态:done(2026-07-26 全部落地并通过验收)
所属模块:Mobile(citizenwallet 冷钱包)
关联:`memory/08-tasks/open/20260726-citizenapp-citizenwallet-hd-wallet-derivation.md`(HD 卡「S1 复查 + S1-FIX」节列为「未纳入(既有基线,非 HD 引入)」的七项,现单独处理)

## 背景

HD 改造 Step 1 复查由两独立评审(security + flutter)发现七项**既有基线问题**——均**非 HD 引入**,按 `no-scope-expansion` 未纳入 HD 线。本卡专门修复,不触碰 HD 语义(两级 Wallet/Account 模型、`//index` 派生、按 accountId 签名、全局扫码、派生金标 `test/wallet/derivation_golden_test.dart`)。

工作根:`/Users/rhett/GMB/citizenwallet/`(仓库顶层 `/Users/rhett/GMB`,子目录)。一切改动只落主检出(`user-evaluates-in-main-checkout`)。

## 用户拍板(2026-07-26)

- ④ 种子加固:**加密 seed_hex + 把 `MnemonicCipher` 重命名为 `SecretCipher`**(通用名,同时加密助记词+种子)。
- ② 分组页死码:**直接删除 `_deleteGroup`**(confirmDismiss 内联为唯一删除入口)。
- ⑦ 寻址混用:**统一到 masterId**(renameWallet/reorderWallets + 2 处 home_page 调用方)。

## 逐项(回码核验后的改法)

1. **[数据污染] 分组名未禁逗号** — `lib/ui/group_management_page.dart`。逗号是 `WalletEntity.groupNames` 存储分隔符(`wallet_isar.dart`),名字含 `,` 会被 `split(',')` 读成多组 + `groupNamesContains` 子串误伤。抽纯函数 `groupNameFormatError`(超长/含逗号),`_addGroup`/`_renameGroup` 复用;补单测 `test/ui/group_name_validation_test.dart`。
2. **[死代码] `_deleteGroup`** — 删除该方法(带 `// ignore: unused_element`),消与 confirmDismiss 的逐行重复。
3. **[潜在崩溃] onDetect 未判空** — `lib/ui/scan_page.dart` 实时流 `capture.barcodes.first` 空列表抛 StateError;加 `if (capture.barcodes.isEmpty) return;`(对齐相册路径)。
4. **[加固] seed_hex 明文** — `lib/wallet/wallet_manager.dart` `_writeMasterSeed`/`_readMasterSeedRaw` 改走 `SecretCipher` AES-256-GCM(与助记词同 AEK);读侧解密失败(FormatException)→ `WalletAuthException('钱包密钥数据异常，请重新导入钱包')`。类改名 `MnemonicCipher`→`SecretCipher`(文件 `mnemonic_cipher.dart`→`secret_cipher.dart`)。AEK 丢失则 seed+助记词皆不可解 → 靠已备份助记词重导(开发期零用户,可接受)。
5. **[加固] SecureStorage 未设选项** — 新建单源 `lib/security/secure_storage.dart` 暴露 `const appSecureStorage`(`AndroidOptions(encryptedSharedPreferences: true)` + `IOSOptions(accessibility: first_unlock_this_device)`);5 处 `FlutterSecureStorage()` 统一引用(main/settings_page/app_lock_service/wallet_manager/secret_cipher)。`encryptedSharedPreferences` 需 Android minSdk ≥ 23 → `android/app/build.gradle.kts` `minSdk = maxOf(23, flutter.minSdkVersion)`。
6. **[UI 矛盾] 绿 banner + 拒绝行** — `lib/ui/offline_sign_page.dart` `_buildTransactionDetails`:`normal + decoded==null`(runtime 升级哈希签,唯一来源)曾落 else 显示「拒绝签名」。detailRows 改三分支(reject / normal+decoded / normal+hash-only)+ hash-only banner 文案改准;补 widget 测试 `test/ui/offline_sign_page_test.dart`。
7. **[架构一致] walletIndex/masterId 寻址** — `renameWallet`/`reorderWallets` 参数 walletIndex→masterId(masterId 是稳定主键,walletIndex 复用槽位会漂);2 处 home_page 调用改传 `wallet.masterId`/`w.masterId`。

## 验收
- `dart analyze`(citizenwallet)0;`flutter test` 全绿(基线 209 passed,新增测试后 ≥209)。
- 死规则:无残留(删 `_deleteGroup` 不留 ignore、重命名不留旧符号)、无兼容层、只改主检出、回复中文。

## 进度(2026-07-26 完成)

逐文件落地:
- 新增 `lib/security/secure_storage.dart`:`const appSecureStorage`(Android `encryptedSharedPreferences:true` + iOS `first_unlock_this_device`);5 处裸 `FlutterSecureStorage()` 全并入(main/settings_page/app_lock_service/wallet_manager/secret_cipher)。⑤
- `lib/wallet/mnemonic_cipher.dart` → `lib/wallet/secret_cipher.dart`,`MnemonicCipher`→`SecretCipher`(通用化,同时加密助记词+种子);import 全同步(main/app_lock_service/wallet_manager)。④
- `lib/wallet/wallet_manager.dart`:`_writeMasterSeed`/`_readMasterSeedRaw` 走 `SecretCipher` AES-GCM,读侧解密失败→`WalletAuthException`;`renameWallet`/`reorderWallets` 参数 `int walletIndex`→`String masterId`。④⑦
- `lib/ui/home_page.dart`:2 处调用改传 `wallet.masterId`/`w.masterId`。⑦
- `lib/ui/group_management_page.dart`:抽 `groupNameFormatError`(超长/禁逗号)纯函数,`_addGroup`/`_renameGroup` 复用;删死方法 `_deleteGroup`(去 `// ignore`);`maxLength` 用单源 `groupNameMaxRunes`。①②
- `lib/ui/scan_page.dart`:onDetect 加 `if (capture.barcodes.isEmpty) return;`。③
- `lib/ui/offline_sign_page.dart`:detailRows 三分支(reject/normal+decoded/normal+hash-only)+ hash-only banner 文案改准。⑥
- `android/app/build.gradle.kts`:`minSdk = maxOf(23, flutter.minSdkVersion)`(EncryptedSharedPreferences 前置)。⑤

测试:
- 新增 `test/ui/group_name_validation_test.dart`(3)、`test/ui/offline_sign_page_test.dart`(1,ScreenshotGuard 通道 mock + runtime hash-only)。
- `test/wallet/mnemonic_cipher_test.dart` → `secret_cipher_test.dart`(改名 + 补主种子加密用例)。
- `test/wallet/wallet_manager_test.dart`:renameWallet/reorderWallets 改 masterId + 新增「seed 落库为密文非明文 hex」。

终验:`dart analyze` **0** + `flutter test` **215 passed**(基线 209 + 6)。残留复扫:`MnemonicCipher`/`mnemonic_cipher`/裸 `FlutterSecureStorage()` 全 0。

HD 卡「未纳入(既有基线)」行已回填指向本卡。
