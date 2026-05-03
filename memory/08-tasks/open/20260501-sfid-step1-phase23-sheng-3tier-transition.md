
- 状态:in_progress(Phase 3 增量基础设施完成,phase23 单卡拆 5 张连环子卡 a/b/c/d/e)
- **本卡降级为 phase23 板块 overview**,实际执行单元下沉到 5 张子卡:
  - `memory/08-tasks/open/20260501-sfid-step1-phase23a-models-mod-split.md`
  - `memory/08-tasks/open/20260501-sfid-step1-phase23b-rsa-blind-relocate.md`
  - `memory/08-tasks/open/20260501-sfid-step1-phase23c-business-to-scope.md`
  - `memory/08-tasks/open/20260501-sfid-step1-phase23d-operate-to-citizens.md`
  - `memory/08-tasks/open/20260501-sfid-step1-phase23e-legacy-role-cleanup.md`
- 创建日期:2026-05-01
- 模块:`sfid/backend`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-sheng-admin-3tier.md`(主卡,Phase 1 已完成)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:Phase 1(已完成,目录已重命名)
- 阻塞下游:Phase 4+5(本卡完工 build 绿后才能继续)

## 任务需求


## 建议模块


## 影响范围(文件级)

### 删除清单(Phase 2)

| 路径 | 说明 |
|---|---|
| `src/chain/balance/`(整目录) | SFID 不查链上余额 |
| `src/chain/sheng_admin/clear_sheng_signing.rs` + `mod.rs` | 旧推链逻辑(Phase 4 重写) |
| `src/business/`(整目录) | 含 `business::scope::in_scope_cpms_site` 等,功能迁 `scope/` |
| `src/operate/`(整目录) | 内容迁 `citizens/`(本卡同步建) |
| `scope::filter_by_scope` KEY 分支 | |
| `AppState` KEY 相关字段(`known_key_seeds`、`rate_limit_redis` KEY 路径等) | |
| `sfid/frontend/src/views/keyring/` 目录 | (不在本卡范围,Phase 6 处理) |

### 新增清单(Phase 3)

| 路径 | 说明 |
|---|---|
| `src/sfid/province.rs` 加 `ProvinceAdmins { main, backup_1, backup_2 }` 结构 | main 沿用现有 const,backup 字段为 `Option<[u8;32]>`(默认 None) |
| `src/models/{role,slot,session,permission,error}.rs` | 拆 `models/mod.rs`(1021 行)→ 6 文件;`Slot::{Main,Backup1,Backup2}` |
| `src/sheng_admins/login.rs` | 受理 main + 链上记录 backup 公钥的签名挑战(链上拉公钥先 mock) |
| `src/sheng_admins/bootstrap.rs` | 首登生成签名 seed,加密落盘;**不**推链(留 Phase 4) |
| `src/sheng_admins/signing_cache.rs` | `Mutex<HashMap<(Province, [u8;32]), Sr25519Pair>>` |
| `src/sheng_admins/roster.rs` | 名册操作 service(实际推链留 Phase 4) |
| `src/store_shards/sheng_signer.rs` | 加密 seed 持久化:`storage/sheng_signer/{province}_{pubkey_hex}.enc` |
| `src/citizens/{mod,handler,binding,vote}.rs` | `operate/binding.rs` `operate/cpms_qr.rs` 内容迁入 |
| `src/institutions/policies/{mod,private,gov,public_security}.rs` | 三类机构策略(从 `institutions/handler.rs` 抽出) |
| `src/sheng_admins/handler.rs` | 重写,适配三 slot |

### 业务凭证签发改造

| 模块 | 改造 |
|---|---|
| `institutions/credential.rs`(新建) | 凭证签发使用 `session.signing_pair`(从 cache 取) |
| `citizens/binding.rs` | 同上 |
| `citizens/vote.rs` | 同上 |
| `shi_admins/*` | 所有市管理员业务凭证改用本省登录省管理员的签名密钥(经由 session 透传) |

## 主要风险点

- **build 中断窗口**:本卡是大改动,需要分小步 commit 但避免主分支 build 红。建议在 feature branch 落地,内部 squash commit 后一次性合主。
- **`models/mod.rs` 1021 行拆分**:类型互引用复杂,split 后所有 `pub use` 需精确;建议保留 `models/mod.rs` 作为 re-export hub,实际定义下沉到 6 个子文件。
- **`bootstrap_sheng_signer` 替换**:旧逻辑在管理员登录时按全局 KEK 推链注册签名公钥;新逻辑改为按 `(province, admin_pubkey)` 落盘 + cache,**不**推链(推链留 Phase 4 调用 chain/sheng_signer/activation),Step 2 链上未就绪时业务凭证签发可继续工作(签名公钥未上链情况下,链上验签会拒绝—但 Step 1 仅完成 SFID 端逻辑)。
- **`operate/cpms_qr.rs` 是否属于 citizens**:cpms_qr 涉及 CPMS 站点扫码,可能不归 citizens。**先沟通边界**或单独留为 `cpms/`(SFID 内子模块)。
- **`business/scope::in_scope_cpms_site`**:被 `main.rs:43` 引入,需要先确定其语义后决定迁入 `scope/` 还是删除。

## 是否需要先沟通

- **是 1 项**:`operate/cpms_qr.rs` 归属 — 是否合并到 `citizens/` 还是独立 `cpms/` 子模块?
- 其余按方案直接执行

## 建议下一步

按以下顺序在 feature branch 内推进,每步 cargo check 验证:

1. 拆 `models/mod.rs` → 6 文件,保 mod.rs 作 re-export hub,cargo check 全绿
2. 在 `src/sfid/province.rs` 加 `ProvinceAdmins` 结构 + `Slot` 枚举(从 models 引用)
3. 新建 `src/citizens/`,从 `operate/` 复制内容(暂双份共存,后删 operate)
4. 新建 `src/sheng_admins/{login,bootstrap,signing_cache,roster,handler}.rs`,先实现 sheng_admins 自治路径(不推链)
5. 业务凭证签发函数全部接入 `signing_cache`(institutions/citizens/shi_admins)
6. 删 `business/`,移 `in_scope_cpms_site` 到合适位置
11. 删 `operate/` 整目录
13. cargo check + clippy + cargo test 全绿
15. **完善注释**:每个新模块顶部 1-3 行中文说明

## 验收清单

- `cargo check` + `cargo clippy -- -D warnings` + `cargo test` 全绿
- 数据库 migration 提供 down 路径(本卡只删表,down 暂留空 + comment)
- 任意 main / backup_1 / backup_2 admin 模拟登录(用 mock 链上 backup 列表)→ 各自独立签名密钥 → 凭证签发 OK

## 工作量预估

- 工时:~3d 集中开发 + 1d cargo 修复迭代 + 0.5d 文档/残留

## 提交策略

- squash commit 后合主,commit message 引用任务卡 + ADR-008
- PR 描述贴上 grep 残留检查输出

## Progress(2026-05-01,SFID Agent 第 1 轮执行)

### 已完成

- **新基础设施(增量、保持 build/test/clippy 无回归)**
  - `src/sfid/province.rs`:新增 `Slot { Main, Backup1, Backup2 }`、`ProvinceAdmins`、`pubkey_from_hex`、`fetch_backup_admins`(Phase 4 mock,固定返回 [None, None] + tracing::warn 提示)、`province_admins_for`。所有新条目挂 `#[allow(dead_code)]` 不污染 clippy 基线
  - `src/sheng_admins/signing_cache.rs`:`ShengSigningCache`,`Mutex<HashMap<(Province, [u8;32]), Sr25519Pair>>`,`load/evict/get/active_count/pair_from_seed`
  - `src/sheng_admins/bootstrap.rs`:`ensure_signing_keypair(cache, province, admin_pubkey)` 实现"已存在密文 → 解密 → 载入 / 否则 → 随机 32-byte seed → 加密落盘 → 载入",seed 用 `Zeroizing` 包裹,**不**推链(留 Phase 4 子卡)
  - `src/sheng_admins/roster.rs`:`add_backup / remove_backup`,内部 `push_chain_mock`(`tracing::warn!("chain push mocked for {name}, awaiting Phase 4 real impl")` 后返回 Ok),滑入 `RosterError` 枚举
  - `src/store_shards/sheng_signer.rs`:AES-256-GCM 加密 seed 持久化,wrap key = `HKDF(SFID_MASTER_KEK 或 fallback SFID_SIGNING_SEED_HEX, salt = admin_pubkey, info = "sfid-sheng-signer-3tier-v1")`,文件路径 `storage/sheng_signer/{province}_{pubkey_hex}.enc`(目录可由 `SFID_SHENG_SIGNER_DIR` 覆盖,测试用)。带两条单元测试:`encrypt_decrypt_roundtrip_in_memory`、`roundtrip_seed_persistence`(均 ok)
- **数据库 migration**
- **mod 注册**
  - `src/store_shards/mod.rs` 加 `pub(crate) mod sheng_signer;`
  - `src/sheng_admins/mod.rs` 加 `signing_cache / bootstrap / roster` 三个子模块
- **验收命令(本轮终态)**
  - `cargo check` 全绿(3 warnings,均为 sfid/province.rs 的 ProvinceCode 字段 `name/code/villages/towns` dead_code,这些是 baseline 既有,与本卡无关)
  - `cargo test` 79 passed / 0 failed(含本卡新增 2 条 sheng_signer 测试,含 main_tests 全部通过)
  - `cargo clippy --all-targets -- -D warnings` 59 errors —— **基线既有 59,本卡未引入新错**(本卡新增的 dead_code 全部加 `#[allow(dead_code)]` 显式抑制,bootstrap.rs `&*seed_arr` 已修为 `&seed_arr`)
- **残留 grep(本卡未做大刀阔斧的删除,故仍有大量残留,本节如实记录,在 `sfid/backend/` 范围)**
  - `operate::|operate/|business::|business/` 合计:**32 条**(`operate/`、`business/` 整目录 + 各 caller)
  - **本卡新增的 `Slot/ProvinceAdmins/fetch_backup_admins/province_admins_for/pubkey_from_hex` + `signing_cache.rs/bootstrap.rs/roster.rs/store_shards/sheng_signer.rs` + 014 SQL** 全部 build 通过 + 测试通过 + clippy 不污染基线;其余删除全部留给 phase23a-e 5 张子卡(见下)

### 未完成 / 阻塞 / 需后续调整

  - `business/scope::province_scope_for_role` 被 `sheng_admins/{catalog,operators}.rs`、`login/mod.rs` 共用,删除前需要在 `scope/` 下提供等价 service;`business::pubkey::{normalize_admin_pubkey, same_admin_pubkey}` 同样在 institutions/handler.rs、sheng_admins/{catalog,operators}.rs、login/mod.rs 多处使用;
  - `operate/binding.rs`(967 行) 是公民绑定的 8 条核心路由实现,迁移至 `citizens/binding.rs` 后还要全量改写所有 `crate::*` glob 引用、`AppState` 私有字段访问、`StoreBackend` enum 匹配等。

**结论**:本卡声明的 18 步必须串成一个 ~10K 行、跨周的大改动并附带配套测试重写,显著超出单轮 SFID Agent 工作单元能覆盖的范围。本轮严格按 task card 的"卡住时策略"——做完所有可独立完成、不破坏 build 的增量基础设施,把后续巨量删改记录到 progress 章节,**绝不闷头让 build 红**。

### 建议任务卡拆分(给主入口评审)

把当前 phase23 子卡进一步拆为下列 4 张子卡逐个落地,每张都能在一个工作单元内完工 + 通过验收:

3. **phase23c-business-scope-relocate**:`business::pubkey::*` + `business::scope::*` 内容并入 `scope/` 子模块,所有 caller 一次性替换;`business/audit.rs` + `business/query.rs` 直接迁入 `citizens/`;删除 `business/` 目录
4. **phase23d-operate-to-citizens**:把 `operate/{binding,cpms_qr,status}.rs` 迁入 `citizens/`,所有 caller(含 `main.rs` 路由 + `shi_admins/mod.rs` 转发)一次性替换;删除 `operate/` 目录


### 待 Phase 4 / Phase 5 决议事项

- `operate/cpms_qr.rs` 归属:cpms_qr 涉及 CPMS 站点扫码,不属于纯公民业务。建议:留为 `citizens/cpms_qr.rs` stub,标注"待 Phase 4 评审是否搬到 `cpms/` 子模块"
- `bootstrap.rs` Phase 3 mock 阶段不推链;Phase 4 子卡 `activate_sheng_signing_pubkey` extrinsic 接入后,需要在 `ensure_signing_keypair` 首次生成 seed 分支末尾追加链上注册步骤(`feedback_sfid_pow_chain_recipe.md`:显式 nonce + immortal + 等 InBestBlock)
- `roster.rs` Phase 4 子卡接入 `add_sheng_admin_backup / remove_sheng_admin_backup` extrinsic 真实推链,删除 `push_chain_mock`
- `province.rs::fetch_backup_admins` Phase 4 子卡接入 `ShengAdmins[Province][Slot]` chain pull,删除 mock + tracing::warn

