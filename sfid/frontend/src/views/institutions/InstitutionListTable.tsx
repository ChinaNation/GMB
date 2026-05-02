// 中文注释:市详情页的机构表格。
// 走任务卡 2 新 API listInstitutions,按 category + province + city 过滤。
// 整行可点击 → 进机构详情页。不显示"操作"列。

import React, { useEffect, useMemo, useState } from 'react';
import { message, Table, Tag, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import {
  listInstitutions,
  type InstitutionCategory,
  type InstitutionListRow,
} from '../../api/institution';
import type { AdminAuth } from '../../api/client';
import {
  isSelfEligibleClearingBank,
  CLEARING_BANK_ELIGIBLE_LABEL,
} from '../../utils/clearingBankEligible';

// 创建者角色中文映射(ADR-008 起 KEY_ADMIN 已删)
const CREATED_BY_ROLE_LABEL: Record<string, string> = {
  SHENG_ADMIN: '省级管理员',
  SHI_ADMIN: '市级管理员',
};

interface Props {
  auth: AdminAuth;
  category: InstitutionCategory;
  province: string;
  /** 为空字符串时表示"该省全部市",用于公安局省级总览 */
  city: string;
  /** 点击某行机构时回调(跳机构详情页) */
  onSelectInstitution?: (sfidId: string) => void;
  /** 让父组件触发刷新 */
  refreshKey?: number;
  /** 模糊搜索关键字(机构名称或 SFID);空=不过滤。scope 由后端按角色自动限制 */
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

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    // 中文注释:city 为空时传 undefined,后端按省过滤即可
    listInstitutions(auth, {
      category,
      province,
      city: city || undefined,
      q: searchQuery && searchQuery.trim() ? searchQuery.trim() : undefined,
    })
      .then((data) => {
        if (!cancelled) setRows(data);
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
  }, [auth.access_token, category, province, city, refreshKey, searchQuery]);

  // 中文注释:"创建用户"列仅对**私权机构**展示。
  // 公安局由后端 reconcile 批量生成,created_by 不具人类语义;
  // 公权机构本轮未做两步式改造,暂不展示(下一步再加)。
  const showCreatedByColumn = category === 'PRIVATE_INSTITUTION';

  const columns = useMemo<ColumnsType<InstitutionListRow>>(() => {
    const cols: ColumnsType<InstitutionListRow> = [
      { title: 'SFID', dataIndex: 'sfid_id', width: 260 },
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
      // 清算行资格 badge(2026-04-24, ADR-007):
      // SFR + JOINT_STOCK 直接判定(列表行字段足够);FFR 因列表行无 parent 字段不在此渲染,
      // 详情页有完整 parent 信息后再判定。
      {
        title: '清算行资格',
        key: 'clearing_eligible',
        width: 130,
        render: (_v, r) =>
          isSelfEligibleClearingBank(r) ? (
            <Tag color="blue">{CLEARING_BANK_ELIGIBLE_LABEL}</Tag>
          ) : null,
      },
    ];
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
  }, [showCreatedByColumn]);

  return (
    <div>
      <Table<InstitutionListRow>
        rowKey={(r) => r.sfid_id}
        loading={loading}
        dataSource={rows}
        pagination={{ pageSize: 10 }}
        // 中文注释:整行可点击,跳详情页。
        onRow={(row) => ({
          onClick: () => onSelectInstitution?.(row.sfid_id),
          style: onSelectInstitution ? { cursor: 'pointer' } : undefined,
        })}
        columns={columns}
      />
    </div>
  );
};
