// 省级管理员视图共享工具函数与类型

import type { FormInstance } from 'antd';
import type { SfidCityItem } from '../sfid/api';
import type { OperatorRow } from '../shi_admins/api';
import type { ShengAdminRow } from './api';

/** 比较两个 hex 公钥是否相同(忽略大小写和 0x 前缀) */
export function sameHexPubkey(a: string | null | undefined, b: string | null | undefined): boolean {
  if (!a || !b) return false;
  return a.trim().replace(/^0x/i, '').toLowerCase() === b.trim().replace(/^0x/i, '').toLowerCase();
}

/** 扫码目标类型 */
export type AccountScanTarget = null | 'operator';

/** 所有子视图共享的状态与回调 */
export interface ShengAdminSharedState {
  shengAdmins: ShengAdminRow[];
  shengAdminsLoading: boolean;
  selectedShengAdmin: ShengAdminRow | null;
  setSelectedShengAdmin: (v: ShengAdminRow | null) => void;
  /** 当前选中的市(三层导航:省→市→市级管理员列表) */
  selectedCity: string | null;
  setSelectedCity: (v: string | null) => void;
  adminDetailTab: 'operators' | 'super-admin';
  setAdminDetailTab: (v: 'operators' | 'super-admin') => void;

  operators: OperatorRow[];
  operatorsLoading: boolean;
  operatorListPage: number;
  setOperatorListPage: (v: number) => void;

  operatorCities: SfidCityItem[];
  operatorCitiesLoading: boolean;

  addOperatorOpen: boolean;
  setAddOperatorOpen: (v: boolean) => void;
  addOperatorLoading: boolean;

  accountScanTarget: AccountScanTarget;
  setAccountScanTarget: (v: AccountScanTarget) => void;

  addOperatorForm: FormInstance<{ operator_pubkey: string; operator_name: string; operator_city: string }>;

  onCreateOperator: (values: { operator_pubkey: string; operator_name: string; city?: string; created_by?: string }) => Promise<void>;
  onToggleOperatorStatus: (row: OperatorRow) => Promise<void>;
  onUpdateOperator: (row: OperatorRow) => void;
  onDeleteOperator: (row: OperatorRow) => void;
}
