// 省级管理员 sub-tab:基本信息 + 签名密钥 + 更换表单(从 ProvinceDetailView.tsx 拆分)

import { Button, Card, Form, Input, Tag, Typography } from 'antd';
import { decodeSs58, tryEncodeSs58 } from '../../utils/ss58';
import type { ShengAdminSharedState } from './shengAdminUtils';

interface SuperAdminSubTabProps {
  selectedShengAdmin: NonNullable<ShengAdminSharedState['selectedShengAdmin']>;
  canReplaceThisAdmin: boolean;
  replaceSuperLoading: boolean;
  replaceSuperForm: ShengAdminSharedState['replaceSuperForm'];
  onReplaceShengAdmin: ShengAdminSharedState['onReplaceShengAdmin'];
  setAccountScanTarget: ShengAdminSharedState['setAccountScanTarget'];
}

export function SuperAdminSubTab({
  selectedShengAdmin,
  canReplaceThisAdmin,
  replaceSuperLoading,
  replaceSuperForm,
  onReplaceShengAdmin,
  setAccountScanTarget,
}: SuperAdminSubTabProps) {
  return (
    <>
      {/* ── 省级管理员(基本信息 + 更换) ── */}
      <Card
        type="inner"
        title="省级管理员"
        extra={
          canReplaceThisAdmin ? (
            <Form
              form={replaceSuperForm}
              layout="inline"
              onFinish={(values: { admin_name: string; admin_pubkey: string }) =>
                onReplaceShengAdmin({ province: selectedShengAdmin.province, admin_name: values.admin_name, admin_pubkey: values.admin_pubkey })
              }
              style={{ rowGap: 8 }}
            >
              <Form.Item
                name="admin_name"
                rules={[{ required: true, message: '请输入姓名' }]}
                style={{ marginBottom: 0 }}
              >
                <Input style={{ width: 140 }} placeholder="新管理员姓名" />
              </Form.Item>
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
        {/* 省级管理员基本信息：姓名 + 账户 */}
        <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', rowGap: 8, columnGap: 12 }}>
          <Typography.Text type="secondary">姓名</Typography.Text>
          <Typography.Text>{selectedShengAdmin.admin_name}</Typography.Text>
          <Typography.Text type="secondary">账户</Typography.Text>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>
            {tryEncodeSs58(selectedShengAdmin.admin_pubkey)}
          </Typography.Text>
        </div>
      </Card>

      {/* ── 签名密钥信息 ── */}
      <Card type="inner" title="签名密钥" style={{ marginTop: 16 }}>
        <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', rowGap: 8, columnGap: 12 }}>
          <Typography.Text type="secondary">状态</Typography.Text>
          <div>
            {selectedShengAdmin.signing_pubkey ? (
              <Tag color="green">已激活</Tag>
            ) : (
              <Tag color="blue">未初始化</Tag>
            )}
          </div>
          <Typography.Text type="secondary">生成时间</Typography.Text>
          <Typography.Text>
            {selectedShengAdmin.signing_created_at
              ? new Date(selectedShengAdmin.signing_created_at).toLocaleString('zh-CN')
              : '-'}
          </Typography.Text>
          <Typography.Text type="secondary">账户</Typography.Text>
          <Typography.Text code style={{ wordBreak: 'break-all' }}>
            {selectedShengAdmin.signing_pubkey ? tryEncodeSs58(selectedShengAdmin.signing_pubkey) : '-'}
          </Typography.Text>
        </div>
      </Card>
    </>
  );
}
