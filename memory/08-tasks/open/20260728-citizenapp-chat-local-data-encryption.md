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

## 完成标准

- Isar 和 App 私有目录不再保存联系人、聊天正文、会话摘要、MLS 秘密或附件明文。
- 密文篡改 fail-closed，密钥删除后数据不可恢复。
- 中文、英文和数字搜索能力按新 HMAC 索引真实可用。
- Android 真机完成升级迁移、聊天、搜索、媒体和换绑验收；iOS 验收依赖硬件金库任务完成。
