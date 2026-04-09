// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 3)
// 密钥管理顶层视图 —— 对应 App.tsx 中 activeView === 'keyring' 分支。
//
// 本步骤迁移内容:
//   - state: keyringState / keyringLoading / keyringActionLoading / keyringChallenge /
//     keyringSignedPayload / keyringScannerActive / keyringScannerReady /
//     keyringScanSubmitting / keyringCommitLoading / keyringScanAccountOpen /
//     mainAccountBalance / mainAccountBalanceError
//   - ref: keyringVideoRef / keyringScanCleanupRef(整体迁出 App.tsx)
//   - handler: refreshKeyringState / stopKeyringScanner /
//     onCreateKeyringRotateChallenge / onCompleteKeyringRotate / onToggleKeyringScanner
//   - useEffect: 挂载刷新状态 + 摄像头扫码启动
//
// 摄像头扫码使用 src/utils/cameraScanner.ts 共享工具,
// 组件 unmount 时通过 useEffect cleanup 强制释放 MediaStream。

import { useEffect, useRef, useState } from 'react';
import { QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Form, Input, QRCode, Typography, message } from 'antd';
import { useAuth } from '../../hooks/useAuth';
import type {
  AdminAuth,
  KeyringRotateChallengeResult,
  KeyringStateResult,
} from '../../api/client';
import {
  commitKeyringRotate,
  createKeyringRotateChallenge,
  getAttestorKeyring,
  getChainBalance,
  verifyKeyringRotateSignature,
} from '../../api/client';
import { decodeSs58, tryEncodeSs58 } from '../../utils/ss58';
import { startCameraScanner } from '../../utils/cameraScanner';
import { parseKeyringSignedPayload, type KeyringSignedPayload } from '../../utils/parseSignedPayload';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';
import { ScanAccountModal } from '../../components/ScanAccountModal';

// 中文注释:步 4 — parseKeyringSignedPayload 已抽到 utils/parseSignedPayload.ts,统一复用。

export function KeyringView() {
  const { auth, capabilities } = useAuth();
  const [keyringState, setKeyringState] = useState<KeyringStateResult | null>(null);
  const [keyringLoading, setKeyringLoading] = useState(false);
  const [keyringActionLoading, setKeyringActionLoading] = useState(false);
  const [keyringChallenge, setKeyringChallenge] = useState<KeyringRotateChallengeResult | null>(null);
  // 中文注释:keyringSignedPayload 仅作过程态保存,UI 没有直接读它,但保留语义不变。
  const [, setKeyringSignedPayload] = useState<KeyringSignedPayload | null>(null);
  const [keyringScannerActive, setKeyringScannerActive] = useState(false);
  const [keyringScannerReady, setKeyringScannerReady] = useState(false);
  const [keyringScanSubmitting, setKeyringScanSubmitting] = useState(false);
  const [, setKeyringCommitLoading] = useState(false);
  const [mainAccountBalance, setMainAccountBalance] = useState<string | null>(null);
  const [mainAccountBalanceError, setMainAccountBalanceError] = useState<string | null>(null);
  const [keyringScanAccountOpen, setKeyringScanAccountOpen] = useState(false);
  const [keyringForm] = Form.useForm<{ new_backup_pubkey: string }>();
  const keyringVideoRef = useRef<HTMLVideoElement | null>(null);
  const keyringScanCleanupRef = useRef<(() => void) | null>(null);

  const stopKeyringScanner = () => {
    if (keyringScanCleanupRef.current) {
      keyringScanCleanupRef.current();
      keyringScanCleanupRef.current = null;
    }
    setKeyringScannerReady(false);
  };

  const refreshKeyringState = async (currentAuth: AdminAuth) => {
    setKeyringLoading(true);
    try {
      const state = await getAttestorKeyring(currentAuth);
      setKeyringState(state);
      // 拉到主账户后立即查链上余额(每次进入密钥管理页都查一次,不缓存)
      if (state?.main_pubkey) {
        setMainAccountBalance(null);
        setMainAccountBalanceError(null);
        try {
          const bal = await getChainBalance(currentAuth, state.main_pubkey);
          setMainAccountBalance(bal.balance_text);
        } catch (err) {
          setMainAccountBalanceError(err instanceof Error ? err.message : String(err));
        }
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载密钥状态失败';
      message.error(msg);
    } finally {
      setKeyringLoading(false);
    }
  };

  const onCreateKeyringRotateChallenge = async (values: { new_backup_pubkey: string }) => {
    if (!auth) return;
    void values;
    // 主公钥不能发起轮换
    if (
      keyringState &&
      auth.admin_pubkey.replace(/^0x/i, '').toLowerCase() ===
        keyringState.main_pubkey.replace(/^0x/i, '').toLowerCase()
    ) {
      message.error('主密钥不能发起轮换,请用备用密钥登录');
      return;
    }
    setKeyringActionLoading(true);
    try {
      const challenge = await createKeyringRotateChallenge(auth, {
        initiator_pubkey: auth.admin_pubkey,
      });
      setKeyringChallenge(challenge);
      setKeyringSignedPayload(null);
      setKeyringScannerActive(false);
      stopKeyringScanner();
      message.success('轮换签名二维码已生成,请用备用私钥钱包扫码签名');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成轮换挑战失败';
      message.error(msg);
      setKeyringChallenge(null);
    } finally {
      setKeyringActionLoading(false);
    }
  };

  const onCompleteKeyringRotate = async (raw: string) => {
    if (!auth || !keyringChallenge) {
      message.error('请先生成轮换二维码');
      return;
    }
    const newBackupAddress = keyringForm.getFieldValue('new_backup_pubkey')?.trim();
    if (!newBackupAddress) {
      message.error('新备用账户不能为空');
      return;
    }
    let newBackupPubkey: string;
    try {
      newBackupPubkey = decodeSs58(newBackupAddress);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setKeyringScanSubmitting(true);
    try {
      const payload = parseKeyringSignedPayload(raw, keyringChallenge.challenge_id);
      await verifyKeyringRotateSignature(auth, {
        challenge_id: payload.challenge_id,
        signature: payload.signature,
      });
      setKeyringSignedPayload(payload);
      setKeyringScannerActive(false);
      stopKeyringScanner();
      message.success('签名校验通过,正在提交轮换...');
      setKeyringCommitLoading(true);
      try {
        const result = await commitKeyringRotate(auth, {
          challenge_id: payload.challenge_id,
          signature: payload.signature,
          new_backup_pubkey: newBackupPubkey,
        });
        if (result.chain_submit_ok) {
          message.success(`主密钥轮换成功,新版本:${result.version}`);
        } else {
          message.warning(
            `主密钥已本地轮换为版本 ${result.version},但上链提交失败:${result.chain_submit_error || '未知错误'}`,
          );
        }
        setKeyringChallenge(null);
        setKeyringSignedPayload(null);
        keyringForm.resetFields();
        await refreshKeyringState(auth);
      } catch (commitErr) {
        const commitMsg = commitErr instanceof Error ? commitErr.message : '提交轮换失败';
        message.error(commitMsg);
      } finally {
        setKeyringCommitLoading(false);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '提交轮换签名失败';
      message.error(msg);
    } finally {
      setKeyringScanSubmitting(false);
    }
  };

  const onToggleKeyringScanner = () => {
    if (!keyringChallenge) {
      message.warning('请先生成轮换二维码');
      return;
    }
    setKeyringScannerActive((v) => !v);
  };

  // 挂载时拉一次密钥状态;auth 切换也重新拉
  useEffect(() => {
    if (!auth) return;
    void refreshKeyringState(auth);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token]);

  // 摄像头扫码 effect:迁自 App.tsx,unmount 时强制释放
  useEffect(() => {
    if (!keyringScannerActive || !keyringChallenge || !keyringVideoRef.current) {
      stopKeyringScanner();
      return;
    }
    keyringScanCleanupRef.current = startCameraScanner(
      keyringVideoRef.current,
      (raw) => {
        setKeyringScannerActive(false);
        stopKeyringScanner();
        void onCompleteKeyringRotate(raw);
      },
      () => setKeyringScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopKeyringScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [keyringScannerActive, keyringChallenge]);

  // 组件 unmount 时兜底清理摄像头(防止 view 切走时摄像头还在跑)
  useEffect(() => {
    return () => {
      if (keyringScanCleanupRef.current) {
        keyringScanCleanupRef.current();
        keyringScanCleanupRef.current = null;
      }
    };
  }, []);

  if (!capabilities.canManageKeyring) {
    return null;
  }

  // 主密钥登录时,一切轮换相关控件(输入框/按钮/扫码图标)都禁用
  const isMainKeySigned = Boolean(
    keyringState &&
      auth &&
      auth.admin_pubkey.replace(/^0x/i, '').toLowerCase() ===
        keyringState.main_pubkey.replace(/^0x/i, '').toLowerCase(),
  );

  return (
    <>
      <Card
        title="签名密钥管理(一主两备)"
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
        extra={
          <Button
            onClick={() => {
              if (auth) {
                void refreshKeyringState(auth);
              }
            }}
            loading={keyringLoading}
          >
            刷新状态
          </Button>
        }
      >
        <Form
          form={keyringForm}
          layout="inline"
          onFinish={onCreateKeyringRotateChallenge}
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
                ref={keyringVideoRef}
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
              <Button onClick={onToggleKeyringScanner} disabled={keyringScanSubmitting} style={{ borderRadius: 10 }}>
                {keyringScannerActive ? '停止扫码' : '开启扫码'}
              </Button>
            </div>
          </div>
        </div>

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
