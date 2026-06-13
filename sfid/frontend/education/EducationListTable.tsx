// 中文注释:教育机构列表。与私权一致的精确搜索形态:必须输入学校名称或 SFID,避免跨省全量扫描。

import React, { useEffect, useMemo, useState } from 'react';
import { Button, Space, Table, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import type { AdminAuth } from '../auth/types';
import type { InstitutionListRow } from './api';
import { listEducationInstitutions } from './api';
import { SUBJECT_PROPERTY_LABEL } from '../subjects/labels';
import { notice } from '../utils/notice';

const CREATED_BY_ROLE_LABEL: Record<string, string> = {
  FEDERAL_ADMIN: '联邦管理员',
  CITY_ADMIN: '市管理员',
};

interface Props {
  auth: AdminAuth;
  province: string;
  city: string;
  onSelectInstitution?: (sfidNumber: string) => void;
  refreshKey?: number;
  searchQuery?: string;
}

export const EducationListTable: React.FC<Props> = ({
  auth,
  province,
  city,
  onSelectInstitution,
  refreshKey,
  searchQuery,
}) => {
  const [rows, setRows] = useState<InstitutionListRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [cursorStack, setCursorStack] = useState<string[]>([]);

  const loadRows = (cursor?: string | null) => {
    const exactQuery = searchQuery?.trim() ?? '';
    if (!exactQuery) {
      setRows([]);
      setNextCursor(null);
      return () => {};
    }

    let cancelled = false;
    setLoading(true);
    listEducationInstitutions(auth, {
      province,
      city: city || undefined,
      q: exactQuery,
      cursor,
      page_size: 50,
    })
      .then((data) => {
        if (cancelled) return;
        setRows(data.items);
        setNextCursor(data.next_cursor ?? null);
      })
      .catch((err) => {
        if (!cancelled) notice.error(err, '');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  };

  useEffect(() => {
    setCursorStack([]);
    return loadRows(null);
  }, [
    auth.access_token,
    auth.admin_pubkey,
    auth.role,
    auth.admin_province,
    auth.admin_city,
    province,
    city,
    refreshKey,
    searchQuery,
  ]);

  const columns = useMemo<ColumnsType<InstitutionListRow>>(
    () => [
      {
        title: '序号',
        width: 70,
        align: 'center',
        render: (_v, _row, index) => cursorStack.length * 50 + index + 1,
      },
      { title: '身份ID', dataIndex: 'sfid_number', width: 260, align: 'center' },
      {
        title: '学校名称',
        dataIndex: 'institution_name',
        width: 180,
        align: 'center',
        render: (v: string | null) =>
          v ? v : <span style={{ color: '#999' }}>(未命名,待完善)</span>,
      },
      {
        title: '主体属性',
        key: 'subject_property',
        width: 140,
        align: 'center',
        render: (_v, r) => (
          <span>
            {SUBJECT_PROPERTY_LABEL[r.subject_property] ?? r.subject_property}
            <Typography.Text type="secondary" style={{ marginLeft: 6, fontSize: 12 }}>
              ({r.p1 === '1' ? '盈利' : '非盈利'})
            </Typography.Text>
          </span>
        ),
      },
      { title: '省/市', render: (_v, r) => `${r.province}/${r.city}`, width: 160, align: 'center' },
      { title: '账户数', dataIndex: 'account_count', width: 90, align: 'center' },
      {
        title: '创建用户',
        key: 'created_by',
        width: 180,
        align: 'center',
        render: (_v, r) => {
          const roleLabel = r.created_by_role
            ? CREATED_BY_ROLE_LABEL[r.created_by_role] ?? r.created_by_role
            : '';
          if (r.created_by_name) {
            return (
              <span>
                {r.created_by_name}
                {roleLabel && (
                  <Typography.Text type="secondary" style={{ marginLeft: 6, fontSize: 12 }}>
                    ({roleLabel})
                  </Typography.Text>
                )}
              </span>
            );
          }
          if (roleLabel) return <span>{roleLabel}</span>;
          return <span style={{ color: '#999' }}>未知</span>;
        },
      },
    ],
    [cursorStack.length],
  );

  const onNextPage = () => {
    if (!nextCursor) return;
    setCursorStack((prev) => [...prev, nextCursor]);
    loadRows(nextCursor);
  };

  const onPrevPage = () => {
    if (cursorStack.length === 0) return;
    const stack = [...cursorStack];
    stack.pop();
    const prevCursor = stack.length > 0 ? stack[stack.length - 1] : null;
    setCursorStack(stack);
    loadRows(prevCursor);
  };

  return (
    <div>
      <Table<InstitutionListRow>
        rowKey={(r) => r.sfid_number}
        loading={loading}
        dataSource={rows}
        pagination={false}
        onRow={(row) => ({
          onClick: () => onSelectInstitution?.(row.sfid_number),
          style: onSelectInstitution ? { cursor: 'pointer' } : undefined,
        })}
        columns={columns}
      />
      <Space style={{ marginTop: 12 }} wrap>
        <Button disabled={loading || cursorStack.length === 0} onClick={onPrevPage}>
          上一页
        </Button>
        <Button disabled={loading || !nextCursor} onClick={onNextPage}>
          下一页
        </Button>
      </Space>
    </div>
  );
};
