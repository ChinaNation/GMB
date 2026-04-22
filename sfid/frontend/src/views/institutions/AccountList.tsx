// 中文注释:某机构下的账户列表。
//
// 2026-04-21 统一两步激活模式:
//   - 所有账户(默认 2 个 + 管理员手工建)创建时都是 `INACTIVE`,不上链
//   - 操作栏显示"激活"按钮:
//       INACTIVE   → [激活]     可点,触发推链
//       PENDING    → [激活中…]  禁用
//       REGISTERED → 已激活       文本(不可操作)
//       FAILED     → [重试激活]   可点,重新推链
//   - 默认账户(主账户 / 费用账户)**不可删除**,无删除按钮
//   - 管理员手工创建的其他账户仍可软删(不触链)

import React, { useState } from 'react';
import { Button, Popconfirm, Space, Table, Tag, message } from 'antd';
import {
  activateAccount,
  type MultisigAccount,
  type MultisigChainStatus,
} from '../../api/institution';
import type { AdminAuth } from '../../api/client';
import { tryEncodeSs58 } from '../../utils/ss58';

/** 默认账户名称(与后端 `service::DEFAULT_ACCOUNT_NAMES` 对齐) */
const DEFAULT_ACCOUNT_NAMES = ['主账户', '费用账户'] as const;

interface Props {
  auth: AdminAuth;
  sfidId: string;
  accounts: MultisigAccount[];
  loading: boolean;
  canDelete: boolean;
  onDelete: (accountName: string) => void;
  /** 激活成功后触发父组件刷新详情(重新拉 accounts) */
  onActivated?: () => void;
}

const STATUS_LABEL: Record<MultisigChainStatus, string> = {
  INACTIVE: '未激活',
  PENDING: '激活中',
  REGISTERED: '已激活',
  FAILED: '激活失败',
};

const STATUS_COLOR: Record<MultisigChainStatus, string> = {
  INACTIVE: 'default',
  PENDING: 'orange',
  REGISTERED: 'green',
  FAILED: 'red',
};

export const AccountList: React.FC<Props> = ({
  auth,
  sfidId,
  accounts,
  loading,
  canDelete,
  onDelete,
  onActivated,
}) => {
  const [activating, setActivating] = useState<string | null>(null);

  const handleActivate = async (accountName: string) => {
    setActivating(accountName);
    try {
      await activateAccount(auth, sfidId, accountName);
      message.success(`账户 "${accountName}" 已激活`);
      onActivated?.();
    } catch (err) {
      const raw = err instanceof Error ? err.message : '激活失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        message.error('本省登录管理员未在线,请联系省管理员登录后重试');
      } else if (raw.includes('密钥管理员不能直接推送')) {
        message.error('请以省或市管理员身份操作');
      } else {
        message.error(raw);
      }
    } finally {
      setActivating(null);
    }
  };

  return (
    <Table<MultisigAccount>
      rowKey={(r) => `${r.sfid_id}|${r.account_name}`}
      loading={loading}
      dataSource={accounts}
      pagination={false}
      columns={[
        { title: '账户名称', dataIndex: 'account_name', width: 200 },
        {
          title: '账户地址',
          dataIndex: 'duoqian_address',
          render: (v: string | null) => {
            if (!v) return '-';
            const ss58 = tryEncodeSs58(v);
            return (
              <span style={{ fontSize: 11, fontFamily: 'monospace', wordBreak: 'break-all' }}>
                {ss58.slice(0, 12)}...{ss58.slice(-8)}
              </span>
            );
          },
        },
        {
          title: '链上状态',
          dataIndex: 'chain_status',
          width: 120,
          render: (v: MultisigChainStatus) => (
            <Tag color={STATUS_COLOR[v] || 'default'}>{STATUS_LABEL[v] || v}</Tag>
          ),
        },
        {
          title: '交易哈希',
          dataIndex: 'chain_tx_hash',
          render: (v: string | null) =>
            v ? (
              <span style={{ fontSize: 11, fontFamily: 'monospace', wordBreak: 'break-all' }}>
                {v.slice(0, 14)}...{v.slice(-8)}
              </span>
            ) : (
              '-'
            ),
        },
        {
          title: '创建时间',
          dataIndex: 'created_at',
          width: 170,
          render: (v: string) => new Date(v).toLocaleString('zh-CN'),
        },
        {
          title: '操作',
          width: 160,
          align: 'center',
          render: (_v, row) => {
            const isDefault = (DEFAULT_ACCOUNT_NAMES as readonly string[]).includes(row.account_name);
            const busy = activating === row.account_name;
            // 激活按钮(所有账户统一逻辑,按 chain_status 分支)
            let activateCell: React.ReactNode;
            switch (row.chain_status) {
              case 'INACTIVE':
                activateCell = (
                  <Button
                    type="primary"
                    size="small"
                    loading={busy}
                    onClick={() => handleActivate(row.account_name)}
                  >
                    激活
                  </Button>
                );
                break;
              case 'PENDING':
                activateCell = (
                  <Button size="small" disabled>
                    激活中…
                  </Button>
                );
                break;
              case 'FAILED':
                activateCell = (
                  <Button
                    danger
                    size="small"
                    loading={busy}
                    onClick={() => handleActivate(row.account_name)}
                  >
                    重试激活
                  </Button>
                );
                break;
              case 'REGISTERED':
                activateCell = <span style={{ color: '#52c41a', fontSize: 12 }}>已激活</span>;
                break;
              default:
                activateCell = null;
            }
            // 删除按钮:默认账户永不显示;其他账户按 canDelete
            const deleteCell =
              !isDefault && canDelete ? (
                <Popconfirm
                  title={`确认删除账户 "${row.account_name}"?`}
                  description="仅删除 sfid 系统记录,不触发链上操作"
                  onConfirm={() => onDelete(row.account_name)}
                  okText="删除"
                  okButtonProps={{ danger: true }}
                  cancelText="取消"
                >
                  <Button size="small" danger type="link">
                    删除
                  </Button>
                </Popconfirm>
              ) : null;
            return (
              <Space size={4}>
                {activateCell}
                {deleteCell}
              </Space>
            );
          },
        },
      ]}
    />
  );
};
