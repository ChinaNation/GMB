// 中文注释:市详情页的机构表格。
// 走任务卡 2 新 API listInstitutions,按 category + province + city 过滤。
// 整行可点击 → 进机构详情页。不显示"操作"列。

import React, { useEffect, useState } from 'react';
import { message, Table } from 'antd';
import {
  listInstitutions,
  type InstitutionCategory,
  type InstitutionListRow,
} from '../../api/institution';
import type { AdminAuth } from '../../api/client';

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
}

export const InstitutionListTable: React.FC<Props> = ({
  auth,
  category,
  province,
  city,
  onSelectInstitution,
  refreshKey,
}) => {
  const [rows, setRows] = useState<InstitutionListRow[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    // 中文注释:city 为空时传 undefined,后端按省过滤即可
    listInstitutions(auth, { category, province, city: city || undefined })
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
  }, [auth.access_token, category, province, city, refreshKey]);

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
        columns={[
          { title: 'SFID', dataIndex: 'sfid_id', width: 260 },
          { title: '机构名称', dataIndex: 'institution_name', width: 180 },
          { title: '省/市', render: (_v, r) => `${r.province}/${r.city}`, width: 160 },
          {
            title: '账户数',
            dataIndex: 'account_count',
            width: 90,
            align: 'center',
          },
        ]}
      />
    </div>
  );
};
