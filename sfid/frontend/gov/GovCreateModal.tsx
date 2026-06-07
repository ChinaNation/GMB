// 中文注释:公权机构手动新增入口只保留教育委员会(JY)学校机构。
// 表单 UI 复用 common/institution,本文件只负责注入 gov API。

import React from 'react';
import type { AdminAuth } from '../auth/types';
import { CreateInstitutionForm } from '../core/institution/CreateInstitutionForm';
import {
  checkInstitutionName,
  createInstitution,
  uploadLegalRepresentativePhoto,
  type CreateInstitutionOutput,
  type GovCategory,
} from './api';

interface Props {
  auth: AdminAuth;
  category: GovCategory;
  open: boolean;
  lockedProvince: string | null;
  lockedCity: string | null;
  onCancel: () => void;
  onCreated: (result: CreateInstitutionOutput) => void;
}

export const GovCreateModal: React.FC<Props> = (props) => (
  <CreateInstitutionForm
    {...props}
    checkInstitutionName={checkInstitutionName}
    createInstitution={createInstitution}
    uploadLegalRepresentativePhoto={uploadLegalRepresentativePhoto}
  />
);
