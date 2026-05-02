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
// 新增业务功能一律下沉到 views/<module>/<ModuleView>.tsx。
// 详见 views/README.md 与 memory/feedback_sfid_frontend_modular_structure.md。
//
// 历史:任务卡 20260408-sfid-frontend-app-tsx-split 把原 3431 行 App.tsx
// 拆到当前规模,步 6 收官清理后的最终形态见此文件。
// =============================================================================

import { useEffect, useState } from 'react';
import { QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Layout, Typography, message } from 'antd';
import { AuthProvider } from '../contexts/AuthContext';
import { useAuth } from '../hooks/useAuth';
import { writeStoredAuth, clearStoredAuth } from '../utils/storedAuth';
import type { AdminAuth, SfidMetaResult } from '../api/client';
import { adminLogout, checkAdminAuth, getSfidMeta } from '../api/client';
import { LoginView } from '../views/auth/LoginView';
import { InstitutionsView } from '../views/institutions/InstitutionsView';
import { OperatorsView } from '../views/shi_admin/OperatorsView';
import { ShengAdminsView } from '../views/sheng_admin/ShengAdminsView';
import { RosterPage } from '../views/sheng_admin/RosterPage';
import { ActivationPage } from '../views/sheng_admin/ActivationPage';
import { RotatePage } from '../views/sheng_admin/RotatePage';
import { CitizensView } from '../views/citizens/CitizensView';

const { Header, Content } = Layout;

/** 业务卡片统一毛玻璃风格(export 给 views/ 子组件复用,保持视觉一致) */
export const glassCardStyle: React.CSSProperties = {
  background: 'rgba(255,255,255,0.92)',
  backdropFilter: 'blur(16px)',
  borderRadius: 16,
  boxShadow: '0 4px 24px rgba(0,0,0,0.06)',
  border: '1px solid rgba(255,255,255,0.6)'
};

/** Card title 左侧 teal 竖条 + 加粗(export 同上) */
export const glassCardHeadStyle: React.CSSProperties = {
  borderBottom: '1px solid #e5e7eb',
  borderLeft: '3px solid #0d9488',
  paddingLeft: 20,
  fontWeight: 600
};

/** Header 右上角的管理员显示名称 */
function resolveHeaderAdminName(auth: AdminAuth | null): string {
  if (!auth) return '';
  const name = typeof auth.admin_name === 'string' ? auth.admin_name.trim() : '';
  if (name) return name;
  // ADR-008(2026-05-01)起 KEY_ADMIN 角色已彻底删除,只剩 SHENG_ADMIN / SHI_ADMIN。
  if (auth.role === 'SHENG_ADMIN') return '省级管理员';
  if (auth.role === 'SHI_ADMIN') return '市级管理员';
  return '';
}

type ActiveView =
  | 'citizens'
  | 'institutions'
  | 'gov-institutions'
  | 'multisig'
  | 'system-settings'
  | 'sheng-admins'
  | 'operators'
  | 'sheng-roster'
  | 'sheng-signer-activate'
  | 'sheng-signer-rotate';

function AppInner() {
  const { auth, setAuth, capabilities } = useAuth();
  const [bootstrapping, setBootstrapping] = useState(true);
  const [activeView, setActiveView] = useState<ActiveView>('citizens');
  // 中文注释:sfidMeta 仍需在 App.tsx 持有,因为它由 Tab 点击事件统一拉取后传给 InstitutionsView。
  const [sfidMeta, setSfidMeta] = useState<SfidMetaResult | null>(null);

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
          admin_city: checked.admin_city ?? null
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

  const onLogout = () => {
    // best-effort 通知后端销毁 session,不阻塞前端退出
    if (auth) adminLogout(auth);
    setAuth(null);
    clearStoredAuth();
    setActiveView('citizens');
    message.success('已退出登录');
  };

  /** 点击机构类 Tab 时统一加载省份列表(传给 InstitutionsView) */
  const loadSfidMetaForInstitutions = async () => {
    if (!auth) return;
    try {
      const meta = await getSfidMeta(auth);
      setSfidMeta(meta);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '加载省份列表失败');
    }
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
                border: '1px solid rgba(255,255,255,0.15)'
              }}
            >
              {resolveHeaderAdminName(auth)}
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
            {/* 中文注释:Tab 顺序 — 首页 → 私权机构 → 公权机构 → 公安局 → 注册局 → 密钥管理 */}
            {([
              { key: 'citizens' as const, label: '首页', visible: true, onClick: () => setActiveView('citizens') },
              {
                key: 'multisig' as const, label: '私权机构',
                visible: capabilities.canViewMultisig,
                onClick: async () => { setActiveView('multisig'); await loadSfidMetaForInstitutions(); }
              },
              {
                key: 'gov-institutions' as const, label: '公权机构',
                visible: capabilities.canViewInstitutions,
                onClick: async () => { setActiveView('gov-institutions'); await loadSfidMetaForInstitutions(); }
              },
              {
                key: 'institutions' as const, label: '公安局',
                visible: capabilities.canViewInstitutions,
                onClick: async () => { setActiveView('institutions'); await loadSfidMetaForInstitutions(); }
              },
              {
                key: 'system-settings' as const, label: '注册局',
                visible: capabilities.canViewSystemSettings,
                onClick: () => setActiveView('system-settings')
              },
              {
                key: 'sheng-roster' as const, label: '省管理员名册',
                visible: auth?.role === 'SHENG_ADMIN',
                onClick: () => setActiveView('sheng-roster')
              },
              {
                key: 'sheng-signer-activate' as const, label: '激活签名',
                visible: auth?.role === 'SHENG_ADMIN',
                onClick: () => setActiveView('sheng-signer-activate')
              },
              {
                key: 'sheng-signer-rotate' as const, label: 'rotate 签名',
                visible: auth?.role === 'SHENG_ADMIN',
                onClick: () => setActiveView('sheng-signer-rotate')
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
                    ...(activeView === tab.key
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

          {activeView === 'operators' && capabilities.canViewShiAdmins ? (
            <OperatorsView />
          ) : activeView === 'sheng-admins' && capabilities.canViewShengAdmins ? (
            <ShengAdminsView mode="list" />
          ) : activeView === 'institutions' && capabilities.canManageInstitutions && auth ? (
            // 中文注释:三机构 tab 共享 InstitutionsView,key=category 保证切 tab 时 state 重置
            <InstitutionsView key="PUBLIC_SECURITY" auth={auth} category="PUBLIC_SECURITY" sfidMeta={sfidMeta} />
          ) : activeView === 'gov-institutions' && capabilities.canManageInstitutions && auth ? (
            <InstitutionsView key="GOV_INSTITUTION" auth={auth} category="GOV_INSTITUTION" sfidMeta={sfidMeta} />
          ) : activeView === 'multisig' && capabilities.canViewMultisig && auth ? (
            <InstitutionsView key="PRIVATE_INSTITUTION" auth={auth} category="PRIVATE_INSTITUTION" sfidMeta={sfidMeta} />
          ) : activeView === 'system-settings' && capabilities.canViewSystemSettings ? (
            <ShengAdminsView mode="system-settings" />
          ) : activeView === 'sheng-roster' && auth?.role === 'SHENG_ADMIN' ? (
            <RosterPage auth={auth} />
          ) : activeView === 'sheng-signer-activate' && auth?.role === 'SHENG_ADMIN' ? (
            <ActivationPage auth={auth} />
          ) : activeView === 'sheng-signer-rotate' && auth?.role === 'SHENG_ADMIN' ? (
            <RotatePage auth={auth} />
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
 * 其它所有状态与业务都在 AppInner / 各 views/ 子组件内。
 */
export default function App() {
  return (
    <AuthProvider>
      <AppInner />
    </AuthProvider>
  );
}
