//! 预算案子域(Phase 4 预留)。
//!
//! 政府提预算案 → 立法机关**单院常规案**表决(《预算法》授权,非宪法直授,亦非法律案)。
//! 结构 = 国标政府收支科目四级:**类 > 款 > 项 > 目**;金额单位**分**(u128 整数,禁浮点)。
//!
//! 链端现状:**无** `PROPOSAL_KIND_BUDGET`、无预算 extrinsic/pallet(仅 kind 0-3)。故本轮**仅锁
//! 链下字段 schema**(`model`),发起/表决/读链在链端支持后另卡。`类/款/项/目` code 编码规则
//! (国标政府收支分类 vs 自定义)为**显式待定项**,当前 code 以自由文本承载。

/// 预算案字段 schema(`BudgetClass>Section>Item>Subitem` + `BudgetPlan` + `ProposeBudgetInput`)。
pub(crate) mod model;
