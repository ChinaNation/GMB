// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 2)
// 市级管理员顶层列表视图 —— 对应 activeView === 'operators' 分支。
// 注意:当前路由表中并不会真正走到 'operators' 这个分支(注册局 tab 走的是 system-settings),
// 但该 JSX 来源于 App.tsx 原文,本步骤只做文件迁移,不删除业务逻辑。

import { useEffect, useState } from 'react';
import { Button, Card, Form, Input, Modal, Select, Space, Table, message } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { useAuth } from '../hooks/useAuth';
import type { OperatorRow, SfidCityItem } from '../api/client';
import {
  createOperator,
  deleteOperator,
  listOperators,
  updateOperator,
  updateOperatorStatus,
} from '../api/client';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../App';
import { ScanAccountModal } from '../common/ScanAccountModal';

function isSr25519HexPubkey(value: string): boolean {
  const normalized = value.trim().replace(/^0x/i, '');
  return /^[0-9a-fA-F]{64}$/.test(normalized);
}

export function OperatorsView() {
  const { auth, capabilities } = useAuth();
  const [operators, setOperators] = useState<OperatorRow[]>([]);
  const [operatorsLoading, setOperatorsLoading] = useState(false);
  const [operatorPage, setOperatorPage] = useState(1);
  const [addOperatorOpen, setAddOperatorOpen] = useState(false);
  const [addOperatorLoading, setAddOperatorLoading] = useState(false);
  const [operatorCities] = useState<SfidCityItem[]>([]);
  const [accountScanOpen, setAccountScanOpen] = useState(false);
  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string; operator_city: string }>();

  const refreshOperators = async () => {
    if (!auth) return;
    setOperatorsLoading(true);
    try {
      const rows = await listOperators(auth);
      setOperators(Array.isArray(rows) ? rows : []);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载市级管理员失败';
      message.error(msg);
    } finally {
      setOperatorsLoading(false);
    }
  };

  useEffect(() => {
    void refreshOperators();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token]);

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
      setOperatorPage(1);
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

  const columns: ColumnsType<OperatorRow> = [
    {
      title: '序号',
      width: 80,
      align: 'center',
      render: (_v, _row, index) => (operatorPage - 1) * 10 + index + 1,
    },
    { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
    { title: '公钥', dataIndex: 'admin_pubkey', align: 'center' },
    { title: '状态', dataIndex: 'status', width: 120, align: 'center' },
    {
      title: '创建者',
      align: 'center',
      render: (_v, row) => row.created_by_name || row.created_by || '-',
    },
    ...(capabilities.canCrudShiAdmins
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
  ];

  return (
    <>
      <Card
        title="市级管理员列表"
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
        extra={
          capabilities.canCrudShiAdmins ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  flexWrap: 'nowrap',
                  gap: 8,
                  width: addOperatorOpen ? 860 : 0,
                  opacity: addOperatorOpen ? 1 : 0,
                  overflow: 'hidden',
                  transform: `translateX(${addOperatorOpen ? 0 : 12}px)`,
                  transition: 'all 0.2s ease',
                }}
              >
                <Button
                  type="link"
                  onClick={() => {
                    addOperatorForm.resetFields();
                    setAddOperatorOpen(false);
                  }}
                >
                  取消新增
                </Button>
                <Form
                  form={addOperatorForm}
                  layout="inline"
                  onFinish={(values) => onCreateOperator(values)}
                  style={{ display: 'flex', flexWrap: 'nowrap', alignItems: 'center', gap: 8 }}
                >
                  <Form.Item
                    name="operator_name"
                    rules={[{ required: true, message: '请输入市级管理员姓名' }]}
                    style={{ marginBottom: 0 }}
                  >
                    <Input style={{ width: 180 }} placeholder="请输入市级管理员姓名" />
                  </Form.Item>
                  <Form.Item
                    name="operator_pubkey"
                    rules={[
                      { required: true, message: '请输入市级管理员公钥' },
                      {
                        validator: async (_rule, value) => {
                          if (!value || isSr25519HexPubkey(String(value))) return;
                          throw new Error('公钥格式必须为 32 字节十六进制');
                        },
                      },
                    ]}
                    style={{ marginBottom: 0 }}
                  >
                    <Input style={{ width: 520 }} placeholder="请输入市级管理员公钥" />
                  </Form.Item>
                </Form>
              </div>
              <Button
                type="primary"
                loading={addOperatorLoading}
                onClick={() => {
                  if (!addOperatorOpen) {
                    setAddOperatorOpen(true);
                    return;
                  }
                  addOperatorForm.submit();
                }}
              >
                {addOperatorOpen ? '确认新增' : '新增市级管理员'}
              </Button>
            </div>
          ) : null
        }
      >
        <Table<OperatorRow>
          rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
          loading={operatorsLoading}
          dataSource={operators}
          pagination={{
            pageSize: 10,
            current: operatorPage,
            onChange: (page) => setOperatorPage(page),
          }}
          columns={columns}
        />
      </Card>

      <ScanAccountModal
        open={accountScanOpen}
        onClose={() => setAccountScanOpen(false)}
        onResolved={(addr) => {
          addOperatorForm.setFieldsValue({ operator_pubkey: addr });
          setAccountScanOpen(false);
        }}
      />
    </>
  );
}
