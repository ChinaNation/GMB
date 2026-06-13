// 中文注释:单一私权类型页面壳。六类业务页面只把自己的 API 和类型配置传进来,
// 这里负责同一种页面结构:搜索、列表、新增、进入详情。

import React, { useState } from 'react';
import { Button, Input } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import type { AdminAuth } from '../../auth/types';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionListRow,
  PageResult,
  PrivateType,
} from '../../subjects/api';
import { PRIVATE_TYPE_LABEL } from '../../subjects/labels';
import { PrivateCreateModal } from './PrivateCreateModal';
import { PrivateListTable } from './PrivateListTable';

export interface PrivateTypePageProps {
  auth: AdminAuth;
  province: string;
  city: string;
  canWrite: boolean;
  privateType: PrivateType;
  title: string;
  createInstitution: (
    auth: AdminAuth,
    input: CreateInstitutionInput,
  ) => Promise<CreateInstitutionOutput>;
  listInstitutions: (auth: AdminAuth, query: {
    province: string;
    city?: string;
    private_type: PrivateType;
    q: string;
    cursor?: string | null;
    page_size?: number;
  }) => Promise<PageResult<InstitutionListRow>>;
  onSelectInstitution: (sfidNumber: string) => void;
}

export const PrivateTypePage: React.FC<PrivateTypePageProps> = ({
  auth,
  province,
  city,
  canWrite,
  privateType,
  title,
  createInstitution,
  listInstitutions,
  onSelectInstitution,
}) => {
  const [createOpen, setCreateOpen] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const [searchInput, setSearchInput] = useState('');
  const [committedSearch, setCommittedSearch] = useState('');

  const onSubmitSearch = () => {
    setCommittedSearch(searchInput.trim());
  };

  return (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, marginBottom: 12 }}>
        <strong>{title}</strong>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0 }}>
          <Input
            value={searchInput}
            placeholder="请输入机构名称、身份ID"
            allowClear
            style={{ width: 360, maxWidth: '42vw' }}
            onChange={(e) => {
              const v = e.target.value;
              setSearchInput(v);
              if (!v) setCommittedSearch('');
            }}
            onPressEnter={onSubmitSearch}
            suffix={
              <span
                style={{ cursor: 'pointer', color: '#1890ff' }}
                onClick={onSubmitSearch}
                title="搜索"
              >
                <SearchOutlined />
              </span>
            }
          />
          {canWrite && (
            <Button type="primary" onClick={() => setCreateOpen(true)}>
              + 新增{PRIVATE_TYPE_LABEL[privateType]}
            </Button>
          )}
        </div>
      </div>

      <PrivateListTable
        auth={auth}
        province={province}
        city={city}
        privateType={privateType}
        listInstitutions={listInstitutions}
        refreshKey={refreshKey}
        searchQuery={committedSearch}
        onSelectInstitution={onSelectInstitution}
      />

      <PrivateCreateModal
        auth={auth}
        open={createOpen}
        lockedProvince={province}
        lockedCity={city}
        privateType={privateType}
        createInstitution={createInstitution}
        onCancel={() => setCreateOpen(false)}
        onCreated={(result) => {
          setCreateOpen(false);
          setRefreshKey((k) => k + 1);
          if (result?.sfid_number) onSelectInstitution(result.sfid_number);
        }}
      />
    </>
  );
};
