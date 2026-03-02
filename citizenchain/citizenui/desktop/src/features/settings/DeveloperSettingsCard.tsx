import { Button, Collapse, Input, Space, Typography } from 'antd';
import { type ChangeEvent, useEffect, useState } from 'react';
import { useSessionStore } from '../../stores/session';
import { assertSafeLocalRpcEndpoint } from '../../utils/rpcEndpoint';

export function DeveloperSettingsCard() {
  const endpoint = useSessionStore((state) => state.endpoint);
  const setEndpoint = useSessionStore((state) => state.setEndpoint);
  const [draft, setDraft] = useState(endpoint);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setDraft(endpoint);
  }, [endpoint]);

  return (
    <Collapse
      items={[
        {
          key: 'rpc',
          label: <Typography.Text>开发设置（RPC 地址）</Typography.Text>,
          children: (
            <Space direction="vertical" size={10} style={{ width: '100%' }}>
              <Input
                value={draft}
                onChange={(event: ChangeEvent<HTMLInputElement>) => setDraft(event.target.value)}
                placeholder="RPC Endpoint"
              />
              <Button
                onClick={() => {
                  try {
                    const next = assertSafeLocalRpcEndpoint(draft);
                    setEndpoint(next);
                    setError(null);
                  } catch (e) {
                    setError(e instanceof Error ? e.message : 'RPC 地址格式不合法');
                  }
                }}
              >
                应用
              </Button>
              {error ? <Typography.Text type="danger">{error}</Typography.Text> : null}
            </Space>
          )
        }
      ]}
    />
  );
}
