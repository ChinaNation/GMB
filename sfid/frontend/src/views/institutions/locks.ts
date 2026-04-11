// 中文注释:按 category + a3 决定"新增机构"弹窗里哪些字段锁定 + 默认值。
// 三个 tab 共用一个 CreateInstitutionModal,只通过 lockProfile 切形态。
// a3 联动规则:
//   SFR(私法人) → P1 锁盈利(1),必须选企业类型;仅股份公司可选储备银行(CH)
//   FFR(非法人) → P1 锁非盈利(0),机构只能选 ZG/TG,无企业类型

import type { InstitutionCategory } from '../../api/institution';

export type ChoiceItem = { value: string; label: string };

export interface InstitutionFieldLocks {
  /** a3 的候选列表;长度=1 时锁死第一项 */
  a3Choices: ChoiceItem[];
  /** p1 的候选列表;长度=1 时锁死第一项 */
  p1Choices: ChoiceItem[];
  /** institution 代码的候选列表;长度=1 时锁死第一项 */
  institutionChoices: ChoiceItem[];
  /** 私法人子类型候选列表(仅 SFR 有值,其余为空数组) */
  subTypeChoices: ChoiceItem[];
  /** 机构名称是否锁死为固定值 */
  lockedInstitutionName: string | null;
  /** 弹窗标题 */
  modalTitle: string;
}

// ── 公权机构可选机构代码 ──
const GFR_NONPROFIT_GOV: ChoiceItem[] = [
  { value: 'ZF', label: '政府 (ZF)' },
  { value: 'LF', label: '立法院 (LF)' },
  { value: 'SF', label: '司法院 (SF)' },
  { value: 'JC', label: '监察院 (JC)' },
  { value: 'JY', label: '教育委员会 (JY)' },
  { value: 'CB', label: '储备委员会 (CB)' },
];

// ── 私法人(SFR)可选机构代码 ──
// 完整列表(含 CH),仅股份公司可用
const SFR_INSTITUTIONS_ALL: ChoiceItem[] = [
  { value: 'ZG', label: '中国 (ZG)' },
  { value: 'CH', label: '储备银行 (CH)' },
  { value: 'TG', label: '他国 (TG)' },
];
// 不含 CH,个人独资/合伙企业/有限责任使用
const SFR_INSTITUTIONS_NO_CH: ChoiceItem[] = [
  { value: 'ZG', label: '中国 (ZG)' },
  { value: 'TG', label: '他国 (TG)' },
];

// ── 非法人(FFR)可选机构代码:不含 CH ──
const FFR_INSTITUTIONS: ChoiceItem[] = [
  { value: 'ZG', label: '中国 (ZG)' },
  { value: 'TG', label: '他国 (TG)' },
];

// ── 私法人子类型 ──
export const SFR_SUB_TYPE_CHOICES: ChoiceItem[] = [
  { value: 'SOLE_PROPRIETORSHIP', label: '个人独资' },
  { value: 'PARTNERSHIP', label: '合伙企业' },
  { value: 'LIMITED_LIABILITY', label: '有限责任' },
  { value: 'JOINT_STOCK', label: '股份公司' },
];

/** 基础 locks(不依赖 a3 动态值的部分) */
export function locksForCategory(category: InstitutionCategory): InstitutionFieldLocks {
  switch (category) {
    case 'PUBLIC_SECURITY':
      return {
        a3Choices: [{ value: 'GFR', label: '公法人 (GFR)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: [{ value: 'ZF', label: '政府 (ZF)' }],
        subTypeChoices: [],
        lockedInstitutionName: '公民安全局',
        modalTitle: '新增公安局',
      };
    case 'GOV_INSTITUTION':
      return {
        a3Choices: [{ value: 'GFR', label: '公法人 (GFR)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: GFR_NONPROFIT_GOV,
        subTypeChoices: [],
        lockedInstitutionName: null,
        modalTitle: '新增公权机构',
      };
    case 'PRIVATE_INSTITUTION':
      // 默认 SFR;a3 切换时由 Modal 动态覆盖 p1/institution/subType
      return {
        a3Choices: [
          { value: 'SFR', label: '私法人 (SFR)' },
          { value: 'FFR', label: '非法人 (FFR)' },
        ],
        p1Choices: [{ value: '1', label: '盈利 (1)' }],
        institutionChoices: SFR_INSTITUTIONS_NO_CH,
        subTypeChoices: SFR_SUB_TYPE_CHOICES,
        lockedInstitutionName: null,
        modalTitle: '新增私权机构',
      };
  }
}

/** 根据 a3 动态计算 P1、机构选项、子类型选项(仅 PRIVATE_INSTITUTION 会变) */
export function dynamicLocksForA3(a3: string): {
  p1Choices: ChoiceItem[];
  p1Default: string;
  institutionChoices: ChoiceItem[];
  subTypeChoices: ChoiceItem[];
} {
  if (a3 === 'FFR') {
    return {
      p1Choices: [{ value: '0', label: '非盈利 (0)' }],
      p1Default: '0',
      institutionChoices: FFR_INSTITUTIONS,
      subTypeChoices: [],
    };
  }
  // SFR 默认不含 CH;选择企业类型后由 institutionChoicesForSubType 细化
  return {
    p1Choices: [{ value: '1', label: '盈利 (1)' }],
    p1Default: '1',
    institutionChoices: SFR_INSTITUTIONS_NO_CH,
    subTypeChoices: SFR_SUB_TYPE_CHOICES,
  };
}

/** 根据企业类型(sub_type)决定 SFR 可选的机构代码:仅股份公司(JOINT_STOCK)可选储备银行(CH) */
export function institutionChoicesForSubType(subType: string | undefined): ChoiceItem[] {
  if (subType === 'JOINT_STOCK') {
    return SFR_INSTITUTIONS_ALL;
  }
  return SFR_INSTITUTIONS_NO_CH;
}
