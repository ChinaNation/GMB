import { Alert, Button, Card, Input, Space, Tag, Typography } from 'antd';
import { type ChangeEvent, useEffect, useState } from 'react';
import { connectNode, readChainHead } from '../../services/rpc/polkadot';
import { useSessionStore } from '../../stores/session';
import { assertSafeLocalRpcEndpoint } from '../../utils/rpcEndpoint';

export function NodeConnectionCard() {
  const { endpoint, state, error, setEndpoint, setState } = useSessionStore();
  const [head, setHead] = useState<number | null>(null);
  const [draft, setDraft] = useState(endpoint);
  const [endpointError, setEndpointError] = useState<string | null>(null);

  useEffect(() => {
    setDraft(endpoint);
  }, [endpoint]);

  const connect = async () => {
    try {
      const nextEndpoint = assertSafeLocalRpcEndpoint(draft);
      setEndpoint(nextEndpoint);
      setEndpointError(null);
      setState('connecting');
      await connectNode(nextEndpoint);
      const block = await readChainHead(nextEndpoint);
      setHead(block);
      setState('connected');
    } catch (e) {
      const message = e instanceof Error ? e.message : '连接失败';
      setEndpointError(message);
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

        <Input
          value={draft}
          onChange={(event: ChangeEvent<HTMLInputElement>) => setDraft(event.target.value)}
          placeholder="RPC Endpoint"
        />
        <Typography.Text type="secondary">
          仅允许本地端点：ws://127.0.0.1:&lt;port&gt; 或 ws://localhost:&lt;port&gt;
        </Typography.Text>

        <Space wrap>
          <Button type="primary" onClick={connect} disabled={state === 'connecting'}>
            {state === 'connecting' ? '连接中...' : '连接节点'}
          </Button>
          <Typography.Text type="secondary">最新区块: {head ?? '-'}</Typography.Text>
        </Space>

        {endpointError ? <Alert type="error" showIcon message={endpointError} /> : null}
        {error && !endpointError ? <Alert type="error" showIcon message={error} /> : null}
      </Space>
    </Card>
  );
}
