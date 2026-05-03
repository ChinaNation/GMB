
- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-sheng-3tier-transition.md`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:phase23a + b + c + d 全部完成

## 任务需求


## 搬迁/删除清单


- `load_signing_key_from_seed` / `derive_pubkey_hex_from_seed` → 迁 `crypto/sr25519.rs`(新文件,放低层)

### login/mod.rs 改造

- 删除 `bootstrap_sheng_signer` 调用,改调 `sheng_admins::bootstrap::ensure_signing_keypair`
- 只保留 SHENG_ADMIN(三 slot)+ SHI_ADMIN

### AppState 改造

```rust
// 删除字段
- signing_seed_hex
- known_key_seeds
- key_id / key_version / key_alg
- public_key_hex
// 保留
+ store / sharded_store / rate_limit_redis / cpms_register_inflight
```

### 路由表删除


### main_tests.rs 重写

- SHENG_ADMIN / SHI_ADMIN 测试保留
- 新加最少:`sheng_signing_3tier_isolation` 1 个集成测试(三 slot 各自独立 keypair)
- 总行数预期 ~500 行(从 966 行降一半)

### 物理删除

- `src/chain/balance/`(整目录)
- `src/chain/sheng_admin/clear_sheng_signing.rs`
- `src/chain/sheng_admin/mod.rs`(若内容只剩 `pub mod clear_sheng_signing;`,直接删 mod.rs;否则保留并精简)

### models 改造


### scope 改造

- `scope::filter_by_scope` KEY 分支删除
- `scope::rules` 中 KEY 相关规则删除

## 影响范围

- 删除:~3000 行
- 修改:`main.rs`、`main_tests.rs`、`login/mod.rs`、`chain/runtime_align.rs`、`models/role.rs`(phase23a 拆分后)、`scope/{filter,rules}.rs`、`store_shards/{shard_types,migration}.rs`、`institutions/handler.rs` 几处分支

## 主要风险点

- **`sheng_signer_cache.rs`** 是 phase23 progress 中保留还是删除:**确认已被 `sheng_admins::signing_cache::ShengSigningCache` 完全替代后整删**

## 验收清单

- `cargo check` + `cargo test` + `cargo clippy --all-targets -- -D warnings` 全绿
- `find sfid/backend/ -name "*-*.rs" -o -type d -name "*-*"` 零结果(排除 city_codes)
- 三 slot mock 登录通过 `sheng_signing_3tier_isolation` 测试

## 工作量

~1.5-2 agent rounds(本卡是最重的一卡)

## 提交策略

- squash commit,引用 ADR-008 + phase23 主卡 + phase23a-d 子卡

## Progress (2026-05-01, SFID Agent)

### 完成情况(对照 A-I 9 个分块)

| 分块 | 状态 | 说明 |
|------|------|------|
| D. 路由表删除 | 100% | `/api/v1/admin/attestor/*`(4 个端点)+ `/api/v1/admin/chain/balance` + `/api/v1/admin/debug/bootstrap-signer` 整段删 |

### 验收数字

- `cargo check`: 全绿(3 baseline province dead_code warning)
- `cargo clippy --all-targets -- -D warnings`: **49 errors**(baseline 57 → 49,减 8 条;**0 新错引入**),全部为 baseline 既有(`Err`-variant size / deref / Default 重置 / type complexity 等命名/编排类)。具体减少:
  - `the Err-variant returned from this function is very large`: 9 → 8
  - `deref which would be done by auto-deref`: 9 → 6
  - `unnecessary use of to_vec`: 3 → 1
  - `match expression looks like matches! macro`: 1 → 0

### grep 残留扫描

- `find sfid/backend/ -name "*-*.rs" -o -type d -name "*-*"`:**0 处**(排除 city_codes 后)
- `chain/sheng_admin/clear_sheng_signing.rs`:**不存在**(整 sheng_admin 目录删)
- `grep -rn "signing_seed_hex\|known_key_seeds\|public_key_hex" sfid/backend/`:**3 处全部为中文注释**

### 代码体量变化

- 新增:~440 行(crypto/ 105 行 + 各 ADR-008 说明注释 + bootstrap_sheng_signing_pair + resolve_business_signer + any_for_province/unload_province + sheng_signing_3tier_isolation 测试)
- 净减 ~3110 行

### 文档更新

- `login/LOGIN_TECHNICAL.md` / `sheng_admins/SHENG_ADMINS_TECHNICAL.md` / `models/MODELS_TECHNICAL.md` / `scope/SCOPE_TECHNICAL.md` / `chain/CHAIN_TECHNICAL.md` / `institutions/INSTITUTIONS_TECHNICAL.md`:追加 ADR-008 Phase 23e 更新章节,指向 ADR-008 决议正文
- `memory/MEMORY.md`:无需新增 feedback,本卡执行完全遵循已有铁律

### 后续任务卡调整建议

- **不需要新建任务卡**;phase23e 完整覆盖任务卡所列 A-I 9 个分块的 100%
- Phase 4(链上 ShengAdmins / ShengSigningPubkey 4 个 extrinsic 接入)留作单独 phase 子卡(任务卡已说明,不属本卡范围)
- Phase 7(main_tests 测试覆盖补全:sheng admin replace/operator CRUD/etc.)留作 SFID Agent 后续 phase 子卡

