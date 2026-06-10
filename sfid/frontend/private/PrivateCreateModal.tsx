// 中文注释:私权机构新增入口,两步式只生成 SFID(机构代码 ZG/TG)。
// 教育委员会(JY)学校机构的新增统一在教育机构 tab(education 模块)。
// 表单 UI 复用 core/institution,本文件只负责注入 private API。

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

export const PrivateCreateModal: React.FC<Props> = (props) => (
  <CreateInstitutionForm
    {...props}
    category="PRIVATE_INSTITUTION"
    checkInstitutionName={checkInstitutionName}
    createInstitution={createInstitution}
    uploadLegalRepresentativePhoto={uploadLegalRepresentativePhoto}
  />
);
