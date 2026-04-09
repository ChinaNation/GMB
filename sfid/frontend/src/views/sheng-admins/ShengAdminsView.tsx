// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 2)
// 省级管理员视图 —— 同时接管 activeView === 'sheng-admins'(顶层列表)
// 以及 activeView === 'system-settings'(注册局)两个分支。
// system-settings 内部又分:
//   - KeyAdmin: 省份网格(点击进入机构详情)
//   - 机构详情页:sub-tab = '市级管理员列表' / '省级管理员'
//     + 新增市级管理员 Modal
// 共用:shengAdmins 列表 / selectedShengAdmin / replaceSuperForm /
//      operators 列表 / addOperatorForm / operatorCities。

import { useEffect, useState } from 'react';
import { Button, Card, Form, Input, Modal, Select, Space, Table, Typography, message } from 'antd';
import { useAuth } from '../../hooks/useAuth';
import type { OperatorRow, ShengAdminRow, SfidCityItem } from '../../api/client';
import {
  createOperator,
  deleteOperator,
  listOperators,
  listSfidCities,
  listShengAdmins,
  replaceShengAdmin,
  updateOperator,
  updateOperatorStatus,
} from '../../api/client';
import { decodeSs58, tryEncodeSs58 } from '../../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';
import { ScanAccountModal } from '../../components/ScanAccountModal';

function isSr25519HexPubkey(value: string): boolean {
  const normalized = value.trim().replace(/^0x/i, '');
  return /^[0-9a-fA-F]{64}$/.test(normalized);
}

function sameHexPubkey(a: string | null | undefined, b: string | null | undefined): boolean {
  if (!a || !b) return false;
  return a.trim().replace(/^0x/i, '').toLowerCase() === b.trim().replace(/^0x/i, '').toLowerCase();
}

export interface ShengAdminsViewProps {
  /// 'list' = 顶层 sheng-admins 列表分支;
  /// 'system-settings' = 注册局分支(KeyAdmin 省份网格 / 机构详情页)
  mode: 'list' | 'system-settings';
}

export function ShengAdminsView({ mode }: ShengAdminsViewProps) {
  const { auth } = useAuth();

  const [shengAdmins, setShengAdmins] = useState<ShengAdminRow[]>([]);
  const [shengAdminsLoading, setShengAdminsLoading] = useState(false);
  const [selectedShengAdmin, setSelectedShengAdmin] = useState<ShengAdminRow | null>(null);
  const [adminDetailTab, setAdminDetailTab] = useState<'operators' | 'super-admin'>('operators');
  const [replaceSuperLoading, setReplaceSuperLoading] = useState(false);

  const [operators, setOperators] = useState<OperatorRow[]>([]);
  const [operatorsLoading, setOperatorsLoading] = useState(false);
  const [operatorListPage, setOperatorListPage] = useState(1);

  const [operatorCities, setOperatorCities] = useState<SfidCityItem[]>([]);
  const [operatorCitiesLoading, setOperatorCitiesLoading] = useState(false);

  const [addOperatorOpen, setAddOperatorOpen] = useState(false);
  const [addOperatorLoading, setAddOperatorLoading] = useState(false);

  const [accountScanTarget, setAccountScanTarget] = useState<null | 'operator' | 'super-admin'>(null);

  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string; operator_city: string }>();
  const [replaceSuperForm] = Form.useForm<{ province: string; admin_pubkey: string }>();

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

  // 首次挂载 / auth 变化时:
  //   - list 模式:只加载 shengAdmins
  //   - system-settings 模式:加载 shengAdmins + operators,
  //     并自动为 ShengAdmin/ShiAdmin 跳到自己的机构详情页(原 openSystemSettings 行为)
  useEffect(() => {
    let cancelled = false;
    const init = async () => {
      if (!auth) return;
      if (mode === 'list') {
        await refreshShengAdmins();
        return;
      }
      // system-settings
      if (auth.role === 'KEY_ADMIN') {
        setSelectedShengAdmin(null);
        await refreshShengAdmins();
        await refreshOperators();
        return;
      }
      const rows = await refreshShengAdmins();
      const ops = await refreshOperators();
      if (cancelled) return;
      let target: ShengAdminRow | null = null;
      if (auth.role === 'SHENG_ADMIN') {
        target = rows.find((r) => sameHexPubkey(r.admin_pubkey, auth.admin_pubkey)) || null;
      } else if (auth.role === 'SHI_ADMIN') {
        const me = ops.find((o) => sameHexPubkey(o.admin_pubkey, auth.admin_pubkey));
        if (me) {
          target = rows.find((r) => sameHexPubkey(r.admin_pubkey, me.created_by)) || null;
        }
      }
      if (!cancelled) setSelectedShengAdmin(target);
    };
    void init();
    return () => {
      cancelled = true;
    };
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

  const onReplaceShengAdmin = async (values: { province: string; admin_pubkey: string }) => {
    if (!auth) return;
    const inputAddr = values.admin_pubkey.trim();
    let hexPubkey: string;
    try {
      hexPubkey = decodeSs58(inputAddr);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setReplaceSuperLoading(true);
    try {
      await replaceShengAdmin(auth, values.province.trim(), hexPubkey);
      message.success(`已更新 ${values.province} 省级管理员`);
      replaceSuperForm.resetFields();
      await refreshShengAdmins();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '更换省级管理员失败';
      message.error(msg);
    } finally {
      setReplaceSuperLoading(false);
    }
  };

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

  // ────────────────────────────────────────────────────────────
  // 渲染
  // ────────────────────────────────────────────────────────────
  if (mode === 'list') {
    return (
      <Card
        title="省级管理员列表"
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
        extra={
          <Form
            form={replaceSuperForm}
            layout="inline"
            onFinish={onReplaceShengAdmin}
            style={{ rowGap: 8 }}
          >
            <Form.Item
              name="province"
              rules={[{ required: true, message: '请选择省份' }]}
              style={{ marginBottom: 0 }}
            >
              <Select
                style={{ width: 160 }}
                placeholder="选择省份"
                options={shengAdmins.map((item) => ({ value: item.province, label: item.province }))}
              />
            </Form.Item>
            <Form.Item
              name="admin_pubkey"
              rules={[
                { required: true, message: '请输入新省级管理员公钥' },
                {
                  validator: async (_rule, value) => {
                    if (!value || isSr25519HexPubkey(String(value))) return;
                    throw new Error('公钥格式必须为 32 字节十六进制');
                  },
                },
              ]}
              style={{ marginBottom: 0 }}
            >
              <Input style={{ width: 420, maxWidth: '60vw' }} placeholder="新省级管理员公钥" />
            </Form.Item>
            <Form.Item style={{ marginBottom: 0 }}>
              <Button type="primary" htmlType="submit" loading={replaceSuperLoading}>
                更换省级管理员
              </Button>
            </Form.Item>
          </Form>
        }
      >
        <Table<ShengAdminRow>
          rowKey={(r) => `${r.province}-${r.admin_pubkey}`}
          loading={shengAdminsLoading}
          dataSource={shengAdmins}
          pagination={{ pageSize: 10 }}
          columns={[
            {
              title: '序号',
              width: 80,
              align: 'center',
              render: (_v: unknown, _row: ShengAdminRow, index: number) => index + 1,
            },
            { title: '省份', dataIndex: 'province', align: 'center', width: 140 },
            { title: '名称', dataIndex: 'admin_name', align: 'center', width: 180 },
            { title: '公钥', dataIndex: 'admin_pubkey', align: 'center' },
            { title: '状态', dataIndex: 'status', align: 'center', width: 100 },
            {
              title: '类型',
              width: 100,
              align: 'center',
              render: (_v: unknown, row: ShengAdminRow) => (row.built_in ? '内置' : '自定义'),
            },
          ]}
        />
      </Card>
    );
  }

  // mode === 'system-settings'
  return (
    <>
      {selectedShengAdmin ? (
        // ── 机构详情页(sub-tab:市级管理员列表 / 省级管理员) ──
        (() => {
          const isKeyAdmin = auth?.role === 'KEY_ADMIN';
          const isSelf = auth ? sameHexPubkey(selectedShengAdmin.admin_pubkey, auth.admin_pubkey) : false;
          const canEditOperators = isKeyAdmin || (auth?.role === 'SHENG_ADMIN' && isSelf);
          const canReplaceThisAdmin = isKeyAdmin;
          const operatorsForThisAdmin = operators.filter((op) =>
            sameHexPubkey(op.created_by, selectedShengAdmin.admin_pubkey),
          );
          const subTabs: Array<{ key: 'operators' | 'super-admin'; label: string }> = [
            { key: 'operators', label: '市级管理员列表' },
            { key: 'super-admin', label: '省级管理员' },
          ];
          return (
            <Card
              bordered={false}
              style={glassCardStyle}
              headStyle={glassCardHeadStyle}
              title={
                <div style={{ position: 'relative', display: 'flex', alignItems: 'center', minHeight: 32 }}>
                  {isKeyAdmin && (
                    <Button type="link" style={{ paddingLeft: 0 }} onClick={() => setSelectedShengAdmin(null)}>
                      ← 返回省列表
                    </Button>
                  )}
                  <span style={{ position: 'absolute', left: '50%', transform: 'translateX(-50%)' }}>
                    {selectedShengAdmin.province}
                  </span>
                </div>
              }
            >
              <div
                style={{
                  display: 'flex',
                  gap: 8,
                  padding: 6,
                  background: 'rgba(15,23,42,0.06)',
                  borderRadius: 10,
                  border: '1px solid rgba(15,23,42,0.12)',
                  width: 'fit-content',
                  marginBottom: 16,
                }}
              >
                {subTabs.map((t) => (
                  <button
                    key={t.key}
                    onClick={() => setAdminDetailTab(t.key)}
                    style={{
                      padding: '6px 18px',
                      borderRadius: 8,
                      border: 'none',
                      cursor: 'pointer',
                      fontSize: 13,
                      fontWeight: 500,
                      transition: 'all 0.2s ease',
                      ...(adminDetailTab === t.key
                        ? {
                            background: 'linear-gradient(135deg, #0d9488, #0f766e)',
                            color: '#fff',
                            boxShadow: '0 2px 6px rgba(13,148,136,0.35)',
                          }
                        : {
                            background: 'rgba(255,255,255,0.7)',
                            color: 'rgba(15,23,42,0.75)',
                          }),
                    }}
                  >
                    {t.label}
                  </button>
                ))}
              </div>

              {adminDetailTab === 'operators' ? (
                <Card
                  type="inner"
                  title="市级管理员列表"
                  extra={
                    canEditOperators ? (
                      <Button type="primary" onClick={() => setAddOperatorOpen(true)}>
                        新增市级管理员
                      </Button>
                    ) : null
                  }
                >
                  <Table<OperatorRow>
                    rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
                    loading={operatorsLoading}
                    dataSource={operatorsForThisAdmin}
                    pagination={{
                      pageSize: 10,
                      current: operatorListPage,
                      onChange: (page) => setOperatorListPage(page),
                      showSizeChanger: false,
                      showTotal: (total) => `共 ${total} 条`,
                    }}
                    columns={[
                      {
                        title: '序号',
                        width: 70,
                        align: 'center',
                        render: (_v, _row, index) => (operatorListPage - 1) * 10 + index + 1,
                      },
                      { title: '市', dataIndex: 'city', align: 'center', width: 120 },
                      { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
                      {
                        title: '账户',
                        dataIndex: 'admin_pubkey',
                        align: 'center',
                        render: (v: string) => tryEncodeSs58(v),
                      },
                      { title: '状态', dataIndex: 'status', align: 'center', width: 100 },
                      ...(canEditOperators
                        ? [
                            {
                              title: '操作',
                              width: 220,
                              align: 'center' as const,
                              render: (_v: unknown, row: OperatorRow) => (
                                <Space>
                                  <Button size="small" onClick={() => onUpdateOperator(row)}>
                                    修改
                                  </Button>
                                  <Button size="small" onClick={() => onToggleOperatorStatus(row)}>
                                    {row.status === 'ACTIVE' ? '停用' : '启用'}
                                  </Button>
                                  <Button size="small" danger onClick={() => onDeleteOperator(row)}>
                                    删除
                                  </Button>
                                </Space>
                              ),
                            },
                          ]
                        : []),
                    ]}
                  />
                </Card>
              ) : (
                // ── 省级管理员(基本信息 + 更换) ──
                <Card
                  type="inner"
                  title="省级管理员"
                  extra={
                    canReplaceThisAdmin ? (
                      <Form
                        form={replaceSuperForm}
                        layout="inline"
                        onFinish={(values: { admin_pubkey: string }) =>
                          onReplaceShengAdmin({ province: selectedShengAdmin.province, admin_pubkey: values.admin_pubkey })
                        }
                        style={{ rowGap: 8 }}
                      >
                        <Form.Item
                          name="admin_pubkey"
                          rules={[
                            { required: true, message: '请输入新省级管理员账户' },
                            {
                              validator: async (_rule, value) => {
                                if (!value) return;
                                try {
                                  decodeSs58(String(value));
                                } catch (e) {
                                  throw new Error(e instanceof Error ? e.message : '账户格式无效');
                                }
                              },
                            },
                          ]}
                          style={{ marginBottom: 0 }}
                        >
                          <Input
                            style={{ width: 420, maxWidth: '60vw' }}
                            placeholder="新省级管理员账户(SS58)"
                            suffix={
                              <span
                                title="扫码识别用户码"
                                style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                                onClick={() => setAccountScanTarget('super-admin')}
                              >
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                  <path d="M3 7V5a2 2 0 0 1 2-2h2" />
                                  <path d="M17 3h2a2 2 0 0 1 2 2v2" />
                                  <path d="M21 17v2a2 2 0 0 1-2 2h-2" />
                                  <path d="M7 21H5a2 2 0 0 1-2-2v-2" />
                                  <rect x="7" y="7" width="10" height="10" rx="1" />
                                </svg>
                              </span>
                            }
                          />
                        </Form.Item>
                        <Form.Item style={{ marginBottom: 0 }}>
                          <Button type="primary" htmlType="submit" loading={replaceSuperLoading}>
                            更换省级管理员
                          </Button>
                        </Form.Item>
                      </Form>
                    ) : null
                  }
                >
                  <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', rowGap: 8, columnGap: 12 }}>
                    <Typography.Text type="secondary">省份</Typography.Text>
                    <Typography.Text>{selectedShengAdmin.province}</Typography.Text>
                    <Typography.Text type="secondary">名称</Typography.Text>
                    <Typography.Text>{selectedShengAdmin.admin_name}</Typography.Text>
                    <Typography.Text type="secondary">账户</Typography.Text>
                    <Typography.Text code style={{ wordBreak: 'break-all' }}>
                      {tryEncodeSs58(selectedShengAdmin.admin_pubkey)}
                    </Typography.Text>
                  </div>
                </Card>
              )}
            </Card>
          );
        })()
      ) : (
        // ── KeyAdmin:注册局省份列表 ──
        <Card title="省份列表" bordered={false} style={glassCardStyle} headStyle={glassCardHeadStyle}>
          {shengAdminsLoading ? (
            <Typography.Text type="secondary">加载中...</Typography.Text>
          ) : (
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
                gap: 12,
              }}
            >
              {shengAdmins.map((row) => (
                <div
                  key={`${row.province}-${row.admin_pubkey}`}
                  onClick={() => setSelectedShengAdmin(row)}
                  style={{
                    padding: 18,
                    borderRadius: 12,
                    border: '1px solid rgba(15,23,42,0.22)',
                    background: 'rgba(226,232,240,0.55)',
                    boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    textAlign: 'center' as const,
                  }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLDivElement).style.background = 'rgba(13,148,136,0.22)';
                    (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(13,148,136,0.55)';
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLDivElement).style.background = 'rgba(226,232,240,0.55)';
                    (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(15,23,42,0.22)';
                  }}
                >
                  <div style={{ fontSize: 16, fontWeight: 600, color: '#0f172a', textAlign: 'center' }}>
                    {row.province}
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* ── 新增市级管理员 Modal(在机构详情页触发) ── */}
      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增市级管理员</div>}
        open={addOperatorOpen}
        onCancel={() => {
          addOperatorForm.resetFields();
          setAddOperatorOpen(false);
        }}
        footer={[
          <Button
            key="cancel"
            onClick={() => {
              addOperatorForm.resetFields();
              setAddOperatorOpen(false);
            }}
          >
            取消新增
          </Button>,
          <Button
            key="submit"
            type="primary"
            loading={addOperatorLoading}
            onClick={() => addOperatorForm.submit()}
          >
            确认新增
          </Button>,
        ]}
        destroyOnClose
      >
        <Form
          form={addOperatorForm}
          layout="vertical"
          onFinish={(values: { operator_name: string; operator_pubkey: string; operator_city: string }) =>
            onCreateOperator({
              operator_name: values.operator_name,
              operator_pubkey: values.operator_pubkey,
              city: values.operator_city,
              created_by: selectedShengAdmin?.admin_pubkey,
            })
          }
        >
          <Form.Item
            label="姓名"
            name="operator_name"
            rules={[{ required: true, message: '请输入市级管理员姓名' }]}
          >
            <Input placeholder="请输入市级管理员姓名" />
          </Form.Item>
          <Form.Item
            label="市"
            name="operator_city"
            rules={[{ required: true, message: '请选择市' }]}
          >
            <Select
              placeholder="请选择市"
              loading={operatorCitiesLoading}
              options={operatorCities
                .filter((c) => c.code !== '000')
                .map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
            />
          </Form.Item>
          <Form.Item
            label="账户"
            name="operator_pubkey"
            rules={[
              { required: true, message: '请输入市级管理员账户' },
              {
                validator: async (_rule, value) => {
                  if (!value) return;
                  try {
                    decodeSs58(String(value));
                  } catch (err) {
                    throw new Error(err instanceof Error ? err.message : '账户格式无效');
                  }
                },
              },
            ]}
          >
            <Input
              placeholder="请输入市级管理员账户(SS58)"
              suffix={
                <span
                  title="扫码识别用户码"
                  style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                  onClick={() => setAccountScanTarget('operator')}
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M3 7V5a2 2 0 0 1 2-2h2" />
                    <path d="M17 3h2a2 2 0 0 1 2 2v2" />
                    <path d="M21 17v2a2 2 0 0 1-2 2h-2" />
                    <path d="M7 21H5a2 2 0 0 1-2-2v-2" />
                    <rect x="7" y="7" width="10" height="10" rx="1" />
                  </svg>
                </span>
              }
            />
          </Form.Item>
        </Form>
      </Modal>

      {/* ── 通用扫码识别账户弹窗(新增市级管理员 / 更换省级管理员) ── */}
      <ScanAccountModal
        open={accountScanTarget !== null}
        onClose={() => setAccountScanTarget(null)}
        onResolved={(addr) => {
          if (accountScanTarget === 'operator') {
            addOperatorForm.setFieldsValue({ operator_pubkey: addr });
          } else if (accountScanTarget === 'super-admin') {
            replaceSuperForm.setFieldsValue({ admin_pubkey: addr });
          }
          setAccountScanTarget(null);
        }}
      />
    </>
  );
}
