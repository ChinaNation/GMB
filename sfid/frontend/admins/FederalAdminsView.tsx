// 注册局视图 —— 调度器:持有所有状态和副作用,按 mode 分派:
//   - 'city-registry'    → CityRegistryView(市注册局 tab:城市表格→市注册局机构详情页)
//   - 'federal-registry' → FederalRegistryView(联邦注册局 tab:联邦注册局机构详情页)
// 联邦管理员城市表格→点市进详情;市管理员直接进本市/本省详情。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Form, Input, Modal, Space, Typography } from 'antd';
import type { ModalProps } from 'antd';
import { useAuth } from '../hooks/useAuth';
import type { AdminAuth } from '../auth/types';
import type { CityAdminRow } from './city_admins_api';
import type { FederalAdminRow } from './api';
import type { SfidCityItem } from '../china/api';
import { listCityAdmins, updateCityAdminName } from './city_admins_api';
import {
  commitAdminAction,
  formatAdminCreateError,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
} from './admin_security_api';
import { listFederalAdmins } from './api';
import { loadCachedSfidCities, readCachedSfidCities } from '../china/metaCache';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { MAX_CITY_ADMINS_PER_CITY, sameHexPubkey } from './adminUtils';
import type { AccountScanTarget, FederalAdminSharedState } from './adminUtils';
import { CityRegistryView, FederalRegistryView } from './ProvinceDetailView';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { CitizenSignatureModal } from '../core/CitizenSignatureModal';
import { SFID_MODAL_Z_INDEX } from '../core/modalStack';
import { notice } from '../utils/notice';
import { getFederalRegistry, listOfficialInstitutions } from '../gov/api';
import type { InstitutionDetail } from '../subjects/api';

export interface FederalAdminsViewProps {
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

const ADMIN_LIST_CACHE_VERSION = 'sfid-admin-list-v1';

interface CachedAdminListPayload<T> {
  version: string;
  rows: T[];
}

function adminListCacheKey(
  kind: 'federal-admins' | 'city-admins',
  auth: AdminAuth,
  mode: FederalAdminsViewProps['mode'],
): string {
  return [
    'sfid:admin-list',
    ADMIN_LIST_CACHE_VERSION,
    kind,
    auth.admin_pubkey,
    auth.role,
    auth.admin_province || 'ALL',
    auth.admin_city || 'ALL',
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

export function FederalAdminsView({ mode }: FederalAdminsViewProps) {
  const { auth } = useAuth();

  const [federalAdmins, setFederalAdmins] = useState<FederalAdminRow[]>([]);
  const [federalAdminsLoading, setFederalAdminsLoading] = useState(false);
  const [selectedFederalAdmin, setSelectedFederalAdmin] = useState<FederalAdminRow | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [adminDetailTab, setAdminDetailTab] = useState<'city-admin' | 'federal-admin'>('city-admin');
  const adminDetailTabRef = useRef<'city-admin' | 'federal-admin'>('city-admin');
  const lastSelectedFederalAdminKey = useRef<string | null>(null);

  const [cityAdmins, setCityAdmins] = useState<CityAdminRow[]>([]);
  const [cityAdminsLoading, setCityAdminsLoading] = useState(false);
  const [cityAdminListPage, setCityAdminListPage] = useState(1);

  const initialCityAdminCities = auth?.admin_province
    ? readCachedSfidCities(auth.admin_province)
    : null;
  const [cityAdminCities, setCityAdminCities] = useState<SfidCityItem[]>(initialCityAdminCities ?? []);
  const [cityAdminCitiesLoading, setCityAdminCitiesLoading] = useState(
    !!auth?.admin_province && !initialCityAdminCities,
  );

  const [addCityAdminOpen, setAddCityAdminOpen] = useState(false);
  const [addCityAdminLoading, setAddCityAdminLoading] = useState(false);

  const [accountScanTarget, setAccountScanTarget] = useState<AccountScanTarget>(null);

  // ── 注册局机构详情页数据源(任务卡 20260608) ──
  // 联邦注册局:全国唯一机构,走 scope-bypass 接口,所有联邦管理员可读。
  const [federalRegistryDetail, setFederalRegistryDetail] = useState<InstitutionDetail | null>(null);
  const [federalRegistryLoading, setFederalRegistryLoading] = useState(false);
  // 市注册局:当前活动市对应的机构 sfid_number。
  const [cityRegistrySfid, setCityRegistrySfid] = useState<string | null>(null);
  const [cityRegistryLoading, setCityRegistryLoading] = useState(false);

  // 活动省/市:联邦管理员看 selectedFederalAdmin + selectedCity;市管理员锁定本省本市。
  const activeProvince = selectedFederalAdmin?.province ?? auth?.admin_province ?? null;
  const activeCity = selectedCity ?? (auth?.role === 'CITY_ADMIN' ? auth?.admin_city ?? null : null);

  const [addCityAdminForm] = Form.useForm<{ city_admin_pubkey: string; city_admin_name: string; city_admin_city: string }>();
  const [adminActionModal, setAdminActionModal] = useState<AdminActionModalState | null>(null);
  const [adminActionLoading, setAdminActionLoading] = useState(false);
  const [adminActionCommitLoading, setAdminActionCommitLoading] = useState(false);

  useEffect(() => {
    adminDetailTabRef.current = adminDetailTab;
  }, [adminDetailTab]);

  // ── 数据加载 ──

  const refreshFederalAdmins = async (): Promise<FederalAdminRow[]> => {
    if (!auth) return [];
    const cacheKey = adminListCacheKey('federal-admins', auth, mode);
    const cached = readCachedAdminList<FederalAdminRow>(cacheKey);
    if (cached !== null) {
      setFederalAdmins(cached);
      setFederalAdminsLoading(false);
    } else {
      setFederalAdminsLoading(true);
    }
    try {
      const rows = await listFederalAdmins(auth);
      const list = Array.isArray(rows) ? rows : [];
      setFederalAdmins(list);
      writeCachedAdminList(cacheKey, list);
      return list;
    } catch (err) {
      notice.error(err, '加载联邦管理员失败');
      return cached ?? [];
    } finally {
      if (cached === null) setFederalAdminsLoading(false);
    }
  };

  const refreshCityAdmins = async (): Promise<CityAdminRow[]> => {
    if (!auth) return [];
    const cacheKey = adminListCacheKey('city-admins', auth, mode);
    const cached = readCachedAdminList<CityAdminRow>(cacheKey);
    if (cached !== null) {
      setCityAdmins(cached);
      setCityAdminsLoading(false);
    } else {
      setCityAdminsLoading(true);
    }
    try {
      const rows = await listCityAdmins(auth);
      const list = Array.isArray(rows) ? rows : [];
      setCityAdmins(list);
      writeCachedAdminList(cacheKey, list);
      return list;
    } catch (err) {
      notice.error(err, '加载市管理员失败');
      return cached ?? [];
    } finally {
      if (cached === null) setCityAdminsLoading(false);
    }
  };

  // 首次挂载 / auth 变化时加载数据。
  // 角色分流由 ProvinceDetailView + useScope 自动处理,这里只负责加载数据
  // 和按当前登录角色定位 selectedFederalAdmin。
  useEffect(() => {
    let cancelled = false;
    const init = async () => {
      if (!auth) return;
      // 注册局视图(city-registry / federal-registry):联邦/市管理员数据都要
      const [rows, ops] = await Promise.all([refreshFederalAdmins(), refreshCityAdmins()]);
      if (cancelled) return;
      // 自动定位到当前登录角色所属省的 FederalAdmin
      if (!selectedFederalAdmin) {
        let target: FederalAdminRow | null = null;
        if (auth.role === 'FEDERAL_ADMIN') {
          target = rows.find((r) => sameHexPubkey(r.admin_pubkey, auth.admin_pubkey)) || null;
        } else if (auth.role === 'CITY_ADMIN') {
          const me = ops.find((o) => sameHexPubkey(o.admin_pubkey, auth.admin_pubkey));
          if (me) {
            target = rows.find((r) => sameHexPubkey(r.admin_pubkey, me.created_by)) || null;
          }
        }
        if (!cancelled && target) setSelectedFederalAdmin(target);
      }
    };
    void init();
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token, mode]);

  // 切换 selectedFederalAdmin 时:
  //   1. 预加载该机构所属省份的城市列表
  //   2. 重置 sub-tab 到默认(市管理员列表)
  //   3. 重置市管理员列表分页到第 1 页
  useEffect(() => {
    if (!auth) {
      setCityAdminCities([]);
      setCityAdminCitiesLoading(false);
      return;
    }
    if (!selectedFederalAdmin) {
      lastSelectedFederalAdminKey.current = null;
      const cachedRows = auth.admin_province ? readCachedSfidCities(auth.admin_province) : null;
      if (cachedRows) {
        setCityAdminCities(cachedRows);
        setCityAdminCitiesLoading(false);
      } else {
        setCityAdminCities([]);
        setCityAdminCitiesLoading(!!auth.admin_province);
      }
      return;
    }
    const selectedKey = `${selectedFederalAdmin.province}:${selectedFederalAdmin.admin_pubkey}`;
    const isRealProvinceSwitch = lastSelectedFederalAdminKey.current !== null
      && lastSelectedFederalAdminKey.current !== selectedKey;
    lastSelectedFederalAdminKey.current = selectedKey;
    const cachedRows = readCachedSfidCities(selectedFederalAdmin.province);
    if (cachedRows) {
      setCityAdminCities(cachedRows);
      setCityAdminCitiesLoading(false);
    } else {
      setCityAdminCities([]);
      setCityAdminCitiesLoading(true);
    }
    // 中文注释:首次自动定位当前省时不能覆盖用户刚点击的“联邦管理员列表”。
    // 只有用户真正切换到另一个省时,才把子页签重置回市列表。
    if (isRealProvinceSwitch && adminDetailTabRef.current !== 'federal-admin') {
      setAdminDetailTab(auth.passkey_bound === false && auth.role === 'FEDERAL_ADMIN' ? 'federal-admin' : 'city-admin');
    }
    setCityAdminListPage(1);
    let cancelled = false;
    loadCachedSfidCities(auth, selectedFederalAdmin.province)
      .then((rows) => {
        if (!cancelled) setCityAdminCities(rows);
      })
      .catch(() => {
        if (!cancelled) setCityAdminCities([]);
      })
      .finally(() => {
        if (!cancelled) setCityAdminCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedFederalAdmin?.admin_pubkey, auth?.access_token]);

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

  // ── 市注册局机构 sfid 解析:从该市公权机构目录里筛 org_code='CITY_REGISTRY' 那条 ──
  useEffect(() => {
    if (!auth || mode !== 'city-registry' || !activeProvince || !activeCity) {
      setCityRegistrySfid(null);
      setCityRegistryLoading(false);
      return;
    }
    let cancelled = false;
    setCityRegistrySfid(null);
    setCityRegistryLoading(true);
    listOfficialInstitutions(auth, { province_name: activeProvince, city_name: activeCity, page_size: 300 })
      .then((res) => {
        if (cancelled) return;
        const row = res.items.find((r) => r.org_code === 'CITY_REGISTRY');
        setCityRegistrySfid(row?.sfid_number ?? null);
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

  const onCreateCityAdmin = async (values: { city_admin_pubkey: string; city_admin_name: string; city?: string }) => {
    if (!auth) return;
    const inputAddr = values.city_admin_pubkey?.trim();
    const admin_name = values.city_admin_name?.trim();
    const city = (values.city ?? '').trim();
    if (!inputAddr) {
      notice.error('请输入管理员账户');
      return;
    }
    if (!admin_name) {
      notice.error('请输入管理员姓名');
      return;
    }
    if (!city) {
      notice.error('请选择市');
      return;
    }
    const cityCityAdminCount = cityAdmins.filter((item) => item.city === city).length;
    if (cityCityAdminCount >= MAX_CITY_ADMINS_PER_CITY) {
      notice.error(`本市市管理员已满 ${MAX_CITY_ADMINS_PER_CITY} 人，不能继续新增`);
      return;
    }
    let admin_pubkey: string;
    try {
      admin_pubkey = decodeSs58(inputAddr);
    } catch (err) {
      notice.error(err, '');
      return;
    }
    setAddCityAdminLoading(true);
    try {
      const created = await runSecuredAction<CityAdminRow>('CREATE_CITY_ADMIN', {
        admin_pubkey,
        admin_name,
        city,
      });
      notice.success('管理员新增成功');
      addCityAdminForm.resetFields();
      setAddCityAdminOpen(false);
      setCityAdmins((prev) => {
        const rest = prev.filter((item) => item.admin_pubkey !== created.admin_pubkey);
        return [created, ...rest];
      });
      await refreshCityAdmins();
    } catch (err) {
      const msg = formatAdminCreateError(err, 'CITY_ADMIN', '新增管理员失败');
      notice.error(msg);
    } finally {
      setAddCityAdminLoading(false);
    }
  };

  const onUpdateCityAdmin = (row: CityAdminRow) => {
    if (!auth) return;
    let nextName = row.admin_name;
    const ss58Address = tryEncodeSs58(row.admin_pubkey);
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>编辑市管理员</div>,
      icon: null,
      centered: true,
      zIndex: SFID_MODAL_Z_INDEX.business,
      footer: centeredConfirmFooter,
      content: (
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div>
            <Typography.Text type="secondary">管理员姓名</Typography.Text>
            <Input
              defaultValue={row.admin_name}
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
        const admin_name = nextName.trim();
        if (!admin_name) {
          notice.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        setCityAdminsLoading(true);
        try {
          await updateCityAdminName(auth, row.id, admin_name);
          notice.success('市管理员信息已更新');
          await refreshCityAdmins();
        } catch (err) {
          notice.error(err, '更新市管理员信息失败');
          throw err;
        } finally {
          setCityAdminsLoading(false);
        }
      },
    });
  };

  const onDeleteCityAdmin = (row: CityAdminRow) => {
    if (!auth) return;
    const ss58Address = tryEncodeSs58(row.admin_pubkey);
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>删除市管理员</div>,
      icon: null,
      centered: true,
      zIndex: SFID_MODAL_Z_INDEX.business,
      footer: centeredConfirmFooter,
      content: (
        <div style={{ textAlign: 'center' }}>
          <Typography.Paragraph style={{ marginBottom: 8 }}>确认删除该市管理员?</Typography.Paragraph>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>{ss58Address}</Typography.Text>
        </div>
      ),
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setCityAdminsLoading(true);
        try {
          await runSecuredAction('DELETE_CITY_ADMIN', { id: row.id });
          notice.success('市管理员已删除');
          await refreshCityAdmins();
        } catch (err) {
          notice.error(err, '删除市管理员失败');
          throw err;
        } finally {
          setCityAdminsLoading(false);
        }
      },
    });
  };

  // ── 组装共享状态 ──

  const shared: FederalAdminSharedState = {
    federalAdmins,
    federalAdminsLoading,
    refreshFederalAdmins,
    selectedFederalAdmin,
    setSelectedFederalAdmin,
    selectedCity,
    setSelectedCity,
    adminDetailTab,
    setAdminDetailTab,
    cityAdmins,
    cityAdminsLoading,
    cityAdminListPage,
    setCityAdminListPage,
    cityAdminCities,
    cityAdminCitiesLoading,
    addCityAdminOpen,
    setAddCityAdminOpen,
    addCityAdminLoading,
    accountScanTarget,
    setAccountScanTarget,
    addCityAdminForm,
    onCreateCityAdmin,
    onUpdateCityAdmin,
    onDeleteCityAdmin,
    runSecuredAction,
    federalRegistryDetail,
    federalRegistryLoading,
    cityRegistrySfid,
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
