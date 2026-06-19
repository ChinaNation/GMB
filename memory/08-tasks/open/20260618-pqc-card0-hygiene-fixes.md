# PQC card0:静态加密卫生修复(与 PQC 算法解耦,可立即先行)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§6 C2)
状态:open

任务需求:
修复 4 处与 PQC 算法无关、但属统一方案"对称/KDF 基线"的现存弱加密 / 不一致,可独立先行、完全可回滚:
1. **node 清算行假 AES-GCM(CRITICAL)**:`citizenchain/node/src/transaction/offchain_transaction/settlement/keystore.rs:181-220` 实为 `XOR(blake2 keystream)+截断 16B blake2 tag`,`derive_key:169-177` 仅 100 轮 blake2(`PBKDF2_ITERATIONS/1000`)。换真 `Aes256Gcm`(node `Cargo.toml` 加 `aes-gcm`)+ `Argon2id`(口令派生),**改掉说谎的函数名 `encrypt_aes256_gcm` 与 doc 注释(line 3/88)**。
2. **App 锁 KDF 统一**:`wumin/lib/security/app_lock_service.dart:148`(PBKDF2 1_000_000)与 `wuminapp/lib/security/app_lock_service.dart:166`(PBKDF2 100_000)收敛到 **Argon2id** 统一参数。
3. **热钱包 at-rest 对齐**:`wuminapp` 助记词无应用层加密(只靠 secure_storage),补一层 AES-256-GCM,对齐 `wumin/lib/wallet/mnemonic_cipher.dart`。
4. **IM 对称达标**:`wuminapp/rust/src/im_mls.rs:27-28` AES-128 → 256-bit 经典套件(先满足 Grover/AES-256 基线,ML-KEM 留 card6)。

所属模块:Blockchain(node)+ Mobile(wuminapp/wumin)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/03-security/security-rules.md
- 对应模块技术文档与完成标准

必须遵守:
- 不引入 PQC 算法(纯卫生修复)。
- 不碰链 / 共识 / 创世。
- 密钥隔离:清算行密钥不与其它角色合并。
- 不留旧 XOR 实现、不留说谎注释。

输出物:
- 代码 + 中文注释 + 测试(加解密往返 / KDF 参数)+ 文档更新 + 残留清理

验收标准:
- 清算行落盘真 AES-256-GCM 往返通过;函数名 / 注释与实现一致(谎言清除);node 加 aes-gcm 依赖。
- App 锁 KDF 统一 Argon2id,参数一致。
- 热钱包助记词应用层 AES-256-GCM 落地。
- IM 套件 ≥256-bit。
- 对应单测通过,真实运行态验收(清算行解锁 / App 锁 / 钱包恢复 / IM 收发)。
