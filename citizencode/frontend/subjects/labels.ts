// 中文注释:按表单入口(category)+ subject_property 决定"新增机构"弹窗里哪些字段锁定 + 默认值。
// private/gov/education 三个新增弹窗共用这一份字段锁定规则,组件分别放在各自业务目录。
//
// 手动新增三个入口(普通公权目录由后端自动生成,公安局不可手动建):
//   PRIVATE_INSTITUTION   私权 tab:按 private_type 锁定主体属性和机构码,创建阶段写入名称
//   GOV_INSTITUTION       公权 tab:G(ZF/LF/SF/JC,排除储备体系自动目录代码)机构名称必填 / F(锁中国ZG)挂公法人
//   EDUCATION_INSTITUTION 教育 tab:G/S/F + 机构锁死教育委员会(JY);
//                         G/S 学校必须选择教育机构类型,F+JY 分校保留原挂靠规则
//
// P1 盈利属性统一按主体属性联动(见 p1LocksForSubject,与后端号码生成器/uninorg 同源):
//   G → 锁死 0(非盈利,生成器硬规则)
//   S → 可选 0/1,默认 1
//   F → 个体经营/无限合伙为独立非法人;教育分校/公权下属非法人继承所属法人
//
// 非法人(F)挂靠只用于从属非法人。个体经营(F+GT)和无限合伙(F+GP)不选择所属法人。

import type { EducationType, PartnershipKind, PrivateType } from './api';

export type ChoiceItem = { value: string; label: string };

/** 手动新增表单的入口类型(查询/存储 category 见 subjects/api.ts 的 InstitutionCategory)。 */
export type CreateFormCategory =
  | 'PRIVATE_INSTITUTION'
  | 'GOV_INSTITUTION'
  | 'EDUCATION_INSTITUTION';

export interface InstitutionFieldLocks {
  /** subject_property 的候选列表;长度=1 时锁死第一项 */
  subjectPropertyChoices: ChoiceItem[];
  /** institution 代码的初始候选列表(随主体属性变化见 institutionChoicesFor) */
  institutionChoices: ChoiceItem[];
  /** 弹窗标题 */
  modalTitle: string;
}

// ── SubjectProperty 中文映射 ──
export const SUBJECT_PROPERTY_LABEL: Record<string, string> = {
  G: '公法人',
  S: '私法人',
  F: '非法人',
  M: '公民',
  Z: '自然人',
  N: '智能人',
};

// ── 机构代码中文映射 ──
export const INSTITUTION_CODE_LABEL: Record<string, string> = {
  ZF: '政府',
  LF: '立法院',
  SF: '司法院',
  JC: '监察院',
  JY: '公民教育委员会',
  CB: '公民储备委员会',
  CH: '公民储备委员会',
  ZG: '中国',
  TG: '他国',
  GT: '个体经营',
  GP: '无限合伙',
  LP: '有限合伙',
  GQ: '股权公司',
  GF: '股份公司',
  GY: '公益组织',
  AS: '注册协会',
};

export const EDUCATION_TYPE_LABEL: Record<EducationType, string> = {
  NATIONAL_CITIZEN_EDU_COMMITTEE: '国家公民教育委员会',
  CITY_CITIZEN_EDU_COMMITTEE: '市公民教育委员会',
  EARLY_SCHOOL: '初学',
  PRIMARY_SCHOOL: '小学',
  SECONDARY_SCHOOL: '中学',
  UNIVERSITY: '大学',
};

export const SCHOOL_EDUCATION_TYPE_OPTIONS: ChoiceItem[] = [
  { value: 'EARLY_SCHOOL', label: EDUCATION_TYPE_LABEL.EARLY_SCHOOL },
  { value: 'PRIMARY_SCHOOL', label: EDUCATION_TYPE_LABEL.PRIMARY_SCHOOL },
  { value: 'SECONDARY_SCHOOL', label: EDUCATION_TYPE_LABEL.SECONDARY_SCHOOL },
  { value: 'UNIVERSITY', label: EDUCATION_TYPE_LABEL.UNIVERSITY },
];

export const SCHOOL_EDUCATION_TYPES: EducationType[] = [
  'EARLY_SCHOOL',
  'PRIMARY_SCHOOL',
  'SECONDARY_SCHOOL',
  'UNIVERSITY',
];

// ── 手动公权机构可选代码:排除储备委员会/省储行(已确定性生成)和 JY(归教育 tab) ──
const GOV_MANUAL_INSTITUTIONS: ChoiceItem[] = [
  { value: 'ZF', label: '政府' },
  { value: 'LF', label: '立法院' },
  { value: 'SF', label: '司法院' },
  { value: 'JC', label: '监察院' },
];

// ── 公权下属非法人机构代码锁死中国(ZG),不开放他国 ──
const GOV_UNINORG_INSTITUTION_ONLY: ChoiceItem[] = [
  { value: 'ZG', label: '中国' },
];

// ── 教育委员会(JY)锁死选项 ──
const EDUCATION_INSTITUTION_ONLY: ChoiceItem[] = [
  { value: 'JY', label: '教育委员会' },
];

// ── P1 盈利属性选项(单一来源,锁死场景取单项) ──
const P1_PROFIT: ChoiceItem = { value: '1', label: '盈利' };
const P1_NON_PROFIT: ChoiceItem = { value: '0', label: '非盈利' };

export const PRIVATE_TYPE_LABEL: Record<PrivateType, string> = {
  SOLE: '个体经营',
  PARTNERSHIP: '合伙企业',
  COMPANY: '股权公司',
  CORPORATION: '股份公司',
  WELFARE: '公益组织',
  ASSOCIATION: '注册协会',
};

export const PARTNERSHIP_KIND_LABEL: Record<PartnershipKind, string> = {
  GENERAL: '无限合伙',
  LIMITED: '有限合伙',
};

export interface PrivateTypeRule {
  privateType: PrivateType;
  partnershipKind?: PartnershipKind;
  subjectProperty: 'S' | 'F';
  institution: string;
  p1: '0' | '1';
  hasLegalPersonality: boolean;
}

export const PRIVATE_TYPE_RULES: Record<PrivateType, PrivateTypeRule> = {
  SOLE: {
    privateType: 'SOLE',
    subjectProperty: 'F',
    institution: 'GT',
    p1: '1',
    hasLegalPersonality: false,
  },
  PARTNERSHIP: {
    privateType: 'PARTNERSHIP',
    partnershipKind: 'GENERAL',
    subjectProperty: 'F',
    institution: 'GP',
    p1: '1',
    hasLegalPersonality: false,
  },
  COMPANY: {
    privateType: 'COMPANY',
    subjectProperty: 'S',
    institution: 'GQ',
    p1: '1',
    hasLegalPersonality: true,
  },
  CORPORATION: {
    privateType: 'CORPORATION',
    subjectProperty: 'S',
    institution: 'GF',
    p1: '1',
    hasLegalPersonality: true,
  },
  WELFARE: {
    privateType: 'WELFARE',
    subjectProperty: 'S',
    institution: 'GY',
    p1: '0',
    hasLegalPersonality: true,
  },
  ASSOCIATION: {
    privateType: 'ASSOCIATION',
    subjectProperty: 'S',
    institution: 'AS',
    p1: '0',
    hasLegalPersonality: true,
  },
};

export function privateRuleFor(
  privateType: PrivateType,
  partnershipKind?: PartnershipKind,
): PrivateTypeRule {
  if (privateType === 'PARTNERSHIP') {
    if (partnershipKind === 'LIMITED') {
      return {
        privateType,
        partnershipKind,
        subjectProperty: 'S',
        institution: 'LP',
        p1: '1',
        hasLegalPersonality: true,
      };
    }
    return { ...PRIVATE_TYPE_RULES.PARTNERSHIP, partnershipKind: 'GENERAL' };
  }
  return PRIVATE_TYPE_RULES[privateType];
}

/** 基础 locks(不依赖 subject_property 动态值的部分) */
export function locksForCategory(category: CreateFormCategory): InstitutionFieldLocks {
  switch (category) {
    case 'GOV_INSTITUTION':
      // G=新公权机构(名称必填同市查重) / F=公权下属非法人(挂公法人)
      return {
        subjectPropertyChoices: [
          { value: 'G', label: '公法人' },
          { value: 'F', label: '非法人' },
        ],
        institutionChoices: GOV_MANUAL_INSTITUTIONS,
        modalTitle: '新增公权机构',
      };
    case 'EDUCATION_INSTITUTION':
      // G=公立学校 / S=私立学校 / F=分校(挂本部),机构锁死教育委员会(JY)
      return {
        subjectPropertyChoices: [
          { value: 'G', label: '公法人' },
          { value: 'S', label: '私法人' },
          { value: 'F', label: '非法人' },
        ],
        institutionChoices: EDUCATION_INSTITUTION_ONLY,
        modalTitle: '新增教育机构',
      };
    case 'PRIVATE_INSTITUTION':
      return {
        subjectPropertyChoices: [{ value: 'F', label: '非法人' }],
        institutionChoices: [{ value: 'GT', label: '个体经营' }],
        modalTitle: '新增机构',
      };
  }
}

/** 机构代码选项随入口 + 主体属性联动(GOV 入口 G 建公权机构、F 建下属非法人锁中国)。 */
export function institutionChoicesFor(
  category: CreateFormCategory,
  subjectProperty: string,
): ChoiceItem[] {
  if (category === 'EDUCATION_INSTITUTION') return EDUCATION_INSTITUTION_ONLY;
  if (category === 'GOV_INSTITUTION') {
    return subjectProperty === 'G' ? GOV_MANUAL_INSTITUTIONS : GOV_UNINORG_INSTITUTION_ONLY;
  }
  return [{ value: 'GT', label: '个体经营' }];
}

/** 非法人盈利属性附属于所属法人:公法人父恒 0,私法人父继承其 p1(与后端 uninorg 同源)。 */
export function inheritedP1(parentSubjectProperty: string, parentP1: string): string {
  return parentSubjectProperty === 'G' ? '0' : parentP1;
}

/**
 * P1 盈利属性统一按主体属性联动(三入口共用,与号码生成器/uninorg 同源):
 *   G → 锁死 0(非盈利);S → 可选 0/1 默认 1;
 *   F → 锁死,继承所属法人;未选父级前 value=undefined(表单必填挡提交)。
 */
export function p1LocksForSubject(
  subjectProperty: string,
  parent: { subject_property: string; p1: string } | null,
): { choices: ChoiceItem[]; value: string | undefined; locked: boolean } {
  if (subjectProperty === 'S') {
    return { choices: [P1_PROFIT, P1_NON_PROFIT], value: '1', locked: false };
  }
  if (subjectProperty === 'F') {
    if (!parent) return { choices: [], value: undefined, locked: true };
    const v = inheritedP1(parent.subject_property, parent.p1);
    return { choices: [v === '1' ? P1_PROFIT : P1_NON_PROFIT], value: v, locked: true };
  }
  // G:公法人恒非盈利
  return { choices: [P1_NON_PROFIT], value: '0', locked: true };
}
