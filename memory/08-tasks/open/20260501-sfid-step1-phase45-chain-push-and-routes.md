# SFID Step 1 / Phase 4+5:chain push 模块 + 路由 / AppState 收敛

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-sheng-admin-3tier.md`(主卡)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:Phase 2+3 卡(本卡需要 `sheng_admins/` 业务先就绪)
- 阻塞下游:Phase 6(前端 API 对接)
- 联调依赖:Step 2(citizenchain runtime)— 链上 4 个 Pays::No extrinsic 落地后才能去 mock

## 任务需求

实现 SFID 主动推链能力:
- `chain/sheng_admin/` — 管理员名册操作(add/remove backup)
- `chain/sheng_signer/` — 签名密钥生命周期(activation / rotation)
- `chain/sheng_admin/query.rs` — 拉链上 3 slot 当前 pubkey

收敛 `main.rs` 路由表与 `AppState`。本卡推链调用先返回 mock 成功,等 Step 2 链上 extrinsic 上线后真正联调(联调由 Phase 7 收尾)。

## 建议模块

- `sfid/backend/chain/`:新增 `sheng_admin/`(扩展)+ `sheng_signer/`
- `sfid/backend/main.rs`:路由表收敛 + AppState 字段精简

## 影响范围(文件级)

### 新建 `chain/sheng_admin/`

| 文件 | 内容 |
|---|---|
| `mod.rs` | 模块导出 + 顶部中文注释 |
| `handler.rs` | HTTP handler:`GET /chain/sheng-admin/list?province=AH` 公开 endpoint(链反向调) |
| `query.rs` | `pub async fn fetch_roster(province: Province) -> Result<[Option<[u8;32]>; 3]>`:从链上读 `ShengAdmins[province][slot]` 三槽(mock:返回 `[Some(province.main_admin_pubkey), None, None]`) |
| `add_backup.rs` | `pub async fn add_backup(session, slot, new_pubkey)`:构造 `add_sheng_admin_backup(province, slot, new_pubkey, sig_by_main)` extrinsic;**Pays::No**;immortal + 显式 nonce + 等 InBestBlock(mock:直接返回 `Ok(MockTxHash)`) |
| `remove_backup.rs` | 同结构,推 `remove_sheng_admin_backup`(mock 同上) |

### 新建 `chain/sheng_signer/`

| 文件 | 内容 |
|---|---|
| `mod.rs` | 模块导出 |
| `handler.rs` | HTTP handler:`POST /sheng-signer/activate` + `POST /sheng-signer/rotate`(需 session) |
| `activation.rs` | `pub async fn activate(session) -> Result<()>`:构造 `activate_sheng_signing_pubkey(province, admin_pubkey, signing_pubkey, sig)`,sig = admin 私钥签 `(province, signing_pubkey, nonce)`,**Pays::No**(mock:返回 ok) |
| `rotation.rs` | `pub async fn rotate(session, new_signing_pubkey)`:**Pays::No**;流程同 activate(mock) |

### 共享:`chain/client.rs`(新建,从 `chain/runtime_align.rs` 抽出)

- 单例 subxt client
- `pub async fn submit_immortal_paysno(call, signer) -> Result<TxStatus>`:封装 SFID 推链三件套(显式 nonce + immortal + Pays::No + 等 InBestBlock + 1010 错误友好转换)

### `main.rs` 路由表收敛(目标终态)

```
POST /login/sheng-admin/challenge
POST /login/sheng-admin/verify
POST /login/shi-admin/challenge
POST /login/shi-admin/verify
POST /logout

GET  /sheng-admin/dashboard
GET  /sheng-admin/roster                  → chain/sheng_admin/handler::list_roster
POST /sheng-admin/roster/add-backup       → chain/sheng_admin/add_backup
POST /sheng-admin/roster/remove-backup    → chain/sheng_admin/remove_backup
POST /sheng-signer/activate               → chain/sheng_signer/activation
POST /sheng-signer/rotate                 → chain/sheng_signer/rotation

GET  /shi-admin/dashboard
POST /shi-admin/manage/add
POST /shi-admin/manage/revoke

GET  /institutions
POST /institutions
GET  /institutions/:sfid_id
POST /institutions/:sfid_id/credential

GET  /citizens/:identity_id
POST /citizens/:identity_id/binding-credential
POST /citizens/:identity_id/vote-credential

GET  /chain/institution-info/:sfid_id
GET  /chain/joint-vote/snapshot[?province=AH]
GET  /chain/citizen-binding/:identity
GET  /chain/citizen-vote/:identity
GET  /chain/sheng-admin/list[?province=AH]
```

### `AppState` 字段精简

- 保留:`rate_limit_redis` / `cpms_register_inflight`
- 保留:`store` / `sharded_store`

## 主要风险点

- **mock 推链返回的可观察性**:mock 返回 `Ok(MockTxHash)` 容易让前端误以为已经上链。建议:mock 路径 logger emit `WARN: chain push mocked, awaiting Step 2`。
- **chain/runtime_align.rs 重复**:旧文件已经有 subxt client,新 `chain/client.rs` 抽出后必须删旧路径,不留双重单例。
- **`/chain/sheng-admin/list` 是公开 endpoint(链反向调)**:无 session 必须做 IP allowlist 或仅在内网暴露;路由表里需 mark 与其他 chain pull endpoint 同级。
- **路由顺序**:`api/v1/` 前缀 vs 当前的 `/sheng-admin/` 等无版本前缀,需要先沟通——本卡建议沿用 `/api/v1/` 前缀(SFID 后端约定)。
- **AppState 删字段连锁影响**:`signing_seed_hex` 在 `main_tests.rs` 等多处被引用,删除时必须配套清理测试。

## 是否需要先沟通

- **是 1 项**:`api/v1/` 路由前缀是否保留?当前实际代码用 `/api/v1/admin/sheng-admins`,本卡方案文 "路由表全景" 给的是去前缀形式;建议保留 `/api/v1/` 前缀以维持 API contract。
- 其余按方案直接执行

## 建议下一步

1. 抽 `chain/runtime_align.rs` 中 client 单例 → `chain/client.rs`,加 `submit_immortal_paysno` helper
2. 新建 `chain/sheng_admin/{handler,query,add_backup,remove_backup}.rs`,推链先 mock
3. 新建 `chain/sheng_signer/{handler,activation,rotation}.rs`,推链先 mock
5. `AppState` 字段精简
6. `cargo check` + `cargo test` 全绿
7. **更新文档**:`memory/05-modules/sfid/backend/chain/` 加 sheng_admin/sheng_signer 模块说明
8. **完善注释**:新模块顶部 1-3 行 + activation/rotation 推链流程详解

## 验收清单

- `cargo check` + `cargo clippy -- -D warnings` + `cargo test` 全绿
- 路由表与 ADR-008 对齐
- mock 推链 logger 可观察(stderr/log/trace)
- 新建 4 个 endpoint 可用 curl 打通(走 mock)
- `chain/runtime_align.rs` client 单例迁移,旧路径 grep 零结果

## 工作量预估

- 净增:~+800 行(主要是 chain/sheng_admin + chain/sheng_signer + helper)
- 工时:~1.5d 集中开发 + 0.5d 文档/残留

## 提交策略

- feature branch:`sfid-step1-phase45-chain-push-and-routes`
- 单 PR 落地,commit message 引用任务卡 + ADR-008
- 留 TODO 标记:`// TODO(step2-联调): 替换 mock 为真实推链 - 见任务卡 phase7`

---

## Progress(2026-05-01)

### A. `chain/sheng_admin/`(管理员名册) — 完工

- [x] `mod.rs`(30 行,顶部 `//!` 注释:Pays::No / 1010 错误规避 / phase45 mock 标注)
- [x] `query.rs`(67 行,`fetch_roster` mock 返回 `[Some(main), None, None]`,带 `tracing::warn!("chain pull mocked, awaiting Step 2")`,附 2 个 tokio test)
- [x] `add_backup.rs`(129 行,service + handler;require_sheng_admin → 取 ctx.admin_province → mock 推链)
- [x] `remove_backup.rs`(110 行,同结构)
- [x] `handler.rs`(137 行,`list_roster_admin` session 版 + `list_roster_public` 公开版,共享 `render_roster` helper)

### B. `chain/sheng_signer/`(签名密钥) — 完工

- [x] `mod.rs`(17 行)
- [x] `activation.rs`(117 行,从 cache 取 keypair → 推 mock activate;cache miss 时返回 1503)
- [x] `rotation.rs`(112 行,接收 `new_signing_pubkey` 入参 → 推 mock rotate)
- [x] `handler.rs`(10 行 re-export 占位,phase7 改造时统一入口)

### C. `chain/client.rs`(共享 helper) — 完工(独立文件,未抽 runtime_align)

- [x] `client.rs`(105 行)
- [x] `MockTxHash::placeholder()` 返回固定 `0x...beef` hash
- [x] `submit_immortal_paysno_mock(extrinsic_label)` emit `tracing::warn!(extrinsic, "chain push mocked, awaiting Step 2")`
- [x] `ChainPushError` 枚举占位:`NotImplemented` / `InvalidTx` / `Other`
- [x] runtime_align.rs 不动(按"卡住时策略",phase7 切真时再合并)

### D. `main.rs` 路由表收敛 — 完工

新增 6 个 endpoint(任务卡指定 7 个,其中 `/api/v1/admin/sheng-admin/dashboard` 已存在为 `sheng_admins::list_sheng_admins`,任务卡说明"若已有就好"未重复挂):

```
GET  /api/v1/admin/sheng-admin/roster              → chain::sheng_admin::handler::list_roster_admin    (admin session)
POST /api/v1/admin/sheng-admin/roster/add-backup   → chain::sheng_admin::add_backup::handler           (admin session)
POST /api/v1/admin/sheng-admin/roster/remove-backup→ chain::sheng_admin::remove_backup::handler        (admin session)
POST /api/v1/admin/sheng-signer/activate           → chain::sheng_signer::activation::handler          (admin session)
POST /api/v1/admin/sheng-signer/rotate             → chain::sheng_signer::rotation::handler            (admin session)
GET  /api/v1/chain/sheng-admin/list?province=XX    → chain::sheng_admin::handler::list_roster_public   (公开,挂在 app_routes 全局 rate limit 之后)
```

### E. `AppState` 字段精简 — 无新增,与 phase23e 既有结构对齐

- 保留:`store` / `rate_limit_redis` / `cpms_register_inflight` / `sheng_signer_cache` / `sharded_store`
- AppState struct 未改

### Cargo 终态

- `cargo check`:全绿,**3 baseline warning**(province.rs `name`/`code`/`towns` dead_code,与 baseline 一致)
- `cargo test`:**66 passed / 0 failed**(baseline 64 + query.rs 新增 2 个 tokio test)
- `cargo clippy --all-targets -- -D warnings`:**51 errors**(与 baseline 51 持平,未引入新错)

### Mock helper 关键字标记(grep 可观察)

- `chain push mocked, awaiting Step 2` — `chain/client.rs::submit_immortal_paysno_mock`
- `chain pull mocked, awaiting Step 2` — `chain/sheng_admin/query.rs::fetch_roster`
- `[chain push] add_sheng_admin_backup 即将提交` 等 4 条 `tracing::info!` 业务前缀,phase7 切真时一并替换为真实 tx 进度

### 后续任务卡调整建议

- 不需要调整。phase7(mock → real)预计仅替换 `chain/client.rs::submit_immortal_paysno_mock` 内部、新增 subxt 单例、保持 service / handler 接口稳定
- 是否抽公共 subxt OnlineClient 单例,phase7 决定;phase45 不强求
- 残留 `sheng_admins/roster.rs::push_chain_mock`(roster.rs 同名旧 mock,99 行起)与新 chain/sheng_admin/add_backup 路径重复:phase7 切真时收敛二者,phase45 暂保留(business service 与 chain push extrinsic 是两层职责)
