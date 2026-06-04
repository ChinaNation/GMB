// 中文注释:公权机构列表。公安局和普通公权机构都是确定性目录,进入页面直接显示。

import React, { useEffect, useMemo, useState } from 'react';
import { Button, message, Space, Table, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import {
  listOfficialInstitutions,
  listPublicSecurityInstitutions,
  type GovCategory,
} from './api';
import type { AdminAuth } from '../auth/types';
import type { InstitutionListRow } from '../subjects/api';
import {
  officialInstitutionCacheKey,
  publicSecurityCacheKey,
  readCachedOfficialInstitutionRows,
  readCachedPublicSecurityRows,
  writeCachedOfficialInstitutionRows,
  writeCachedPublicSecurityRows,
} from '../china/metaCache';

const DETERMINISTIC_PAGE_SIZE = 20;

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
            writeCachedPublicSecurityRows(cacheKey, auth, province, city, data.items);
          } else {
            writeCachedOfficialInstitutionRows(cacheKey, auth, province, city, data.items);
          }
        }
      })
      .catch((err) => {
        if (!cancelled) message.error(err instanceof Error ? err.message : '加载机构列表失败');
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

  const columns = useMemo<ColumnsType<InstitutionListRow>>(
    () => [
      {
        title: '序号',
        key: 'index',
        width: 80,
        align: 'center',
        render: (_v, _r, index) =>
          (deterministicPage - 1) * DETERMINISTIC_PAGE_SIZE + index + 1,
      },
      { title: '身份ID', dataIndex: 'sfid_number', width: 260, align: 'center' },
      {
        title: '机构名称',
        dataIndex: 'institution_name',
        width: 180,
        align: 'center',
        render: (v: string | null) =>
          v ? v : <span style={{ color: '#999' }}>(未命名,待完善)</span>,
      },
      { title: '省/市', render: (_v, r) => `${r.province}/${r.city}`, width: 160, align: 'center' },
      { title: '账户数', dataIndex: 'account_count', width: 90, align: 'center' },
    ],
    [deterministicPage],
  );

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
