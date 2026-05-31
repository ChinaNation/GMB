// 中文注释:市详情页的机构表格。
// 走任务卡 2 新 API listInstitutions,按 category + province + city 过滤。
// 整行可点击 → 进机构详情页。不显示"操作"列。

import React, { useEffect, useMemo, useState } from 'react';
import { Button, message, Space, Table, Tag, Typography } from 'antd';
import { ReloadOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import {
  listInstitutions,
  listPublicSecurityInstitutions,
  type InstitutionCategory,
  type InstitutionListRow,
} from './api';
import type { AdminAuth } from '../auth/types';
import {
  isSelfEligibleClearingBank,
  CLEARING_BANK_ELIGIBLE_LABEL,
} from './utils/clearingBankEligible';
import {
  clearCachedPublicSecurityRows,
  publicSecurityCacheKey,
  readCachedPublicSecurityRows,
  writeCachedPublicSecurityRows,
} from '../sfid/metaCache';

// 创建者角色中文映射。
const CREATED_BY_ROLE_LABEL: Record<string, string> = {
  SHENG_ADMIN: '省级管理员',
  SHI_ADMIN: '市级管理员',
};

const PUBLIC_SECURITY_PAGE_SIZE = 20;

interface Props {
  auth: AdminAuth;
  category: InstitutionCategory;
  province: string;
  /** 为空字符串时表示"该省全部市",用于公安局省级总览 */
  city: string;
  /** 点击某行机构时回调(跳机构详情页) */
  onSelectInstitution?: (sfidNumber: string) => void;
  /** 让父组件触发刷新 */
  refreshKey?: number;
  /** 精确搜索关键字(机构名称或 SFID);空=返回空页。scope 由后端按角色自动限制 */
  searchQuery?: string;
}

export const InstitutionListTable: React.FC<Props> = ({
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
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [cursorStack, setCursorStack] = useState<string[]>([]);
  const [publicSecurityPage, setPublicSecurityPage] = useState(1);
  const isPublicSecurity = category === 'PUBLIC_SECURITY';
  const psCacheKey = isPublicSecurity ? publicSecurityCacheKey(auth, province, city) : '';

  const loadRows = (cursor?: string | null, forceRefresh = false) => {
    const exactQuery = searchQuery?.trim() ?? '';
    if (!isPublicSecurity && !exactQuery) {
      setRows([]);
      setNextCursor(null);
      return () => {};
    }
    if (isPublicSecurity && !cursor && !forceRefresh) {
      const cachedRows = readCachedPublicSecurityRows(psCacheKey);
      if (cachedRows) {
        setRows(cachedRows);
        setNextCursor(null);
        setPublicSecurityPage(1);
        return () => {};
      }
    }
    let cancelled = false;
    setLoading(true);
    const request = isPublicSecurity
      ? listPublicSecurityInstitutions(auth, { cursor, page_size: 300 })
      : listInstitutions(auth, {
          category,
          province,
          city: city || undefined,
          q: exactQuery,
          cursor,
          page_size: 50,
        });
    request
      .then((data) => {
        if (!cancelled) {
          setRows(data.items);
          setNextCursor(data.next_cursor ?? null);
          if (isPublicSecurity && !cursor) {
            setPublicSecurityPage(1);
            writeCachedPublicSecurityRows(psCacheKey, auth, province, city, data.items);
          }
        }
      })
      .catch((err) => {
        if (!cancelled) message.error(err instanceof Error ? err.message : '加载机构列表失败');
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
    setPublicSecurityPage(1);
    return loadRows(null);
  }, [
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

  const onRefreshPublicSecurity = () => {
    if (!isPublicSecurity) return;
    clearCachedPublicSecurityRows(psCacheKey);
    setCursorStack([]);
    setPublicSecurityPage(1);
    loadRows(null, true);
  };

  const publicSecurityTotalPages = Math.max(
    1,
    Math.ceil(rows.length / PUBLIC_SECURITY_PAGE_SIZE),
  );
  const publicSecurityDisplayRows = isPublicSecurity
    ? rows.slice(
        (publicSecurityPage - 1) * PUBLIC_SECURITY_PAGE_SIZE,
        publicSecurityPage * PUBLIC_SECURITY_PAGE_SIZE,
      )
    : rows;
  const onPrevPublicSecurityPage = () => {
    setPublicSecurityPage((page) => Math.max(1, page - 1));
  };
  const onNextPublicSecurityPage = () => {
    setPublicSecurityPage((page) => Math.min(publicSecurityTotalPages, page + 1));
  };
  const prevDisabled = isPublicSecurity
    ? loading || publicSecurityPage <= 1
    : loading || cursorStack.length === 0;
  const nextDisabled = isPublicSecurity
    ? loading || publicSecurityPage >= publicSecurityTotalPages
    : loading || !nextCursor;

  // 中文注释:"创建用户"列仅对**私权机构**展示。
  // 公安局由后端 reconcile 批量生成,created_by 不具人类语义;
  // 公权机构本轮未做两步式改造,暂不展示(下一步再加)。
  const showCreatedByColumn = category === 'PRIVATE_INSTITUTION';
  const showClearingEligibleColumn = category === 'PRIVATE_INSTITUTION';

  const columns = useMemo<ColumnsType<InstitutionListRow>>(() => {
    const cols: ColumnsType<InstitutionListRow> = [
      { title: 'SFID', dataIndex: 'sfid_number', width: 260 },
      {
        title: '机构名称',
        dataIndex: 'institution_name',
        width: 180,
        // 两步式:第一步生成 SFID 后、详情页补填前为 null,此处展示占位
        render: (v: string | null) =>
          v ? v : <span style={{ color: '#999' }}>(未命名,待完善)</span>,
      },
      { title: '省/市', render: (_v, r) => `${r.province}/${r.city}`, width: 160 },
      {
        title: '账户数',
        dataIndex: 'account_count',
        width: 90,
        align: 'center',
      },
    ];
    if (showClearingEligibleColumn) {
      // 中文注释:清算行资格只属于私权机构;公安局和公权机构列表不得展示该列。
      cols.push({
        title: '清算行资格',
        key: 'clearing_eligible',
        width: 130,
        render: (_v, r) =>
          isSelfEligibleClearingBank(r) ? (
            <Tag color="blue">{CLEARING_BANK_ELIGIBLE_LABEL}</Tag>
          ) : null,
      });
    }
    if (showCreatedByColumn) {
      cols.push({
        title: '创建用户',
        key: 'created_by',
        width: 180,
        render: (_v, r) => {
          const roleLabel = r.created_by_role
            ? CREATED_BY_ROLE_LABEL[r.created_by_role] ?? r.created_by_role
            : '';
          // 三态:姓名+角色 / 仅角色(内置管理员未设姓名)/ 完全未知
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
          if (roleLabel) {
            return <span>{roleLabel}</span>;
          }
          return <span style={{ color: '#999' }}>未知</span>;
        },
      });
    }
    return cols;
  }, [showClearingEligibleColumn, showCreatedByColumn]);

  return (
    <div>
      <Table<InstitutionListRow>
        rowKey={(r) => r.sfid_number}
        loading={loading}
        dataSource={publicSecurityDisplayRows}
        pagination={false}
        // 中文注释:整行可点击,跳详情页。
        onRow={(row) => ({
          onClick: () => onSelectInstitution?.(row.sfid_number),
          style: onSelectInstitution ? { cursor: 'pointer' } : undefined,
        })}
        columns={columns}
      />
      <Space style={{ marginTop: 12 }} wrap>
        {isPublicSecurity && (
          <>
            <Typography.Text type="secondary">
              共 {rows.length} 条 / 第 {publicSecurityPage} 页
            </Typography.Text>
            <Button
              icon={<ReloadOutlined />}
              disabled={loading}
              onClick={onRefreshPublicSecurity}
            >
              刷新
            </Button>
          </>
        )}
        <Button
          disabled={prevDisabled}
          onClick={isPublicSecurity ? onPrevPublicSecurityPage : onPrevPage}
        >
          上一页
        </Button>
        <Button
          disabled={nextDisabled}
          onClick={isPublicSecurity ? onNextPublicSecurityPage : onNextPage}
        >
          下一页
        </Button>
      </Space>
    </div>
  );
};
