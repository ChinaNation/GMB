import { Button, Card, Layout, Space, Tag, Typography } from 'antd';
import { Navigate, Route, Routes, useNavigate } from 'react-router-dom';
import { AntdLoginPreview } from './features/auth/AntdLoginPreview';
import { RoleGate } from './features/auth/RoleGate';
import { NrcDashboard } from './pages/Nrc/NrcDashboard';
import { PrcDashboard } from './pages/Prc/PrcDashboard';
import { PrbDashboard } from './pages/Prb/PrbDashboard';
import { FullDashboard } from './pages/Full/FullDashboard';
import { useAuthStore } from './stores/auth';
import { getOrganizationName } from './utils/organization';

const { Content } = Layout;

export default function App() {
  const navigate = useNavigate();
  const session = useAuthStore((state) => state.session);
  const logout = useAuthStore((state) => state.logout);
  const homePath = session ? `/${session.role}` : '/';

  if (!session) {
    return <AntdLoginPreview />;
  }

  return (
    <Layout style={{ minHeight: '100vh', background: 'transparent' }}>
      <Content style={{ maxWidth: 1180, width: '100%', margin: '0 auto', padding: '24px 20px' }}>
        <Space direction="vertical" size={16} style={{ width: '100%' }}>
          <Card>
            <Space direction="vertical" size={8} style={{ width: '100%' }}>
              <Typography.Title level={3} style={{ margin: 0 }}>
                公民护照管理系统
              </Typography.Title>
              <Typography.Text type="secondary">CPMS 本地管理端（离线/局域网）</Typography.Text>
              <Space>
                <Tag color="gold">{getOrganizationName(session)}</Tag>
                <Button
                  onClick={() => {
                    logout();
                    navigate('/');
                  }}
                >
                  退出登录
                </Button>
              </Space>
            </Space>
          </Card>

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
            <Route
              path="/full"
              element={
                <RoleGate role="full">
                  <FullDashboard />
                </RoleGate>
              }
            />
            <Route path="*" element={<Navigate to={homePath} replace />} />
          </Routes>
        </Space>
      </Content>
    </Layout>
  );
}
