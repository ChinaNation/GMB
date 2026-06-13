// 联邦管理员视图共享工具函数与类型

import type { FormInstance } from 'antd';
import type { SfidCityItem } from '../china/api';
import type { CityAdminRow } from './city_admins_api';
import type { FederalAdminRow } from './api';
import type { AdminActionType } from './admin_security_api';
import type { InstitutionDetail } from '../subjects/api';

export const MAX_CITY_ADMINS_PER_CITY = 30;

/** 比较两个 hex 公钥是否相同(忽略大小写和 0x 前缀) */
export function sameHexPubkey(a: string | null | undefined, b: string | null | undefined): boolean {
  if (!a || !b) return false;
  return a.trim().replace(/^0x/i, '').toLowerCase() === b.trim().replace(/^0x/i, '').toLowerCase();
}

/** 扫码目标类型 */
export type AccountScanTarget = null | 'city_admin';

/** 所有子视图共享的状态与回调 */
export interface FederalAdminSharedState {
  federalAdmins: FederalAdminRow[];
  federalAdminsLoading: boolean;
  refreshFederalAdmins: () => Promise<FederalAdminRow[]>;
  selectedFederalAdmin: FederalAdminRow | null;
  setSelectedFederalAdmin: (v: FederalAdminRow | null) => void;
  /** 当前选中的市(三层导航:省→市→市管理员列表) */
  selectedCity: string | null;
  setSelectedCity: (v: string | null) => void;
  adminDetailTab: 'city-admin' | 'federal-admin';
  setAdminDetailTab: (v: 'city-admin' | 'federal-admin') => void;

  cityAdmins: CityAdminRow[];
  cityAdminsLoading: boolean;
  cityAdminListPage: number;
  setCityAdminListPage: (v: number) => void;

  cityAdminCities: SfidCityItem[];
  cityAdminCitiesLoading: boolean;

  addCityAdminOpen: boolean;
  setAddCityAdminOpen: (v: boolean) => void;
  addCityAdminLoading: boolean;

  accountScanTarget: AccountScanTarget;
  setAccountScanTarget: (v: AccountScanTarget) => void;

  addCityAdminForm: FormInstance<{ city_admin_pubkey: string; city_admin_name: string; city_admin_city: string }>;

  onCreateCityAdmin: (values: { city_admin_pubkey: string; city_admin_name: string; city?: string; created_by?: string }) => Promise<void>;
  onUpdateCityAdmin: (row: CityAdminRow) => void;
  onDeleteCityAdmin: (row: CityAdminRow) => void;
  runSecuredAction: <T = unknown>(actionType: AdminActionType, payload: unknown) => Promise<T>;

  /** 联邦注册局机构详情(全国唯一,scope-bypass 接口加载)。用于「联邦注册局」子 tab 详情页。 */
  federalRegistryDetail: InstitutionDetail | null;
  federalRegistryLoading: boolean;
  /** 当前活动市(selectedCity / 市管理员 lockedCity)对应的市注册局机构 sfid_number。 */
  cityRegistrySfid: string | null;
  cityRegistryLoading: boolean;
}
