import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './authz/AuthProvider';
import { installStatus } from './initialize/api';
import ProtectedRoute from './authz/ProtectedRoute';
import NotFound from './common/NotFound';
import LoginPage from './login/LoginPage';
import InstallPage from './initialize/InstallPage';
import AdminLayout from './super_admin/AdminLayout';
import AdminList from './super_admin/AdminList';
import SystemSettings from './super_admin/SystemSettings';
import ArchiveList from './operator_admin/ArchiveList';
import ArchiveCreate from './operator_admin/ArchiveCreate';
import ArchiveDetail from './operator_admin/ArchiveDetail';

function RootRedirect() {
  const { ready, user } = useAuth();
  const [checking, setChecking] = useState(true);
  const [initialized, setInitialized] = useState(true);

  useEffect(() => {
    installStatus().then(res => {
      const data = res.data;
      if (!data || !data.initialized || data.super_admin_bound_count < 1) {
        setInitialized(false);
      }
    }).catch(() => {
      setInitialized(false);
    }).finally(() => setChecking(false));
  }, []);

  if (checking || !ready) return null;
  if (!initialized) return <Navigate to="/install" replace />;
  if (!user) return <Navigate to="/login" replace />;
  return <Navigate to="/admin" replace />;
}

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<RootRedirect />} />
          <Route path="/login" element={<LoginPage />} />
          <Route path="/install" element={<InstallPage />} />

          <Route element={<ProtectedRoute role="SUPER_ADMIN,OPERATOR_ADMIN" />}>
            <Route element={<AdminLayout />}>
              {/* 首页：公民信息 */}
              <Route path="/admin" element={<ArchiveList />} />
              <Route path="/admin/create" element={<ArchiveCreate />} />
              <Route path="/admin/archives/:id" element={<ArchiveDetail />} />
              <Route element={<ProtectedRoute role="SUPER_ADMIN" />}>
                {/* 管理员与系统设置仅超级管理员可访问 */}
                <Route path="/admin/admins" element={<AdminList />} />
                <Route path="/admin/settings" element={<SystemSettings />} />
              </Route>
            </Route>
          </Route>

          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}
