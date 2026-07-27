# citizenwallet 存储反转:冷钱包存种子+助记词 + 4 项复查修复

状态:done(2026-07-27 全部落地并通过)
所属模块:Mobile(citizenwallet 冷钱包)
工作根:`/Users/rhett/GMB/citizenwallet/`(只改主检出)

## 决策(反转 D1)
- **citizenwallet(冷)= 存种子 + 助记词**(每钱包一份,SecretCipher AES-GCM);citizenapp(热)反过来改无根(citizenapp 侧另一窗口处理)。
- 派生仍 model B 全 `//index`(账户0=`//0`),金标不变。签名/私钥展示从存储种子现场派生。
- 钱包详情第一卡加助记词区(reveal 模式,样式同账户私钥区)。

## 逐文件
- `wallet_secure_keys.dart`:恢复 `masterSeedHexV1`/`masterMnemonicV1`;删 `accountMiniSecretV1`。
- `wallet_manager.dart`:恢复 seed/助记词存取 + `getMasterMnemonic`;`_establishWallet` 存 seed+助记词;`addAccount(masterId)` 用存储种子(去 mnemonic 参);签名/`getAccountPrivateKey` 从存储种子派生;删每账户密钥存储;delete 清 seed/助记词。
- `wallet_detail_page.dart`:身份卡加助记词区(reveal);`_addAccount` 改回直接 addAccount + loading;删 add_account_page import。
- 删 `add_account_page.dart` + `add_account_page_test.dart`。
- `main.dart`(#1 H1):超时重锁 `popUntil(isFirst)` 清深层页再锁。
- `login_qr_handler.dart`(#3 M1):校验 env.id(复用 qr_signer 规则)。
- 注释(#4):scan_page/home_page 更新;secret_cipher 注释此时已正确。

## 测试(#2)
- wallet_manager_test 改回种子存储 + addAccount(masterId) + getMasterMnemonic;wallet_secure_keys_test 改回;删 add_account_page_test;新增 scan 纯谓词单测 + 私钥/助记词确认弹窗 widget 测。

## 验收
- `dart analyze` 0 + `flutter test` 全绿;残留清扫(add_account_page/accountMiniSecretV1/无根语义 0);更新记忆(D1 反转)。

## 进度(done 2026-07-27)
- 存储反转:`wallet_secure_keys` 恢复 `masterSeedHexV1`/`masterMnemonicV1`(删 accountMiniSecretV1);`wallet_manager` 恢复 seed/助记词存取 + `getMasterMnemonic`,`addAccount(masterId)` 读存储种子(去 mnemonic 参),签名/`getAccountPrivateKey` 从种子现场派生,delete 清 seed/助记词;`wallet_isar` masterId 注释更新。
- UI:`wallet_detail_page` 身份卡加助记词区(reveal 模式,样式同账户私钥区,防截屏/不可复制)+ `_addAccount` 改直接 addAccount + loading;删 `add_account_page.dart` + 其测试。
- 复查修复:#1 `main.dart` 超时重锁 `popUntil(isFirst)`(设备锁深层页绕过);#3 `qr_signer.isValidRequestId` 共享 + `login_qr_handler` 复用校验 env.id;#4 scan_page/home_page 类注释更新(secret_cipher 注释反转后已正确)。
- 测试:`wallet_manager_test`/`wallet_secure_keys_test` 改回种子存储口径;新增 `scan_target_test`(跨钱包谓词)、`wallet_detail_page_test`(助记词区)、account_detail 查看确认取消;删 add_account 测试。
- 终验:`dart analyze` **0** + `flutter test` **213 passed**;残留复扫(add_account/accountMiniSecret/无根语义)全 0;改动只在主检出 citizenwallet。
