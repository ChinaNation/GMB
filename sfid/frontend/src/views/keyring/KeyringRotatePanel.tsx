// 中文注释:密钥轮换面板 —— 包含轮换表单、二维码展示、扫码窗口。
// 从 KeyringView.tsx 拆出(任务卡 20260408-sfid-frontend-app-tsx-split 步 3 模块化)。

import { QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Form, Input, QRCode, Typography } from 'antd';
import { decodeSs58, tryEncodeSs58 } from '../../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';
import { ScanAccountModal } from '../../components/ScanAccountModal';
import type { KeyringSharedState } from './keyringUtils';

export function KeyringRotatePanel({ state }: { state: KeyringSharedState }) {
  const {
    keyringState,
    keyringLoading,
    keyringActionLoading,
    keyringChallenge,
    keyringScannerActive,
    keyringScannerReady,
    keyringScanSubmitting,
    keyringScanAccountOpen,
    setKeyringScanAccountOpen,
    mainAccountBalance,
    mainAccountBalanceError,
    isMainKeySigned,
    keyringForm,
    keyringVideoRef,
    onRefresh,
    onCreateRotateChallenge,
    onToggleScanner,
  } = state;

  return (
    <>
      <Card
        title="签名密钥管理(一主两备)"
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
        extra={
          <Button onClick={onRefresh} loading={keyringLoading}>
            刷新状态
          </Button>
        }
      >
        <Form
          form={keyringForm}
          layout="inline"
          onFinish={onCreateRotateChallenge}
          style={{ marginBottom: 12, rowGap: 8 }}
        >
          <Form.Item
            name="new_backup_pubkey"
            rules={[
              { required: true, message: '请输入新备用账户' },
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
              style={{ width: 420, maxWidth: '72vw' }}
              placeholder="新备用账户(SS58 地址)"
              disabled={isMainKeySigned}
              suffix={
                <span
                  title={isMainKeySigned ? '主密钥登录无法轮换' : '扫码识别用户码'}
                  style={{
                    cursor: isMainKeySigned ? 'not-allowed' : 'pointer',
                    display: 'inline-flex',
                    color: isMainKeySigned ? 'rgba(148,163,184,0.6)' : '#0d9488',
                  }}
                  onClick={() => {
                    if (isMainKeySigned) return;
                    setKeyringScanAccountOpen(true);
                  }}
                >
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
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
            <Button type="primary" htmlType="submit" loading={keyringActionLoading} disabled={isMainKeySigned}>
              发起轮换
            </Button>
          </Form.Item>
        </Form>

        <Typography.Paragraph type="secondary" style={{ marginBottom: 12 }}>
          {'流程:输入新备用账户 -> 生成轮换二维码 -> 备用钱包扫码签名 -> 本页面扫码验签 -> 自动完成轮换并推链。'}
        </Typography.Paragraph>

        <div style={{ display: 'flex', gap: 16, alignItems: 'flex-start', flexWrap: 'wrap', marginBottom: 12 }}>
          {/* ── 轮换二维码 ── */}
          <div style={{ flex: '1 1 320px', minWidth: 300 }}>
            <Typography.Text strong style={{ fontSize: 14, color: '#374151', display: 'block', marginBottom: 8 }}>
              轮换二维码
            </Typography.Text>
            <div style={{ display: 'flex', justifyContent: 'center' }}>
              <div
                style={{
                  filter: keyringChallenge ? 'none' : 'blur(3px) opacity(0.4)',
                  transition: 'filter 0.3s ease',
                }}
              >
                <QRCode
                  value={keyringChallenge?.challenge_text || 'SFID_KEYRING_ROTATE_PENDING'}
                  size={260}
                  color="#134e4a"
                  bordered={false}
                />
              </div>
            </div>
            <Typography.Paragraph type="secondary" style={{ marginTop: 10, marginBottom: 8 }}>
              {keyringChallenge
                ? `轮换挑战有效期至:${new Date(keyringChallenge.expire_at * 1000).toLocaleTimeString()}`
                : ''}
            </Typography.Paragraph>
          </div>

          {/* ── 扫码窗口 ── */}
          <div style={{ flex: '1 1 260px', minWidth: 260 }}>
            <Typography.Text strong style={{ fontSize: 14, color: '#374151', display: 'block', marginBottom: 8 }}>
              扫码窗口
            </Typography.Text>
            <div
              style={{
                width: 260,
                height: 260,
                boxSizing: 'border-box',
                background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                borderRadius: 16,
                overflow: 'hidden',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                position: 'relative',
                border: '2px solid #334155',
                boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
              }}
            >
              {/* 四角装饰线 */}
              <div
                style={{
                  position: 'absolute',
                  top: 8,
                  left: 8,
                  width: 16,
                  height: 16,
                  borderTop: '2px solid #0d9488',
                  borderLeft: '2px solid #0d9488',
                  borderTopLeftRadius: 4,
                  zIndex: 2,
                }}
              />
              <div
                style={{
                  position: 'absolute',
                  top: 8,
                  right: 8,
                  width: 16,
                  height: 16,
                  borderTop: '2px solid #0d9488',
                  borderRight: '2px solid #0d9488',
                  borderTopRightRadius: 4,
                  zIndex: 2,
                }}
              />
              <div
                style={{
                  position: 'absolute',
                  bottom: 8,
                  left: 8,
                  width: 16,
                  height: 16,
                  borderBottom: '2px solid #0d9488',
                  borderLeft: '2px solid #0d9488',
                  borderBottomLeftRadius: 4,
                  zIndex: 2,
                }}
              />
              <div
                style={{
                  position: 'absolute',
                  bottom: 8,
                  right: 8,
                  width: 16,
                  height: 16,
                  borderBottom: '2px solid #0d9488',
                  borderRight: '2px solid #0d9488',
                  borderBottomRightRadius: 4,
                  zIndex: 2,
                }}
              />
              <video
                ref={keyringVideoRef as React.RefObject<HTMLVideoElement>}
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                muted
                playsInline
              />
              {!keyringScannerReady && (
                <div
                  style={{
                    position: 'absolute',
                    inset: 0,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: 8,
                  }}
                >
                  <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                  <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                    {keyringScannerActive ? '摄像头初始化中...' : '等待开启摄像头'}
                  </Typography.Text>
                </div>
              )}
            </div>
            <div style={{ marginTop: 12 }}>
              <Button onClick={onToggleScanner} disabled={keyringScanSubmitting} style={{ borderRadius: 10 }}>
                {keyringScannerActive ? '停止扫码' : '开启扫码'}
              </Button>
            </div>
          </div>
        </div>

        {/* ── 当前密钥状态 ── */}
        <Card
          size="small"
          loading={keyringLoading}
          style={{
            background: '#f0fdfa',
            borderRadius: 12,
            borderLeft: '3px solid #0d9488',
            border: '1px solid #ccfbf1',
          }}
        >
          <Typography.Text strong style={{ fontSize: 13, color: '#134e4a', display: 'block', marginBottom: 10 }}>
            当前密钥状态
          </Typography.Text>
          <Typography.Paragraph style={{ marginBottom: 6 }}>
            主账户:<Typography.Text code>{tryEncodeSs58(keyringState?.main_pubkey)}</Typography.Text>
            {mainAccountBalance != null && (
              <span style={{ marginLeft: 12, color: '#0d9488', fontWeight: 600 }}>
                余额:{mainAccountBalance} 元
              </span>
            )}
            {mainAccountBalanceError && (
              <span style={{ marginLeft: 12, color: '#ef4444', fontSize: 12 }}>
                余额查询失败:{mainAccountBalanceError}
              </span>
            )}
          </Typography.Paragraph>
          <Typography.Paragraph style={{ marginBottom: 6 }}>
            备用A 账户:<Typography.Text code>{tryEncodeSs58(keyringState?.backup_a_pubkey)}</Typography.Text>
          </Typography.Paragraph>
          <Typography.Paragraph style={{ marginBottom: 0 }}>
            备用B 账户:<Typography.Text code>{tryEncodeSs58(keyringState?.backup_b_pubkey)}</Typography.Text>
          </Typography.Paragraph>
        </Card>
      </Card>

      {/* ── 密钥管理:扫码识别"新备用账户"弹窗 ── */}
      <ScanAccountModal
        open={keyringScanAccountOpen}
        onClose={() => setKeyringScanAccountOpen(false)}
        onResolved={(addr) => {
          keyringForm.setFieldsValue({ new_backup_pubkey: addr });
          setKeyringScanAccountOpen(false);
        }}
      />
    </>
  );
}
