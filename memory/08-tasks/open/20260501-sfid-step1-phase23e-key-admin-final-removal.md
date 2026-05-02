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
