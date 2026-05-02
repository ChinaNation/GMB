// 绑定弹窗 —— 双模式:
// 模式1 bind_archive: 新账户绑档案(PENDING) — 扫 CPMS QR4 → 签名 → 绑定
// 模式2 bind_pubkey:  旧档案绑新账户(UNLINKED) — 扫用户二维码 → 系统比对 → 签名 → 绑定

import { useEffect, useRef, useState } from 'react';
import { Button, Form, Input, Modal, QRCode, Typography, message } from 'antd';

import type { AdminAuth } from '../auth/types';
import {
  citizenBind,
  citizenBindChallenge,
  type CitizenBindChallengeResult,
  type CitizenRow,
} from './api';
import { ScanAccountModal } from '../common/ScanAccountModal';
import { startCameraScanner } from '../utils/cameraScanner';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';

type BindMode = 'bind_archive' | 'bind_pubkey';
type BindStep =
  | 'scan_qr4'           // 模式1第一步:扫 CPMS QR4
  | 'scan_user_qr'       // 模式2第一步:扫用户钱包二维码
  | 'confirm_address'    // 模式2第二步:确认识别的地址并点击绑定
  | 'sign_challenge'     // 共用:显示签名挑战二维码
  | 'scan_signature'     // 共用:扫签名结果
  | 'done';

interface BindModalProps {
  auth: AdminAuth | null;
  open: boolean;
  record: CitizenRow | null;
  onClose: () => void;
  onBound: () => Promise<void> | void;
}

export function BindModal({ auth, open, record, onClose, onBound }: BindModalProps) {
  const [bindMode, setBindMode] = useState<BindMode>('bind_archive');
  const [bindStep, setBindStep] = useState<BindStep>('scan_qr4');
  const [bindChallenge, setBindChallenge] = useState<CitizenBindChallengeResult | null>(null);
  const [bindChallengeLoading, setBindChallengeLoading] = useState(false);
  const [bindQr4Payload, setBindQr4Payload] = useState<string | null>(null);
  const [bindQr4ScanLoading, setBindQr4ScanLoading] = useState(false);
  const [, setBindSignature] = useState<string | null>(null);
  const [bindTargetPubkey, setBindTargetPubkey] = useState('');
  // 模式2:扫用户二维码识别的地址
  const [scannedAddress, setScannedAddress] = useState('');
  const [scanAccountOpen, setScanAccountOpen] = useState(false);
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindScannerReady, setBindScannerReady] = useState(false);
  const bindVideoRef = useRef<HTMLVideoElement | null>(null);
  const bindScanCleanupRef = useRef<(() => void) | null>(null);

  // record 变化时重置
  useEffect(() => {
    if (!open || !record) return;
    const mode: BindMode = record.archive_no ? 'bind_pubkey' : 'bind_archive';
    setBindTargetPubkey(record.account_pubkey || '');
    setBindMode(mode);
    setBindChallenge(null);
    setBindQr4Payload(null);
    setBindSignature(null);
    setScannedAddress('');
    setBindStep(mode === 'bind_archive' ? 'scan_qr4' : 'scan_user_qr');
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

  // 模式1:扫 CPMS QR4
  const onScanBindQr4 = async (qrPayload: string) => {
    if (!auth) return;
    if (!qrPayload.trim()) { message.error('二维码识别失败'); return; }
    setBindQr4ScanLoading(true);
    try {
      setBindQr4Payload(qrPayload);
      setBindScannerActive(false);
      stopBindScanner();
      const challenge = await citizenBindChallenge(auth);
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      message.error(err instanceof Error ? err.message : 'QR4 扫码处理失败');
    } finally {
      setBindQr4ScanLoading(false);
    }
  };

  // 模式2:扫用户二维码后识别到地址
  const onUserAddressResolved = (addr: string) => {
    setScannedAddress(addr);
    setScanAccountOpen(false);
    setBindStep('confirm_address');
  };

  // 模式2:确认地址后点击绑定 → 生成签名挑战
  const onConfirmBind = async () => {
    if (!auth || !scannedAddress.trim()) return;
    setBindChallengeLoading(true);
    try {
      const challenge = await citizenBindChallenge(auth);
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      message.error(err instanceof Error ? err.message : '生成签名挑战失败');
    } finally {
      setBindChallengeLoading(false);
    }
  };

  // 共用:扫签名结果
  const onScanBindSignature = async (raw: string) => {
    if (!auth || !bindChallenge) return;
    if (!raw.trim()) { message.error('签名二维码识别失败'); return; }
    setBindQr4ScanLoading(true);
    try {
      const payload = parseSignedReceiptPayload(raw.trim(), bindChallenge.challenge_id);
      setBindSignature(payload.signature);
      setBindScannerActive(false);
      stopBindScanner();
      const userAddress = bindMode === 'bind_pubkey' ? scannedAddress.trim() : (bindTargetPubkey || '');
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
      message.error(err instanceof Error ? err.message : '绑定失败');
    } finally {
      setBindQr4ScanLoading(false);
    }
  };

  // 扫码 effect
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

  // 扫码窗口 UI
  const scannerBox = (label: string) => (
    <div
      style={{
        width: '84%', maxWidth: 320, aspectRatio: '1 / 1',
        background: 'linear-gradient(145deg, #0f172a, #1e293b)',
        borderRadius: 16, overflow: 'hidden',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        position: 'relative', margin: '14px auto 12px',
        border: '2px solid #334155', boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
      }}
    >
      {[{ t: 8, l: 8, bt: true, bl: true, r: 'borderTopLeftRadius' },
        { t: 8, r2: 8, bt: true, br: true, r: 'borderTopRightRadius' },
        { b: 8, l: 8, bb: true, bl: true, r: 'borderBottomLeftRadius' },
        { b: 8, r2: 8, bb: true, br: true, r: 'borderBottomRightRadius' },
      ].map((c, i) => (
        <div key={i} style={{
          position: 'absolute', width: 16, height: 16, zIndex: 2,
          ...(c.t !== undefined ? { top: c.t } : {}),
          ...(c.b !== undefined ? { bottom: c.b } : {}),
          ...(c.l !== undefined ? { left: c.l } : {}),
          ...(c.r2 !== undefined ? { right: c.r2 } : {}),
          ...(c.bt ? { borderTop: '2px solid #0d9488' } : {}),
          ...(c.bb ? { borderBottom: '2px solid #0d9488' } : {}),
          ...(c.bl ? { borderLeft: '2px solid #0d9488' } : {}),
          ...(c.br ? { borderRight: '2px solid #0d9488' } : {}),
          [c.r]: 4,
        }} />
      ))}
      <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
      {!bindScannerReady && (
        <div
          style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: bindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
          onClick={() => { if (!bindScannerActive) onToggleBindScanner(); }}
        >
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.25)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/><rect x="7" y="7" width="10" height="10" rx="1"/></svg>
          <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
            {bindScannerActive ? '摄像头初始化中...' : label}
          </Typography.Text>
        </div>
      )}
    </div>
  );

  return (
    <>
      <Modal
        title={<span style={{ fontSize: 20, fontWeight: 600 }}>绑定身份</span>}
        open={open}
        footer={null}
        onCancel={() => { setBindScannerActive(false); stopBindScanner(); onClose(); }}
        destroyOnClose
        width={520}
      >
        <Typography.Paragraph type="secondary" style={{ marginBottom: 16 }}>
          {bindMode === 'bind_archive'
            ? '模式：新账户绑档案（扫描 CPMS 档案二维码 → 签名验证 → 完成绑定）'
            : '模式：旧档案绑新账户（扫描用户二维码 → 系统比对 → 签名验证 → 完成绑定）'}
        </Typography.Paragraph>

        {/* ── 模式1:扫 CPMS QR4 ── */}
        {bindMode === 'bind_archive' && bindStep === 'scan_qr4' && (
          <>
            <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
              第一步：扫描 CPMS 档案二维码（QR4）
            </Typography.Text>
            {scannerBox('点击扫描档案二维码')}
            <div style={{ textAlign: 'center' }}>
              <Button onClick={onToggleBindScanner} loading={bindQr4ScanLoading}>
                {bindScannerActive ? '停止扫码' : '开启扫码'}
              </Button>
            </div>
          </>
        )}

        {/* ── 模式2第一步:扫用户钱包二维码 ── */}
        {bindMode === 'bind_pubkey' && bindStep === 'scan_user_qr' && (
          <>
            <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
              第一步：扫描用户钱包中的用户二维码
            </Typography.Text>
            <Form layout="vertical">
              <Form.Item label="档案号">
                <Input value={record?.archive_no ?? ''} disabled />
              </Form.Item>
              <Form.Item label="SFID码">
                <Input value={record?.sfid_code ?? ''} disabled />
              </Form.Item>
            </Form>
            <div style={{ textAlign: 'center' }}>
              <Button type="primary" onClick={() => setScanAccountOpen(true)}>
                扫描用户二维码
              </Button>
            </div>
          </>
        )}

        {/* ── 模式2第二步:确认识别的地址 ── */}
        {bindMode === 'bind_pubkey' && bindStep === 'confirm_address' && (
          <>
            <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
              第二步：确认用户账户并绑定
            </Typography.Text>
            <Form layout="vertical">
              <Form.Item label="档案号">
                <Input value={record?.archive_no ?? ''} disabled />
              </Form.Item>
              <Form.Item label="SFID码">
                <Input value={record?.sfid_code ?? ''} disabled />
              </Form.Item>
              <Form.Item label="用户账户（扫码识别）">
                <Input value={scannedAddress} disabled />
              </Form.Item>
            </Form>
            <div style={{ display: 'flex', gap: 12, justifyContent: 'center' }}>
              <Button onClick={() => { setScannedAddress(''); setBindStep('scan_user_qr'); }}>
                重新扫码
              </Button>
              <Button type="primary" onClick={onConfirmBind} loading={bindChallengeLoading}>
                绑定
              </Button>
            </div>
          </>
        )}

        {/* ── 共用:签名挑战二维码 ── */}
        {bindStep === 'sign_challenge' && bindChallenge && (
          <>
            <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
              {bindMode === 'bind_archive' ? '第二步' : '第三步'}：用公民钱包扫码签名
            </Typography.Text>
            <div style={{ display: 'flex', justifyContent: 'center', margin: '12px 0' }}>
              <QRCode value={bindChallenge.sign_request} size={260} color="#134e4a" />
            </div>
            <Typography.Paragraph type="secondary" style={{ textAlign: 'center' }}>
              有效期至：{new Date(bindChallenge.expire_at * 1000).toLocaleTimeString()}
            </Typography.Paragraph>
            <div style={{ textAlign: 'center' }}>
              <Button type="primary" onClick={() => { setBindStep('scan_signature'); setBindScannerActive(true); }}>
                下一步：扫描签名结果
              </Button>
            </div>
          </>
        )}

        {/* ── 共用:扫签名结果 ── */}
        {bindStep === 'scan_signature' && (
          <>
            <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
              {bindMode === 'bind_archive' ? '第三步' : '第四步'}：扫描签名结果二维码
            </Typography.Text>
            {scannerBox('点击扫描签名二维码')}
            <div style={{ textAlign: 'center' }}>
              <Button onClick={onToggleBindScanner} loading={bindQr4ScanLoading}>
                {bindScannerActive ? '停止扫码' : '开启扫码'}
              </Button>
            </div>
          </>
        )}
      </Modal>

      {/* 模式2:扫用户二维码弹窗(复用 ScanAccountModal) */}
      <ScanAccountModal
        open={scanAccountOpen}
        onClose={() => setScanAccountOpen(false)}
        onResolved={onUserAddressResolved}
      />
    </>
  );
}
