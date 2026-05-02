// 省级管理员列表视图 mode='list'(从 ShengAdminsView.tsx 拆分)

import { Button, Card, Form, Input, Select, Table, Tag, Tooltip } from 'antd';
import type { ShengAdminRow } from '../../api/client';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';
import { isSr25519HexPubkey } from './shengAdminUtils';
import type { ShengAdminSharedState } from './shengAdminUtils';

interface ShengAdminListViewProps {
  state: ShengAdminSharedState;
}

export function ShengAdminListView({ state }: ShengAdminListViewProps) {
  const {
    shengAdmins,
    shengAdminsLoading,
    replaceSuperLoading,
    replaceSuperForm,
    onReplaceShengAdmin,
  } = state;

  return (
    <Card
      title="省级管理员列表"
      bordered={false}
      style={glassCardStyle}
      headStyle={glassCardHeadStyle}
      extra={
        <Form
          form={replaceSuperForm}
          layout="inline"
          onFinish={onReplaceShengAdmin}
          style={{ rowGap: 8 }}
        >
          <Form.Item
            name="province"
            rules={[{ required: true, message: '请选择省份' }]}
            style={{ marginBottom: 0 }}
          >
            <Select
              style={{ width: 160 }}
              placeholder="选择省份"
              options={shengAdmins.map((item) => ({ value: item.province, label: item.province }))}
            />
          </Form.Item>
          <Form.Item
            name="admin_name"
            rules={[{ required: true, message: '请输入姓名' }]}
            style={{ marginBottom: 0 }}
          >
            <Input style={{ width: 140 }} placeholder="新管理员姓名" />
          </Form.Item>
          <Form.Item
            name="admin_pubkey"
            rules={[
              { required: true, message: '请输入新省级管理员公钥' },
              {
                validator: async (_rule, value) => {
                  if (!value || isSr25519HexPubkey(String(value))) return;
                  throw new Error('公钥格式必须为 32 字节十六进制');
                },
              },
            ]}
            style={{ marginBottom: 0 }}
          >
            <Input style={{ width: 420, maxWidth: '60vw' }} placeholder="新省级管理员公钥" />
          </Form.Item>
          <Form.Item style={{ marginBottom: 0 }}>
            <Button type="primary" htmlType="submit" loading={replaceSuperLoading}>
              更换省级管理员
            </Button>
          </Form.Item>
        </Form>
      }
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
            title: '签名密钥状态',
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
          { title: '状态', dataIndex: 'status', align: 'center', width: 100 },
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
