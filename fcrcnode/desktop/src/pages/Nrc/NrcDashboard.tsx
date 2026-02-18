import { Card, CardContent, Stack, Typography } from '@mui/material';
import { AccountBalanceCard } from '../../features/monitor/AccountBalanceCard';
import { RecentTransactionsCard } from '../../features/monitor/RecentTransactionsCard';

export function NrcDashboard() {
  return (
    <Stack spacing={2}>
      <Card>
        <CardContent>
          <Stack spacing={1}>
            <Typography variant="h6">国储会工作台</Typography>
            <Typography variant="body2" color="text.secondary">
              下一步将接入：超级管理员提案、状态转换升级提案、内部投票交易。
            </Typography>
          </Stack>
        </CardContent>
      </Card>
      <AccountBalanceCard />
      <RecentTransactionsCard />
    </Stack>
  );
}
