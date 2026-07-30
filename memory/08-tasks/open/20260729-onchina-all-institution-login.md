# OnChina 全机构可登录 + 登录错误可诊断性

状态：open（2026-07-29 用户已确认方向并执行完毕；Rust 侧回归验证被 `occupy.rs` 的并发编辑阻塞，待其恢复后重跑）

## 实现（2026-07-29）

### Step 1：授权作用域改为「投影优先 + 机构 CID 兜底」

`auth/repo.rs` 的 `authorization_scope_from_identity_conn`：投影命中则沿用（行为逐字节不变），
投影缺失时用 `cid_scope_codes` 从机构 CID 的 R5（省2+市3）兜底派生，不再 `Err`。

**偏离原方案一处（关键）**：原方案写"非 Tier1 无条件从 CID 派生"，实测发现不可行——
`primitives::cid::code::admin_level` 定义了 **14 种镇级机构码**
（`TGOV/TCWF/THUD/TAGR/TFIN/TDEF/THSC/TCOM/TENR/TTRN/TPOL/TSLF/TSUP/TJUD` → `AdminLevel::Town`），
而 `cid_scope_codes` 只解出省+市、**不含镇码**。无条件改走 CID 会让镇级管理员
`scope_town_name` 恒为 `None`，进而在 `scope::rules` 的 `Some("TOWN")` 分支
fail-closed 成 `VisibleScope::empty()`——镇级管理员看不到任何数据。

因此保留投影优先：
- 90 个公权机构（投影命中）→ 行为不变，含将来镇级机构的镇码
- 私权机构（无投影）→ CID 兜底出省+市；其 `admin_level` 为 `None`，
  `scope::rules` 走 `scope_city_or_empty`，**只需省+市**，正好够用
- CID 也解不出（自然人 CID 等）→ 仍 fail-closed

### Step 2：可诊断性（4 项全做）

1. 新增 `GateError::AuthorizationScopeUnavailable(String)` + `classify_state_error`
   按 `repo::SCOPE_ERROR_PREFIX`（`"scope:"`）前缀分流，与 `qr_login` 的 `http:<kind>:`
   前缀约定同源。映射 `422` + `"authorization scope unavailable: {reason}"`。
   **变体命名偏离原推荐**：原推荐叫 `InstitutionProjectionMissing`，但 Step 1 之后
   "投影缺失"不再是错误，那个名字会指向一个没有产生点的死变体。改为覆盖全部作用域
   派生失败原因（FRG 缺省码 / FRG 省码未知 / 机构 CID 非法 / scope 为空）。
2. `"login persist failed"` → `"login state error"`（它覆盖查询+写入两类，`persist` 一词本身错误）；
   `ONCHINA_LOGIN_PERSIST_FAILED` → `ONCHINA_LOGIN_STATE_ERROR`，前端文案同步。
3. `Db` 分支完整原因只进 `tracing::error!`，响应不带库结构细节；作用域类错误改为
   `tracing::warn!`（不是故障）并把可操作原因带进响应。
4. 新增 4 个单测：作用域错误必带 `scope:` 前缀、非 Tier1 接受 CID 派生的作用域、
   私权机构 CID 能解出省市、自然人 CID fail-closed。

## 验证结果

| 项 | 结果 |
|---|---|
| `cargo check`（改动落地后首次） | 通过 |
| onchina 前端 `tsc -b` | 通过 |
| node 前端 `tsc -b` | 通过 |
| `cargo test` / `clippy` | **阻塞** — 见下 |

### 阻塞项

`domains/citizens/occupy.rs` 正被另一个会话并发修改，当前处于半成品状态
（`encode_admin_rebind_cid_account_id_call` / `verify_admin_rebind_signature` /
`build_domain_sign_request_bytes` 改了签名但调用点未同步），整个 crate 编译不过。
该文件与本卡改动无交集（本卡只动 `auth/repo.rs`、`auth/login/onchain_gate.rs`、`main.rs`）。
待其恢复后重跑 `cargo test` + `cargo clippy --all-targets`，并 `cargo build` 重建二进制。

## 运行态复核（重建重启后执行）

1. `GZ018-SFGYR-201206100-2026` 私权机构管理员完整登录一次。
2. 任一公权机构管理员登录一次，确认作用域与改动前一致。
3. 构造一个作用域无法派生的场景，确认返回 `422` + `ONCHINA_LOGIN_SCOPE_UNAVAILABLE`，
   文案不再是「请稍后重试」。

## 任务需求

用户 2026-07-29 定调：**所有机构都应能登录链上中国平台，不只公权机构。**

实测缺陷：链上 91 个管理员机构中恰好 1 个在本地 `subjects` 投影缺失——
`GZ018-SFGYR-201206100-2026`（`SFGY` 私权机构）。该管理员登录第 1 步成功（`console_admin_pallets`
对 `is_private_legal_code` 放行）、冷钱包签名成功，第 2 步必失败：

```
issue_session_after_onchain_gate
  → candidates_from_memberships_conn
    → derive_candidate_authorization_scope_conn
      → authorization_scope_from_identity_conn        (repo.rs:340)
        → 非 Tier1(FRG) → resolve_binding_candidate_metadata_conn(subjects) → 0 行
          → Err("binding institution metadata not found")   (repo.rs:346)
  → GateError::Db → "login persist failed"           (onchain_gate.rs:87)
  → 前端「登录会话保存失败，请稍后重试」                    (notice.ts:175)
```

旁证：`node_binding_challenges` 与 `node_institution_bindings` 均 0 行，证明流程死在构造候选，
未到会话签发。`subjects` 共 49593 行、72 种 institution_code，`kind` 全为 `PUBLIC`，`SFG*` 0 行。

这是既有缺陷，非三码分类改造引入——旧登录判据要求二维码携带 CID 并双向闭环，私权机构管理员
（私钥在离线冷钱包）根本过不了第 1 步。第 1 步的死锁解开后，下游这个从未被触达的缺口才暴露。

## 已确认边界

- **授权作用域真源改为 CID 本身，不再依赖 `subjects` 投影。** 现成原语
  `cid_scope_codes(raw) -> Result<([u8;2],[u8;3])>`（`runtime/primitives/cid/number.rs:84`）对非自然人
  CID 返回 (省码, 市码)；onchina 内已有先例 `genesis_projection.rs:32` 的 `split_province_city`。
  符合 `china-code-immutable`、`institution-bundle-code-from-cid`、ADR-021（存 code + join 显示）。
- **不放宽授权**：`authorization_scope_from_sources` 对非 FRG 强制要求 scope 是安全边界
  （`scope-auto-filter`：作用域过滤 fail-closed），不得改成允许 None。
- **不新建私权机构投影链路**：投影是缓存，会漂移（`registry-regen-after-genesis` 已踩过）。
- 机构名称（`cid_full_name`/`cid_short_name`）仍来自投影，私权机构显示空——这是既定 fail-safe
  降级（`repo.rs:207` 注释：「无对应行返回 None，前端按空处理，绝不另造名字」），不阻断登录。
  机构名有唯一生成器，可后续单独补，不在本卡。
- 不修改 `citizenchain/runtime/`、`citizenchain/pallets/`。

## 实施步骤

### Step 1：授权作用域改从 CID 派生

`citizenchain/onchina/src/auth/repo.rs` 的 `authorization_scope_from_identity_conn`：
非 Tier1 分支不再要求 `subjects` 命中，改为 `cid_scope_codes` 解出省市码 → `cid::china::area_display_names`
取名称。投影查不到不再是错误。

### Step 2：可诊断性

1. 新增 `GateError::InstitutionProjectionMissing(String)`（携带 cid_number），
   `gate_error_response` 映射 `422` + `"institution projection missing"`，前端给可操作指引。
2. `GateError::Db` 文案 `"login persist failed"` → `"login state error"`；它覆盖查询+写入两类，
   `persist` 一词本身错误。前端 `ONCHINA_LOGIN_PERSIST_FAILED` 同步改名。
3. `GateError::Db` 响应体带不含敏感信息的 `reason` 短码，供前端展示——真实原因目前只在
   onchina 的 `tracing::error!` 里，而它继承桌面端 stdout（`cmd.spawn()` 未设 Stdio）。
4. 补单测钉住「非 Tier1 + 投影缺失」不得报成 persist failed（当前无 HTTP 集成基建，
   测 `authorization_scope_from_identity_conn` 的错误分类即可）。

## 主要风险

- Step 1 动的是**鉴权路径**。改完必须复核：公权机构 scope 与改动前逐字一致（90 个机构回归）、
  FRG 仍走 `frg_province_code` 分支不受影响、私权机构能拿到正确省市。
- `cid_scope_codes` 对自然人 CID 返回 Err（`is_person_code` 分支）。机构候选不会传自然人 CID，
  但错误路径要 fail-closed，不得吞掉。
- 改完需 `cargo build` 重建二进制并在节点设置页重启链上中国才生效。

## 完成标准

- `GZ018-SFGYR-201206100-2026` 私权机构管理员能完成登录全流程。
- 90 个公权机构 scope 派生结果与改动前一致。
- 投影缺失场景返回专用错误码，文案不再是「请稍后重试」。
- `cargo test` 全绿、`cargo clippy --all-targets` 不新增 warning。
