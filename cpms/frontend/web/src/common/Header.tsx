// 顶部状态栏

import { useAuth } from '../auth';
import { useNavigate } from 'react-router-dom';
import * as api from '../api';

export default function Header({ title }: { title: string }) {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    try { await api.authLogout(); } catch { /* ignore */ }
    logout();
    navigate('/login');
  };

  return (
    <header className="layout__header">
      <span className="header__title">{title}</span>
      <div className="header__user">
        <span>{user?.role === 'SUPER_ADMIN' ? '超级管理员' : '操作员'}</span>
        <span style={{ opacity: 0.5 }}>|</span>
        <span className="text-ellipsis" style={{ maxWidth: 120 }}>{user?.user_id}</span>
        <button className="btn btn--ghost btn--sm" onClick={handleLogout}>登出</button>
      </div>
    </header>
  );
}
