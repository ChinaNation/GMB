// 中文注释:CID 元数据 API。这里承接省份、城市、SubjectProperty/机构类型等跨页面选择项。

import { adminHeaders, request } from '../utils/http';
import type { AdminAuth } from '../auth/types';

export type CidOptionItem = {
  label: string;
  value: string;
};

export type CidProvinceItem = {
  name: string;
  code: string;
};

export type CidCityItem = {
  name: string;
  code: string;
};

export type CidMetaResult = {
  subject_property_options: CidOptionItem[];
  institution_options: CidOptionItem[];
  provinces: CidProvinceItem[];
  scoped_province_name?: string | null;
};

export async function getCidMeta(auth: AdminAuth): Promise<CidMetaResult> {
  return request<CidMetaResult>('/api/v1/admin/number/meta', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

export async function listCidCities(auth: AdminAuth, province_name: string): Promise<CidCityItem[]> {
  const q = `?province_name=${encodeURIComponent(province_name)}`;
  return request<CidCityItem[]>(`/api/v1/admin/china/cities${q}`, {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}
