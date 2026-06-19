# PQC card4:SFID 系统签名 ML-DSA-65 + KDF 收敛

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§1 §3)
状态:open(依赖 card1 gmb-pqc;链上验签器在 card2)

任务需求:
SFID 后端自身签名密钥(非用户钱包)迁 ML-DSA-65 + KDF 收敛:
1. **MAIN signer ML-DSA-65**:`sfid/backend/crypto/sr25519.rs:12-16/40-52` 单点(签登录 QR `admins/login/signature.rs:51-75`、CPMS 安装授权 `cpms/handler.rs:1410-1418`)换 fips204;凭证 `meta.alg` 由 `sr25519` 升 `ml-dsa-65`(0x02)。
   - 注意:MAIN signer 当前是 `env SFID_SIGNING_SEED_HEX` **裸 seed 无 KDF**(`chain_runtime.rs:339-364`),纳入 HKDF+domain 体系。
2. **应用层验签换库**:`login/signature.rs:13-49`、`cpms/handler.rs:1420-1438` 的 schnorrkel verify 换 ML-DSA(fips204 / 对应实现)。
3. **激活死 hkdf 依赖**:`sfid/backend/Cargo.toml:42` hkdf=0.12 当前零 import,接入实际 HKDF-SHA512 派生。
4. **登录 QR / CPMS 安装授权凭证补 algo tag**(当前 login-QR / cpms-install 签名源无算法标识)。
5. 协议登记:`unified-protocols.md` P-CRED / 登录 QR / 安装授权签名算法字段按 algo tag 升级。

> 链上机构注册验签器在 **card2**(`configs/mod.rs` algo-tag 路由)。本卡是 SFID **应用层**签名 / 验签。
> **过时记忆已订正**:`feedback_sfid_sheng_signing_keyring` 的"省级 wrap + 级联重加密"在 ADR-008 P23e 已删,现为单一 MAIN signer。

所属模块:SFID 后端

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md
- sfid 模块完成标准

必须遵守:
- SFID 不托管用户钱包私钥(边界铁律)。
- 密钥隔离:SFID MAIN 独立密钥域,不走 AccountSeedV1。
- 不搞兼容双轨;algo tag 统一升级,旧 sr25519 路径收敛。
- 改协议字段前先更 unified-protocols 登记。

输出物:
- MAIN signer / 验签 / KDF 改造 + 中文注释 + 测试(签发 / 跨端验签 fixture)+ 文档 + 残留清理

验收标准:
- MAIN signer ML-DSA-65 签发,登录 QR / CPMS 授权被对应验签方通过。
- HKDF 实际接入(死依赖激活);凭证 algo tag 落地。
- 残留 sr25519 / 裸 seed 路径收敛;协议登记更新;真实运行态(登录 / 安装授权)验收。
