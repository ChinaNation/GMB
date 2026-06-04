// 省级管理员列表视图 mode='list'(从 ShengAdminsView.tsx 拆分)

import { Card, Table } from 'antd';
import type { ShengAdminRow } from './api';
import { glassCardStyle, glassCardHeadStyle } from '../core/cardStyles';
import type { ShengAdminSharedState } from './shengAdminUtils';
import { tryEncodeSs58 } from '../utils/ss58';

interface ShengAdminListViewProps {
  state: ShengAdminSharedState;
}

export function ShengAdminListView({ state }: ShengAdminListViewProps) {
  const {
    shengAdmins,
    shengAdminsLoading,
  } = state;

  return (
    <Card
      title="省级管理员列表"
      bordered={false}
      style={glassCardStyle}
      headStyle={glassCardHeadStyle}
    >
      <Table<ShengAdminRow>
        rowKey={(r) => `${r.province}-${r.admin_pubkey}`}
        loading={shengAdminsLoading}
        dataSource={shengAdmins}
        pagination={{ pageSize: 10 }}
        columns={[
          {
            title: '序号',
            width: 80,
            align: 'center',
            render: (_v: unknown, _row: ShengAdminRow, index: number) => index + 1,
          },
          { title: '省份', dataIndex: 'province', align: 'center', width: 140 },
          { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 180 },
          { title: '账户', dataIndex: 'admin_pubkey', align: 'center', render: (value: string) => tryEncodeSs58(value) },
        ]}
      />
    </Card>
  );
}
