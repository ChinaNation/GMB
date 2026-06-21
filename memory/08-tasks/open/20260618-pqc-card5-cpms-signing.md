# PQC card5:CPMS 档案签名 ML-DSA-65 + wallet_sig_alg 放开 + install_sig 验证(归属下沉 CID)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§9/§13/§15 决策5/8)
状态:open(依赖 card1)

任务需求:
1. **ARCHIVE 档案签名 ML-DSA-65**:`dangan/mod.rs:422-444`(`sign_archive_payload_with_secret`)+ keygen `initialize/mod.rs:759-768` 换 fips204。🔴 **(H11)`[u8;32]` 撞穿点全列**:`encrypt_secret`/`decrypt_secret`(`initialize/mod.rs:59/73`)签名改 `&[u8]` 通用长度;`generate_sr25519_keypair_raw→[u8;32]`、`dangan/mod.rs:426 len!=32` 守卫、所有 `len()==32` 隐式断言移除;**install_secret(32B)与 ARCHIVE 私钥(~4032B)存储路径拆清或统一变长**;签名源协议串升版本号。
2. 🔴 **(B7/决策8)CPMS 无链客户端(无 subxt/jsonrpsee/reqwest),不验钱包签名**(P-CRED-002 本就只存不验):**ML-DSA 钱包签名归属验证一律下沉 CID(经 subxt 查链 AccountPqcKey)**;card5 不实现①查链/②状态证明(物理不可行),只保留③"bootstrap 前由 CID 拒"。删 card 里"CPMS 验用户钱包 ML-DSA 签名"伪命题表述。
3. **wallet_sig_alg 放开(声明字段)**:🔴 **(H10)用符号/函数名定位,不用过时行号**——写入 `routes.rs` 新建 ARCHIVE / `save_archive_wallet`(实际硬写 `'sr25519'`)接收并落库实际 alg;`WalletBindRequest` 增 `sig_alg`/ML-DSA pubkey 入参;读取点补全(`routes.rs` 读、`admins/mod.rs`、`export.rs`);校验三路(写/导出/删)一致,漏一处静默拒签。
4. 🔴 **(决策5)install_sig 真实验证**:CPMS 启动用 CID MAIN 公钥真正验 `system_install.install_sig`(`initialize/mod.rs:384` 当前只存不验)——补"任何人投递 install QR 即可初始化 CPMS"安全洞;算法随 CID MAIN 迁 ML-DSA(card4)按 verify_by_algo。
5. **KDF 收敛**:`secret_cipher`(`initialize/mod.rs:50` Blake2b 单次)+ `derive_geo_seal_key`(`dangan/mod.rs:258`)→ HKDF-SHA512;🔴 **info 带固定域前缀(非裸 key_id)**,与 card1 domain `[u8;N]` 体系统一;geo_seal IKM 走 HKDF-Extract;AES-256-GCM 不变(已是真 GCM)。
6. **DB 列宽**:pubkey/secret/wallet_pubkey/archive_qr_payload 均 Postgres TEXT(非定长),容 ML-DSA hex,确认无需改 schema。
7. 历史 ARCHIVE 走现有 `clear_archive_qr_payload` 重签机制(资料变更即重签),非数据迁移。

所属模块:CPMS 后端

输入文档:ADR-022(§9/§13/§15)/ unified-protocols(P-CRED-002)/ cpms 完成标准

必须遵守:CPMS 不托管钱包私钥、不验钱包签名(归属下沉 CID);密钥隔离;wallet_sig_alg 写/读/校验同步零遗漏;协议串/AAD 升版本号。

输出物:ARCHIVE 签名(脱离[u8;32])+ wallet_sig_alg 放开(符号定位)+ install_sig 验证 + KDF→HKDF(域前缀)+ 中文注释 + 测试 + 文档 + 残留清理。

验收标准:
- ARCHIVE ML-DSA-65 签发被 CID `verify_cpms_archive_qr`(card4)验过;`[u8;32]`/`len==32` 断言全解除(含 encrypt_secret/decrypt_secret)。
- wallet_sig_alg 接受 ml-dsa-65(写+导出+删三路一致);install_sig 被真实验证(伪造 install QR 被拒)。
- master/geo_seal 走 HKDF(域前缀);DB TEXT 容 ML-DSA;真实运行态(签发/扫码/导出/安装)验收。
