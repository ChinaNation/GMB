# 任务卡：Phase 1 — admins-change 链→postgres 双通道投影

- 任务编号：20260621-admins-chain-sync
- 状态：open
- 所属模块：citizencode/backend（admins + indexer）
- 当前负责人：CID Agent
- 创建时间：2026-06-21

## 任务需求

把链上 `admins-change::AdminAccounts` 作为唯一真源,CID 用双通道投影进 postgres `admins` 表,使创世写入与运行期变更都能自动同步,长期取代 P0 的 seed CLI。详见 ADR-023。

## 落地内容

- 新建 `citizencode/backend/admins/chain_sync.rs`：
  - `spawn_admins_snapshot`（通道①启动全量快照）：因创世 `build()` 零 `deposit_event`,必须 subxt `storage().at(finalized).iter("AdminsChange","AdminAccounts")` 全量迭代,按机构主账户反查 `subjects.org_code` 只投影 FEDERAL_REGISTRY/CITY_REGISTRY,逐条 `upsert_admin_from_chain_conn`。
  - `sync_admins_from_block_events`（通道②indexer 增量）：监听 finalized 的 `AdminSetChanged/AdminAccountActivated/AdminAccountClosed`（事件只含 `admins_len` 不含名单,必须按 account 做 storage 点查取最新 admins）。
- `citizencode/backend/admins/repo.rs`：`upsert_admin_from_chain_conn`/`delete_admin_from_chain_conn`;`delete_admin_runtime_state_conn` 调用必须包单事务（HIGH-4）。
- `citizencode/backend/main.rs`：启动 `tokio::spawn(spawn_admins_snapshot)`,失败仅 warn 不阻断 serve。
- `citizencode/backend/indexer/worker.rs`：`process_block_at_hash` 尾部挂同步钩子。

## 必须遵守（评审阻塞项,务必先堵）

- **reconcile 只在快照完整跑完无错才删,且删前阈值守卫（联邦 admin < 200 放弃删除）**;`federal_registry_scope` 有 ON DELETE CASCADE,删 admin 连带删 scope,需单事务（CRITICAL-3）。
- **冲突键统一 `lower(admin_account)`,写前强制 0x 小写 hex**（HIGH-1）。
- **登录读投影不读链**;高危写动作（签发机构凭证、CPMS）commit 前实时点查 AdminAccounts 防窗口期提权。
- 动态 SCALE 解码 AdminAccount 需一条"解出含某已知创世 pubkey"的集成测试兜底（MEDIUM-2）。
- 只投影注册局(FEDERAL/CITY)管理员进登录表;普通 PUP 机构(公安局等)管理员不进登录表,机构详情"管理员 tab"改对该机构 AdminAccounts 实时点查显示。

## 依赖

- 链端 `20260621-admins-change-builtin-pup-selfgovern.md`（联邦自治阻塞,决定 PUP kind 语义,影响投影分类）。

## 输出物

- 代码 + 中文注释 + 集成测试 + 文档更新 + 残留清理

## 待确认问题

- 机构详情"管理员 tab"用实时点查还是单独投影表(建议实时点查,永远最新)。
