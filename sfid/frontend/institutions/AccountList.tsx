// 中文注释:某机构下的账户列表。
//
// SFID 账户列表只展示链上同步状态,不提供后台手动激活入口。
// 链上注册/注销由区块链软件完成后同步回 SFID。

import React from 'react';
import { Button, Popconfirm, Space, Table, Tag } from 'antd';
import {
  type MultisigAccount,
  type MultisigChainStatus,
} from './api';
import { tryEncodeSs58 } from '../utils/ss58';

/** 默认账户名称(与后端 `service::DEFAULT_ACCOUNT_NAMES` 对齐) */
const DEFAULT_ACCOUNT_NAMES = ['主账户', '费用账户'] as const;

interface Props {
  accounts: MultisigAccount[];
  loading: boolean;
  canDelete: boolean;
  onDelete: (accountName: string) => void;
}

const STATUS_LABEL: Record<MultisigChainStatus, string> = {
  NOT_ON_CHAIN: '未上链',
  PENDING_ON_CHAIN: '上链中',
  ACTIVE_ON_CHAIN: '已上链',
  REVOKED_ON_CHAIN: '已注销',
};

const STATUS_COLOR: Record<MultisigChainStatus, string> = {
  NOT_ON_CHAIN: 'default',
  PENDING_ON_CHAIN: 'orange',
  ACTIVE_ON_CHAIN: 'green',
  REVOKED_ON_CHAIN: 'purple',
};

export const AccountList: React.FC<Props> = ({
  accounts,
  loading,
  canDelete,
  onDelete,
}) => {
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
            const canDeleteRow =
              canDelete &&
              !isDefault &&
              (row.chain_status === 'NOT_ON_CHAIN' || row.chain_status === 'REVOKED_ON_CHAIN');
            // 删除按钮:默认账户永不显示;上链中/已上链账户不能删除。
            const deleteCell =
              canDeleteRow ? (
                <Popconfirm
                  title={`确认删除账户 "${row.account_name}"?`}
                  description="仅删除 SFID 系统中的账户名称记录,不触发链上操作"
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
            if (deleteCell) return <Space size={4}>{deleteCell}</Space>;
            if (!isDefault && row.chain_status === 'ACTIVE_ON_CHAIN') {
              return <span style={{ color: '#999', fontSize: 12 }}>链上账户不可删</span>;
            }
            return <span style={{ color: '#999', fontSize: 12 }}>-</span>;
          },
        },
      ]}
    />
  );
};
