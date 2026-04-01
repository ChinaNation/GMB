import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './auth';
import * as api from './api';
import ProtectedRoute from './common/ProtectedRoute';
import NotFound from './common/NotFound';
import LoginPage from './login/LoginPage';
import InstallPage from './install/InstallPage';
import AdminLayout from './admin/AdminLayout';
import OperatorList from './admin/OperatorList';
import SystemSettings from './admin/SystemSettings';
import ArchiveList from './operator/ArchiveList';
import ArchiveCreate from './operator/ArchiveCreate';
import ArchiveDetail from './operator/ArchiveDetail';

function RootRedirect() {
  const { user } = useAuth();
  const [checking, setChecking] = useState(true);
  const [initialized, setInitialized] = useState(true);

  useEffect(() => {
    api.installStatus().then(res => {
      const data = res.data;
      if (!data || !data.initialized || data.super_admin_bound_count < 1) {
        setInitialized(false);
      }
    }).catch(() => {
      setInitialized(false);
    }).finally(() => setChecking(false));
  }, []);

  if (checking) return null;
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
              {/* 管理员（仅超管） */}
              <Route path="/admin/operators" element={<OperatorList />} />
              {/* 系统设置（仅超管） */}
              <Route path="/admin/settings" element={<SystemSettings />} />
            </Route>
          </Route>

          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}
