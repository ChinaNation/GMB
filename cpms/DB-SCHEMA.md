# CPMS 数据库结构（离线内网版）

## 1. 设计原则
- 单主库强一致：所有客户端写入同一主机数据库。
- 档案唯一性：`archive_index_no`、`passport_no` 全局唯一。
- 审计可追溯：关键业务动作必须留痕。
- 软删除优先：核心业务数据默认不做物理删除。

## 2. 表结构建议（PostgreSQL）

### 2.1 `users`
用途：CPMS 登录账户（超级管理员/系统管理员）。

字段：
- `id` `bigserial` PK
- `user_id` `varchar(64)` UNIQUE NOT NULL
- `username` `varchar(64)` UNIQUE NOT NULL
- `password_hash` `varchar(255)` NOT NULL
- `role` `varchar(32)` NOT NULL
- `status` `varchar(16)` NOT NULL
- `failed_login_count` `int` NOT NULL DEFAULT 0
- `locked_until` `timestamptz` NULL
- `last_login_at` `timestamptz` NULL
- `created_by` `varchar(64)` NULL
- `created_at` `timestamptz` NOT NULL
- `updated_at` `timestamptz` NOT NULL

索引：
- `idx_users_role` (`role`)
- `idx_users_status` (`status`)

### 2.2 `citizen_archives`
用途：公民档案主表。

字段：
- `id` `bigserial` PK
- `archive_id` `varchar(64)` UNIQUE NOT NULL
- `archive_index_no` `varchar(32)` UNIQUE NOT NULL
- `passport_no` `varchar(32)` UNIQUE NOT NULL
- `full_name` `varchar(128)` NOT NULL
- `birth_date` `date` NOT NULL
- `gender_code` `varchar(1)` NOT NULL
- `height_cm` `numeric(5,2)` NULL
- `province_code` `varchar(2)` NOT NULL
- `status` `varchar(16)` NOT NULL
- `remark` `text` NULL
- `created_by` `varchar(64)` NOT NULL
- `updated_by` `varchar(64)` NOT NULL
- `created_at` `timestamptz` NOT NULL
- `updated_at` `timestamptz` NOT NULL
- `deleted_at` `timestamptz` NULL

索引：
- `idx_archives_name` (`full_name`)
- `idx_archives_birth_gender` (`birth_date`, `gender_code`)
- `idx_archives_status` (`status`)

### 2.3 `archive_sequence_counters`
用途：按“省代码 + 性别 + 生日”维护档案编号流水，生成索引号末 6 位。

字段：
- `id` `bigserial` PK
- `province_code` `varchar(2)` NOT NULL
- `gender_code` `varchar(1)` NOT NULL
- `birth_yyyymmdd` `varchar(8)` NOT NULL
- `next_seq` `int` NOT NULL
- `updated_at` `timestamptz` NOT NULL

约束：
- `uq_archive_seq_counter` UNIQUE (`province_code`, `gender_code`, `birth_yyyymmdd`)

### 2.4 `biometric_assets`
用途：照片/指纹文件索引与版本记录。

字段：
- `id` `bigserial` PK
- `asset_id` `varchar(64)` UNIQUE NOT NULL
- `archive_id` `varchar(64)` NOT NULL
- `asset_type` `varchar(16)` NOT NULL
- `file_path` `text` NOT NULL
- `file_sha256` `varchar(64)` NOT NULL
- `file_size` `bigint` NOT NULL
- `mime_type` `varchar(64)` NOT NULL
- `version_no` `int` NOT NULL
- `status` `varchar(16)` NOT NULL
- `created_by` `varchar(64)` NOT NULL
- `created_at` `timestamptz` NOT NULL
- `deleted_at` `timestamptz` NULL

索引：
- `idx_assets_archive_type` (`archive_id`, `asset_type`)
- `idx_assets_status` (`status`)

### 2.5 `archive_materials`
用途：档案材料（证件扫描件、附表等）索引。

字段：
- `id` `bigserial` PK
- `material_id` `varchar(64)` UNIQUE NOT NULL
- `archive_id` `varchar(64)` NOT NULL
- `material_type` `varchar(32)` NOT NULL
- `title` `varchar(255)` NOT NULL
- `file_path` `text` NOT NULL
- `file_sha256` `varchar(64)` NOT NULL
- `file_size` `bigint` NOT NULL
- `version_no` `int` NOT NULL
- `status` `varchar(16)` NOT NULL
- `created_by` `varchar(64)` NOT NULL
- `created_at` `timestamptz` NOT NULL
- `deleted_at` `timestamptz` NULL

索引：
- `idx_materials_archive_type` (`archive_id`, `material_type`)
- `idx_materials_status` (`status`)

### 2.6 `audit_logs`
用途：审计日志（登录、权限变更、档案操作、导入导出）。

字段：
- `id` `bigserial` PK
- `log_id` `varchar(64)` UNIQUE NOT NULL
- `operator_user_id` `varchar(64)` NULL
- `action` `varchar(64)` NOT NULL
- `target_type` `varchar(32)` NOT NULL
- `target_id` `varchar(64)` NULL
- `result` `varchar(16)` NOT NULL
- `trace_id` `varchar(64)` NULL
- `client_host` `varchar(128)` NULL
- `detail` `jsonb` NULL
- `prev_hash` `varchar(64)` NULL
- `curr_hash` `varchar(64)` NOT NULL
- `created_at` `timestamptz` NOT NULL

索引：
- `idx_audit_operator_time` (`operator_user_id`, `created_at`)
- `idx_audit_action_time` (`action`, `created_at`)

### 2.7 `backup_records`
用途：备份与恢复记录。

字段：
- `id` `bigserial` PK
- `backup_id` `varchar(64)` UNIQUE NOT NULL
- `operation` `varchar(16)` NOT NULL
- `package_path` `text` NOT NULL
- `package_sha256` `varchar(64)` NOT NULL
- `result` `varchar(16)` NOT NULL
- `operator_user_id` `varchar(64)` NOT NULL
- `detail` `jsonb` NULL
- `created_at` `timestamptz` NOT NULL

## 3. 状态值建议
- `users.role`：`SUPER_ADMIN | ADMIN`
- `users.status`：`ACTIVE | DISABLED | LOCKED`
- `citizen_archives.status`：`ACTIVE | SUSPENDED | ARCHIVED`
- `biometric_assets.status`：`ACTIVE | REPLACED | DELETED`
- `archive_materials.status`：`ACTIVE | REPLACED | DELETED`
- `audit_logs.result`：`SUCCESS | FAILED`
- `backup_records.operation`：`BACKUP | RESTORE`

## 4. 索引号生成规则
- 规则：`省代码(2位大写字母) + 性别码(M/W) + 生日码(YYYYMMDD) + 档案编号(6位)`
- 正则：`^[A-Z]{2}(M|W)[0-9]{8}[0-9]{6}$`
- 生成过程（事务内）：
1. 校验省代码、性别码、生日格式。
2. 锁定 `archive_sequence_counters` 对应行（不存在则初始化）。
3. 取 `next_seq`，左补零成 6 位。
4. 拼接索引号并写入 `citizen_archives`。
5. `next_seq` 自增并提交事务。

## 5. 一致性规则
1. 新建档案事务：
- 生成索引号
- 校验 `passport_no` 唯一
- 写入档案
- 写入审计日志

2. 替换照片/指纹事务：
- 旧记录标记 `REPLACED`
- 新记录写入 `ACTIVE` + `version_no + 1`
- 写入审计日志

3. 删除策略：
- 档案、资产、材料默认软删除（`deleted_at`）
- 审计日志与备份记录禁止业务层删除
