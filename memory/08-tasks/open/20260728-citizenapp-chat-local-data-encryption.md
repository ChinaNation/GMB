# CitizenApp Chat 与本地隐私数据静止态加密

状态：open

## 任务需求

- 使用现有 HKDF-SHA256 + AES-256-GCM 体系完成字段级静止态加密，不引入 PQC 或第二套密码体系。
- 加密 OpenMLS provider 状态和设备签名秘密。
- 加密 Chat 正文、会话摘要和本地附件缓存。
- 建立 HMAC-SHA256 搜索索引，替代对明文正文的直接子串搜索。
- 加密通讯录本地副本；Cloudflare 继续只保存通讯录端到端密文。

## 已确认边界

- 密钥从身份账户 child mini-secret 域隔离派生，必须覆盖派生、保存、读取、换绑、删除生命周期。
- Isar 社区版不提供库级加密，本任务采用字段级/文件级信封加密。
- Cloudflare Chat 不保存消息或普通附件；大媒体中转继续只承载端到端密文。
- 每一步必须先提交技术方案，用户确认后才能执行。
- 执行后必须更新文档、完善中文注释与测试、清理明文字段和旧存储残留。

## 预计修改目录

- `citizenapp/lib/chat/`：消息、摘要、搜索索引、附件缓存和迁移流程。
- `citizenapp/rust/src/`：OpenMLS provider 状态的加密、原子写入和恢复。
- `citizenapp/lib/my/user/`：通讯录本地密文存储。
- `citizenapp/lib/isar/`：密文字段、索引实体及本地迁移。
- `citizenapp/lib/wallet/core/`：域隔离密钥派生和生命周期。
- `citizenapp/test/`、`citizenapp/rust/` 测试：密码边界、篡改、迁移、搜索和崩溃恢复。
- `memory/03-security/`、`memory/05-modules/citizenapp/`、`memory/07-ai/`：安全与模块文档。

## 主要风险

- OpenMLS 状态迁移失败会导致历史群状态不可读，必须使用临时文件、认证成功后原子替换。
- HMAC 索引会泄露相同 token 的频率关系；不得保存明文 token。
- 媒体展示期间可能产生临时明文字节，必须限定生命周期并在异常路径清理。
- CID 换绑时必须重封装本地密文，不得依赖已泄漏旧账户密钥。

## 实施记录

> 执行线程：线程 A。文件边界见 `20260728-square-local-copy-membership-purge.md`（线程 B）。
> **`citizenapp/lib/isar/` 归线程 A**，第 3 步会改其 schema，线程 B 若要加广场帖子
> collection 须先协调，否则两线程会互相覆盖 Isar schema。

### 第 1 步：本地静止态加密基座（2026-07-29 完成，提交 bf017a8e）

**设计定案：信封（LDK）而非直接派生。** 若照通讯录钥直接从账户 child 派生本地加密钥，
CID 换绑就要把整个聊天历史 + MLS 状态 + 全部附件重新加密一遍（手机上可能数 GB，
必然卡死或中断）。改为：

```text
账户 child mini-secret
      │ HKDF(info="citizenapp.local/kek", salt=sha256(accountId))
      ▼
    KEK ── AES-256-GCM wrap/unwrap ──► LDK(32B 随机，终身不变)
                                         │ HKDF(info=用途域)
                                         ▼
                  chat / chat-index / mls / attachment / contacts-local 五把子钥
```

换绑只重 wrap 一次 LDK（O(1)），**已落盘密文一个字节都不用重写**，死契约
`cid-rebind-subkeys-must-auto-migrate` 以 O(1) 成本满足。

**实施中发现的 UX 缺陷及对策**：LDK 每次解包都需账户 child，而读 child 会触发生物识别
→ 每次开聊天都要按指纹，不可接受。沿用通讯录钥既有对策：**wrap 是持久真源与换绑迁移用，
日常读走静默缓存**；钱包创建/导入时 child 本就在手，预置 LDK 零额外弹窗。

- 新增 `lib/security/local_cipher.dart`（AES-256-GCM，12B 随机 nonce，
  单串 `base64(nonce||ct||mac)`，**AAD 必填**防串位重放；错误密钥/AAD 不符/篡改/
  非法 base64/长度不足一律抛 `LocalCipherException`，绝不静默返回空）
- 新增 `lib/security/local_data_key.dart`（`LocalKeyPurpose` 五用途域、子钥派生、信封金库）
- `lib/wallet/core/wallet_manager.dart` 加 4 个公开入口 + blob store 适配器
  （方向固定「钱包依赖安全基座」，基座不反向依赖钱包）；创建/导入两处预置 LDK。
  **未改动任何既有通讯录派生逻辑。**
- 验收：analyze 零问题；`test/security/` 25/25；`test/security/ + test/wallet/` 169/169

### 第 2 步：MLS 私钥与群秘密加密（2026-07-29 完成）

风险最高的一项优先做：拿到 `openmls_storage.json` 即可解密并伪造全部聊天，
只加密正文而不管它等于门没锁。

**关键实现选择**：上游 `MemoryStorage` 的 `save_to_file`/`load_from_file` 硬绑 `&File`，
**只能把明文直接写盘**。但 `MemoryStorage.values` 是公开字段，故改为自己序列化到
内存缓冲再整体 AEAD 加密落盘 —— **全程无明文触盘**，无需临时明文文件。

- `rust/src/chat_mls.rs`：新增 `seal_state`/`open_state`（AES-256-GCM，
  `nonce||ct||tag`）、`parse_state_key`、`atomic_write`、`purge_legacy_plaintext_state`；
  `load_provider`/`save_provider`/`ensure_device_signer` 全部带密钥；
  8 个请求结构体加 `state_key_hex`。
- **原子写入**：`atomic_write` 先写 `.tmp` + `sync_all`，再 `rename` 覆盖。
  MLS 状态写坏会导致该设备**全部群与会话不可读**，必须原子替换。
- 文件改名并加密：`openmls_storage.json`→`.bin`、`device.json`→`.bin`、
  `pending_inbound.json`→`.bin`（后者由 Dart `MlsStateStore` 用同钥加密）。
- **旧明文一律直接删除，零迁移零兼容**（用户 2026-07-29 决定：开发期零用户）。
  Rust 侧 `purge_legacy_plaintext_state`、Dart 侧 `_purgeLegacyPlaintext`。
- 密钥走 `LocalKeyPurpose.mls` 子钥，Dart 经 FFI 下传 hex，**Rust 侧只收密钥、
  不接触钱包种子**；storage 与 device 用**不同 AAD 域**，防两个文件被对调。
- `rust/Cargo.toml` 显式加 `aes-gcm`（RustCrypto 官方实现，禁自造 AEAD）+ `base64`；
  两者本已在 lock 中，无新增下载。
- Dart：`mls_state_store.dart` 增 `stateKey`/`stateKeyHex` 并加密 pending 队列；
  `mls_native.dart` 9 处 payload 下传密钥；`chat_runtime.dart` 从 LDK 取 mls 子钥。
- 验收：`cargo test` **13/13**（含真实三方群会话往返、信封往返、错误密钥/错误 AAD/
  篡改拒绝、nonce 随机、旧明文清除、原子写不留临时文件）；
  `flutter analyze lib/ test/` 零问题；`flutter test test/security/ test/chat/` **186 通过**。

**与本卡原风险条目的偏差（须知悉）**：卡里写「OpenMLS 状态迁移失败……」预设有迁移，
实际按用户决定**不做迁移**，旧明文直接删除；对应地，`atomic_write` 仍按卡要求实现，
用于防止正常写入过程崩溃导致状态损坏。

### 第 3+4 步（合并）：聊天正文加密 + HMAC 分词搜索索引（2026-07-29 完成）

用户选择**合并实施**，避免出现「加密完成但搜索只能全解密扫描」的中间降级态。

**三个设计决策**
1. **索引用字符 bigram**（去重后存）。中文没有词边界，英文/数字也要支持子串搜索，
   bigram 两者通吃；查询不足 2 字符时回落到按 `accountId` 收窄后解密扫描——
   单字符查询在中文里很常见，不能直接拒绝。
2. **token = HMAC-SHA256 截断 8 字节**（`LocalKeyPurpose.chatIndex` 子钥，
   与正文钥域隔离）。截断换索引体积，代价是假阳性。
3. **解密边界收敛在 `ChatStore` 一层**，UI 与业务层拿到的仍是明文 DTO
   （`ChatStoredMessage.plaintext` / `ChatConversationPreview.lastMessage`），
   不散落解密代码、不散落密钥。

**搜索改为两段式（关键）**：token 命中只用来**收窄候选**，解密后**必须再验一次
真实子串**。不可省——token 是截断值有假阳性，且 bigram 命中 ≠ 原串顺序命中
（查 `abc` 会命中含 `ab`/`bc` 但实为 `bcab` 的记录）。复验保证语义与旧的明文
`contains` 完全一致。已写成测试钉死。

**改动**
- `lib/isar/app_isar.dart`：`ChatConversationEntity.lastMessage` →
  `lastMessageCipher`；`ChatMessageEntity.plaintext` → `plaintextCipher`；
  新增 `searchTokens: List<String>` 带 `@Index(type: IndexType.value)` 多值索引。
  `app_isar.g.dart` 已重新生成。
- **`envelopeBytesHex` 刻意不加密**：其内容是 MLS 端到端密文，且随附元数据
  （sender/recipient/conversation）本就是明文列，再套一层不减少泄露面。
- 新增 `lib/chat/storage/chat_crypto.dart`：`ChatCrypto`（按 accountId 缓存子钥、
  加解密、分词、`tokenize` 静态方法便于单测）。含 `debugFixedLocalDataKey`
  测试注入口，仿 `WalletManager.debugSeedStore` 惯例。
- `lib/chat/storage/chat_store.dart`：4 个写入路径改为**事务外预加密**
  （不让密码学运算占住 Isar 写事务）；2 个读取路径解密；`searchMessages` 两段式重写；
  两个 mapper 改为接收已解密文本。

**测试策略**：`test/support/isar_test_env.dart` 注入固定 LDK。这是
**换密钥来源、不绕过加密**——测试仍走真实 AES-GCM 与真实 HMAC 索引，
否则加密就成了测试盲区。

**验收（实跑）**
- `flutter analyze lib/ test/` 零问题
- `flutter test test/chat/` **169 通过**（既有测试全部恢复，无回归）
- 新增 `test/chat/storage/{chat_crypto_test,chat_store_encryption_test}.dart`
  **16 通过**，其中两条关键断言：
  1. 绕过 `ChatStore` 直接查 Isar 原始行，`plaintextCipher` / `lastMessageCipher` /
     `searchTokens` 中**均不含明文片段**；
  2. `bcab` 作为 `abc` 的索引假阳性**必须被复验滤掉**，只返回真正含 `abc` 的记录。
  另覆盖中英数子串命中、大小写不敏感、单字符回落、不命中返空、
  密文损坏必抛错（不静默返回空白，否则用户会看到聊天记录凭空变空）。

## 完成标准

- Isar 和 App 私有目录不再保存联系人、聊天正文、会话摘要、MLS 秘密或附件明文。
- 密文篡改 fail-closed，密钥删除后数据不可恢复。
- 中文、英文和数字搜索能力按新 HMAC 索引真实可用。
- Android 真机完成升级迁移、聊天、搜索、媒体和换绑验收；iOS 验收依赖硬件金库任务完成。
