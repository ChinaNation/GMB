import { useState } from 'react';
import {
  Alert,
  Button,
  Card,
  CardContent,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TextField,
  Typography
} from '@mui/material';
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
      <CardContent>
        <Stack spacing={1.5}>
          <Typography variant="h6">最近区块交易</Typography>
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
            <TextField
              label="地址过滤（可选）"
              value={addressFilter}
              onChange={(event) => setAddressFilter(event.target.value)}
              size="small"
              fullWidth
              error={invalidFilter}
              helperText={invalidFilter ? '地址格式不合法（SS58）' : '留空则显示全量交易'}
            />
            <Button variant="contained" onClick={query} disabled={loading || invalidFilter}>
              {loading ? '查询中...' : '查询交易'}
            </Button>
          </Stack>

          {rows.length > 0 ? (
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>区块</TableCell>
                  <TableCell>调用</TableCell>
                  <TableCell>签名者</TableCell>
                  <TableCell>哈希</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {rows.map((row) => (
                  <TableRow key={`${row.blockNumber}-${row.extrinsicIndex}-${row.hash}`}>
                    <TableCell>#{row.blockNumber}</TableCell>
                    <TableCell>{row.section + '.' + row.method}</TableCell>
                    <TableCell>{row.signer ?? 'unsigned'}</TableCell>
                    <TableCell>{row.hash.slice(0, 14) + '...'}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <Typography variant="body2" color="text.secondary">
              暂无数据，点击“查询交易”开始加载。
            </Typography>
          )}

          {error ? <Alert severity="error">{error}</Alert> : null}
        </Stack>
      </CardContent>
    </Card>
  );
}
