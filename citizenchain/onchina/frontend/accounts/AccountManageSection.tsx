// 机构自助账户管理区块。机构自定义账户的新增/删除归机构自己的在册管理员,
// 挂在机构自己的工作区(私权/通用/司法等),注册局详情页只读账户、不在此增删。
//
// 数据源用 getOwnInstitution(基于激活节点绑定,不接受前端传 cid_number),
// 拿到本机构 cid_number/全称与账户列表;增删走 accounts/api 的 createAccount/deleteAccount,
// 后端按 is_institution_admin 授权。协议账户(主/费用/两和基金/安全基金)由后端
// can_delete=false 标记,AccountList 据此自动隐藏删除按钮。

import { useEffect, useState } from 'react';
import { Button, Card } from 'antd';
import type { AdminAuth } from '../auth/types';
import { getOwnInstitution } from '../admins/api';
import { createScanSignSecurityGrant } from '../admins/securityApi';
import { useScanSignGrant } from '../core/useScanSignGrant';
import { deleteAccount, type InstitutionAccount } from './api';
import { AccountList } from './AccountList';
import { CreateAccountModal } from './CreateAccountModal';
import { notice } from '../utils/notice';

export type AccountManageSectionProps = {
  auth: AdminAuth;
};

export function AccountManageSection({ auth }: AccountManageSectionProps) {
  const [cidNumber, setCidNumber] = useState('');
  const [cidFullName, setCidFullName] = useState('');
  const [accounts, setAccounts] = useState<InstitutionAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  // refreshKey 自增触发本机构账户重载(增删成功后调用 reload)。
  const [refreshKey, setRefreshKey] = useState(0);
  // 删除属 PASSKEY_COLD_SIGN,复用扫码签名 hook 拿一次性授权。
  const { signWithScan, scanSignModal } = useScanSignGrant('账户删除签名确认');

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    getOwnInstitution(auth)
      .then((detail) => {
        if (cancelled) return;
        setCidNumber(detail.institution.cid_number);
        setCidFullName(detail.institution.cid_full_name ?? '');
        setAccounts(detail.accounts);
      })
      .catch((err) => {
        if (!cancelled) notice.error(err, '加载本机构账户失败');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [auth.access_token, refreshKey]);

  const reload = () => setRefreshKey((v) => v + 1);

  const onDeleteAccount = async (accountName: string) => {
    try {
      const grant = await createScanSignSecurityGrant(
        auth,
        'INSTITUTION_DELETE_ACCOUNT',
        { target: cidNumber, cid_number: cidNumber, account_name: accountName },
        signWithScan,
      );
      await deleteAccount(auth, cidNumber, accountName, grant);
      notice.success(`账户 "${accountName}" 已删除`);
      reload();
    } catch (err) {
      notice.error(err, '');
    }
  };

  return (
    <Card
      title={`账户管理(${accounts.length})`}
      extra={
        <Button type="primary" disabled={!cidNumber} onClick={() => setCreateOpen(true)}>
          + 新建账户
        </Button>
      }
    >
      <AccountList accounts={accounts} loading={loading} canDelete onDelete={onDeleteAccount} />
      <CreateAccountModal
        auth={auth}
        cidNumber={cidNumber}
        cidFullName={cidFullName}
        existingAccounts={accounts}
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onCreated={() => {
          setCreateOpen(false);
          reload();
        }}
      />
      {scanSignModal}
    </Card>
  );
}
