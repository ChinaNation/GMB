//! 任免案子域(Phase 4 预留)。
//!
//! 中文注释:政府提「人事任免职书」→ 参议会(NSN)/省参议会(PSN)/市立法会(CLEG)**单院常规案**
//! 表决任免(宪法第100/106条;副总统/部长/省长/市长任免走第53/55/57/64条)。**非法律案**,
//! 无公投/签署/护宪。
//!
//! 链端现状:**无** `PROPOSAL_KIND_PERSONNEL`、无任免 extrinsic/pallet(仅 kind 0-3)。故本轮**仅锁
//! 链下字段 schema**(`model`),发起/表决/读链在链端支持后另卡(含 runtime 二次确认 + 重新创世)。
//! 升级路径(3 次驳回→重要案→直授)字段化随该链路上线时定,本轮不引入。

/// 任免职书字段 schema(`PersonnelAction` / `PersonnelDecision` / `ProposePersonnelInput`)。
pub(crate) mod model;
