// 中文注释:前端对齐后端 scope::rules::VisibleScope,三角色范围派生。
// 铁律:feedback_scope_auto_filter.md(KEY=全国 / SHENG=本省 / SHI=本市)
//
// 任务卡 4 的 views/ 子组件在进入 tab 时用本 hook 做三种分流:
//   - KeyAdmin:  显示 43 省卡片
//   - ShengAdmin: skipProvinceList=true,直接进本省的市列表
//   - ShiAdmin:   skipCityList=true,直接进本市的详情页

import { useMemo } from 'react';
import type { AdminAuth } from '../api/client';

export interface VisibleScope {
  /** 可见省份列表。空数组 = "全国"(仅 KeyAdmin)。 */
  provinces: string[];
  /** 可见市列表。空数组 = "不限市"(KeyAdmin + ShengAdmin)。 */
  cities: string[];
  /** 是否可以增删改。 */
  canWrite: boolean;
  /** 进 tab 时跳过省列表直接进入省详情(ShengAdmin + ShiAdmin)。 */
  skipProvinceList: boolean;
  /** 进 tab 时跳过市列表直接进入市详情(仅 ShiAdmin)。 */
  skipCityList: boolean;
  /** 锁定的省份(非 KeyAdmin 必填)。 */
  lockedProvince: string | null;
  /** 锁定的市(仅 ShiAdmin 必填)。 */
  lockedCity: string | null;

  /** 判断某省是否在范围内。 */
  includesProvince(province: string): boolean;
  /** 判断某市是否在范围内。 */
  includesCity(city: string): boolean;
}

function makeScope(base: Omit<VisibleScope, 'includesProvince' | 'includesCity'>): VisibleScope {
  return {
    ...base,
    includesProvince(province: string) {
      return base.provinces.length === 0 || base.provinces.includes(province);
    },
    includesCity(city: string) {
      return base.cities.length === 0 || base.cities.includes(city);
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
      case 'KEY_ADMIN':
        return makeScope({
          provinces: [],
          cities: [],
          canWrite: true,
          skipProvinceList: false,
          skipCityList: false,
          lockedProvince: null,
          lockedCity: null,
        });
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
