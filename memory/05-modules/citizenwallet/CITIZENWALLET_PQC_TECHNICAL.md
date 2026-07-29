# citizenwallet 公民钱包 PQC 抗量子签名升级技术设计

- 状态:设计 / 待实现。**真源 = `memory/04-decisions/ADR-022-unified-pqc-crypto.md`**(取代旧 PQC 迁移方案)。
- 任务卡:`memory/08-tasks/open/20260618-pqc-card3-wallet-derivation-signing.md`。

## 要点(以 ADR-022 为准)

- `citizenwallet` 完全离线,PQC 后仍须纯离线完成 ML-DSA-65 签名,不依赖在线服务。
- **同源派生(model B 全 `//index` 无根,sr25519 不套 HKDF)**:每账户 `AccountSeedV1_N` = 该账户 child mini-secret(账户 N = 助记词 `//N` 硬派生,账户0=`//0`,无 bare 根);sr25519 地址锚点 = `sr25519.fromSeed(AccountSeedV1_N)` **直接派生**(不经 HKDF);ML-DSA-65/ML-KEM-768 才用 `HKDF-SHA512(AccountSeedV1_N, "GMB/account/ml-dsa-65/v1" | ".../ml-kem-768/v1")`。冷热共用 `gmb-pqc` crate(原生 C FFI)。
- **无感 bootstrap**:未绑定账户首次扫码,**同一确认**同时出 sr25519 bootstrap 签名(证旧地址主人)+ ML-DSA 交易签名;后续只签 ML-DSA。**无单独绑定步骤、无账户状态切换**。
- **QR 扩展**:`sig_alg(sr25519|ml-dsa-65)` + `auth_mode(normal|pqc|bootstrap-pqc)` + `key_version` + `chunk_index/chunk_total`(ML-DSA ~3.3KB,最坏体积按 bootstrap 实测分片)。
- **安全**:`AccountSeedV1`/私钥不出本机、不进二维码;payload 带 `genesis_hash`+`spec_version` 防跨链/跨升级重放;bootstrap 强度=sr25519,窗口须在量子破 sr25519 前关闭。

> 实现以本文 + ADR-022 为准,旧路线不再适用。

## 当前账户与本地存储模型

- `AccountId` 是 CitizenWallet 内的钱包身份、Isar 唯一索引、重复检查和签名目标核验
  真源；Dart 使用 `accountId`，文本固定为小写 `0x` 加 64 位十六进制。
- `ss58Address` 只用于页面展示、用户二维码和扫码输入输出，不作为授权、去重或
  持久化身份主键。
- `signerPublicKey` 只表示真实签名公钥。当前 sr25519 的 `AccountId32` 直接取该公钥
  32 字节，因此登录和离线签名可做字节等值核验；不得据此把字段重新混称为
  `pubkeyHex` 或裸 `address`。
- `WalletProfileEntity` 最终字段保存 `accountId + ss58Address + ss58Prefix` 及钱包
  展示元数据，不重复保存同一 32 字节值的公钥别名。
- 正式创世切换前，CitizenWallet 已只按最终 Isar schema 完成旧业务库重建；运行态不执行
  旧格式 migration，不读取旧 `address` / `pubkeyHex` 字段。正式创世后不得再把开发期
  “删除重建”口径用于现行钱包数据或安全材料。
- Secure Storage、Android Keystore、iOS Keychain 中的 seed、助记词密文、PIN
  派生材料和私钥保护材料不属于 Isar 业务数据，业务库重建不得删除或改写它们。
- CitizenWallet 的 Isar 引擎固定使用 `isar_community`、
  `isar_community_flutter_libs`、`isar_community_generator` 3.3.2。旧 Isar 3.1
  预编译 `libisar.so` 只有 4 KB ELF 对齐，禁止恢复；community 引擎的
  arm64-v8a/x86_64 `libisar.so` 必须保持 `0x4000` 或更高。
- Isar 引擎升级不得改变 `WalletEntity`、`AccountEntity`、`AppKvEntity` 的
  Collection ID、字段或索引。3.1 业务库已用非敏感测试数据验证可由 community 3.3.2
  原地打开、按索引读取并继续写入；生成 schema 只允许更新引擎版本字符串。
- community 引擎使用 `<name>.isar-lck`；启动前幂等删除 Isar 3.1 遗留的空
  `<name>.isar.lock`。该清理只处理旧锁文件，不删除 `<name>.isar` 业务库或任何
  secure storage 数据。
- Android 发布产物必须同时验证两层：APK 用 Build-Tools 35+ 执行 16 KB ZIP
  alignment 检查；APK/AAB 内 arm64-v8a 与 x86_64 每个 `.so` 的 ELF `LOAD` 段不得
  出现 `0x1000`。普通 4 KB 真机或模拟器只能验证升级安装和冷启动，不能冒充 16 KB
  内核运行态。

## CI 与发布边界

- `citizenwallet-ci.yml` 的 push 自动 CI 只执行索引同步、Flutter 依赖安装、`flutter analyze`、`flutter test` 和 Debug APK 检查构建。
- 移动端 CI 与本机开发统一锁定 Flutter `3.44.4`，版本真源为仓库根目录 `.fvm/fvm_config.json`；CI workflow 不使用浮动 `channel: stable`。
- CI 与本地启动脚本同步的转账入口是 `OnchainTransaction(4).transfer_with_remark(0)`；公民钱包不得恢复 `Balances` 直签入口。
- 正式签名 `公民钱包.apk` 只允许 GitHub 页面手动 `Run workflow` 构建和上传。
- 手动发布只读取一个 GitHub Secret：`GMB_APP_KEY`。它与公民 App 共用，内容至少包含 `keystore=<base64后的jks>` 和 `password=<keystore密码>`；默认 Android key alias 为 `upload`，如现有 keystore 使用其他别名，可在同一个 secret 内增加 `alias=<key别名>`；key password 默认复用同一个 `password`。
