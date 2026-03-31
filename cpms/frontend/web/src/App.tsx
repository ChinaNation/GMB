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
import SiteKeyRegister from './admin/SiteKeyRegister';
import CitizenStatusEdit from './admin/CitizenStatusEdit';
import AnonCertScan from './admin/AnonCertScan';
import OperatorLayout from './operator/OperatorLayout';
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
      // 未初始化或管理员未绑定 → 去安装页
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
  return <Navigate to={user.role === 'SUPER_ADMIN' ? '/admin' : '/operator'} replace />;
}

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<RootRedirect />} />
          <Route path="/login" element={<LoginPage />} />
          <Route path="/install" element={<InstallPage />} />

          <Route element={<ProtectedRoute role="SUPER_ADMIN" />}>
            <Route element={<AdminLayout />}>
              <Route path="/admin" element={<OperatorList />} />
              <Route path="/admin/site-keys" element={<SiteKeyRegister />} />
              <Route path="/admin/citizen-status" element={<CitizenStatusEdit />} />
              <Route path="/admin/anon-cert" element={<AnonCertScan />} />
            </Route>
          </Route>

          <Route element={<ProtectedRoute role="OPERATOR_ADMIN" />}>
            <Route element={<OperatorLayout />}>
              <Route path="/operator" element={<ArchiveList />} />
              <Route path="/operator/create" element={<ArchiveCreate />} />
              <Route path="/operator/archives/:id" element={<ArchiveDetail />} />
            </Route>
          </Route>

          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}
