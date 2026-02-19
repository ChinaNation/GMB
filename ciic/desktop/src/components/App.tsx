import { useEffect, useState } from 'react';
import { ExclamationCircleFilled } from '@ant-design/icons';
import {
  Button,
  Card,
  Form,
  Input,
  Layout,
  Modal,
  Space,
  Table,
  Typography,
  message
} from 'antd';
import type { AdminAuth, CitizenRow } from '../api/client';
import { checkAdminAuth, confirmBind, listCitizens, unbind } from '../api/client';

const { Header, Content } = Layout;
const AUTH_STORAGE_KEY = 'ciic_admin_auth_v1';

function readStoredAuth(): AdminAuth | null {
  try {
    const raw = localStorage.getItem(AUTH_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as AdminAuth;
    if (!parsed?.user || !parsed?.password) return null;
    return parsed;
  } catch {
    return null;
  }
}

function writeStoredAuth(auth: AdminAuth) {
  localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(auth));
}

function clearStoredAuth() {
  localStorage.removeItem(AUTH_STORAGE_KEY);
}

export default function App() {
  const [auth, setAuth] = useState<AdminAuth | null>(() => readStoredAuth());
  const [rows, setRows] = useState<CitizenRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [binding, setBinding] = useState(false);
  const [bindModalOpen, setBindModalOpen] = useState(false);
  const [bindTargetPubkey, setBindTargetPubkey] = useState('');
  const [bootstrapping, setBootstrapping] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const bootstrap = async () => {
      if (!auth) {
        setBootstrapping(false);
        return;
      }
      try {
        await checkAdminAuth(auth);
        const list = await listCitizens(auth);
        if (!cancelled) {
          setRows(list);
        }
      } catch {
        if (!cancelled) {
          clearStoredAuth();
          setAuth(null);
          setRows([]);
          message.warning('登录状态已失效，请重新登录');
        }
      } finally {
        if (!cancelled) {
          setBootstrapping(false);
        }
      }
    };
    bootstrap();
    return () => {
      cancelled = true;
    };
  }, []);

  const onLogin = async (values: AdminAuth) => {
    if (!values.user || !values.password) {
      message.error('请输入管理员账号和密码');
      return;
    }
    try {
      await checkAdminAuth(values);
      setAuth(values);
      writeStoredAuth(values);
      message.success('登录成功');
      await refreshList(values, undefined, true);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '登录失败';
      message.error(msg);
      setAuth(null);
    }
  };

  const onLogout = () => {
    setAuth(null);
    clearStoredAuth();
    setRows([]);
    setBindModalOpen(false);
    setBindTargetPubkey('');
    message.success('已退出登录');
  };

  const refreshList = async (currentAuth: AdminAuth, keyword?: string, silent?: boolean) => {
    setLoading(true);
    try {
      const list = await listCitizens(currentAuth, keyword);
      setRows(list);
      if (keyword && list.length === 0) {
        Modal.warning({
          title: '查询结果',
          content: '没有的公民信息'
        });
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '查询失败';
      if (!silent) {
        message.error(msg);
      } else if (msg.includes('(404)')) {
        message.warning('后端接口版本较旧，请重启后端到最新代码');
      }
    } finally {
      setLoading(false);
    }
  };

  const onSearch = async (values: { keyword: string }) => {
    if (!auth) return;
    await refreshList(auth, values.keyword?.trim());
  };

  const openBindModal = (pubkey: string) => {
    setBindTargetPubkey(pubkey);
    setBindModalOpen(true);
  };

  const onConfirmBind = async (values: { archive_index: string }) => {
    if (!auth) return;
    if (!bindTargetPubkey) return;
    setBinding(true);
    try {
      const res = await confirmBind(auth, {
        account_pubkey: bindTargetPubkey,
        archive_index: values.archive_index.trim()
      });
      message.success(`绑定成功，CIIC码：${res.ciic_code}`);
      setBindModalOpen(false);
      setBindTargetPubkey('');
      await refreshList(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '绑定失败';
      message.error(msg);
    } finally {
      setBinding(false);
    }
  };

  const onUnbind = async (pubkey: string) => {
    if (!auth) return;
    Modal.confirm({
      centered: true,
      icon: null,
      title: null,
      content: (
        <div style={{ textAlign: 'center', paddingTop: 8 }}>
          <ExclamationCircleFilled style={{ color: '#faad14', fontSize: 28, marginBottom: 8 }} />
          <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>确认解绑</div>
          <div style={{ color: '#4b5563', lineHeight: 1.6 }}>
            确定要解绑并删除该公民信息吗？
            <br />
            公钥：{pubkey}
          </div>
        </div>
      ),
      okText: '确认解绑',
      okButtonProps: { danger: true },
      cancelText: '取 消',
      footer: (_, { OkBtn, CancelBtn }) => (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 12 }}>
          <CancelBtn />
          <OkBtn />
        </div>
      ),
      onOk: async () => {
        setLoading(true);
        try {
          await unbind(auth, pubkey);
          message.success('解绑成功');
          await refreshList(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '解绑失败';
          message.error(msg);
        } finally {
          setLoading(false);
        }
      }
    });
  };

  return (
    <Layout style={{ minHeight: '100vh', background: '#f0fdfa' }}>
      <Header
        style={{
          background: '#115e59',
          color: '#fff',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}
      >
        <Typography.Title level={4} style={{ color: '#fff', margin: 0, lineHeight: '64px' }}>
          公民身份识别码管理系统
        </Typography.Title>
        {auth && (
          <Button danger onClick={onLogout}>
            退出登录
          </Button>
        )}
      </Header>

      {bootstrapping ? (
        <Content style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 }}>
          <Card bordered={false} style={{ width: 420, maxWidth: '92vw' }} loading />
        </Content>
      ) : !auth ? (
        <Content style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 }}>
          <Card title="管理员登录" bordered={false} style={{ width: 420, maxWidth: '92vw' }}>
            <Form layout="vertical" onFinish={onLogin}>
              <Form.Item label="管理员账号" name="user" rules={[{ required: true }]}>
                <Input placeholder="admin" />
              </Form.Item>
              <Form.Item label="管理员密码" name="password" rules={[{ required: true }]}>
                <Input.Password placeholder="admin123" />
              </Form.Item>
              <Button htmlType="submit" type="primary">
                登录
              </Button>
            </Form>
          </Card>
        </Content>
      ) : (
        <Content style={{ padding: 24 }}>
          <Card bordered={false} style={{ marginBottom: 16 }}>
            <Form layout="inline" onFinish={onSearch}>
              <Form.Item name="keyword" style={{ width: '100%', maxWidth: 700 }}>
                <Input placeholder="请输入公钥、档案索引号、CIIC码" allowClear />
              </Form.Item>
              <Form.Item>
                <Button htmlType="submit" type="primary" loading={loading}>
                  查询
                </Button>
              </Form.Item>
            </Form>
          </Card>

          <Card title="公民身份信息" bordered={false}>
            <Table<CitizenRow>
              rowKey={(r) => `${r.seq}-${r.account_pubkey}`}
              dataSource={rows}
              loading={loading}
              pagination={{ pageSize: 10 }}
              columns={[
                {
                  title: '序号',
                  width: 80,
                  align: 'center',
                  render: (_v, _r, idx) => idx + 1
                },
                {
                  title: '公钥',
                  dataIndex: 'account_pubkey',
                  align: 'center'
                },
                {
                  title: '档案索引号',
                  dataIndex: 'archive_index',
                  align: 'center',
                  render: (v) => v ?? '-'
                },
                {
                  title: 'CIIC码',
                  dataIndex: 'ciic_code',
                  align: 'center',
                  render: (v) => v ?? '-'
                },
                {
                  title: '解绑或绑定',
                  width: 150,
                  align: 'center',
                  render: (_v, row) =>
                    row.is_bound ? (
                      <Button danger onClick={() => onUnbind(row.account_pubkey)}>
                        解绑
                      </Button>
                    ) : (
                      <Button type="primary" onClick={() => openBindModal(row.account_pubkey)}>
                        绑定
                      </Button>
                    )
                }
              ]}
            />
          </Card>
        </Content>
      )}

      <Modal
        title="绑定公民身份"
        open={bindModalOpen}
        footer={null}
        onCancel={() => setBindModalOpen(false)}
        destroyOnClose
      >
        <Form layout="vertical" onFinish={onConfirmBind}>
          <Form.Item label="公钥">
            <Input value={bindTargetPubkey} disabled />
          </Form.Item>
          <Form.Item
            label="档案索引号"
            name="archive_index"
            rules={[{ required: true, message: '请输入档案索引号' }]}
          >
            <Input placeholder="请输入档案索引号" />
          </Form.Item>
          <Space>
            <Button onClick={() => setBindModalOpen(false)}>取消</Button>
            <Button htmlType="submit" type="primary" loading={binding}>
              确认绑定
            </Button>
          </Space>
        </Form>
      </Modal>
    </Layout>
  );
}
