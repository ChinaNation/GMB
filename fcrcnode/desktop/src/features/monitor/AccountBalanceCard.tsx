import { useEffect, useState } from 'react';
import { Alert, Button, Card, CardContent, Stack, TextField, Typography } from '@mui/material';
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
      <CardContent>
        <Stack spacing={1.5}>
          <Typography variant="h6">账户余额查询</Typography>
          <TextField
            label="账户地址"
            value={address}
            onChange={(event) => setAddress(event.target.value)}
            size="small"
            fullWidth
            error={invalidAddress}
            helperText={invalidAddress ? '地址格式不合法（SS58）' : undefined}
          />
          <Stack direction="row" spacing={1.5}>
            <Button variant="contained" onClick={query} disabled={!queryAddress || invalidAddress}>
              查询余额
            </Button>
            {balanceFen ? (
              <Typography variant="body2" color="text.secondary">
                余额：{balanceFen} 分（{fenToYuan(balanceFen)} 元）
              </Typography>
            ) : null}
          </Stack>
          {error ? <Alert severity="error">{error}</Alert> : null}
        </Stack>
      </CardContent>
    </Card>
  );
}
