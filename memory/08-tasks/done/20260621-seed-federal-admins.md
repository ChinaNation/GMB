# 任务卡：P0 止血 — 联邦注册局管理员 seed-federal-admins

- 任务编号：20260621-seed-federal-admins
- 状态：done
- 所属模块：citizencode/backend/admins
- 当前负责人：CID Agent
- 创建时间：2026-06-21

## 任务需求

重新创世后 CID `admins` 表为空,联邦注册局 215 名管理员全部无法扫码登录(`get_admin_by_account_conn` 返回 None → 403「非管理员禁止登录本系统」)。新增 `seed-federal-admins` CLI,从编译期创世常量 `china_zf.rs`「总统府联邦注册局」条目直接恢复 215 名管理员的 CID 登录投影,不依赖链、不需重新创世即可止血。这是 ADR-023 双通道投影上线前的应急 / 离线引导路径。

## 落地内容

- `citizencode/backend/gov/service.rs`：新增 `federal_registry_admins()`,仅取「总统府联邦注册局」单条 admins(不混入安全局/情报局等其它联邦机构管理员)。
- `citizencode/backend/admins/seed.rs`（新建）：`run_seed_federal_admins`,按 43 省序(每 5 人一省)写 `admins` + `federal_registry_scope`;0x 小写 hex;`built_in=true`、`created_by=SYSTEM`;含长度守卫断言 + 2 个单测。
- `citizencode/backend/admins/mod.rs`：挂载 `pub(crate) mod seed;`。
- `citizencode/backend/main.rs`：`BackendCommand::SeedFederalAdmins` + `"seed-federal-admins"` 解析 + `run_gov_directory_command` 分发。

## 验证

- `cargo check` 通过(30s);`cargo test seed::` 2/2 通过。
- 实跑 `citizencode-backend seed-federal-admins`:`seeded=215 provinces=43`。
- 开发库:`admins=215 / federal_registry_scope=215 / 43 省各 5 人`;用户报障的 `0xd641dbfe…9930` 已就位(FEDERAL_REGISTRY, 中枢省, built_in)。

## 必须遵守（已遵守）

- 唯一真源仍是链上 admins-change;本播种只是应急/离线兜底,不进运行时热路径(见 ADR-023)。
- 幂等 `ON CONFLICT(admin_account) DO UPDATE`,可反复运行。

## 后续

- Phase 1 双通道投影上线后,本 CLI 退居"重新创世/链不可达"应急位:见 `20260621-admins-chain-sync.md`。
- 链端联邦注册局自治阻塞:见 `20260621-admins-change-builtin-pup-selfgovern.md`。

## 待确认问题

- 暂无
