# SFID / CPMS 独立签名域抗量子迁移(sr25519 → ML-DSA-65)

- 状态:open(需求已明确,设计未出,代码未启动)
- 创建日期:2026-06-07
- 关联决策:`memory/04-decisions/ADR-016-account-key-pqc-migration.md`(钱包账户密钥迁移,本卡的姊妹范围)
- 拟产出:`ADR-017 系统签名域抗量子迁移` + 各模块设计文档

## 任务需求

把 SFID、CPMS 各自持有的、**不属于用户钱包账户**的独立 sr25519 签名密钥迁移到后量子签名 ML-DSA-65。这是 ADR-016 没有覆盖的"第二套签名域"——ADR-016 只管用户钱包账户签名,本卡管系统自身的签名密钥。两个系统一起规划、一起实现。

## 范围边界(与 ADR-016 区分)

- ADR-016(范围 A):用户钱包账户签名 → runtime + node 清算行热钱包 + wuminapp + wumin。
- 本卡(范围 B):系统自身签名密钥 → SFID 省级签名 / SFID main signer / CPMS 本机档案密钥。
- 两者共用底座:`fips204` crate(链端 WASM 验签)、`gmb-pqc` 派生/签名约定、HKDF domain 口径。

## 当前代码事实(只读核实结论)

### SFID
1. **省级签名密钥** `ShengSigningPubkey`(sr25519):签机构注册信息凭证(registration-info)。
   - 链上**验签**:`citizenchain/runtime/src/configs/mod.rs:759-779`(`sr25519_verify`,payload = `DUOQIAN || OP_SIGN_INST || genesis_hash || sfid_number || institution_name || account_names || nonce || province || signer_admin_pubkey`)。
   - 后端密钥:AES-256-GCM wrap,wrap key 来自 SFID MAIN HKDF,按省验签、轮换级联重加密(见 `feedback_sfid_sheng_signing_keyring`)。
   - **这是链上验签的非账户签名,抗量子迁移必须改链上验签器**。
2. **SFID main signer**(env `SFID_SIGNING_SEED_HEX`,sr25519,`sfid/backend/crypto/sr25519.rs:12-22`):签登录 QR(`sfid/backend/admins/login/signature.rs:51-75`)+ CPMS 安装授权(`sfid/backend/cpms/handler.rs:1267-1275`)。**SFID 应用层验签,不在链上**。
3. SFID **不**触碰用户钱包私钥(边界已在 `CRYPTO_TECHNICAL.md §4` 落档)。

### CPMS
1. **本机档案签名密钥**(sr25519/schnorrkel,32B seed,AES-GCM 加密存 PostgreSQL,`CPMS_KEY_ENCRYPT_SECRET`,`cpms/backend/initialize/mod.rs:30-112`):签 ARCHIVE 凭证(`cpms/backend/dangan/mod.rs:250-263,362-384`)。
   - 验签在 **SFID 应用层**(`sfid/backend/cpms/handler.rs:1277-1295`),不在链上。
2. ARCHIVE 凭证里的 `wallet_sig_alg` / `wallet_pubkey` 是**记录用户钱包的算法/公钥**(`cpms/backend/dangan/mod.rs:50-66`),不是 CPMS 自己的签名算法——用户迁移后此字段需放开接受 `ml-dsa-65`。
3. CPMS **不**触碰用户钱包私钥(只记录地址/公钥供绑定)。

## 目标方案(待 ADR-017 定稿)

1. **SFID 省级签名**:省级密钥派生/签名换 ML-DSA-65;链上机构注册验签器(`configs/mod.rs:759-779` + `RuntimeSfidInstitutionVerifier`)按 algo tag 分流,支持 ML-DSA-65 验签;后端 wrap/轮换流程带 algo 版本。
2. **SFID main signer**:登录 QR / CPMS 安装授权签名换 ML-DSA-65;SFID 应用层验签库换 fips204/对应 Dart 实现。
3. **CPMS 本机档案密钥**:档案签名换 ML-DSA-65;SFID 端验签同步;`wallet_sig_alg` 放开 `sr25519 | ml-dsa-65`。
4. 协议登记:registration-info credential(P-CRED-001)、ARCHIVE(P-CRED-002 / P-CPMS-001)、登录 QR 的签名算法字段全部按 algo tag 升级登记。
5. 与 ADR-016 共用 `fips204` / `gmb-pqc` / HKDF domain 口径,避免第二套实现。

## 必须遵守

- 不与 ADR-016 的用户钱包账户密钥混淆;本卡是系统自身签名密钥。
- 链上机构注册验签器迁移必须 bump `spec_version` 并走 setCode(chainspec 创世冻结)。
- 不搞兼容双轨;按 algo tag 统一升级,旧 sr25519 验签路径在切换完成后收敛。
- 改协议字段前先更新 `unified-protocols.md` 登记项。

## 待确认问题

- 是否与 ADR-016 同期实现,还是 ADR-016 钱包侧先行、本卡随后。
- 省级签名密钥后端 wrap/轮换如何携带 algo 版本与级联重加密。
- CPMS 本机密钥迁移后,历史已签 ARCHIVE 是否需要重签或只对新档案生效。
- 应用层(SFID/CPMS 后端 Rust + 可能的前端 TS)ML-DSA 验签库选型(fips204 Rust + 何种 TS/WASM 绑定)。

## 验收标准

- SFID 省级签名、main signer、CPMS 档案密钥均可用 ML-DSA-65 签发并被对应验签方(链上 / SFID 应用层)验证通过。
- 链上机构注册验签器按 algo tag 正确分流,spec_version 已 bump。
- ARCHIVE `wallet_sig_alg` 接受 `ml-dsa-65`。
- 协议登记项已更新;文档已回写;残留 sr25519 硬编码已清理。
- 对应单测 / fixture / 跨端验签测试通过。
