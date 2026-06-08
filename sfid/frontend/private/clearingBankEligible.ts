// 清算行资格白名单(2026-04-24, ADR-007)— 前端复刻版本
//
// 与后端 sfid/backend/subjects/service.rs::is_clearing_bank_eligible 严格一致:
//   - S + sub_type=JOINT_STOCK            → ✅
//   - F + parent.S + parent.JOINT_STOCK → ✅
//   - 其他                                   → ❌
//
// 详细规则见:
//   memory/04-decisions/ADR-007-clearing-bank-three-phase.md
//   memory/05-modules/sfid/clearing-bank-eligibility.md

/** 资格判定所需的最小字段集(覆盖 InstitutionListRow / Institution / ParentInstitutionRow)。 */
export interface ClearingEligibleInst {
  subject_property: string;
  sub_type?: string | null;
  parent_sfid_number?: string | null;
}

/** 资格判定所需的 parent 最小字段集。 */
export interface ClearingEligibleParent {
  subject_property: string;
  sub_type?: string | null;
}

/**
 * 单机构(S)直接判定 — 不需要 parent 信息。
 *
 * 用于 PrivateListTable 列渲染:列表行只有自身字段,没有 parent 详情,
 * 因此 F 一律不在列表里直接判定为"可作为清算行";其资格在详情页有 parent 信息后再判定。
 */
export function isSelfEligibleClearingBank(inst: ClearingEligibleInst): boolean {
  return inst.subject_property === 'S' && inst.sub_type === 'JOINT_STOCK';
}

/**
 * 完整判定 — 需要 parent(若 subject_property=F)。
 *
 * 用于详情页 / PrivateDetailLayout:有完整的机构 + 所属法人信息后做权威判定。
 */
export function isClearingBankEligible(
  inst: ClearingEligibleInst,
  parent: ClearingEligibleParent | null | undefined,
): boolean {
  if (inst.subject_property === 'S') {
    return inst.sub_type === 'JOINT_STOCK';
  }
  if (inst.subject_property === 'F') {
    if (!parent) return false;
    return parent.subject_property === 'S' && parent.sub_type === 'JOINT_STOCK';
  }
  return false;
}

/** badge 文案常量。 */
export const CLEARING_BANK_ELIGIBLE_LABEL = '可作为清算行';

/** sub_type=JOINT_STOCK 时给注册者的提示文案(PrivateDetailLayout 第二步用)。 */
export const JOINT_STOCK_CLEARING_HINT = '股份公司可参与清算业务,可在区块链节点软件中注册为清算行。';
