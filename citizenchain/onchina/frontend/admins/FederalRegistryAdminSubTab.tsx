// 注册局联邦注册局管理员列表。链上按省级 5 人组存放,本页展示全部省份管理员;
// 当前登录省份置顶且只允许本省组执行更换,其它省份只读。

import { useMemo, useState } from 'react';
import { Badge, Button, Card, Form, Input, Modal, Space, Table, Typography } from 'antd';
import { KeyOutlined, ScanOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useAuth } from '../hooks/useAuth';
import { normalizeScopeProvinceName } from '../hooks/useScope';
import { isTier1Registry } from '../platform/registryTier';
import { decodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../core/ScanAccountModal';
import { CID_MODAL_Z_INDEX } from '../core/modalStack';
import type { FederalRegistryAdminRow } from './api';
import { formatAdminCreateError, type AdminActionType } from './securityApi';
import { sameHexAccount } from './adminUtils';
import { notice } from '../utils/notice';
import { usePasskeyRegistration } from '../auth/passkey/usePasskey';
import {
  AdminProfileDetails,
  adminDisplayName,
  formatAdminBalanceFen,
} from './AdminProfileCard';

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

const FEDERAL_REGISTRY_PAGE_SIZE = 20;

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
  const [detailTarget, setDetailTarget] = useState<FederalRegistryAdminRow | null>(null);
  const [federalListPage, setFederalListPage] = useState(1);
  const [form] = Form.useForm<ReplaceFormValues>();

  const currentProvinceName = normalizeScopeProvinceName(auth?.scope_province_name) || selectedFederalRegistry.province_name;
  const titleProvinceName = currentProvinceName || selectedFederalRegistry.province_name;
  const canOperateFederalRegistry = isTier1Registry(auth?.institution_code);
  const canManageSameProvince = (row: FederalRegistryAdminRow) =>
    canOperateFederalRegistry && normalizeScopeProvinceName(row.province_name) === currentProvinceName;

  const displayedFederalRegistryAdmins = useMemo(
    () => federalRegistryAdmins
      .map((row, index) => ({ row, index }))
      .sort((left, right) => {
        const leftCurrent = normalizeScopeProvinceName(left.row.province_name) === currentProvinceName;
        const rightCurrent = normalizeScopeProvinceName(right.row.province_name) === currentProvinceName;
        if (leftCurrent !== rightCurrent) return leftCurrent ? -1 : 1;
        return left.index - right.index;
      })
      .map((entry) => entry.row),
    [currentProvinceName, federalRegistryAdmins],
  );

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

  const columns: ColumnsType<FederalRegistryAdminRow> = [
    {
      title: '序号',
      width: 72,
      align: 'center',
      render: (_value, _row, index) => (federalListPage - 1) * FEDERAL_REGISTRY_PAGE_SIZE + index + 1,
    },
    { title: '省份', dataIndex: 'province_name', align: 'center', width: 120 },
    {
      title: '姓名',
      dataIndex: 'name',
      render: (_value, row) => adminDisplayName(row) || '-',
    },
    {
      title: '余额',
      dataIndex: 'balance_fen',
      width: 160,
      align: 'right',
      render: (_value, row) => formatAdminBalanceFen(row.balance_fen) || '-',
    },
  ];

  columns.push({
    title: '操作',
    width: 220,
    align: 'center',
    render: (_value, row) => {
      const canManage = canManageSameProvince(row);
      const isSelf = sameHexAccount(row.admin_account, auth?.admin_account);
      return (
        <Space onClick={(event) => event.stopPropagation()}>
          <Button
            size="small"
            disabled={!canManage}
            title={!canManage ? '只能更换本省联邦注册局管理员' : undefined}
            onClick={() => openReplaceModal(row)}
          >
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
                密钥
              </Button>
            </Badge>
          ) : null}
        </Space>
      );
    },
  });

  return (
    <>
      <Card
        type="inner"
        title={
          <Space size={6} align="center">
            {/* 标题保留省份,让联邦注册局管理员明确看到当前所属省。 */}
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
        dataSource={displayedFederalRegistryAdmins}
        pagination={{
          pageSize: FEDERAL_REGISTRY_PAGE_SIZE,
          current: federalListPage,
          showSizeChanger: false,
          showTotal: (total) => `共 ${total} 条`,
          onChange: (page) => setFederalListPage(page),
        }}
        columns={columns}
        onRow={(row) => ({
          onClick: () => setDetailTarget(row),
          style: { cursor: 'pointer' },
        })}
      />
      </Card>

      <Modal
        title="管理员完整信息"
        open={!!detailTarget}
        footer={null}
        centered
        onCancel={() => setDetailTarget(null)}
        zIndex={CID_MODAL_Z_INDEX.business}
      >
        {detailTarget ? (
          <AdminProfileDetails
            profile={detailTarget}
            areaLabel="省份"
            areaValue={detailTarget.province_name}
          />
        ) : null}
      </Modal>

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
              <Typography.Text type="secondary">当前省份</Typography.Text>
              <div style={{ marginTop: 6, marginBottom: 8 }}>
                {replaceTarget.province_name}
              </div>
              <AdminProfileDetails
                profile={replaceTarget}
                areaLabel="省份"
                areaValue={replaceTarget.province_name}
              />
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
