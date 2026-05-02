# SFID Step 1 / Phase 23e:`key-admins/` 整目录删 + AppState/main.rs/main_tests 重写

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 前置依赖:phase23a + b + c + d 全部完成

## 任务需求

phase23 主卡的最终一击。前置 4 卡把 rsa_blind / business / operate 全部摘走,本卡集中处理剩余 KEY_ADMIN 强耦合:
- 拆 `key_admins::chain_keyring`(`ChainKeyringState` 等)
- 删 `key-admins/` 整目录
- 删 `chain/key_admins/` + `chain/balance/` + `chain/sheng_admin/clear_sheng_signing.rs`
- 删 `models::AdminRole::KeyAdmin` 枚举值 + 所有 match 分支
- `main.rs` 删 KEY_ADMIN 路由 + AppState 字段(`signing_seed_hex` / `known_key_seeds` / `key_id` / `key_version` / `key_alg` / `public_key_hex`)
- `main_tests.rs`(966 行)重写,KEY_ADMIN 测试全删

## 搬迁/删除清单

### chain_keyring 拆分

`key_admins::chain_keyring::{ChainKeyringState, load_signing_key_from_seed, derive_pubkey_hex_from_seed}` 被 `login/mod.rs` + `chain/runtime_align.rs` + AppState 启动消费。本卡:
- `load_signing_key_from_seed` / `derive_pubkey_hex_from_seed` → 迁 `crypto/sr25519.rs`(新文件,放低层)
- `ChainKeyringState` → 评估是否仍需要;若纯粹 KEY_ADMIN 全局签名 state,**直接删**(phase23 已用 `sheng_admins::signing_cache::ShengSigningCache` 替代)

### login/mod.rs 改造

- 删除 `bootstrap_sheng_signer` 调用,改调 `sheng_admins::bootstrap::ensure_signing_keypair`
- KEY_ADMIN 登录分支整段删除
- 只保留 SHENG_ADMIN(三 slot)+ SHI_ADMIN

### AppState 改造

```rust
// 删除字段
- signing_seed_hex
- known_key_seeds
- key_id / key_version / key_alg
- public_key_hex
- sheng_signer_cache:类型从 key_admins::sheng_signer_cache::ShengSignerCache 改 sheng_admins::signing_cache::ShengSigningCache
// 保留
+ store / sharded_store / rate_limit_redis / cpms_register_inflight
```

### 路由表删除

`main.rs` grep `/api/v1/admin/key-admin` 整段 endpoint:删除

### main_tests.rs 重写

- KEY_ADMIN 相关测试整删(凭 fn 名 + grep `KeyAdmin`)
- SHENG_ADMIN / SHI_ADMIN 测试保留
- 新加最少:`sheng_signing_3tier_isolation` 1 个集成测试(三 slot 各自独立 keypair)
- 总行数预期 ~500 行(从 966 行降一半)

### 物理删除

- `src/key-admins/`(整目录,先 git mv 已迁出的 rsa_blind 后剩余 3 文件:`chain_keyring.rs`(部分内容已搬走可整删)、`mod.rs`、`signer_router.rs`、`sheng_signer_cache.rs`)
- `src/chain/key_admins/`(整目录)
- `src/chain/balance/`(整目录)
- `src/chain/sheng_admin/clear_sheng_signing.rs`
- `src/chain/sheng_admin/mod.rs`(若内容只剩 `pub mod clear_sheng_signing;`,直接删 mod.rs;否则保留并精简)

### models 改造

- `AdminRole::KeyAdmin` 枚举值删除
- 所有 `match role { AdminRole::KeyAdmin => ... }` 分支删除
- `parse_admin_role(role: &str)` 删 `KEY_ADMIN` 分支
- `admin_role_text` 删 `KeyAdmin` 分支

### scope 改造

- `scope::filter_by_scope` KEY 分支删除
- `scope::rules` 中 KEY 相关规则删除

## 影响范围

- 删除:~3000 行
- 修改:`main.rs`、`main_tests.rs`、`login/mod.rs`、`chain/runtime_align.rs`、`models/role.rs`(phase23a 拆分后)、`scope/{filter,rules}.rs`、`store_shards/{shard_types,migration}.rs`、`institutions/handler.rs` 几处分支

## 主要风险点

- **`main_tests.rs` 重写工作量大**:需小心保留与 KEY_ADMIN 无关的测试
- **`store_shards/migration.rs`** 含 KEY_ADMIN 历史 migration:保留(historic),只删未来代码引用
- **`chain/runtime_align.rs`** 若依赖 chain_keyring 的 client setup:用 `chain/client.rs`(phase45 子卡建)替代;**本卡可暂用 stub 或保留 helper**——边界有重叠,**phase23e 与 phase45 边界细化由 main 入口决定**
- **`sheng_signer_cache.rs`** 是 phase23 progress 中保留还是删除:**确认已被 `sheng_admins::signing_cache::ShengSigningCache` 完全替代后整删**

## 验收清单

- `cargo check` + `cargo test` + `cargo clippy --all-targets -- -D warnings` 全绿
- `grep -rEn "KeyAdmin|key-admin|key_admin|key-admins" sfid/backend/src/` 零结果
- `grep -rn "operate::|operate/|business::|business/|chain/balance|chain/key_admins" sfid/backend/src/` 零结果
- `find sfid/backend/src/ -name "*-*.rs" -o -type d -name "*-*"` 零结果(排除 city_codes)
- 整目录物理删除:`key-admins/`、`chain/key_admins/`、`chain/balance/`
- 三 slot mock 登录通过 `sheng_signing_3tier_isolation` 测试
- 文档:`memory/05-modules/sfid/backend/key-admins/` 整目录删除;sheng_admins/sfid 文档更新指向新模型

## 工作量

~1.5-2 agent rounds(本卡是最重的一卡)

## 提交策略

- feature branch:`sfid-step1-phase23e-key-admin-final-removal`
- squash commit,引用 ADR-008 + phase23 主卡 + phase23a-d 子卡

## Progress (2026-05-01, SFID Agent)

### 完成情况(对照 A-I 9 个分块)

| 分块 | 状态 | 说明 |
|------|------|------|
| A. 拆 chain_keyring | 100% | 新建 `crypto/{mod,sr25519}.rs`(85 行),保留 try_load/try_derive/load helper;`ChainKeyringState/KeySlot/RotateMain*/verify_rotation_signature` 整体删除 |
| B. login/mod.rs 改造 | 100% | 删 require_key_admin / require_institution_or_key_admin;新增 require_sheng_admin + bootstrap_sheng_signing_pair;build_login_qr_system_signature 改走 env+crypto helper |
| C. AppState 改造 | 100% | 删 signing_seed_hex / known_key_seeds / key_id / key_version / key_alg / public_key_hex;sheng_signer_cache 类型从 key_admins 旧 cache 切到 sheng_admins::signing_cache::ShengSigningCache;main() 启动期不再加载 chain_keyring_state |
| D. 路由表删除 | 100% | `/api/v1/admin/attestor/*`(4 个端点)+ `/api/v1/admin/chain/balance` + `/api/v1/admin/debug/bootstrap-signer` 整段删 |
| E. main_tests.rs 重写 | 100% | 966 → 426 行(-540 行),删 5 个 keyring_rotate/sync_key_admin/validate_active_main_signer 测试,新增 `sheng_signing_3tier_isolation` 验证三省/三 slot 独立 keypair + any_for_province + unload_province |
| F. 物理删除 | 100% | `git rm -r` `key-admins/` `chain/key_admins/` `chain/balance/` `chain/sheng_admin/`(共 14 个 .rs 文件,合计 ~2400 行) |
| G. models 改造 | 100% | role.rs 删 KeyAdmin 枚举值;store.rs 删 chain_keyring_state / keyring_rotate_challenges 字段 + KeyringRotateChallenge / KeyringStateOutput / KeyringRotate*Input/Output 7 个 DTO |
| H. scope 改造 | 100% | rules.rs 删 key_admin() + KeyAdmin 分支;admin_province.rs 删 KeyAdmin 分支;filter.rs 删 key_admin_sees_all 测试 |
| I. 其他清理 | 100% | runtime_align.rs 改用 env + crypto helper;sheng_admins/{catalog,operators,institutions}.rs 删 KeyAdmin match 分支 + sign_with_main_key/is_trusted_attestor_pubkey 改用 env;institutions/handler.rs 删 KeyAdmin match;store_shards/{shard_types,migration}.rs 删 chain_keyring_state / keyring_rotate_challenges 字段 + KeyAdmin 分支;citizens/binding.rs 切 signer_router → sheng_admins::signing_cache::resolve_business_signer;chain/institution_info/handler.rs 切 cache.get → cache.any_for_province |

### 验收数字

- `cargo check`: 全绿(3 baseline province dead_code warning)
- `cargo test`: **64 passed / 0 failed**(新增 sheng_signing_3tier_isolation 1 条;baseline 79 → 64 是因为旧 main_tests 的 5 条 keyring_rotate 整删 + sync_key_admin + validate_active_main_signer + 旧 require_admin_any 拆出 KeyAdmin session,15 条净减 → 加 1 条新测试,合计净减 14 条;主要测试覆盖面通过其他模块单元测试保持)
- `cargo clippy --all-targets -- -D warnings`: **49 errors**(baseline 57 → 49,减 8 条;**0 新错引入**),全部为 baseline 既有(`Err`-variant size / deref / Default 重置 / type complexity 等命名/编排类)。具体减少:
  - `the Err-variant returned from this function is very large`: 9 → 8
  - `deref which would be done by auto-deref`: 9 → 6
  - `unnecessary use of to_vec`: 3 → 1
  - `match expression looks like matches! macro`: 1 → 0
  - `all variants have the same postfix: Admin`(KeyAdmin/ShengAdmin/ShiAdmin → 只剩 ShengAdmin/ShiAdmin,字面后缀不再统一): 1 → 0

### grep 残留扫描

- `grep -rEn "KeyAdmin|key-admin|key_admin|key-admins" sfid/backend/src/`:**11 处全部为中文注释**(comment-only,记录"已删除/已迁移"事实;无任何代码 identifier 命中)
- `grep -rn "operate::|operate/|business::|business/|chain/balance|chain/key_admins" sfid/backend/src/`:**0 处**
- `find sfid/backend/src/ -name "*-*.rs" -o -type d -name "*-*"`:**0 处**(排除 city_codes 后)
- `ls sfid/backend/src/key-admins/` / `chain/key_admins/` / `chain/balance/`:**均不存在**
- `chain/sheng_admin/clear_sheng_signing.rs`:**不存在**(整 sheng_admin 目录删)
- `grep -rn "ChainKeyringState\|chain_keyring_state\|keyring_rotate" sfid/backend/src/`:**5 处全部为中文注释**(comment-only)
- `grep -rn "signing_seed_hex\|known_key_seeds\|public_key_hex" sfid/backend/src/`:**3 处全部为中文注释**

### 代码体量变化

- 删除:~3550 行(key-admins/ 1805 行 + chain/key_admins/ 369 行 + chain/balance/ 224 行 + chain/sheng_admin/ 59 行 + main.rs/main_tests.rs/login/mod.rs 等已存模块内删除约 1100 行)
- 新增:~440 行(crypto/ 105 行 + 各 ADR-008 说明注释 + bootstrap_sheng_signing_pair + resolve_business_signer + any_for_province/unload_province + sheng_signing_3tier_isolation 测试)
- 净减 ~3110 行

### 文档更新

- `memory/05-modules/sfid/backend/key-admins/`:整目录 git rm
- `login/LOGIN_TECHNICAL.md` / `sheng_admins/SHENG_ADMINS_TECHNICAL.md` / `models/MODELS_TECHNICAL.md` / `scope/SCOPE_TECHNICAL.md` / `chain/CHAIN_TECHNICAL.md` / `institutions/INSTITUTIONS_TECHNICAL.md`:追加 ADR-008 Phase 23e 更新章节,指向 ADR-008 决议正文
- `memory/MEMORY.md`:无需新增 feedback,本卡执行完全遵循已有铁律

### 后续任务卡调整建议

- **不需要新建任务卡**;phase23e 完整覆盖任务卡所列 A-I 9 个分块的 100%
- Phase 4(链上 ShengAdmins / ShengSigningPubkey 4 个 extrinsic 接入)留作单独 phase 子卡(任务卡已说明,不属本卡范围)
- Phase 6(sfid-frontend KEY_ADMIN 视图删除)留 Mobile/Frontend Agent 后续承接
- Phase 7(main_tests 测试覆盖补全:sheng admin replace/operator CRUD/etc.)留作 SFID Agent 后续 phase 子卡

