// 省级管理员视图共享工具函数与类型

import type { FormInstance } from 'antd';
import type { OperatorRow, ShengAdminRow, SfidCityItem } from '../api/client';

/** 检查是否为合法 Sr25519 hex 公钥(32 字节十六进制) */
export function isSr25519HexPubkey(value: string): boolean {
  const normalized = value.trim().replace(/^0x/i, '');
  return /^[0-9a-fA-F]{64}$/.test(normalized);
}

/** 比较两个 hex 公钥是否相同(忽略大小写和 0x 前缀) */
export function sameHexPubkey(a: string | null | undefined, b: string | null | undefined): boolean {
  if (!a || !b) return false;
  return a.trim().replace(/^0x/i, '').toLowerCase() === b.trim().replace(/^0x/i, '').toLowerCase();
}

/** 扫码目标类型 */
export type AccountScanTarget = null | 'operator' | 'super-admin';

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
  replaceSuperLoading: boolean;

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
  replaceSuperForm: FormInstance<{ province: string; admin_name: string; admin_pubkey: string }>;

  onReplaceShengAdmin: (values: { province: string; admin_name?: string; admin_pubkey: string }) => Promise<void>;
  onCreateOperator: (values: { operator_pubkey: string; operator_name: string; city?: string; created_by?: string }) => Promise<void>;
  onToggleOperatorStatus: (row: OperatorRow) => Promise<void>;
  onUpdateOperator: (row: OperatorRow) => void;
  onDeleteOperator: (row: OperatorRow) => void;
}
