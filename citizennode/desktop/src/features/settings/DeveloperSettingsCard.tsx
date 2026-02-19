import { Button, Collapse, Input, Space, Typography } from 'antd';
import { useState } from 'react';
import { useSessionStore } from '../../stores/session';

export function DeveloperSettingsCard() {
  const endpoint = useSessionStore((state) => state.endpoint);
  const setEndpoint = useSessionStore((state) => state.setEndpoint);
  const [draft, setDraft] = useState(endpoint);

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
                onChange={(event) => setDraft(event.target.value)}
                placeholder="RPC Endpoint"
              />
              <Button
                onClick={() => {
                  const next = draft.trim();
                  if (next) setEndpoint(next);
                }}
              >
                应用
              </Button>
            </Space>
          )
        }
      ]}
    />
  );
}
