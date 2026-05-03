// 省级管理员视图 —— 调度器:持有所有状态和副作用,
// 按 mode 分派到 ShengAdminListView / ProvinceDetailView。
// system-settings 两层导航:
//   - 省管理员: 市列表 → 市详情(该市管理员列表)
//   - 市管理员: 直接进入自己所在市的管理员列表(不显示省列表和市列表)

import { useEffect, useState } from 'react';
import { Form, Input, Modal, Select, Space, message } from 'antd';
import { useAuth } from '../hooks/useAuth';
import type { OperatorRow } from '../shi_admins/api';
import type { ShengAdminRow } from './api';
import type { SfidCityItem } from '../sfid/api';
import {
  createOperator,
  deleteOperator,
  listOperators,
  updateOperator,
  updateOperatorStatus,
} from '../shi_admins/api';
import { listShengAdmins } from './api';
import { listSfidCities } from '../sfid/api';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { sameHexPubkey } from './shengAdminUtils';
import type { AccountScanTarget, ShengAdminSharedState } from './shengAdminUtils';
import { ShengAdminListView } from './ShengAdminListView';
import { ProvinceDetailView } from './ProvinceDetailView';

export interface ShengAdminsViewProps {
  /// 'list' = 顶层 sheng_admin 列表分支(全省网格);
  /// 'system-settings' = 注册局分支(省份网格 / 机构详情页)
  mode: 'list' | 'system-settings';
}

export function ShengAdminsView({ mode }: ShengAdminsViewProps) {
  const { auth } = useAuth();

  const [shengAdmins, setShengAdmins] = useState<ShengAdminRow[]>([]);
  const [shengAdminsLoading, setShengAdminsLoading] = useState(false);
  const [selectedShengAdmin, setSelectedShengAdmin] = useState<ShengAdminRow | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [adminDetailTab, setAdminDetailTab] = useState<'operators' | 'super-admin'>('operators');

  const [operators, setOperators] = useState<OperatorRow[]>([]);
  const [operatorsLoading, setOperatorsLoading] = useState(false);
  const [operatorListPage, setOperatorListPage] = useState(1);

  const [operatorCities, setOperatorCities] = useState<SfidCityItem[]>([]);
  const [operatorCitiesLoading, setOperatorCitiesLoading] = useState(false);

  const [addOperatorOpen, setAddOperatorOpen] = useState(false);
  const [addOperatorLoading, setAddOperatorLoading] = useState(false);

  const [accountScanTarget, setAccountScanTarget] = useState<AccountScanTarget>(null);

  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string; operator_city: string }>();

  // ── 数据加载 ──

  const refreshShengAdmins = async (): Promise<ShengAdminRow[]> => {
    if (!auth) return [];
    setShengAdminsLoading(true);
    try {
      const rows = await listShengAdmins(auth);
      const list = Array.isArray(rows) ? rows : [];
      setShengAdmins(list);
      return list;
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载省级管理员失败';
      message.error(msg);
      return [];
    } finally {
      setShengAdminsLoading(false);
    }
  };

  const refreshOperators = async (): Promise<OperatorRow[]> => {
    if (!auth) return [];
    setOperatorsLoading(true);
    try {
      const rows = await listOperators(auth);
      const list = Array.isArray(rows) ? rows : [];
      setOperators(list);
      return list;
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载市级管理员失败';
      message.error(msg);
      return [];
    } finally {
      setOperatorsLoading(false);
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
      // system-settings
      const [rows, ops] = await Promise.all([refreshShengAdmins(), refreshOperators()]);
      if (cancelled) return;
      // 自动定位到当前登录角色所属省的 ShengAdmin
      if (!selectedShengAdmin) {
        let target: ShengAdminRow | null = null;
        if (auth.role === 'SHENG_ADMIN') {
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
    if (!selectedShengAdmin || !auth) {
      setOperatorCities([]);
      return;
    }
    setOperatorCities([]);
    setAdminDetailTab('operators');
    setOperatorListPage(1);
    setOperatorCitiesLoading(true);
    let cancelled = false;
    listSfidCities(auth, selectedShengAdmin.province)
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

  // ── 事件处理 ──

  const onCreateOperator = async (values: { operator_pubkey: string; operator_name: string; city?: string; created_by?: string }) => {
    if (!auth) return;
    const inputAddr = values.operator_pubkey?.trim();
    const admin_name = values.operator_name?.trim();
    const city = (values.city ?? '').trim();
    if (!inputAddr) {
      message.error('请输入管理员账户');
      return;
    }
    if (!admin_name) {
      message.error('请输入管理员姓名');
      return;
    }
    if (!city) {
      message.error('请选择市');
      return;
    }
    let admin_pubkey: string;
    try {
      admin_pubkey = decodeSs58(inputAddr);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setAddOperatorLoading(true);
    try {
      const created = await createOperator(auth, { admin_pubkey, admin_name, city, created_by: values.created_by });
      message.success('管理员新增成功');
      addOperatorForm.resetFields();
      setAddOperatorOpen(false);
      setOperators((prev) => {
        const rest = prev.filter((item) => item.admin_pubkey !== created.admin_pubkey);
        return [created, ...rest];
      });
      await refreshOperators();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '新增管理员失败';
      message.error(msg);
    } finally {
      setAddOperatorLoading(false);
    }
  };

  const onToggleOperatorStatus = async (row: OperatorRow) => {
    if (!auth) return;
    const target = row.status === 'ACTIVE' ? 'DISABLED' : 'ACTIVE';
    setOperatorsLoading(true);
    try {
      await updateOperatorStatus(auth, row.id, target);
      message.success(target === 'ACTIVE' ? '已启用市级管理员' : '已停用市级管理员');
      await refreshOperators();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '更新市级管理员状态失败';
      message.error(msg);
    } finally {
      setOperatorsLoading(false);
    }
  };

  const onUpdateOperator = (row: OperatorRow) => {
    if (!auth) return;
    let nextName = row.admin_name;
    let nextAddr = tryEncodeSs58(row.admin_pubkey);
    let nextCity = row.city;
    const cityOptions = operatorCities
      .filter((c) => c.code !== '000')
      .map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }));
    Modal.confirm({
      title: '修改市级管理员',
      content: (
        <Space direction="vertical" style={{ width: '100%' }}>
          <Input
            defaultValue={row.admin_name}
            placeholder="请输入管理员姓名"
            onChange={(event) => {
              nextName = event.target.value;
            }}
          />
          <Select
            defaultValue={row.city || undefined}
            placeholder="请选择市"
            style={{ width: '100%' }}
            options={cityOptions}
            onChange={(value: string) => {
              nextCity = value;
            }}
          />
          <Input
            defaultValue={tryEncodeSs58(row.admin_pubkey)}
            placeholder="请输入新的管理员账户(SS58)"
            onChange={(event) => {
              nextAddr = event.target.value;
            }}
          />
        </Space>
      ),
      okText: '确认修改',
      cancelText: '取消',
      onOk: async () => {
        const admin_name = nextName.trim();
        const inputAddr = nextAddr.trim();
        const city = (nextCity || '').trim();
        if (!admin_name) {
          message.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        if (!inputAddr) {
          message.error('请输入管理员账户');
          throw new Error('admin_pubkey is required');
        }
        if (!city) {
          message.error('请选择市');
          throw new Error('city is required');
        }
        let admin_pubkey: string;
        try {
          admin_pubkey = decodeSs58(inputAddr);
        } catch (err) {
          message.error(err instanceof Error ? err.message : '账户格式无效');
          throw err;
        }
        setOperatorsLoading(true);
        try {
          await updateOperator(auth, row.id, { admin_name, admin_pubkey, city });
          message.success('市级管理员信息已更新');
          await refreshOperators();
        } catch (err) {
          const msg = err instanceof Error ? err.message : '更新市级管理员信息失败';
          message.error(msg);
          throw err;
        } finally {
          setOperatorsLoading(false);
        }
      },
    });
  };

  const onDeleteOperator = (row: OperatorRow) => {
    if (!auth) return;
    Modal.confirm({
      title: '删除市级管理员',
      content: `确认删除该市级管理员?\n${row.admin_pubkey}`,
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setOperatorsLoading(true);
        try {
          await deleteOperator(auth, row.id);
          message.success('市级管理员已删除');
          await refreshOperators();
        } catch (err) {
          const msg = err instanceof Error ? err.message : '删除市级管理员失败';
          message.error(msg);
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
    onToggleOperatorStatus,
    onUpdateOperator,
    onDeleteOperator,
  };

  // ── 渲染:按 mode 分派 ──

  if (mode === 'list') {
    return <ShengAdminListView state={shared} />;
  }
  return <ProvinceDetailView state={shared} />;
}
