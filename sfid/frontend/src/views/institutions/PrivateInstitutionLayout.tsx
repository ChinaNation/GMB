// 中文注释:私权机构详情页三板块布局 — 自治模块。
// 上=机构信息,中=账户列表,下=资料库。
// 仅被 InstitutionDetailPage 在 category=PRIVATE_INSTITUTION 时调用。
// 修改私权机构布局只需改本文件,不影响公安局/公权机构详情页。

import React, { useState } from 'react';
import { Button, Card, Descriptions, Typography } from 'antd';
import type { AdminAuth } from '../../api/client';
import { A3_LABEL, INSTITUTION_CODE_LABEL } from './locks';
import type { InstitutionDetail } from '../../api/institution';
import { AccountList } from './AccountList';
import { CreateAccountModal } from './CreateAccountModal';
import { DocumentLibrary } from './DocumentLibrary';

// 企业类型 label 映射
const SUB_TYPE_LABEL: Record<string, string> = {
  SOLE_PROPRIETORSHIP: '个人独资',
  PARTNERSHIP: '合伙企业',
  LIMITED_LIABILITY: '有限责任',
  JOINT_STOCK: '股份公司',
};

interface Props {
  auth: AdminAuth;
  detail: InstitutionDetail;
  canWrite: boolean;
  loading: boolean;
  onReload: () => void;
  onDeleteAccount: (accountName: string) => void;
}

export const PrivateInstitutionLayout: React.FC<Props> = ({
  auth,
  detail,
  canWrite,
  loading,
  onReload,
  onDeleteAccount,
}) => {
  const inst = detail.institution;
  const accounts = detail.accounts;
  const [createAccountOpen, setCreateAccountOpen] = useState(false);

  return (
    <>
      {/* 上:机构信息 */}
      <Card
        title={<span style={{ fontSize: 18, fontWeight: 600 }}>{inst.institution_name}</span>}
        style={{ marginBottom: 16 }}
      >
        <Descriptions column={1} size="small">
          <Descriptions.Item label="机构 SFID">
            <Typography.Text code style={{ fontSize: 12, wordBreak: 'break-all' }}>
              {inst.sfid_id}
            </Typography.Text>
          </Descriptions.Item>
          <Descriptions.Item label="省份">{inst.province}</Descriptions.Item>
          <Descriptions.Item label="城市">{inst.city}</Descriptions.Item>
          <Descriptions.Item label="A3 类型">{inst.a3}/{A3_LABEL[inst.a3] || inst.a3}</Descriptions.Item>
          {inst.sub_type && (
            <Descriptions.Item label="企业类型">
              {SUB_TYPE_LABEL[inst.sub_type] || inst.sub_type}
            </Descriptions.Item>
          )}
          <Descriptions.Item label="机构代码">{inst.institution_code}/{INSTITUTION_CODE_LABEL[inst.institution_code] || inst.institution_code}</Descriptions.Item>
          <Descriptions.Item label="创建时间">
            {new Date(inst.created_at).toLocaleString('zh-CN')}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {/* 中:账户列表 */}
      <Card
        type="inner"
        title={`账户列表(${accounts.length})`}
        extra={
          canWrite && (
            <Button type="primary" onClick={() => setCreateAccountOpen(true)}>
              + 新建账户
            </Button>
          )
        }
        style={{ marginBottom: 16 }}
      >
        <AccountList
          accounts={accounts}
          loading={loading}
          canDelete={canWrite}
          onDelete={onDeleteAccount}
        />
      </Card>

      {/* 下:资料库(自治模块) */}
      <DocumentLibrary auth={auth} sfidId={inst.sfid_id} canWrite={canWrite} />

      <CreateAccountModal
        auth={auth}
        sfidId={inst.sfid_id}
        institutionName={inst.institution_name}
        existingAccounts={accounts}
        open={createAccountOpen}
        onCancel={() => setCreateAccountOpen(false)}
        onCreated={() => {
          setCreateAccountOpen(false);
          onReload();
        }}
      />
    </>
  );
};
