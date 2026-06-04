// 中文注释:私权机构新增入口。普通私权只生成 SFID,教育委员会(JY)创建学校机构。
// 表单 UI 复用 common/institution,本文件只负责注入 private API。

import React from 'react';
import type { AdminAuth } from '../auth/types';
import { CreateInstitutionForm } from '../common/institution/CreateInstitutionForm';
import {
  checkInstitutionName,
  createInstitution,
  type CreateInstitutionOutput,
  type InstitutionCategory,
} from './api';

interface Props {
  auth: AdminAuth;
  category: InstitutionCategory;
  open: boolean;
  lockedProvince: string | null;
  lockedCity: string | null;
  onCancel: () => void;
  onCreated: (result: CreateInstitutionOutput) => void;
}

export const PrivateCreateModal: React.FC<Props> = (props) => (
  <CreateInstitutionForm
    {...props}
    checkInstitutionName={checkInstitutionName}
    createInstitution={createInstitution}
  />
);
