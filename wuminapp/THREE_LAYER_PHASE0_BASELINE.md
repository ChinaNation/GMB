# WuminApp 三层数据架构第0阶段基线（Phase 0）

更新时间：2026-03-05

## 1. 目标与范围

本阶段只做三件事：

1. 冻结当前真实数据存储现状（As-Is）。
2. 给出迁移目标落点（To-Be）。
3. 形成逐项映射清单（旧键/内存态 -> Isar/Postgres/Secure Storage）。

范围：`wuminapp/mobile` 与 `wuminapp/backend`。

## 2. 当前现状（As-Is）

### 2.1 手机端存储现状

当前手机端没有 Isar/SQLite，业务数据主要在 `SharedPreferences`，机密数据在 `flutter_secure_storage`。

#### 2.1.1 SharedPreferences 现有键

| Key | 模块 | 说明 | 现状去向 |
|---|---|---|---|
| `wallet.has_wallet` | wallet | 是否有钱包（布尔） | 迁移后可删除（可由 `wallet_profiles` 推导） |
| `wallet.items` | wallet | 钱包列表 JSON | 迁移到 Isar `wallet_profiles` |
| `wallet.active_index` | wallet | 当前激活钱包 index | 迁移到 Isar `wallet_settings.active_wallet_id` |
| `wallet.index` | wallet(legacy) | 旧单钱包 index | 一次性迁移后删除 |
| `wallet.address` | wallet(legacy) | 旧单钱包地址 | 一次性迁移后删除 |
| `wallet.pubkey_hex` | wallet(legacy) | 旧单钱包公钥 | 一次性迁移后删除 |
| `wallet.alg` | wallet(legacy) | 旧单钱包算法 | 一次性迁移后删除 |
| `wallet.ss58` | wallet(legacy) | 旧单钱包 ss58 | 一次性迁移后删除 |
| `wallet.created_at_millis` | wallet(legacy) | 旧单钱包创建时间 | 一次性迁移后删除 |
| `wallet.source` | wallet(legacy) | 旧单钱包来源 | 一次性迁移后删除 |
| `wallet.mnemonic` | wallet(legacy) | 旧单钱包助记词（明文遗留） | 一次性迁移到 secure 后删除 |
| `wallet.counter` | wallet(legacy) | 历史计数器遗留键 | 删除 |
| `settings.face_auth_enabled` | wallet | 签名前生物识别开关 | 迁移到 Isar `wallet_settings` |
| `trade.onchain.records` | wallet/trade | 链上交易记录 JSON | 迁移到 Isar `tx_records` |
| `wallet.admin_catalog.role_map` | wallet | 管理员角色缓存 | 迁移到 Isar `admin_role_cache` |
| `wallet.admin_catalog.updated_at` | wallet | 角色缓存更新时间 | 迁移到 Isar `admin_role_cache` |
| `attest.token` | wallet | 证明 token | 迁移到 secure `wallet.session.attest.token.v1` |
| `attest.expires_at_millis` | wallet | 证明 token 过期时间 | 迁移到 Isar `session_meta` |
| `attest.policy` | wallet | 证明策略 | 迁移到 Isar `session_meta` |
| `attest.last_payload` | wallet | 证明请求载荷缓存 | 迁移到 Isar `session_meta` |
| `sfid.bind.status` | wallet | SFID 绑定状态 | 迁移到 Isar `sfid_bind_state` |
| `sfid.bind.address` | wallet | 绑定钱包地址 | 迁移到 Isar `sfid_bind_state` |
| `sfid.bind.updated_at` | wallet | 绑定更新时间 | 迁移到 Isar `sfid_bind_state` |
| `login.used_request_ids` | login | 登录防重放本地缓存 | 迁移到 Isar `login_replay_cache` |
| `login.whitelist_config.v1` | login | aud 白名单配置 | 迁移到 Isar `login_policy` |
| `observe.accounts` | observe | 观察账户列表 JSON | 迁移到 Isar `observed_accounts` |
| `user.profile.nickname` | profile | 用户昵称 | 迁移到 Isar `user_profile` |
| `user.profile.avatar_path` | profile | 头像路径 | 迁移到 Isar `user_profile` |

#### 2.1.2 flutter_secure_storage 现有键

| Key | 模块 | 说明 | 目标 |
|---|---|---|---|
| `wallet.mnemonic.<walletIndex>` | wallet | 当前多钱包助记词 | 保留（后续改为 wallet_id 命名） |
| `login.whitelist_hmac_secret.v1` | login | 白名单配置完整性 HMAC secret | 保留 |

### 2.2 后端存储现状

当前 `wuminapp/backend` 未接入 Postgres/SQLite/Redis，核心是无状态 API + 进程内状态。

| 位置 | 现状 | 风险 |
|---|---|---|
| `tx_service` 的 `PREPARED_STATE`/`TX_STATE`（内存 HashMap） | 交易预处理态与运行态在内存 | 进程重启即丢，无法多实例共享 |
| `admin_catalog_seed.json`（`include_str!`） | 机构中文名映射是编译期静态资源 | 更新依赖发版；不是数据库 |
| `wallet_service`/`admin_catalog_service` | 直接请求链 RPC | 无本地持久化缓存 |
| `chain_binding_service` | 转发网关请求，不落库 | 无审计记录 |

## 3. 目标架构（To-Be）

### 3.1 手机端机密层（Secure Storage）

仅保存高敏材料：

- `wallet.secret.<wallet_id>.mnemonic.v1`
- `wallet.secret.<wallet_id>.sr25519.v1`（可选）
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`

约束：助记词/私钥不得进入 Isar 与日志。

### 3.2 手机端业务层（Isar）

建议集合（Phase 1 实施）：

- `wallet_profiles`
- `wallet_settings`
- `tx_records`
- `admin_role_cache`
- `observed_accounts`
- `login_replay_cache`
- `login_policy`
- `sfid_bind_state`
- `session_meta`
- `user_profile`
- `app_kv_cache`

### 3.3 后端服务层（Postgres）

建议核心表（Phase 1 实施）：

- `tx_prepared`：替代 `PREPARED_STATE`
- `tx_runtime`：替代 `TX_STATE`
- `chain_bind_requests`：绑定请求审计与状态
- `admin_catalog_snapshot`（可选）：管理员目录短期快照
- `api_audit_logs`（可选）：关键接口审计

## 4. 映射矩阵（旧 -> 新）

### 4.1 手机端映射

| 旧存储 | 新存储 | 迁移策略 |
|---|---|---|
| `wallet.items` | Isar `wallet_profiles` | 全量一次迁移 |
| `wallet.active_index` | Isar `wallet_settings.active_wallet_id` | index -> wallet_id 映射后写入 |
| `wallet.*` 单钱包旧键 | Isar + secure | 一次性迁移，迁后删除旧键 |
| `wallet.mnemonic.<index>` | secure `wallet.secret.<wallet_id>.mnemonic.v1` | 复制后校验，再删除旧命名 |
| `trade.onchain.records` | Isar `tx_records` | 全量迁移 |
| `wallet.admin_catalog.*` | Isar `admin_role_cache` | 全量迁移 |
| `observe.accounts` | Isar `observed_accounts` | 全量迁移 |
| `login.used_request_ids` | Isar `login_replay_cache` | 保留 TTL 字段 |
| `login.whitelist_config.v1` | Isar `login_policy` | 全量迁移 |
| `attest.*` | secure + Isar | token 入 secure，元信息入 Isar |
| `sfid.bind.*` | Isar `sfid_bind_state` | 全量迁移 |
| `user.profile.*` | Isar `user_profile` | 全量迁移 |

### 4.2 后端映射

| 旧存储 | 新存储 | 迁移策略 |
|---|---|---|
| 内存 `PREPARED_STATE` | Postgres `tx_prepared` | 代码重构（不做历史迁移） |
| 内存 `TX_STATE` | Postgres `tx_runtime` | 代码重构（不做历史迁移） |
| 绑定请求无落库 | Postgres `chain_bind_requests` | 新增写入 |
| 静态 `admin_catalog_seed.json` | 保持静态 + 可选 Postgres 快照 | 先保留静态，再加快照（可选） |

## 5. Phase 0 输出物（本次完成）

1. 完成 mobile 与 backend 的实际存储盘点。  
2. 完成旧键/内存态到三层目标的映射表。  
3. 确认下一阶段实施顺序：先后端 Postgres，再手机端 Isar，再机密键重命名与清理。

## 6. Phase 1 开始前的确认项

1. 确认 Isar 最终集合名与索引（是否沿用本文命名）。
2. 确认 Postgres 表清单（是否启用 `admin_catalog_snapshot` 与 `api_audit_logs`）。
3. 确认迁移策略：新版本保留旧 SharedPreferences 读取 1 个发布周期。

