// 中文注释:注册局省级管理员列表。省级管理员不再区分主/备,
// 初始省级管理员只作为不可删除安全根保留。

import { useMemo, useState } from 'react';
import { Button, Form, Input, Modal, Space, Table, Typography, message } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../common/ScanAccountModal';
import type { ShengAdminRow } from './api';
import type { AdminActionType } from './admin_security_api';
import { sameHexPubkey } from './shengAdminUtils';
import { AdminPasskeyTool } from './AdminPasskeyTool';

interface SuperAdminSubTabProps {
  selectedShengAdmin: ShengAdminRow;
  shengAdmins: ShengAdminRow[];
  shengAdminsLoading: boolean;
  refreshShengAdmins: () => Promise<ShengAdminRow[]>;
  runSecuredAction: <T = unknown>(actionType: AdminActionType, payload: unknown) => Promise<T>;
}

type AddFormValues = {
  admin_name: string;
  admin_pubkey: string;
};

export function SuperAdminSubTab({
  selectedShengAdmin,
  shengAdmins,
  shengAdminsLoading,
  refreshShengAdmins,
  runSecuredAction,
}: SuperAdminSubTabProps) {
  const { auth } = useAuth();
  const [addOpen, setAddOpen] = useState(false);
  const [addLoading, setAddLoading] = useState(false);
  const [scanOpen, setScanOpen] = useState(false);
  const [form] = Form.useForm<AddFormValues>();

  const provinceAdmins = useMemo(
    () => shengAdmins.filter((row) => row.province === selectedShengAdmin.province),
    [selectedShengAdmin.province, shengAdmins],
  );
  const currentAdminRow = auth
    ? provinceAdmins.find((row) => sameHexPubkey(row.admin_pubkey, auth.admin_pubkey)) || null
    : null;
  const canAddShengAdmin = auth?.role === 'SHENG_ADMIN';
  const canDeleteShengAdmin = (row: ShengAdminRow) =>
    !!currentAdminRow?.built_in
    && !row.built_in
    && !sameHexPubkey(row.admin_pubkey, auth?.admin_pubkey);

  const submitAdd = async (values: AddFormValues) => {
    const adminName = values.admin_name.trim();
    if (!adminName) {
      message.error('请输入省级管理员姓名');
      return;
    }
    let adminPubkey: string;
    try {
      adminPubkey = decodeSs58(values.admin_pubkey.trim());
    } catch (error) {
      message.error(error instanceof Error ? error.message : '账户格式无效');
      return;
    }
    setAddLoading(true);
    try {
      await runSecuredAction<ShengAdminRow>('CREATE_SHENG_ADMIN', {
        admin_pubkey: adminPubkey,
        admin_name: adminName,
      });
      message.success('省级管理员已新增');
      form.resetFields();
      setAddOpen(false);
      await refreshShengAdmins();
    } catch (error) {
      message.error(error instanceof Error ? error.message : '新增省级管理员失败');
    } finally {
      setAddLoading(false);
    }
  };

  const editShengAdmin = (row: ShengAdminRow) => {
    let nextName = row.admin_name;
    Modal.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>编辑省级管理员</div>,
      icon: null,
      centered: true,
      content: (
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div>
            <Typography.Text type="secondary">管理员姓名</Typography.Text>
            <Input
              defaultValue={row.admin_name}
              placeholder="请输入管理员姓名"
              style={{ marginTop: 6 }}
              onChange={(event) => {
                nextName = event.target.value;
              }}
            />
          </div>
          <div>
            <Typography.Text type="secondary">账户地址</Typography.Text>
            <Input value={tryEncodeSs58(row.admin_pubkey)} disabled style={{ marginTop: 6 }} />
          </div>
        </Space>
      ),
      okText: '确认修改',
      cancelText: '取消',
      footer: (_originNode, { OkBtn, CancelBtn }) => (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 8 }}>
          <CancelBtn />
          <OkBtn />
        </div>
      ),
      onOk: async () => {
        const adminName = nextName.trim();
        if (!adminName) {
          message.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        try {
          await runSecuredAction<ShengAdminRow>('UPDATE_SHENG_ADMIN', {
            id: row.id,
            admin_name: adminName,
          });
          message.success('省级管理员已更新');
          await refreshShengAdmins();
        } catch (error) {
          message.error(error instanceof Error ? error.message : '更新省级管理员失败');
          throw error;
        }
      },
    });
  };

  const deleteShengAdmin = (row: ShengAdminRow) => {
    Modal.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>删除省级管理员</div>,
      icon: null,
      centered: true,
      content: (
        <div style={{ textAlign: 'center' }}>
          <Typography.Paragraph style={{ marginBottom: 8 }}>确认删除该省级管理员?</Typography.Paragraph>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>{tryEncodeSs58(row.admin_pubkey)}</Typography.Text>
        </div>
      ),
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      footer: (_originNode, { OkBtn, CancelBtn }) => (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 8 }}>
          <CancelBtn />
          <OkBtn />
        </div>
      ),
      onOk: async () => {
        try {
          await runSecuredAction('DELETE_SHENG_ADMIN', { id: row.id });
          message.success('省级管理员已删除');
          await refreshShengAdmins();
        } catch (error) {
          message.error(error instanceof Error ? error.message : '删除省级管理员失败');
          throw error;
        }
      },
    });
  };

  return (
    <>
      <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: 12 }}>
        {canAddShengAdmin ? (
          <Button type="primary" onClick={() => setAddOpen(true)}>新增省级管理员</Button>
        ) : null}
      </div>
      <Table<ShengAdminRow>
        rowKey={(row) => `${row.id}-${row.admin_pubkey}`}
        loading={shengAdminsLoading}
        dataSource={provinceAdmins}
        pagination={false}
        columns={[
          { title: '序号', width: 70, align: 'center', render: (_v, _row, index) => index + 1 },
          { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
          { title: '账户', dataIndex: 'admin_pubkey', align: 'center', render: (value: string) => tryEncodeSs58(value) },
          {
            title: '操作',
            width: 260,
            align: 'center',
            render: (_value, row) => {
              const isSelf = sameHexPubkey(row.admin_pubkey, auth?.admin_pubkey);
              return (
                <Space>
                  {canAddShengAdmin ? <Button size="small" onClick={() => editShengAdmin(row)}>编辑</Button> : null}
                  <Button
                    size="small"
                    danger
                    disabled={!canDeleteShengAdmin(row)}
                    onClick={() => deleteShengAdmin(row)}
                  >
                    删除
                  </Button>
                  <AdminPasskeyTool size="small" disabled={!isSelf} />
                </Space>
              );
            },
          },
        ]}
      />

      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增省级管理员</div>}
        open={addOpen}
        centered
        destroyOnClose
        confirmLoading={addLoading}
        okText="确认新增"
        cancelText="取消"
        onOk={() => form.submit()}
        onCancel={() => {
          form.resetFields();
          setAddOpen(false);
        }}
      >
        <Form form={form} layout="vertical" onFinish={submitAdd}>
          <Form.Item
            label="姓名"
            name="admin_name"
            rules={[{ required: true, message: '请输入省级管理员姓名' }]}
          >
            <Input placeholder="请输入省级管理员姓名" />
          </Form.Item>
          <Form.Item
            label="账户"
            name="admin_pubkey"
            rules={[{ required: true, message: '请扫码或输入省级管理员账户' }]}
          >
            <Input
              placeholder="请输入省级管理员账户(SS58)"
              suffix={
                <Button type="link" size="small" onClick={() => setScanOpen(true)} style={{ paddingInline: 0 }}>
                  扫码
                </Button>
              }
            />
          </Form.Item>
        </Form>
      </Modal>

      <ScanAccountModal
        open={scanOpen}
        onClose={() => setScanOpen(false)}
        onResolved={(address) => {
          form.setFieldsValue({ admin_pubkey: address });
          setScanOpen(false);
        }}
      />
    </>
  );
}
