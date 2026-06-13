// 中文注释:注册协会前端类型边界。固定 `S + AS`,具有法人资格。

import type { PrivateType } from '../../subjects/api';

export const ASSOCIATION_PRIVATE_TYPE: PrivateType = 'ASSOCIATION';
export const ASSOCIATION_ROUTE_SEGMENT = 'association';
export const ASSOCIATION_TITLE = '注册协会';

export interface AssociationProfileFields {
  identityCode: 'AS';
  p1: '0';
  memberRole: 'MEMBER';
  hasLegalPersonality: true;
}
