// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 4)
// 解绑弹窗 —— 确认 → 生成签名挑战 → 扫描签名结果。
// 内部持有所有 unbind 相关 state 和 unbindVideoRef。

import { useEffect, useRef, useState } from 'react';
import { Button, Modal, QRCode, Typography, message } from 'antd';
import { QrcodeOutlined } from '@ant-design/icons';
import type { AdminAuth, CitizenBindChallengeResult, CitizenRow } from '../../api/client';
import { citizenBindChallenge, citizenUnbind } from '../../api/client';
import { tryEncodeSs58 } from '../../utils/ss58';
import { startCameraScanner } from '../../utils/cameraScanner';
import { parseKeyringSignedPayload } from '../../utils/parseSignedPayload';

type UnbindStep = 'confirm' | 'sign_challenge' | 'scan_signature';

interface UnbindModalProps {
  auth: AdminAuth | null;
  open: boolean;
  target: CitizenRow | null;
  onClose: () => void;
  onUnbound: () => Promise<void> | void;
}

export function UnbindModal({ auth, open, target, onClose, onUnbound }: UnbindModalProps) {
  const [unbindChallenge, setUnbindChallenge] = useState<CitizenBindChallengeResult | null>(null);
  const [unbindChallengeLoading, setUnbindChallengeLoading] = useState(false);
  const [unbindScannerActive, setUnbindScannerActive] = useState(false);
  const [unbindScannerReady, setUnbindScannerReady] = useState(false);
  const [unbindSubmitting, setUnbindSubmitting] = useState(false);
  const [unbindStep, setUnbindStep] = useState<UnbindStep>('confirm');
  const unbindVideoRef = useRef<HTMLVideoElement | null>(null);
  const unbindScanCleanupRef = useRef<(() => void) | null>(null);

  // 当弹窗打开或目标切换时重置(等价于原 openUnbindModal)
  useEffect(() => {
    if (!open) return;
    setUnbindChallenge(null);
    setUnbindStep('confirm');
    setUnbindScannerActive(false);
    stopUnbindScanner();
  }, [open, target]);

  const stopUnbindScanner = () => {
    if (unbindScanCleanupRef.current) {
      unbindScanCleanupRef.current();
      unbindScanCleanupRef.current = null;
    }
    setUnbindScannerReady(false);
  };

  const onUnbindGenerateChallenge = async () => {
    if (!auth) return;
    setUnbindChallengeLoading(true);
    try {
      const challenge = await citizenBindChallenge(auth);
      setUnbindChallenge(challenge);
      setUnbindStep('sign_challenge');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成解绑签名挑战失败';
      message.error(msg);
    } finally {
      setUnbindChallengeLoading(false);
    }
  };

  const onScanUnbindSignature = async (raw: string) => {
    if (!auth || !unbindChallenge || !target) return;
    const trimmed = raw.trim();
    if (!trimmed) {
      message.error('签名二维码识别失败');
      return;
    }
    setUnbindSubmitting(true);
    try {
      const payload = parseKeyringSignedPayload(trimmed, unbindChallenge.challenge_id);
      setUnbindScannerActive(false);
      stopUnbindScanner();
      await citizenUnbind(auth, {
        citizen_id: target.id,
        challenge_id: payload.challenge_id,
        signature: payload.signature,
      });
      message.success('解绑成功');
      onClose();
      await onUnbound();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '解绑失败';
      message.error(msg);
    } finally {
      setUnbindSubmitting(false);
    }
  };

  useEffect(() => {
    if (!open || !unbindScannerActive || !unbindVideoRef.current) {
      stopUnbindScanner();
      return;
    }
    unbindScanCleanupRef.current = startCameraScanner(
      unbindVideoRef.current,
      (raw) => { setUnbindScannerActive(false); stopUnbindScanner(); void onScanUnbindSignature(raw); },
      () => setUnbindScannerReady(true),
      (msg) => { message.error(msg); setUnbindScannerActive(false); },
    );
    return () => stopUnbindScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, unbindScannerActive]);

  return (
    <Modal
      title={<span style={{ fontSize: 20, fontWeight: 600 }}>解绑身份</span>}
      open={open}
      footer={null}
      onCancel={() => {
        setUnbindScannerActive(false);
        stopUnbindScanner();
        onClose();
      }}
      destroyOnClose
      width={520}
    >
      {target && (
        <>
          <div style={{ marginBottom: 16, padding: '12px 16px', background: '#fff7ed', borderRadius: 8, border: '1px solid #fed7aa' }}>
            <div style={{ color: '#9a3412', fontWeight: 500, marginBottom: 4 }}>
              解绑后账户将被清除，档案号和SFID码保留。
            </div>
            <div style={{ color: '#78716c', fontSize: 13 }}>
              账户：{target.account_pubkey ? tryEncodeSs58(target.account_pubkey) : '-'}
            </div>
          </div>

          {unbindStep === 'confirm' && (
            <div style={{ textAlign: 'center' }}>
              <Button
                type="primary"
                danger
                onClick={onUnbindGenerateChallenge}
                loading={unbindChallengeLoading}
              >
                确认解绑 — 生成签名挑战
              </Button>
            </div>
          )}

          {unbindStep === 'sign_challenge' && unbindChallenge && (
            <>
              <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                请用该公钥的 公民 钱包扫码签名
              </Typography.Text>
              <div style={{ display: 'flex', justifyContent: 'center', margin: '12px 0' }}>
                <QRCode value={unbindChallenge.sign_request} size={260} color="#134e4a" />
              </div>
              <Typography.Paragraph type="secondary" style={{ textAlign: 'center' }}>
                有效期至：{new Date(unbindChallenge.expire_at * 1000).toLocaleTimeString()}
              </Typography.Paragraph>
              <div style={{ textAlign: 'center' }}>
                <Button
                  type="primary"
                  onClick={() => {
                    setUnbindStep('scan_signature');
                    setUnbindScannerActive(true);
                  }}
                >
                  下一步：扫描签名结果
                </Button>
              </div>
            </>
          )}

          {unbindStep === 'scan_signature' && (
            <>
              <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                扫描签名结果二维码
              </Typography.Text>
              <div
                style={{
                  width: '84%',
                  maxWidth: 320,
                  aspectRatio: '1 / 1',
                  background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                  borderRadius: 16,
                  overflow: 'hidden',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  position: 'relative',
                  margin: '14px auto 12px',
                  border: '2px solid #334155',
                  boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
                }}
              >
                <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
                <video ref={unbindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                {!unbindScannerReady && (
                  <div
                    style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: unbindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
                    onClick={() => { if (!unbindScannerActive) setUnbindScannerActive(true); }}
                  >
                    <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                    <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      {unbindScannerActive ? '摄像头初始化中...' : '点击扫描签名二维码'}
                    </Typography.Text>
                  </div>
                )}
              </div>
              <div style={{ textAlign: 'center' }}>
                <Button
                  onClick={() => setUnbindScannerActive((v) => !v)}
                  loading={unbindSubmitting}
                >
                  {unbindScannerActive ? '停止扫码' : '开启扫码'}
                </Button>
              </div>
            </>
          )}
        </>
      )}
    </Modal>
  );
}
