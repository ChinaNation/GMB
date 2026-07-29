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

### 第 5 步：附件本地缓存加密（2026-07-29 完成）

**现状订正**：项 11 不是"完全没做"——大媒体**传输**早已加密（上传 R2 前流式
AES-256-GCM）；缺的是**下载解密后直接明文写进长期附件缓存**。

**方案 A（用户 2026-07-29 选定）**：长期缓存一律密文，播放/预览时才解密到
**短命明文临时文件**。选 A 而非"全程无明文落盘"的 B：图片/视频播放器要的是
文件路径而不是内存字节，B 对视频的工程代价过高。

**明文生命周期（关键决策，用户确认「前台存活 + 三点 purge」）**

原打算"用完即删、逐处交接所有权"，实施前发现该口径**不可靠**：UI 侧有预览、
播放、打开、转发多条路径，逐个 widget 交接极易漏，而漏一次这份明文就永久留盘，
方案 A 的安全性即名存实亡。改为**不依赖任何调用方记得释放**，靠生命周期兜底：

1. **App 启动** purge（`main.dart` `initState`）——崩溃/强杀会跳过退后台清理，
   没有这道兜底明文会跨会话存活；
2. **退到后台** purge（`didChangeAppLifecycleState` paused）——把明文窗口
   压到一次前台会话内；
3. **删会话 / 退出账户** purge（`deleteLocalConversation`）。

代价：同一次前台会话内看过的附件明文存活到切后台。比"逐处交接但漏几处永久留盘"
安全得多，且不可能因新增 UI 路径而退化。

**改动**
- 新增 `lib/chat/media/attachment_vault.dart`：`AttachmentVault`。密文后缀 `.enc`
  （与明文路径永不重名，杜绝"以为加密了其实读的是旧明文"）；明文只落
  `<cache>/.plain/` 专用目录；复用 `MediaRelayCrypto` 分块流式（5GB 不进内存）；
  **解密失败也删半截明文**。
- `lib/chat/chat_flow.dart`：`importAttachmentFileToCache` / `acceptReceivedMediaToCache`
  / `readCachedAttachment` / `downloadAttachment` 增 `attachmentKey` + `plainDirectory`。
  **`readCachedAttachment` 判据必须改**：密文长度含分块 GCM 框架开销、与明文不等，
  不能再拿密文 stat 比对 `clearByteSize`；改为解密后验明文长度，不符即清明文返回 null。
  改造后 `_streamCopy` 失去引用，已删除（无残桩）。
- `lib/chat/chat_runtime.dart`：`_attachmentKey()`（`LocalKeyPurpose.attachment` 子钥）、
  `_plainDirectory()`、`purgePlainAttachments()`；5 处调用点接线。
- `lib/main.dart`：启动与退后台两个 purge 点，失败静默（纵深防御，不阻断启动/切换）。

**验收（实跑）**
- 新增 `test/chat/media/attachment_vault_test.dart` **7 项通过**：
  密文落盘且明文源被删、明文路径根本不存在只有 `.enc`、往返完整还原、
  **错误密钥失败不留半截明文**、密文缺失明确报错、**崩溃残留由启动清理兜底**、
  重复 open 不残留多份。
- `flutter analyze lib/ test/` 零问题。

### 第 6 步：通讯录本地副本加密（2026-07-29 完成，线程 A 五项收官）

**现状订正**：项 12 不是"完全没做"——云端**早已加密**（AES-256-GCM + HMAC 不透明
索引，Worker 只见密文）；缺的是本地仍以 `jsonEncode` 明文写进 Isar KV。

**改动**：本地四把 KV（`contact_book_by_account:` / `contact_pending_by_account:` /
`contact_sync_by_account:` / `contact_cloud_reset_by_account:`）的值改为 AES-256-GCM。
加解密收敛在 `_readKv` / `_writeKv` / `_writeSnapshot` 三处，上层模型与 UI 零改动。

- **密钥与云端严格分开**：本地用 LDK 的 `LocalKeyPurpose.contactsLocal` 子钥，
  云端仍用 `citizenapp.contacts/encryption`（账户 child 直接派生）。**不复用云端钥**——
  否则本地密文一旦被拿到就等于同时暴露云端密文。
- 归属账户直接从 KV 键名后缀解析（四个前缀都以 `:<accountId>` 结尾），
  无需把 accountId 层层透传；子钥按账户缓存。
- AAD 绑完整 KV 键名，防三份密文被互换。
- 加密在 Isar 事务**外**完成（与聊天同一原则）。
- `_readKv` 解密失败**直接抛错，不静默返回 null**——静默会被上层当成"本地无缓存"
  而拉云端整表覆盖，悄悄丢掉待同步的本地改动。

**"顺手修 cid_number" 一项经核查无需修改**：该缺陷已在 CID 重构中由另一线程修复——
`cid_number` 现已进密文（`contact_service.dart:155`）、解密还原（`:195`）、
且作为合并主键不可能为 null，旧的链读回填 `cacheContactCidNumber` 已删除。
本卡先前审计结论基于重构前代码，现已过期。

**验收（实跑）**
- `test/user/` **26 项通过**，新增 3 项关键断言：
  1. 绕过服务直接读 Isar 原始 KV 行，**备注 / CID / SS58 / 连字段名都不出现**；
  2. 加密后读回仍是完整明文对象；
  3. 本地密文被篡改必须抛 `LocalCipherException`，不得静默当成"无本地缓存"。
- 两个既有换绑迁移测试因第二个 fake（`_RecordingWallet`）未覆盖新方法而落到真实
  平台通道，已补同款测试注入口；同时订正了两处已过期注释（"只落本地明文缓存"）。
- `flutter analyze lib/ test/` 零问题。

---

## 线程 A 五项收官汇总（2026-07-29）

| 项 | 步骤 | 结果 |
|---|---|---|
| 9 MLS 私钥/群秘密 | 第 2 步 | ✅ AEAD 信封 + 原子写 |
| 10 聊天正文/会话摘要 | 第 3 步 | ✅ 字段级密文 |
| 8 HMAC 分词索引 | 第 4 步 | ✅ 两段式搜索（收窄 + 复验） |
| 11 附件本地缓存 | 第 5 步 | ✅ 密文缓存 + 前台存活明文 |
| 12 通讯录本地副本 | 第 6 步 | ✅ 本地 KV 密文 |

全部密钥出自第 1 步的 LDK 信封，五个用途域隔离；CID 换绑只重 wrap 一次 LDK，
已落盘密文无需重写。

**仍未覆盖的明文面（须知悉，不在本卡范围）**：iOS 端 `HardwareBoundSeedVault`
无原生桥（线程 B 负责），故 iOS 上 child mini-secret 的硬件保护弱于 Android，
会传导到 KEK。加密设计本身不受影响，但 iOS 整体强度须等线程 B 补齐。

### 自审与修复(2026-07-29,六步交付后)

按 `memory/07-ai/audit-recipe.md` 对本卡六步产出做自审,发现 **1 个 CRITICAL + 1 个
HIGH + 1 个 MEDIUM**,均已修复。

#### 🔴 A(CRITICAL,已修)换绑未接 LDK 重 wrap → 全部本地密文永久不可读

**证据锚点**:`rewrapLocalDataKeyForRebind` 曾**零调用点**——
`wallet_manager.dart:1154` 只有定义,全仓 `lib/` + `test/` 无调用方;换绑主流程
`myid_service.dart:_doRunRebindMigration` 只调了通讯录迁移。

**后果链**:换绑后 `ensureLocalDataKeyForAccountId(新账户)` 查不到缓存与 wrap →
`vault.ensureForAccount` **新生成一把 LDK** → 五把子钥全变 → 聊天/MLS/附件/通讯录
已落盘密文全部不可解密。且因解密失败被刻意做成**抛错而非静默降级**,表现为
相关页面直接报错。**直接违反死契约 `cid-rebind-subkeys-must-auto-migrate`。**

**修复**:`myid_service.dart` 本地重建插入步骤 3,顺序**必须**是
「通讯录迁移 → LDK 重 wrap → 广播身份变化」:
- 在迁移**之后**——迁移要读旧账户本地密文 KV,而重 wrap 会删旧账户 LDK wrap,
  倒过来迁移就读不出旧数据(这是修复时的真实陷阱,直接补调用会打断迁移);
- 在广播**之前**——广播后各页按新账户读本地密文,此时新账户必须已有 wrap。
- 不吞异常:失败让整个迁移重试。

**测试补齐**:`myid_service_test.dart` 新增断言——重 wrap 实参正确 +
`trace == ['contact_migrate', 'ldk_rewrap']` 钉死顺序。删掉调用即测试红。

#### 🟠 B(HIGH,已修)打开会话会解密该会话全部媒体

**证据锚点**:`chat_page.dart:_resolveMediaPaths` 遍历会话内每条媒体消息逐个解析
路径,而第 5 步把 `readCachedAttachment` 改成了「每次调用完整解密整个文件」。
改造前该函数注释原文是「只按文件存在性 + 大小(stat,不读整块字节)判定」——
刻意的轻量设计被改成了重操作。有若干视频的会话首屏要解出上 GB。

**修复**:新增 `AttachmentVault.existingPlain`(只探不解密);
`readCachedAttachment` 先复用已解密明文(长度校验通过即用),未命中才解密。

#### 🟡 C(MEDIUM,已修)`ChatCrypto.evict` 死代码

零调用点。核验确认**不是正确性问题**:缓存按 accountId 分键,且换绑后 LDK 与
五把子钥都不变,无陈旧风险。按「无残桩」死规则删除,并留注释说明为何不需要。

#### 撤回:一条先前不成立的说法

第 1 步任务卡与提交 `bf017a8e` 写「死契约以 O(1) 成本**满足**」——**该说法不成立**,
当时只做到金库层具备能力、业务链路未接。现已接上并有顺序断言,该说法此刻才成立。

#### 根因反思(给后续 auditor)

A 与 B 同源:验收停在「单元级能力正确」,没有沿**真实调用链**回推(换绑流程谁调、
UI 每帧调几次)。982 项测试全绿反而给了虚假信心。这正是仓库「真实验收硬规则」
要求真机运行态验收的原因——本卡六步至今**零真机验收**。

#### 已核验不成立的疑点(避免下轮重复排查)

- 通讯录 `cid_number` 缺陷:已由 CID 重构修好,非本卡问题;
- `_localKvKey` 从键名解析 accountId:四前缀各只含一个 `:`、accountId 是纯 hex,解析正确;
- `lib/isar/` 与线程 B / 广场双存的边界冲突:至今未发生,`git status` 干净。

## 完成标准

- Isar 和 App 私有目录不再保存联系人、聊天正文、会话摘要、MLS 秘密或附件明文。
- 密文篡改 fail-closed，密钥删除后数据不可恢复。
- 中文、英文和数字搜索能力按新 HMAC 索引真实可用。
- Android 真机完成升级迁移、聊天、搜索、媒体和换绑验收；iOS 验收依赖硬件金库任务完成。
