// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 4)
// 绑定弹窗 —— 双模式:bind_archive 扫描 CPMS 档案 QR4 / bind_pubkey 输入新账户。
// 内部持有所有 bind 相关 state 和 bindVideoRef,扫码 useEffect 完整迁移。

import { useEffect, useRef, useState } from 'react';
import { Button, Form, Input, Modal, QRCode, Typography, message } from 'antd';
import { QrcodeOutlined } from '@ant-design/icons';
import type { AdminAuth, CitizenBindChallengeResult, CitizenRow } from '../../api/client';
import { citizenBind, citizenBindChallenge } from '../../api/client';
import { startCameraScanner } from '../../utils/cameraScanner';
import { parseKeyringSignedPayload } from '../../utils/parseSignedPayload';

type BindMode = 'bind_archive' | 'bind_pubkey';
type BindStep = 'scan_qr4' | 'sign_challenge' | 'scan_signature' | 'input_pubkey' | 'done';

interface BindModalProps {
  auth: AdminAuth | null;
  open: boolean;
  record: CitizenRow | null;
  onClose: () => void;
  onBound: () => Promise<void> | void;
}

export function BindModal({ auth, open, record, onClose, onBound }: BindModalProps) {
  const [bindTargetPubkey, setBindTargetPubkey] = useState('');
  const [bindMode, setBindMode] = useState<BindMode>('bind_archive');
  const [bindChallenge, setBindChallenge] = useState<CitizenBindChallengeResult | null>(null);
  const [bindChallengeLoading, setBindChallengeLoading] = useState(false);
  const [bindQr4Payload, setBindQr4Payload] = useState<string | null>(null);
  const [bindQr4ScanLoading, setBindQr4ScanLoading] = useState(false);
  const [, setBindSignature] = useState<string | null>(null);
  const [bindStep, setBindStep] = useState<BindStep>('scan_qr4');
  const [bindNewPubkey, setBindNewPubkey] = useState('');
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindScannerReady, setBindScannerReady] = useState(false);
  const bindVideoRef = useRef<HTMLVideoElement | null>(null);
  const bindScanCleanupRef = useRef<(() => void) | null>(null);

  // 当 record 变化时,重置表单状态(等价于原 openBindModal 初始化)
  useEffect(() => {
    if (!open || !record) return;
    const mode: BindMode = (record.status === 'UNLINKED' || record.status === 'PENDING') ? 'bind_pubkey' : 'bind_archive';
    setBindTargetPubkey(record.account_pubkey || '');
    setBindMode(mode);
    setBindChallenge(null);
    setBindQr4Payload(null);
    setBindSignature(null);
    setBindNewPubkey('');
    setBindStep(mode === 'bind_archive' ? 'scan_qr4' : 'input_pubkey');
    setBindScannerActive(false);
    stopBindScanner();
  }, [open, record]);

  const stopBindScanner = () => {
    if (bindScanCleanupRef.current) {
      bindScanCleanupRef.current();
      bindScanCleanupRef.current = null;
    }
    setBindScannerReady(false);
  };

  const onToggleBindScanner = () => {
    if (!open) return;
    if (bindScannerActive) {
      setBindScannerActive(false);
      stopBindScanner();
      return;
    }
    setBindScannerActive(true);
  };

  const onScanBindQr4 = async (qrPayload: string) => {
    if (!auth) return;
    if (!qrPayload.trim()) {
      message.error('二维码识别失败');
      return;
    }
    setBindQr4ScanLoading(true);
    try {
      setBindQr4Payload(qrPayload);
      message.success('QR4 扫码成功，正在生成签名挑战...');
      setBindScannerActive(false);
      stopBindScanner();
      const challenge = await citizenBindChallenge(auth);
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'QR4 扫码处理失败';
      message.error(msg);
    } finally {
      setBindQr4ScanLoading(false);
    }
  };

  const onBindPubkeyNext = async () => {
    if (!auth) return;
    if (!bindNewPubkey.trim()) {
      message.error('请输入新账户');
      return;
    }
    setBindChallengeLoading(true);
    try {
      const challenge = await citizenBindChallenge(auth);
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成签名挑战失败';
      message.error(msg);
    } finally {
      setBindChallengeLoading(false);
    }
  };

  const onScanBindSignature = async (raw: string) => {
    if (!auth || !bindChallenge) return;
    const trimmed = raw.trim();
    if (!trimmed) {
      message.error('签名二维码识别失败');
      return;
    }
    setBindQr4ScanLoading(true);
    try {
      const payload = parseKeyringSignedPayload(trimmed, bindChallenge.challenge_id);
      setBindSignature(payload.signature);
      setBindScannerActive(false);
      stopBindScanner();
      const userAddress = bindMode === 'bind_pubkey' ? bindNewPubkey.trim() : (bindTargetPubkey || '');
      const result = await citizenBind(auth, {
        mode: bindMode,
        user_address: userAddress,
        qr4_payload: bindQr4Payload || undefined,
        citizen_id: record?.id,
        challenge_id: payload.challenge_id,
        signature: payload.signature,
      });
      message.success(`绑定成功${result.sfid_code ? `，SFID码：${result.sfid_code}` : ''}`);
      onClose();
      await onBound();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '绑定失败';
      message.error(msg);
    } finally {
      setBindQr4ScanLoading(false);
    }
  };

  useEffect(() => {
    if (!open || !bindScannerActive || !bindVideoRef.current) {
      stopBindScanner();
      return;
    }
    const currentStep = bindStep;
    bindScanCleanupRef.current = startCameraScanner(
      bindVideoRef.current,
      (raw) => {
        setBindScannerActive(false);
        stopBindScanner();
        if (currentStep === 'scan_qr4') {
          void onScanBindQr4(raw);
        } else if (currentStep === 'scan_signature') {
          void onScanBindSignature(raw);
        }
      },
      () => setBindScannerReady(true),
      (msg) => { message.error(msg); setBindScannerActive(false); },
    );
    return () => stopBindScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, bindScannerActive, bindStep]);

  return (
    <Modal
      title={<span style={{ fontSize: 20, fontWeight: 600 }}>绑定身份</span>}
      open={open}
      footer={null}
      onCancel={() => {
        setBindScannerActive(false);
        stopBindScanner();
        onClose();
      }}
      destroyOnClose
      width={520}
    >
      <Typography.Paragraph type="secondary" style={{ marginBottom: 16 }}>
        {bindMode === 'bind_archive'
          ? '模式：有账户绑档案（扫描 CPMS 档案二维码 → 签名验证 → 完成绑定）'
          : '模式：有档案绑账户（输入新账户 → 签名验证 → 完成绑定）'}
      </Typography.Paragraph>

      {bindMode === 'bind_archive' && bindStep === 'scan_qr4' && (
        <>
          <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
            第一步：扫描 CPMS 档案二维码（QR4）
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
            <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
            {!bindScannerReady && (
              <div
                style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: bindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
                onClick={() => { if (!bindScannerActive) onToggleBindScanner(); }}
              >
                <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                  {bindScannerActive ? '摄像头初始化中...' : '点击扫描档案二维码'}
                </Typography.Text>
              </div>
            )}
          </div>
          <div style={{ textAlign: 'center' }}>
            <Button onClick={onToggleBindScanner} loading={bindQr4ScanLoading}>
              {bindScannerActive ? '停止扫码' : '开启扫码'}
            </Button>
          </div>
        </>
      )}

      {bindMode === 'bind_pubkey' && bindStep === 'input_pubkey' && (
        <>
          <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
            第一步：输入新账户
          </Typography.Text>
          <Form layout="vertical">
            <Form.Item label="记录ID">
              <Input value={record?.id ?? ''} disabled />
            </Form.Item>
            <Form.Item label="档案号">
              <Input value={record?.archive_no ?? ''} disabled />
            </Form.Item>
            <Form.Item label="SFID码">
              <Input value={record?.sfid_code ?? ''} disabled />
            </Form.Item>
            <Form.Item label="新账户" required>
              <Input
                value={bindNewPubkey}
                onChange={(e) => setBindNewPubkey(e.target.value)}
                placeholder="请输入新账户（SS58 地址）"
              />
            </Form.Item>
            <Button type="primary" onClick={onBindPubkeyNext} loading={bindChallengeLoading}>
              下一步：生成签名挑战
            </Button>
          </Form>
        </>
      )}

      {bindStep === 'sign_challenge' && bindChallenge && (
        <>
          <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
            第二步：用 公民 钱包扫码签名
          </Typography.Text>
          <div style={{ display: 'flex', justifyContent: 'center', margin: '12px 0' }}>
            <QRCode value={bindChallenge.sign_request} size={260} color="#134e4a" />
          </div>
          <Typography.Paragraph type="secondary" style={{ textAlign: 'center' }}>
            有效期至：{new Date(bindChallenge.expire_at * 1000).toLocaleTimeString()}
          </Typography.Paragraph>
          <div style={{ textAlign: 'center' }}>
            <Button
              type="primary"
              onClick={() => {
                setBindStep('scan_signature');
                setBindScannerActive(true);
              }}
            >
              下一步：扫描签名结果
            </Button>
          </div>
        </>
      )}

      {bindStep === 'scan_signature' && (
        <>
          <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
            第三步：扫描签名结果二维码
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
            <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
            {!bindScannerReady && (
              <div
                style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: bindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
                onClick={() => { if (!bindScannerActive) onToggleBindScanner(); }}
              >
                <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                  {bindScannerActive ? '摄像头初始化中...' : '点击扫描签名二维码'}
                </Typography.Text>
              </div>
            )}
          </div>
          <div style={{ textAlign: 'center' }}>
            <Button onClick={onToggleBindScanner} loading={bindQr4ScanLoading}>
              {bindScannerActive ? '停止扫码' : '开启扫码'}
            </Button>
          </div>
        </>
      )}
    </Modal>
  );
}
