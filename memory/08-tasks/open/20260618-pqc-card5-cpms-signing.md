# PQC card5:CPMS 档案签名 ML-DSA-65 + KDF 收敛 + wallet_sig_alg 放开

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§1 §3)
状态:open(依赖 card1 gmb-pqc)

任务需求:
CPMS 后端自身档案签名密钥迁 ML-DSA-65 + 静态 KDF 收敛 + 放开钱包算法字段:
1. **ARCHIVE 档案签名 ML-DSA-65**:`cpms/backend/dangan/mod.rs:422-444`(`sign_archive_payload_with_secret`)+ keygen `initialize/mod.rs:759-768`(`generate_sr25519_keypair_raw`)换 fips204。**注意 ML-DSA 私钥 ~4032B / 公钥 ~1952B 撞穿 `[u8;32]` 假设**:`encrypt_secret &[u8;32]`(`initialize/mod.rs:59`)、`len()!=32` 守卫(`dangan/mod.rs:426`)、返回类型全改;签名源协议串 `sfid-cpms-v1|archive` 升版本号。
2. **master KDF 收敛 HKDF**:`initialize/mod.rs:50-57` `secret_cipher` 的 `Blake2b256(master‖key_id)` 单次 → HKDF-SHA512(`info=key_id`)。**无数据 = 无需 re-wrap**(创世起即 HKDF)。AES-256-GCM 不变(已是真 GCM)。
3. **geo_seal KDF 收敛**:`dangan/mod.rs:258-263` `derive_geo_seal_key` 的 Blake2b 单次 → HKDF;geo_seal 即时重签,无需 re-wrap。
4. **wallet_sig_alg 放开 ml-dsa-65**(公民钱包算法声明字段):写入 `routes.rs:348/689/1800`、校验放开 `export.rs:233-234` + `routes.rs:1042`(漏一处会静默拒签);DB 列宽核对。
5. **DB schema 审计**:ML-DSA 大密钥 / 签名对 cpms 私钥存储列、`archives.wallet_sig_alg` 容纳能力。
6. **历史 ARCHIVE**:走现有 `clear_archive_qr_payload` + 重签机制(ARCHIVE 本就资料变更即重签),非数据迁移。

所属模块:CPMS 后端

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(P-CRED-002 ARCHIVE)
- cpms 模块完成标准

必须遵守:
- CPMS 不托管用户钱包私钥(只记录地址/公钥)。
- 密钥隔离:CPMS-ARCHIVE 独立 OsRng 密钥域,不走 AccountSeedV1。
- 不搞兼容双轨;wallet_sig_alg 写入 / 校验同步放开,零遗漏。
- 协议串 / AAD 升版本号区分新旧载荷。

输出物:
- ARCHIVE 签名 / KDF / wallet_sig_alg 改造 + DB 列宽核对 + 中文注释 + 测试 + 文档 + 残留清理

验收标准:
- ARCHIVE ML-DSA-65 签发,被 SFID 应用层验签通过;`[u8;32]` 硬编码全解除。
- master/geo_seal 走 HKDF;wallet_sig_alg 接受 ml-dsa-65(写入+导出+删除校验三路一致)。
- DB 列宽容纳 ML-DSA;真实运行态(签发档案 / 扫码 / 导出)验收。
