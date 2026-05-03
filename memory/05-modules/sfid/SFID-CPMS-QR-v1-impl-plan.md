# SFID_CPMS_V1 改造方案（已完成）

> 基于 SFID-CPMS-QR-v1.md 定稿协议的实施计划。
>
> **状态：已完成。** 协议已从 `SFID-CPMS QR v1` 统一为 `SFID_CPMS_V1`，字段已精简（见 SFID-CPMS-QR-v1.md）。
> 以下为历史实施记录，字段名以最新协议文档为准。

## 一、SFID 后端改造

### 1.1 新增 RSA 密钥管理

**新文件：** `sfid/backend/cpms/rsa_blind.rs`

- 生成 RSA 密钥对（启动时从环境变量 `SFID_ANON_RSA_PRIVATE_KEY_PEM` 加载，或首次启动时生成并持久化）
- 实现 RSABSSA-SHA384-PSS-Randomized（RFC 9474）盲签名
- 暴露两个函数：
  - `blind_sign(blind_anon_req: &[u8]) -> Vec<u8>` — 对盲化消息签名
  - `verify_anon_cert(message: &str, signature: &[u8]) -> bool` — 验证匿名证书签名
- RSA 公钥需可导出（供 CPMS 预置或通过 QR1 携带）

**依赖：** `rsa` crate + `blind-rsa-signatures` crate（Rust 实现 RFC 9474）

### 1.2 数据模型变更

**文件：** `sfid/backend/cpms/model.rs`

**CpmsSiteKeys 改造：**

去掉字段：
- `pubkey_1`
- `pubkey_2`
- `pubkey_3`
- `init_qr_payload`
- `chain_register_tx_hash`
- `chain_register_block_number`
- `chain_register_at`

新增字段：
- `install_token: String` — UUID 一次性安装令牌
- `install_token_status: InstallTokenStatus` — PENDING / USED / REVOKED

保留字段：
- `site_sfid`
- `status`（语义改为：PENDING=已生成未注册, ACTIVE=已注册, DISABLED, REVOKED）
- `version`
- `admin_province`
- `created_by`, `created_at`, `updated_by`, `updated_at`

**新增枚举：**

```rust
enum InstallTokenStatus {
    Pending,  // 已生成，未使用
    Used,     // 注册成功，已消费
    Revoked,  // 管理员手工作废
}
```

**新增结构体：**

```rust
struct ImportedArchive {
    archive_no: String,        // PK
    province_code: String,     // 以 anon_cert 内为准
    anon_cert_hash: String,    // SHA-256 摘要
    imported_at: DateTime<Utc>,
    status: ArchiveImportStatus, // ACTIVE / REVOKED
}
```

**清理：** 去掉 `UpdateCpmsKeysInput`、`CpmsRegisterQrPayload`、`CpmsInstitutionInitClaims` 等旧结构体

### 1.3 API 端点变更

**文件：** `sfid/backend/cpms/handler.rs` + `sfid/backend/main.rs`

#### 去掉的端点

| 端点 | 说明 |
|------|------|
| `PUT /api/v1/admin/cpms-keys/{site_sfid}` | 更新公钥 1/2/3 |
| `POST /api/v1/admin/cpms-keys/register-scan` | 旧注册流程 |
| `POST /api/v1/admin/cpms-status/scan` | 旧状态扫码 |

#### 改造的端点

**`POST /api/v1/admin/cpms-keys/sfid/generate`**

改造点：
- 生成 `install_token`（UUID v4）
- 不再生成旧的 `CpmsInstitutionInitClaims` 和 QR payload
- 改为构造 QR1 payload：
  ```json
  {
    "ver": 1,
    "qr_type": "SFID_CPMS_INSTALL",
    "site_sfid": "...",
    "install_token": "...",
    "signature": "sr25519 签名"
  }
  ```
- 签名原文：`sfid-cpms-install-v1|{site_sfid}|{install_token}`
- 用 SFID 主密钥（现有 `SFID_SIGNING_SEED_HEX`）签名
- 存入 `CpmsSiteKeys`，`install_token_status = PENDING`
- 返回 `site_sfid` + QR1 JSON

#### 新增的端点

**`POST /api/v1/admin/cpms/register`** — 处理 QR2，返回 QR3

逻辑：
1. 解析 QR2 JSON（`qr_type == CPMS_REGISTER_REQ`）
2. 查找 `site_sfid`，校验 `install_token` 状态为 PENDING
3. 从 `site_sfid` 提取 `province_code`（r5 段前两位）
4. 构造盲签名原文的公共部分：`sfid-anon-cert-v1|{province_code}`
5. 对 `blind_anon_req` 执行 RSABSSA 盲签名 → `blind_anon_sig`
6. 标记 `install_token_status = USED`，`status = ACTIVE`
7. 构造 QR3 返回：
   ```json
   {
     "ver": 1,
     "qr_type": "SFID_ANON_CERT",
     "province_code": "GD",
     "blind_anon_sig": "..."
   }
   ```
8. 审计日志

**`POST /api/v1/admin/cpms/archive/import`** — 处理 QR4，录入档案

逻辑：
1. 解析 QR4 JSON（`qr_type == CPMS_ARCHIVE_QR`）
2. 验证步骤（按协议顺序）：
   - 用 RSA 公钥验 `anon_cert.sfid_sig`，原文 `sfid-anon-cert-v1|{anon_cert.province_code}|{anon_cert.anon_pubkey}`
   - 验 `anon_cert.province_code == qr.province_code`
   - 用 `anon_cert.anon_pubkey`（sr25519）验 `archive_sig`，原文 `cpms-archive-qr-v1|{province_code}|{archive_no}|{citizen_status}|{voting_eligible}`
   - 查 `imported_archives` 表验 `archive_no` 未录入过
3. 全通过后写入 `imported_archives`：
   - `province_code` = `anon_cert.province_code`（不信任顶层字段）
   - `anon_cert_hash` = SHA-256 of anon_cert JSON
4. 审计日志
5. 返回录入结果

**`POST /api/v1/admin/cpms-keys/{site_sfid}/revoke-token`** — 作废安装令牌

逻辑：
1. 校验 `site_sfid` 存在
2. 将 `install_token_status` 设为 REVOKED
3. 审计日志

**`POST /api/v1/admin/cpms-keys/{site_sfid}/reissue`** — 重新签发安装令牌

逻辑：
1. 校验 `site_sfid` 存在，当前 token 状态为 USED 或 REVOKED
2. 生成新的 `install_token`
3. 设 `install_token_status = PENDING`
4. 构造新 QR1 返回
5. 审计日志

#### 改造的端点

**`GET /api/v1/admin/cpms-keys`** — 列表

返回字段改为：
- `site_sfid`
- `install_token_status`（PENDING / USED / REVOKED）
- `status`（PENDING / ACTIVE / DISABLED / REVOKED）
- `province_code`
- `created_by`, `created_at`

去掉 `pubkey_1/2/3`。

### 1.4 前端变更

**文件：** `sfid/frontend/src/components/App.tsx`

#### 表格列改造

去掉：
- 公钥1 列 + "更新"按钮
- 公钥2 列 + "更新"按钮
- 公钥3 列 + "更新"按钮

改为：
- `安装令牌状态` 列（显示 PENDING/USED/REVOKED，颜色区分）
- `操作` 列：作废令牌 / 重新签发 / 查看 QR1

#### 生成 SFID 后

- 不再显示空的公钥列
- 显示 QR1 二维码（安装授权码）
- 可复制 install_token

#### 新增扫码功能

- "扫描注册请求"按钮 → 扫 QR2 → 调 `/cpms/register` → 页面展示 QR3 二维码供 CPMS 扫描
- "扫描档案二维码"按钮 → 扫 QR4 → 调 `/cpms/archive/import` → 显示录入结果

**文件：** `sfid/frontend/institutions/api.ts`

去掉：
- `updateCpmsKeys()`
- `registerCpmsKeysScan()`

新增：
- `registerCpms(auth, { qr_payload })` → POST `/cpms/register`
- `importArchive(auth, { qr_payload })` → POST `/cpms/archive/import`
- `revokeInstallToken(auth, siteSfid)` → POST `/cpms-keys/{siteSfid}/revoke-token`
- `reissueInstallToken(auth, siteSfid)` → POST `/cpms-keys/{siteSfid}/reissue`

类型改造：
- `CpmsSiteRow` 去掉 `pubkey_1/2/3`，加 `install_token_status`

---

## 二、CPMS 后端改造

### 2.1 安装流程改造

**文件：** `cpms/backend/src/initialize/mod.rs`

#### 当前逻辑（保留）

- 接收 SFID 安装二维码
- 验证 SFID 签名（`SFID_ROOT_PUBKEY`）
- 存储 `site_sfid`
- 生成 K1/K2/K3 sr25519 签名密钥（用于旧的 QR 签名）

#### 新增逻辑

在现有安装流程中新增：

1. **验证 QR1 签名** — 签名原文改为 `sfid-cpms-install-v1|{site_sfid}|{install_token}`
2. **生成 anon_keypair** — sr25519 密钥对，私钥加密存储在本地（与现有 K1/K2/K3 同样的加密方式）
3. **生成 blind_anon_req** — 对 `anon_pubkey` 做 RSABSSA 盲化（需预置 SFID RSA 公钥）
4. **构造 QR2** 展示给管理员
5. **存储 install_token** 到 `system_install` 表

**新增数据库字段（system_install 表）：**
- `install_token` — 安装令牌
- `anon_pubkey` — 匿名公钥（hex）
- `anon_cert` — 完整匿名证书 JSON（QR3 解盲后写入）
- `anon_key_encrypted` — 匿名私钥（加密存储）
- `blinding_factor` — RSABSSA 盲化因子（解盲前暂存，解盲后清除）

**新增依赖：** `blind-rsa-signatures` crate

### 2.2 QR3 处理

**新增端点或安装流程第二步：**

CPMS 管理员扫描 SFID 返回的 QR3 后：

1. 解析 QR3（`qr_type == SFID_ANON_CERT`）
2. 用本地存储的 `blinding_factor` 对 `blind_anon_sig` 解盲
3. 得到最终匿名证书：
   ```json
   {
     "province_code": "GD",
     "anon_pubkey": "0x...",
     "sfid_sig": "..."
   }
   ```
4. 用预置的 SFID RSA 公钥验证 `sfid_sig`，原文 `sfid-anon-cert-v1|{province_code}|{anon_pubkey}`
5. 验证通过后持久化 `anon_cert` 到 `system_install`
6. 清除 `blinding_factor`
7. 安装完成

### 2.3 档案号生成改造

**文件：** `cpms/backend/src/dangan/mod.rs`

#### 当前档案号格式（V3）

```
{province_code}{city_code}{check_digit}{random9}{date_yyyymmdd}
```

含省、市、日期信息 — 不符合协议要求。

#### 改为新格式

```
AR4-<26位Base32随机体>-<2位校验>
```

**改造 `generate_archive_no_with_retry()`：**
- 去掉 `province_code`、`city_code`、`created_date_yyyymmdd` 参数
- 用安全随机数生成 26 位 Base32 随机体（`A-Z` + `2-7`，130 位熵）
- 计算 2 位校验码（Blake2b 摘要取模）
- 拼接为 `AR4-{random26}-{check2}`
- 碰撞检测逻辑保留

**改造 `archive_checksum_digit()` → `archive_checksum_2()`：**
- 输入：26 位随机体
- 输出：2 位校验字符

### 2.4 QR4 构造改造

**文件：** `cpms/backend/src/dangan/mod.rs`

**改造 `build_qr_payload()`：**

当前：用 K1/K2/K3 签名密钥签，载荷含 `site_sfid`、`sign_key_id` 等实名信息。

改为：
1. 从 `system_install` 读取 `anon_cert` 和 `anon_key`（解密）
2. 从 `anon_cert` 提取 `province_code`
3. 构造签名原文：`cpms-archive-qr-v1|{province_code}|{archive_no}|{citizen_status}|{voting_eligible}`
4. 用 anon 私钥（sr25519）签名 → `archive_sig`
5. 构造 QR4：
   ```json
   {
     "ver": 1,
     "qr_type": "CPMS_ARCHIVE_QR",
     "province_code": "GD",
     "archive_no": "AR4-...",
     "citizen_status": "NORMAL",
     "voting_eligible": true,
     "anon_cert": { ... },
     "archive_sig": "..."
   }
   ```

去掉：`site_sfid`、`sign_key_id`、`issuer_id`、`issued_at`、`expire_at`、`qr_id` 等旧字段。

### 2.5 K1/K2/K3 密钥的处置

现有的 K1（PRIMARY）、K2（BACKUP）、K3（EMERGENCY）三把 sr25519 密钥：

- **保留**用于 CPMS 内部管理（管理员登录签名验证、本地操作审计）
- **不再用于** QR4 档案签名（改用 anon_keypair）
- 不再通过 `SiteKeyRegistrationPayload` 注册回 SFID

### 2.6 数据库 Schema 变更

**文件：** `cpms/backend/db/migrations/` 新增迁移文件

```sql
-- system_install 表新增字段
ALTER TABLE system_install ADD COLUMN install_token TEXT;
ALTER TABLE system_install ADD COLUMN anon_pubkey TEXT;
ALTER TABLE system_install ADD COLUMN anon_cert JSONB;
ALTER TABLE system_install ADD COLUMN anon_key_encrypted BYTEA;
ALTER TABLE system_install ADD COLUMN blinding_factor BYTEA;

-- archives 表 archive_no 格式变更（新记录用 AR4 格式，旧数据保留）
-- 无需 ALTER，格式由应用层控制
```

---

## 三、改造顺序

| 阶段 | 内容 | 模块 |
|------|------|------|
| 1 | RSA 盲签名模块（签名 + 验签 + 盲化 + 解盲） | SFID 后端 + CPMS 后端共用 |
| 2 | SFID 数据模型改造（去掉 pubkey，加 install_token） | SFID 后端 |
| 3 | SFID generate 端点改造（生成 QR1） | SFID 后端 |
| 4 | SFID register 端点新增（处理 QR2 → 返回 QR3） | SFID 后端 |
| 5 | SFID archive/import 端点新增（处理 QR4） | SFID 后端 |
| 6 | SFID 前端改造（去掉公钥列，加令牌状态，新增扫码） | SFID 前端 |
| 7 | CPMS 安装流程改造（QR1 验签，生成 anon_keypair，构造 QR2） | CPMS 后端 |
| 8 | CPMS QR3 处理（解盲，存证书） | CPMS 后端 |
| 9 | CPMS 档案号格式改造（V3 → AR4） | CPMS 后端 |
| 10 | CPMS QR4 构造改造（用 anon_keypair 签名） | CPMS 后端 |

---

## 四、文件清单

### SFID 后端

| 文件 | 操作 |
|------|------|
| `sfid/backend/cpms/rsa_blind.rs` | 新建 — RSA 盲签名模块 |
| `sfid/backend/cpms/model.rs` | 改造 — 去掉 pubkey，加 install_token，新增 ImportedArchive |
| `sfid/backend/cpms/handler.rs` | 改造 — generate 改为 QR1，去掉旧 register/update，新增 register + archive/import + revoke + reissue |
| `sfid/backend/main.rs` | 改造 — 新增路由 |
| `sfid/backend/Cargo.toml` | 改造 — 新增 rsa / blind-rsa-signatures 依赖 |

### SFID 前端

| 文件 | 操作 |
|------|------|
| `sfid/frontend/institutions/api.ts` | 改造 — 去掉 updateCpmsKeys/registerScan，新增 register/import/revoke/reissue |
| `sfid/frontend/cpms/CpmsSitePanel.tsx` / `CpmsRegisterModal.tsx` | 改造 — 去掉公钥列，改为令牌状态，新增扫码入口 |

### CPMS 后端

| 文件 | 操作 |
|------|------|
| `cpms/backend/src/initialize/mod.rs` | 改造 — QR1 验签改造，新增 anon_keypair 生成 + 盲化 + QR3 解盲 |
| `cpms/backend/src/dangan/mod.rs` | 改造 — 档案号 V3→AR4，QR 签名改用 anon_keypair |
| `cpms/backend/src/main.rs` | 改造 — 新增 QR3 处理路由 |
| `cpms/backend/Cargo.toml` | 改造 — 新增 blind-rsa-signatures 依赖 |
| `cpms/backend/db/migrations/0002_anon_cert.sql` | 新建 — schema 变更 |
