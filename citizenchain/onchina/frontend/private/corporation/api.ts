// 股份公司 API。调用 `/api/v1/private/corporation`,不得回退到旧聚合接口。

import type { AdminAuth } from '../../auth/types';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionListRow,
  PageResult,
  PrivateType,
} from '../../subjects/api';
import {
  createInstitution as createPrivateInstitution,
  listPrivateInstitutions,
} from '../common/api';
import type { ScanSignResolver } from '../../admins/securityApi';
import { CORPORATION_ROUTE_SEGMENT } from './types';

export function createCorporationInstitution(
  auth: AdminAuth,
  input: CreateInstitutionInput,
  signWithScan: ScanSignResolver,
): Promise<CreateInstitutionOutput> {
  return createPrivateInstitution(auth, CORPORATION_ROUTE_SEGMENT, input, signWithScan);
}

export function listCorporationInstitutions(
  auth: AdminAuth,
  query: {
    province_name: string;
    city_name?: string;
    private_type: PrivateType;
    q: string;
    cursor?: string | null;
    page_size?: number;
  },
): Promise<PageResult<InstitutionListRow>> {
  return listPrivateInstitutions(auth, CORPORATION_ROUTE_SEGMENT, query);
}
