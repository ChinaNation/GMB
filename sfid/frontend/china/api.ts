// 中文注释:SFID 元数据 API。这里承接省份、城市、SubjectProperty/机构类型等跨页面选择项。

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
  subject_property_options: SfidOptionItem[];
  institution_options: SfidOptionItem[];
  provinces: SfidProvinceItem[];
  scoped_province_name?: string | null;
};

export async function getSfidMeta(auth: AdminAuth): Promise<SfidMetaResult> {
  return request<SfidMetaResult>('/api/v1/admin/number/meta', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

export async function listSfidCities(auth: AdminAuth, province_name: string): Promise<SfidCityItem[]> {
  const q = `?province_name=${encodeURIComponent(province_name)}`;
  return request<SfidCityItem[]>(`/api/v1/admin/china/cities${q}`, {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}
