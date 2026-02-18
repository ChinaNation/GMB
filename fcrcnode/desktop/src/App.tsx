import { Box, Chip, Container, Stack, Typography } from '@mui/material';
import { Navigate, Route, Routes } from 'react-router-dom';
import { LoginCard } from './features/auth/LoginCard';
import { NodeStatusBanner } from './features/chain/NodeStatusBanner';
import { useAutoConnect } from './features/chain/useAutoConnect';
import { RoleGate } from './features/auth/RoleGate';
import { DeveloperSettingsCard } from './features/settings/DeveloperSettingsCard';
import { NrcDashboard } from './pages/Nrc/NrcDashboard';
import { PrcDashboard } from './pages/Prc/PrcDashboard';
import { PrbDashboard } from './pages/Prb/PrbDashboard';
import { useAuthStore } from './stores/auth';
import { getOrganizationName } from './utils/organization';

export default function App() {
  useAutoConnect();
  const session = useAuthStore((state) => state.session);
  const homePath = session ? `/${session.role}` : '/';

  if (!session) {
    return (
      <Container maxWidth="md" sx={{ py: 8, minHeight: '100vh', display: 'flex', alignItems: 'center' }}>
        <Stack spacing={2} justifyContent="center" sx={{ width: '100%' }}>
          <LoginCard />
        </Stack>
      </Container>
    );
  }

  return (
    <Container maxWidth="lg" sx={{ py: 5 }}>
      <Stack spacing={3}>
        <Box>
          <Typography variant="h4" fontWeight={700}>
            公民储备委员会节点前端
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mt: 0.5 }}>
            登录后显示对应机构并进入专属工作台
          </Typography>
          <Chip sx={{ mt: 1.5 }} color="warning" label={getOrganizationName(session)} />
        </Box>

        <NodeStatusBanner />
        <DeveloperSettingsCard />

        <Routes>
          <Route
            path="/nrc"
            element={
              <RoleGate role="nrc">
                <NrcDashboard />
              </RoleGate>
            }
          />
          <Route
            path="/prc"
            element={
              <RoleGate role="prc">
                <PrcDashboard />
              </RoleGate>
            }
          />
          <Route
            path="/prb"
            element={
              <RoleGate role="prb">
                <PrbDashboard />
              </RoleGate>
            }
          />
          <Route path="*" element={<Navigate to={homePath} replace />} />
        </Routes>
      </Stack>
    </Container>
  );
}
