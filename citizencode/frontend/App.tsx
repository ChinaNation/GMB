// =============================================================================
// 中文文件头:App.tsx = 路由壳子 + Layout 壳子
// -----------------------------------------------------------------------------
// 本文件是 cid-frontend 的顶层路由壳,职责仅限:
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
// 详见 memory/05-modules/citizencode/frontend/FRONTEND_LAYOUT.md。
//
// 历史:任务卡 20260408-cid-frontend-app-tsx-split 把原 3431 行 App.tsx
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
import type { CidMetaResult } from './china/api';
import { loadCachedCidMeta } from './china/metaCache';
import { LoginView } from './auth/LoginView';
import { GovView } from './gov/GovView';
import { PrivateShell } from './private/PrivateShell';
import { EducationView } from './education/EducationView';
import { RegistryAdminsView } from './admins/RegistryAdminsView';
import { CitizensView } from './citizens/CitizensView';
import type { PrivateType } from './subjects/api';
import { notice } from './utils/notice';

const { Header, Content } = Layout;

/** Header 右上角管理员身份与姓名,样式与 CPMS 管理端保持一致。 */
function resolveHeaderAdminIdentity(auth: AdminAuth | null): { cidShortName: string; adminDisplayName: string } {
  if (!auth) return { cidShortName: '', adminDisplayName: '' };
  const name = typeof auth.admin_display_name === 'string' ? auth.admin_display_name.trim() : '';
  // 中文注释:左段显示所属机构简称,取自 auth.cid_short_name(= subjects.cid_short_name 单一字段),
  // 不再由 registry_org_code 硬编码另造名字;空时(简称未加载)整段留空,不显示伪造名。
  const cidShortName =
    typeof auth.cid_short_name === 'string' ? auth.cid_short_name.trim() : '';
  return { cidShortName, adminDisplayName: name || '暂未设置' };
}

type ActiveView =
  | 'citizens'
  | 'public-security'
  | 'gov'
  | 'private-sole'
  | 'private-partnership'
  | 'private-company'
  | 'private-corporation'
  | 'private-welfare'
  | 'private-association'
  | 'education'
  | 'city-registry'
  | 'federal-registry';

function privateTypeForView(view: ActiveView): PrivateType | null {
  switch (view) {
    case 'private-sole':
      return 'SOLE';
    case 'private-partnership':
      return 'PARTNERSHIP';
    case 'private-company':
      return 'COMPANY';
    case 'private-corporation':
      return 'CORPORATION';
    case 'private-welfare':
      return 'WELFARE';
    case 'private-association':
      return 'ASSOCIATION';
    default:
      return null;
  }
}

function AppInner() {
  const { auth, setAuth, capabilities } = useAuth();
  const [bootstrapping, setBootstrapping] = useState(true);
  const [activeView, setActiveView] = useState<ActiveView>('citizens');
  const [viewResetToken, setViewResetToken] = useState(0);
  // 中文注释:cidMeta 仍需在 App.tsx 持有,因为机构类 Tab 点击事件要统一拉取省市元数据。
  const [cidMeta, setCidMeta] = useState<CidMetaResult | null>(null);

  useEffect(() => {
    setCidMeta(null);
  }, [auth?.admin_account, auth?.registry_org_code]);

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
          admin_account: checked.admin_account,
          registry_org_code: checked.registry_org_code,
          admin_display_name: checked.admin_display_name,
          scope_province_name: checked.scope_province_name ?? null,
          scope_city_name: checked.scope_city_name ?? null,
          passkey_bound: checked.passkey_bound,
          cid_short_name: checked.cid_short_name ?? null,
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

  // 未绑定 passkey 时强制进入本角色可绑定 passkey 的注册局 tab:
  //   联邦注册局管理员在「联邦注册局」的联邦注册局管理员列表绑定;市注册局管理员在「市注册局」的市注册局管理员列表绑定。
  const passkeyBindView: ActiveView = auth?.registry_org_code === 'CITY_REGISTRY' ? 'city-registry' : 'federal-registry';
  useEffect(() => {
    if (mustUpdatePasskey && activeView !== passkeyBindView) {
      setActiveView(passkeyBindView);
    }
  }, [mustUpdatePasskey, activeView, passkeyBindView]);
  const routedView: ActiveView = mustUpdatePasskey ? passkeyBindView : activeView;
  const routedPrivateType = privateTypeForView(routedView);
  const headerAdminIdentity = resolveHeaderAdminIdentity(auth);

  const onLogout = () => {
    // best-effort 通知后端销毁 session,不阻塞前端退出
    if (auth) adminLogout(auth);
    setAuth(null);
    clearStoredAuth();
    setActiveView('citizens');
    setViewResetToken((v) => v + 1);
    setCidMeta(null);
    notice.success('已退出登录');
  };

  /** 点击机构类 Tab 时统一加载省份列表(传给 gov/private 页面) */
  const loadCidMetaForInstitutions = async () => {
    if (!auth) return;
    if (cidMeta) return;
    try {
      const meta = await loadCachedCidMeta(auth);
      setCidMeta(meta);
    } catch (err) {
      notice.error(err, '');
    }
  };

  const switchView = async (view: ActiveView, options?: { loadCidMeta?: boolean }) => {
    // 中文注释:重复点击当前 tab 也要重置子页面,用于从机构详情页回到模块入口。
    setActiveView(view);
    setViewResetToken((v) => v + 1);
    if (options?.loadCidMeta) await loadCidMetaForInstitutions();
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
              {headerAdminIdentity.cidShortName && (
                <>
                  <span>{headerAdminIdentity.cidShortName}</span>
                  <span style={{ display: 'inline-flex', alignItems: 'center', justifyContent: 'center', lineHeight: 1 }}>·</span>
                </>
              )}
              <span>{headerAdminIdentity.adminDisplayName}</span>
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
            {/* 中文注释:Tab 顺序 — 首页 → 六类私权机构 → 教育机构 → 公权机构 → 市公安局 → 市注册局 → 联邦注册局。 */}
            {([
              { key: 'citizens' as const, label: '首页', visible: !mustUpdatePasskey, onClick: () => switchView('citizens') },
              { key: 'private-sole' as const, label: '个体经营', visible: !mustUpdatePasskey && capabilities.canViewPrivate, onClick: () => switchView('private-sole', { loadCidMeta: true }) },
              { key: 'private-partnership' as const, label: '合伙企业', visible: !mustUpdatePasskey && capabilities.canViewPrivate, onClick: () => switchView('private-partnership', { loadCidMeta: true }) },
              { key: 'private-company' as const, label: '股权公司', visible: !mustUpdatePasskey && capabilities.canViewPrivate, onClick: () => switchView('private-company', { loadCidMeta: true }) },
              { key: 'private-corporation' as const, label: '股份公司', visible: !mustUpdatePasskey && capabilities.canViewPrivate, onClick: () => switchView('private-corporation', { loadCidMeta: true }) },
              { key: 'private-welfare' as const, label: '公益组织', visible: !mustUpdatePasskey && capabilities.canViewPrivate, onClick: () => switchView('private-welfare', { loadCidMeta: true }) },
              { key: 'private-association' as const, label: '注册协会', visible: !mustUpdatePasskey && capabilities.canViewPrivate, onClick: () => switchView('private-association', { loadCidMeta: true }) },
              {
                key: 'education' as const, label: '教育机构',
                visible: !mustUpdatePasskey && capabilities.canViewEducation,
                onClick: () => switchView('education', { loadCidMeta: true })
              },
              {
                key: 'gov' as const, label: '公权机构',
                visible: !mustUpdatePasskey && capabilities.canViewInstitutions,
                onClick: () => switchView('gov', { loadCidMeta: true })
              },
              {
                key: 'public-security' as const, label: '市公安局',
                visible: !mustUpdatePasskey && capabilities.canViewInstitutions,
                onClick: () => switchView('public-security', { loadCidMeta: true })
              },
              {
                key: 'city-registry' as const, label: '市注册局',
                visible: capabilities.canViewCityRegistry && (!mustUpdatePasskey || passkeyBindView === 'city-registry'),
                onClick: () => switchView('city-registry')
              },
              {
                key: 'federal-registry' as const, label: '联邦注册局',
                visible: capabilities.canViewFederalRegistry && (!mustUpdatePasskey || passkeyBindView === 'federal-registry'),
                onClick: () => switchView('federal-registry')
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

          {routedView === 'public-security' && capabilities.canManageInstitutions && auth ? (
            // 中文注释:公安局属于 gov 前端边界,但仍保持独立 Tab。
            <GovView key={`public-security-${viewResetToken}`} auth={auth} category="PUBLIC_SECURITY" cidMeta={cidMeta} resetToken={viewResetToken} />
          ) : routedView === 'gov' && capabilities.canManageInstitutions && auth ? (
            <GovView key={`gov-${viewResetToken}`} auth={auth} category="GOV_INSTITUTION" cidMeta={cidMeta} resetToken={viewResetToken} />
          ) : routedPrivateType && capabilities.canViewPrivate && auth ? (
            <PrivateShell
              key={`${routedView}-${viewResetToken}`}
              auth={auth}
              cidMeta={cidMeta}
              privateType={routedPrivateType}
            />
          ) : routedView === 'education' && capabilities.canViewEducation && auth ? (
            <EducationView key={`education-${viewResetToken}`} auth={auth} cidMeta={cidMeta} />
          ) : routedView === 'city-registry' && capabilities.canViewCityRegistry ? (
            <RegistryAdminsView key={`city-registry-${viewResetToken}`} mode="city-registry" />
          ) : routedView === 'federal-registry' && capabilities.canViewFederalRegistry ? (
            <RegistryAdminsView key={`federal-registry-${viewResetToken}`} mode="federal-registry" />
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
