// 中文注释:注册局联邦注册局管理员列表。所有联邦注册局管理员同级,
// 代码内置初始联邦注册局管理员只作为不可删除安全根保留。

import { useMemo, useState } from 'react';
import { Button, Card, Form, Input, Modal, Space, Table, Typography } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../core/ScanAccountModal';
import { CID_MODAL_Z_INDEX } from '../core/modalStack';
import { updateFederalRegistryName, type FederalRegistryAdminRow } from './api';
import { formatAdminCreateError, type AdminActionType } from './admin_security_api';
import { sameHexAccount } from './adminUtils';
import { Passkey } from './Passkey';
import { notice } from '../utils/notice';

interface FederalRegistryAdminSubTabProps {
  selectedFederalRegistry: FederalRegistryAdminRow;
  federalRegistryAdmins: FederalRegistryAdminRow[];
  federalRegistryAdminsLoading: boolean;
  refreshFederalRegistryAdmins: () => Promise<FederalRegistryAdminRow[]>;
  runSecuredAction: <T = unknown>(actionType: AdminActionType, payload: unknown) => Promise<T>;
  /** 该机构 cid_short_name 单一字段,如「联邦注册局」;标题左段显示用。 */
  federalRegistryCidShortName?: string | null;
}

type AddFormValues = {
  admin_display_name: string;
  admin_account: string;
};

export function FederalRegistryAdminSubTab({
  selectedFederalRegistry,
  federalRegistryAdmins,
  federalRegistryAdminsLoading,
  refreshFederalRegistryAdmins,
  runSecuredAction,
  federalRegistryCidShortName,
}: FederalRegistryAdminSubTabProps) {
  const { auth } = useAuth();
  const [addOpen, setAddOpen] = useState(false);
  const [addLoading, setAddLoading] = useState(false);
  const [scanOpen, setScanOpen] = useState(false);
  const [form] = Form.useForm<AddFormValues>();

  const provinceAdmins = useMemo(
    () => federalRegistryAdmins.filter((row) => row.province_name === selectedFederalRegistry.province_name),
    [selectedFederalRegistry.province_name, federalRegistryAdmins],
  );
  const currentAdminRow = auth
    ? provinceAdmins.find((row) => sameHexAccount(row.admin_account, auth.admin_account)) || null
    : null;
  const canAddFederalRegistryAdmin = auth?.registry_org_code === 'FEDERAL_REGISTRY';
  const federalRegistryAdminLimitReached = provinceAdmins.length >= 5;
  const canDeleteFederalRegistry = (row: FederalRegistryAdminRow) =>
    !!currentAdminRow?.built_in
    && !row.built_in
    && !sameHexAccount(row.admin_account, auth?.admin_account);

  const submitAdd = async (values: AddFormValues) => {
    const adminDisplayName = values.admin_display_name.trim();
    if (!adminDisplayName) {
      notice.error('请输入联邦注册局管理员姓名');
      return;
    }
    let adminAccount: string;
    try {
      adminAccount = decodeSs58(values.admin_account.trim());
    } catch (error) {
      notice.error(error, '');
      return;
    }
    setAddLoading(true);
    try {
      await runSecuredAction<FederalRegistryAdminRow>('CREATE_FEDERAL_REGISTRY', {
        admin_account: adminAccount,
        admin_display_name: adminDisplayName,
      });
      notice.success('联邦注册局管理员已新增');
      form.resetFields();
      setAddOpen(false);
      await refreshFederalRegistryAdmins();
    } catch (error) {
      notice.error(formatAdminCreateError(error, 'FEDERAL_REGISTRY', '新增联邦注册局管理员失败'));
    } finally {
      setAddLoading(false);
    }
  };

  const editFederalRegistry = (row: FederalRegistryAdminRow) => {
    let nextName = row.admin_display_name;
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>编辑联邦注册局管理员</div>,
      icon: null,
      centered: true,
      zIndex: CID_MODAL_Z_INDEX.business,
      content: (
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div>
            <Typography.Text type="secondary">管理员姓名</Typography.Text>
            <Input
              defaultValue={row.admin_display_name}
              placeholder="请输入管理员姓名"
              style={{ marginTop: 6 }}
              onChange={(event) => {
                nextName = event.target.value;
              }}
            />
          </div>
          <div>
            <Typography.Text type="secondary">账户地址</Typography.Text>
            <Input value={tryEncodeSs58(row.admin_account)} disabled style={{ marginTop: 6 }} />
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
        const adminDisplayName = nextName.trim();
        if (!adminDisplayName) {
          notice.error('请输入管理员姓名');
          throw new Error('admin_display_name is required');
        }
        try {
          if (!auth) throw new Error('请先登录');
          await updateFederalRegistryName(auth, row.id, adminDisplayName);
          notice.success('联邦注册局管理员已更新');
          await refreshFederalRegistryAdmins();
        } catch (error) {
          notice.error(error, '');
          throw error;
        }
      },
    });
  };

  const deleteFederalRegistry = (row: FederalRegistryAdminRow) => {
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>删除联邦注册局管理员</div>,
      icon: null,
      centered: true,
      zIndex: CID_MODAL_Z_INDEX.business,
      content: (
        <div style={{ textAlign: 'center' }}>
          <Typography.Paragraph style={{ marginBottom: 8 }}>确认删除该联邦注册局管理员?</Typography.Paragraph>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>{tryEncodeSs58(row.admin_account)}</Typography.Text>
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
          await runSecuredAction('DELETE_FEDERAL_REGISTRY', { id: row.id });
          notice.success('联邦注册局管理员已删除');
          await refreshFederalRegistryAdmins();
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
            {/* 中文注释:左段显示机构简称(cid_short_name 单一真源),不再硬编码「联邦注册局管理员列表」。 */}
            <span>{federalRegistryCidShortName || '联邦注册局'}</span>
            <span style={{ lineHeight: 1, color: 'rgba(15,23,42,0.45)' }}>·</span>
            <span>{selectedFederalRegistry.province_name}</span>
          </Space>
        }
        extra={
          <Space size="middle" align="center">
            <Typography.Text type="secondary" style={{ fontWeight: 400, fontSize: 13 }}>
              用户数：{provinceAdmins.length} / 5
            </Typography.Text>
            {canAddFederalRegistryAdmin ? (
              <Button
                type="primary"
                disabled={federalRegistryAdminLimitReached}
                title={federalRegistryAdminLimitReached ? '联邦注册局管理员已满 5 人' : undefined}
                onClick={() => setAddOpen(true)}
              >
                新增联邦注册局管理员
              </Button>
            ) : null}
          </Space>
        }
      >
      <Table<FederalRegistryAdminRow>
        rowKey={(row) => `${row.id}-${row.admin_account}`}
        loading={federalRegistryAdminsLoading}
        dataSource={provinceAdmins}
        pagination={false}
        columns={[
          { title: '序号', width: 70, align: 'center', render: (_v, _row, index) => index + 1 },
          { title: '姓名', dataIndex: 'admin_display_name', align: 'center', width: 160 },
          { title: '账户', dataIndex: 'admin_account', align: 'center', render: (value: string) => tryEncodeSs58(value) },
          {
            title: '操作',
            width: 260,
            align: 'center',
            render: (_value, row) => {
              const isSelf = sameHexAccount(row.admin_account, auth?.admin_account);
              return (
                <Space>
                  {canAddFederalRegistryAdmin ? <Button size="small" onClick={() => editFederalRegistry(row)}>编辑</Button> : null}
                  <Button
                    size="small"
                    danger
                    disabled={!canDeleteFederalRegistry(row)}
                    onClick={() => deleteFederalRegistry(row)}
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
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增联邦注册局管理员</div>}
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
        zIndex={CID_MODAL_Z_INDEX.business}
      >
        <Form form={form} layout="vertical" onFinish={submitAdd}>
          <Form.Item
            label="姓名"
            name="admin_display_name"
            rules={[{ required: true, message: '请输入联邦注册局管理员姓名' }]}
          >
            <Input placeholder="请输入联邦注册局管理员姓名" />
          </Form.Item>
          <Form.Item
            label="账户"
            name="admin_account"
            rules={[{ required: true, message: '请扫码或输入联邦注册局管理员账户' }]}
          >
            <Input
              placeholder="请输入联邦注册局管理员账户(SS58)"
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
          form.setFieldsValue({ admin_account: address });
          setScanOpen(false);
        }}
      />
    </>
  );
}
