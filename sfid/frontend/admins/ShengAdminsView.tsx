// 注册局视图 —— 调度器:持有所有状态和副作用,按 mode 分派:
//   - 'list'             → ShengAdminListView(顶层全省网格,旧入口)
//   - 'city-registry'    → CityRegistryView(市注册局 tab:城市网格→市注册局机构详情页)
//   - 'federal-registry' → FederalRegistryView(联邦注册局 tab:联邦注册局机构详情页)
// 联邦管理员城市网格→点市进详情;市级管理员直接进本市/本省详情。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Form, Input, Modal, Space, Typography } from 'antd';
import type { ModalProps } from 'antd';
import { useAuth } from '../hooks/useAuth';
import type { AdminAuth } from '../auth/types';
import type { OperatorRow } from './operators_api';
import type { ShengAdminRow } from './api';
import type { SfidCityItem } from '../china/api';
import { listOperators, updateOperatorName } from './operators_api';
import {
  commitAdminAction,
  formatAdminCreateError,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
} from './admin_security_api';
import { listShengAdmins } from './api';
import { loadCachedSfidCities, readCachedSfidCities } from '../china/metaCache';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { MAX_SHI_ADMINS_PER_CITY, sameHexPubkey } from './shengAdminUtils';
import type { AccountScanTarget, ShengAdminSharedState } from './shengAdminUtils';
import { ShengAdminListView } from './ShengAdminListView';
import { CityRegistryView, FederalRegistryView } from './ProvinceDetailView';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { WuminSignatureModal } from '../core/WuminSignatureModal';
import { SFID_MODAL_Z_INDEX } from '../core/modalStack';
import { notice } from '../utils/notice';
import { getFederalRegistry, listOfficialInstitutions } from '../gov/api';
import type { InstitutionDetail } from '../subjects/api';

export interface ShengAdminsViewProps {
  /// 'list' = 顶层 sheng_admin 列表分支(全省网格);
  /// 'city-registry' = 市注册局 tab(城市网格→市注册局机构详情页);
  /// 'federal-registry' = 联邦注册局 tab(联邦注册局机构详情页)
  mode: 'list' | 'city-registry' | 'federal-registry';
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
  kind: 'sheng-admins' | 'city-admins',
  auth: AdminAuth,
  mode: ShengAdminsViewProps['mode'],
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

export function ShengAdminsView({ mode }: ShengAdminsViewProps) {
  const { auth } = useAuth();

  const [shengAdmins, setShengAdmins] = useState<ShengAdminRow[]>([]);
  const [shengAdminsLoading, setShengAdminsLoading] = useState(false);
  const [selectedShengAdmin, setSelectedShengAdmin] = useState<ShengAdminRow | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [adminDetailTab, setAdminDetailTab] = useState<'operators' | 'sheng-admin'>('operators');
  const adminDetailTabRef = useRef<'operators' | 'sheng-admin'>('operators');
  const lastSelectedShengAdminKey = useRef<string | null>(null);

  const [operators, setOperators] = useState<OperatorRow[]>([]);
  const [operatorsLoading, setOperatorsLoading] = useState(false);
  const [operatorListPage, setOperatorListPage] = useState(1);

  const initialOperatorCities = auth?.admin_province
    ? readCachedSfidCities(auth.admin_province)
    : null;
  const [operatorCities, setOperatorCities] = useState<SfidCityItem[]>(initialOperatorCities ?? []);
  const [operatorCitiesLoading, setOperatorCitiesLoading] = useState(
    !!auth?.admin_province && !initialOperatorCities,
  );

  const [addOperatorOpen, setAddOperatorOpen] = useState(false);
  const [addOperatorLoading, setAddOperatorLoading] = useState(false);

  const [accountScanTarget, setAccountScanTarget] = useState<AccountScanTarget>(null);

  // ── 注册局机构详情页数据源(任务卡 20260608) ──
  // 联邦注册局:全国唯一机构,走 scope-bypass 接口,所有省管理员可读。
  const [federalRegistryDetail, setFederalRegistryDetail] = useState<InstitutionDetail | null>(null);
  const [federalRegistryLoading, setFederalRegistryLoading] = useState(false);
  // 市注册局:当前活动市对应的机构 sfid_number。
  const [cityRegistrySfid, setCityRegistrySfid] = useState<string | null>(null);
  const [cityRegistryLoading, setCityRegistryLoading] = useState(false);

  // 活动省/市:联邦管理员看 selectedShengAdmin + selectedCity;市管理员锁定本省本市。
  const activeProvince = selectedShengAdmin?.province ?? auth?.admin_province ?? null;
  const activeCity = selectedCity ?? (auth?.role === 'SHI_ADMIN' ? auth?.admin_city ?? null : null);

  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string; operator_city: string }>();
  const [adminActionModal, setAdminActionModal] = useState<AdminActionModalState | null>(null);
  const [adminActionLoading, setAdminActionLoading] = useState(false);
  const [adminActionCommitLoading, setAdminActionCommitLoading] = useState(false);

  useEffect(() => {
    adminDetailTabRef.current = adminDetailTab;
  }, [adminDetailTab]);

  // ── 数据加载 ──

  const refreshShengAdmins = async (): Promise<ShengAdminRow[]> => {
    if (!auth) return [];
    const cacheKey = adminListCacheKey('sheng-admins', auth, mode);
    const cached = readCachedAdminList<ShengAdminRow>(cacheKey);
    if (cached !== null) {
      setShengAdmins(cached);
      setShengAdminsLoading(false);
    } else {
      setShengAdminsLoading(true);
    }
    try {
      const rows = await listShengAdmins(auth);
      const list = Array.isArray(rows) ? rows : [];
      setShengAdmins(list);
      writeCachedAdminList(cacheKey, list);
      return list;
    } catch (err) {
      notice.error(err, '加载联邦管理员失败');
      return cached ?? [];
    } finally {
      if (cached === null) setShengAdminsLoading(false);
    }
  };

  const refreshOperators = async (): Promise<OperatorRow[]> => {
    if (!auth) return [];
    const cacheKey = adminListCacheKey('city-admins', auth, mode);
    const cached = readCachedAdminList<OperatorRow>(cacheKey);
    if (cached !== null) {
      setOperators(cached);
      setOperatorsLoading(false);
    } else {
      setOperatorsLoading(true);
    }
    try {
      const rows = await listOperators(auth);
      const list = Array.isArray(rows) ? rows : [];
      setOperators(list);
      writeCachedAdminList(cacheKey, list);
      return list;
    } catch (err) {
      notice.error(err, '加载市级管理员失败');
      return cached ?? [];
    } finally {
      if (cached === null) setOperatorsLoading(false);
    }
  };

  // 首次挂载 / auth 变化时加载数据。
  // 角色分流由 ProvinceDetailView + useScope 自动处理,这里只负责加载数据
  // 和按当前登录角色定位 selectedShengAdmin。
  useEffect(() => {
    let cancelled = false;
    const init = async () => {
      if (!auth) return;
      if (mode === 'list') {
        await refreshShengAdmins();
        return;
      }
      // 注册局视图(city-registry / federal-registry):联邦/市级管理员数据都要
      const [rows, ops] = await Promise.all([refreshShengAdmins(), refreshOperators()]);
      if (cancelled) return;
      // 自动定位到当前登录角色所属省的 ShengAdmin
      if (!selectedShengAdmin) {
        let target: ShengAdminRow | null = null;
        if (auth.role === 'FEDERAL_ADMIN') {
          target = rows.find((r) => sameHexPubkey(r.admin_pubkey, auth.admin_pubkey)) || null;
        } else if (auth.role === 'SHI_ADMIN') {
          const me = ops.find((o) => sameHexPubkey(o.admin_pubkey, auth.admin_pubkey));
          if (me) {
            target = rows.find((r) => sameHexPubkey(r.admin_pubkey, me.created_by)) || null;
          }
        }
        if (!cancelled && target) setSelectedShengAdmin(target);
      }
    };
    void init();
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token, mode]);

  // 切换 selectedShengAdmin 时:
  //   1. 预加载该机构所属省份的城市列表
  //   2. 重置 sub-tab 到默认(市级管理员列表)
  //   3. 重置市级管理员列表分页到第 1 页
  useEffect(() => {
    if (!auth) {
      setOperatorCities([]);
      setOperatorCitiesLoading(false);
      return;
    }
    if (!selectedShengAdmin) {
      lastSelectedShengAdminKey.current = null;
      const cachedRows = auth.admin_province ? readCachedSfidCities(auth.admin_province) : null;
      if (cachedRows) {
        setOperatorCities(cachedRows);
        setOperatorCitiesLoading(false);
      } else {
        setOperatorCities([]);
        setOperatorCitiesLoading(!!auth.admin_province);
      }
      return;
    }
    const selectedKey = `${selectedShengAdmin.province}:${selectedShengAdmin.admin_pubkey}`;
    const isRealProvinceSwitch = lastSelectedShengAdminKey.current !== null
      && lastSelectedShengAdminKey.current !== selectedKey;
    lastSelectedShengAdminKey.current = selectedKey;
    const cachedRows = readCachedSfidCities(selectedShengAdmin.province);
    if (cachedRows) {
      setOperatorCities(cachedRows);
      setOperatorCitiesLoading(false);
    } else {
      setOperatorCities([]);
      setOperatorCitiesLoading(true);
    }
    // 中文注释:首次自动定位当前省时不能覆盖用户刚点击的“联邦管理员列表”。
    // 只有用户真正切换到另一个省时,才把子页签重置回市列表。
    if (isRealProvinceSwitch && adminDetailTabRef.current !== 'sheng-admin') {
      setAdminDetailTab(auth.passkey_bound === false && auth.role === 'FEDERAL_ADMIN' ? 'sheng-admin' : 'operators');
    }
    setOperatorListPage(1);
    let cancelled = false;
    loadCachedSfidCities(auth, selectedShengAdmin.province)
      .then((rows) => {
        if (!cancelled) setOperatorCities(rows);
      })
      .catch(() => {
        if (!cancelled) setOperatorCities([]);
      })
      .finally(() => {
        if (!cancelled) setOperatorCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedShengAdmin?.admin_pubkey, auth?.access_token]);

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
    listOfficialInstitutions(auth, { province: activeProvince, city: activeCity, page_size: 300 })
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
      if (!signRequest) throw new Error('该治理操作缺少冷钱包签名请求');
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

  const onCreateOperator = async (values: { operator_pubkey: string; operator_name: string; city?: string }) => {
    if (!auth) return;
    const inputAddr = values.operator_pubkey?.trim();
    const admin_name = values.operator_name?.trim();
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
    const cityOperatorCount = operators.filter((item) => item.city === city).length;
    if (cityOperatorCount >= MAX_SHI_ADMINS_PER_CITY) {
      notice.error(`本市市级管理员已满 ${MAX_SHI_ADMINS_PER_CITY} 人，不能继续新增`);
      return;
    }
    let admin_pubkey: string;
    try {
      admin_pubkey = decodeSs58(inputAddr);
    } catch (err) {
      notice.error(err, '');
      return;
    }
    setAddOperatorLoading(true);
    try {
      const created = await runSecuredAction<OperatorRow>('CREATE_OPERATOR', {
        admin_pubkey,
        admin_name,
        city,
      });
      notice.success('管理员新增成功');
      addOperatorForm.resetFields();
      setAddOperatorOpen(false);
      setOperators((prev) => {
        const rest = prev.filter((item) => item.admin_pubkey !== created.admin_pubkey);
        return [created, ...rest];
      });
      await refreshOperators();
    } catch (err) {
      const msg = formatAdminCreateError(err, 'SHI_ADMIN', '新增管理员失败');
      notice.error(msg);
    } finally {
      setAddOperatorLoading(false);
    }
  };

  const onUpdateOperator = (row: OperatorRow) => {
    if (!auth) return;
    let nextName = row.admin_name;
    const ss58Address = tryEncodeSs58(row.admin_pubkey);
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>编辑市级管理员</div>,
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
        setOperatorsLoading(true);
        try {
          await updateOperatorName(auth, row.id, admin_name);
          notice.success('市级管理员信息已更新');
          await refreshOperators();
        } catch (err) {
          notice.error(err, '更新市级管理员信息失败');
          throw err;
        } finally {
          setOperatorsLoading(false);
        }
      },
    });
  };

  const onDeleteOperator = (row: OperatorRow) => {
    if (!auth) return;
    const ss58Address = tryEncodeSs58(row.admin_pubkey);
    notice.confirm({
      title: <div style={{ textAlign: 'center', width: '100%' }}>删除市级管理员</div>,
      icon: null,
      centered: true,
      zIndex: SFID_MODAL_Z_INDEX.business,
      footer: centeredConfirmFooter,
      content: (
        <div style={{ textAlign: 'center' }}>
          <Typography.Paragraph style={{ marginBottom: 8 }}>确认删除该市级管理员?</Typography.Paragraph>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>{ss58Address}</Typography.Text>
        </div>
      ),
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setOperatorsLoading(true);
        try {
          await runSecuredAction('DELETE_OPERATOR', { id: row.id });
          notice.success('市级管理员已删除');
          await refreshOperators();
        } catch (err) {
          notice.error(err, '删除市级管理员失败');
          throw err;
        } finally {
          setOperatorsLoading(false);
        }
      },
    });
  };

  // ── 组装共享状态 ──

  const shared: ShengAdminSharedState = {
    shengAdmins,
    shengAdminsLoading,
    refreshShengAdmins,
    selectedShengAdmin,
    setSelectedShengAdmin,
    selectedCity,
    setSelectedCity,
    adminDetailTab,
    setAdminDetailTab,
    operators,
    operatorsLoading,
    operatorListPage,
    setOperatorListPage,
    operatorCities,
    operatorCitiesLoading,
    addOperatorOpen,
    setAddOperatorOpen,
    addOperatorLoading,
    accountScanTarget,
    setAccountScanTarget,
    addOperatorForm,
    onCreateOperator,
    onUpdateOperator,
    onDeleteOperator,
    runSecuredAction,
    federalRegistryDetail,
    federalRegistryLoading,
    cityRegistrySfid,
    cityRegistryLoading,
  };

  // ── 渲染:按 mode 分派 ──

  const content = mode === 'list'
    ? <ShengAdminListView state={shared} />
    : mode === 'city-registry'
      ? <CityRegistryView state={shared} />
      : <FederalRegistryView state={shared} />;

  return (
    <>
      {content}
      <WuminSignatureModal
        title="冷钱包签名确认"
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
