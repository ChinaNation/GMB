# 任务卡:省级管理员独立推链密钥(简化版)

- **任务 ID**: `20260409-sfid-sheng-admin-per-province-keyring`
- **模块**: citizenchain-runtime + sfid-backend + sfid-frontend
- **优先级**: 高(解决 50K SHI_ADMIN 并发推链的单 nonce 瓶颈)
- **前置依赖**: 无
- **状态**: 待启动
- **预估工作量**: 2~2.5 天

## 背景

当前状态:
- 链端 `sfid_system` pallet 的 verifier 只信任这一个账户
- 省级管理员管理通过 `replace_sheng_admin` HTTP handler 进行,但**不涉及链上权限**

目标(用户拍板的简化方案):
- 保留现有 43 省 × 1 管理员的数据模型
- 每省管理员拥有**唯一一把**推链密钥(1 key,非多备份)
- 该密钥**公钥注册到链上**,作为链端授权 signer
  同时后端生成新私钥并替换 Store 里旧的
- SHI_ADMIN 推链时,后端自动用该 SHI_ADMIN 所属省的 sheng admin private key 签名

信任模型说明:
- 省级密钥的独立性主要体现在**链上 nonce 并行**(43 条通道)和**审计可追溯**

铁律参照:
- `feedback_chainspec_frozen.md`:chainspec 创世后冻结,runtime 升级走链上 `setCode`
- `feedback_sfid_pow_chain_recipe.md`:显式 nonce + immortal + InBestBlock
- `feedback_no_chain_restart.md`:链数据必须保留
- `feedback_scale_domain_must_be_array.md`:链端固定长度用 `[u8; N]`

## 架构总览

```
┌──────────────────────────────────────────────────────────────┐
│                  citizenchain runtime                        │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  sfid_system pallet                                 │  │
│  │  Storage:                                              │  │
│  │    ShengAdminPubkey:                                   │  │
│  │      map Province → [u8; 32]           ← 新增           │  │
│  │  Extrinsic:                                            │  │
│  │    set_sheng_admin_pubkey(province, new_pub)           │  │
│  │      effect: 覆写该省 pubkey(替换/初始化一体)         │  │
│  │  Verifier(register_sfid_institution 等):              │  │
│  │                 ShengAdminPubkey::iter().value         │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                            ▲
                            │ 签名交易(sr25519)
                            │
┌──────────────────────────────────────────────────────────────┐
│                    sfid-backend                              │
│  Store 新增:                                                 │
│    sheng_admin_signer: HashMap<Province, ShengSignerRow>     │
│      { province, pubkey, privkey_encrypted,                  │
│        chain_version, updated_at, updated_by }               │
│  内存缓存:                                                   │
│    内存中 PairSigner 按 province 索引                         │
│  签名路由(resolve_chain_signer):                             │
│    ShengAdmin 推链 → 本省 sheng signer                       │
│    ShiAdmin   推链 → 本省 sheng signer  ← 解决瓶颈核心       │
│    POST /api/v1/admin/sheng-signer/:province/init            │
│    POST /api/v1/admin/sheng-signer/:province/rotate          │
│    GET  /api/v1/admin/sheng-signer/list                      │
│    POST /api/v1/admin/sheng-signer/unlock(启动解锁)          │
└──────────────────────────────────────────────────────────────┘
                            ▲
                            │ HTTP
                            │
┌──────────────────────────────────────────────────────────────┐
│                    sfid-frontend                             │
│  views/keyring/ShengSignerPanel.tsx(新)                     │
│    43 省列表 + 每省 pubkey / 初始化状态 / 链上版本            │
│    操作:初始化 / 轮换                                         │
└──────────────────────────────────────────────────────────────┘
```

## 阶段划分

### 阶段 1 — 链端 runtime 修改(0.75 天)

**文件**:`citizenchain/frame/sfid-system/src/lib.rs`

**1.1 新增 storage**:
```rust
#[pallet::storage]
pub type ShengAdminPubkey<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    BoundedVec<u8, ConstU32<64>>,  // province UTF-8 字节
    [u8; 32],                      // sr25519 pub(固定数组,见 feedback_scale_domain_must_be_array)
    OptionQuery,
>;
```

**1.2 新增 extrinsic**:
```rust
#[pallet::call_index(N)]
#[pallet::weight(10_000)]
pub fn set_sheng_admin_pubkey(
    origin: OriginFor<T>,
    province: BoundedVec<u8, ConstU32<64>>,
    new_pubkey: [u8; 32],
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    ensure!(
    );
    ensure!(
    );
    // 覆写(替换 = 轮换)
    ShengAdminPubkey::<T>::insert(&province, new_pubkey);
    Self::deposit_event(Event::ShengAdminPubkeyUpdated {
        province,
        new_pubkey,
    });
    Ok(())
}
```

**1.3 修改 verifier**:

找到现有 `register_sfid_institution` 等 extrinsic 中校验 `origin` 的地方,扩展为:

```rust
fn is_authorized_sfid_writer<T: Config>(who: &T::AccountId) -> bool {
    let who_bytes = who.encode();
    if who_bytes == ka.main.encode()
        || who_bytes == ka.backup_a.encode()
        || who_bytes == ka.backup_b.encode()
    {
        return true;
    }
    // 2. 任一省 sheng admin pubkey
    for (_, pub_bytes) in ShengAdminPubkey::<T>::iter() {
        if who_bytes == pub_bytes.encode() {
            return true;
        }
    }
    false
}
```

**1.4 runtime version bump**:
- `runtime/src/lib.rs` 里 `spec_version +1`
- 产出 `runtime.compact.compressed.wasm`

**1.5 升级流程**:
- 本地 `cargo build --release`
- 通过 `system.setCode(new_wasm)` 链上升级(`feedback_chainspec_frozen`)

**1.6 单元测试**:
- 老 extrinsic 升级后旧签名方式仍工作

**验收**:
- `cargo build --release` 绿
- 本地节点 setCode 升级成功
- polkadot.js 手动调 `set_sheng_admin_pubkey` 成功
- 新 signer 推 `register_sfid_institution` 成功

---

### 阶段 2 — sfid-backend(1 天)

**2.1 Store 新增字段**(`sfid/backend/src/models/mod.rs`):

```rust
#[serde(default)]
pub(crate) sheng_admin_signer: HashMap<String, ShengSignerRow>,

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShengSignerRow {
    pub(crate) province: String,
    pub(crate) pubkey: String,              // hex, 32 字节
    pub(crate) privkey_encrypted: String,   // base64(ciphertext + nonce + tag)
    /// 链上该省 pubkey 当前版本(每次 set_sheng_admin_pubkey 递增,用于
    /// 检测后端与链上是否同步)
    #[serde(default)]
    pub(crate) chain_version: u32,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
}
```


```rust
/// 省级 signer 内存缓存,进程生命周期内保持。
pub(crate) struct ShengSignerCache {
    inner: RwLock<HashMap<String, PairSigner<PolkadotConfig, sr25519::Pair>>>,
    unlocked: AtomicBool,
}

impl ShengSignerCache {
    pub fn new() -> Self { ... }
    pub fn is_unlocked(&self) -> bool { self.unlocked.load(Relaxed) }
    /// 一次性解密所有省私钥到内存。
    pub fn unlock(&self, store: &Store, derived_key: &[u8; 32]) -> Result<usize> { ... }
    pub fn get(&self, province: &str) -> Option<PairSigner<...>> { ... }
    /// 替换某省 signer(轮换时使用),不需要 unlock 权限
    pub fn replace(&self, province: String, signer: PairSigner<...>) { ... }
    pub fn clear(&self) { ... }
}
```

**2.3 加密方案**:

- 对称算法:AES-256-GCM


```
POST /api/v1/admin/sheng-signer/unlock
  body: { challenge_id, signature }
  action: 后端验签 → 派生对称密钥 → 解密所有省 privkey → 载入内存缓存
          响应返回已解锁的省数量

POST /api/v1/admin/sheng-signer/:province/init
  body: { }  ← 无输入,后端自己生成新 sr25519 keypair
  action: 1. 生成新 keypair
          2. 加密 privkey 存 Store
          3. 调用链端 set_sheng_admin_pubkey(province, pubkey)
          4. 成功后更新 chain_version,内存缓存加入该省 signer
          5. 返回 pubkey + chain_version + tx_hash

POST /api/v1/admin/sheng-signer/:province/rotate
  body: { }
  action: 同 init,但是覆盖已有行(替换 = 轮换)
  注意: 旧 pubkey 从此在链上不再被认证,老 SHI_ADMIN 推链会自动用新 pubkey

GET /api/v1/admin/sheng-signer/list
  response: [{ province, pubkey, chain_version, updated_at, initialized }]
```

**2.5 签名路由改造**(核心):

当前 `sheng-admins/institutions.rs::submit_register_sfid_institution_extrinsic` 里:
```rust
```

改成:
```rust
let signer = resolve_chain_signer(&state, &ctx)?;
```

```rust
pub(crate) fn resolve_chain_signer(
    state: &AppState,
    ctx: &AdminCtx,
) -> Result<PairSigner<PolkadotConfig, sr25519::Pair>, ApiError> {
    match ctx.role {
        }
        AdminRole::ShengAdmin | AdminRole::ShiAdmin => {
            let province = ctx.admin_province.as_ref()
                .ok_or_else(|| ApiError::bad_request("admin missing province"))?;
            // 先查内存缓存
            if let Some(signer) = state.sheng_signer_cache.get(province) {
                return Ok(signer);
            }
            tracing::warn!(
                province,
                role = ?ctx.role,
            );
        }
    }
}
```

**2.6 启动钩子**:


**2.7 Store 序列化 + 兼容**:
- `sheng_admin_signer` 字段 `#[serde(default)]`,老快照无此字段时为空 map
- 完成率可通过 `GET /sheng-signer/list` 观察

**2.8 密钥轮换流程**:

```
  ↓
后端生成新 sr25519 keypair
  ↓
加密新 privkey → 写 Store(覆盖旧行)
  ↓
调 set_sheng_admin_pubkey(province="辽宁省", new_pub=...) 链上交易
  ↓
等 InBestBlock
  ↓
内存缓存 cache.replace(province, new_signer)
  ↓
返回前端 tx_hash + new_pubkey + chain_version+1
```

老 pubkey 从此刻起在链端 verifier 里不再匹配,但已在链上的交易不受影响(签名已验)。

**2.9 验收**:
- `cargo check` + 单元测试绿
- 集成测试:unlock → init 辽宁 → 本省 SHI_ADMIN 推 `register_sfid_institution` 成功 →
  检查链上 extrinsic signer = sheng admin pubkey
- 集成测试:rotate 辽宁 → 新 pubkey 生效 → 旧 pubkey 推链被链端拒绝

---

### 阶段 3 — sfid-frontend(0.5 天)

**3.1 新建 `src/views/keyring/ShengSignerPanel.tsx`**:


内容:
- 顶部:解锁状态 Banner
  - 🔒 未解锁 → 按钮"解锁省级密钥"(调 unlock)
  - 🔓 已解锁 → 绿色"已解锁,X 省可用"
- 43 省 Table,列:
  - 省份名
  - 公钥(hex 前 16 位 + 复制按钮)
  - 链上版本(chain_version)
  - 最后更新时间
  - 初始化状态(Tag:🔵 未初始化 / 🟢 已初始化)
  - 操作:`初始化` / `轮换`

**3.2 初始化/轮换确认弹窗**:
- 无输入框(后端自己生成 keypair)
- 二次确认:"确认初始化 `辽宁省` 的推链密钥?此操作将上链"
- 轮换特别警告:"轮换后旧 pubkey 立即失效,本省所有 SHI_ADMIN 未完成的推链请求需要重试"
- 提交后显示 loading → 返回新 pubkey + tx_hash + chain_version

**3.3 API 客户端**(`src/api/shengSigner.ts`):
```ts
export async function unlockShengSigner(auth): Promise<{ unlocked_count: number }>
export async function listShengSigners(auth): Promise<ShengSignerRow[]>
export async function initShengSigner(auth, province): Promise<ShengSignerRow>
export async function rotateShengSigner(auth, province): Promise<ShengSignerRow>
```

**3.4 验证**:
- `npx tsc --noEmit` + `npm run build` 全绿
  本省 SHI_ADMIN 登录 → 推 `register_sfid_institution` → 链上成功

---

### 阶段 4 — 联调 + 压测 + 切换(0.5 天)

**4.1 灰度切换**:

后端环境变量 `SFID_SHENG_SIGNER_ENABLED=true|false`(默认 false,灰度期间)
- `true`:按角色路由

第一次生产发布先在灰度节点 `true`,验证稳定后切换全量。

**4.2 压测**:
- 500 并发 SHI_ADMIN(模拟 30 省的 SHI_ADMIN 同时推链)
- 对比 flag on vs off 下的 p50 / p99 延迟、失败率
- 目标:flag on 后 p99 低 10× 以上

**4.3 监控**:
- 新增 Prometheus 指标:
  - `sfid_chain_submit_by_role{role}` counter
  - `sfid_chain_submit_by_province{province}` counter
  - `sfid_sheng_signer_cache_hit` counter
- Grafana 面板:每省推链 QPS + fallback 率

**4.4 回滚预案**:
- 任何异常:`SFID_SHENG_SIGNER_ENABLED=false` 立即回退
- 链上 `ShengAdminPubkey` storage 保留不删,下次开启时无需重新初始化
- 已写入链的机构不受影响

---

## 与原方案 A 对比

| 维度 | 原方案 A(多备份) | 本方案(单密钥) |
|---|---|---|
| 每省密钥数 | 3(main + 2 backup) | 1 |
| 链上 storage | 复合 struct | 简单 `[u8; 32]` |
| 链上 extrinsic | 2(set + rotate) | 1(set_sheng_admin_pubkey,替换即轮换) |
| 轮换复杂度 | main→backup 提升逻辑 | 直接生成新 key 覆盖 |
| 前端 UI | 每省 3 行 | 每省 1 行 |
| 安全冗余 | 备份密钥可恢复 | 无冗余,丢失必须立即轮换 |
| 工作量 | 3.5~4.5 天 | **2~2.5 天** |
| 决策难点 | 私钥存储三选一 | 明确后端托管 |

## 风险清单

| 风险 | 等级 | 缓解 |
|---|---|---|
| runtime setCode 失败导致链停摆 | 高 | 本地节点先 dry-run,再 stage 环境,最后生产 |
| 加密派生密钥在内存中被 dump | 低 | 进程级隔离 + 短生命周期(解密完立即丢弃派生密钥,只保留 PairSigner 实例) |
| 43 省 iter 在 verifier 里的性能 | 低 | 每次 extrinsic ~43 次 O(1) 比对,远低于其他 pallet 开销 |
| 替换省级管理员(`replace_sheng_admin` 老接口)时推链密钥不同步 | 中 | 在 `replace_sheng_admin` handler 内部级联触发 `rotate_sheng_signer`,两件事一笔请求完成 |

## 不做的事

- 不引入 pallet_proxy
- 不做每省多密钥备份(明确放弃冗余换简单)
- 不改数据库 schema(走 Store JSON 持久化)
- 不改动 CPMS / citizen 绑定 / 注册局相关代码
- 不做跨省推链

## 时间线

| 阶段 | 工作量 |
|---|---|
| 1 链端 runtime 修改 | 0.75 天 |
| 2 后端密钥管理 + 签名路由 | 1 天 |
| 3 前端管理界面 | 0.5 天 |
| 4 联调 + 压测 + 灰度切换 | 0.5 天 |
| **合计** | **2.75 天** |

## 用户拍板决定(2026-04-09)

1. ✅ **链上 runtime 升级**:走 `system.setCode`,符合 `feedback_chainspec_frozen`
2. ✅ **加密算法统一**:**AES-256-GCM + HKDF-SHA256**,全系统(Store privkey 加密 /
   session token / 任何密钥派生)使用同一套
3. ✅ **命名统一**:删除 `sheng_admin_signer` 单独概念,**省级管理员 = 省级密钥**,
   一个实体一个名字。`sheng_admin_province_by_pubkey` 映射里的 pubkey 就是链上授权
   signer,对应 privkey 加密存在同一结构体里。用于登录系统 + 推链签名**同一把密钥**。
4. ✅ **压测工具**:自写 Rust 脚本(`sfid/backend/tools/load_test.rs` 或独立 crate)
5. ✅ **执行顺序**:**多端一次性改造**,链端 → 后端 → 前端严格串行,每端改完整后再动下一端

## 命名统一后的数据模型

**删除**(原方案里独立的):
- ~~`sheng_admin_signer: HashMap<Province, ShengSignerRow>`~~
- ~~`ShengSignerCache` 独立结构~~

**改动**(在现有 `AdminUser` / `sheng_admin_province_by_pubkey` 基础上扩展):
```rust
// sfid/backend/src/models/mod.rs - 扩展现有 AdminUser
pub(crate) struct AdminUser {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    pub(crate) status: AdminStatus,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    pub(crate) city: String,

    // ── 新增字段(仅 ShengAdmin 必填,其他角色 None) ──
    /// 中文注释:省级管理员的私钥,AES-256-GCM 加密后 base64 编码。
    #[serde(default)]
    pub(crate) encrypted_privkey: Option<String>,
    /// 链上该 admin_pubkey 当前版本(每次 set_sheng_admin_pubkey extrinsic 成功后递增)
    #[serde(default)]
    pub(crate) chain_version: u32,
}
```

- 登录挑战应答:`admin_pubkey` 作为验签公钥(现有流程不变)
- 推链签名:从内存 cache 里按 `admin_province` 索引到对应 AdminUser,用解密后的 privkey 构造 `PairSigner`
- 轮换 = 调用 `replace_sheng_admin` 接口,内部动作:
  1. 后端生成新 sr25519 keypair
  2. 更新 `AdminUser.admin_pubkey` + 加密存储 privkey
  3. 调用链端 `set_sheng_admin_pubkey(province, new_pub)`
  4. 成功后更新内存 cache

## 相关后端接口简化

原方案的 4 个独立接口全部折叠进现有的 `replace_sheng_admin`:

```
POST /api/v1/admin/sheng-admins/unlock

PUT /api/v1/admin/sheng-admins/:province
  替换/初始化某省管理员(复用现有 handler,扩展内部逻辑)
  action:
    2. 加密 privkey 写入 AdminUser.encrypted_privkey
    3. 调 chain set_sheng_admin_pubkey 成功后递增 chain_version
    4. 内存 cache.replace(province, new_signer)
    5. 返回新 pubkey + tx_hash + chain_version

GET /api/v1/admin/sheng-admins(复用现有 list,新增字段:chain_version / cache_loaded)
```

前端 ShengSignerPanel 也合并到现有的省级管理员管理界面,不单独开 sub-tab。

## 相关文档

- 前置讨论:会话中"方案 A 省级管理员 3 把密钥"段落,后被用户简化为单密钥方案
- 参考铁律:
  - `feedback_chainspec_frozen.md`
  - `feedback_sfid_pow_chain_recipe.md`
  - `feedback_no_chain_restart.md`
  - `feedback_scale_domain_must_be_array.md`
  - `feedback_sfid_three_roles_naming.md`
