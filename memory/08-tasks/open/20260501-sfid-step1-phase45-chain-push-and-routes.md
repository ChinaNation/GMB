# SFID Step 1 / Phase 4+5:chain push 模块 + 路由 / AppState 收敛

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-sheng-admin-3tier-and-key-admin-removal.md`(主卡)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
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

- `sfid/backend/src/chain/`:新增 `sheng_admin/`(扩展)+ `sheng_signer/`
- `sfid/backend/src/main.rs`:路由表收敛 + AppState 字段精简

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

- 删除:`signing_seed_hex`(KEY_ADMIN 全局 seed,不再需要)
- 删除:`known_key_seeds`(KEY_ADMIN 已知 seed cache)
- 保留:`rate_limit_redis` / `cpms_register_inflight`
- 改造:`sheng_signer_cache` 类型从 `key_admins::sheng_signer_cache::ShengSignerCache` 改 `sheng_admins::signing_cache::ShengSigningCache`(Phase 2+3 卡已实现)
- 删除:`key_id` / `key_version` / `key_alg` / `public_key_hex`(KEY_ADMIN 顶级签名 metadata)
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
4. `main.rs` 路由表:删 KEY_ADMIN 旧路由(若 Phase 2 没删干净)+ 加 sheng-admin/sheng-signer 新路由
5. `AppState` 字段精简
6. `cargo check` + `cargo test` 全绿
7. **更新文档**:`memory/05-modules/sfid/backend/chain/` 加 sheng_admin/sheng_signer 模块说明
8. **完善注释**:新模块顶部 1-3 行 + activation/rotation 推链流程详解
9. **清理残留**:Grep `runtime_align.*Client|chain_keyring` 零结果

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
