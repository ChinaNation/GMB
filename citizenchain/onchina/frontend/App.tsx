// =============================================================================
// 中文文件头:App.tsx = 路由壳子 + Layout 壳子
// -----------------------------------------------------------------------------
// 本文件是 OnChina 前端的顶层路由壳,职责仅限:
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
// 详见 memory/05-modules/citizenchain/onchina/FRONTEND_TECHNICAL.md。
// =============================================================================

import { useEffect, useState } from 'react';
import { QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Layout, Typography } from 'antd';
import { AuthProvider } from './auth/AuthContext';
import { useAuth } from './hooks/useAuth';
import { writeStoredAuth, clearStoredAuth } from './utils/storedAuth';
import type { AdminAuth } from './auth/types';
import { adminLogout, checkAdminAuth } from './auth/api';
import { getPasskeyStatus } from './auth/passkey/passkeyClient';
import type { CidMetaResult } from './china/api';
import { loadCachedCidMeta } from './china/metaCache';
import { LoginView, OrganizationCaNotice } from './auth/LoginView';
import type { RoleCapabilities } from './auth/AuthContext';
import { GovView } from './gov/GovView';
import { PrivateShell } from './private/PrivateShell';
import { EducationView } from './education/EducationView';
import { OwnInstitutionAdminsView, RegistryAdminsView } from './admins/RegistryAdminsView';
import { isSubordinateRegistry, isTier1Registry } from './platform/registryTier';
import { CitizensView } from './citizens/CitizensView';
import { AddressManageView } from './address/AddressManageView';
import { LegislationView } from './legislation/operator/LegislationView';
import type { PrivateType } from './subjects/api';
import { notice } from './utils/notice';

const { Header, Content } = Layout;

/** Header 右上角管理员身份与姓名。 */
function resolveHeaderAdminIdentity(auth: AdminAuth | null): { cidShortName: string; adminName: string } {
  if (!auth) return { cidShortName: '', adminName: '' };
  const name = typeof auth.admin_name === 'string' ? auth.admin_name.trim() : '';
  // 中文注释:左段显示所属机构简称,取自 auth.cid_short_name(= subjects.cid_short_name 单一字段);
  // 空时(简称未加载)整段留空,不显示伪造名。
  const cidShortName =
    typeof auth.cid_short_name === 'string' ? auth.cid_short_name.trim() : '';
  return { cidShortName, adminName: name || '暂未设置' };
}

type ActiveView =
  | 'citizens'
  | 'gov'
  | 'private-sole'
  | 'private-partnership'
  | 'private-company'
  | 'private-corporation'
  | 'private-welfare'
  | 'private-association'
  | 'education'
  | 'address'
  | 'own-admins'
  | 'city-registry'
  | 'federal-registry'
  | 'legislation';

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

function registryAdminListView(institutionCode: string | null | undefined): ActiveView | null {
  if (isTier1Registry(institutionCode)) return 'federal-registry';
  if (isSubordinateRegistry(institutionCode)) return 'city-registry';
  return null;
}

function firstBusinessView(capabilities: RoleCapabilities): ActiveView {
  if (capabilities.canViewCitizens) return 'citizens';
  if (capabilities.canViewInstitutions) return 'gov';
  if (capabilities.canViewPrivate) return 'private-sole';
  if (capabilities.canViewEducation) return 'education';
  if (capabilities.canViewCityRegistry) return 'city-registry';
  if (capabilities.canViewFederalRegistry) return 'federal-registry';
  if (capabilities.canViewLegislation) return 'legislation';
  if (capabilities.canViewOwnAdmins) return 'own-admins';
  return 'citizens';
}

function AppInner() {
  const { auth, setAuth, capabilities } = useAuth();
  const [bootstrapping, setBootstrapping] = useState(true);
  const [activeView, setActiveView] = useState<ActiveView>('citizens');
  const [viewResetToken, setViewResetToken] = useState(0);
  // 中文注释:cidMeta 仍需在 App.tsx 持有,因为机构类 Tab 点击事件要统一拉取省市元数据。
  const [cidMeta, setCidMeta] = useState<CidMetaResult | null>(null);
  // 中文注释:当前管理员是否已注册 passkey(null=未知);驱动登录默认跳转到管理员列表。
  const [passkeyRegistered, setPasskeyRegistered] = useState<boolean | null>(null);
  // 中文注释:默认落地 tab 只在会话首次确定(机构码 + passkey 状态都就绪)时设置一次,
  // 之后 passkey 状态异步到达不再覆盖用户手动切换的 tab。
  const [hasInitializedView, setHasInitializedView] = useState(false);

  useEffect(() => {
    setCidMeta(null);
  }, [auth?.admin_account, auth?.institution_code]);

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
          institution_code: checked.institution_code,
          admin_level: checked.admin_level ?? null,
          capabilities: checked.capabilities,
          admin_name: checked.admin_name,
          scope_province_name: checked.scope_province_name ?? null,
          scope_city_name: checked.scope_city_name ?? null,
          scope_town_name: checked.scope_town_name ?? null,
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

  // 中文注释:登录管理员变化(含登出后重新登录)时重取 passkey 状态并重置默认落地标志。
  useEffect(() => {
    // 管理员变化即重置默认落地标志,让新会话重新决定首屏 tab。
    setHasInitializedView(false);
    if (!auth) {
      setPasskeyRegistered(null);
      return;
    }
    let cancelled = false;
    getPasskeyStatus(auth)
      // 出错保守视为已注册(不强制跳转打扰)。
      .then((registered) => {
        if (!cancelled) setPasskeyRegistered(registered);
      })
      .catch(() => {
        if (!cancelled) setPasskeyRegistered(true);
      });
    return () => {
      cancelled = true;
    };
  }, [auth?.admin_account]);

  // 中文注释:按 passkey 状态决定默认落地 tab。未设置 passkey 的注册局管理员先落到
  // 自己机构的管理员列表;已设置后进入首个业务 tab,FRG/CREG 的完整 tab 由后端能力表控制。
  // 未注册 passkey 的管理员默认改进自己机构的管理员列表(看到自己那行红点去设置密钥)。
  // 依赖机构码 + passkey 状态,会话内状态稳定后不再覆盖用户手动切换。
  useEffect(() => {
    if (!auth?.institution_code) return;
    if (passkeyRegistered === null) return;
    if (hasInitializedView) return;
    setHasInitializedView(true);
    const adminListTab = registryAdminListView(auth.institution_code);
    if (!passkeyRegistered) {
      setActiveView(adminListTab ?? firstBusinessView(capabilities));
      return;
    }
    setActiveView(firstBusinessView(capabilities));
  }, [auth?.institution_code, capabilities, passkeyRegistered, hasInitializedView]);

  const routedView: ActiveView = activeView;
  const routedPrivateType = privateTypeForView(routedView);
  const headerAdminIdentity = resolveHeaderAdminIdentity(auth);
  const passkeyLockedRegistryView =
    auth && passkeyRegistered === false ? registryAdminListView(auth.institution_code) : null;

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
              链上中国平台
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
          <OrganizationCaNotice compact />
          <div
            style={{
              display: 'flex', gap: 6, marginBottom: 20, padding: '8px 12px',
              background: 'rgba(255,255,255,0.08)', backdropFilter: 'blur(12px)',
              borderRadius: 14, border: '1px solid rgba(255,255,255,0.1)',
              width: 'fit-content'
            }}
          >
            {/* 中文注释:Tab 顺序 — 公民 → 六类私权机构 → 教育机构 → 公权机构 → 本机构管理员 → 市注册局 → 联邦注册局。
                FRG 能力是 CREG 超集;CREG 可只读本省联邦注册局 tab。未设置 passkey 的注册局
                管理员只显示自己机构的管理员列表入口;普通机构管理员进入本机构管理员页设置 passkey。 */}
            {([
              { key: 'citizens' as const, label: '公民', visible: capabilities.canViewCitizens, onClick: () => switchView('citizens') },
              { key: 'private-sole' as const, label: '个体经营', visible: capabilities.canViewPrivate, onClick: () => switchView('private-sole', { loadCidMeta: true }) },
              { key: 'private-partnership' as const, label: '合伙企业', visible: capabilities.canViewPrivate, onClick: () => switchView('private-partnership', { loadCidMeta: true }) },
              { key: 'private-company' as const, label: '股权公司', visible: capabilities.canViewPrivate, onClick: () => switchView('private-company', { loadCidMeta: true }) },
              { key: 'private-corporation' as const, label: '股份公司', visible: capabilities.canViewPrivate, onClick: () => switchView('private-corporation', { loadCidMeta: true }) },
              { key: 'private-welfare' as const, label: '公益组织', visible: capabilities.canViewPrivate, onClick: () => switchView('private-welfare', { loadCidMeta: true }) },
              { key: 'private-association' as const, label: '注册协会', visible: capabilities.canViewPrivate, onClick: () => switchView('private-association', { loadCidMeta: true }) },
              {
                key: 'education' as const, label: '教育机构',
                visible: capabilities.canViewEducation,
                onClick: () => switchView('education', { loadCidMeta: true })
              },
              {
                key: 'gov' as const, label: '公权机构',
                visible: capabilities.canViewInstitutions,
                onClick: () => switchView('gov', { loadCidMeta: true })
              },
              {
                key: 'address' as const, label: '地址库',
                visible: capabilities.canViewInstitutions,
                onClick: () => switchView('address')
              },
              {
                key: 'legislation' as const, label: '立法与表决',
                visible: capabilities.canViewLegislation,
                onClick: () => switchView('legislation')
              },
              {
                key: 'own-admins' as const, label: '本机构管理员',
                visible: capabilities.canViewOwnAdmins,
                onClick: () => switchView('own-admins')
              },
              {
                key: 'city-registry' as const, label: '市注册局',
                visible: capabilities.canViewCityRegistry,
                onClick: () => switchView('city-registry')
              },
              {
                key: 'federal-registry' as const, label: '联邦注册局',
                visible: capabilities.canViewFederalRegistry,
                onClick: () => switchView('federal-registry')
              }
            ] as const)
              .filter((tab) => tab.visible && (!passkeyLockedRegistryView || tab.key === passkeyLockedRegistryView))
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

          {routedView === 'gov' && capabilities.canManageInstitutions && auth ? (
            // 中文注释:市公安局已折叠为普通公权机构,统一在公权机构列表展示,不再有独立分支。
            <GovView key={`gov-${viewResetToken}`} auth={auth} cidMeta={cidMeta} resetToken={viewResetToken} />
          ) : routedPrivateType && capabilities.canViewPrivate && auth ? (
            <PrivateShell
              key={`${routedView}-${viewResetToken}`}
              auth={auth}
              cidMeta={cidMeta}
              privateType={routedPrivateType}
            />
          ) : routedView === 'education' && capabilities.canViewEducation && auth ? (
            <EducationView key={`education-${viewResetToken}`} auth={auth} cidMeta={cidMeta} />
          ) : routedView === 'address' && capabilities.canViewInstitutions && auth ? (
            <AddressManageView key={`address-${viewResetToken}`} auth={auth} />
          ) : routedView === 'own-admins' && capabilities.canViewOwnAdmins ? (
            <OwnInstitutionAdminsView key={`own-admins-${viewResetToken}`} />
          ) : routedView === 'city-registry' && capabilities.canViewCityRegistry ? (
            <RegistryAdminsView key={`city-registry-${viewResetToken}`} mode="city-registry" />
          ) : routedView === 'federal-registry' && capabilities.canViewFederalRegistry ? (
            <RegistryAdminsView key={`federal-registry-${viewResetToken}`} mode="federal-registry" />
          ) : routedView === 'legislation' && capabilities.canViewLegislation && auth ? (
            <LegislationView key={`legislation-${viewResetToken}`} auth={auth} />
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
