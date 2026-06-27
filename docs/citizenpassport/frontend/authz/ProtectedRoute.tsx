// 路由守卫：未登录跳转登录页，用户分组不匹配跳转 404

import { Navigate, Outlet } from 'react-router-dom';
import { useAuth } from './AuthProvider';

export default function ProtectedRoute({ user_group }: { user_group?: string }) {
  const { ready, user } = useAuth();
  if (!ready) return null;
  if (!user) return <Navigate to="/login" replace />;
  if (user_group && !user_group.split(',').map(r => r.trim()).includes(user.user_group)) return <Navigate to="/404" replace />;
  return <Outlet />;
}
