# PQC card4:SFID 系统签名 ML-DSA-65 + 钱包绑定归属验证(查链)+ 跨系统验签器

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§9/§15 决策5/8;§3 §6)
状态:open(依赖 card1;**MAIN signer 迁移须与 card2 原子同批上线**)

任务需求:
1. **SFID MAIN signer ML-DSA-65**:`crypto/sr25519.rs` 单点(签登录 QR `admins/login/signature.rs`、CPMS 安装授权 `cpms/handler.rs:1410`)换 fips204。MAIN signer 当前 env `SFID_SIGNING_SEED_HEX` 裸 seed,经 HKDF+domain 派生 ML-DSA(🔴 **仅 ML-DSA 分支套 HKDF;sr25519 阶段绝不套,否则链上 ShengSigningPubkey 登记公钥漂移**)。
   - 🔴 **(B6)硬依赖**:MAIN/sheng 系统签名产出的凭证被链端 `configs/mod.rs:781/890/961/1037` sr25519_verify 验证 → **MAIN signer 切 ML-DSA 必须与 card2 §5/§6(验签器 algo-tag + ShengSigningPubkey value→BoundedVec)原子同批上线**,或链端先双 algo 路由再切;否则全省机构注册/投票/人口快照全红。
2. 🔴 **(B5)SFID 钱包绑定归属验证 = §9 P0 唯一真实落点(被原 card 漏列)**:`citizens/binding.rs:79/340/878` 当前硬拒 wallet_sig_alg!=sr25519、固定 `[u8;32]`、公钥从 SS58 地址反推。改:`ml-dsa-65` 时(a)QR 单独带 ML-DSA 公钥(不再 ss58 反推);(b)**经 subxt 查链 `AccountPqcKey[wallet_address]` 确认该 ML-DSA 公钥已绑到此 sr25519 锚点**(决策8 唯一权威源);(c)gmb-pqc verify_by_algo 验签;(d)**未完成 bootstrap → 拒**。`verify_citizen_bind_signature`/`resolve_bind_wallet` 脱离 `[u8;32]`。
3. 🔴 **(H10)SFID 侧 ARCHIVE 验签器** `cpms/handler.rs:986-1015` `verify_cpms_archive_qr` 硬编码 sr25519,card5 改 CPMS 签名后会全红 → 本卡增列改造:按 ARCHIVE 协议串算法标识走 verify_by_algo;`cpms_pubkey` 解析脱离 32B。
4. 🔴 **(H9)系统签名原文含 sig_alg**:`core/qr/mod.rs:118-141` `build_signature_message` 把 sig_alg 纳入签名 preimage(防算法混淆/降级),验签先按原文 alg 选验签器;**algo tag 进签名原文,不只进 meta**。
5. 🔴 **(决策5)CPMS install_sig 真实验证**:install_sig 由 SFID MAIN 签发,当前全系统无验证方(任何人投递 install QR 即可初始化 CPMS)。补真实验证(CPMS 启动用 SFID MAIN 公钥验,见 card5);本卡保证 SFID 侧签发口径与验证方一致。
6. 激活 SFID 死 `hkdf=0.12` 依赖;协议登记(P-CRED/登录QR/安装授权)按 algo tag 升级。

> 链上机构注册验签器(`configs/mod.rs`)在 **card2**;本卡是 SFID 应用层签名/验签 + 钱包绑定归属(查链)。

所属模块:SFID 后端

输入文档:ADR-022(§9/§15)/ unified-protocols / sfid 完成标准

必须遵守:SFID 不托管用户钱包私钥;密钥隔离;归属验证统一查链 AccountPqcKey(与 CPMS 同口径,CPMS 下沉本系统);MAIN signer 与 card2 原子上线;改协议字段先更 unified-protocols。

输出物:MAIN signer/验签库 + binding.rs 归属验证(subxt 查链)+ verify_cpms_archive_qr + build_signature_message 含 sig_alg + install_sig 验证口径 + 中文注释 + 测试 + 文档 + 残留清理。

验收标准:
- MAIN signer ML-DSA-65 与 card2 原子上线,机构注册/投票/人口快照验签通过(不全红)。
- `ml-dsa-65` 钱包绑定:查链 AccountPqcKey 证明归属、未 bootstrap 拒;`[u8;32]`/ss58 反推清零。
- verify_cpms_archive_qr 走 verify_by_algo;build_signature_message 含 sig_alg 进 preimage;install_sig 被真实验证(伪造 install QR 被拒);真实运行态(登录/绑定/安装授权)验收。
