// 中文注释:按表单入口(category)+ subject_property 决定"新增机构"弹窗里哪些字段锁定 + 默认值。
// private/gov/education 三个新增弹窗共用这一份字段锁定规则,组件分别放在各自业务目录。
//
// 手动新增三个入口(普通公权目录由后端自动生成,公安局不可手动建):
//   PRIVATE_INSTITUTION   私权 tab:按 private_type 锁定主体属性和机构码,创建阶段写入名称
//   GOV_INSTITUTION       公权 tab:G(ZF/LF/SF/JC,排除央行CB)机构名称必填 / F(锁中国ZG)挂公法人
//   EDUCATION_INSTITUTION 教育 tab:G/S/F + 机构锁死教育委员会(JY),学校名称弹窗内必填
//
// P1 盈利属性统一按主体属性联动(见 p1LocksForSubject,与后端号码生成器/uninorg 同源):
//   G → 锁死 0(非盈利,生成器硬规则)
//   S → 可选 0/1,默认 1
//   F → 个体经营/无限合伙为独立非法人;教育分校/公权下属非法人继承所属法人
//
// 非法人(F)挂靠只用于从属非法人。个体经营(F+GT)和无限合伙(F+GP)不选择所属法人。

import type { PartnershipKind, PrivateType } from './api';

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
  JY: '教育委员会',
  CB: '中央银行',
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

// ── 公权机构细类中文映射;仅用于展示,业务仍以 org_code 精确区分。 ──
export const ORG_CODE_LABEL: Record<string, string> = {
  NATIONAL_PRESIDENT_OFFICE: '总统府',
  MINISTRY_FOREIGN: '外事交流部',
  MINISTRY_DEFENSE: '国家防务部',
  MINISTRY_SECURITY: '国土安全部',
  MINISTRY_CIVIL_LIFE: '公民生活保障部',
  MINISTRY_HOUSING: '住房与城镇建设部',
  MINISTRY_AGRICULTURE: '农业与农村发展部',
  MINISTRY_COMMERCE: '商务与市场贸易部',
  MINISTRY_FINANCE_TAX: '财政与税务部',
  MINISTRY_ENERGY: '能源与环保发展部',
  MINISTRY_TRANSPORT: '交通运输部',
  NATIONAL_LEGISLATURE: '国家立法院',
  NATIONAL_COURT: '国家司法院',
  NATIONAL_SUPERVISION: '国家监察院',
  NATIONAL_EDU: '国家教育委员会',
  NATIONAL_RESERVE: '国家储备委员会',
  FEDERAL_SPECIAL_SERVICE: '联邦特勤局',
  FEDERAL_SECURITY: '联邦安全局',
  FEDERAL_INTELLIGENCE: '联邦情报局',
  FEDERAL_PERSONNEL: '联邦人事局',
  FEDERAL_REGISTRY: '联邦注册局',
  FEDERAL_INTEGRITY: '联邦廉政署',
  FEDERAL_AUDIT: '联邦审计署',
  FEDERAL_INVESTIGATION: '联邦调查署',
  NATIONAL_SENATE_COUNCIL: '国家参议会',
  NATIONAL_REPRESENTATIVE_COUNCIL: '国家众议会',
  PROVINCE_GOV: '省政府',
  PROVINCE_LEGISLATURE: '省立法院',
  PROVINCE_COURT: '省司法院',
  PROVINCE_SUPERVISION: '省监察院',
  PROVINCE_RESERVE: '省储备委员会',
  PROVINCE_RESERVE_BANK: '省公民储备银行',
  PROVINCE_DEFENSE: '国家防务厅',
  PROVINCE_SECURITY: '国土安全厅',
  PROVINCE_CIVIL_LIFE: '公民生活保障厅',
  PROVINCE_HOUSING: '住房与城镇建设厅',
  PROVINCE_AGRICULTURE: '农业与农村发展厅',
  PROVINCE_COMMERCE: '商务与市场贸易厅',
  PROVINCE_FINANCE_TAX: '财政与税务厅',
  PROVINCE_ENERGY: '能源与环保发展厅',
  PROVINCE_TRANSPORT: '交通运输厅',
  PROVINCE_SENATE_COUNCIL: '参议员议政会',
  PROVINCE_REPRESENTATIVE_COUNCIL: '众议员议政会',
  CITY_GOV: '自治政府',
  CITY_LEGISLATURE: '公民立法委员会',
  CITY_SUPERVISION: '监察院',
  CITY_COURT: '司法院',
  CITY_EDU: '公民教育委员会',
  CITY_CITIZEN_SELF_GOV: '公民自治委员会',
  CITY_DEFENSE: '国家防务局',
  CITY_SECURITY: '国土安全局',
  CITY_CIVIL_LIFE: '公民生活保障局',
  CITY_HOUSING: '住房与城镇建设局',
  CITY_AGRICULTURE: '农业与农村发展局',
  CITY_COMMERCE: '商务与市场贸易局',
  CITY_FINANCE_TAX: '财政与税务局',
  CITY_ENERGY: '能源与环保发展局',
  CITY_TRANSPORT: '交通运输局',
  CITY_REGISTRY: '身份注册局',
  CITY_POLICE: '公民安全局',
  TOWN_GOV: '自治政府',
  TOWN_CIVIL_LIFE: '公民生活保障科',
  TOWN_HOUSING: '住房与城镇建设科',
  TOWN_AGRICULTURE: '农业与农村发展科',
  TOWN_FINANCE_TAX: '财政与税务科',
  PUBLIC_ORG: '公权机构',
};

// ── 手动公权机构可选代码:排除央行 CB(省公民储备银行每省唯一已生成)和 JY(归教育 tab) ──
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
