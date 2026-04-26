// 清算行资格白名单(2026-04-24, ADR-007)— 前端复刻版本
//
// 与后端 sfid/backend/src/institutions/service.rs::is_clearing_bank_eligible 严格一致:
//   - SFR + sub_type=JOINT_STOCK            → ✅
//   - FFR + parent.SFR + parent.JOINT_STOCK → ✅
//   - 其他                                   → ❌
//
// 详细规则见:
//   memory/04-decisions/ADR-007-clearing-bank-three-phase.md
//   memory/05-modules/sfid/clearing-bank-eligibility.md

/** 资格判定所需的最小字段集(覆盖 InstitutionListRow / MultisigInstitution / ParentInstitutionRow)。 */
export interface ClearingEligibleInst {
  a3: string;
  sub_type?: string | null;
  parent_sfid_id?: string | null;
}

/** 资格判定所需的 parent 最小字段集。 */
export interface ClearingEligibleParent {
  a3: string;
  sub_type?: string | null;
}

/**
 * 单机构(SFR)直接判定 — 不需要 parent 信息。
 *
 * 用于 InstitutionListTable 列渲染:列表行只有自身字段,没有 parent 详情,
 * 因此 FFR 一律不在列表里直接判定为"可作为清算行";其资格在详情页有 parent 信息后再判定。
 */
export function isSelfEligibleClearingBank(inst: ClearingEligibleInst): boolean {
  return inst.a3 === 'SFR' && inst.sub_type === 'JOINT_STOCK';
}

/**
 * 完整判定 — 需要 parent(若 a3=FFR)。
 *
 * 用于详情页 / PrivateInstitutionLayout:有完整的机构 + 所属法人信息后做权威判定。
 */
export function isClearingBankEligible(
  inst: ClearingEligibleInst,
  parent: ClearingEligibleParent | null | undefined,
): boolean {
  if (inst.a3 === 'SFR') {
    return inst.sub_type === 'JOINT_STOCK';
  }
  if (inst.a3 === 'FFR') {
    if (!parent) return false;
    return parent.a3 === 'SFR' && parent.sub_type === 'JOINT_STOCK';
  }
  return false;
}

/** badge 文案常量。 */
export const CLEARING_BANK_ELIGIBLE_LABEL = '可作为清算行';

/** sub_type=JOINT_STOCK 时给注册者的提示文案(PrivateInstitutionLayout 第二步用)。 */
export const JOINT_STOCK_CLEARING_HINT = '股份公司可参与清算业务,可在区块链节点软件中注册为清算行。';
