import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useAuth } from '../auth';
import * as api from '../api';
import { QrIcon } from '../components/QrIcon';

export default function AdminLayout() {
  const { user, logout } = useAuth();
  const location = useLocation();
  const navigate = useNavigate();
  const isSuperAdmin = user?.role === 'SUPER_ADMIN';

  const handleLogout = async () => {
    try { await api.authLogout(); } catch { /* ignore */ }
    logout();
    navigate('/login');
  };

  const tabs = [
    { key: '/admin', label: '首页', visible: true },
    { key: '/admin/operators', label: '管理员', visible: isSuperAdmin },
    { key: '/admin/settings', label: '系统设置', visible: isSuperAdmin },
  ];

  const activeTab = (() => {
    // 精确匹配或前缀匹配（/admin/create, /admin/archives/:id 都归首页）
    if (location.pathname.startsWith('/admin/operators')) return '/admin/operators';
    if (location.pathname.startsWith('/admin/settings')) return '/admin/settings';
    return '/admin';
  })();

  return (
    <div style={{
      minHeight: '100vh',
      background: 'linear-gradient(145deg, #0f172a 0%, #134e4a 40%, #0f766e 70%, #115e59 100%)',
      backgroundAttachment: 'fixed',
      position: 'relative',
    }}>
      {/* 背景装饰层 */}
      <div style={{ position: 'fixed', inset: 0, pointerEvents: 'none', zIndex: 0, overflow: 'hidden' }}>
        <div style={{ position: 'absolute', top: '-20%', right: '-10%', width: '50vw', height: '50vw', borderRadius: '50%', background: 'radial-gradient(circle, rgba(13,148,136,0.25) 0%, transparent 70%)' }} />
        <div style={{ position: 'absolute', bottom: '-15%', left: '-10%', width: '45vw', height: '45vw', borderRadius: '50%', background: 'radial-gradient(circle, rgba(20,184,166,0.15) 0%, transparent 70%)' }} />
        <div style={{ position: 'absolute', inset: 0, backgroundImage: 'linear-gradient(rgba(255,255,255,0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.03) 1px, transparent 1px)', backgroundSize: '60px 60px' }} />
      </div>

      {/* Header */}
      <header style={{
        position: 'relative', zIndex: 1,
        background: 'linear-gradient(135deg, rgba(13,148,136,0.7) 0%, rgba(15,118,110,0.8) 100%)',
        backdropFilter: 'blur(12px)',
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        paddingInline: 32, height: 72,
        borderBottom: '1px solid rgba(255,255,255,0.15)',
        boxShadow: '0 2px 16px rgba(0,0,0,0.12)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{
            width: 44, height: 44, borderRadius: 10,
            background: 'rgba(255,255,255,0.18)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            border: '1px solid rgba(255,255,255,0.25)',
          }}>
            <QrIcon size={22} color="#fff" />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.2 }}>
            <span style={{ color: '#fff', fontSize: 20, fontWeight: 700, letterSpacing: 2 }}>中华民族联邦共和国</span>
            <span style={{ color: 'rgba(255,255,255,0.8)', fontSize: 13, fontWeight: 500, letterSpacing: 4, marginTop: 2 }}>护照管理系统</span>
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <span style={{
            color: '#fff', fontSize: 14, fontWeight: 500,
            background: 'rgba(255,255,255,0.12)',
            padding: '6px 16px', borderRadius: 8,
            border: '1px solid rgba(255,255,255,0.15)',
          }}>
            {isSuperAdmin ? '超级管理员' : '系统管理员'}
          </span>
          <button
            onClick={handleLogout}
            style={{
              background: 'rgba(255,255,255,0.1)',
              border: '1px solid rgba(255,255,255,0.25)',
              color: '#fca5a5', fontWeight: 500, borderRadius: 8,
              padding: '6px 16px', cursor: 'pointer', fontSize: 13,
            }}
          >
            退出
          </button>
        </div>
      </header>

      {/* Tab 栏：普通管理员不显示（只有首页一个页面） */}
      {isSuperAdmin && (
      <div style={{ position: 'relative', zIndex: 1, padding: '12px 24px 0' }}>
        <div style={{
          display: 'flex', gap: 6, padding: '8px 12px',
          background: 'rgba(255,255,255,0.08)',
          backdropFilter: 'blur(12px)',
          borderRadius: 14,
          border: '1px solid rgba(255,255,255,0.1)',
          width: 'fit-content',
        }}>
          {tabs.filter(t => t.visible).map(tab => (
            <button
              key={tab.key}
              onClick={() => navigate(tab.key)}
              style={{
                padding: '8px 20px', borderRadius: 10, border: 'none',
                cursor: 'pointer', fontSize: 14, fontWeight: 500,
                transition: 'all 0.2s ease',
                ...(activeTab === tab.key
                  ? { background: 'linear-gradient(135deg, #0d9488, #0f766e)', color: '#fff', boxShadow: '0 2px 8px rgba(13,148,136,0.35)' }
                  : { background: 'transparent', color: 'rgba(255,255,255,0.7)' }),
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>
      )}

      {/* Content */}
      <div style={{ position: 'relative', zIndex: 1, padding: '16px 24px 24px' }}>
        <Outlet />
      </div>
    </div>
  );
}
