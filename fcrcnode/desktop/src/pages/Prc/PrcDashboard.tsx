import { Card, CardContent, Stack, Typography } from '@mui/material';
import { AccountBalanceCard } from '../../features/monitor/AccountBalanceCard';
import { RecentTransactionsCard } from '../../features/monitor/RecentTransactionsCard';

export function PrcDashboard() {
  return (
    <Stack spacing={2}>
      <Card>
        <CardContent>
          <Stack spacing={1}>
            <Typography variant="h6">省储会工作台</Typography>
            <Typography variant="body2" color="text.secondary">
              下一步将接入：升级投票、决议发行投票、交易记录查询。
            </Typography>
          </Stack>
        </CardContent>
      </Card>
      <AccountBalanceCard />
      <RecentTransactionsCard />
    </Stack>
  );
}
