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

**现有后端**（`citizenapp/cloudflare`，唯一 production API `https://www.crcfrcn.com/api`）：
- `POST /v1/square/auth/challenge` → 最新 finalized 双向绑定先解析 CID，再将
  `account_id ‖ cid_number ‖ challenge_id ‖ expires_at` SCALE payload 存入 D1
  `square_login_challenges`；客户端固定走 `OP_SIGN_SQUARE_LOGIN` 摘要，不存在字符串域。
- `POST /v1/square/auth/session` → 按 `(cid_number, device_id)` 读取登记的 P-256 公钥，
  使用 Workers Web Crypto ES256 验证设备请求；会话同时保存 CID 与签发时 `account_id`。
- 路由手写于 `src/routes.ts`；migrations 顺序编号 `0001..0007`；Env 有 `DB:D1Database` / `FEED_CACHE:KVNamespace`。

**协议**：
1. **子钥**：per-wallet P-256（Keystore `PURPOSE_SIGN` 无 auth / iOS SE）。
2. **绑定（首次进入需 CID 功能时一次性执行）**：当前绑定账户对
   `signing_message(OP_SIGN_SQUARE_DEVICE_BIND, account_id ‖ p256_public_key ‖ issued_at)`
   签名，调用 `POST /v1/square/auth/device/register`；Worker 验签并从 finalized 双向绑定
   解析 `cid_number` 后落库。建钱包时尚无 CID，禁止提前注册。
3. **握手（静默 P-256）**：challenge 不变；client 用 P-256 子钥签 `signing_payload`（静默）；`session` 查该 owner 已注册 p256_pubkey → **Web Crypto ES256** 验（`subtle.verify({name:ECDSA,hash:SHA-256})`）；无绑定 → 401 `device_not_registered` → client 注册后重试。
4. **格式**：pubkey=裸未压缩点 65B(`0x04||X||Y`) hex；sig=裸 `r||s` 64B hex（client 把平台 DER→raw）。
5. **D1**：`square_device_subkeys` 以 `(cid_number, device_id)` 为主键，保存当次绑定
   `account_id`、P-256 公钥与时间；同一身份允许多设备，换绑后旧账户由 guard 失效。
6. **client 接入**：广场与 Chat 静默登录统一使用 `DeviceSubkey.signRawHex`；Android
   Keystore 与 iOS Secure Enclave 原生桥均已实现。

**当前决策**：clean cutover；设备子钥只在首次进入需 CID 功能时经
`IdentityRegistrationGate` 绑定。后台登录遇 `device_not_registered` 直接失败，禁止后台
读取账户 child 或弹出生物识别。

**后端实现（当前基线）**：
- `migrations/0001_square_core.sql`：`square_device_subkeys(cid_number, device_id, account_id, ...)`。
- `src/auth/device_subkey.ts`：`buildDeviceBindingSigningMessage` 使用唯一 op_tag 摘要；
  `assertP256PublicKeyHex` 校验 65B 裸点，`verifyP256Signature` 使用 Web Crypto ES256。
- `src/auth/service.ts`：`registerDeviceSubkey`（sr25519 验绑定证明 → upsert）；`createSession` 验签 **sr25519 → ES256**（查子钥，无则 401 `device_not_registered`）。
- `src/routes.ts` 挂 `POST /v1/square/auth/device/register`；`types.ts` 加 `DeviceSubkeyRow`。
- 测试 `test/device_subkey.test.ts` 5 例（ES256 往返/0x前缀/篡改/畸形/pubkey 校验），全套 **72/72 绿**、typecheck 干净。**未 `wrangler deploy`、未 apply migration**——等 App 端就绪，与 App 发布**同步 clean cutover**（deploy 属对外操作，需用户明确许可）。

**客户端当前状态**：Android/iOS 原生 P-256、Dart `DeviceSubkey`、三处静默登录、
CID 页面懒绑定和 `HardwareBoundSeedVault` 均已接通；不存在钱包创建注册或后台懒注册分支。

**native P-256 + Dart DeviceSubkey 落地记录（2026-07-09）**：
- Android `DeviceSubkeyBridge.kt`（通道 `org.citizenapp/device_subkey`：`publicKey`/`sign`/`delete`）：Keystore EC P-256 `PURPOSE_SIGN` **无 auth** 静默硬件 ECDSA；导出裸点 65B、返回平台 DER 签名。`MainActivity` 挂通道。
- Dart `lib/wallet/core/device_subkey.dart`：`DeviceSubkey`（publicKeyHex / signRaw / signRawHex / delete）+ `derEcdsaToRaw`（DER→裸 r||s，去符号 0 前导 / 左补）+ hex 工具。单测 `test/wallet/device_subkey_test.dart` **8/8**（DER 三形态 + 通道往返 + null 错误），analyze 干净。
- 三处静默路径与 CID 页面绑定门禁现均已接入；钱包创建路径明确不注册设备子钥。
- ⚠️ worker 全套 **71/72**：唯一 fail=`chain_confirm.test.ts` 存储回收（expected 1024 to be 0），**与本任务无关**——来自并发合入的 account-deletion/session-index 代码，非本 P-256 改动引起（不碰 posts/storage）；typecheck 干净、`device_subkey`/`auth` 全绿。

**Chat 路径范围澄清（重要）**：静默签名有两类，别搞混：
- **频繁**=广场 session 握手（`square_session_provider` / `square_compose_signers.signLogin` / im 的 `_signSquareLoginPayload`）→ 走 square `/auth/session`（已改 ES256）→ **改 P-256 子钥**（本轮目标）。
- **罕见**=Chat 设备绑定（`chat_runtime._signWalletPayload` → chat `registerChatDevice` → `src/chat/binding.ts` 结构化 sr25519 op_tag `OP_SIGN_CHAT_DEVICE_BIND`，缓存到期才重签）。它是 ADR-026 op_tag 钱包授权证明，**保持 sr25519**、step ③ 把它从 `requireAuth:false` 翻成 `true`（罕见，弹一次生物识别可接受）→ **chat/binding.ts 零改**。故本次 cutover 后端只动 `createSession`。

## Step 3 集成落地（当前状态）

全部 code-complete、`flutter analyze` 0 error、全套单测 **458 passed / 5 skipped / 0 failed（`--concurrency=1`，Isar 并行须串行，见 [[feedback_isar_is_community_fork]]）**：
- **WalletManager 切硬件金库**：`_store = HardwareBoundSeedVault()`；删 local_auth / `_requireBiometric` / `debugLocalAuth` / `signWithWallet(requireAuth)` 参数——「每次动钱动权验证」现由硬件金库读 seed 的**原子生物识别**实现；`verifyWalletAccess` / `getSeedHex` / `getMnemonic` 去软门禁。
- **子钥注册**：`DeviceSubkeyRegistrar` 仅由 CID 功能门禁触发；钱包创建与导入只保存
  账户 child，不调用注册器。
- **3 静默路径改子钥**：`square_session_provider` / `square_compose_signers.signLogin` /
  `chat_runtime._signSquareLoginPayload` → `DeviceSubkey.signRawHex`；后台不存在
  `device_not_registered` 懒注册重试。
- **Chat 设备绑定**（罕见）保持 sr25519：`_signWalletPayload` 去 `requireAuth:false` → 读硬件金库弹一次。
- 当前创世基线与客户端只保留 `account_id + cid_number + device_id` 契约，不保留历史字段、
  历史字符串签名域、旧钱包双读或兼容登录。
- **死码清理完成**：删 `lib/wallet/core/biometric_secure_seed_store.dart` + `test/wallet/biometric_secure_seed_store_test.dart`；`secure_seed_store.dart` 文档引用改 `[HardwareBoundSeedVault]`。**`local_auth` dep 保留**（main.dart / user.dart / create_wallet_onboarding 设备锁探测仍用）。analyze 干净。
- 2026-07-29 已完成 Android 隔离 instrumentation（设备子钥删除后重建公钥不同）及
  iOS 16 arm64 模拟器完整构建；按用户要求不执行 iPhone 真机验收。

## 生产事故：后台狂弹生物识别 + 根因 + 修复（2026-07-09）

**症状**：部署 worker（ES256）+ 装新 App 后，后台每隔几秒弹一次生物识别，不停。
**根因（4 铁证定位）**：
1. 生产 D1 `square_device_subkeys` = 0 行（子钥从没注册上）。
2. `wrangler tail`：反复 `challenge`+`session`、`device/register` **0 次**。
3. logcat：后台每几秒读 `tier=strict`（严档 seed）→ 弹窗，decrypt SUCCESS。
4. 死循环：旧钱包（idx=1，旧格式/未注册）→ 后台会话 401 `device_not_registered` → **Step 3 加的懒注册在后台读 seed 弹验证** → 注册没成 → 又 401 → 又弹；多服务 × 重试 = 狂弹。
**Step 3 设计缺陷**：后台流程不该碰硬件 seed；懒注册在后台弹窗错误。**单测用 fake 没跑真实后台流，骗过了验证 → 没做 e2e 就部署+装机是判断失误。**
**修复**：删掉后台懒注册；后台永不读取账户 child、不弹生物识别。子钥绑定只允许在
用户主动进入需 CID 功能时由门禁完成，失败则停留在门禁并明确报错。
**真机验证**：新 App 静置 14s，后台零 `HW_SEED_VAULT` 读取 = **弹窗消失 ✅**。
**同期对齐（并行 agent 的 ADR-026 SCALE 迁移）**：签名统一走 `signing_message(op_tag)`（登录 0x1b / 设备绑定 0x1c），登录/绑定签名器参数改 `Uint8List` 摘要；`WalletSubkeyRegistrar.signBinding` 同步改 `Uint8List`。app 编译净、worker 92 测试绿、worker 部署 `6bf9ecd1`。
**协议端到端已验（2026-07-09 脚本对线上 worker）**：真 sr25519 主钥 + P-256 子钥跑 `register → challenge → session` **全 200**、换到 `session_token`（SCALE `signing_message` 逐字节对，测试行已清）。

**运行态结论**：创建钱包只静默写账户 child 信封，不注册设备子钥；后台无任何账户
child 读取或生物识别弹窗。首次进入需 CID 功能时由用户前台完成一次绑定。
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

**iOS 端更新（2026-07-29）**：Xcode 27 beta 与 CocoaPods 已就绪，CitizenApp 与
CitizenWallet 最低版本统一为 iOS 16.0。最终方案不复用 flutter_secure_storage 软桥，
已实现 Secure Enclave P-256 ECIES 严档 KEK 和 P-256 设备子钥两条原生通道；
CitizenApp `arm64` 模拟器目标完整构建通过。本任务不执行 iPhone 真机验收，
Apple 开发者账户不属于当前完成条件。

**Step 1 落地（2026-07-09，Android 生产桥，纯新增未切生产）**：
- 删 `SpikeBiometricVault.kt` / `lib/spike_main.dart`（spike 转正）。
- 新增原生桥 `HardwareSeedVaultBridge.kt`：通道 `org.citizenapp/hw_seed_vault`（`authStatus`/`encrypt`/`decrypt`/`deleteKey`），双档 `strict`(seed,仅生物,invalidatedByEnrollment=true)/`recovery`(助记词,生物或设备凭证,false) + 混合 AES-256-GCM DEK 信封（KEK RSA-OAEP wrap DEK，规避 24 词超块）。`MainActivity` 通道换生产。
- 新增 Dart `HardwareBoundSeedVault`（实现 `SecureSeedStore`；注入式 `VaultBlobStore`—默认 flutter_secure_storage 持久化 blob—便于单测；原生错误码→`SecureSeedException` 分类）、`FakeHardwareBoundSeedVault`、单测 `test/wallet/hardware_bound_seed_vault_test.dart`（fake 往返 + 错误映射 + tier/key 断言 + authStatus）。
- **未动 `WalletManager`**（`_store` 仍 `BiometricSecureSeedStore`，Step 3 才切、并去 local_auth 软门禁）；`androidx.biometric`/`USE_BIOMETRIC` 转正保留。
- 遗留开发 harness `lib/dev/hw_vault_harness.dart`（驱动**生产**路径真机验证严档/宽档两档，Step 3/4 e2e 后删）。
- **真机验证（Pixel 8a）通过**：两档静默写（strict blob 319B / recovery 362B 混合信封）；strict 读弹纯生物识别、recovery 读弹生物或设备凭证；decrypt 全 `SUCCESS`（AES-GCM 认证=round-trip 正确），零 `KEY_USER_NOT_AUTHENTICATED` / 零 `AEADBadTag` / 零 `INCOMPATIBLE_MGF`。Dart 17/17 单测绿、analyze 干净。
- **Step 1 完成**；Android 与 iOS 原生桥、Worker 子钥协议和删除生命周期均已接通。

## 测试
- Dart 封装层使用 fake vault 覆盖错误分类、账户信封与删除；Android 使用隔离
  instrumentation 直接验证 Keystore 子钥生成、删除和重建。iOS 使用 iOS 16 arm64 模拟器
  完整编译原生通道；本任务不做 iPhone 真机验收。

## 钱包删除生命周期收口（2026-07-29）

- `WalletManager.deleteWallet`、删除末账户的级联路径和 `clearWallet` 现统一销毁热钱包
  `walletIndex` 对应的 Android Keystore / iOS Secure Enclave P-256 设备子钥；删除非末
  账户与冷钱包不误删共享子钥。
- 同一删除生命周期同时清除账户 child、通讯录当前/旧命名密钥、LDK 信封与静默缓存、
  钱包 KEK。每项独立尝试，全部完成后以 `WalletLocalCleanupException` 汇总失败，避免
  首个安全存储错误留下其余密钥。
- 现有 WalletManager 测试覆盖整钱包、非末账户、全量清空、冷热钱包隔离及多项失败仍
  继续清理；账户 child 信封键 clean cutover 为 `account_child_key_{account_id}`，不双读
  旧键。最终统一测试与原生构建结果见本卡测试段及广场清理任务第 7 步记录。

## 备注
- **开发期铁律**（[[feedback_no_compatibility]] [[feedback_no_remnants]]）：彻底替换、无兼容、无数据迁移，无生物识别设备不能建热钱包（方案 A 沿用）。
- 关联 [[project_seed_biometric_binding_design]]；硬件私钥操作不提供软件密钥降级路径。
