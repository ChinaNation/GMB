// 登录页:WUMIN_QR_V1 双向扫码登录
// 左侧展示 challenge 二维码 → 手机扫码签名
// 右侧摄像头扫码 → 扫描手机签名回执 → 完成登录

import { useState, useEffect, useRef } from 'react';
import { ScanIcon } from '../components/ScanIcon';
import { useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import { useAuth } from '../auth';
import * as api from '../api';
import { startCameraScanner } from '../utils/cameraScanner';

export default function LoginPage() {
  const { login } = useAuth();
  const navigate = useNavigate();

  const [qrChallenge, setQrChallenge] = useState<{
    challenge_id: string;
    login_qr_payload: string;
    session_id: string;
    expire_at: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [scannerActive, setScannerActive] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [scanSubmitting, setScanSubmitting] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scanCleanupRef = useRef<(() => void) | null>(null);
  const pollingRef = useRef<number | null>(null);
  const loggedInRef = useRef(false);

  const stopScanner = () => {
    if (scanCleanupRef.current) {
      scanCleanupRef.current();
      scanCleanupRef.current = null;
    }
    setScannerReady(false);
  };

  const stopPolling = () => {
    if (pollingRef.current !== null) {
      window.clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
  };

  const doLogin = (accessToken: string, user: { user_id: string; role: string }) => {
    if (loggedInRef.current) return;
    loggedInRef.current = true;
    stopPolling();
    stopScanner();
    setScannerActive(false);
    login(accessToken, user);
    navigate(user.role === 'SUPER_ADMIN' ? '/admin' : '/operator');
  };

  useEffect(() => {
    return () => { stopScanner(); stopPolling(); };
  }, []);

  const handleGenerateQr = async () => {
    setError('');
    setLoading(true);
    stopPolling();
    loggedInRef.current = false;
    try {
      const res = await api.authQrChallenge();
      if (res.data) {
        setQrChallenge(res.data);
        const { challenge_id, session_id } = res.data;
        pollingRef.current = window.setInterval(async () => {
          try {
            const r = await api.authQrResult(challenge_id, session_id);
            if (r.data?.status === 'SUCCESS' && r.data.access_token && r.data.user) {
              doLogin(r.data.access_token, r.data.user);
            } else if (r.data?.status === 'EXPIRED') {
              stopPolling();
              setError('二维码已过期，请重新生成');
              setQrChallenge(null);
            }
          } catch { /* keep polling */ }
        }, 1500);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '生成登录二维码失败');
    } finally {
      setLoading(false);
    }
  };

  // 摄像头扫码
  useEffect(() => {
    if (!scannerActive || !qrChallenge || !videoRef.current) {
      stopScanner();
      return;
    }
    const video = videoRef.current;
    const cleanup = startCameraScanner(
      video,
      (raw) => { handleReceiptScanned(raw); },
      () => { setScannerReady(true); },
      (msg) => { setError(msg); setScannerActive(false); },
    );
    scanCleanupRef.current = cleanup;
    return () => stopScanner();
  }, [scannerActive, qrChallenge]);

  const handleReceiptScanned = async (raw: string) => {
    if (!qrChallenge) return;
    setScanSubmitting(true);
    try {
      const { parseQrEnvelope, QrParseError } = await import('../qr/wuminQr');
      let env;
      try {
        env = parseQrEnvelope(raw);
      } catch (e) {
        const msg = e instanceof QrParseError ? e.message : '签名二维码格式无效';
        setError(msg);
        setScanSubmitting(false);
        return;
      }
      if (env.kind !== 'login_receipt') {
        setError(`期望 login_receipt,实际: ${env.kind}`);
        setScanSubmitting(false);
        return;
      }
      const body = env.body as { pubkey: string; signature: string };
      if (!body.pubkey || !body.signature) {
        setError('签名二维码缺少 pubkey/signature');
        setScanSubmitting(false);
        return;
      }

      await api.authQrComplete({
        challenge_id: env.id || qrChallenge.challenge_id,
        session_id: qrChallenge.session_id,
        admin_pubkey: body.pubkey,
        signature: body.signature,
      });

      const result = await api.authQrResult(qrChallenge.challenge_id, qrChallenge.session_id);
      if (result.data?.status === 'SUCCESS' && result.data.access_token && result.data.user) {
        doLogin(result.data.access_token, result.data.user);
        return;
      }
      setError('登录验证失败，请重试');
    } catch (e) {
      const msg = e instanceof Error ? e.message : '签名处理失败';
      if (msg.includes('admin not found')) {
        setError('非管理员禁止登录本系统');
      } else {
        setError(msg);
      }
    } finally {
      setScanSubmitting(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card" style={{ width: 680 }}>
        <div className="login-card__header">
          <div className="login-card__title">CPMS</div>
          <div className="login-card__subtitle">公民护照管理系统 — 管理员扫码登录</div>
        </div>
        <div className="login-card__body">
          {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{error}</div>}

          <div style={{ display: 'flex', gap: 24, alignItems: 'stretch', flexWrap: 'wrap' }}>
            {/* 左侧：登录二维码 */}
            <div style={{ flex: '1 1 260px', minWidth: 240, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              <div style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text)', marginBottom: 12 }}>登录二维码</div>
              <div style={{
                width: 260, height: 260,
                background: '#f8fffe',
                borderRadius: 16,
                border: '2px solid #e6f7f5',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                overflow: 'hidden',
              }}>
                <div style={{
                  filter: qrChallenge ? 'none' : 'blur(3px) opacity(0.4)',
                  transition: 'filter 0.3s ease',
                }}>
                  <QRCodeSVG
                    value={qrChallenge?.login_qr_payload || 'CPMS_LOGIN_PENDING'}
                    size={228}
                    fgColor="#134e4a"
                  />
                </div>
              </div>
              <div style={{ marginTop: 10, textAlign: 'center', fontSize: 12, color: 'var(--color-text-secondary)' }}>
                {qrChallenge
                  ? `有效期至 ${new Date(qrChallenge.expire_at * 1000).toLocaleTimeString()}`
                  : '请点击按钮生成二维码'}
              </div>
              <button
                className="btn btn--primary"
                style={{ width: 200, marginTop: 10 }}
                onClick={handleGenerateQr}
                disabled={loading}
              >
                {loading ? '生成中...' : qrChallenge ? '重新生成' : '生成二维码'}
              </button>
            </div>

            {/* 分割线 */}
            <div style={{
              width: 1,
              background: 'linear-gradient(to bottom, transparent, var(--color-border), transparent)',
              alignSelf: 'stretch',
            }} />

            {/* 右侧：扫码窗口 */}
            <div style={{ flex: '1 1 260px', minWidth: 240, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              <div style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text)', marginBottom: 12 }}>扫码窗口</div>
              <div style={{
                width: 260, height: 260,
                background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                borderRadius: 16,
                overflow: 'hidden',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                position: 'relative',
                border: '2px solid #334155',
              }}>
                <video ref={videoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                {!scannerReady && (
                  <div style={{
                    position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column',
                    alignItems: 'center', justifyContent: 'center', gap: 8,
                  }}>
                    <ScanIcon size={32} color="rgba(255,255,255,0.25)" />
                    <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      {scannerActive ? '摄像头初始化中...' : '等待开启摄像头'}
                    </div>
                  </div>
                )}
              </div>
              <div style={{ marginTop: 10, textAlign: 'center', fontSize: 12, color: 'var(--color-text-secondary)' }}>
                开启摄像头扫描签名回执二维码
              </div>
              <button
                className="btn btn--ghost"
                style={{ width: 200, marginTop: 10 }}
                onClick={() => {
                  if (!qrChallenge) {
                    setError('请先生成登录二维码');
                    return;
                  }
                  setScannerActive(v => !v);
                }}
                disabled={scanSubmitting}
              >
                {scannerActive ? '停止扫码' : '开启扫码'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
