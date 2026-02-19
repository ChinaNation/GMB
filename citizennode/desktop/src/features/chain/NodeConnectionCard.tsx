import { Alert, Button, Card, Input, Space, Tag, Typography } from 'antd';
import { useState } from 'react';
import { connectNode, readChainHead } from '../../services/rpc/polkadot';
import { useSessionStore } from '../../stores/session';

export function NodeConnectionCard() {
  const { endpoint, state, error, setEndpoint, setState } = useSessionStore();
  const [head, setHead] = useState<number | null>(null);

  const connect = async () => {
    try {
      setState('connecting');
      await connectNode(endpoint);
      const block = await readChainHead(endpoint);
      setHead(block);
      setState('connected');
    } catch (e) {
      const message = e instanceof Error ? e.message : '连接失败';
      setState('error', message);
    }
  };

  const color = state === 'connected' ? 'success' : state === 'error' ? 'error' : 'default';

  return (
    <Card>
      <Space direction="vertical" size={12} style={{ width: '100%' }}>
        <Space wrap>
          <Typography.Title level={5} style={{ margin: 0 }}>
            节点连接
          </Typography.Title>
          <Tag color={color}>{state}</Tag>
        </Space>

        <Input value={endpoint} onChange={(event) => setEndpoint(event.target.value)} placeholder="RPC Endpoint" />

        <Space wrap>
          <Button type="primary" onClick={connect} disabled={state === 'connecting'}>
            {state === 'connecting' ? '连接中...' : '连接节点'}
          </Button>
          <Typography.Text type="secondary">最新区块: {head ?? '-'}</Typography.Text>
        </Space>

        {error ? <Alert type="error" showIcon message={error} /> : null}
      </Space>
    </Card>
  );
}
