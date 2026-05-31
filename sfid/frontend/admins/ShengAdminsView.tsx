// 省级管理员视图 —— 调度器:持有所有状态和副作用,
// 按 mode 分派到 ShengAdminListView / ProvinceDetailView。
// system-settings 两层导航:
//   - 省管理员: 市列表 → 市详情(该市管理员列表)
//   - 市管理员: 直接进入自己所在市的管理员列表(不显示省列表和市列表)

import { useCallback, useEffect, useState } from 'react';
import { Form, Input, Modal, Space, Typography, message } from 'antd';
import type { ModalProps } from 'antd';
import { useAuth } from '../hooks/useAuth';
import type { OperatorRow } from './operators_api';
import type { ShengAdminRow } from './api';
import type { SfidCityItem } from '../sfid/api';
import { listOperators, updateOperatorName } from './operators_api';
import {
  commitAdminAction,
  formatAdminCreateError,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
} from './admin_security_api';
import { listShengAdmins } from './api';
import { listSfidCities } from '../sfid/api';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { MAX_SHI_ADMINS_PER_CITY, sameHexPubkey } from './shengAdminUtils';
import type { AccountScanTarget, ShengAdminSharedState } from './shengAdminUtils';
import { ShengAdminListView } from './ShengAdminListView';
import { ProvinceDetailView } from './ProvinceDetailView';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { WuminSignatureModal } from '../common/WuminSignatureModal';
import { SFID_MODAL_Z_INDEX } from '../common/modalStack';

export interface ShengAdminsViewProps {
  /// 'list' = 顶层 sheng_admin 列表分支(全省网格);
  /// 'system-settings' = 注册局分支(省份网格 / 机构详情页)
  mode: 'list' | 'system-settings';
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

export function ShengAdminsView({ mode }: ShengAdminsViewProps) {
  const { auth } = useAuth();

  const [shengAdmins, setShengAdmins] = useState<ShengAdminRow[]>([]);
  const [shengAdminsLoading, setShengAdminsLoading] = useState(false);
  const [selectedShengAdmin, setSelectedShengAdmin] = useState<ShengAdminRow | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [adminDetailTab, setAdminDetailTab] = useState<'operators' | 'sheng-admin'>('operators');

  const [operators, setOperators] = useState<OperatorRow[]>([]);
  const [operatorsLoading, setOperatorsLoading] = useState(false);
  const [operatorListPage, setOperatorListPage] = useState(1);

  const [operatorCities, setOperatorCities] = useState<SfidCityItem[]>([]);
  const [operatorCitiesLoading, setOperatorCitiesLoading] = useState(false);

  const [addOperatorOpen, setAddOperatorOpen] = useState(false);
  const [addOperatorLoading, setAddOperatorLoading] = useState(false);

  const [accountScanTarget, setAccountScanTarget] = useState<AccountScanTarget>(null);

  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string; operator_city: string }>();
  const [adminActionModal, setAdminActionModal] = useState<AdminActionModalState | null>(null);
  const [adminActionLoading, setAdminActionLoading] = useState(false);
  const [adminActionCommitLoading, setAdminActionCommitLoading] = useState(false);

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
    setAdminDetailTab(auth.passkey_bound === false && auth.role === 'SHENG_ADMIN' ? 'sheng-admin' : 'operators');
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
      const msg = error instanceof Error ? error.message : '签名回执处理失败';
      message.error(msg);
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
    const cityOperatorCount = operators.filter((item) => item.city === city).length;
    if (cityOperatorCount >= MAX_SHI_ADMINS_PER_CITY) {
      message.error(`本市市级管理员已满 ${MAX_SHI_ADMINS_PER_CITY} 人，不能继续新增`);
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
      const created = await runSecuredAction<OperatorRow>('CREATE_OPERATOR', {
        admin_pubkey,
        admin_name,
        city,
      });
      message.success('管理员新增成功');
      addOperatorForm.resetFields();
      setAddOperatorOpen(false);
      setOperators((prev) => {
        const rest = prev.filter((item) => item.admin_pubkey !== created.admin_pubkey);
        return [created, ...rest];
      });
      await refreshOperators();
    } catch (err) {
      const msg = formatAdminCreateError(err, 'SHI_ADMIN', '新增管理员失败');
      message.error(msg);
    } finally {
      setAddOperatorLoading(false);
    }
  };

  const onUpdateOperator = (row: OperatorRow) => {
    if (!auth) return;
    let nextName = row.admin_name;
    const ss58Address = tryEncodeSs58(row.admin_pubkey);
    Modal.confirm({
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
          message.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        setOperatorsLoading(true);
        try {
          await updateOperatorName(auth, row.id, admin_name);
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
    const ss58Address = tryEncodeSs58(row.admin_pubkey);
    Modal.confirm({
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
          message.success('市级管理员已删除');
          await refreshOperators();
        } catch (err) {
          const msg = err instanceof Error ? err.message : '删除市级管理员失败';
          message.error(msg);
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
  };

  // ── 渲染:按 mode 分派 ──

  const content = mode === 'list'
    ? <ShengAdminListView state={shared} />
    : <ProvinceDetailView state={shared} />;

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
        onScannerError={(msg) => message.error(msg)}
      />
    </>
  );
}
