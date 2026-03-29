// 路由守卫：未登录跳转登录页，角色不匹配跳转 404

import { Navigate, Outlet } from 'react-router-dom';
import { useAuth } from '../auth';

export default function ProtectedRoute({ role }: { role?: string }) {
  const { token, user } = useAuth();
  if (!token || !user) return <Navigate to="/login" replace />;
  if (role && user.role !== role) return <Navigate to="/404" replace />;
  return <Outlet />;
}
