import { Alert, Button, Card, Input, Space, Table, Typography } from 'antd';
import { useState } from 'react';
import { readRecentTransactions, type RecentTransaction } from '../../services/rpc/polkadot';
import { useAuthStore } from '../../stores/auth';
import { useSessionStore } from '../../stores/session';
import { isValidAddress } from '../../utils/address';

export function RecentTransactionsCard() {
  const endpoint = useSessionStore((state) => state.endpoint);
  const session = useAuthStore((state) => state.session);

  const [addressFilter, setAddressFilter] = useState(session?.publicKey ?? '');
  const [rows, setRows] = useState<RecentTransaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const filter = addressFilter.trim();
  const invalidFilter = filter.length > 0 && !isValidAddress(filter);

  const query = async () => {
    try {
      setLoading(true);
      setError(null);
      const list = await readRecentTransactions(endpoint, {
        address: filter || undefined,
        depth: 40,
        limit: 25
      });
      setRows(list);
    } catch (e) {
      setRows([]);
      setError(e instanceof Error ? e.message : '交易查询失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <Space direction="vertical" size={12} style={{ width: '100%' }}>
        <Typography.Title level={5} style={{ margin: 0 }}>
          最近区块交易
        </Typography.Title>

        <Space wrap style={{ width: '100%' }}>
          <div style={{ minWidth: 320, flex: 1 }}>
            <Input
              value={addressFilter}
              onChange={(event) => setAddressFilter(event.target.value)}
              placeholder="地址过滤（可选）"
              status={invalidFilter ? 'error' : ''}
            />
            <Typography.Text type={invalidFilter ? 'danger' : 'secondary'}>
              {invalidFilter ? '地址格式不合法（SS58）' : '留空则显示全量交易'}
            </Typography.Text>
          </div>
          <Button type="primary" onClick={query} disabled={loading || invalidFilter}>
            {loading ? '查询中...' : '查询交易'}
          </Button>
        </Space>

        <Table<RecentTransaction>
          size="small"
          rowKey={(row) => `${row.blockNumber}-${row.extrinsicIndex}-${row.hash}`}
          dataSource={rows}
          pagination={false}
          locale={{ emptyText: '暂无数据，点击“查询交易”开始加载。' }}
          columns={[
            { title: '区块', dataIndex: 'blockNumber', render: (v) => `#${v}` },
            {
              title: '调用',
              render: (_, row) => `${row.section}.${row.method}`
            },
            { title: '签名者', dataIndex: 'signer', render: (v) => v ?? 'unsigned' },
            { title: '哈希', dataIndex: 'hash', render: (v) => `${v.slice(0, 14)}...` }
          ]}
        />

        {error ? <Alert type="error" showIcon message={error} /> : null}
      </Space>
    </Card>
  );
}
