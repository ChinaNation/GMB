// =============================================================================
// 中文文件头:App.tsx = 路由壳子 + Layout 壳子
// -----------------------------------------------------------------------------
// 本文件是 sfid-frontend 的顶层路由壳,职责仅限:
//   1. 挂 <AuthProvider>(AppOuter)
//   2. 渲染 Layout / Header / Sider Menu / Content(AppInner)
//   3. 根据 useAuth().capabilities 决定哪些 Tab 可见
//   4. 按 activeView switch 渲染对应的 <XxxView>
//
// 严禁:
//   - 在本文件里新增 useState / handler / 业务 JSX
//   - 在本文件里写 api 调用 / 表单 / 扫码逻辑
//
// 新增业务功能一律下沉到一级业务目录;链交互页面/接口放到所属模块的 chain_* 文件。
// 详见 memory/05-modules/sfid/frontend/FRONTEND_LAYOUT.md。
//
// 历史:任务卡 20260408-sfid-frontend-app-tsx-split 把原 3431 行 App.tsx
// 拆到当前规模,步 6 收官清理后的最终形态见此文件。
// =============================================================================

import { useEffect, useState } from 'react';
import { QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Layout, Typography } from 'antd';
import { AuthProvider } from './auth/AuthContext';
import { useAuth } from './hooks/useAuth';
import { writeStoredAuth, clearStoredAuth } from './utils/storedAuth';
import type { AdminAuth } from './auth/types';
import { adminLogout, checkAdminAuth } from './auth/api';
import type { SfidMetaResult } from './china/api';
import { loadCachedSfidMeta } from './china/metaCache';
import { LoginView } from './auth/LoginView';
import { GovView } from './gov/GovView';
import { PrivateView } from './private/PrivateView';
import { OperatorsView } from './admins/OperatorsView';
import { ShengAdminsView } from './admins/ShengAdminsView';
import { CitizensView } from './citizens/CitizensView';
import { notice } from './utils/notice';

const { Header, Content } = Layout;

/** Header 右上角管理员身份与姓名,样式与 CPMS 管理端保持一致。 */
function resolveHeaderAdminIdentity(auth: AdminAuth | null): { roleLabel: string; adminName: string } {
  if (!auth) return { roleLabel: '', adminName: '' };
  const name = typeof auth.admin_name === 'string' ? auth.admin_name.trim() : '';
  // 中文注释:当前只剩 FEDERAL_ADMIN / SHI_ADMIN 两个管理员角色。
  const roleLabel = auth.role === 'FEDERAL_ADMIN'
    ? '联邦管理员'
    : auth.role === 'SHI_ADMIN'
      ? '市级管理员'
      : '';
  return { roleLabel, adminName: name || '暂未设置' };
}

type ActiveView =
  | 'citizens'
  | 'public-security'
  | 'gov'
  | 'private'
  | 'system-settings'
  | 'sheng-admins'
  | 'operators';

function AppInner() {
  const { auth, setAuth, capabilities } = useAuth();
  const [bootstrapping, setBootstrapping] = useState(true);
  const [activeView, setActiveView] = useState<ActiveView>('citizens');
  const [viewResetToken, setViewResetToken] = useState(0);
  // 中文注释:sfidMeta 仍需在 App.tsx 持有,因为机构类 Tab 点击事件要统一拉取省市元数据。
  const [sfidMeta, setSfidMeta] = useState<SfidMetaResult | null>(null);

  useEffect(() => {
    setSfidMeta(null);
  }, [auth?.admin_pubkey, auth?.role]);

  useEffect(() => {
    let cancelled = false;
    const bootstrap = async () => {
      if (!auth) {
        setBootstrapping(false);
        return;
      }
      try {
        const checked = await checkAdminAuth(auth);
        const refreshedAuth: AdminAuth = {
          ...auth,
          admin_pubkey: checked.admin_pubkey,
          role: checked.role,
          admin_name: checked.admin_name,
          admin_province: checked.admin_province ?? null,
          admin_city: checked.admin_city ?? null,
          passkey_bound: checked.passkey_bound,
        };
        setAuth(refreshedAuth);
        writeStoredAuth(refreshedAuth);
      } catch {
        if (!cancelled) {
          clearStoredAuth();
          setAuth(null);
        }
      } finally {
        if (!cancelled) setBootstrapping(false);
      }
    };
    bootstrap();
    return () => {
      cancelled = true;
    };
  }, []);

  const mustUpdatePasskey = !!auth && auth.passkey_bound === false;

  useEffect(() => {
    if (mustUpdatePasskey && activeView !== 'system-settings') {
      setActiveView('system-settings');
    }
  }, [mustUpdatePasskey, activeView]);
  const routedView: ActiveView = mustUpdatePasskey ? 'system-settings' : activeView;
  const headerAdminIdentity = resolveHeaderAdminIdentity(auth);

  const onLogout = () => {
    // best-effort 通知后端销毁 session,不阻塞前端退出
    if (auth) adminLogout(auth);
    setAuth(null);
    clearStoredAuth();
    setActiveView('citizens');
    setViewResetToken((v) => v + 1);
    setSfidMeta(null);
    notice.success('已退出登录');
  };

  /** 点击机构类 Tab 时统一加载省份列表(传给 gov/private 页面) */
  const loadSfidMetaForInstitutions = async () => {
    if (!auth) return;
    if (sfidMeta) return;
    try {
      const meta = await loadCachedSfidMeta(auth);
      setSfidMeta(meta);
    } catch (err) {
      notice.error(err, '');
    }
  };

  const switchView = async (view: ActiveView, options?: { loadSfidMeta?: boolean }) => {
    // 中文注释:重复点击当前 tab 也要重置子页面,用于从机构详情页回到模块入口。
    setActiveView(view);
    setViewResetToken((v) => v + 1);
    if (options?.loadSfidMeta) await loadSfidMetaForInstitutions();
  };

  return (
    <Layout
      style={{
        minHeight: '100vh',
        background: 'linear-gradient(145deg, #0f172a 0%, #134e4a 40%, #0f766e 70%, #115e59 100%)',
        backgroundAttachment: 'fixed',
        position: 'relative'
      }}
    >
      {/* 背景装饰层 */}
      <div style={{ position: 'fixed', inset: 0, pointerEvents: 'none', zIndex: 0, overflow: 'hidden' }}>
        <div
          style={{
            position: 'absolute', top: '-20%', right: '-10%', width: '50vw', height: '50vw',
            borderRadius: '50%',
            background: 'radial-gradient(circle, rgba(13,148,136,0.25) 0%, transparent 70%)'
          }}
        />
        <div
          style={{
            position: 'absolute', bottom: '-15%', left: '-10%', width: '45vw', height: '45vw',
            borderRadius: '50%',
            background: 'radial-gradient(circle, rgba(20,184,166,0.15) 0%, transparent 70%)'
          }}
        />
        <div
          style={{
            position: 'absolute', inset: 0,
            backgroundImage:
              'linear-gradient(rgba(255,255,255,0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.03) 1px, transparent 1px)',
            backgroundSize: '60px 60px'
          }}
        />
        <div
          style={{
            position: 'absolute', inset: 0,
            backgroundImage:
              'linear-gradient(135deg, transparent 48.5%, rgba(255,255,255,0.015) 48.5%, rgba(255,255,255,0.015) 51.5%, transparent 51.5%)',
            backgroundSize: '120px 120px'
          }}
        />
      </div>

      <Header
        style={{
          position: 'relative', zIndex: 1,
          background: 'linear-gradient(135deg, rgba(13,148,136,0.7) 0%, rgba(15,118,110,0.8) 100%)',
          backdropFilter: 'blur(12px)',
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          paddingInline: 32, height: 72,
          borderBottom: '1px solid rgba(255,255,255,0.15)',
          boxShadow: '0 2px 16px rgba(0,0,0,0.12)'
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <div
            style={{
              width: 44, height: 44, borderRadius: 10,
              background: 'rgba(255,255,255,0.18)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontSize: 22, border: '1px solid rgba(255,255,255,0.25)'
            }}
          >
            <QrcodeOutlined style={{ color: '#fff' }} />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.2 }}>
            <Typography.Text style={{ color: '#ffffff', fontSize: 20, fontWeight: 700, letterSpacing: 2 }}>
              中华民族联邦共和国
            </Typography.Text>
            <Typography.Text
              style={{
                color: 'rgba(255,255,255,0.8)', fontSize: 13, fontWeight: 500,
                letterSpacing: 4, marginTop: 2
              }}
            >
              身份识别码系统
            </Typography.Text>
          </div>
        </div>
        {auth && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <Typography.Text
              style={{
                color: '#ffffff', fontSize: 14, fontWeight: 500,
                background: 'rgba(255,255,255,0.12)',
                padding: '6px 16px', borderRadius: 8,
                border: '1px solid rgba(255,255,255,0.15)',
                display: 'inline-flex', alignItems: 'center', gap: 8,
              }}
            >
              <span>{headerAdminIdentity.roleLabel}</span>
              <span style={{ display: 'inline-flex', alignItems: 'center', justifyContent: 'center', lineHeight: 1 }}>·</span>
              <span>{headerAdminIdentity.adminName}</span>
            </Typography.Text>
            <Button
              size="small"
              danger
              onClick={onLogout}
              style={{
                background: 'rgba(255,255,255,0.1)',
                borderColor: 'rgba(255,255,255,0.25)',
                color: '#fca5a5', fontWeight: 500, borderRadius: 8
              }}
            >
              退出
            </Button>
          </div>
        )}
      </Header>

      {bootstrapping ? (
        <Content
          style={{
            position: 'relative', zIndex: 1,
            display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24
          }}
        >
          <Card bordered={false} style={{ width: 520, maxWidth: '92vw' }} loading />
        </Content>
      ) : !auth ? (
        <Content
          style={{
            position: 'relative', zIndex: 1,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            padding: 24, minHeight: 'calc(100vh - 72px)'
          }}
        >
          <LoginView />
        </Content>
      ) : (
        <Content style={{ position: 'relative', zIndex: 1, padding: '16px 24px 24px' }}>
          <div
            style={{
              display: 'flex', gap: 6, marginBottom: 20, padding: '8px 12px',
              background: 'rgba(255,255,255,0.08)', backdropFilter: 'blur(12px)',
              borderRadius: 14, border: '1px solid rgba(255,255,255,0.1)',
              width: 'fit-content'
            }}
          >
            {/* 中文注释:Tab 顺序 — 首页 → 私权机构 → 公权机构 → 公安局 → 注册局。 */}
            {([
              { key: 'citizens' as const, label: '首页', visible: !mustUpdatePasskey, onClick: () => switchView('citizens') },
              {
                key: 'private' as const, label: '私权机构',
                visible: !mustUpdatePasskey && capabilities.canViewPrivate,
                onClick: () => switchView('private', { loadSfidMeta: true })
              },
              {
                key: 'gov' as const, label: '公权机构',
                visible: !mustUpdatePasskey && capabilities.canViewInstitutions,
                onClick: () => switchView('gov', { loadSfidMeta: true })
              },
              {
                key: 'public-security' as const, label: '公安局',
                visible: !mustUpdatePasskey && capabilities.canViewInstitutions,
                onClick: () => switchView('public-security', { loadSfidMeta: true })
              },
              {
                key: 'system-settings' as const, label: '注册局',
                visible: capabilities.canViewSystemSettings,
                onClick: () => switchView('system-settings')
              }
            ] as const)
              .filter((tab) => tab.visible)
              .map((tab) => (
                <button
                  key={tab.key}
                  onClick={tab.onClick}
                  style={{
                    padding: '8px 20px', borderRadius: 10, border: 'none', cursor: 'pointer',
                    fontSize: 14, fontWeight: 500, transition: 'all 0.2s ease',
                    ...(routedView === tab.key
                      ? {
                          background: 'linear-gradient(135deg, #0d9488, #0f766e)',
                          color: '#fff',
                          boxShadow: '0 2px 8px rgba(13,148,136,0.35)'
                        }
                      : { background: 'transparent', color: 'rgba(255,255,255,0.7)' })
                  }}
                >
                  {tab.label}
                </button>
              ))}
          </div>

          {routedView === 'operators' && capabilities.canViewShiAdmins ? (
            <OperatorsView />
          ) : routedView === 'sheng-admins' && capabilities.canViewShengAdmins ? (
            <ShengAdminsView mode="list" />
          ) : routedView === 'public-security' && capabilities.canManageInstitutions && auth ? (
            // 中文注释:公安局属于 gov 前端边界,但仍保持独立 Tab。
            <GovView key={`public-security-${viewResetToken}`} auth={auth} category="PUBLIC_SECURITY" sfidMeta={sfidMeta} resetToken={viewResetToken} />
          ) : routedView === 'gov' && capabilities.canManageInstitutions && auth ? (
            <GovView key={`gov-${viewResetToken}`} auth={auth} category="GOV_INSTITUTION" sfidMeta={sfidMeta} resetToken={viewResetToken} />
          ) : routedView === 'private' && capabilities.canViewPrivate && auth ? (
            <PrivateView auth={auth} sfidMeta={sfidMeta} />
          ) : routedView === 'system-settings' && capabilities.canViewSystemSettings ? (
            <ShengAdminsView mode="system-settings" />
          ) : (
            <CitizensView />
          )}
        </Content>
      )}
    </Layout>
  );
}

/**
 * AppOuter:只负责挂 <AuthProvider>。
 * 其它所有状态与业务都在 AppInner / 各业务子目录组件内。
 */
export default function App() {
  return (
    <AuthProvider>
      <AppInner />
    </AuthProvider>
  );
}
