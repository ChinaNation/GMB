// 中文注释:某机构下的账户列表。展示状态 + 链上地址 + 删除按钮。
// 删除只是软删(sfid 系统记录),不触链。

import React from 'react';
import { Button, Popconfirm, Table, Tag } from 'antd';
import type { MultisigAccount, MultisigChainStatus } from '../../api/institution';
import { tryEncodeSs58 } from '../../utils/ss58';

interface Props {
  accounts: MultisigAccount[];
  loading: boolean;
  canDelete: boolean;
  onDelete: (accountName: string) => void;
}

const STATUS_LABEL: Record<MultisigChainStatus, string> = {
  PENDING: '等待上链',
  REGISTERED: '已注册',
  FAILED: '失败',
};

const STATUS_COLOR: Record<MultisigChainStatus, string> = {
  PENDING: 'orange',
  REGISTERED: 'green',
  FAILED: 'red',
};

export const AccountList: React.FC<Props> = ({ accounts, loading, canDelete, onDelete }) => {
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
        canDelete
          ? {
              title: '操作',
              width: 100,
              align: 'center',
              render: (_v, row) => (
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
              ),
            }
          : { title: '操作', width: 1, render: () => null },
      ]}
    />
  );
};
