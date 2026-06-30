// 中文注释:注册局联邦注册局管理员列表。每节点单省部署,展示本省 5 人组(链上
// FederalRegistryProvinceGroups 全走链读);FRG 可在本省组内操作,CREG 只能只读查看本省组。

import { useState } from 'react';
import { Badge, Button, Card, Form, Input, Modal, Space, Table, Typography } from 'antd';
import { KeyOutlined, ScanOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useAuth } from '../hooks/useAuth';
import { normalizeScopeProvinceName } from '../hooks/useScope';
import { isTier1Registry } from '../platform/registryTier';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../core/ScanAccountModal';
import { CID_MODAL_Z_INDEX } from '../core/modalStack';
import { updateFederalRegistryName, type FederalRegistryAdminRow } from './api';
import { formatAdminCreateError, type AdminActionType } from './admin_security_api';
import { sameHexAccount } from './adminUtils';
import { notice } from '../utils/notice';
import { usePasskeyRegistration } from '../auth/passkey/usePasskey';

interface FederalRegistryAdminSubTabProps {
  selectedFederalRegistry: FederalRegistryAdminRow;
  federalRegistryAdmins: FederalRegistryAdminRow[];
  federalRegistryAdminsLoading: boolean;
  refreshFederalRegistryAdmins: () => Promise<FederalRegistryAdminRow[]>;
  runSecuredAction: <T = unknown>(actionType: AdminActionType, payload: unknown) => Promise<T>;
  /** 该机构 cid_short_name 单一字段,如「联邦注册局」;标题左段显示用。 */
  federalRegistryCidShortName?: string | null;
}

type ReplaceFormValues = {
  admin_account: string;
};

function federalAdminDisplayName(row: FederalRegistryAdminRow): string {
  const name = row.admin_name?.trim();
  return name || '联邦注册局管理员';
}

export function FederalRegistryAdminSubTab({
  selectedFederalRegistry,
  federalRegistryAdmins,
  federalRegistryAdminsLoading,
  refreshFederalRegistryAdmins,
  runSecuredAction,
  federalRegistryCidShortName,
}: FederalRegistryAdminSubTabProps) {
  const { auth, logout } = useAuth();
  const { registered: passkeyRegistered, busy: passkeyBusy, register: doRegisterPasskey } =
    usePasskeyRegistration();
  const [replaceOpen, setReplaceOpen] = useState(false);
  const [replaceLoading, setReplaceLoading] = useState(false);
  const [scanOpen, setScanOpen] = useState(false);
  const [replaceTarget, setReplaceTarget] = useState<FederalRegistryAdminRow | null>(null);
  const [form] = Form.useForm<ReplaceFormValues>();

  const currentProvinceName = normalizeScopeProvinceName(auth?.scope_province_name) || selectedFederalRegistry.province_name;
  const titleProvinceName = currentProvinceName || selectedFederalRegistry.province_name;
  const canOperateFederalRegistry = isTier1Registry(auth?.institution_code);
  const canManageSameProvince = (row: FederalRegistryAdminRow) =>
    canOperateFederalRegistry && row.province_name === currentProvinceName;

  const openReplaceModal = (row: FederalRegistryAdminRow) => {
    if (!canManageSameProvince(row)) {
      notice.error('只能更换本省联邦注册局管理员');
      return;
    }
    setReplaceTarget(row);
    form.resetFields();
    setReplaceOpen(true);
  };

  const submitReplace = async (values: ReplaceFormValues) => {
    if (!replaceTarget) return;
    let adminAccount: string;
    try {
      adminAccount = decodeSs58(values.admin_account.trim());
    } catch (error) {
      notice.error(error, '');
      return;
    }
    if (sameHexAccount(adminAccount, replaceTarget.admin_account)) {
      notice.error('新账户不能与当前账户相同');
      return;
    }
    setReplaceLoading(true);
    const replacingSelf = sameHexAccount(replaceTarget.admin_account, auth?.admin_account);
    try {
      await runSecuredAction<FederalRegistryAdminRow>('REPLACE_GOVERNING_REGISTRY', {
        id: replaceTarget.id,
        admin_account: adminAccount,
      });
      notice.success(replacingSelf ? '管理员已更换，请使用新账户重新登录' : '联邦注册局管理员已更换');
      form.resetFields();
      setReplaceOpen(false);
      setReplaceTarget(null);
      if (replacingSelf) {
        logout();
        return;
      }
      await refreshFederalRegistryAdmins();
    } catch (error) {
      notice.error(formatAdminCreateError(error, 'FEDERAL_REGISTRY', '更换联邦注册局管理员失败'));
    } finally {
      setReplaceLoading(false);
    }
  };

  const editFederalRegistry = (row: FederalRegistryAdminRow) => {
    let nextName = row.admin_name?.trim() || '';
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
        const adminName = nextName.trim();
        if (!adminName) {
          notice.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        try {
          if (!auth) throw new Error('请先登录');
          await updateFederalRegistryName(auth, row.id, adminName);
          notice.success('联邦注册局管理员已更新');
          await refreshFederalRegistryAdmins();
        } catch (error) {
          notice.error(error, '');
          throw error;
        }
      },
    });
  };

  const columns: ColumnsType<FederalRegistryAdminRow> = [
    { title: '序号', width: 70, align: 'center', render: (_v, _row, index) => index + 1 },
    { title: '省份', dataIndex: 'province_name', align: 'center', width: 120 },
    { title: '姓名', align: 'center', width: 160, render: (_v, row) => federalAdminDisplayName(row) },
    { title: '账户', dataIndex: 'admin_account', align: 'center', render: (value: string) => tryEncodeSs58(value) },
  ];

  if (canOperateFederalRegistry) {
    columns.push({
      title: '操作',
      width: 320,
      align: 'center',
      render: (_value, row) => {
        const canManage = canManageSameProvince(row);
        const isSelf = sameHexAccount(row.admin_account, auth?.admin_account);
        return (
          <Space>
            <Button size="small" disabled={!canManage} onClick={() => editFederalRegistry(row)}>
              编辑
            </Button>
            <Button size="small" disabled={!canManage} onClick={() => openReplaceModal(row)}>
              更换
            </Button>
            {/* passkey 登录密钥 self-only:只能为当前登录管理员自己注册本机认证器。 */}
            {isSelf ? (
              <Badge dot={passkeyRegistered === false} size="small">
                <Button
                  size="small"
                  icon={<KeyOutlined />}
                  loading={passkeyBusy}
                  onClick={doRegisterPasskey}
                >
                  {passkeyRegistered ? '更新passkey密钥' : '设置passkey密钥'}
                </Button>
              </Badge>
            ) : null}
          </Space>
        );
      },
    });
  }

  return (
    <>
      <Card
        type="inner"
        title={
          <Space size={6} align="center">
            {/* 中文注释:标题保留省份,让联邦注册局管理员明确看到当前所属省。 */}
            <span>{federalRegistryCidShortName || '联邦注册局'}</span>
            {titleProvinceName ? (
              <>
                <span>·</span>
                <span>{titleProvinceName}</span>
              </>
            ) : null}
          </Space>
        }
      >
      <Table<FederalRegistryAdminRow>
        rowKey={(row) => `${row.id}-${row.admin_account}`}
        loading={federalRegistryAdminsLoading}
        dataSource={federalRegistryAdmins}
        pagination={false}
        columns={columns}
      />
      </Card>

      <Modal
        title={
          <div style={{ textAlign: 'center', width: '100%' }}>
            更换联邦注册局管理员
          </div>
        }
        open={replaceOpen}
        centered
        destroyOnClose
        confirmLoading={replaceLoading}
        cancelButtonProps={{ disabled: replaceLoading }}
        okText="确认更换"
        cancelText="取消"
        onOk={() => form.submit()}
        onCancel={() => {
          if (replaceLoading) return;
          form.resetFields();
          setReplaceTarget(null);
          setReplaceOpen(false);
        }}
        closable={!replaceLoading}
        maskClosable={!replaceLoading}
        zIndex={CID_MODAL_Z_INDEX.business}
      >
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          {replaceTarget ? (
            <div>
              <Typography.Text type="secondary">当前管理员</Typography.Text>
              <div style={{ marginTop: 6 }}>
                {replaceTarget.province_name} · {federalAdminDisplayName(replaceTarget)}
              </div>
              <Typography.Text code style={{ wordBreak: 'break-all' }}>
                {tryEncodeSs58(replaceTarget.admin_account)}
              </Typography.Text>
            </div>
          ) : null}
          <Form form={form} layout="vertical" onFinish={submitReplace}>
            <Form.Item
              label="新账户"
              name="admin_account"
              rules={[{ required: true, message: '请扫码或输入新的管理员账户' }]}
            >
              <Input
                placeholder="请输入新的管理员账户(SS58)"
                suffix={
                  <Button
                    type="text"
                    size="small"
                    icon={<ScanOutlined />}
                    title="扫码识别用户码"
                    onClick={() => setScanOpen(true)}
                  />
                }
              />
            </Form.Item>
          </Form>
        </Space>
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
