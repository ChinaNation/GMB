// 中文注释:按 category + a3 决定"新增机构"弹窗里哪些字段锁定 + 默认值。
// 三个 tab 共用一个 CreateInstitutionModal,只通过 lockProfile 切形态。
//
// 两步式机构创建(2026-04-19 改造):
//   第一步 弹窗:SFR/FFR 只选 A3、P1、institution_code、省/市、T2/C1,仅生成 SFID,
//                **不要求** institution_name / sub_type
//   第二步 详情页:设置 institution_name(全国唯一)、sub_type(SFR)及其他可变信息
//
// A3 联动:
//   SFR(私法人) → P1 可选 0/1;sub_type 选项由 P1 决定(见 subTypeChoicesForP1)
//   FFR(非法人) → P1 可选 0/1;无 sub_type;机构代码只有 ZG/TG
//
// 机构代码选项:
//   GFR → 政府/立法/司法/监察/教委(移除 CB 储备委员会)
//   SFR/FFR → ZG/TG(移除 CH 储备银行,死规则全局删除)

import type { InstitutionCategory } from '../../api/institution';

export type ChoiceItem = { value: string; label: string };

export interface InstitutionFieldLocks {
  /** a3 的候选列表;长度=1 时锁死第一项 */
  a3Choices: ChoiceItem[];
  /** p1 的候选列表;长度=1 时锁死第一项 */
  p1Choices: ChoiceItem[];
  /** institution 代码的候选列表;长度=1 时锁死第一项 */
  institutionChoices: ChoiceItem[];
  /** 机构名称是否锁死为固定值(仅公安局) */
  lockedInstitutionName: string | null;
  /** 弹窗标题 */
  modalTitle: string;
}

// ── A3 类型中文映射 ──
export const A3_LABEL: Record<string, string> = {
  GFR: '公法人',
  SFR: '私法人',
  FFR: '非法人',
  GMR: '公民人',
  ZNR: '自然人',
  ZRR: '智能人',
};

// ── 机构代码中文映射 ──
export const INSTITUTION_CODE_LABEL: Record<string, string> = {
  ZF: '政府',
  LF: '立法院',
  SF: '司法院',
  JC: '监察院',
  JY: '教育委员会',
  ZG: '中国',
  TG: '他国',
};

// ── 公权机构可选机构代码(移除 CB 储备委员会) ──
const GFR_NONPROFIT_GOV: ChoiceItem[] = [
  { value: 'ZF', label: '政府 (ZF)' },
  { value: 'LF', label: '立法院 (LF)' },
  { value: 'SF', label: '司法院 (SF)' },
  { value: 'JC', label: '监察院 (JC)' },
  { value: 'JY', label: '教育委员会 (JY)' },
];

// ── 私法人/非法人可选机构代码(移除 CH 储备银行) ──
const PRIVATE_INSTITUTIONS: ChoiceItem[] = [
  { value: 'ZG', label: '中国 (ZG)' },
  { value: 'TG', label: '他国 (TG)' },
];

// ── 私法人企业类型(详情页使用;P1 联动见 subTypeChoicesForP1) ──
// P1=0 → 仅 NON_PROFIT;P1=1 → 四种企业类型
export const SFR_SUB_TYPE_CHOICES: ChoiceItem[] = [
  { value: 'SOLE_PROPRIETORSHIP', label: '个人独资' },
  { value: 'PARTNERSHIP', label: '合伙企业' },
  { value: 'LIMITED_LIABILITY', label: '有限责任' },
  { value: 'JOINT_STOCK', label: '股份公司' },
  { value: 'NON_PROFIT', label: '公益组织' },
];

// ── 企业类型 label 映射(详情页展示用) ──
export const SUB_TYPE_LABEL: Record<string, string> = {
  SOLE_PROPRIETORSHIP: '个人独资',
  PARTNERSHIP: '合伙企业',
  LIMITED_LIABILITY: '有限责任',
  JOINT_STOCK: '股份公司',
  NON_PROFIT: '公益组织',
};

/** 基础 locks(不依赖 a3 动态值的部分) */
export function locksForCategory(category: InstitutionCategory): InstitutionFieldLocks {
  switch (category) {
    case 'PUBLIC_SECURITY':
      return {
        a3Choices: [{ value: 'GFR', label: '公法人 (GFR)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: [{ value: 'ZF', label: '政府 (ZF)' }],
        lockedInstitutionName: '公民安全局',
        modalTitle: '新增公安局',
      };
    case 'GOV_INSTITUTION':
      return {
        a3Choices: [{ value: 'GFR', label: '公法人 (GFR)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: GFR_NONPROFIT_GOV,
        lockedInstitutionName: null,
        modalTitle: '新增公权机构',
      };
    case 'PRIVATE_INSTITUTION':
      // 两步式:第一步弹窗不含 institution_name/sub_type;P1 可 0/1 由用户选
      return {
        a3Choices: [
          { value: 'SFR', label: '私法人 (SFR)' },
          { value: 'FFR', label: '非法人 (FFR)' },
        ],
        p1Choices: [
          { value: '1', label: '盈利 (1)' },
          { value: '0', label: '非盈利 (0)' },
        ],
        institutionChoices: PRIVATE_INSTITUTIONS,
        lockedInstitutionName: null,
        modalTitle: '新增私权机构',
      };
  }
}

/** 根据 a3 动态计算 P1、机构选项(SFR/FFR 都用同一批 institution 选项)。 */
export function dynamicLocksForA3(a3: string): {
  p1Choices: ChoiceItem[];
  p1Default: string;
  institutionChoices: ChoiceItem[];
} {
  // SFR/FFR 通用:P1 用户可选 0/1;机构代码 ZG/TG
  const p1Choices: ChoiceItem[] = [
    { value: '1', label: '盈利 (1)' },
    { value: '0', label: '非盈利 (0)' },
  ];
  const p1Default = a3 === 'FFR' ? '0' : '1';
  return {
    p1Choices,
    p1Default,
    institutionChoices: PRIVATE_INSTITUTIONS,
  };
}

/**
 * 根据 P1 决定 SFR 详情页可选 sub_type:
 *   P1=0(非盈利) → 只能 NON_PROFIT
 *   P1=1(盈利)   → 四种企业类型,排除 NON_PROFIT
 */
export function subTypeChoicesForP1(p1: string | number): ChoiceItem[] {
  const p1Str = String(p1);
  if (p1Str === '0') {
    return SFR_SUB_TYPE_CHOICES.filter((c) => c.value === 'NON_PROFIT');
  }
  return SFR_SUB_TYPE_CHOICES.filter((c) => c.value !== 'NON_PROFIT');
}
