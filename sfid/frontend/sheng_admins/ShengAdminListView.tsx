// 省级管理员列表视图 mode='list'(从 ShengAdminsView.tsx 拆分)

import { Card, Table, Tag, Tooltip } from 'antd';
import type { ShengAdminRow } from './api';
import { glassCardStyle, glassCardHeadStyle } from '../common/cardStyles';
import type { ShengAdminSharedState } from './shengAdminUtils';

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
          { title: '公钥', dataIndex: 'admin_pubkey', align: 'center' },
          {
            title: '签名密钥',
            width: 140,
            align: 'center',
            render: (_v: unknown, row: ShengAdminRow) => {
              if (!row.signing_pubkey) {
                return <Tag color="blue">未初始化</Tag>;
              }
              const pk = row.signing_pubkey;
              const brief = pk.length > 14 ? `${pk.slice(0, 7)}...${pk.slice(-5)}` : pk;
              return (
                <Tooltip title={pk}>
                  <Tag color="green">已激活 {brief}</Tag>
                </Tooltip>
              );
            },
          },
          {
            title: '类型',
            width: 100,
            align: 'center',
            render: (_v: unknown, row: ShengAdminRow) => (row.built_in ? '内置' : '自定义'),
          },
        ]}
      />
    </Card>
  );
}
