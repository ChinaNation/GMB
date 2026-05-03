# Phase 1 实施方案:省级签名密钥独立推链(v3 实施后总结)

> **v3 状态**:Phase 1.A(链端)+ Phase 1.B(后端)+ Phase 1.C(前端)已全部落地,
> cargo check + tsc + build 全绿。Phase 1.D(压测)推迟,Phase 1.E(文档 + 清理 + 归档)
> 本轮完成。本章节为 v3 最终版对实际架构和实际改动的总结,下方保留 v1/v2 讨论作为
> "方案演化"历史。

---

## 0. v3 实施后实际架构总结

### 0.1 链端实际落地(`citizenchain/runtime/otherpallet/sfid-system` + 相关 pallet)

- `sfid-system` pallet:
  - 新增 storage `ShengSigningPubkey: map Province → [u8;32]`
  - 新增 storage `ProvinceBySigningPubkey: map [u8;32] → Province`(O(1) 反向索引)
  - 新增 extrinsic `set_sheng_signing_pubkey(province, new_pubkey: Option<[u8;32]>)`:
    origin 必须 `ensure_signed` 且 `who == SfidMainAccount`,`None` 表示清除该省
  - 新增 getter `sheng_signing_pubkey(province) -> Option<[u8;32]>`
  - 新增 Event:`ShengSigningPubkeyUpdated { province, new_pubkey }`
- `duoqian-manage` pallet:
  - `register_sfid_institution` 追加参数 `signing_province: Option<Vec<u8>>`,
    `SfidInstitutionVerifier` trait 同步加该字段
  - `None` → 回退到 SFID MAIN 验签(向后兼容);`Some(province)` → 要求
    `origin_pubkey == sheng_signing_pubkey(province)`
- `runtime/src/configs/mod.rs`:
  - 4 个签名 payload domain tag 统一为 `b"GMB_SFID_V1"`（2026-04-20 统一为 `DUOQIAN_DOMAIN + OP_SIGN_*`）(代码里的 verifier 共享常量,
    使用 `[u8; N]` 形式,符合 `feedback_scale_domain_must_be_array` 铁律)
  - `RuntimeSfidInstitutionVerifier` 根据 `signing_province` 分流到省级 pubkey
- `runtime/src/lib.rs`:
  - `spec_version: 5 → 6`
  - `transaction_version: 1 → 2`

### 0.2 后端实际落地(`sfid/backend`)

- `Cargo.toml`:新增 `aes-gcm = 0.10` / `hkdf = 0.12` / `base64 = 0.22` / `getrandom = 0.2`
- Postgres `admins` 表通过 `ALTER TABLE IF NOT EXISTS` 扩两列:
  - `encrypted_signing_privkey TEXT`(base64(nonce_12B || ciphertext || tag_16B))
  - `signing_pubkey TEXT`(hex,便于链上对账)
- `src/models/mod.rs::AdminUser` + `sheng-admins` Row 扩同名两字段
  内部持 `HashMap<Province, sr25519::Pair>` 与 AES-256-GCM wrap key
  (wrap key 由 HKDF-SHA256(SFID MAIN seed, salt, info) 派生,salt =
  `sfid-sheng-signer-v1-salt`、info = `sfid-sheng-signer-v1-info`),
  整个生命周期明文 seed 用 `zeroize`
  - `resolve_business_signer(state, ctx) -> (sr25519::Pair, Province)` 按 ctx.role 路由
  - Sheng/Shi 未在线/本省未 bootstrap → 503 友好提示
  `submit_set_sheng_signing_pubkey(state, ka_signer, province, Option<[u8;32]>)` 的
  extrinsic 组装 + 上链 + InBestBlock 等待
  如果本省已有 encrypted_signing_privkey → 解密载入 cache;
  否则生成新 sr25519 keypair → 加密入库 → 推链 `set_sheng_signing_pubkey` → 载入 cache
- 登录 handler 接入 bootstrap:
  - `admin_auth_verify`(扫码)
  - `admin_auth_qr_complete`(二维码完成)
- `cleanup_admin_sessions` 返回 `Vec<String>`(被驱逐的 province 列表),
  `admin_auth` 所有调用点在拿到返回值后遍历 `sheng_signer_cache.unload_province`
- `replace_sheng_admin`:级联清链(`set_sheng_signing_pubkey(province, None)`)+
  清 Store `encrypted_signing_privkey` / `signing_pubkey` + 驱逐 cache
- `set_active_main_signer`(即 `replace_active_main_seed` 的落地点):**级联重加密**
  所有 sheng admin 的 `encrypted_signing_privkey`(旧 wrap key 解密 → 新 wrap key 加密),
  与 `rotate_main_seed` 原子完成
- `submit_register_sfid_institution_extrinsic`:
  - signer 改为 `resolve_business_signer` 返回的本省 sr25519 Pair
  - 签名 payload 追加 `province` 字段,使用 `b"GMB_SFID_V1"`（2026-04-20 统一为 `DUOQIAN_DOMAIN + OP_SIGN_*`） domain
  - extrinsic args 追加 `signing_province: Some(province)`
  - **签字段包和 submit 必须用同一把 Pair**(否则链端 verifier 会挂)
- `src/chain/runtime_align.rs`:
  - 4 个 `DOMAIN_*` 常量统一为 `b"GMB_SFID_V1"`（2026-04-20 统一为 `DUOQIAN_DOMAIN + OP_SIGN_*`）
  - 新建 `build_institution_credential_with_province(state, ctx, ...)`
- `src/institutions/chain.rs` + `src/institutions/handler.rs` +
  `src/sheng-admins/multisig.rs` 调用点全部级联追加 `ctx: &AdminAuthContext` 参数

### 0.3 前端实际落地(`sfid/frontend`)

- `src/views/sheng_admins/ShengAdminsView.tsx`:新增"签名密钥状态"列
  - 未初始化:灰色 Tag "未初始化"
  - 已激活:绿色 Tag "已激活" + `<Tooltip>` 展示完整 pubkey hex
- `src/api/client.ts::ShengAdminRow`:新增 `signing_pubkey?: string | null`
- `src/views/institutions/CreateInstitutionModal.tsx` +
  `src/views/institutions/CreateAccountModal.tsx`:
  翻译成友好中文提示

### 0.4 已知残留 / 推迟项

- **Phase 1.D 压测推迟**:未跑 500 并发 SHI_ADMIN 压测,灰度开关未设计(当前实现
  无 feature flag,直接全量切换本省签名路由)
- Setup 文档(环境变量说明)未写,但不需要新 env —— 只是复用现有
  `SFID_SIGNING_SEED_HEX`(SFID MAIN seed)作为 HKDF 派生源
- 运维须知:后端重启后所有省 signer 清零,需要 43 省各自重新扫码登录才能恢复业务推链

### 0.5 v1 / v2 作废的决定

- ❌ **2.5.1 的独立 `POST /api/v1/admin/system/unlock` 接口**:不存在。实际实现里
- ❌ **3.1 `SystemLockBanner` + 3.2 `UnlockBackendModal`**:不存在。前端不需要
- ❌ **第 9 节"省 seed 如何从后端传到 sheng login admin 冷钱包"拍板**:作废。
  最终架构里 sheng admin 的签名私钥**完全由后端托管**(加密 Store + 运行时 cache),
  不需要导出到任何冷钱包
- ❌ **2.3 `ShengSignerCache` 用 subxt `PairSigner<...>`**:作废。实际 cache 只存
  `sr25519::Pair`,`PairSigner` 在 `submit_*_extrinsic` 调用点现场包装,避免
  `PairSigner` 克隆 + 泛型污染整个 signer_router
- ❌ **2.8 持久化只讲 Store JSON**:作废。实际持久化落在 **Postgres admins 表**,
  通过 `ALTER TABLE IF NOT EXISTS` 扩两列,Store JSON 只是内存视图
- ❌ **2.9 `replace_active_main_seed` 未讲级联重加密**:补强。现在 `set_active_main_signer`
  会在单次事务里完成旧 wrap → 新 wrap 的所有 sheng 密文级联重加密,zeroize
  中间明文

---

## 方案演化(v1 / v2 历史讨论,仅供溯源)

# Phase 1 最终实施方案:省级签名密钥独立推链(v2 定稿)

- **任务 ID**: `20260409-sfid-sheng-admin-per-province-keyring`
- **版本**: v2 定稿(覆盖 v1)
- **日期**: 2026-04-09
- **工作量**: 2.5 天(链端 0.5 + 后端 1 + 前端 0.5 + 联调 0.5)
- **依赖**: App.tsx 拆分任务卡已完成
- **状态**: 已拍板,待开工

---

## 一、架构最终定稿

### 1.1 三层治理关系

```
  职责:管理省登录管理员,只签一种链交易 set_sheng_signing_pubkey
  ↓ 管理
省登录管理员(43 位,每省 1 位)
  职责:
    • 登录 sfid 触发本省签名密钥生成/解锁
    • 管理本省签名密钥对(通过登录/登出控制 cache)
    • 管理本省市管理员(HTTP 层 CRUD,无链上操作)
  私钥位置:自己的 wumin 冷钱包
  公钥位置:sfid 后端 Store(不上链)
  ↓ 管理
省签名密钥对(43 组)
  职责:代签本省所有业务 extrinsic(register_sfid_institution 等)
  私钥位置:sfid 后端 Store 加密 + 运行时解密到内存 cache
  公钥位置:链上 ShengSigningPubkey[province]
  ↓ 被透明使用
市管理员(每市 ~10 位,共 ~50K)
  职责:触发业务操作
  私钥位置:自己的冷钱包
  公钥位置:sfid 后端 Store(不上链)
```

### 1.2 链上存储(两份,一份已有)

```rust
// 已有

// 新增
pub type ShengSigningPubkey<T> = StorageMap<
    _,
    Blake2_128Concat,
    BoundedVec<u8, ConstU32<64>>,   // province UTF-8
    [u8; 32],                        // sr25519 pubkey
    OptionQuery,
>;

// 新增:O(1) 反向索引(verifier 专用)
pub type ProvinceBySigningPubkey<T> = StorageMap<
    _,
    Blake2_128Concat,
    [u8; 32],
    BoundedVec<u8, ConstU32<64>>,
    OptionQuery,
>;
```

**链上不存**:省登录管理员公钥、市管理员公钥、任何私钥或加密数据。

### 1.3 链上 extrinsic(一新 + 一改)

#### 1.3.1 新增:`set_sheng_signing_pubkey`

```rust
#[pallet::call_index(N)]
#[pallet::weight(T::DbWeight::get().reads_writes(2, 3))]
pub fn set_sheng_signing_pubkey(
    origin: OriginFor<T>,
    province: Vec<u8>,
    new_pubkey: Option<[u8; 32]>,   // None = 清除该省
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let bounded: BoundedVec<u8, ConstU32<64>> = province
        .try_into()
        .map_err(|_| Error::<T>::ProvinceTooLong)?;


    // 清理旧反向索引
    if let Some(old_pub) = ShengSigningPubkey::<T>::get(&bounded) {
        ProvinceBySigningPubkey::<T>::remove(&old_pub);
    }

    match new_pubkey {
        Some(pub_key) => {
            ensure!(
                pub_key != ka.main
                    && pub_key != ka.backup_a
                    && pub_key != ka.backup_b,
            );
            // 冲突校验:新 pubkey 不得已被其他省占用
            ensure!(
                !ProvinceBySigningPubkey::<T>::contains_key(&pub_key),
                Error::<T>::PubkeyAlreadyUsed
            );
            ShengSigningPubkey::<T>::insert(&bounded, pub_key);
            ProvinceBySigningPubkey::<T>::insert(&pub_key, &bounded);
        }
        None => {
            ShengSigningPubkey::<T>::remove(&bounded);
        }
    }
    Self::deposit_event(Event::ShengSigningPubkeyUpdated {
        province: bounded,
        new_pubkey,
    });
    Ok(())
}
```

#### 1.3.2 改造:业务 extrinsic 的 verifier


```rust
fn ensure_business_writer<T: Config>(who: &T::AccountId) -> DispatchResult {
    let caller_bytes: [u8; 32] = who
        .encode()
        .try_into()
        .map_err(|_| Error::<T>::InvalidCallerLength)?;
    ensure!(
        ProvinceBySigningPubkey::<T>::contains_key(&caller_bytes),
        Error::<T>::NotAuthorizedSfidWriter
    );
    Ok(())
}
```



#### 1.3.3 新增 Error

```rust
#[pallet::error]
pub enum Error<T> {
    // ...existing...
    ProvinceTooLong,
    PubkeyAlreadyUsed,
    NotAuthorizedSfidWriter,
    InvalidCallerLength,
}
```

#### 1.3.4 新增 Event

```rust
#[pallet::event]
pub enum Event<T: Config> {
    // ...existing...
    ShengSigningPubkeyUpdated {
        province: BoundedVec<u8, ConstU32<64>>,
        new_pubkey: Option<[u8; 32]>,
    },
}
```

### 1.4 runtime 升级

- `spec_version +1`
- 走 `system.setCode` on-chain 升级(`feedback_chainspec_frozen` 铁律)
- 本地 dev 链先演练,stage 验证后生产

---

## 二、后端实施细节

### 2.1 依赖(Cargo.toml)

```toml
[dependencies]
# 现有 + 新增
aes-gcm = "0.10"
hkdf = "0.12"
sha2 = "0.10"
base64 = "0.22"
zeroize = "1.7"
getrandom = "0.2"
```

### 2.2 Store 数据模型

**文件**:`sfid/backend/src/models/mod.rs`

扩展现有 `AdminUser`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminUser {
    // ...existing fields...

    /// 仅 ShengAdmin 使用。加密存储的本省签名私钥种子。
    /// 格式:base64(nonce_12B || ciphertext || tag_16B)
    /// 明文 = sr25519 seed 32 字节
    #[serde(default)]
    pub(crate) encrypted_signing_privkey: Option<String>,

    /// 仅 ShengAdmin 使用。对应签名密钥的公钥(便于和链上对账)。
    #[serde(default)]
    pub(crate) signing_pubkey: Option<String>,   // hex
}
```

**不新建任何独立的数据结构**(保持"省管理员 = 一个 AdminUser"的统一命名)。

### 2.3 ShengSignerCache 内存缓存


```rust

use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::Engine as _;
use hkdf::Hkdf;
use sha2::Sha256;
use sp_core::{sr25519, Pair};
use subxt::tx::PairSigner;
use subxt::PolkadotConfig;
use zeroize::Zeroize;

pub(crate) type ProvinceSigner = PairSigner<PolkadotConfig, sr25519::Pair>;

// HKDF 固定参数,写死
const WRAP_SALT: &[u8] = b"sfid-sheng-signer-v1-salt";
const WRAP_INFO: &[u8] = b"sfid-sheng-signer-v1-info";

const NONCE_LEN: usize = 12;

pub(crate) struct ShengSignerCache {
    /// 省名 → 本省签名 PairSigner
    signers: RwLock<HashMap<String, ProvinceSigner>>,
    /// AES-256 wrap key(加密/解密省签名私钥用)
    wrap_key: RwLock<Option<[u8; 32]>>,
    unlocked: AtomicBool,
}

impl ShengSignerCache {
    pub(crate) fn new() -> Self {
        Self {
            signers: RwLock::new(HashMap::new()),
            wrap_key: RwLock::new(None),
            unlocked: AtomicBool::new(false),
        }
    }

    pub(crate) fn is_unlocked(&self) -> bool {
        self.unlocked.load(Ordering::Acquire)
    }

    pub(crate) fn unlock(
        &self,
        ka_seed: &mut [u8; 32],
    ) -> Result<(), String> {
        let ka_pair = sr25519::Pair::from_seed(ka_seed);
        let ka_signer = PairSigner::new(ka_pair);

        // 2. 派生 wrap key
        let mut wrap_key = [0u8; 32];
        {
            let hk = Hkdf::<Sha256>::new(Some(WRAP_SALT), ka_seed.as_slice());
            hk.expand(WRAP_INFO, &mut wrap_key)
                .map_err(|_| "hkdf expand failed".to_string())?;
        }
        ka_seed.zeroize();

        // 3. 写入内存
        *self.wrap_key.write().map_err(|_| "cache poisoned")? = Some(wrap_key);
        self.unlocked.store(true, Ordering::Release);
        Ok(())
    }

    /// 解密一条 base64 密文 → 32 字节 seed
    pub(crate) fn decrypt_seed(&self, encrypted_b64: &str) -> Result<[u8; 32], String> {
        let wrap_guard = self.wrap_key.read().map_err(|_| "cache poisoned")?;
        let wrap = wrap_guard.as_ref().ok_or("cache not unlocked")?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(wrap));

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encrypted_b64)
            .map_err(|e| format!("base64: {e}"))?;
        if bytes.len() < NONCE_LEN + 16 {
            return Err("ciphertext too short".into());
        }
        let (nonce_bytes, ct) = bytes.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let mut plaintext = cipher
            .decrypt(nonce, ct)
            .map_err(|e| format!("aes decrypt: {e}"))?;
        if plaintext.len() != 32 {
            plaintext.zeroize();
            return Err("seed length wrong".into());
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&plaintext);
        plaintext.zeroize();
        Ok(out)
    }

    /// 加密一条 32 字节 seed → base64 密文
    pub(crate) fn encrypt_seed(&self, seed: &[u8; 32]) -> Result<String, String> {
        let wrap_guard = self.wrap_key.read().map_err(|_| "cache poisoned")?;
        let wrap = wrap_guard.as_ref().ok_or("cache not unlocked")?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(wrap));

        let mut nonce_bytes = [0u8; NONCE_LEN];
        getrandom::getrandom(&mut nonce_bytes).map_err(|e| format!("rng: {e}"))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, seed.as_slice())
            .map_err(|e| format!("aes encrypt: {e}"))?;

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(base64::engine::general_purpose::STANDARD.encode(&out))
    }

    /// 省登录管理员登录时调用:生成/解锁该省 signing key 到内存
    pub(crate) fn load_province(&self, province: String, signer: ProvinceSigner) {
        if let Ok(mut guard) = self.signers.write() {
            guard.insert(province, signer);
        }
    }

    /// 省登录管理员登出时调用
    pub(crate) fn unload_province(&self, province: &str) {
        if let Ok(mut guard) = self.signers.write() {
            guard.remove(province);
        }
    }

    pub(crate) fn get(&self, province: &str) -> Option<ProvinceSigner> {
        self.signers.read().ok()?.get(province).cloned()
    }

    }

    pub(crate) fn active_province_count(&self) -> usize {
        self.signers.read().map(|g| g.len()).unwrap_or(0)
    }
}
```

### 2.4 签名路由


```rust

use crate::models::AdminRole;

pub(crate) struct AdminCtx {
    pub(crate) admin_pubkey: String,
    pub(crate) role: AdminRole,
    pub(crate) admin_province: Option<String>,
}

/// 业务推链:必须有本省签名密钥 cache,无则返回错误(由调用方转 503)
pub(crate) fn resolve_business_signer(
    state: &AppState,
    ctx: &AdminCtx,
) -> Result<ProvinceSigner, ApiError> {
    if !state.sheng_signer_cache.is_unlocked() {
        return Err(ApiError::service_unavailable(
        ));
    }
    let province = match ctx.role {
            return Err(ApiError::forbidden(
            ));
        }
        AdminRole::ShengAdmin | AdminRole::ShiAdmin => {
            ctx.admin_province.as_deref().ok_or_else(|| {
                ApiError::bad_request("管理员缺少省份信息")
            })?
        }
    };
    state
        .sheng_signer_cache
        .get(province)
        .ok_or_else(|| {
            ApiError::service_unavailable(&format!(
                "本省({province})登录管理员未在线,暂无法推链"
            ))
        })
}

    state: &AppState,
) -> Result<ProvinceSigner, ApiError> {
    state
        .sheng_signer_cache
        .ok_or_else(|| ApiError::service_unavailable("sfid 后端未解锁"))
}
```

### 2.5 HTTP 接口

#### 2.5.1 POST `/api/v1/admin/system/unlock`

```
body: { challenge_id, signature, seed_hex }
action:
  2. 解析 seed_hex → 32 字节种子
  3. 调用 sheng_signer_cache.unlock(&mut seed)
  4. 遍历 Store 所有 ShengAdmin,凡是有 encrypted_signing_privkey 的,
     解密 seed → 构造 PairSigner → 塞进 cache(但 session 没打开,不代表可推链)
  5. zeroize 传入的 seed
  6. 返回 { unlocked: true }
```

**注意**:解锁后**不会自动让所有省立刻可用**。sheng login admin 登录后才把本省 signer 塞进 cache.signers。这一步只是:
- 派生 wrap key(用于后续加密/解密)


**修正**:为了减少"后端重启后所有省都要重登"的麻烦,可以在解锁时**预解密所有已有 encrypted_signing_privkey 到 cache**,跳过 sheng login admin 的重新登录步骤。但这违反"登出即驱逐"的业务规则。

**取舍**:
- [ ] 严格语义:登出=驱逐,后端重启后必须每省 sheng login admin 重新登录
- [ ] 方便运维:解锁时预加载所有有存量的省,后端重启对 sheng login admin 透明

**推荐严格语义**,符合"登入才能推链"的业务承诺。

#### 2.5.2 省登录管理员登录 handler 改造

**文件**:`sfid/backend/src/login/sheng_admin_login.rs`(或登录 handler 所在位置)

在现有登录验签成功后追加:

```rust
if ctx.role == AdminRole::ShengAdmin {
    let province = ctx.admin_province.as_deref().ok_or_else(|| {
        ApiError::internal("sheng admin missing province")
    })?;

    // 检查 cache 是否已解锁
    if !state.sheng_signer_cache.is_unlocked() {
        tracing::warn!("sfid backend not unlocked, sheng login proceeds but SHI推链 unavailable");
        // 登录仍然成功,但本省 SHI_ADMIN 推链会失败(返回 503)
        return Ok(login_response);
    }

    // 查 Store 是否已有 signing privkey
    let store = store_read_or_500(&state)?;
    let user = store
        .admin_users_by_pubkey
        .get(&ctx.admin_pubkey)
        .ok_or_else(|| ApiError::not_found("admin not found"))?;
    let existing_encrypted = user.encrypted_signing_privkey.clone();
    drop(store);

    match existing_encrypted {
        Some(enc) => {
            // 已有:解密载入
            let mut seed = state.sheng_signer_cache.decrypt_seed(&enc)?;
            let pair = sr25519::Pair::from_seed(&seed);
            seed.zeroize();
            let signer = PairSigner::new(pair);
            state.sheng_signer_cache.load_province(
                province.to_string(),
                signer,
            );
            tracing::info!(province, "sheng signer loaded from store");
        }
        None => {
            // 首次:生成 + 加密存 Store + 签 set_sheng_signing_pubkey 上链
            let (pair, mut seed) = sr25519::Pair::generate();
            let new_pubkey = pair.public().0;
            let encrypted = state.sheng_signer_cache.encrypt_seed(&seed)?;

            // 写 Store(先本地,后链)
            {
                let mut store = store_write_or_500(&state)?;
                let user = store
                    .admin_users_by_pubkey
                    .get_mut(&ctx.admin_pubkey)
                    .ok_or_else(|| ApiError::not_found("admin not found"))?;
                user.encrypted_signing_privkey = Some(encrypted.clone());
                user.signing_pubkey = Some(hex::encode(new_pubkey));
            }

            // 推链
            let ka_signer = state
                .sheng_signer_cache
            submit_set_sheng_signing_pubkey(
                &state,
                &ka_signer,
                province,
                Some(new_pubkey),
            )
            .await?;

            // 上链成功后,塞进 cache
            let signer = PairSigner::new(pair);
            state.sheng_signer_cache.load_province(
                province.to_string(),
                signer,
            );
            seed.zeroize();
            tracing::info!(province, "sheng signer generated and pushed to chain");
        }
    }
}
```

**错误处理**:
- 推链失败 → 回滚 Store 写入(把 encrypted_signing_privkey 设回 None)
- 加密失败 → 不影响 Store,返回 500
- cache 未解锁 → 登录成功但推链不可用,返回 200 + 提示

#### 2.5.3 省登录管理员登出 handler 改造

```rust
if ctx.role == AdminRole::ShengAdmin {
    if let Some(province) = ctx.admin_province.as_deref() {
        state.sheng_signer_cache.unload_province(province);
        tracing::info!(province, "sheng signer unloaded on logout");
    }
}
```

#### 2.5.4 更换省登录管理员 handler 改造

**文件**:`sfid/backend/src/sheng_admins/catalog.rs::replace_sheng_admin`

在现有逻辑之后追加:

```rust
// 如果后端已解锁,清除链上 + Store 的 signing 密钥
if state.sheng_signer_cache.is_unlocked() {
    // 1. 推链清除
    let ka_signer = state
        .sheng_signer_cache
    submit_set_sheng_signing_pubkey(&state, &ka_signer, &province, None).await?;

    // 2. 清 Store 的 encrypted_signing_privkey(新 admin 的记录)
    let mut store = store_write_or_500(&state)?;
    if let Some(user) = store.admin_users_by_pubkey.get_mut(&new_admin_pubkey) {
        user.encrypted_signing_privkey = None;
        user.signing_pubkey = None;
    }
    drop(store);

    // 3. 驱逐 cache
    state.sheng_signer_cache.unload_province(&province);
}
```

#### 2.5.5 业务 handler 接入


替换:
```rust
// 原:

// 新:
```

### 2.6 链端交易提交 helper


```rust
pub(crate) async fn submit_set_sheng_signing_pubkey(
    state: &AppState,
    signer: &ProvinceSigner,
    province: &str,
    new_pubkey: Option<[u8; 32]>,
) -> Result<String, String> {
    let client = state.chain_client.clone();

    // 构造动态 extrinsic
    let province_arg = Value::from_bytes(province.as_bytes());
    let pub_arg = match new_pubkey {
        Some(p) => Value::variant(
            "Some",
            subxt::dynamic::Composite::Unnamed(vec![Value::from_bytes(p.to_vec())]),
        ),
        None => Value::variant(
            "None",
            subxt::dynamic::Composite::Unnamed(vec![]),
        ),
    };
    let call = tx(
        "SfidSystem",
        "set_sheng_signing_pubkey",
        vec![province_arg, pub_arg],
    );

    // 显式 nonce + immortal + InBestBlock(feedback_sfid_pow_chain_recipe)
    let account_id = signer.account_id().clone();
    let legacy = LegacyRpcMethods::<PolkadotConfig>::new(client.backend_rpc_client());
    let nonce = legacy
        .system_account_next_index(&account_id)
        .await
        .map_err(|e| format!("get nonce: {e}"))?;
    let params = DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(nonce)
        .build();

    let submitted = client
        .tx()
        .sign_and_submit_then_watch(&call, signer, params)
        .await
        .map_err(|e| format!("submit: {e}"))?;
    let in_block = submitted
        .wait_for_in_block()
        .await
        .map_err(|e| format!("in_block: {e}"))?;
    in_block.wait_for_success().await.map_err(|e| format!("tx failed: {e}"))?;

    Ok(format!("0x{}", hex::encode(in_block.extrinsic_hash().0)))
}
```

### 2.7 AppState 扩展

```rust
pub(crate) struct AppState {
    // ...existing...
    pub(crate) sheng_signer_cache: Arc<ShengSignerCache>,
    pub(crate) chain_client: OnlineClient<PolkadotConfig>,
}
```

初始化时 `Arc::new(ShengSignerCache::new())`,`chain_client` 由 subxt connect 创建。

### 2.8 启动日志

```rust
tracing::info!(
);
```

---

## 三、前端实施细节

### 3.1 新增系统解锁入口

**位置**:首页 Header 或侧边栏底部新增 "系统状态" 指示

**组件**:`sfid/frontend/src/components/SystemLockBanner.tsx`(新)

```tsx
// 顶部 Banner 条,所有角色都能看到
// - 已解锁:🔓 绿色细条(可选折叠)
```

逻辑:
```tsx
- 挂载时调 GET /api/v1/admin/system/status
- 返回 { unlocked: boolean, active_provinces: number }
- 已解锁 → 显示"已解锁,${active_provinces} 省在线"
```

### 3.2 解锁 Modal

**组件**:`sfid/frontend/src/components/UnlockBackendModal.tsx`(新)

流程:
2. 签一个固定 challenge "sfid-system-unlock-v1-{timestamp}"
4. 前端发 POST `/api/v1/admin/system/unlock` with `{ challenge_id, signature, seed_hex }`
5. 成功后刷新 Banner 状态

**wumin 改动需求**:在签 challenge 的同时也返回私钥种子(或派生的 wrap key)。需要用户确认 wumin 是否支持。


### 3.3 ShengAdminsView 改动

**现状**:`src/views/sheng-admins/ShengAdminsView.tsx` 显示省管理员列表(`w5GR...`)。

**改动**:
- 新增列:**签名密钥状态**
  - 🔵 未生成(未初始化)
  - 🟢 已激活(链上有,cache 已加载)
  - 🟡 已激活但 cache 未加载(sheng admin 未登录)
- 无需任何"激活"按钮,全自动流程

### 3.4 业务推链错误处理

业务 handler 可能返回的新错误:
- 503 "本省(辽宁省)登录管理员未在线,暂无法推链"

前端在机构注册等业务 Modal 里捕获这些错误,展示友好提示 + 引导操作(比如"请通知辽宁省管理员登录")。

---

## 四、压测工具

### 4.1 新 crate:`sfid/backend/tools/load_test/`

```toml
[package]
name = "sfid-load-test"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
sp-core = "..."
rand = "0.8"
hex = "0.4"
```

### 4.2 参数设计

```
sfid-load-test \
  --backend http://localhost:8080 \
  --concurrency 500 \
  --provinces 10 \
  --duration 60s \
  --ramp-up 10s \
  --report ./report.json
```

### 4.3 输出指标

- 总 tx 数 / 成功数 / 失败数 / 成功率
- P50 / P95 / P99 端到端延迟
- 每省 TPS 分布
- 错误分类(503 signer 未解锁 / 503 省离线 / 其他)

### 4.4 场景

1. `--concurrency 50 --provinces 1`:单省 50 并发基线
2. `--concurrency 500 --provinces 10`:跨省基线

---

## 五、执行顺序(多端一次性改造,严格串行)

### 阶段 1:链端(0.5 天)

- [ ] 新增 `ShengSigningPubkey` + `ProvinceBySigningPubkey` storage
- [ ] 新增 `set_sheng_signing_pubkey` extrinsic
- [ ] 改造业务 extrinsic 的 verifier(`register_sfid_institution` 等)
- [ ] 新增 Error + Event
- [ ] runtime `spec_version +1`
- [ ] 单元测试:set / remove / 冲突 / 权限 / verifier
- [ ] `cargo build --release` 产出 wasm
- [ ] 本地 dev 节点 `setCode` 升级演练

### 阶段 2:后端(1 天)

- [ ] 依赖更新(aes-gcm / hkdf / sha2 / base64 / zeroize)
- [ ] `models/mod.rs` AdminUser 扩展字段
- [ ] AppState 扩展 + 初始化
- [ ] `unlock` handler 接入
- [ ] 省登录管理员 login handler 追加 signer 生成/解锁逻辑
- [ ] 省登录管理员 logout handler 驱逐逻辑
- [ ] `replace_sheng_admin` handler 追加清链 + 清 Store
- [ ] 所有业务 extrinsic 提交点改走 `resolve_business_signer`
- [ ] `cargo check` + 单元测试

### 阶段 3:前端(0.5 天)

- [ ] 新建 `SystemLockBanner` + `UnlockBackendModal`
- [ ] `ShengAdminsView` 新列(签名密钥状态)
- [ ] 业务错误提示友好化
- [ ] `npx tsc --noEmit` + `npm run build`

### 阶段 4:压测 + 联调(0.5 天)

- [ ] 新 crate `sfid-load-test` 写完
- [ ] 本地起链 + 后端 + 前端
- [ ] 压测 baseline:50 并发 × 1 省
- [ ] 压测对比:改造前后 P99 对比报告

---

## 六、关键风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| runtime setCode 失败导致链停摆 | 🔴 高 | 本地 dev → stage → 生产三段演练 |
| 升级后存量省未登录导致业务全挂 | 🔴 高 | 升级前先让所有 sheng admin 至少登录一次;或升级后先挂公告再 setCode |
| 加密 wrap key 被内存 dump 泄露 | 🟢 低 | 生产禁用 core dump;定期重启清理内存 |
| 链上 O(43) 遍历性能 | ✅ 已解决 | 用 ProvinceBySigningPubkey 反向索引 O(1) |

---

## 七、灰度切换

**不设灰度开关**。用户明确"多端一次性改造"。

**部署流程**:
1. 维护窗口公告,预计中断 10~30 分钟
2. 链端 setCode 升级
3. 升级成功后,立即停 sfid 后端,部署新版本
5. 通知各省 sheng login admin 登录(触发 signing key 生成 + 上链)
6. 逐省验证 signing pubkey 已上链
7. 启用前端新版本,解除维护
8. 监控:`sheng_signer_cache` 在线省数 + 推链成功率

**回滚**:
- 如果链端升级后 30 分钟内发现严重问题:用 `setCode` 回滚到旧 runtime wasm
- 后端回退到上一版本
- 前端回退到上一版本

---

## 八、验收清单

### 8.1 链端
- [ ] `cargo build --release` 绿
- [ ] 所有单元测试通过
- [ ] 本地节点 setCode 升级成功
- [ ] 业务 extrinsic 由省签名密钥签发可通过
- [ ] 反向索引在 set/remove 时正确维护

### 8.2 后端
- [ ] `cargo check` + `cargo test` 绿
- [ ] 启动后 cache locked
- [ ] Sheng admin 首次登录:生成 + 加密 + 上链 + cache 载入(四步全成功)
- [ ] Sheng admin 再次登录:解密 + cache 载入(两步全成功)
- [ ] Sheng admin 登出:cache 驱逐,本省业务立即 503
- [ ] `replace_sheng_admin` 级联清链 + 清 Store + 驱逐 cache
- [ ] 业务 extrinsic 按省正确路由
- [ ] wrap key 全生命周期不落盘

### 8.3 前端
- [ ] `npx tsc --noEmit` + `npm run build` 绿
- [ ] SystemLockBanner 正确显示状态
- [ ] ShengAdminsView 显示 signing 密钥状态 Tag
- [ ] 业务错误(503)有友好提示

### 8.4 联调
- [ ] 辽宁 sheng admin 登出 → 辽宁 SHI_ADMIN 推链立即失败
- [ ] 辽宁 sheng admin 重新登录 → 推链恢复(无需重新生成 keypair)
- [ ] 压测 baseline 报告产出

---

## 九、用户最后确认的事项

**已全部拍板**(前面对话)。唯一剩一个实现细节:


- [ ] **方式 1**:HTTP body 明文 hex 传 seed(简单,依赖 TLS 信任)—— **推荐**
- [ ] **方式 2**:wumin 冷钱包支持派生 wrap key,只传 wrap key(需要 wumin 支持额外派生功能)

**默认方式 1**,除非你指定换。

---

## 十、预估工作量

| 阶段 | 时长 |
|---|---|
| 1 链端 | 0.5 天 |
| 2 后端 | 1 天 |
| 3 前端 | 0.5 天 |
| 4 压测+联调 | 0.5 天 |
| **合计** | **2.5 天** |

---

## 十一、开工信号

回复 "**开工**" 即按本方案进入阶段 1。

回复 "**改某某处**" 则先修订方案再开工。
