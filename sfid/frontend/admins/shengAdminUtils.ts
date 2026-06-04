// 省级管理员视图共享工具函数与类型

import type { FormInstance } from 'antd';
import type { SfidCityItem } from '../china/api';
import type { OperatorRow } from './operators_api';
import type { ShengAdminRow } from './api';
import type { AdminActionType } from './admin_security_api';

export const MAX_SHI_ADMINS_PER_CITY = 30;

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
  refreshShengAdmins: () => Promise<ShengAdminRow[]>;
  selectedShengAdmin: ShengAdminRow | null;
  setSelectedShengAdmin: (v: ShengAdminRow | null) => void;
  /** 当前选中的市(三层导航:省→市→市级管理员列表) */
  selectedCity: string | null;
  setSelectedCity: (v: string | null) => void;
  adminDetailTab: 'operators' | 'sheng-admin';
  setAdminDetailTab: (v: 'operators' | 'sheng-admin') => void;

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
  onUpdateOperator: (row: OperatorRow) => void;
  onDeleteOperator: (row: OperatorRow) => void;
  runSecuredAction: <T = unknown>(actionType: AdminActionType, payload: unknown) => Promise<T>;
}
