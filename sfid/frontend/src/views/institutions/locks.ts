// 中文注释:按 category 决定 "新增机构" 弹窗里哪些字段锁定 + 默认值。
// 三个 tab 共用一个 CreateInstitutionModal,只通过 lockProfile 切形态。

import type { InstitutionCategory } from '../../api/institution';

export interface InstitutionFieldLocks {
  /** a3 的候选列表;长度=1 时锁死第一项 */
  a3Choices: Array<{ value: string; label: string }>;
  /** p1 的候选列表;长度=1 时锁死第一项 */
  p1Choices: Array<{ value: string; label: string }>;
  /** institution 代码的候选列表;长度=1 时锁死第一项 */
  institutionChoices: Array<{ value: string; label: string }>;
  /** 机构名称是否锁死为固定值 */
  lockedInstitutionName: string | null;
  /** 弹窗标题 */
  modalTitle: string;
}

const GFR_NONPROFIT_GOV: InstitutionFieldLocks['institutionChoices'] = [
  { value: 'ZF', label: '政府 (ZF)' },
  { value: 'LF', label: '立法院 (LF)' },
  { value: 'SF', label: '司法院 (SF)' },
  { value: 'JC', label: '监察院 (JC)' },
  { value: 'JY', label: '教育委员会 (JY)' },
  { value: 'CB', label: '储备委员会 (CB)' },
];

const SFR_FFR_INSTITUTIONS: InstitutionFieldLocks['institutionChoices'] = [
  { value: 'ZG', label: '中国 (ZG)' },
  { value: 'CH', label: '储备银行 (CH)' },
  { value: 'TG', label: '他国 (TG)' },
];

export function locksForCategory(category: InstitutionCategory): InstitutionFieldLocks {
  switch (category) {
    case 'PUBLIC_SECURITY':
      // 公安局:全部锁死
      return {
        a3Choices: [{ value: 'GFR', label: '公法人 (GFR)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: [{ value: 'ZF', label: '政府 (ZF)' }],
        lockedInstitutionName: '公民安全局',
        modalTitle: '新增公安局',
      };
    case 'GOV_INSTITUTION':
      // 公权机构:a3/p1 锁死,institution 可选(非公安局的其他政府机构),机构名自由
      return {
        a3Choices: [{ value: 'GFR', label: '公法人 (GFR)' }],
        p1Choices: [{ value: '0', label: '非盈利 (0)' }],
        institutionChoices: GFR_NONPROFIT_GOV,
        lockedInstitutionName: null,
        modalTitle: '新增公权机构',
      };
    case 'PRIVATE_INSTITUTION':
      // 私权机构:a3 在 SFR/FFR 二选,p1 可选,institution 可选,机构名自由
      return {
        a3Choices: [
          { value: 'SFR', label: '私法人 (SFR)' },
          { value: 'FFR', label: '非法人 (FFR)' },
        ],
        p1Choices: [
          { value: '0', label: '非盈利 (0)' },
          { value: '1', label: '盈利 (1)' },
        ],
        institutionChoices: SFR_FFR_INSTITUTIONS,
        lockedInstitutionName: null,
        modalTitle: '新增私权机构',
      };
  }
}
