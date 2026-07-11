# 20260709 citizenapp 真·硬件密码学绑定 seed 金库（Android CryptoObject + iOS Secure Enclave 原生桥）

## 目标 / 威胁模型

把钱包 seed 的**解密密钥绑死在安全硬件里、由生物识别原子解锁**，做到"没真人指纹/人脸，硬件就不吐明文私钥"。防的是 **root/越狱 / App 被 hook 篡改 / 进程注入**的高级威胁。

**现状（已在 [[project_seed_biometric_binding_design]] 落地）**：seed 存 flutter_secure_storage（Keystore/Keychain 硬件加密**静止态**），动钱动权用 `local_auth` 弹一次生物识别 = **UI 层软门禁**。弱点：解密 seed **不要**生物识别，root/hook 能绕过 Dart 层的 local_auth 判断直接读 seed（"锁在门上、钥匙在门内桌上"）。本卡把它升级成**密码学绑定**。

## 决策锁定 + 密码学约束（2026-07-09 需求分析确认）

**用户拍板两项：**
1. **助记词 = Plan A**：助记词也硬件绑定，存「宽档 recoveryVault」；换指纹后自愈仅多一次验证、不手输。（Plan B「不存、靠纸备份」未采用，留作将来「高安全模式」开关。）
2. **后台静默 = 方案①（P-256 硬件子钥，改后端）**：广场/Chat 后端握手不再静默读 sr25519 seed，改用一把 SE/Keystore 原生 **P-256 子钥**（passkey 式，真硬件签名、不碰 seed、不弹生物识别）；Cloudflare Worker 后端注册子钥公钥绑定钱包。过渡期若后端来不及改，可先走方案②内存缓存，但**目标态 = ①**。

**密码学硬约束（为什么只能信封加密）：**
- sr25519 = Schnorrkel/Ristretto255，**不进任何手机安全元件**。Apple SE 只有 EC **P-256**；Android Keystore 只有 RSA / EC-NIST / AES（+可选 Ed25519）。连比特币 secp256k1 都不进 SE（故硬件钱包才独立存在）。两家能硬件签名的交集 = **ECDSA/P-256（secp256r1，≠ 币圈的 secp256k1）**。
- 故 seed 的硬件保护**只能是信封加密**：硬件 KEK（Android RSA-2048 / iOS SE P-256）当「锁」加解密 seed，**sr25519 签名永远在软件层（Dart）完成**。
- **对 PQC 前向兼容**：将来迁 ML-DSA（[[project_pqc_unified_adr022]]）同样不进 SE → 金库层不变，只把被锁明文从「sr25519 seed」换成「ML-DSA 私钥种子」。

**威胁模型（诚实措辞，写死避免名不副实，勿重蹈 attestation_service 假占位覆辙）：**
- **净增益**：root/hook 无法再静默偷读 seed/助记词 —— 不触发真·生物识别，硬件根本不解密，拿到的只是密文；换指纹令 seed 严档 KEK 永久失效。
- **硬天花板**：sr25519 签名必须软件做 → seed 明文在「用户本人已授权、正在签名的毫秒级窗口」仍进内存，已 root 且能在该瞬间注入 dump 内存者仍可抓到。硬件绑定把窗口从「随时」压到「仅授权操作的瞬时」，消不掉。**准确定位 = 「解密受硬件+生物识别原子门禁，明文仅授权瞬时在内存」，不是「私钥永不出硬件」。**

**两金库分档（核心）：**

| | 严档 seedVault | 宽档 recoveryVault（助记词） |
|---|---|---|
| 用途 | 每次动钱动权签名（高频） | 查看/备份 + seed 失效自愈（低频） |
| 认证 | 仅生物识别 | 生物识别 **或** 设备 PIN |
| 换指纹 | 永久失效 → 自愈 | 存活（锚定设备凭证） |
| 安全底线 | 真人生物 | 设备 PIN |
| Android | RSA-2048/OAEP；`AUTH_BIOMETRIC_STRONG`；`invalidatedByEnrollment=true` | 同 KEK；`AUTH_BIOMETRIC_STRONG\|AUTH_DEVICE_CREDENTIAL`；`invalidatedByEnrollment=false` |
| iOS | `.biometryCurrentSet` | `.userPresence` |
| 写 / 读 | 公钥静默 / 私钥弹验证 | 公钥静默 / 私钥弹验证 |

**铁律**：能扛过「换指纹」的金库，安全底线必然 = 设备 PIN（唯一跨生物变更持久的锚点）。故严档=生物档、宽档=PIN 档，**无免费午餐**。创建钱包两金库均公钥静默写 = **0 弹窗**，首验在首次动钱动权。

**信封结构（混合加密，两平台对齐）**：随机 AES-256 DEK 加密明文（AES-GCM）→ 硬件 KEK 公钥 wrap DEK；读时私钥经 CryptoObject(Android)/LAContext(iOS) 原子解 DEK → 解明文 → 立即清零。iOS ECIES 内建混合，一次 `SecKeyCreateEncrypted/DecryptedData` 即可。

**避坑铁律**：auth-bound key **永远配对 CryptoObject**（认证令牌原子绑定该次 `doFinal`），规避 biometric_storage 的 `validity:0` 令牌 0 秒过期 → `KEY_USER_NOT_AUTHENTICATED`。API30+ `setUserAuthenticationParameters(0, AUTH_BIOMETRIC_STRONG)`；API24-29 降级 `setUserAuthenticationValidityDurationSeconds(-1)`。

**P-256 子钥（方案①）后端改动范围：**
- 客户端：SE/Keystore 生成 per-wallet P-256 子钥（硬件原生签名）；首次用 sr25519 主钥对「子钥公钥」签一次名做绑定证明。
- 后端 Cloudflare Worker：新增子钥注册端点（验 sr25519 绑定证明 → 存 `walletAddress↔P256pubkey`）；登录挑战验签从 sr25519 改验 P-256。
- 涉 3 处客户端静默签名：`square_session_provider`、`square_compose_signers`、`chat_runtime`。
- **待评估**：是否单开 ADR 记「后端 P-256 子钥会话协议」（倾向是，跨端协议）。

## 核心设计：信封加密 + 硬件 auth-bound KEK

- 在 **Android Keystore / iOS Secure Enclave** 生成一把加密 seed 的密钥（KEK），**永不出硬件**；
- **写入（创建钱包）静默**：用**非对称公钥**加密 seed（公钥操作不需认证）→ 密文存普通存储；
- **读取（动钱动权）强制生物识别**：私钥操作被 `setUserAuthenticationRequired(true)` 锁住，解密动作**必须先过一次生物识别**，且验证与解密经 **CryptoObject（Android）/ LAContext（iOS）原子绑定** —— 不是"返回 true 让代码判断"，是"没验证硬件不解密"；
- **每次一验**：Android `setUserAuthenticationParameters(0, AUTH_BIOMETRIC_STRONG)` **必须搭 CryptoObject**（CryptoObject 携带认证令牌原子解锁，规避之前 validity:0/时间窗踩的 `KEY_USER_NOT_AUTHENTICATED`）。

## Android 桥（Kotlin，MethodChannel/Pigeon）
- AndroidX `BiometricPrompt` + `BiometricPrompt.CryptoObject(cipher)`；`FlutterFragmentActivity`（已是）。
- Keystore 密钥 = **RSA-2048 OAEP**（或 EC）非对称对：公钥加密 seed（写静默）、私钥解密（读弹生物识别）。32B seed < RSA 块，直接 OAEP；或混合（随机 AES DEK 加密 seed，KEK 包 DEK）。
- `KeyGenParameterSpec`：`setUserAuthenticationRequired(true)` + `setUserAuthenticationParameters(0, AUTH_BIOMETRIC_STRONG)` + `setInvalidatedByBiometricEnrollment(true)`（增删指纹即失效→走自愈）+ 可选 `setIsStrongBoxBacked(true)`（StrongBox）。
- 错误分类回 Dart：`userCancelled` / `keyPermanentlyInvalidated`(生物识别变更) / `notEnrolled` / `lockout`。

## iOS 桥（Swift，MethodChannel）
- **优先方案（简单且真硬件绑定）**：Keychain item 设 `SecAccessControlCreateWithFlags(.biometryCurrentSet)` —— item 本身被生物识别锁，**读取即触发 Face ID/Touch ID**，硬件后端。Step 0 需确认 flutter_secure_storage 的 `IOSOptions(accessControlFlags:[biometryCurrentSet])` 是否已等价（是则 iOS 可**不写原生桥**，直接用它）。
- **备选（最强）**：Secure Enclave 生成 EC P-256 密钥对，私钥永不出 SE，ECIES 公钥加密 / 私钥经 `LAContext` 生物识别解密。
- `NSFaceIDUsageDescription` 已存在。

## ⚠️ 关键难点：硬件绑定 vs 后台静默签名的矛盾（必须先解）
硬件绑定后**任何读 seed 都弹生物识别**，但现设计里广场/Chat 后端会话握手（`requireAuth:false`）是**静默读 seed 签名**的 → 直接绑定会让开 App 就弹/狂弹。三个候选解，Step 0 spike + 与用户定：
1. **分离密钥（最正确）**：seed=硬件绑定的**花钱主钥**；另派生**非花钱的 session/device 子钥**静默存，登录握手/Chat 用子钥签，后端注册子钥公钥绑定钱包。**需改 Cloudflare Worker 后端**接受 session key。
2. **内存会话缓存**：首次动钱动权生物识别读出 seed → 内存缓存（进后台/被杀即清零），后台握手用缓存静默、动钱动权强制**跳过缓存重新生物识别**。改动小，但"缓存"安全性弱于方案1。
3. **全都验**：握手也弹（回到"一直弹"，用户已否，排除）。
→ **已定：方案①（P-256 子钥，改后端）；过渡可②。详见上「决策锁定」。**

## 架构落点
- 新 Dart 接口 `HardwareBoundSeedVault`（store/read(触发生物识别)/delete/isAvailable + sealed 错误），实现 `SecureSeedStore` 语义，替换 `WalletManager._store` 的动钱动权读路径；
- **接入后**：动钱动权的 `local_auth _requireBiometric` 可去除（读 seed 本身即硬件弹验证，不再软门禁）；后台静默路径按上面难点的选解走子钥/缓存；
- **自愈**：`keyPermanentlyInvalidated`（换指纹）→ 读宽档助记词重派生 seed → 重建硬件金库（与现自愈一致）。

## 分阶段
- **Step 0 spike**：验 Android CryptoObject 每次一验在 Pixel 8a/Android 16 真机跑通（RSA auth-bound + BiometricPrompt CryptoObject 解密成功、无 KEY_USER_NOT_AUTHENTICATED）；验 iOS biometryCurrentSet 是否 flutter_secure_storage 已够（够则免 iOS 桥）；后台静默方案**已定①**（见「决策锁定」）。
- **Step 1**：Android Kotlin 桥 + Dart FFI/MethodChannel 封装 + 错误分类。
- **Step 2**：iOS 桥（或确认复用 flutter_secure_storage）。
- **Step 3**：接入 WalletManager，去软门禁，落后台静默选解，自愈接通。
- **Step 4**：真机 e2e（创建静默 / 动钱动权每次弹 / 换指纹自愈 / root 绕过验证失败）。

## P-256 设备子钥协议（方案①详设，2026-07-09）

**目标**：后台握手（广场 session / Chat 设备绑定）不再静默读 sr25519 seed（硬件绑定后会弹），改用 per-wallet **P-256 硬件子钥**（Keystore/SE，`PURPOSE_SIGN`、**无 user-auth** → 静默硬件 ECDSA，私钥永不出硬件；passkey 式）。

**现有后端**（`citizenapp/cloudflare`，live worker `citizenapp-square-api.stews87-fawn.workers.dev`）：
- `POST /v1/square/auth/challenge` → `buildLoginPayload`（`GMB_SQUARE_LOGIN_V1\nowner_account:..\nchallenge_id:..\nexpires_at:..`）存 D1 `square_login_challenges`。
- `POST /v1/square/auth/session` → `verifyWalletSignature(payload,sig,owner)` = `@polkadot/util-crypto` `signatureVerify`（sr25519）。
- 路由手写于 `src/routes.ts`；migrations 顺序编号 `0001..0007`；Env 有 `DB:D1Database` / `FEED_CACHE:KVNamespace`。

**协议**：
1. **子钥**：per-wallet P-256（Keystore `PURPOSE_SIGN` 无 auth / iOS SE）。
2. **绑定（一次性，钱包创建时 seed 在内存零额外弹窗）**：sr25519 签 `GMB_SQUARE_DEVICE_BIND_V1\nowner_account:{addr}\np256_pubkey:{hex}\nissued_at:{ms}` → `POST /v1/square/auth/device/register {owner_account,p256_pubkey,issued_at,binding_signature}`；后端复用 `verifyWalletSignature`（sr25519）验绑定后存 D1。
3. **握手（静默 P-256）**：challenge 不变；client 用 P-256 子钥签 `signing_payload`（静默）；`session` 查该 owner 已注册 p256_pubkey → **Web Crypto ES256** 验（`subtle.verify({name:ECDSA,hash:SHA-256})`）；无绑定 → 401 `device_not_registered` → client 注册后重试。
4. **格式**：pubkey=裸未压缩点 65B(`0x04||X||Y`) hex；sig=裸 `r||s` 64B hex（client 把平台 DER→raw）。
5. **D1**：新 `migrations/0008_device_subkeys.sql` 表 `square_device_subkeys(owner_account PK,p256_pubkey,issued_at,created_at,updated_at)`（一账户一活跃子钥，重注册覆盖=换机/轮换）。
6. **client 接入**：3 处静默 `signWithWallet(requireAuth:false)`（`square_session_provider`/`square_compose_signers`/`chat_runtime`）换 `DeviceSubkey.sign`。子钥 P-256 gen+sign 需**原生**（Android 加桥；iOS 卡硬件）。

**决策（已定 2026-07-09）**：A=**不单开 ADR**（任务卡为准）；B=**clean cutover**（worker+新 App 同发，session 直接 ES256，旧 App 短暂断登可接受）；C=钱包创建时注册（seed 新鲜零额外弹窗）+ 遇 401 `device_not_registered` 懒注册兜底。

**后端实现（2026-07-09，本地完成，⚠️未部署 / 未 apply migration）**：
- `migrations/0008_device_subkeys.sql`（表 `square_device_subkeys`，owner_account PK，一账户一活跃子钥重注册覆盖）。
- `src/auth/device_subkey.ts`：`buildDeviceBindingPayload`（`GMB_SQUARE_DEVICE_BIND_V1\nowner\np256_pubkey\nissued_at`）+ `assertP256PublicKeyHex`（65B 裸点 0x04）+ `verifyP256Signature`（Workers Web Crypto ES256，sig 裸 r||s 64B）。
- `src/auth/service.ts`：`registerDeviceSubkey`（sr25519 验绑定证明 → upsert）；`createSession` 验签 **sr25519 → ES256**（查子钥，无则 401 `device_not_registered`）。
- `src/routes.ts` 挂 `POST /v1/square/auth/device/register`；`types.ts` 加 `DeviceSubkeyRow`。
- 测试 `test/device_subkey.test.ts` 5 例（ES256 往返/0x前缀/篡改/畸形/pubkey 校验），全套 **72/72 绿**、typecheck 干净。**未 `wrangler deploy`、未 apply migration**——等 App 端就绪，与 App 发布**同步 clean cutover**（deploy 属对外操作，需用户明确许可）。

**剩余（客户端，与 worker 部署强耦合、必须同发）**：
1. native P-256 gen+sign（Android 加桥：Keystore P-256 `PURPOSE_SIGN` 无 auth、ECDSA 签、导出裸点；iOS SE 卡硬件）。
2. Dart `DeviceSubkey`（ensureKey/pubkey/sign；**平台 DER→raw r||s**；调 `/auth/device/register`；401 懒注册）。
3. 3 处静默路径（`square_session_provider`/`square_compose_signers`/`chat_runtime`）改 `DeviceSubkey.sign`；钱包创建时 sr25519 签绑定注册。
4. **Step 3**：`WalletManager._store` 切 `HardwareBoundSeedVault` + 去 `_requireBiometric` local_auth 软门禁（前台动钱动权改硬件金库弹验证）。

**native P-256 + Dart DeviceSubkey 落地（2026-07-09，纯新增，未接入）**：
- Android `DeviceSubkeyBridge.kt`（通道 `org.citizenapp/device_subkey`：`publicKey`/`sign`/`delete`）：Keystore EC P-256 `PURPOSE_SIGN` **无 auth** 静默硬件 ECDSA；导出裸点 65B、返回平台 DER 签名。`MainActivity` 挂通道。
- Dart `lib/wallet/core/device_subkey.dart`：`DeviceSubkey`（publicKeyHex / signRaw / signRawHex / delete）+ `derEcdsaToRaw`（DER→裸 r||s，去符号 0 前导 / 左补）+ hex 工具。单测 `test/wallet/device_subkey_test.dart` **8/8**（DER 三形态 + 通道往返 + null 错误），analyze 干净。
- **未接入**：3 静默路径改子钥 + 钱包创建注册 + Step 3 是下一步，与 worker 部署同发。
- ⚠️ worker 全套 **71/72**：唯一 fail=`chain_confirm.test.ts` 存储回收（expected 1024 to be 0），**与本任务无关**——来自并发合入的 account-deletion/session-index 代码，非本 P-256 改动引起（不碰 posts/storage）；typecheck 干净、`device_subkey`/`auth` 全绿。

**Chat 路径范围澄清（重要）**：静默签名有两类，别搞混：
- **频繁**=广场 session 握手（`square_session_provider` / `square_compose_signers.signLogin` / im 的 `_signSquareLoginPayload`）→ 走 square `/auth/session`（已改 ES256）→ **改 P-256 子钥**（本轮目标）。
- **罕见**=Chat 设备绑定（`chat_runtime._signWalletPayload` → chat `registerChatDevice` → `src/chat/binding.ts` 结构化 sr25519 op_tag `OP_SIGN_CHAT_DEVICE_BIND`，缓存到期才重签）。它是 ADR-026 op_tag 钱包授权证明，**保持 sr25519**、step ③ 把它从 `requireAuth:false` 翻成 `true`（罕见，弹一次生物识别可接受）→ **chat/binding.ts 零改**。故本次 cutover 后端只动 `createSession`。

## Step 3 集成落地（2026-07-09，代码完成，⚠️未部署未 e2e）

全部 code-complete、`flutter analyze` 0 error、全套单测 **458 passed / 5 skipped / 0 failed（`--concurrency=1`，Isar 并行须串行，见 [[feedback_isar_is_community_fork]]）**：
- **WalletManager 切硬件金库**：`_store = HardwareBoundSeedVault()`；删 local_auth / `_requireBiometric` / `debugLocalAuth` / `signWithWallet(requireAuth)` 参数——「每次动钱动权验证」现由硬件金库读 seed 的**原子生物识别**实现；`verifyWalletAccess` / `getSeedHex` / `getMnemonic` 去软门禁。
- **子钥注册**：`typedef WalletSubkeyRegistrar` 注入钩子（`main.dart` 注入 `DeviceSubkeyRegistrar().register`，wallet/core 不反依赖 8964）；`createWallet` / `importWallet` best-effort 用**内存 keypair** 签绑定注册（零额外弹窗、失败不阻塞创建）。
- **3 静默路径改子钥**：`square_session_provider` / `square_compose_signers.signLogin` / `chat_runtime._signSquareLoginPayload` → `DeviceSubkey.signRawHex`；`SquareApiClient.ensureSession` 加 `onDeviceNotRegistered` 懒注册重试 + `registerDeviceSubkey`；新增 `DeviceSubkeyRegistrar`（8964/services）。
- **Chat 设备绑定**（罕见）保持 sr25519：`_signWalletPayload` 去 `requireAuth:false` → 读硬件金库弹一次。
- **部署完成（2026-07-09，用户授权 clean cutover）**：`migrate:production`（0008 `square_device_subkeys` 表 ✅）+ `deploy:production`（Version `ec94eb4e`）。**curl 验证生效**：`/health` ok；`register` 端点在线且校验（空→`invalid_owner_account`、缺 pubkey→`invalid_device_pubkey`）；`session` 对未注册子钥返回 `device_not_registered`（sr25519 旧登录已拒 = ES256 cutover 生效）。旧格式钱包需助记词重导入（clean cutover 代价，数据未毁、降级旧 App 仍可读）。
- **死码清理完成**：删 `lib/wallet/core/biometric_secure_seed_store.dart` + `test/wallet/biometric_secure_seed_store_test.dart`；`secure_seed_store.dart` 文档引用改 `[HardwareBoundSeedVault]`。**`local_auth` dep 保留**（main.dart / user.dart / create_wallet_onboarding 设备锁探测仍用）。analyze 干净。
- **待做**：iOS native P-256 + 硬件金库（**卡 Xcode 未装**：只有 CommandLineTools、无模拟器、无 CocoaPods；且模拟器无 Secure Enclave，真绑定需真 iPhone）；**Step 4 Android 真机 e2e**（新 App 已构建 102.6MB，装机时 **Pixel 掉线待重连**）：创建静默注册（查生产 D1 `square_device_subkeys` 确认）/ 广场登录静默 / 动钱动权弹验证 / 换指纹自愈 / 401 懒注册。

## 生产事故：后台狂弹生物识别 + 根因 + 修复（2026-07-09）

**症状**：部署 worker（ES256）+ 装新 App 后，后台每隔几秒弹一次生物识别，不停。
**根因（4 铁证定位）**：
1. 生产 D1 `square_device_subkeys` = 0 行（子钥从没注册上）。
2. `wrangler tail`：反复 `challenge`+`session`、`device/register` **0 次**。
3. logcat：后台每几秒读 `tier=strict`（严档 seed）→ 弹窗，decrypt SUCCESS。
4. 死循环：旧钱包（idx=1，旧格式/未注册）→ 后台会话 401 `device_not_registered` → **Step 3 加的懒注册在后台读 seed 弹验证** → 注册没成 → 又 401 → 又弹；多服务 × 重试 = 狂弹。
**Step 3 设计缺陷**：后台流程不该碰硬件 seed；懒注册在后台弹窗错误。**单测用 fake 没跑真实后台流，骗过了验证 → 没做 e2e 就部署+装机是判断失误。**
**修复**：删掉后台懒注册（`onDeviceNotRegistered` 从 `square_session_provider` / `chat_runtime` 移除）——后台**永不读 seed / 不弹 / 不懒注册**；子钥注册只在钱包创建时用**内存 keypair** 静默做；未注册钱包（旧格式）广场登不了但也不弹，**重建即注册**。
**真机验证**：新 App 静置 14s，后台零 `HW_SEED_VAULT` 读取 = **弹窗消失 ✅**。
**同期对齐（并行 agent 的 ADR-026 SCALE 迁移）**：签名统一走 `signing_message(op_tag)`（登录 0x1b / 设备绑定 0x1c），登录/绑定签名器参数改 `Uint8List` 摘要；`WalletSubkeyRegistrar.signBinding` 同步改 `Uint8List`。app 编译净、worker 92 测试绿、worker 部署 `6bf9ecd1`。
**协议端到端已验（2026-07-09 脚本对线上 worker）**：真 sr25519 主钥 + P-256 子钥跑 `register → challenge → session` **全 200**、换到 `session_token`（SCALE `signing_message` 逐字节对，测试行已清）。

**App 真机 e2e 已验（2026-07-09）**：新 App 内新建热钱包 → `HW_SEED_VAULT` 仅两条静默 encrypt（strict+recovery）、**创建 0 弹窗**；子钥**自动注册**（生产 D1 出现新行）；清 logcat 后至今**无任何自发后台弹窗**（循环已灭）；唯一一次弹窗 = 用户**切换默认钱包**（`verifyWalletAccess` 动权，设计内正确）。**结论：狂弹事故彻底修复，后台永久静默，仅用户动权时弹。** 遗留 Chat 设备绑定（罕见 sr25519，缓存 90 天）保持一次弹，属设计。
**教训铁律**：硬件金库/签名类改动，**真机 e2e 通过后才部署**，绝不靠 fake 单测就上线。

## Step 0 结果（2026-07-09 Android PASS）

Pixel 8a / Android 16（API 36，adb `3C071JEKB09000`）真机 spike **通过**。隔离验证包：Kotlin `SpikeBiometricVault` + 独立入口 `lib/spike_main.dart` + 通道 `org.citizenapp/spike_vault`（profile 包，生产 `main.dart` 零改动）。

**证实（logcat `SPIKE_VAULT`）：**
- 公钥加密**全程静默、零弹窗**（ctLen=256）→ 创建钱包静默成立。
- 私钥经 `BiometricPrompt.CryptoObject` 解密：连续 7 次**每次一验、每次成功**、round-trip 明文正确；**零 `KEY_USER_NOT_AUTHENTICATED`、零 `INCOMPATIBLE_MGF_DIGEST`**。「one attempt failed (not fatal)」= 指纹单次不匹配的正常重试。
- **结论**：自写桥 + CryptoObject 彻底规避 biometric_storage 历史踩坑；Android 地基坐实。

**跑通的关键参数（Step 1 转正照抄）：**
- KEK：RSA-2048，`PURPOSE_ENCRYPT|PURPOSE_DECRYPT`，`ENCRYPTION_PADDING_RSA_OAEP`，`DIGEST_SHA256`，`setUserAuthenticationRequired(true)`，API30+ `setUserAuthenticationParameters(0, AUTH_BIOMETRIC_STRONG)`（API24-29 降级 `setUserAuthenticationValidityDurationSeconds(-1)`），严档 `setInvalidatedByBiometricEnrollment(true)`。
- **OAEP 铁律（新踩坑，写死）**：变换 `RSA/ECB/OAEPPadding` + `OAEPParameterSpec("SHA-256","MGF1",MGF1ParameterSpec.SHA1,PSpecified.DEFAULT)` —— **MGF1 掩码摘要必须 SHA-1**（主摘要 SHA-256）。传 MGF1-SHA256 → keystore2 私钥操作抛 `INCOMPATIBLE_MGF_DIGEST(-78)`、根本走不到弹窗。加解密两端逐字节共用同一 spec。
- 公钥加密前用 `KeyFactory + X509EncodedKeySpec` 重建「无授权约束」公钥，避免公钥加密也要认证。
- 解密：`cipher.init(DECRYPT_MODE, priv, oaepSpec)` 不触发认证；`BiometricPrompt.CryptoObject(cipher)` 承载令牌，`onAuthenticationSucceeded` 里 `result.cryptoObject.cipher.doFinal(ct)` 原子解密。
- 依赖：`androidx.biometric:biometric:1.1.0`；`MainActivity` 已 `FlutterFragmentActivity`；manifest 加 `USE_BIOMETRIC`。

**iOS 端**：本机无 Xcode/iOS 真机 → Step 0 iOS（flutter_secure_storage `.biometryCurrentSet` 是否逐读弹 Face ID）**待有 Mac+iOS 设备再验**。

**Step 1 落地（2026-07-09，Android 生产桥，纯新增未切生产）**：
- 删 `SpikeBiometricVault.kt` / `lib/spike_main.dart`（spike 转正）。
- 新增原生桥 `HardwareSeedVaultBridge.kt`：通道 `org.citizenapp/hw_seed_vault`（`authStatus`/`encrypt`/`decrypt`/`deleteKey`），双档 `strict`(seed,仅生物,invalidatedByEnrollment=true)/`recovery`(助记词,生物或设备凭证,false) + 混合 AES-256-GCM DEK 信封（KEK RSA-OAEP wrap DEK，规避 24 词超块）。`MainActivity` 通道换生产。
- 新增 Dart `HardwareBoundSeedVault`（实现 `SecureSeedStore`；注入式 `VaultBlobStore`—默认 flutter_secure_storage 持久化 blob—便于单测；原生错误码→`SecureSeedException` 分类）、`FakeHardwareBoundSeedVault`、单测 `test/wallet/hardware_bound_seed_vault_test.dart`（fake 往返 + 错误映射 + tier/key 断言 + authStatus）。
- **未动 `WalletManager`**（`_store` 仍 `BiometricSecureSeedStore`，Step 3 才切、并去 local_auth 软门禁）；`androidx.biometric`/`USE_BIOMETRIC` 转正保留。
- 遗留开发 harness `lib/dev/hw_vault_harness.dart`（驱动**生产**路径真机验证严档/宽档两档，Step 3/4 e2e 后删）。
- **真机验证（Pixel 8a）通过**：两档静默写（strict blob 319B / recovery 362B 混合信封）；strict 读弹纯生物识别、recovery 读弹生物或设备凭证；decrypt 全 `SUCCESS`（AES-GCM 认证=round-trip 正确），零 `KEY_USER_NOT_AUTHENTICATED` / 零 `AEADBadTag` / 零 `INCOMPATIBLE_MGF`。Dart 17/17 单测绿、analyze 干净。
- **Step 1 完成**。下一不阻塞工作 = P-256 子钥后端（Cloudflare Worker，Step 3 前置）；iOS（Step 2）卡硬件。

## 测试
- 原生桥纯 Dart CI 测不了（无宿主实现）→ 走 `integration_test` 真机 + skip 守卫（参考 `smoldot_native_probe`）；Dart 封装层用 fake vault 单测错误分类与自愈。

## 备注
- **开发期铁律**（[[feedback_no_compatibility]] [[feedback_no_remnants]]）：彻底替换、无兼容、无数据迁移，无生物识别设备不能建热钱包（方案 A 沿用）。
- 关联 [[project_seed_biometric_binding_design]]（本卡是其"极致加固"续作，local_auth 档保留为无 CryptoObject 时的降级）。
- 工程量：~1-2 天原生 + 后端子钥（若选方案 1）。
