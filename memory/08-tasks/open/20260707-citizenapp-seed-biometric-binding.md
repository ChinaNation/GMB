# 公民App 钱包密钥硬件级生物识别绑定（Keystore/Keychain 强制）

> 前置：20260707-citizenapp-wallet-gate.md（账户门禁）已完成。
> Step 0 spike 已完成（2026-07-08，结论见 §3）。开发阶段**无数据、不做兼容、不做迁移、彻底替换**（死规则 feedback_no_compatibility / feedback_no_remnants）。

## 1. 问题定位（已实证）

- seed（`wallet.secret.$id.seed_hex.v1`）与助记词（`wallet.secret.$id.mnemonic.v1`）存 `flutter_secure_storage ^9.2.2`，**全项目 5 处实例均无 options**，Android 默认 = Keystore 包 AES 密钥但**未绑定用户认证**；`local_auth` 结果只是 `WalletManager._authenticateIfSupported` 的 UI 布尔值，root/hook 绕过 UI 直接读即解密。
- `flutter_secure_storage` 不暴露用户认证绑定 → 换后端。
- 收敛面封闭：seed/助记词读写全在 `WalletManager` 6 私有方法（wallet_manager.dart:556-699）。**另 4 处 `FlutterSecureStorage`（main.dart:156 device_lock、app_lock PIN、user.dart、attestation）存非密钥材料，不纳入绑定**。
- 平台：iOS 13.0；Android compileSdk 36，minSdk 需 ≥ 23。

## 2. 架构：信封加密 + 双档金库

硬件 KEK 绑定用户认证，解密动作本身触发系统验证，KEK 永不出硬件（`biometric_storage` 内部即此机制，我们只调 read/write/delete）。两档：

| 金库 | 存 | 失效行为 | 平台机制 |
|---|---|---|---|
| **seedVault** | 每钱包 seed hex | 增/删任一指纹即永久失效 | iOS `.biometryCurrentSet`；Android `AUTH_BIOMETRIC_STRONG` 每次认证（默认 invalidatedByBiometricEnrollment=true） |
| **recoveryVault** | 每钱包助记词 | 跟随生物变更**不失效**，设备密码兜底 | iOS `.userPresence`；Android `AUTH_DEVICE_CREDENTIAL\|AUTH_BIOMETRIC_STRONG`（锚定 credential SID） |

## 3. Step 0 Spike 结论（源码级确认，5.0.1）

依赖 `biometric_storage: ^5.0.1` 已加入 pubspec 并 `pub get` 干净解析（无冲突）。读将要打包的原生+Dart 源码确认：

- **两档纯配置即可表达，无需自写 platform channel。** 精确配置：
  - **seedVault（严）**：`StorageFileInitOptions(authenticationValidityDurationSeconds: -1, androidBiometricOnly: true, darwinBiometricOnly: true)`
    - iOS `BiometricStorageImpl.swift:201` → `darwinBiometricOnly=true` ⇒ `.biometryCurrentSet`（增删生物即失效）。
    - Android `BiometricStorageFile.kt:47` → `duration==-1` ⇒ `setUserAuthenticationParameters(0, AUTH_BIOMETRIC_STRONG)`，未调 `setInvalidatedByBiometricEnrollment` → Android 默认 true（严）。
  - **recoveryVault（宽）**：`StorageFileInitOptions(authenticationValidityDurationSeconds: 0, androidBiometricOnly: false, darwinBiometricOnly: false)`
    - iOS → `darwinBiometricOnly=false` ⇒ `.userPresence`（跟随生物变更不失效 + 设备密码兜底）。
    - Android → `duration>=0` ⇒ `setUserAuthenticationParameters(0, AUTH_DEVICE_CREDENTIAL|AUTH_BIOMETRIC_STRONG)`，锚定 credential SID → 生物变更不失效；设备密码可用。
- **写需认证**：对称 AES-GCM 密钥 `setUserAuthenticationRequired(true)`（CryptographyManager.kt:170，PURPOSE_ENCRYPT|DECRYPT）⇒ 加密与解密都要认证 → 创建钱包写 seed 时弹一次认证。无迁移场景下（仅创建时）可接受。
- **StrongBox 加成**：API 28+ 有 StrongBox 时自动 `setIsStrongBoxBacked(true)`（BiometricStorageFile.kt:44），密钥进独立安全芯片。
- **关键约束（改 §6）**：Android 上"严档失效"与"认证有效窗口"**互斥**——`duration>=0` 会切成含 device-credential 的密钥（反而不失效）。故 **seedVault 严档在 Android 锁定 `duration=-1` 每次认证**，两阶段预热窗口对严档无效。
- **失效无专用错误码**：`AuthExceptionCode` 仅 `userCanceled/canceled/timeout/unknown`，密钥失效落入 `unknown`（biometric_storage.dart:417 兜底）。自愈检测策略见 §5。
- Dart API：`BiometricStorage().getStorage(name, options: StorageFileInitOptions(...))` → `BiometricStorageFile.read/write/delete({PromptInfo?})`；`canAuthenticate()` → `CanAuthenticateResponse{success, errorNoBiometricEnrolled, errorNoHardware, errorHwUnavailable, errorPasscodeNotSet, unsupported, statusUnknown}`。

## 4. 数据模型 / 新抽象

新增 `lib/wallet/core/secure_seed_store.dart`：
```
abstract interface class SecureSeedStore {
  Future<void> putSeed(int walletIndex, String seedHex);   // 严档写（弹认证）
  Future<String?> readSeed(int walletIndex);               // 严档读（弹认证）；失效→抛 SeedKeyInvalidated
  Future<void> deleteSeed(int walletIndex);                // 清严档条目（连带清失效 keystore key）
  Future<void> putMnemonic(int walletIndex, String mnemonic);
  Future<String?> readMnemonic(int walletIndex);           // 宽档读
  Future<void> deleteMnemonic(int walletIndex);
  Future<AuthCapability> capability();                     // canAuthenticate 归一：生物/仅凭证/无锁屏
}
```
- 后端 `BiometricSecureSeedStore`：seedVault/recoveryVault 各一组 `getStorage` 配置（§3）。
- 存储文件名（干净命名，无 v1/v2 兼容包袱）：seedVault `wallet_seed_$id`，recoveryVault `wallet_recovery_$id`（每钱包一 keystore key，delete=删该文件；失效爆炸半径限单钱包）。
- 错误类型：`SeedKeyInvalidated`（触发自愈）/ `AuthCancelled`（userCanceled/canceled/timeout → 中止）/ `NoDeviceCredential`（无锁屏 → fail-closed）。

## 5. 自愈流程（见状态机图）

`readSeed`：成功→签名；失败且**非** AuthCancelled（即 unknown，含失效）→ 归为 `SeedKeyInvalidated` → 读 recoveryVault 助记词 → 重新派生 seed → **先 `deleteSeed` 清失效 keystore key** → `putSeed` 重新封装 → 重试读 → 签名（用户仅多一次验证，不手输）；助记词也读不出（数据被清/无锁屏）→ 引导走现有 `importWallet`；AuthCancelled → 中止（非自愈）。收敛在 `WalletManager` 读取入口统一 catch。自愈重派生后必校验 pubkey 与 profile 一致再落库。

## 6. 签名路径改造（严档=每次一次认证，去掉两阶段窗口）

- Spike 结论：seedVault 严档 Android 强制每次认证，`authenticateForSigning` 预热+`signWithWalletNoAuth` 的窗口复用在严档拿不到 → **收敛为单次认证**：构造好待签 payload 后，一次 `readSeed`（弹一次认证）→ 派生 → 签名 → seed 清零。
- 删除现 `signWithWalletNoAuth`/两阶段 split；`signWithWallet`/`signUtf8WithWallet` 内部认证改由 `readSeed` 解密触发（不再 `_authenticateIfSupported`）。
- 为保留"验证在等 RPC 之前"的体验：先备好 payload 再 `readSeed`（一次弹窗即签），或调用方在构造前提示。派生逻辑（`_mnemonicToMiniSecret`/`_deriveSr25519FromSeed`）零改动 → 现有签名金标向量测试原样通过。

## 7. 设备凭证强制（D3）

- vault 配 `androidBiometricOnly=false`（recovery）/ iOS 非 biometricOnly 允许设备密码；无生物识别机型可用图案/数字密码。
- `capability()` 判定：`errorPasscodeNotSet`/无锁屏 → `NoDeviceCredential` → **fail-closed 禁止创建/读取**，与 `createWallet._ensureDeviceSecure` 口径一致。**必须有设备密码才能创建/读取钱包。**

## 8. 影响文件

- 新增 `lib/wallet/core/secure_seed_store.dart`（抽象 + biometric_storage 后端 + 自愈）。
- `lib/wallet/core/wallet_manager.dart`：6 私有存储方法换 `SecureSeedStore`；签名路径改单次认证；读取入口统一 catch 自愈；`_authenticateIfSupported` 相关删除。
- `pubspec.yaml`：已加 `biometric_storage: ^5.0.1`（spike 落地）。
- `android/app/build.gradle.kts`：minSdk → 23（D4）。
- iOS `Info.plist`：补 `NSFaceIDUsageDescription`（Face ID 必需，Step 3 核对）。
- 彻底替换：删除 `flutter_secure_storage` 承载 seed/助记词的旧路径与 `WalletSecureKeys.seedHexV1/mnemonicV1`（无残留；flutter_secure_storage 仍供 PIN/device_lock 用，保留）。

## 9. 测试策略

- Mock `biometric_storage` MethodChannel（channel `biometric_storage`）：write→read 往返；模拟 `unknown` 读失败→自愈重派生成功；`userCanceled`→中止；无助记词→手动导入信号；`errorPasscodeNotSet`→fail-closed。
- 签名金标向量（现有）原样通过（派生零改动）。
- 设备凭证-only 路径（无生物识别机型）。
- e2e（真机/集成，headless 测不了生物）：换/加指纹→自愈；移除锁屏→fail-closed 且提示手抄本恢复。

## 10. 实现分步（建议卡）

- ~~Step 0 spike~~ ✅ 完成（§3）。
- ~~Step 1~~ ✅ 完成（2026-07-08，§10b）：`SecureSeedStore` 抽象 + `BiometricSecureSeedStore` 后端 + 12 条 mock 单测全绿，WalletManager 未动、App 行为零变化。
- ~~Step 2~~ ✅ 完成（2026-07-08，§10d）：WalletManager 6 方法切后端 + 两档签名（Tier1 每次认证 / Tier2 会话密钥）+ 自愈接入 + 删旧（flutter_secure_storage/local_auth/7 私有存取/2 弃用 getter/WalletSecureKeys.seedHexV1·mnemonicV1）+ 16 调用点加 walletIndex + `_AppLockGate` 清会话密钥。彻底替换无兼容无迁移。flutter analyze 干净、32 新单测 + signer/派生/门禁回归全绿（签名金标向量不变=派生零改）。
- Step 3：D3 原生落地校验 + D4 minSdk 23 + iOS Info.plist NSFaceIDUsageDescription + 真机 e2e（换指纹自愈 / 移除锁屏 fail-closed）。
- ~~遗留清理~~ ✅ 完成（2026-07-08）：删 `LocalSigner`（整个 `lib/signer/local_signer.dart`）+ `WalletSecret` 类（`wallet_manager.dart`）+ 唯一消费方 `test/signer/local_signer_test.dart`；连带删 `lib/signer/signer.dart` barrel 的 `export 'local_signer.dart';` 行（否则悬空 export 编译失败）。seed 出类模型彻底移除（违背硬件绑定）。grep 已验四类符号 + `WalletSecret` 生产零引用（仅定义处 + 该测试）。`flutter analyze` 干净（唯一 info 为无关旧文件 onchain_payment_service.dart）；`flutter test --concurrency=1` 全绿（433 passed / 5 skip / 0 fail；默认并发下唯一 fail 为 personal_proposal_history 的 Isar 跨测隔离既有 flake，隔离单跑绿，与本清理无关）。纯清理零行为变更无残留。

## 10b. Step 1 详细规格 ✅ 已实现（抽象 + 后端 + mock 单测，不动 WalletManager）

**边界**：只交付存储层；不碰 WalletManager/签名/UI；App 行为零变化（后端未接线）。**自愈编排属 Step 2**，Step 1 的 store 只做"存储 + 错误分类"，不做重派生。

**已落地文件**：
- `lib/wallet/core/secure_seed_store.dart`：`abstract interface class SecureSeedStore` + `enum SecureAuthStatus{available,noDeviceLock,unsupported}` + sealed `SecureSeedException`（`SeedKeyInvalidated`/`AuthCancelled`/`NoDeviceCredential`/`SecureStoreUnavailable`）。
- `lib/wallet/core/biometric_secure_seed_store.dart`：`BiometricSecureSeedStore implements SecureSeedStore` + 双档配置常量 + `AuthException`→我们错误的映射（`_mapCancelOr`，取消三类→AuthCancelled，其余→调用点决定）+ handle 缓存 + 中文 PromptInfo。写前做 `authStatus()==noDeviceLock` 的 D3 fail-closed。
- `test/wallet/biometric_secure_seed_store_test.dart`：mock `MethodChannel('biometric_storage')`，12 条全绿。
- `pubspec.yaml`：`biometric_storage: ^5.0.1`（现被 Step 1 代码真实引用，非残留）。

**注**：`FakeSecureSeedStore`（内存版，供 Step 2 WalletManager 测试注入）**推迟到 Step 2 与消费方一起落地**——Step 1 无引用，避免死文件（YAGNI / 清理残留）。

**验证**：`dart analyze lib test/wallet` 干净（唯一 info 为无关旧文件 onchain_payment_service.dart）；`flutter test test/wallet/` 73 条全绿（含门禁/manager/新 store）。

**双档配置（源码级锁定，§3）**：
- 严档 seedVault：`StorageFileInitOptions(authenticationValidityDurationSeconds:-1, androidBiometricOnly:true, darwinBiometricOnly:true)`，文件名 `wallet_seed_$id`。
- 宽档 recoveryVault：`StorageFileInitOptions(authenticationValidityDurationSeconds:0, androidBiometricOnly:false, darwinBiometricOnly:false)`，文件名 `wallet_recovery_$id`。

**错误映射**（catch `AuthException`，其 code∈{userCanceled,canceled,timeout,unknown}）：
- readSeed：cancel/canceled/timeout→`AuthCancelled`（中止，绝不自愈）；unknown→`SeedKeyInvalidated`（Step 2 触发自愈）。
- 写路径 / readMnemonic：cancel 三类→`AuthCancelled`；其余→`SecureStoreUnavailable`（宽档不该失效，写无可自愈对象，不误判成 SeedKeyInvalidated）。
- `deleteSeed` 必须真正删条目（连带释放失效 keystore key，供自愈 re-put 生成新 key）。
- `authStatus()` 仅**咨询用**（UI 文案）：`Success`→available，`ErrorPasscodeNotSet`→noDeviceLock，其余→unsupported；**D3 硬门禁靠 write/read 的 NoDeviceCredential**，不靠它。

**测试矩阵**（mock channel，沿用现有 wallet_manager_test 风格，内存 name→content map）：
1. putSeed → 断言 `init` 收到严档 options（-1/true/true）+ `write` 写入 `wallet_seed_1`。
2. putMnemonic → `init` 宽档 options（0/false/false）+ 写入 `wallet_recovery_1`。
3. readSeed 正常 → 返回存值。
4. readSeed channel 抛 `PlatformException('AuthError:UserCanceled')` → `AuthCancelled`。
5. readSeed channel 抛 `PlatformException('AuthError:Foo')`（未映射 AuthError→unknown）→ `SeedKeyInvalidated`。
6. deleteSeed → `delete` 命中 `wallet_seed_1`。
7. authStatus：canAuthenticate 返回 `Success`/`ErrorPasscodeNotSet`/`ErrorNoHardware` → available/noDeviceLock/unsupported。
8. readMnemonic 取消→`AuthCancelled`；缺失→null。
- 注：host 测试 `Platform.isMacOS`=true，插件走 iosPromptInfo 分支，mock 忽略 prompt 参数即可。

**完成判据**：`dart analyze` 干净、`dart format` 干净、新单测全绿、现有套件不受影响（WalletManager 未动）、App 行为零变化。

## 10c. Step 2 详细规格（WalletManager 切后端 + 会话签名密钥 + 自愈 + 删旧路径）

### 签名真相核实（2026-07-08，纠正早前误判）

- **聊天消息不做钱包签名**：im_runtime 的钱包签名仅用于 ① mailbox 后端会话认证 ② IM 设备绑定（MLS key package），聊天消息本体走 MLS 端到端加密（设备密钥，非 seed）。**发消息无每条签名**。
- **"登录"= 后端会话握手**：square_session_provider 对广场/IM Cloudflare Worker 签 challenge 证明钱包所有权，约一次/会话，无用户登录界面。
- **广场发布 = 链上交易**：`publish_square_post` extrinsic（square_compose_signers.signChain），故扣费；费种见「广场发布费率」。

### ⚠️ 最终定案（2026-07-08）：完全统一签名，废弃两档 + 会话密钥

下方"两档签名 + 会话密钥"及 D5/D6 为演进中间态，**已被用户否决并废弃**。最终落地模型：

- **完全统一**：凡动钱动权（转账 / 投票 / 切换默认身份 / **发布动态**）一律走 `WalletManager.signWithWallet(walletIndex, payload)`，**每次读严档 seed 触发一次生物识别 / Face ID**，派生密钥用后即弃。「要么验证、要么不验证」，一次操作一次验证。
- **无会话密钥、无 Tier1/Tier2、无 authenticateForSigning**（全删）。
- **发布按钮改「签名发布」**：点击→弹验证→直接发布，用户仍一步。
- **切换身份**无签名负载 → `verifyWalletAccess(walletIndex)`（读 seed 触发验证即弃）。
- **废弃理由**：seed 是全权私钥，发动态与转账同一把钥匙，"发动态免验证"和"转账必验证"不可兼得；会话密钥又因 App 进程被杀掉内存而退化成"每次开 App 验证"，而开 App 比发动态频繁，得不偿失。
- **IM 聊天不签名**（仅 mailbox 会话 + MLS 设备绑定，罕见、令牌缓存过期才重签，不拦浏览）。
- 若日后要"发动态真正零验证"，唯一干净解 = 链上"发帖专用子钥"（低权限、不能转账、低摩擦存储）——用户当前选择不做，维持统一。

---

### 核心解法：两档签名 + 会话密钥（session key）〔已废弃，见上〕

- **冲突（缩小后）**：seed 严档 = Android 每次读都认证。真正的高频项只有**广场发布**（聊天不签、会话认证/设备绑定是一次/会话）。若发布每条弹指纹则体验差且发布非敏感操作。
- **两档模型**：
  - **Tier 1 严档（每次操作新鲜生物识别）**：转账、投票、切换默认身份。`authenticateForSigning(walletIndex)` 强制新鲜 per-use readSeed → 刷新会话密钥；随后 `signWithWalletNoAuth` 用之。
  - **Tier 2 会话密钥（会话内一次生物识别，之后静默）**：后端会话认证、IM 设备绑定、广场发布。裸 `signWithWalletNoAuth`：会话槽命中即静默签；空则惰性加载一次。
- **会话密钥**：一次认证读 seed → 派生 sr25519 KeyPair → 缓存**静态**会话槽（`WalletManager` 非单例，40 处 new，故 class-level static）→ 静默签不再读 seed。**生命周期 = App 进程存活期**（进程被杀 / 删钱包 / 登出才清），**不随 5 分钟 App 锁清除**——否则用户每隔 >5 分钟重进就为发广场动态重复生物识别，体验差（2026-07-08 用户明确否决）。App 锁只拦入口，会话密钥留着；Tier1 动钱/换身份不受影响仍每次强制认证。
- **UX 口径（D5）**：推荐 **P1 惰性**——会话内首个签名（通常是会话认证，早发生）弹一次，之后广场发布全静默；每会话约一次验证。备选 P2 解锁预载（每次打开弹）——不推荐。
- **安全口径（如实更新 §2）**：硬件绑定完整保护**静止态**（关机/锁屏/离线/被盗提取）；会话解锁活跃期 KeyPair 在内存，运行时被攻破可取——标准移动钱包权衡，相对现状（seed 无认证绑定 root 随时可读）仍大幅提升。

### 广场发布费率（citizenchain runtime，单独卡 + 重新创世）

- 现：`configs/mod.rs:402` `RuntimeCall::SquarePost(_) => FeeChargeKind::VoteFlat`（VOTE_FLAT_FEE=100 FEN=1元）。
- 改：`=> FeeChargeKind::OnchainAmount(0)` → `max(0×rate, ONCHAIN_MIN_FEE=10 FEN)` = **0.1元**；更新测试 `runtime_square_post_fee_kind_is_vote_flat_one_yuan`。
- 属 citizenchain 改动，**不并入本 citizenapp 卡**，另开链端卡。

### 签名 API 改造

- 新增静态会话槽 `_SessionKey{walletIndex, KeyPair}? _sessionKey` + `static void clearSessionKey()`（`_AppLockGate` 锁定时调用）。
- `authenticateForSigning(int walletIndex)`：**新增 walletIndex 参数**（原无参）→ `_loadSigningKey(index)`（强制新鲜 readSeed + 自愈）→ 刷新会话槽。15 处调用点补 `walletIndex`（值域内即 `wallet.walletIndex`）。
- `signWithWalletNoAuth(index, payload)`：会话槽命中即签；否则惰性 `_loadSigningKey(index)`（一次验证）→ 缓存 → 签。保留方法名避免大改（语义=用会话密钥签）。
- `signWithWallet` / `signUtf8WithWallet`（myid_sign_page、登录 UTF8）：改走 `_loadSigningKey` + 自愈（去掉 `_authenticateIfSupported`）。
- `_loadSigningKey(index)`：`readSeed` → 派生 KeyPair → 校验 pubkey 与 profile 一致 → 返回；捕获 `SeedKeyInvalidated` 走自愈（§5）。seed hex 用后清零，仅缓存 KeyPair。

### 存储方法切后端（6 私有方法 → SecureSeedStore）

- 注入：`static SecureSeedStore _store = BiometricSecureSeedStore()` + `@visibleForTesting static set debugStore`。
- `_writeSeedHex→_store.putSeed`、`_readSeedHex/_readSeedHexRaw→_store.readSeed`、`_deleteSeedHex→_store.deleteSeed`、`_writeMnemonic→_store.putMnemonic`、`_readMnemonicRaw→_store.readMnemonic`、`_deleteMnemonic→_store.deleteMnemonic`。
- `createWallet`/`importWallet`：`putSeed`+`putMnemonic`（putSeed 触发一次写认证=创建时验证，符合 D3）；用户在写认证处取消 → `AuthCancelled` → 回滚 + 页面提示。
- `_verifyWalletPersisted`：改为**只校验 Isar profile 落库**，不再回读 seed（回读会二次弹认证）；seed 落库由 `putSeed` 成功保证。
- `getSeedHex`/`getMnemonic`（查看密钥 UI）：走 `_store.readSeed`/`readMnemonic`（弹认证，查看密钥应验证）。

### 删除（彻底替换，无兼容 / 无残留）

- `WalletManager._secureStorage` 字段、`_seedKey`/`_mnemonicKey`/`_seedHexPattern`、`WalletSecureKeys.seedHexV1`/`mnemonicV1`（`sessionTokenV1`/`sessionKeyV1` 保留他用）。
- `_authenticateIfSupported`（认证改由 store 解密触发）；`local_auth` 仅 `_ensureDeviceSecure` 的 `isDeviceSupported` 保留（快速预检）。
- `@Deprecated` 的 `getLatestWalletSecret`/`getWalletSecretByIndex`（把 seed 泄出类，违反设计；grep 确认仅测试引用）→ 删，同步删相关测试断言。
- `WalletSecret` 类若仅上述两法使用 → 一并删。
- flutter_secure_storage 包保留（PIN/device_lock 仍用）。

### 测试（新建 FakeSecureSeedStore）

- `test/support/fake_secure_seed_store.dart`：内存实现 `SecureSeedStore`，开关 `Set<int> invalidatedSeeds`（readSeed 抛 `SeedKeyInvalidated`）、`noDeviceLock`、`cancelReads`（抛 `AuthCancelled`）。
- 用例：createWallet 落 seed+mnemonic；`authenticateForSigning`+`signWithWalletNoAuth` 只认证一次（会话密钥复用）；**自愈**（标记失效→signWithWalletNoAuth 读助记词重派生 re-put 签名成功、store seed 刷新）；自愈 pubkey 不一致→抛错不签；`AuthCancelled`→上抛不自愈；D3 noDeviceLock→createWallet 抛；`clearSessionKey` 后重新认证；delete/clearWallet 清 store；getMnemonic/getSeedHex 走 store；签名金标向量不变（派生零改动）。
- `_AppLockGate` 锁定/超时处调用 `WalletManager.clearSessionKey()`。

### 决策待确认

- **D5 静默签名 UX**：P1 惰性（推荐，每会话约一次验证，登录时）/ P2 解锁预载（每次打开弹）。
- **D6 会话密钥生命周期（已定案 2026-07-08）**：= App 进程存活期（进程杀 / 删钱包 / 登出清），**不随 5 分钟 App 锁清**。理由=避免每次重进 App 为发广场动态重复验证。残留：cold start（进程被 OS 杀）后首次发布仍需一次生物识别（seed 是全权私钥，不能存低摩擦槽，否则可被离线签转账）——不可再省，但已不是"每 5 分钟"。

## 10d. Step 2 详细规格（WalletManager 切后端 + 自愈 + 删旧）

> 注：本节"两档签名 API"部分已被「⚠️ 最终定案」取代为**完全统一签名**（一个 `signWithWallet` + `verifyWalletAccess`，每次验证，无会话密钥）。其余（切后端、自愈、删旧、测试）仍有效。

**新增静态状态**（`WalletManager` 非单例，缓存必须 class-level static）：
- `static SecureSeedStore _store = BiometricSecureSeedStore();` + `@visibleForTesting set debugSeedStore`。
- `static _SessionKey? _sessionKey;`（`{int walletIndex; KeyPair pair;}`）会话签名密钥。
- `static void clearSessionKey()`：置空；由 `main.dart _AppLockGate` 重新上锁（5 分钟后台超时 / 显式锁）时调用。

**核心加载器（自愈落点，统一收敛）**：
- `_readSeedHexWithSelfHeal(walletIndex, profile) → seedHex`：
  - `await _store.readSeed(i)`；正常返回。
  - `on SeedKeyInvalidated` 或返回 null → `_selfHeal`：读宽档 `readMnemonic`（null → 抛「生物识别已变更，请用助记词重新导入」）→ `_mnemonicToMiniSecret` → 派生校验 pubkey 与 profile 一致（不一致抛）→ `deleteSeed` 释放失效 key → `putSeed` 重新封装 → 返回新 seedHex。
  - `AuthCancelled`/`NoDeviceCredential` 直接上抛，**不自愈**。
- `_loadSigningKey(i) → KeyPair`：`_readSeedHexWithSelfHeal` → `_keyPairFromSeedHex`（保留现有 pubkey 一致性校验）。

**两档签名 API**：
- **Tier 1（转账/投票/切换身份，每次新鲜认证）**：`authenticateForSigning(int walletIndex)`（**签名新增 walletIndex**）→ `_sessionKey = _loadSigningKey(i)`（强制新鲜 readSeed=生物识别）。随后 `signWithWalletNoAuth(i,payload)` 命中缓存静默签。每次 value 操作流各调一次 `authenticateForSigning` → 每次都新鲜认证。
- **Tier 2（后端会话认证/设备绑定/广场发布，会话内一次）**：裸 `signWithWalletNoAuth(i,payload)` → `_sessionKeyFor(i)`：缓存命中且 index 一致 → 静默；否则惰性 `_loadSigningKey`（一次生物识别）+ 缓存。
- `signWithWallet`/`signUtf8WithWallet`（myid 身份签名，Tier 1）：直接 `_loadSigningKey`（新鲜认证）+ 刷新缓存 + 签。
- 16 处 `authenticateForSigning()` 调用点加 `wallet.walletIndex`（机械改；均在 scope 内）。

**存取方法切后端（6 私有 + 2 公有）**：
- `createWallet`/`importWallet`：`_writeSeedHex`→`_store.putSeed`、`_writeMnemonic`→`_store.putMnemonic`。**创建时两次生物识别**（seed 严档 + 助记词宽档各一次写认证）——罕见操作，接受；文案提示。
- `_verifyWalletPersisted`：改为**只校验 Isar profile**，不再回读 seed（避免第三次弹窗）；putSeed 成功即信。
- `getSeedHex`→`_readSeedHexWithSelfHeal`（查看 seed 也自愈+认证）；`getMnemonic`→`_store.readMnemonic`。
- `clearWallet`/`deleteWallet`/`_rollbackWalletCreation`：`_deleteSeedHex/_deleteMnemonic`→`_store.deleteSeed/deleteMnemonic`；删除命中钱包时 `clearSessionKey()`。
- `_ensureDeviceSecure`：改用 `_store.authStatus()==noDeviceLock` 抛（D3 早失败），弃 local_auth。

**删除清单（无残留）**：`_secureStorage`、`_localAuth`、`_authenticateIfSupported`、`_readSeedHex`、`_readSeedHexRaw`、`_writeSeedHex`、`_deleteSeedHex`、`_writeMnemonic`、`_readMnemonicRaw`、`_deleteMnemonic`、`_seedKey`、`_mnemonicKey`、`_seedHexPattern`、`getLatestWalletSecret`、`getWalletSecretByIndex`；`WalletSecret` 类（无他用则删）；`WalletSecureKeys.seedHexV1/mnemonicV1`（保 sessionTokenV1/sessionKeyV1）；wallet_manager.dart 的 local_auth import。

**main.dart 接线**：`_AppLockGate` 重新上锁处（`didChangeAppLifecycleState` 超时分支 + 显式锁）调 `WalletManager.clearSessionKey()`。

**测试（新建 `test/support/fake_secure_seed_store.dart`）**：内存 `FakeSecureSeedStore`（toggles：`invalidatedSeeds`/`noLock`/`cancelSeedReads`/计数器）。`WalletManager.debugSeedStore=fake`、`clearSessionKey()` 在 setUp/tearDown。用例：① createWallet 写两档+profile；② Tier1 authenticate+sign 仅一次 readSeed；③ Tier2 惰性缓存命中零重读；④ 自愈成功（invalidated→读助记词重派生重封装签成，putSeed 计数+1）；⑤ 自愈 pubkey 不符抛；⑥ 无助记词抛需重导；⑦ AuthCancelled 上抛不自愈不 putSeed；⑧ D3 noLock createWallet 抛；⑨ clearSessionKey 后重读；⑩ deleteWallet 清两档+清缓存；⑪ 签名金标向量（派生零改）。改写现有 wallet_manager_test：删两个 deprecated 用例、create/import/delete 断言改走 fake。

## 10e. Step 3 详细规格（原生构建验证 + 真机 e2e）

**现状核实（原生地基已就绪，2026-07-08）**：
- Android MainActivity = `FlutterFragmentActivity`（`android/app/src/main/kotlin/org/citizenapp/MainActivity.kt`）✅——biometric_storage 的 BiometricPrompt 硬性要求，天然满足。
- minSdk = `flutter.minSdkVersion` = **24**（Flutter 3.41 `FlutterExtension.kt:26` 默认）≥ biometric_storage 声明的 23 → **D4 无需显式改**（不硬钉，跟随 Flutter 默认；未来若 Flutter 降默认再钉）。
- iOS `NSFaceIDUsageDescription` 已存在（Info.plist:76）✅；部署目标 13.0 ✅。
- 故 Step 3 **基本无新配置代码**，核心是验证 + 真机行为确认。

**S3-1 构建验证**：
- **Android ✅ 通过（2026-07-08）**：`flutter build apk --debug` → `✓ Built app-debug.apk`。biometric_storage native + androidx.biometric 编入 APK，minSdk 24≥23、依赖解析、manifest 合并全过；合并后 manifest 自动含 `USE_BIOMETRIC` + `USE_FINGERPRINT`（androidx.biometric 自带，无需手动声明）。
  - **唯一修复**：biometric_storage 5.x 用 `jvmToolchain(17)` 编译，本机仅 JDK 21 → 首次构建报 "Cannot find a Java installation matching {languageVersion=17}"。修法=`android/settings.gradle.kts` plugins 块加 `org.gradle.toolchains.foojay-resolver-convention` 0.9.0（Gradle 按需自动拉 JDK 17，CI/任意机器可复现）。属仓库级必要配置，已入库。
- **iOS ⏳ 待验**：本机 Xcode 安装不完整（`flutter doctor` [✗]），无法在此机器构建。待有完整 Xcode 的机器跑 `flutter build ios --no-codesign --debug`（预期无碍：Info.plist/部署目标已就绪，仅需确认 pod 集成）。

**S3-2 Android 两档失效语义（核心，仅真机可验）**：
- 严档 seed（`androidBiometricOnly:true, validity:-1`）：`AUTH_BIOMETRIC_STRONG` + `setInvalidatedByBiometricEnrollment` 默认 true → **增/删任一指纹即失效**。
- 宽档助记词（`androidBiometricOnly:false, validity:0`）：含 `AUTH_DEVICE_CREDENTIAL`，锚定锁屏凭证 → **随生物变更不失效**。
- 这套"严失效 / 宽存活"是 D2 静默自愈的地基，biometric_storage 原生不暴露该 flag，只能真机确认（Step 0 已源码推断，Step 3 实测锁定）。

**S3-3 真机 e2e 协议（手动，精确步骤 + 预期）**：
- T1 创建钱包 → **两次生物识别**（seed 严档写 + 助记词宽档写）→ 落库、可正常进 App。
- T2 Tier1 转账/投票/切默认身份 → 每次各弹一次生物识别。
- T3 Tier2 广场发布 → App 进程内首个签名弹一次，之后连发静默；后台 >5 分钟重进（App 锁重验入口）后发布**仍静默**（会话密钥未清）；只有 App 进程被杀后 cold start 首次发布才再弹一次。
- T4 **换指纹自愈**：系统设置增/删一枚指纹 → 回 App 发起签名 → 应自动读宽档助记词重派生重封装（用户仅多一次验证，**不手输助记词**）→ 签名成功；再次签名不再自愈（新严档 key 已生效）。
- T5 **移除全部锁屏**：创建/签名应 fail-closed 报错（引导重开锁屏；已有钱包用户无锁屏则严档+宽档皆不可读，提示用手抄助记词重导）。
- T6 App 后台 >5 分钟回前台 → App 锁重验入口（PIN/设备锁）但**会话密钥不清**；发广场动态仍静默。仅 App 进程被 OS 杀掉后 cold start 才需重新认证。
- iOS 对照：`biometryCurrentSet`（严）换脸/改指纹后 seed 失效→自愈；`userPresence`（宽）存活。

**S3-4 可选 UX（小）**：创建页加一行"创建时需两次身份验证（分别保护私钥与助记词）"提示，消解 T1 两次弹窗的困惑。

**完成判据**：apk/ios 构建通过；真机 T1–T6 全部符合预期（尤其 T4 自愈、T5 fail-closed）；记录 Android/iOS 各一台真机结果回写本卡。

## 11. 已知风险 / 边界

- 移除**全部**设备锁屏 → 两档 KEK 均失效 → 需链下手抄助记词重导（正确 fail-closed，文案明示）。
- 严档每次签名一次生物/密码验证（Android 硬约束，非 bug）——钱包场景可接受，文案不承诺免验证。
- `NSFaceIDUsageDescription` 缺失 → Face ID 静默失败，Step 3 必查。
- `biometric_storage` 5.0.1 为 2 年前稳定版；用其发布版源码验证过双档语义，锁版本。
- 若未继续 Step 1：`pubspec.yaml` 的 `biometric_storage` 为 spike 唯一残留，revert 即可。

## 12. 加固排查与进展（2026-07-08，全部已核实）

**已做：**
- **[CRITICAL·真机阻断] seed 金库改为允许设备密码**：原 `_seedOptions` = `androidBiometricOnly:true`/`darwinBiometricOnly:true`（**仅生物识别**）+ `validity:-1`。真机首次「创建钱包」点击即 `SecureStoreUnavailable:unexpected error`——根因：biometric_storage 里 `biometricOnly:true` 禁用 PIN/图案且 `-1` 强制 biometricOnly，**无指纹/无人脸设备（PIN-only 机型 / 模拟器）建密钥失败**，且违背 D3。改 seed 金库为 `validity:0 + androidBiometricOnly:false + darwinBiometricOnly:false`（与 recovery 金库一致，每次验证但允许 PIN/图案）。代价：seed 不再「增删指纹即失效」，锚定设备凭证、随生物变更不失效（自愈保留兜底）。§3 D2「seed 严档 biometryCurrentSet」作废。测试断言同步更新。
- **[UI] 按钮「创建热钱包」→「创建钱包」**（onboarding 页 + 我的钱包两处入口）。
- **[LOW] 内存清零**：`_keyPairFromSeedHex` 派生后 finally 清零 seedBytes（重构统一签名时丢失，已补回；`fromSeed` 派生独立 SecretKey 不引用输入，清零安全）。
- **[HIGH] Android 关闭明文流量**：`network_security_config.xml` `cleartextTrafficPermitted` `true`→`false` + 127.0.0.1/localhost/10.0.2.2 dev 例外。**已验证安全**：生产 HTTP 端点全 HTTPS（cid/square/chain-bootstrap/update）、smoldot bootnodes 全 `/dns4/…/wss`、纯 ws localhost bootnode 仅 DEV dart-define 注入；iOS 无 ATS 配置 = 默认已阻断明文。`flutter build apk` 通过。

**真机根因修复（2026-07-08，Pixel 8a Android 16，logcat 坐实）：**
- 真机首次「创建钱包」报 `SecureStoreUnavailable:Unexpected Error`。前两次判断（无生物识别 / CryptoObject 路径）**均错**。logcat keystore2 铁证：`KEY_USER_NOT_AUTHENTICATED / No operation auth token received`。
- **真根因**：两个金库 `authenticationValidityDurationSeconds: 0`（seed 是我上一处"修复"改坏的；助记词金库从 Step 1 起就是 0，一直坏、只是没真机测过）。biometric_storage 把该值传给 `setUserAuthenticationParameters(N,…)`（BiometricStorageFile.kt:47-60），**0 = 认证令牌 0 秒过期** → 弹验证成功后、真正加/解密那一刻令牌已失效 → keystore2 拒绝。**任何 seed/助记词读写都必挂**（创建、签名、拖动切换全因此）。
- **修复**：两个金库 `validity: 0 → 10`（`_authTokenTtlSeconds=10`）。窗口只覆盖"验证成功→那一次 AES 解/加密"的毫秒级间隙；sr25519 签名在 Dart 用派生密钥做、不碰 Keystore、不受窗口约束。插件每次读写都重弹验证 → 「每次操作一次验证」不变。`-1` 不能用（强制 biometricOnly、禁 PIN）；`0` 不能用（本 bug）；故用正数。测试断言 0→10，全绿；已装 Pixel 8a 待真机复测。

**待决策 / 待办（非本卡范围，另行安排）：**
- **[MEDIUM] attestation_service 是假占位**：token=`Random(now)` 伪随机、签名=djb2 折叠哈希、payload 写死 `device_integrity:"…placeholder"`、自注释 "MVP placeholder replace with real sr25519 later"、生产无调用点。**提供零真实保证，名不副实**。若要按设备可信/密钥硬件背书放行，须真做 Play Integrity + Android Key Attestation；否则别让任何逻辑依赖它。
- **[MEDIUM] root/越狱仅警告不处置**（main.dart:371 banner）：硬件密钥保证在 root 设备被削弱。策略题——是否禁止 root 设备创建/签名热钱包，待用户定。
- **[MEDIUM] 无证书固定**：RPC / square·IM Worker 无 cert pinning，纵深防御可加。
- **[LOW/未来] StrongBox**：seed KEK 走 TEE（biometric_storage 默认，不暴露安全元件开关），有安全元件机型可升 StrongBox（需自写极薄 channel）。
- **[LOW] 助记词展示**：已防截屏（备份弹窗+查看页+main 覆盖 ✓）；可选加点按才显示 + 自动关闭。

## 是否需要先沟通

- 决策 D1-D4 已定，Step 0/1/2 完成、Step 3 S3-1（Android 构建）通过。加固已做内存清零 + Android 明文流量;attestation 空壳与 root 策略待用户决策。
