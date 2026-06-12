// 中文注释:按表单入口(category)+ subject_property 决定"新增机构"弹窗里哪些字段锁定 + 默认值。
// private/gov/education 三个新增弹窗共用这一份字段锁定规则,组件分别放在各自业务目录。
//
// 手动新增三个入口(普通公权目录由后端自动生成,公安局不可手动建):
//   PRIVATE_INSTITUTION   私权 tab:S/F + ZG/TG,两步式(弹窗只生成 SFID,详情页补名称/sub_type)
//   GOV_INSTITUTION       公权 tab:G(ZF/LF/SF/JC,排除央行CB)机构名称必填 / F(锁中国ZG)挂公法人
//   EDUCATION_INSTITUTION 教育 tab:G/S/F + 机构锁死教育委员会(JY),学校名称弹窗内必填
//
// P1 盈利属性统一按主体属性联动(见 p1LocksForSubject,与后端号码生成器/uninorg 同源):
//   G → 锁死 0(非盈利,生成器硬规则)
//   S → 可选 0/1,默认 1
//   F → 锁死,继承所属法人(公法人父恒 0;私法人父继承其 p1;未选父级前置空)
//
// 非法人(F)统一流程:创建时必选"所属法人"(真实父级,创建即挂),搜索范围由后端按
// subjects/uninorg 地域规则预过滤(分校→本市学校本部;公权→市级同市/省级同省/国家级;私权→全国)。

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

// ── 私法人/非法人可选机构代码(教育委员会 JY 已统一收口教育机构 tab) ──
const PRIVATE_INSTITUTIONS: ChoiceItem[] = [
  { value: 'ZG', label: '中国' },
  { value: 'TG', label: '他国' },
];

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
      // 两步式:第一步弹窗不含 institution_name/sub_type
      return {
        subjectPropertyChoices: [
          { value: 'S', label: '私法人' },
          { value: 'F', label: '非法人' },
        ],
        institutionChoices: PRIVATE_INSTITUTIONS,
        modalTitle: '新增私权机构',
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
  return PRIVATE_INSTITUTIONS;
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
