# 20260729 CitizenApp 客户端本地静止态加密(线程 A)

状态:open
所属模块:citizenapp(Flutter lib + rust/src)
线程边界:本卡由**线程 A** 独占执行;线程 B 见
`20260729-cloudflare-lifecycle-purge-ios-vault.md`。两线程共用
`/Users/rhett/GMB` 同一工作树(禁 worktree),必须严守下方文件边界,不得越界改文件。

## 任务需求

CitizenApp 已有"传输加密"和"云端加密",但**本地磁盘静止态全是明文**。本卡把
编号 8/9/10/11/12 五项一次性做完,让聊天正文、MLS 私钥、附件、通讯录在手机
磁盘上不再明文可读。

审计口径订正(2026-07-29):
- 项 11 媒体:**传输已加密**(上传 R2 前流式 AES-256-GCM),缺的是下载解密后
  长期附件缓存的本地加密。
- 项 12 通讯录:**云端已加密**(AES-256-GCM + HMAC 索引),缺的是本地 Isar KV。
- 项 8:全仓并非"HMAC 零命中"(通讯录 contact_id、支付都有),缺的是**聊天分词索引**。
- 项 10:`Isar.open` 无加密参数不是核心证据(isar_community 本就不支持库级加密),
  真正缺的是**密文字段与解密边界**。

## 五项目标状态

| 项 | 现状证据 | 目标 |
|---|---|---|
| 10 聊天正文 | `ChatMessageEntity.plaintext`、会话 `lastMessage` 明文字段(`lib/isar/app_isar.dart:570`) | 改密文字段,明确解密边界 |
| 8 分词索引 | 明文 `contains` 搜索(`lib/chat/storage/chat_store.dart:193`) | HMAC 分词加密索引 |
| 9 MLS 秘密 | `save_to_file` 裸写(`rust/src/chat_mls.rs:566`),含设备签名私钥与群 ratchet 秘密 | AEAD 信封加密落盘 |
| 11 附件 | 解密后直接写长期缓存(`lib/chat/chat_runtime.dart:727`) | 本地加密落盘 |
| 12 通讯录本地 | `jsonEncode` 明文入 Isar KV(`lib/my/user/contact_service.dart:607`) | 密文入库 |

风险优先级:**9 > 10 > 11 > 12**(MLS 私钥泄露可解密/伪造全部聊天,只加密正文
而不管它等于门没锁)。

## 技术约束(已定,不再重议)

- **密钥来源**:复用 `wallet_manager` 现有 HKDF-SHA256 派生
  (从 CID 绑定账户的 child mini-secret 派生,salt=`sha256(accountId)`,
  靠 `info` 串做域隔离),新增本地静止态域;该路径已含派生/持久化/读取/
  换绑重建/删除五件套,天然满足 `cid-rebind-subkeys-must-auto-migrate` 死契约。
- **算法**:AES-256-GCM(`cryptography` 包),已在通讯录与媒体中转生产运行。
- **粒度**:只能字段级 —— `isar_community` 3.3.2 的 `Isar.open` 无 `encryptionKey` 参数。
- **不上 PQC**:ADR-022 明确"创世前对 PQC 零改动",ML-KEM/ML-DSA 全仓零落地。
- `--allow-undefined` / 工具链相关见 `project_rust_pin_197_wasm_allow_undefined` 记忆。

## 文件边界(线程 A 独占,线程 B 不得触碰)

- `citizenapp/lib/isar/app_isar.dart`
- `citizenapp/lib/chat/storage/chat_store.dart`
- `citizenapp/lib/chat/chat_runtime.dart`
- `citizenapp/lib/chat/media/chat_relay_media.dart`
- `citizenapp/lib/my/user/contact_service.dart`
- `citizenapp/rust/src/chat_mls.rs`
- `citizenapp/lib/wallet/core/wallet_manager.dart`
- 以上对应 `citizenapp/test/**` 测试

**不属于本卡**:`lib/wallet/core/hardware_bound_seed_vault.dart`(线程 B 改 iOS 分支)、
`cloudflare/**`、`citizenapp/ios/**`。

## 执行方式(用户明确要求)

每一步:**先出技术方案 → 等用户确认 → 执行 → 更新文档 + 完善注释 + 完善测试 +
清理残留 → 输出下一步技术方案**。不得连跳。

## 已知代价(须在方案中正面处理)

- 正文加密后现有内存 `contains` 搜索失效,必须同步给出分词索引方案,否则功能回归。
- 除 Isar 外还有三处明文面:`openmls_storage.json`、`device.json`、
  `pending_inbound.json`、附件文件,须一并覆盖,不能只加密 `plaintext` 字段。

## 验收标准

- 手机磁盘上聊天正文、MLS 私钥、附件、通讯录均不可明文读取
- 聊天搜索功能不回归
- 换绑 CID 后本地密文可正常迁移/重加密(对齐死契约)
- 单测覆盖加解密往返、错误密钥拒绝、换绑迁移
- 文档与注释同步、无残留旧明文路径
