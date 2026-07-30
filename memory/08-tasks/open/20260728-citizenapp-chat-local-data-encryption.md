# CitizenApp Chat 与本地隐私数据静止态加密

状态：open

## 任务需求

- 使用现有 HKDF-SHA256 + AES-256-GCM 体系完成字段级静止态加密，不引入 PQC 或第二套密码体系。
- 加密 OpenMLS provider 状态和设备签名秘密。
- 加密 Chat 正文、会话摘要和本地附件缓存。
- 建立 HMAC-SHA256 搜索索引，替代对明文正文的直接子串搜索。
- 加密通讯录本地副本；Cloudflare 继续只保存通讯录端到端密文。

## 已确认边界

- 业务用途子钥从 CID 稳定数据根域隔离派生；当前身份账户 child 只包装数据根，必须覆盖
  发放、包装、读取、换绑接管和删除生命周期。
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
- CID 换绑不得重写业务密文，也不得依赖此前账户密钥；当前新账户必须领取同一 CID
  数据根并用自己的 child 包装。

## 实施记录

> 执行线程：线程 A。文件边界见 `20260728-square-local-copy-membership-purge.md`（线程 B）。
> **`citizenapp/lib/isar/` 归线程 A**，第 3 步会改其 schema，线程 B 若要加广场帖子
> collection 须先协调，否则两线程会互相覆盖 Isar schema。

### 第 1 步：本地静止态加密基座（2026-07-29 完成，提交 bf017a8e）

**最终设计：CID 稳定数据根信封。** 业务密钥不能直接由钱包账户 child 决定，否则换绑
会改变全部用途子钥。目标模型为：

```text
CID 永久业务数据 ──► CID 稳定数据根(32B，换绑不变)
                            │ HKDF(info=cid 用途域)
                            ├─ chat / chat-index / mls / attachment
                            ├─ contacts-local / contacts-cloud
                            └─ drafts

当前绑定账户 child
      │ HKDF(info="citizenapp.cid-data-root/kek",
      │      salt=sha256(cid|revision|account))
      ▼
    KEK ── AES-256-GCM wrap/unwrap ──► CID 稳定数据根
```

绑定 finalized 后，当前新账户通过一次性挑战取得同一数据根，完成新包装写入和读回摘要
校验后激活精确绑定标记，再删除低版本包装。已落盘密文不重写；此前账户、此前私钥和此前
设备都不是输入。钱包创建/导入阶段不得生成 CID 数据根，因为当时尚无 CID。

- 新增 `lib/security/local_cipher.dart`（AES-256-GCM，12B 随机 nonce，
  单串 `base64(nonce||ct||mac)`，**AAD 必填**防串位重放；错误密钥/AAD 不符/篡改/
  非法 base64/长度不足一律抛 `LocalCipherException`，绝不静默返回空）
- `lib/security/local_data_key.dart` 提供 `CidDataRoot`、七个 CID 用途域和
  `CidDataRootVault`；文件名沿用现有安全基座路径，类型与协议语义已全部 CID 化。
- `lib/wallet/core/wallet_manager.dart` 只在 finalized 精确绑定接管时安装数据根，
  钱包创建/导入不再生成根；日常读取走已验证的 CID 缓存。
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
  `mls_native.dart` 9 处 payload 下传密钥；`chat_runtime.dart` 从 CID 数据根取 mls 子钥。
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
  加解密、分词、`tokenize` 静态方法便于单测）。含测试专用 `debugFixedCidDataRoot`
  测试注入口，仿 `WalletManager.debugSeedStore` 惯例。
- `lib/chat/storage/chat_store.dart`：4 个写入路径改为**事务外预加密**
  （不让密码学运算占住 Isar 写事务）；2 个读取路径解密；`searchMessages` 两段式重写；
  两个 mapper 改为接收已解密文本。

**测试策略**：`test/support/isar_test_env.dart` 注入固定 CID 数据根。这是
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

**最终改动**：本地三把 CID 分区 KV（`contact_book_by_cid:` /
`contact_pending_by_cid:` / `contact_sync_by_cid:`）的值使用 AES-256-GCM。
加解密收敛在 `_readKv` / `_writeKv` / `_writeSnapshot` 三处，上层模型与 UI 零改动。

- **用途域严格分开**：本地用 CID 数据根的 `contactsLocal` 子钥，云端用同一根的
  `contactsCloud` 子钥。两者不复用，当前账户 child 只负责包装根——
  否则本地密文一旦被拿到就等于同时暴露云端密文。
- 属主 CID 直接从业务上下文取得；当前绑定账户只用于领取并解包同一 CID 稳定数据根，
  子钥按 `cid_number + 当前账户` 的已验证上下文缓存，不改变密文归属。
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

全部业务密钥出自 CID 稳定数据根，各用途域隔离；CID 换绑由当前新账户重新包装同一根，
已落盘密文无需重写。

**仍未覆盖的明文面（须知悉，不在本卡范围）**：iOS 端 `HardwareBoundSeedVault`
无原生桥（线程 B 负责），故 iOS 上 child mini-secret 的硬件保护弱于 Android，
会传导到 KEK。加密设计本身不受影响，但 iOS 整体强度须等线程 B 补齐。

### 自审与修复(2026-07-29,六步交付后)

按 `memory/07-ai/audit-recipe.md` 对本卡六步产出做自审,发现 **1 个 CRITICAL + 1 个
HIGH + 1 个 MEDIUM**,均已修复。

#### 🔴 A(CRITICAL,最终修复)账户信封曾被误当成身份数据根

旧实现把随机根与账户绑定，并试图在换绑时依赖此前账户解包再重封装；此前账户私钥或设备
不可用时会导致全部本地密文不可读。最终修复改为三层模型：数据永久归 CID，稳定数据根
由 CID 层按当前 finalized 绑定发放，当前账户仅包装。接管顺序固定为：

1. 当前新账户签一次性挑战；
2. Worker 前后两次核验 finalized `cid + revision + account`；
3. 新账户包装同一根并读回校验；
4. 派生 CID 用途子钥并登记当前设备；
5. 写完整接管标记后清低版本包装。

测试覆盖此前账户零输入、错误新账户 secret、摘要不符、revision 回退/冲突、同 CID 子钥
稳定、旧密文换绑后可解和低版本包装清理。

#### 🟠 B(HIGH,已修)打开会话会解密该会话全部媒体

**证据锚点**:`chat_page.dart:_resolveMediaPaths` 遍历会话内每条媒体消息逐个解析
路径,而第 5 步把 `readCachedAttachment` 改成了「每次调用完整解密整个文件」。
改造前该函数注释原文是「只按文件存在性 + 大小(stat,不读整块字节)判定」——
刻意的轻量设计被改成了重操作。有若干视频的会话首屏要解出上 GB。

**修复**:新增 `AttachmentVault.existingPlain`(只探不解密);
`readCachedAttachment` 先复用已解密明文(长度校验通过即用),未命中才解密。

#### 🟡 C(MEDIUM,已修)`ChatCrypto.evict` 死代码

零调用点。核验确认**不是正确性问题**：缓存按当前绑定校验，且换绑后 CID 数据根与
五把子钥都不变,无陈旧风险。按「无残桩」死规则删除,并留注释说明为何不需要。

#### 最终订正

早期“账户间重包装即可接管”的方案已撤销。O(1) 的正确含义是：当前新账户从 CID 层领取
同一稳定数据根并创建自己的包装，不扫描或重写业务密文，也不要求此前账户参与。

#### 根因反思(给后续 auditor)

A 与 B 同源:验收停在「单元级能力正确」,没有沿**真实调用链**回推(换绑流程谁调、
UI 每帧调几次)。982 项测试全绿反而给了虚假信心。这正是仓库「真实验收硬规则」
要求真机运行态验收的原因——本卡六步至今**零真机验收**。

#### 已核验不成立的疑点(避免下轮重复排查)

- 通讯录 `cid_number` 缺陷:已由 CID 重构修好,非本卡问题;
- 本地 KV 已按 CID 分区，不再从键名解析账户作为归属；
- `lib/isar/` 与线程 B / 广场双存的边界冲突:至今未发生,`git status` 干净。

## 完成标准

- Isar 和 App 私有目录不再保存联系人、聊天正文、会话摘要、MLS 秘密或附件明文。
- 密文篡改 fail-closed，密钥删除后数据不可恢复。
- 中文、英文和数字搜索能力按新 HMAC 索引真实可用。
- Android 真机完成聊天、搜索、媒体和 finalized 换绑接管验收；iOS 验收依赖硬件金库任务完成。
