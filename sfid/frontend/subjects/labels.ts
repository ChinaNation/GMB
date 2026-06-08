// 中文注释:按 category + subject_property 决定"新增机构"弹窗里哪些字段锁定 + 默认值。
// gov/private 新增弹窗共用这一份字段锁定规则,但组件分别放在各自业务目录。
//
// 两步式机构创建(2026-04-19 改造):
//   第一步 弹窗:S/F 只选 SubjectProperty、P1、institution_code、省/市、T2/C1,仅生成 SFID,
//                **不要求** institution_name / sub_type
//   第二步 详情页:设置 institution_name(全国唯一)、sub_type(S)及其他可变信息
//
// SubjectProperty 联动:
//   S(私法人) → P1 可选 0/1;sub_type 选项由 P1 决定(见 subTypeChoicesForP1)
//   F(非法人) → P1 可选 0/1;无 sub_type;机构代码为 ZG/TG/教育委员会(JY)
//
// 机构代码选项:
//   G → 仅允许手动新增教育委员会(JY)类型学校机构,普通公权机构由后端自动生成
//   S/F → ZG/TG/教育委员会(JY)

import type { InstitutionCategory } from './api';

export type ChoiceItem = { value: string; label: string };

export interface InstitutionFieldLocks {
  /** subject_property 的候选列表;长度=1 时锁死第一项 */
  subjectPropertyChoices: ChoiceItem[];
  /** p1 的候选列表;长度=1 时锁死第一项 */
  p1Choices: ChoiceItem[];
  /** institution 代码的候选列表;长度=1 时锁死第一项 */
  institutionChoices: ChoiceItem[];
  /** 机构名称是否锁死为固定值(仅公安局) */
  lockedInstitutionName: string | null;
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

// ── 公权机构手动新增只保留教育委员会类型学校机构 ──
const G_NONPROFIT_GOV: ChoiceItem[] = [
  { value: 'JY', label: '教育委员会 (JY)' },
];

// ── 私法人/非法人可选机构代码 ──
const PRIVATE_INSTITUTIONS: ChoiceItem[] = [
  { value: 'ZG', label: '中国 (ZG)' },
  { value: 'JY', label: '教育委员会 (JY)' },
  { value: 'TG', label: '他国 (TG)' },
];

// ── 私法人企业类型(详情页使用;P1 联动见 subTypeChoicesForP1) ──
// P1=0 → 仅 NON_PROFIT;P1=1 → 四种企业类型
export const S_SUB_TYPE_CHOICES: ChoiceItem[] = [
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

/** 基础 locks(不依赖 subject_property 动态值的部分) */
export function locksForCategory(category: InstitutionCategory): InstitutionFieldLocks {
  switch (category) {
    case 'PUBLIC_SECURITY':
      return {
        subjectPropertyChoices: [{ value: 'G', label: '公法人 (G)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: [{ value: 'ZF', label: '政府 (ZF)' }],
        lockedInstitutionName: null,
        modalTitle: '公安局',
      };
    case 'GOV_INSTITUTION':
      return {
        subjectPropertyChoices: [{ value: 'G', label: '公法人 (G)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: G_NONPROFIT_GOV,
        lockedInstitutionName: null,
        modalTitle: '新增机构',
      };
    case 'PRIVATE_INSTITUTION':
      // 两步式:第一步弹窗不含 institution_name/sub_type;P1 可 0/1 由用户选
      return {
        subjectPropertyChoices: [
          { value: 'S', label: '私法人 (S)' },
          { value: 'F', label: '非法人 (F)' },
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

/** 根据 subject_property 动态计算 P1、机构选项(S/F 都用同一批 institution 选项)。 */
export function dynamicLocksForSubjectProperty(subject_property: string): {
  p1Choices: ChoiceItem[];
  p1Default: string;
  institutionChoices: ChoiceItem[];
} {
  // S/F 通用:P1 用户可选 0/1;机构代码 ZG/TG
  const p1Choices: ChoiceItem[] = [
    { value: '1', label: '盈利 (1)' },
    { value: '0', label: '非盈利 (0)' },
  ];
  const p1Default = subject_property === 'F' ? '0' : '1';
  return {
    p1Choices,
    p1Default,
    institutionChoices: PRIVATE_INSTITUTIONS,
  };
}

/**
 * 根据 P1 决定 S 详情页可选 sub_type:
 *   P1=0(非盈利) → 只能 NON_PROFIT
 *   P1=1(盈利)   → 四种企业类型,排除 NON_PROFIT
 */
export function subTypeChoicesForP1(p1: string | number): ChoiceItem[] {
  const p1Str = String(p1);
  if (p1Str === '0') {
    return S_SUB_TYPE_CHOICES.filter((c) => c.value === 'NON_PROFIT');
  }
  return S_SUB_TYPE_CHOICES.filter((c) => c.value !== 'NON_PROFIT');
}
