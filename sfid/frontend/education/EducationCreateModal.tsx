// 中文注释:教育机构新增入口。机构锁死教育委员会(JY),主体属性 G(公立)/S(私立)/F(分校)。
// 表单 UI 复用 core/institution,本文件只负责注入 education API。

import React from 'react';
import type { AdminAuth } from '../auth/types';
import { CreateInstitutionForm } from '../core/institution/CreateInstitutionForm';
import {
  checkInstitutionName,
  createInstitution,
  uploadLegalRepresentativePhoto,
  type CreateInstitutionOutput,
} from './api';

interface Props {
  auth: AdminAuth;
  open: boolean;
  lockedProvince: string | null;
  lockedCity: string | null;
  onCancel: () => void;
  onCreated: (result: CreateInstitutionOutput) => void;
}

export const EducationCreateModal: React.FC<Props> = (props) => (
  <CreateInstitutionForm
    {...props}
    category="EDUCATION_INSTITUTION"
    checkInstitutionName={checkInstitutionName}
    createInstitution={createInstitution}
    uploadLegalRepresentativePhoto={uploadLegalRepresentativePhoto}
  />
);
