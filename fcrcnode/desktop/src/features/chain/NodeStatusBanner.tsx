import { Chip, Stack, Typography } from '@mui/material';
import { useSessionStore } from '../../stores/session';

export function NodeStatusBanner() {
  const { state, endpoint, error } = useSessionStore();

  return (
    <Stack spacing={1}>
      <Stack direction="row" spacing={1.5} alignItems="center">
        <Typography variant="body2">本机节点状态</Typography>
        <Chip
          size="small"
          label={state}
          color={state === 'connected' ? 'success' : state === 'error' ? 'error' : 'default'}
        />
        <Typography variant="caption" color="text.secondary">
          {endpoint}
        </Typography>
      </Stack>
      {state === 'error' ? (
        <Typography variant="body2" sx={{ color: '#ff8a80' }}>
          自动连接失败：{error}
        </Typography>
      ) : null}
    </Stack>
  );
}
