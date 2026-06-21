# PQC card0:静态加密卫生修复(与 PQC 算法解耦,可立即先行)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§6 C2)
状态:open

任务需求:
修复 3 处与 PQC 算法无关、但属"对称/KDF 基线"的现存弱加密(**IM 套件升级移到 card6**,决策6:card0 不动 IM):
1. **node 清算行假 AES-GCM(CRITICAL)**:`...settlement/keystore.rs:181-220` 实为 `XOR(blake2 keystream)+截断 16B blake2 tag`,`derive_key:169-177` 仅 100 轮 blake2。换真 `Aes256Gcm`(node `Cargo.toml` 加 `aes-gcm`)+ `Argon2id`,改掉说谎函数名/注释;🔴 **每次 12B CSPRNG 随机 nonce**(或 XChaCha20-Poly1305 24B nonce 规避 GCM nonce 悬崖);🔴 **旧 XOR `.enc` 作废、重新 `save_signing_key` 导入**(chain-in-dev 无迁移)。
2. **App 锁 KDF 统一**:`citizenwallet/.../app_lock_service.dart:148`(1M)与 `citizenapp/.../app_lock_service.dart:166`(100K)收敛 **Argon2id**;🔴 **参数(m/t/p)单一常量源,跨 citizenwallet/citizenapp/node 与清算行 Argon2id 同源统一**。
3. **热钱包 at-rest 对齐**:`citizenapp` 助记词无应用层加密(只靠 secure_storage),补一层 AES-256-GCM,对齐 `citizenwallet/lib/wallet/mnemonic_cipher.dart`。

所属模块:Blockchain(node)+ Mobile(citizenapp/citizenwallet)

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
- 热钱包助记词应用层 AES-256-GCM 落地;清算行 nonce 每次随机、Argon2id 参数单一源、旧 .enc 作废说明。
- 对应单测通过,真实运行态验收(清算行解锁 / App 锁 / 钱包恢复)。(IM 不在本卡)
