import { useState } from 'react';
import { Alert, Button, Card, CardContent, Chip, Stack, TextField, Typography } from '@mui/material';
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

  return (
    <Card sx={{ backgroundColor: 'rgba(255, 255, 255, 0.04)' }}>
      <CardContent>
        <Stack spacing={2}>
          <Stack direction="row" spacing={1.5} alignItems="center">
            <Typography variant="h6">节点连接</Typography>
            <Chip label={state} color={state === 'connected' ? 'success' : state === 'error' ? 'error' : 'default'} />
          </Stack>

          <TextField
            label="RPC Endpoint"
            value={endpoint}
            onChange={(event) => setEndpoint(event.target.value)}
            fullWidth
            size="small"
          />

          <Stack direction="row" spacing={1.5} alignItems="center">
            <Button variant="contained" color="warning" onClick={connect} disabled={state === 'connecting'}>
              {state === 'connecting' ? '连接中...' : '连接节点'}
            </Button>
            <Typography variant="body2" color="text.secondary">
              最新区块: {head ?? '-'}
            </Typography>
          </Stack>

          {error ? <Alert severity="error">{error}</Alert> : null}
        </Stack>
      </CardContent>
    </Card>
  );
}
