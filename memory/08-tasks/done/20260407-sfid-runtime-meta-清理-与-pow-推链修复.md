# 任务卡：移除 sfid runtime_meta 空壳持久化 + 修复 sfid→PoW 链推链失败

- 创建日期：2026-04-07
- 完成日期：2026-04-07
- 模块归属：SFID Agent (`sfid/backend`, `sfid/deploy`, `sfid/backend/db/migrations`)
- 状态：✅ 已完成

> 第一阶段：清理 runtime_meta 空壳（已完成）。
> 第二阶段：用户在 dev 环境点"生成机构 SFID"持续失败，深挖后发现是
> sfid 推链路径在 PoW 链下三处 subxt 默认行为踩坑 + 一个 INSTITUTION_DOMAIN
> 类型 bug，一并修复。详见底部"第二阶段"。

## 背景

`sfid/backend` 的 `runtime_meta` 表与配套加解密逻辑历史上用于持久化主签名人状态，
现已彻底废弃：

- `load_runtime_state` 解密后 `_snapshot` 立即丢弃
- `persist_runtime_state` 写入的只是 `{version: 2}` 空壳
- 代码注释已明确："runtime_meta 现在只作为'运行态元信息占位'，不再恢复任何主私钥/主公钥状态"
- 但 `SFID_RUNTIME_META_KEY` 仍是启动期 `required_env`，导致缺失即 panic→destructor 二次 panic→abort

## 目标

完全删除 `runtime_meta` 相关代码、env 变量、部署脚本和数据库表，
让 sfid backend 启动不再依赖 `SFID_RUNTIME_META_KEY`。

## 改动范围

### 代码
- `backend/src/main.rs`：删 bootstrap DDL、`required_env` 校验、启动期 load/persist 调用、`load_runtime_state` / `persist_runtime_state_checked` / `persist_runtime_state` 三个函数
- `backend/src/models/mod.rs`：删 `PersistedRuntimeMeta` 及其 `Debug impl`
- `backend/src/app_core/http_security.rs`：删 `runtime_meta_cipher_key`
- 17 处 `persist_runtime_state(&state)` 调用点全部移除（key-admins / operate / super-admins）
- `backend/src/main_tests.rs`：删相关 `set_var` 与 `PersistedRuntimeMeta` 测试

### 部署 / 文档
- `deploy/prod/scripts/install_sfid_app.sh`：删 `SFID_RUNTIME_META_KEY` 默认值与必需变量校验
- `.env.dev.local` / `.env.example` / 任何文档里残留的该变量

### 数据库
- 新增 `backend/db/migrations/003_drop_runtime_meta.sql`：`DROP TABLE IF EXISTS runtime_meta;`
- 不修改既有 `002_runtime_store.sql`（保留历史）

## 验收

1. `cargo check` / `cargo build` 通过
2. `bash sfid-run.sh` 在不设置 `SFID_RUNTIME_META_KEY` 的情况下启动成功
3. `grep -r runtime_meta sfid/` 仅剩 migration 文件
4. 回写 `memory/MEMORY.md` 一条 feedback，避免被恢复

## 风险

- 生产环境仍存在 `runtime_meta` 表：新代码不再访问，无害；migration 在确认无人依赖后再执行
- 滚动升级顺序：**先部署新代码 → 稳定 → 跑 003 migration**

---

## 第二阶段：sfid 后端推链失败修复

### 现象

`/api/v1/multisig/sfid/generate` 持续失败，前端显示"chain register failed"。
中途经历的 audit 错误依次为：

1. `Connection refused` —— `.env.dev.local` 配 9944 而链一度跑在其他端口
2. `Invalid Transaction (1010) AncientBirthBlock` —— mortal era birth block 视图不一致
3. `Invalid Transaction (1010) Stale` —— nonce 视图不一致
4. `timed out waiting for finalization` —— GRANDPA finality 在 PoW 下慢于 120s 业务超时
5. `Pallet error: DuoqianManagePow::InvalidSfidInstitutionSignature` —— SCALE 编码差异

### 真正根因（共 4 个，不是同一个问题）

| # | 根因 | 表面错误 |
|---|------|---------|
| ① | env 配置端口与实际链端口不一致 | Connection refused |
| ② | subxt 0.43 默认 mortal era 用 finalized 块 hash 当 birth block，链端 best 视图查不到 | AncientBirthBlock |
| ③ | subxt 0.43 默认从 finalized 块读 nonce，PoW 链 finalized 远落后 best | Stale |
| ④ | sfid 调 `wait_for_finalized()`，PoW 链 GRANDPA finality 慢于业务超时 | timed out |
| ⑤ | `INSTITUTION_DOMAIN` 被错误声明为 `&[u8]` 而非 `[u8; 23]`，SCALE 编码多 1 字节长度前缀 | InvalidSfidInstitutionSignature |

② ③ ④ 共同根源：subxt 0.43 默认行为是为 PoS 链(finality≈head)设计的，
PoW 链下 finalized 显著落后 best，所有依赖 finalized 视图的默认逻辑都踩坑。

### 修复

代码改动：

- `sfid/backend/src/super-admins/institutions.rs::submit_register_sfid_institution_extrinsic`
  - 用 `LegacyRpcMethods::system_account_next_index` 显式取 best+pool 视图 nonce
  - `DefaultExtrinsicParamsBuilder::new().immortal().nonce(N).build()`
  - submit 后只等 `TxStatus::InBestBlock`（用 `submitted.next()` loop）
- `sfid/backend/src/key-admins/mod.rs::submit_rotate_sfid_keys_extrinsic`
  - 同上三件套
- `sfid/backend/src/chain/runtime_align.rs`
  - `INSTITUTION_DOMAIN: [u8; 23] = *b"GMB_SFID_INSTITUTION_V2"`（之前是 `&[u8]`；该域名已于 2026-04-20 彻底退役，改用 `DUOQIAN_DOMAIN + OP_SIGN_INST`）
  - 文件顶部加注释，警告任何 `*_DOMAIN` 常量必须用数组类型
- `sfid/.env.dev.local`
  - `SFID_CHAIN_RPC_URL=http://127.0.0.1:9944`（与实际链端口对齐）

文档：

- ADR-005-sfid-subxt-0.43-pow-chain-quirks.md
- memory feedback：`feedback_sfid_pow_chain_recipe.md`、`feedback_scale_domain_must_be_array.md`

### 第二阶段验收

1. ✅ `cargo build` 通过
2. ✅ 前端"生成机构 SFID"成功上链（用户实测）
3. ⏳ 链上验证收到 SFID 号 → 后续独立任务卡

### 教训

- subxt 0.43 默认参数对 PoW 链不适用，必须四件全配齐：显式 nonce + immortal + 等 best + 协议 domain 类型对齐
- 任何与链端 verifier 共享的 SCALE message domain 必须用 `[u8; N]` 数组
- 诊断顺序应当从"链端实际错误码"出发(audit_logs 里的 RawValue)，
  而不是先猜外部因素（节点端口、finality、矿工算力）
