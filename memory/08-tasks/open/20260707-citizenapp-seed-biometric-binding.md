# 公民App 钱包密钥硬件级生物识别绑定（Keystore/Keychain 强制）

> 前置：待 20260707-citizenapp-wallet-gate.md（账户门禁）完成后再启动本卡。

## 任务需求

现状：`local_auth` 的验证结果只是 UI 层布尔值，seed/助记词存在 `flutter_secure_storage`（Android Keystore / iOS Keychain 加密），但**未绑定用户认证**——root/hook 场景下绕过 UI 判断即可读密钥。目标：把 seed 与助记词条目在系统密钥库层面强制绑定生物识别/设备凭证：

- Android：Keystore 密钥 `setUserAuthenticationRequired(true)`（配 `setUserAuthenticationParameters` 设定超时/每次认证）。
- iOS：Keychain 条目 `kSecAccessControlBiometryCurrentSet`（生物特征变更即失效，需重新导入——需确认口径）或 `UserPresence`。
- 效果：不通过系统级验证就**解密不出**密钥，密码学层面强制，而非 UI 门禁。

## 已知约束（执行前先核实）

- `flutter_secure_storage` 对 Android `setUserAuthenticationRequired` / iOS `SecAccessControl` 的暴露程度有限，大概率需要换 `biometric_storage` 之类插件或自写 platform channel——涉及**存量密钥迁移**（读旧写新，一次性，迁移失败不得丢 seed）。
- 影响面：`WalletManager` 全部 seed/助记词读写路径（`_writeSeedHex/_readSeedHexRaw/_writeMnemonic/_readMnemonicRaw`）、签名路径 `signWithWallet*`（现在是先 `_authenticateIfSupported` 再读，改造后认证由解密动作本身触发，`authenticateForSigning` 预认证时序需重排）。
- iOS `BiometryCurrentSet` 会在用户增删指纹/换脸后永久失效——对钱包 App 这既是特性也是支持成本，需定口径（失效后引导用助记词恢复）。
- 与门禁卡的「未开锁屏禁止创建」逻辑衔接：绑定后未开锁屏连读取都不可能，页面文案需同步。

## 是否需要先沟通

- 是：插件选型 + iOS 失效口径（CurrentSet vs UserPresence）+ 存量迁移窗口，开工前给一版方案确认。
