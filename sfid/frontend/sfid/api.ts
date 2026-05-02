// 中文注释:SFID 元数据 API。这里承接省份、城市、A3/机构类型等跨页面选择项。

import { adminHeaders, request } from '../utils/http';
import type { AdminAuth } from '../auth/types';

export type SfidOptionItem = {
  label: string;
  value: string;
};

export type SfidProvinceItem = {
  name: string;
  code: string;
};

export type SfidCityItem = {
  name: string;
  code: string;
};

export type SfidMetaResult = {
  a3_options: SfidOptionItem[];
  institution_options: SfidOptionItem[];
  provinces: SfidProvinceItem[];
  scoped_province?: string | null;
};

export async function getSfidMeta(auth: AdminAuth): Promise<SfidMetaResult> {
  return request<SfidMetaResult>('/api/v1/admin/sfid/meta', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

export async function listSfidCities(auth: AdminAuth, province: string): Promise<SfidCityItem[]> {
  const q = `?province=${encodeURIComponent(province)}`;
  return request<SfidCityItem[]>(`/api/v1/admin/sfid/cities${q}`, {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}
