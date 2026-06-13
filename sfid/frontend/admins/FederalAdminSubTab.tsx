// 中文注释:注册局联邦管理员列表。所有联邦管理员同级,
// 代码内置初始联邦管理员只作为不可删除安全根保留。

import { useMemo, useState } from 'react';
import { Button, Card, Form, Input, Modal, Space, Table, Typography } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../core/ScanAccountModal';
import { SFID_MODAL_Z_INDEX } from '../core/modalStack';
import { updateFederalAdminName, type FederalAdminRow } from './api';
import { formatAdminCreateError, type AdminActionType } from './admin_security_api';
import { sameHexPubkey } from './adminUtils';
import { Passkey } from './Passkey';
import { notice } from '../utils/notice';

interface FederalAdminSubTabProps {
  selectedFederalAdmin: FederalAdminRow;
  federalAdmins: FederalAdminRow[];
  federalAdminsLoading: boolean;
  refreshFederalAdmins: () => Promise<FederalAdminRow[]>;
  runSecuredAction: <T = unknown>(actionType: AdminActionType, payload: unknown) => Promise<T>;
}

type AddFormValues = {
  admin_name: string;
  admin_pubkey: string;
};

export function FederalAdminSubTab({
  selectedFederalAdmin,
  federalAdmins,
  federalAdminsLoading,
  refreshFederalAdmins,
  runSecuredAction,
}: FederalAdminSubTabProps) {
  const { auth } = useAuth();
  const [addOpen, setAddOpen] = useState(false);
  const [addLoading, setAddLoading] = useState(false);
  const [scanOpen, setScanOpen] = useState(false);
  const [form] = Form.useForm<AddFormValues>();

  const provinceAdmins = useMemo(
    () => federalAdmins.filter((row) => row.province === selectedFederalAdmin.province),
    [selectedFederalAdmin.province, federalAdmins],
  );
  const currentAdminRow = auth
    ? provinceAdmins.find((row) => sameHexPubkey(row.admin_pubkey, auth.admin_pubkey)) || null
    : null;
  const canAddFederalAdmin = auth?.role === 'FEDERAL_ADMIN';
  const federalAdminLimitReached = provinceAdmins.length >= 5;
  const canDeleteFederalAdmin = (row: FederalAdminRow) =>
    !!currentAdminRow?.built_in
    && !row.built_in
    && !sameHexPubkey(row.admin_pubkey, auth?.admin_pubkey);

  const submitAdd = async (values: AddFormValues) => {
    const adminName = values.admin_name.trim();
    if (!adminName) {
      notice.error('请输入联邦管理员姓名');
      return;
    }
    let adminPubkey: string;
    try {
      adminPubkey = decodeSs58(values.admin_pubkey.trim());
    } catch (error) {
      notice.error(error, '');
      return;
    }
    setAddLoading(true);
    try {
      await runSecuredAction<FederalAdminRow>('CREATE_FEDERAL_ADMIN', {
        admin_pubkey: adminPubkey,
        admin_name: adminName,
      });
      notice.success('联邦管理员已新增');
      form.resetFields();
      setAddOpen(false);
      await refreshFederalAdmins();
    } catch (error) {
      notice.error(formatAdminCreateError(error, 'FEDERAL_ADMIN', '新增联邦管理员失败'));
    } finally {
      setAddLoading(false);
    }
  };

  const editFederalAdmin = (row: FederalAdminRow) => {
    let nextName = row.admin_name;
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>编辑联邦管理员</div>,
      icon: null,
      centered: true,
      zIndex: SFID_MODAL_Z_INDEX.business,
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
          notice.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        try {
          if (!auth) throw new Error('请先登录');
          await updateFederalAdminName(auth, row.id, adminName);
          notice.success('联邦管理员已更新');
          await refreshFederalAdmins();
        } catch (error) {
          notice.error(error, '');
          throw error;
        }
      },
    });
  };

  const deleteFederalAdmin = (row: FederalAdminRow) => {
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>删除联邦管理员</div>,
      icon: null,
      centered: true,
      zIndex: SFID_MODAL_Z_INDEX.business,
      content: (
        <div style={{ textAlign: 'center' }}>
          <Typography.Paragraph style={{ marginBottom: 8 }}>确认删除该联邦管理员?</Typography.Paragraph>
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
          await runSecuredAction('DELETE_FEDERAL_ADMIN', { id: row.id });
          notice.success('联邦管理员已删除');
          await refreshFederalAdmins();
        } catch (error) {
          notice.error(error, '');
          throw error;
        }
      },
    });
  };

  return (
    <>
      <Card
        type="inner"
        title={
          <Space size={6} align="center">
            <span>联邦管理员列表</span>
            <span style={{ lineHeight: 1, color: 'rgba(15,23,42,0.45)' }}>·</span>
            <span>{selectedFederalAdmin.province}</span>
          </Space>
        }
        extra={
          <Space size="middle" align="center">
            <Typography.Text type="secondary" style={{ fontWeight: 400, fontSize: 13 }}>
              用户数：{provinceAdmins.length} / 5
            </Typography.Text>
            {canAddFederalAdmin ? (
              <Button
                type="primary"
                disabled={federalAdminLimitReached}
                title={federalAdminLimitReached ? '联邦管理员已满 5 人' : undefined}
                onClick={() => setAddOpen(true)}
              >
                新增联邦管理员
              </Button>
            ) : null}
          </Space>
        }
      >
      <Table<FederalAdminRow>
        rowKey={(row) => `${row.id}-${row.admin_pubkey}`}
        loading={federalAdminsLoading}
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
                  {canAddFederalAdmin ? <Button size="small" onClick={() => editFederalAdmin(row)}>编辑</Button> : null}
                  <Button
                    size="small"
                    danger
                    disabled={!canDeleteFederalAdmin(row)}
                    onClick={() => deleteFederalAdmin(row)}
                  >
                    删除
                  </Button>
                  <Passkey size="small" disabled={!isSelf} />
                </Space>
              );
            },
          },
        ]}
      />
      </Card>

      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增联邦管理员</div>}
        open={addOpen}
        centered
        destroyOnClose
        confirmLoading={addLoading}
        cancelButtonProps={{ disabled: addLoading }}
        okText="确认新增"
        cancelText="取消"
        onOk={() => form.submit()}
        onCancel={() => {
          if (addLoading) return;
          form.resetFields();
          setAddOpen(false);
        }}
        closable={!addLoading}
        maskClosable={!addLoading}
        zIndex={SFID_MODAL_Z_INDEX.business}
      >
        <Form form={form} layout="vertical" onFinish={submitAdd}>
          <Form.Item
            label="姓名"
            name="admin_name"
            rules={[{ required: true, message: '请输入联邦管理员姓名' }]}
          >
            <Input placeholder="请输入联邦管理员姓名" />
          </Form.Item>
          <Form.Item
            label="账户"
            name="admin_pubkey"
            rules={[{ required: true, message: '请扫码或输入联邦管理员账户' }]}
          >
            <Input
              placeholder="请输入联邦管理员账户(SS58)"
              suffix={
                <span
                  title="扫码识别用户码"
                  style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                  onClick={() => setScanOpen(true)}
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M3 7V5a2 2 0 0 1 2-2h2" />
                    <path d="M17 3h2a2 2 0 0 1 2 2v2" />
                    <path d="M21 17v2a2 2 0 0 1-2 2h-2" />
                    <path d="M7 21H5a2 2 0 0 1-2-2v-2" />
                    <rect x="7" y="7" width="10" height="10" rx="1" />
                  </svg>
                </span>
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
