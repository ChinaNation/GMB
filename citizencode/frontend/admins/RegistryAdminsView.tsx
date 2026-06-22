// 注册局视图 —— 调度器:持有所有状态和副作用,按 mode 分派:
//   - 'city-registry'    → CityRegistryView(市注册局 tab:城市表格→市注册局机构详情页)
//   - 'federal-registry' → FederalRegistryView(联邦注册局 tab:联邦注册局机构详情页)
// 联邦注册局管理员城市表格→点市进详情;市注册局管理员直接进本市/本省详情。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Form, Input, Modal, Space, Typography } from 'antd';
import type { ModalProps } from 'antd';
import { useAuth } from '../hooks/useAuth';
import type { AdminAuth } from '../auth/types';
import type { CityRegistryAdminRow } from './city_registry_admins_api';
import type { FederalRegistryAdminRow } from './api';
import type { CidCityItem } from '../china/api';
import { listCityRegistryAdmins, updateCityRegistryName } from './city_registry_admins_api';
import {
  commitAdminAction,
  formatAdminCreateError,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
} from './admin_security_api';
import { listFederalRegistryAdmins } from './api';
import { loadCachedCidCities, readCachedCidCities } from '../china/metaCache';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { MAX_CITY_REGISTRY_ADMINS_PER_CITY, sameHexAccount } from './adminUtils';
import type { AccountScanTarget, RegistryAdminsSharedState } from './adminUtils';
import { CityRegistryView, FederalRegistryView } from './ProvinceDetailView';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { CitizenSignatureModal } from '../core/CitizenSignatureModal';
import { CID_MODAL_Z_INDEX } from '../core/modalStack';
import { notice } from '../utils/notice';
import { getFederalRegistry, listOfficialInstitutions } from '../gov/api';
import type { InstitutionDetail } from '../subjects/api';

export interface RegistryAdminsViewProps {
  /// 'city-registry' = 市注册局 tab(城市表格→市注册局机构详情页);
  /// 'federal-registry' = 联邦注册局 tab(联邦注册局机构详情页)
  mode: 'city-registry' | 'federal-registry';
}

type AdminActionModalState = {
  actionId: string;
  signRequest: string;
  payloadHash: string;
  passkeyAssertion: unknown;
  resolve: (value: unknown) => void;
  reject: (reason?: unknown) => void;
};

const centeredConfirmFooter: ModalProps['footer'] = (_originNode, { OkBtn, CancelBtn }) => (
  <div style={{ display: 'flex', justifyContent: 'center', gap: 8 }}>
    <CancelBtn />
    <OkBtn />
  </div>
);

const ADMIN_LIST_CACHE_VERSION = 'cid-admin-list-v1';

interface CachedAdminListPayload<T> {
  version: string;
  rows: T[];
}

function adminListCacheKey(
  kind: 'federal-registry-admins' | 'city-registry-admins',
  auth: AdminAuth,
  mode: RegistryAdminsViewProps['mode'],
): string {
  return [
    'cid:admin-list',
    ADMIN_LIST_CACHE_VERSION,
    kind,
    auth.admin_account,
    auth.registry_org_code,
    auth.scope_province_name || 'ALL',
    auth.scope_city_name || 'ALL',
    mode,
  ].join(':');
}

function readCachedAdminList<T>(key: string): T[] | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as CachedAdminListPayload<T>;
    if (parsed.version !== ADMIN_LIST_CACHE_VERSION || !Array.isArray(parsed.rows)) {
      localStorage.removeItem(key);
      return null;
    }
    return parsed.rows;
  } catch {
    localStorage.removeItem(key);
    return null;
  }
}

function writeCachedAdminList<T>(key: string, rows: T[]) {
  try {
    localStorage.setItem(
      key,
      JSON.stringify({ version: ADMIN_LIST_CACHE_VERSION, rows } satisfies CachedAdminListPayload<T>),
    );
  } catch {
    // 中文注释:注册局管理员列表缓存只是减少重复转圈,写失败不能影响业务操作。
  }
}

export function RegistryAdminsView({ mode }: RegistryAdminsViewProps) {
  const { auth } = useAuth();

  const [federalRegistryAdmins, setFederalRegistryAdmins] = useState<FederalRegistryAdminRow[]>([]);
  const [federalRegistryAdminsLoading, setFederalRegistryAdminsLoading] = useState(false);
  const [selectedFederalRegistry, setSelectedFederalRegistry] = useState<FederalRegistryAdminRow | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [adminDetailTab, setAdminDetailTab] = useState<'city-registry-admin' | 'federal-registry-admin'>('city-registry-admin');
  const adminDetailTabRef = useRef<'city-registry-admin' | 'federal-registry-admin'>('city-registry-admin');
  const lastSelectedFederalRegistryKey = useRef<string | null>(null);

  const [cityRegistryAdmins, setCityRegistryAdmins] = useState<CityRegistryAdminRow[]>([]);
  const [cityRegistryAdminsLoading, setCityRegistryAdminsLoading] = useState(false);
  const [cityRegistryAdminListPage, setCityRegistryListPage] = useState(1);

  const initialCityRegistryCities = auth?.scope_province_name
    ? readCachedCidCities(auth.scope_province_name)
    : null;
  const [cityRegistryAdminCities, setCityRegistryCities] = useState<CidCityItem[]>(initialCityRegistryCities ?? []);
  const [cityRegistryAdminCitiesLoading, setCityRegistryCitiesLoading] = useState(
    !!auth?.scope_province_name && !initialCityRegistryCities,
  );

  const [addCityRegistryOpen, setAddCityRegistryOpen] = useState(false);
  const [addCityRegistryLoading, setAddCityRegistryLoading] = useState(false);

  const [accountScanTarget, setAccountScanTarget] = useState<AccountScanTarget>(null);

  // ── 注册局机构详情页数据源(任务卡 20260608) ──
  // 联邦注册局:全国唯一机构,走 scope-bypass 接口,所有联邦注册局管理员可读。
  const [federalRegistryDetail, setFederalRegistryDetail] = useState<InstitutionDetail | null>(null);
  const [federalRegistryLoading, setFederalRegistryLoading] = useState(false);
  // 市注册局:当前活动市对应的机构 cid_number。
  const [cityRegistryCid, setCityRegistryCid] = useState<string | null>(null);
  const [cityRegistryLoading, setCityRegistryLoading] = useState(false);

  // 活动省/市:联邦注册局管理员看 selectedFederalRegistry + selectedCity;市注册局管理员锁定本省本市。
  const activeProvince = selectedFederalRegistry?.province_name ?? auth?.scope_province_name ?? null;
  const activeCity = selectedCity ?? (auth?.registry_org_code === 'CITY_REGISTRY' ? auth?.scope_city_name ?? null : null);

  const [addCityRegistryForm] = Form.useForm<{ city_registry_account: string; city_registry_display_name: string; city_scope_city_name: string }>();
  const [adminActionModal, setAdminActionModal] = useState<AdminActionModalState | null>(null);
  const [adminActionLoading, setAdminActionLoading] = useState(false);
  const [adminActionCommitLoading, setAdminActionCommitLoading] = useState(false);

  useEffect(() => {
    adminDetailTabRef.current = adminDetailTab;
  }, [adminDetailTab]);

  // ── 数据加载 ──

  const refreshFederalRegistryAdmins = async (): Promise<FederalRegistryAdminRow[]> => {
    if (!auth) return [];
    const cacheKey = adminListCacheKey('federal-registry-admins', auth, mode);
    const cached = readCachedAdminList<FederalRegistryAdminRow>(cacheKey);
    if (cached !== null) {
      setFederalRegistryAdmins(cached);
      setFederalRegistryAdminsLoading(false);
    } else {
      setFederalRegistryAdminsLoading(true);
    }
    try {
      const rows = await listFederalRegistryAdmins(auth);
      const list = Array.isArray(rows) ? rows : [];
      setFederalRegistryAdmins(list);
      writeCachedAdminList(cacheKey, list);
      return list;
    } catch (err) {
      notice.error(err, '加载联邦注册局管理员失败');
      return cached ?? [];
    } finally {
      if (cached === null) setFederalRegistryAdminsLoading(false);
    }
  };

  const refreshCityRegistryAdmins = async (): Promise<CityRegistryAdminRow[]> => {
    if (!auth) return [];
    const cacheKey = adminListCacheKey('city-registry-admins', auth, mode);
    const cached = readCachedAdminList<CityRegistryAdminRow>(cacheKey);
    if (cached !== null) {
      setCityRegistryAdmins(cached);
      setCityRegistryAdminsLoading(false);
    } else {
      setCityRegistryAdminsLoading(true);
    }
    try {
      const rows = await listCityRegistryAdmins(auth);
      const list = Array.isArray(rows) ? rows : [];
      setCityRegistryAdmins(list);
      writeCachedAdminList(cacheKey, list);
      return list;
    } catch (err) {
      notice.error(err, '加载市注册局管理员失败');
      return cached ?? [];
    } finally {
      if (cached === null) setCityRegistryAdminsLoading(false);
    }
  };

  // 首次挂载 / auth 变化时加载数据。
  // 角色分流由 ProvinceDetailView + useScope 自动处理,这里只负责加载数据
  // 和按当前登录角色定位 selectedFederalRegistry。
  useEffect(() => {
    let cancelled = false;
    const init = async () => {
      if (!auth) return;
      // 注册局视图(city-registry / federal-registry):联邦/市注册局管理员数据都要
      const [rows, ops] = await Promise.all([refreshFederalRegistryAdmins(), refreshCityRegistryAdmins()]);
      if (cancelled) return;
      // 自动定位到当前登录角色所属省的 FederalRegistry
      if (!selectedFederalRegistry) {
        let target: FederalRegistryAdminRow | null = null;
        if (auth.registry_org_code === 'FEDERAL_REGISTRY') {
          target = rows.find((r) => sameHexAccount(r.admin_account, auth.admin_account)) || null;
        } else if (auth.registry_org_code === 'CITY_REGISTRY') {
          const me = ops.find((o) => sameHexAccount(o.admin_account, auth.admin_account));
          if (me) {
            target = rows.find((r) => sameHexAccount(r.admin_account, me.created_by)) || null;
          }
        }
        if (!cancelled && target) setSelectedFederalRegistry(target);
      }
    };
    void init();
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token, mode]);

  // 切换 selectedFederalRegistry 时:
  //   1. 预加载该机构所属省份的城市列表
  //   2. 重置 sub-tab 到默认(市注册局管理员列表)
  //   3. 重置市注册局管理员列表分页到第 1 页
  useEffect(() => {
    if (!auth) {
      setCityRegistryCities([]);
      setCityRegistryCitiesLoading(false);
      return;
    }
    if (!selectedFederalRegistry) {
      lastSelectedFederalRegistryKey.current = null;
      const cachedRows = auth.scope_province_name ? readCachedCidCities(auth.scope_province_name) : null;
      if (cachedRows) {
        setCityRegistryCities(cachedRows);
        setCityRegistryCitiesLoading(false);
      } else {
        setCityRegistryCities([]);
        setCityRegistryCitiesLoading(!!auth.scope_province_name);
      }
      return;
    }
    const selectedKey = `${selectedFederalRegistry.province_name}:${selectedFederalRegistry.admin_account}`;
    const isRealProvinceSwitch = lastSelectedFederalRegistryKey.current !== null
      && lastSelectedFederalRegistryKey.current !== selectedKey;
    lastSelectedFederalRegistryKey.current = selectedKey;
    const cachedRows = readCachedCidCities(selectedFederalRegistry.province_name);
    if (cachedRows) {
      setCityRegistryCities(cachedRows);
      setCityRegistryCitiesLoading(false);
    } else {
      setCityRegistryCities([]);
      setCityRegistryCitiesLoading(true);
    }
    // 中文注释:首次自动定位当前省时不能覆盖用户刚点击的“联邦注册局管理员列表”。
    // 只有用户真正切换到另一个省时,才把子页签重置回市列表。
    if (isRealProvinceSwitch && adminDetailTabRef.current !== 'federal-registry-admin') {
      setAdminDetailTab(auth.passkey_bound === false && auth.registry_org_code === 'FEDERAL_REGISTRY' ? 'federal-registry-admin' : 'city-registry-admin');
    }
    setCityRegistryListPage(1);
    let cancelled = false;
    loadCachedCidCities(auth, selectedFederalRegistry.province_name)
      .then((rows) => {
        if (!cancelled) setCityRegistryCities(rows);
      })
      .catch(() => {
        if (!cancelled) setCityRegistryCities([]);
      })
      .finally(() => {
        if (!cancelled) setCityRegistryCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedFederalRegistry?.admin_account, auth?.access_token]);

  // ── 联邦注册局机构详情:进入联邦注册局 tab 时加载一次(scope-bypass) ──
  useEffect(() => {
    if (!auth || mode !== 'federal-registry') return;
    let cancelled = false;
    setFederalRegistryLoading(true);
    getFederalRegistry(auth)
      .then((d) => { if (!cancelled) setFederalRegistryDetail(d); })
      .catch((err) => { if (!cancelled) notice.error(err, '加载联邦注册局失败'); })
      .finally(() => { if (!cancelled) setFederalRegistryLoading(false); });
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token, mode]);

  // ── 市注册局机构 cid 解析:从该市公权机构目录里筛 institution_code='CREG' 那条 ──
  useEffect(() => {
    if (!auth || mode !== 'city-registry' || !activeProvince || !activeCity) {
      setCityRegistryCid(null);
      setCityRegistryLoading(false);
      return;
    }
    let cancelled = false;
    setCityRegistryCid(null);
    setCityRegistryLoading(true);
    listOfficialInstitutions(auth, { province_name: activeProvince, city_name: activeCity, page_size: 300 })
      .then((res) => {
        if (cancelled) return;
        const row = res.items.find((r) => r.institution_code === 'CREG');
        setCityRegistryCid(row?.cid_number ?? null);
      })
      .catch((err) => { if (!cancelled) notice.error(err, '加载市注册局失败'); })
      .finally(() => { if (!cancelled) setCityRegistryLoading(false); });
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token, mode, activeProvince, activeCity]);

  const runSecuredAction = async <T,>(actionType: AdminActionType, payload: unknown): Promise<T> => {
    if (!auth) throw new Error('请先登录');
    setAdminActionLoading(true);
    try {
      const prepared = await prepareAdminAction(auth, actionType, payload);
      const signRequest = prepared.sign_request;
      if (!signRequest) throw new Error('该治理操作缺少公民钱包签名请求');
      const passkeyAssertion = await getPasskeyAssertion(prepared.webauthn_options);
      return await new Promise<T>((resolve, reject) => {
        setAdminActionModal({
          actionId: prepared.action_id,
          signRequest,
          payloadHash: prepared.payload_hash,
          passkeyAssertion,
          resolve: resolve as (value: unknown) => void,
          reject,
        });
      });
    } finally {
      setAdminActionLoading(false);
    }
  };

  const handleAdminActionSignedResponse = useCallback(async (raw: string) => {
    if (!auth || !adminActionModal) return;
    setAdminActionCommitLoading(true);
    try {
      const signed = parseSignedReceiptPayload(raw, adminActionModal.actionId);
      if (signed.challenge_id !== adminActionModal.actionId) {
        throw new Error('签名回执与当前请求不匹配');
      }
      if (!signed.signer_pubkey || !signed.payload_hash) {
        throw new Error('签名回执缺少 signer_pubkey 或 payload_hash');
      }
      const result = await commitAdminAction(auth, {
        action_id: adminActionModal.actionId,
        passkey_assertion: adminActionModal.passkeyAssertion,
        signer_pubkey: signed.signer_pubkey,
        signature: signed.signature,
        payload_hash: signed.payload_hash,
      });
      adminActionModal.resolve(result);
      setAdminActionModal(null);
    } catch (error) {
      notice.error(error, '签名回执处理失败');
      adminActionModal.reject(error);
    } finally {
      setAdminActionCommitLoading(false);
    }
  }, [adminActionModal, auth]);

  // ── 事件处理 ──

  const onCreateCityRegistry = async (values: { city_registry_account: string; city_registry_display_name: string; city_name?: string }) => {
    if (!auth) return;
    const inputAddr = values.city_registry_account?.trim();
    const admin_display_name = values.city_registry_display_name?.trim();
    const city = (values.city_name ?? '').trim();
    if (!inputAddr) {
      notice.error('请输入管理员账户');
      return;
    }
    if (!admin_display_name) {
      notice.error('请输入管理员姓名');
      return;
    }
    if (!city) {
      notice.error('请选择市');
      return;
    }
    const cityCityRegistryCount = cityRegistryAdmins.filter((item) => item.city_name === city).length;
    if (cityCityRegistryCount >= MAX_CITY_REGISTRY_ADMINS_PER_CITY) {
      notice.error(`本市市注册局管理员已满 ${MAX_CITY_REGISTRY_ADMINS_PER_CITY} 人，不能继续新增`);
      return;
    }
    let admin_account: string;
    try {
      admin_account = decodeSs58(inputAddr);
    } catch (err) {
      notice.error(err, '');
      return;
    }
    setAddCityRegistryLoading(true);
    try {
      const created = await runSecuredAction<CityRegistryAdminRow>('CREATE_CITY_REGISTRY', {
        admin_account,
        admin_display_name,
        city_name: city,
      });
      notice.success('管理员新增成功');
      addCityRegistryForm.resetFields();
      setAddCityRegistryOpen(false);
      setCityRegistryAdmins((prev) => {
        const rest = prev.filter((item) => item.admin_account !== created.admin_account);
        return [created, ...rest];
      });
      await refreshCityRegistryAdmins();
    } catch (err) {
      const msg = formatAdminCreateError(err, 'CITY_REGISTRY', '新增管理员失败');
      notice.error(msg);
    } finally {
      setAddCityRegistryLoading(false);
    }
  };

  const onUpdateCityRegistry = (row: CityRegistryAdminRow) => {
    if (!auth) return;
    let nextName = row.admin_display_name;
    const ss58Address = tryEncodeSs58(row.admin_account);
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>编辑市注册局管理员</div>,
      icon: null,
      centered: true,
      zIndex: CID_MODAL_Z_INDEX.business,
      footer: centeredConfirmFooter,
      content: (
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div>
            <Typography.Text type="secondary">管理员姓名</Typography.Text>
            <Input
              defaultValue={row.admin_display_name}
              placeholder="请输入管理员姓名"
              style={{ marginTop: 6 }}
              onChange={(event) => {
                nextName = event.target.value;
              }}
            />
          </div>
          <div>
            <Typography.Text type="secondary">账户地址</Typography.Text>
            <Input
              value={ss58Address}
              disabled
              style={{ marginTop: 6 }}
            />
          </div>
        </Space>
      ),
      okText: '确认修改',
      cancelText: '取消',
      onOk: async () => {
        const admin_display_name = nextName.trim();
        if (!admin_display_name) {
          notice.error('请输入管理员姓名');
          throw new Error('admin_display_name is required');
        }
        setCityRegistryAdminsLoading(true);
        try {
          await updateCityRegistryName(auth, row.id, admin_display_name);
          notice.success('市注册局管理员信息已更新');
          await refreshCityRegistryAdmins();
        } catch (err) {
          notice.error(err, '更新市注册局管理员信息失败');
          throw err;
        } finally {
          setCityRegistryAdminsLoading(false);
        }
      },
    });
  };

  const onDeleteCityRegistry = (row: CityRegistryAdminRow) => {
    if (!auth) return;
    const ss58Address = tryEncodeSs58(row.admin_account);
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>删除市注册局管理员</div>,
      icon: null,
      centered: true,
      zIndex: CID_MODAL_Z_INDEX.business,
      footer: centeredConfirmFooter,
      content: (
        <div style={{ textAlign: 'center' }}>
          <Typography.Paragraph style={{ marginBottom: 8 }}>确认删除该市注册局管理员?</Typography.Paragraph>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>{ss58Address}</Typography.Text>
        </div>
      ),
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setCityRegistryAdminsLoading(true);
        try {
          await runSecuredAction('DELETE_CITY_REGISTRY', { id: row.id });
          notice.success('市注册局管理员已删除');
          await refreshCityRegistryAdmins();
        } catch (err) {
          notice.error(err, '删除市注册局管理员失败');
          throw err;
        } finally {
          setCityRegistryAdminsLoading(false);
        }
      },
    });
  };

  // ── 组装共享状态 ──

  const shared: RegistryAdminsSharedState = {
    federalRegistryAdmins,
    federalRegistryAdminsLoading,
    refreshFederalRegistryAdmins,
    selectedFederalRegistry,
    setSelectedFederalRegistry,
    selectedCity,
    setSelectedCity,
    adminDetailTab,
    setAdminDetailTab,
    cityRegistryAdmins,
    cityRegistryAdminsLoading,
    cityRegistryAdminListPage,
    setCityRegistryListPage,
    cityRegistryAdminCities,
    cityRegistryAdminCitiesLoading,
    addCityRegistryOpen,
    setAddCityRegistryOpen,
    addCityRegistryLoading,
    accountScanTarget,
    setAccountScanTarget,
    addCityRegistryForm,
    onCreateCityRegistry,
    onUpdateCityRegistry,
    onDeleteCityRegistry,
    runSecuredAction,
    federalRegistryDetail,
    federalRegistryLoading,
    cityRegistryCid,
    cityRegistryLoading,
  };

  // ── 渲染:按 mode 分派 ──

  const content = mode === 'city-registry'
    ? <CityRegistryView state={shared} />
    : <FederalRegistryView state={shared} />;

  return (
    <>
      {content}
      <CitizenSignatureModal
        title="公民钱包签名确认"
        open={!!adminActionModal}
        onCancel={() => {
          adminActionModal?.reject(new Error('admin action cancelled'));
          setAdminActionModal(null);
          setAdminActionCommitLoading(false);
        }}
        qrTitle="签名二维码"
        qrValue={adminActionModal?.signRequest}
        qrHint="使用当前管理员冷钱包扫码签名"
        scannerHint="扫描冷钱包生成的签名回执二维码"
        scannerDisabled={adminActionCommitLoading}
        scannerLoading={adminActionCommitLoading}
        onDetected={handleAdminActionSignedResponse}
        onScannerError={(msg) => notice.error(msg)}
      />
    </>
  );
}
