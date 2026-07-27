# citizenwallet 终审修复(安全+一致性+测试)

状态:done(2026-07-27)
所属模块:Mobile(citizenwallet 冷钱包)
背景:存储反转落地后做最终一轮独立评审(安全+字段统一+官方派生+残留+遗漏),逐条回码核验([[read-audit-recipe-first]]),修确认项。

## 修复(用户拍板全修)
- **HIGH ScreenshotGuard 全局单例互踢**:`screenshot_guard.dart` 改**引用计数 + 监听器集合**——`enable([cb])`/`disable([cb])` 幂等叠加,平台 FLAG_SECURE 只在计数 0↔1 切换,事件广播给所有在用页;子页 dispose 不再关掉父页仍需的防护。删 `onSecurityEvent` static setter;4 调用方(account_detail/wallet_detail/sensitive_page_mixin 传 cb;offline_sign 无 cb `enable()`)同步。
- **MEDIUM 建钱包非原子写**:`_establishWallet` 种子+助记词两写包 try,任一失败回滚(`_deleteWalletInternal` 删种子+Isar 行,尽力而为)再 rethrow,杜绝"能签名但助记词丢失"半成品。
- **MEDIUM getMasterMnemonic 静默 null**:改为找不到即抛 `WalletAuthException`(与 getAccountPrivateKey 同口径),`_revealMnemonic` catch 统一提示;不再让 UI 把 null 当"无数据"渲染。
- **LOW deleteWallet 非原子 delete**:两次 SecureStorage delete 各自 try/尽力而为(Isar 行已删=钱包已删,清理失败不误报"删除失败")。
- **LOW 命名统一**:`_mnemonicToMiniSecret`→`_mnemonicToSeed`(与他处 seed/seedHex/masterSeedHexV1 一致)。

## 测试(新增/补)
- `test/util/screenshot_guard_test.dart`:引用计数(两页 enable,子页 disable 不关、父页 disable 才关)。
- `test/login/login_qr_handler_test.dart`:请求 id 校验(合法过/含竖线拒/过短拒)——补 M1 的空白覆盖。
- `test/ui/wallet_detail_page_test.dart`:补助记词**揭示成功**路径(真实钱包→查看→显示明文)。

## 评审结论(对 5 问)
- 字段统一:是(唯一漂移 `_mnemonicToMiniSecret` 已改名)。
- substrate 官方实现:是(SURI `//index` 硬 junction;金标钉死 `fromSeed(child)==<助记词>//index`+`//Alice`==权威;`_childMiniSecret` 用 sr25519 公开 API,仅导出私钥必需)。
- 残留:代码符号 0;`getWalletByMasterId` 失去生产调用(保留作对称 API)。
- 漏洞:上轮 H1/M1 已修;本轮 HIGH ScreenshotGuard + 2 MEDIUM 已修。
- 遗漏:测试已补(见上)。

## 验收
- `dart analyze` **0** + `flutter test` **218 passed**;残留复扫(`onSecurityEvent =`/`_mnemonicToMiniSecret`)全 0;改动只在主检出 citizenwallet。
