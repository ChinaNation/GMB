# 公民身份绑定改造方案

## 一、核心模型变更

### 当前模型（以公钥为主键）

```
bindings_by_pubkey: HashMap<公钥, BindingRecord>
```

- 公钥传入 → 生成 SFID 码 → 扫码绑定档案号 → 三者关联
- 解绑 = 清除档案号
- 删除 = 删除整条
- 解绑后记录仍以公钥为 key 存在

### 新模型（自增 ID 为主键，三字段各自唯一）

```
citizen_records: Vec<CitizenRecord>  // 或 HashMap<u64, CitizenRecord>
```

```rust
struct CitizenRecord {
    id: u64,                          // 主键，自增
    account_pubkey: Option<String>,   // 可为空（解绑后），唯一
    archive_no: Option<String>,       // 可为空（新公钥未绑定），唯一
    sfid_code: Option<String>,        // 可为空（新公钥未绑定），唯一
    sfid_signature: String,           // SFID 签名
    province_code: String,            // 省代码
    bound_at: Option<DateTime<Utc>>,  // 绑定时间
    bound_by: Option<String>,         // 操作人
    created_at: DateTime<Utc>,        // 记录创建时间
}
```

### 三种状态

| 状态 | account_pubkey | archive_no | sfid_code | 操作列 |
|------|---------------|------------|-----------|--------|
| 新公钥（未绑定） | ✓ | None | None | 绑定按钮 |
| 已绑定 | ✓ | ✓ | ✓ | 解绑按钮 |
| 已解绑（无公钥） | None | ✓ | ✓ | 绑定按钮 |

### 唯一约束

- `account_pubkey` UNIQUE（非空时）
- `archive_no` UNIQUE（非空时）
- `sfid_code` UNIQUE（非空时）

## 二、业务流程

### 绑定流程（状态 1 → 状态 2）：有公钥，需要档案号

1. 列表中有一条只有公钥的记录
2. 点击"绑定"→ 打开扫码弹窗
3. 扫描 CPMS 档案二维码（QR4）
4. 后端验证 QR4（anon_cert 签名 + archive_sig）
5. 从 QR4 提取 archive_no + province_code
6. SFID 生成 challenge
7. 用户用该公钥对 challenge 签名（wumin 冷钱包扫码签名）
8. 后端验签，确认公钥持有者同意绑定
9. 根据 archive_no + province_code 自动生成 SFID 码（A3=GMR，其他字段自动）
10. 写入 archive_no + sfid_code，状态变为"已绑定"

### 重新绑定流程（状态 3 → 状态 2）：有档案号，需要公钥

1. 列表中有一条只有档案号/SFID 码的记录
2. 点击"绑定"→ 打开输入公钥弹窗
3. 输入新公钥
4. SFID 生成 challenge
5. 用户用该公钥对 challenge 签名
6. 后端验签，确认公钥持有者同意绑定
7. 检查该公钥是否已被其他记录占用
8. 写入 account_pubkey，状态变为"已绑定"

### 解绑流程（状态 2 → 状态 3）

1. 点击"解绑"→ 确认弹窗
2. 将 account_pubkey 设为 None
3. 档案号和 SFID 码保留不变
4. 记录变为"已解绑"状态

## 三、后端改动

### 3.1 数据模型（models/mod.rs）

**去掉：**
- `BindingRecord`（替换为 `CitizenRecord`）
- `PendingBindScan`（扫码逻辑合并到绑定流程）
- `PendingRequest`
- Store 中的 `bindings_by_pubkey`、`pubkey_by_archive_index`、`pending_by_pubkey`、`pending_bind_scan_by_qr_id`、`generated_sfid_by_pubkey`

**新增：**
- `CitizenRecord`（新结构体）
- Store 中的 `citizen_records: HashMap<u64, CitizenRecord>`
- 三个反向索引：`citizen_id_by_pubkey`、`citizen_id_by_archive_no`、`citizen_id_by_sfid_code`

**改造 `CitizenRow`（列表返回）：**
```rust
pub(crate) struct CitizenRow {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) status: CitizenBindStatus,  // Unbound / Bound / Unlinked
}
```

### 3.2 API 端点（binding.rs）

**去掉：**
- `admin_bind_scan`（旧扫码流程）
- `admin_bind_confirm`（旧确认流程）
- `admin_unbind`（旧解绑）
- `admin_delete_citizen`（旧删除，与解绑合并了逻辑改变）
- 状态变更相关端点

**新增：**

| 端点 | 方法 | 说明 |
|------|------|------|
| `POST /api/v1/admin/citizen/bind` | 绑定 | 两种模式：有公钥绑档案 / 有档案绑公钥 |
| `POST /api/v1/admin/citizen/bind/challenge` | 生成 challenge | 绑定前获取签名 challenge |
| `POST /api/v1/admin/citizen/unbind` | 解绑 | 清除公钥，保留档案号+SFID码 |
| `GET /api/v1/admin/citizens` | 列表 | 返回三种状态的所有记录 |

**绑定端点合并逻辑：**
```
POST /api/v1/admin/citizen/bind
{
  // 模式 1：有公钥绑档案（扫码后提交）
  "mode": "bind_archive",
  "account_pubkey": "0x...",
  "qr4_payload": "...",          // QR4 二维码内容
  "challenge_id": "...",
  "signature": "..."             // 公钥对 challenge 的签名

  // 模式 2：有档案绑公钥
  "mode": "bind_pubkey",
  "citizen_id": 123,             // 记录 ID
  "account_pubkey": "0x...",
  "challenge_id": "...",
  "signature": "..."
}
```

### 3.3 路由（main.rs）

**去掉：**
- `/api/v1/admin/bind/scan`
- `/api/v1/admin/bind/confirm`
- `/api/v1/admin/bind/unbind`
- `/api/v1/admin/citizen/delete`
- `/api/v1/admin/sfid/generate`（SFID 生成合并到绑定流程）

**新增：**
- `/api/v1/admin/citizen/bind/challenge`
- `/api/v1/admin/citizen/bind`
- `/api/v1/admin/citizen/unbind`

**保留：**
- `/api/v1/admin/citizens`（列表查询）

### 3.4 SFID 生成逻辑

当前 `admin_generate_sfid` 是独立端点。改造后 SFID 生成内嵌到绑定流程中：

绑定模式 1（有公钥绑档案）时：
1. 验证 QR4
2. 验证公钥签名
3. 提取 archive_no + province_code
4. 调用 `generate_sfid_code(a3="GMR", province=从QR4取, ...)` 生成 SFID 码
5. 写入 CitizenRecord

不再作为独立操作暴露。

## 四、前端改动

### 4.1 API client（client.ts）

**去掉：**
- `confirmBind()`
- `unbind()`
- `deleteCitizen()`
- `generateSfid()`
- `scanBindQr()`

**新增：**
- `citizenBindChallenge(auth, { account_pubkey }) → { challenge_id, challenge_text }`
- `citizenBind(auth, { mode, ... }) → CitizenRecord`
- `citizenUnbind(auth, { citizen_id }) → string`

### 4.2 类型（client.ts）

**改造 `CitizenRow`：**
```typescript
export type CitizenRow = {
  id: number;
  account_pubkey?: string;
  archive_no?: string;
  sfid_code?: string;
  province_code?: string;
  status: 'UNBOUND' | 'BOUND' | 'UNLINKED';
};
```

### 4.3 UI 组件（App.tsx）

**表格列改造：**
- 序号、公钥（可空显示"-"）、档案号（可空显示"-"）、SFID码（可空显示"-"）、操作

**操作列：**
- UNBOUND（有公钥没档案）→ "绑定"按钮
- BOUND（三者都有）→ "解绑"按钮
- UNLINKED（有档案没公钥）→ "绑定"按钮

**绑定弹窗（两种模式）：**

模式 1（有公钥绑档案）：
- 扫码区域（扫 QR4）
- 扫码后自动填充 archive_no + province_code
- 显示 challenge 二维码 → 用户用 wumin 钱包签名 → 扫回签名
- 提交绑定

模式 2（有档案绑公钥）：
- 输入框：新公钥
- 显示 challenge 二维码 → 用户用该公钥签名 → 扫回签名
- 提交绑定

**去掉的 UI：**
- "生成公民身份识别码"弹窗（合并到绑定流程）
- "绑定身份"旧弹窗（替换为新弹窗）
- 状态变更扫码弹窗
- SFID码列的"生成"按钮

## 五、改造顺序

| 阶段 | 内容 |
|------|------|
| 1 | 后端 CitizenRecord 新模型 + Store 改造 |
| 2 | 后端 challenge + bind + unbind 三个新端点 |
| 3 | 后端路由更新 + 旧端点清理 |
| 4 | 后端 listCitizens 查询改造 |
| 5 | 前端 API client 改造 |
| 6 | 前端列表 + 绑定弹窗 + 解绑 UI |

## 六、向后兼容

Store 中已有的 `bindings_by_pubkey` 旧数据需要迁移到新的 `citizen_records`。启动时检测旧数据存在则自动迁移：

- 旧 `BindingRecord` 有 pubkey + archive_index + sfid_code → 迁移为 BOUND 状态
- 旧 `PendingRequest` 只有 pubkey → 迁移为 UNBOUND 状态
- 迁移完成后清除旧字段
