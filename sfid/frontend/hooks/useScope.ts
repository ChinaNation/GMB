// 中文注释:前端对齐后端 scope::rules::VisibleScope,按注册局机构范围派生。
//
// federal_registry / city_registry 两个视图的 Dashboard 走"全局视图(43 省可看)+ 跨省按钮置灰":
//   - FEDERAL_REGISTRY: skipProvinceList=true → 直接进本省的市列表,只读其他省
//   - CITY_REGISTRY:   skipCityList=true     → 直接进本市的详情页,只读其他市

import { useMemo } from 'react';
import type { AdminAuth } from '../auth/types';

export interface VisibleScope {
  /** 可见省份列表。空数组保留含义"全国可见(只读)"——当前只用于未登录场景占位。 */
  provinces: string[];
  /** 可见市列表。空数组 = "不限市"(FEDERAL_REGISTRY)。 */
  cities: string[];
  /** 是否可以增删改(本省/本市内才有写权限)。 */
  canWrite: boolean;
  /** 进 tab 时跳过省列表直接进入省详情(FEDERAL_REGISTRY + CITY_REGISTRY)。 */
  skipProvinceList: boolean;
  /** 进 tab 时跳过市列表直接进入市详情(仅 CITY_REGISTRY)。 */
  skipCityList: boolean;
  /** 锁定的省名称(必填)。 */
  lockedProvinceName: string | null;
  /** 锁定的市名称(仅 CITY_REGISTRY 必填)。 */
  lockedCityName: string | null;

  /** 判断某省是否在可见范围内。 */
  includesProvince(province_name: string): boolean;
  /** 判断某市是否在可见范围内。 */
  includesCity(city_name: string): boolean;
  /** 判断某省是否允许写操作(跨省一律置灰)。 */
  canWriteProvince(province_name: string): boolean;
  /** 判断某市是否允许写操作(跨市一律置灰,FEDERAL_REGISTRY 本省内任意市可写)。 */
  canWriteCity(province_name: string, city_name: string): boolean;
}

function makeScope(base: Omit<VisibleScope, 'includesProvince' | 'includesCity' | 'canWriteProvince' | 'canWriteCity'>): VisibleScope {
  return {
    ...base,
    includesProvince(_province: string) {
      // ADR-008 起前端 Dashboard 全局视图:任意省都"可见",只是写权限按 lockedProvinceName 收紧。
      return true;
    },
    includesCity(_city: string) {
      return true;
    },
    canWriteProvince(province_name: string) {
      if (!base.canWrite) return false;
      if (!base.lockedProvinceName) return false;
      return province_name === base.lockedProvinceName;
    },
    canWriteCity(province_name: string, city_name: string) {
      if (!base.canWrite) return false;
      if (!base.lockedProvinceName || province_name !== base.lockedProvinceName) return false;
      // FEDERAL_REGISTRY 本省内任意市;CITY_REGISTRY 必须等于自己 lockedCityName
      if (base.lockedCityName && city_name !== base.lockedCityName) return false;
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
        lockedProvinceName: null,
        lockedCityName: null,
      });
    }
    switch (auth.registry_org_code) {
      case 'FEDERAL_REGISTRY': {
        const province_name = auth.scope_province_name || '__FEDERAL_REGISTRY_MISSING_PROVINCE__';
        return makeScope({
          provinces: [province_name],
          cities: [],
          canWrite: true,
          skipProvinceList: true,
          skipCityList: false,
          lockedProvinceName:  province_name,
          lockedCityName: null,
        });
      }
      case 'CITY_REGISTRY': {
        const province_name = auth.scope_province_name || '__CITY_REGISTRY_MISSING_PROVINCE__';
        const city_name = auth.scope_city_name || '__CITY_REGISTRY_MISSING_CITY__';
        return makeScope({
          provinces: [province_name],
          cities: [city_name],
          canWrite: true,
          skipProvinceList: true,
          skipCityList: true,
          lockedProvinceName:  province_name,
          lockedCityName:  city_name,
        });
      }
      default:
        return makeScope({
          provinces: ['__UNKNOWN_REGISTRY_ORG__'],
          cities: ['__UNKNOWN_REGISTRY_ORG__'],
          canWrite: false,
          skipProvinceList: false,
          skipCityList: false,
          lockedProvinceName: null,
          lockedCityName: null,
        });
    }
  }, [auth?.registry_org_code, auth?.scope_province_name, auth?.scope_city_name]);
}
