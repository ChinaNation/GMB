# CIIC 数据库结构（PostgreSQL）

## 1. 设计原则
- 绑定关系强一致：索引号唯一、公钥唯一。
- 审计可追溯：人工操作必须留痕。
- 凭证可核查：每次签发都有记录。

## 2. 表结构建议

### 2.1 `bind_requests`
用途：轻节点发起绑定申请。

字段：
- `id` `bigserial` PK
- `request_id` `varchar(64)` UNIQUE NOT NULL
- `account_pubkey` `varchar(128)` NOT NULL
- `chain_id` `varchar(32)` NOT NULL
- `client_nonce` `varchar(128)` NULL
- `status` `varchar(16)` NOT NULL
- `created_ip` `varchar(64)` NULL
- `created_at` `timestamptz` NOT NULL
- `expires_at` `timestamptz` NOT NULL
- `updated_at` `timestamptz` NOT NULL

索引：
- `idx_bind_requests_pubkey` (`account_pubkey`)
- `idx_bind_requests_status` (`status`)

### 2.2 `archive_bindings`
用途：保存索引号与公钥绑定关系。

字段：
- `id` `bigserial` PK
- `binding_id` `varchar(64)` UNIQUE NOT NULL
- `archive_index` `varchar(128)` NOT NULL
- `account_pubkey` `varchar(128)` NOT NULL
- `ciic_identity_hash` `varchar(128)` NOT NULL
- `status` `varchar(16)` NOT NULL
- `bound_by` `varchar(64)` NOT NULL
- `bound_at` `timestamptz` NOT NULL
- `updated_at` `timestamptz` NOT NULL

约束：
- `uq_archive_index_active`：`archive_index` 唯一
- `uq_pubkey_active`：`account_pubkey` 唯一

索引：
- `idx_archive_bindings_identity_hash` (`ciic_identity_hash`)
- `idx_archive_bindings_status` (`status`)

### 2.3 `credential_issues`
用途：记录绑定凭证/投票凭证签发流水。

字段：
- `id` `bigserial` PK
- `issue_id` `varchar(64)` UNIQUE NOT NULL
- `credential_type` `varchar(16)` NOT NULL
- `account_pubkey` `varchar(128)` NOT NULL
- `ciic_identity_hash` `varchar(128)` NOT NULL
- `proposal_id` `bigint` NULL
- `nonce_hash` `varchar(128)` NOT NULL
- `signature` `text` NOT NULL
- `key_id` `varchar(32)` NOT NULL
- `issued_at` `timestamptz` NOT NULL
- `expired_at` `timestamptz` NOT NULL

索引：
- `idx_credential_pubkey` (`account_pubkey`)
- `idx_credential_nonce_hash` (`nonce_hash`)
- `idx_credential_type_time` (`credential_type`, `issued_at`)

### 2.4 `admin_users`
用途：管理员账户与角色。

字段：
- `id` `bigserial` PK
- `user_id` `varchar(64)` UNIQUE NOT NULL
- `username` `varchar(64)` UNIQUE NOT NULL
- `password_hash` `varchar(255)` NOT NULL
- `role` `varchar(32)` NOT NULL
- `two_fa_secret` `varchar(255)` NULL
- `status` `varchar(16)` NOT NULL
- `last_login_at` `timestamptz` NULL
- `created_at` `timestamptz` NOT NULL
- `updated_at` `timestamptz` NOT NULL

### 2.5 `audit_logs`
用途：记录人工和关键自动动作。

字段：
- `id` `bigserial` PK
- `log_id` `varchar(64)` UNIQUE NOT NULL
- `operator_id` `varchar(64)` NULL
- `action` `varchar(64)` NOT NULL
- `target_type` `varchar(32)` NOT NULL
- `target_id` `varchar(64)` NOT NULL
- `result` `varchar(16)` NOT NULL
- `trace_id` `varchar(64)` NULL
- `detail` `jsonb` NULL
- `created_at` `timestamptz` NOT NULL

索引：
- `idx_audit_operator_time` (`operator_id`, `created_at`)
- `idx_audit_action_time` (`action`, `created_at`)

## 3. 状态值建议
- `bind_requests.status`：`PENDING | APPROVED | REJECTED | EXPIRED`
- `archive_bindings.status`：`ACTIVE | UNBOUND | SUSPENDED`
- `admin_users.status`：`ACTIVE | DISABLED`
- `audit_logs.result`：`SUCCESS | FAILED`

## 4. 一致性规则
1. 管理员确认绑定时事务内执行：
- 校验申请状态为 `PENDING`
- 校验 `archive_index` 未被占用
- 校验 `account_pubkey` 未被占用
- 写入 `archive_bindings`
- 更新 `bind_requests.status=APPROVED`
- 写入 `audit_logs`

2. 投票凭证签发前必须校验：
- `archive_bindings.status=ACTIVE`
- 申请公钥与绑定公钥一致

## 5. 生命周期与归档
- `bind_requests`：保留 180 天后归档。
- `credential_issues`：保留 365 天后冷归档。
- `audit_logs`：建议长期保留，不少于 3 年。

