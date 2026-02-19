import { Alert, Button, Card, Input, Space, Typography } from 'antd';
import { useEffect, useState } from 'react';
import { readAccountBalance } from '../../services/rpc/polkadot';
import { useAuthStore } from '../../stores/auth';
import { useSessionStore } from '../../stores/session';
import { isValidAddress } from '../../utils/address';

function fenToYuan(fen: string): string {
  const value = BigInt(fen);
  const integer = value / 100n;
  const decimal = (value % 100n).toString().padStart(2, '0');
  return `${integer.toString()}.${decimal}`;
}

export function AccountBalanceCard() {
  const endpoint = useSessionStore((state) => state.endpoint);
  const session = useAuthStore((state) => state.session);

  const [address, setAddress] = useState(session?.publicKey ?? '');
  const [balanceFen, setBalanceFen] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const queryAddress = address.trim();
  const invalidAddress = queryAddress.length > 0 && !isValidAddress(queryAddress);

  useEffect(() => {
    if (session?.publicKey) {
      setAddress(session.publicKey);
    }
  }, [session?.publicKey]);

  const query = async () => {
    try {
      setError(null);
      const value = await readAccountBalance(endpoint, queryAddress);
      setBalanceFen(value);
    } catch (e) {
      setBalanceFen(null);
      setError(e instanceof Error ? e.message : '余额查询失败');
    }
  };

  return (
    <Card>
      <Space direction="vertical" size={12} style={{ width: '100%' }}>
        <Typography.Title level={5} style={{ margin: 0 }}>
          账户余额查询
        </Typography.Title>
        <div>
          <Input
            value={address}
            onChange={(event) => setAddress(event.target.value)}
            placeholder="账户地址"
            status={invalidAddress ? 'error' : ''}
          />
          {invalidAddress ? <Typography.Text type="danger">地址格式不合法（SS58）</Typography.Text> : null}
        </div>
        <Space wrap>
          <Button type="primary" onClick={query} disabled={!queryAddress || invalidAddress}>
            查询余额
          </Button>
          {balanceFen ? (
            <Typography.Text type="secondary">
              余额：{balanceFen} 分（{fenToYuan(balanceFen)} 元）
            </Typography.Text>
          ) : null}
        </Space>
        {error ? <Alert type="error" showIcon message={error} /> : null}
      </Space>
    </Card>
  );
}
