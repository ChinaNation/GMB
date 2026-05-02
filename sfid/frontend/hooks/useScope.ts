// 中文注释:前端对齐后端 scope::rules::VisibleScope,两角色范围派生(ADR-008 删 KEY_ADMIN)。
// 铁律:feedback_scope_auto_filter.md(SHENG=本省 / SHI=本市)
//
// sheng_admin / shi_admin 两个视图的 Dashboard 走"全局视图(43 省可看)+ 跨省按钮置灰":
//   - SHENG_ADMIN: skipProvinceList=true → 直接进本省的市列表,只读其他省
//   - SHI_ADMIN:   skipCityList=true     → 直接进本市的详情页,只读其他市

import { useMemo } from 'react';
import type { AdminAuth } from '../auth/types';

export interface VisibleScope {
  /** 可见省份列表。空数组保留含义"全国可见(只读)"——当前只用于未登录场景占位。 */
  provinces: string[];
  /** 可见市列表。空数组 = "不限市"(SHENG_ADMIN)。 */
  cities: string[];
  /** 是否可以增删改(本省/本市内才有写权限)。 */
  canWrite: boolean;
  /** 进 tab 时跳过省列表直接进入省详情(SHENG_ADMIN + SHI_ADMIN)。 */
  skipProvinceList: boolean;
  /** 进 tab 时跳过市列表直接进入市详情(仅 SHI_ADMIN)。 */
  skipCityList: boolean;
  /** 锁定的省份(必填)。 */
  lockedProvince: string | null;
  /** 锁定的市(仅 SHI_ADMIN 必填)。 */
  lockedCity: string | null;

  /** 判断某省是否在可见范围内。 */
  includesProvince(province: string): boolean;
  /** 判断某市是否在可见范围内。 */
  includesCity(city: string): boolean;
  /** 判断某省是否允许写操作(跨省一律置灰)。 */
  canWriteProvince(province: string): boolean;
  /** 判断某市是否允许写操作(跨市一律置灰,SHENG_ADMIN 本省内任意市可写)。 */
  canWriteCity(province: string, city: string): boolean;
}

function makeScope(base: Omit<VisibleScope, 'includesProvince' | 'includesCity' | 'canWriteProvince' | 'canWriteCity'>): VisibleScope {
  return {
    ...base,
    includesProvince(_province: string) {
      // ADR-008 起前端 Dashboard 全局视图:任意省都"可见",只是写权限按 lockedProvince 收紧。
      return true;
    },
    includesCity(_city: string) {
      return true;
    },
    canWriteProvince(province: string) {
      if (!base.canWrite) return false;
      if (!base.lockedProvince) return false;
      return province === base.lockedProvince;
    },
    canWriteCity(province: string, city: string) {
      if (!base.canWrite) return false;
      if (!base.lockedProvince || province !== base.lockedProvince) return false;
      // SHENG_ADMIN 本省内任意市;SHI_ADMIN 必须等于自己 lockedCity
      if (base.lockedCity && city !== base.lockedCity) return false;
      return true;
    },
  };
}

export function useScope(auth: AdminAuth | null): VisibleScope {
  return useMemo<VisibleScope>(() => {
    if (!auth) {
      // 未登录:零范围
      return makeScope({
        provinces: ['__NO_AUTH__'],
        cities: ['__NO_AUTH__'],
        canWrite: false,
        skipProvinceList: false,
        skipCityList: false,
        lockedProvince: null,
        lockedCity: null,
      });
    }
    switch (auth.role) {
      case 'SHENG_ADMIN': {
        const province = auth.admin_province || '__SHENG_ADMIN_MISSING_PROVINCE__';
        return makeScope({
          provinces: [province],
          cities: [],
          canWrite: true,
          skipProvinceList: true,
          skipCityList: false,
          lockedProvince: province,
          lockedCity: null,
        });
      }
      case 'SHI_ADMIN': {
        const province = auth.admin_province || '__SHI_ADMIN_MISSING_PROVINCE__';
        const city = auth.admin_city || '__SHI_ADMIN_MISSING_CITY__';
        return makeScope({
          provinces: [province],
          cities: [city],
          canWrite: true,
          skipProvinceList: true,
          skipCityList: true,
          lockedProvince: province,
          lockedCity: city,
        });
      }
      default:
        return makeScope({
          provinces: ['__UNKNOWN_ROLE__'],
          cities: ['__UNKNOWN_ROLE__'],
          canWrite: false,
          skipProvinceList: false,
          skipCityList: false,
          lockedProvince: null,
          lockedCity: null,
        });
    }
  }, [auth?.role, auth?.admin_province, auth?.admin_city]);
}
