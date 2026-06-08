// 中文注释:公权机构列表。公安局和普通公权机构都是确定性目录,进入页面直接显示。

import React, { useEffect, useMemo, useState } from 'react';
import { Button, Space, Table, Tag, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import {
  listOfficialInstitutions,
  listPublicSecurityInstitutions,
  type GovCategory,
} from './api';
import type { AdminAuth } from '../auth/types';
import type { InstitutionListRow } from '../subjects/api';
import { INSTITUTION_CODE_LABEL, ORG_CODE_LABEL } from '../subjects/labels';
import {
  officialInstitutionCacheKey,
  publicSecurityCacheKey,
  readCachedOfficialInstitutionRows,
  readCachedPublicSecurityRows,
  writeCachedOfficialInstitutionRows,
  writeCachedPublicSecurityRows,
} from '../china/metaCache';
import { notice } from '../utils/notice';

const DETERMINISTIC_PAGE_SIZE = 20;

const SUBJECT_STATUS_LABEL: Record<string, string> = {
  ACTIVE: '正常',
  REVOKED: '已注销',
};

const CPMS_STATUS_LABEL: Record<string, string> = {
  PENDING: '待安装',
  ACTIVE: '已启用',
  DISABLED: '已禁用',
  REVOKED: '已吊销',
};

const INSTALL_TOKEN_STATUS_LABEL: Record<string, string> = {
  PENDING: '待使用',
  USED: '已使用',
  REVOKED: '已吊销',
};

const IDENTITY_SERVICE_STATUS_LABEL: Record<string, string> = {
  WAITING_INSTALL: '待安装',
  WAITING_IDENTITY_CODE: '待绑定身份码',
  READY: '可办理',
  DISABLED: '已禁用',
  REVOKED: '已吊销',
};

function areaText(row: InstitutionListRow) {
  return [row.province, row.city, row.town].filter(Boolean).join('/') || '-';
}

function nameText(row: InstitutionListRow) {
  return row.institution_name || row.short_name || row.sfid_name || '';
}

function statusTag(status: string | null | undefined, labels: Record<string, string>) {
  if (!status) return <Tag>未生成</Tag>;
  const color = status === 'ACTIVE' || status === 'READY' || status === 'USED' ? 'green'
    : status === 'PENDING' || status.startsWith('WAITING') ? 'orange'
      : 'red';
  return <Tag color={color}>{labels[status] || status}</Tag>;
}

interface Props {
  auth: AdminAuth;
  category: GovCategory;
  province: string;
  /** 为空字符串时表示“该省全部市”,用于公安局省级总览。 */
  city: string;
  onSelectInstitution?: (sfidNumber: string) => void;
  refreshKey?: number;
  searchQuery?: string;
}

export const GovListTable: React.FC<Props> = ({
  auth,
  category,
  province,
  city,
  onSelectInstitution,
  refreshKey,
  searchQuery,
}) => {
  const [rows, setRows] = useState<InstitutionListRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [deterministicPage, setDeterministicPage] = useState(1);
  const isPublicSecurity = category === 'PUBLIC_SECURITY';
  const cacheKey = isPublicSecurity
    ? publicSecurityCacheKey(auth, province, city)
    : officialInstitutionCacheKey(auth, province, city);

  const loadRows = () => {
    const exactQuery = searchQuery?.trim() ?? '';
    let hasImmediateRows = false;
    if (!exactQuery) {
      const cachedRows = isPublicSecurity
        ? readCachedPublicSecurityRows(cacheKey)
        : readCachedOfficialInstitutionRows(cacheKey);
      if (cachedRows) {
        setRows(cachedRows);
        setDeterministicPage(1);
        hasImmediateRows = true;
      }
    }

    let cancelled = false;
    if (!hasImmediateRows) setLoading(true);
    const request = isPublicSecurity
      ? listPublicSecurityInstitutions(auth, { page_size: 300 })
      : listOfficialInstitutions(auth, {
          province,
          city: city || undefined,
          q: exactQuery || undefined,
          page_size: 300,
        });

    request
      .then((data) => {
        if (cancelled) return;
        setRows(data.items);
        setDeterministicPage(1);
        if (!exactQuery) {
          if (isPublicSecurity) {
            writeCachedPublicSecurityRows(cacheKey, auth, province, city, data.items, data.manifest_version);
          } else {
            writeCachedOfficialInstitutionRows(cacheKey, auth, province, city, data.items, data.manifest_version);
          }
        }
      })
      .catch((err) => {
        if (!cancelled) notice.error(err, '');
      })
      .finally(() => {
        if (!cancelled && !hasImmediateRows) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  };

  useEffect(() => loadRows(), [
    auth.access_token,
    auth.admin_pubkey,
    auth.role,
    auth.admin_province,
    auth.admin_city,
    category,
    province,
    city,
    refreshKey,
    searchQuery,
  ]);

  const totalPages = Math.max(1, Math.ceil(rows.length / DETERMINISTIC_PAGE_SIZE));
  const displayRows = rows.slice(
    (deterministicPage - 1) * DETERMINISTIC_PAGE_SIZE,
    deterministicPage * DETERMINISTIC_PAGE_SIZE,
  );

  const columns = useMemo<ColumnsType<InstitutionListRow>>(() => {
    if (isPublicSecurity) {
      return [
        {
          title: '序号',
          width: 70,
          align: 'center',
          render: (_v, _row, index) =>
            (deterministicPage - 1) * DETERMINISTIC_PAGE_SIZE + index + 1,
        },
        { title: '身份ID', dataIndex: 'sfid_number', width: 260, align: 'center' },
        {
          title: '公安局名称',
          width: 180,
          align: 'center',
          render: (_v, row) => nameText(row) || <span style={{ color: '#999' }}>(未命名)</span>,
        },
        { title: '所属行政区', width: 180, align: 'center', render: (_v, row) => areaText(row) },
        {
          title: 'CPMS状态',
          dataIndex: 'cpms_status',
          width: 120,
          align: 'center',
          render: (v: string | null | undefined) => statusTag(v, CPMS_STATUS_LABEL),
        },
        {
          title: '安装码状态',
          dataIndex: 'install_token_status',
          width: 130,
          align: 'center',
          render: (v: string | null | undefined) => statusTag(v, INSTALL_TOKEN_STATUS_LABEL),
        },
        {
          title: '身份码业务状态',
          dataIndex: 'identity_service_status',
          width: 150,
          align: 'center',
          render: (v: string | null | undefined) => statusTag(v, IDENTITY_SERVICE_STATUS_LABEL),
        },
      ];
    }
    return [
      {
        title: '序号',
        width: 70,
        align: 'center',
        render: (_v, _row, index) =>
          (deterministicPage - 1) * DETERMINISTIC_PAGE_SIZE + index + 1,
      },
      { title: '身份ID', dataIndex: 'sfid_number', width: 260, align: 'center' },
      {
        title: '机构名称',
        width: 180,
        align: 'center',
        render: (_v, row) => nameText(row) || <span style={{ color: '#999' }}>(未命名)</span>,
      },
      { title: '行政区', width: 180, align: 'center', render: (_v, row) => areaText(row) },
      {
        title: '机构类型',
        dataIndex: 'institution_code',
        width: 130,
        align: 'center',
        render: (v: string, row) => {
          const base = INSTITUTION_CODE_LABEL[v] || v;
          const org = row.org_code ? (ORG_CODE_LABEL[row.org_code] || row.org_code) : '';
          return org ? `${base} / ${org}` : base;
        },
      },
      {
        title: '状态',
        dataIndex: 'status',
        width: 100,
        align: 'center',
        render: (v: string) => statusTag(v, SUBJECT_STATUS_LABEL),
      },
      { title: '账户数', dataIndex: 'account_count', width: 90, align: 'center' },
    ];
  }, [deterministicPage, isPublicSecurity]);

  return (
    <div>
      <Table<InstitutionListRow>
        rowKey={(r) => r.sfid_number}
        loading={loading}
        dataSource={displayRows}
        pagination={false}
        onRow={(row) => ({
          onClick: () => onSelectInstitution?.(row.sfid_number),
          style: onSelectInstitution ? { cursor: 'pointer' } : undefined,
        })}
        columns={columns}
      />
      <Space style={{ marginTop: 12 }} wrap>
        <Typography.Text type="secondary">
          共 {totalPages} 页 / 第 {deterministicPage} 页
        </Typography.Text>
        <Typography.Text type="secondary">共 {rows.length} 条</Typography.Text>
        <Button disabled={loading || deterministicPage <= 1} onClick={() => setDeterministicPage((p) => Math.max(1, p - 1))}>
          上一页
        </Button>
        <Button disabled={loading || deterministicPage >= totalPages} onClick={() => setDeterministicPage((p) => Math.min(totalPages, p + 1))}>
          下一页
        </Button>
      </Space>
    </div>
  );
};
