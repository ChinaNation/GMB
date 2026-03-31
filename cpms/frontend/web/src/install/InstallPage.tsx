import { useState, useEffect, useRef } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';
import type { InstallStatus } from '../types';

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};
type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

export default function InstallPage() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [msg, setMsg] = useState('');
  const [scannerActive, setScannerActive] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [qr2Generating, setQr2Generating] = useState(false);
  const [qr2Payload, setQr2Payload] = useState<string | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);

  const load = async () => {
    try {
      const res = await api.installStatus();
      if (res.data) {
        setStatus(res.data);
        if (res.data.qr2_payload) {
          setQr2Payload(res.data.qr2_payload);
        }
      }
    } catch { /* ignore */ }
  };

  useEffect(() => { load(); }, []);

  const stopScanner = () => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach(t => t.stop());
      streamRef.current = null;
    }
    setScannerReady(false);
  };

  // 摄像头扫码 QR1
  useEffect(() => {
    if (!scannerActive) {
      stopScanner();
      return;
    }
    let cancelled = false;
    const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
    if (!win.BarcodeDetector) {
      setError('当前浏览器不支持摄像头扫码，请使用 Chrome');
      setScannerActive(false);
      return;
    }
    const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
    const start = async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: 'environment' },
          audio: false,
        });
        if (cancelled) {
          stream.getTracks().forEach(t => t.stop());
          return;
        }
        streamRef.current = stream;
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          await videoRef.current.play();
          setScannerReady(true);
        }
        const timer = window.setInterval(async () => {
          if (!videoRef.current || loading) return;
          try {
            const codes = await detector.detect(videoRef.current);
            const raw = codes[0]?.rawValue?.trim();
            if (raw) {
              window.clearInterval(timer);
              await handleQr1Scanned(raw);
            }
          } catch { /* ignore frame errors */ }
        }, 700);
        return () => window.clearInterval(timer);
      } catch {
        setError('无法打开摄像头，请检查权限');
        setScannerActive(false);
      }
    };
    let clear: (() => void) | undefined;
    start().then(fn => { clear = fn; });
    return () => {
      cancelled = true;
      if (clear) clear();
      stopScanner();
    };
  }, [scannerActive, loading]);

  const handleQr1Scanned = async (qrContent: string) => {
    setError('');
    setMsg('');
    setLoading(true);
    setScannerActive(false);
    stopScanner();
    try {
      const res = await api.installInitialize(qrContent);
      if (res.data) {
        setMsg(`初始化成功，站点 SFID: ${res.data.site_sfid}`);
      }
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '初始化失败');
    }
    setLoading(false);
  };

  const handleGenerateQr2 = async () => {
    setError('');
    setQr2Generating(true);
    try {
      const res = await api.installGenerateQr2();
      if (res.data) {
        setQr2Payload(res.data.qr2_payload);
        setMsg('QR2 已生成，请将此二维码展示给 SFID 管理员扫码注册');
      }
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '生成 QR2 失败');
    }
    setQr2Generating(false);
  };

  // 判断当前步骤
  const initialized = status?.initialized ?? false;
  const adminBound = (status?.super_admin_bound_count ?? 0) >= 1;
  const hasQr2 = Boolean(qr2Payload);
  const anonCertDone = status?.anon_cert_done ?? false;

  let currentStep = 1;
  if (initialized && !adminBound) currentStep = 2;
  if (initialized && adminBound && !hasQr2) currentStep = 3;
  if (initialized && adminBound && hasQr2) currentStep = 4;
  if (anonCertDone) currentStep = 5;

  return (
    <div className="login-page">
      <div className="login-card" style={{ width: 580 }}>
        <div className="login-card__header">
          <div className="login-card__title">CPMS 系统初始化</div>
          <div className="login-card__subtitle">公民护照管理系统</div>
        </div>
        <div className="login-card__body">
          {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{error}</div>}
          {msg && <div style={{ color: 'var(--color-success)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{msg}</div>}

          {/* 步骤指示器 */}
          <div style={{ display: 'flex', gap: 8, marginBottom: 20, justifyContent: 'center' }}>
            {['扫码QR1', '绑定管理员', '生成QR2', '完成'].map((label, i) => (
              <div key={i} style={{
                padding: '4px 12px',
                borderRadius: 6,
                fontSize: 12,
                fontWeight: 500,
                background: currentStep > i + 1 ? '#dcfce7' : currentStep === i + 1 ? 'var(--color-primary)' : '#f3f4f6',
                color: currentStep > i + 1 ? 'var(--color-success)' : currentStep === i + 1 ? '#fff' : '#9ca3af',
              }}>
                {label}
              </div>
            ))}
          </div>

          {/* 步骤1：扫码 QR1 */}
          {currentStep === 1 && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title">步骤 1：扫描 SFID 安装授权二维码（QR1）</div>
              <div style={{
                width: '80%',
                maxWidth: 280,
                aspectRatio: '1 / 1',
                background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                borderRadius: 16,
                overflow: 'hidden',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                position: 'relative',
                margin: '12px auto',
                border: '2px solid #334155',
              }}>
                <video ref={videoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                {!scannerReady && (
                  <div style={{
                    position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column',
                    alignItems: 'center', justifyContent: 'center', gap: 8,
                    cursor: scannerActive ? 'default' : 'pointer', userSelect: 'none',
                  }} onClick={() => { if (!scannerActive) setScannerActive(true); }}>
                    <div style={{ fontSize: 28, color: 'rgba(255,255,255,0.25)' }}>📷</div>
                    <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      {scannerActive ? '摄像头初始化中...' : '点击开启摄像头扫码'}
                    </div>
                  </div>
                )}
              </div>
              <div style={{ textAlign: 'center', marginTop: 8 }}>
                <button
                  className="btn btn--primary"
                  onClick={() => setScannerActive(v => !v)}
                  disabled={loading}
                >
                  {loading ? '处理中...' : scannerActive ? '停止扫码' : '开启扫码'}
                </button>
              </div>
            </div>
          )}

          {/* 步骤2：绑定超级管理员 */}
          {currentStep === 2 && status && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title">步骤 2：绑定超级管理员</div>
              <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 12 }}>
                使用公民钱包 App 扫描以下二维码完成管理员绑定
              </div>
              <div style={{ display: 'flex', gap: 16, justifyContent: 'center', flexWrap: 'wrap' }}>
                {status.super_admin_bind_qrs.map(qr => (
                  <div key={qr.key_id} style={{ textAlign: 'center' }}>
                    {qr.bound ? (
                      <div style={{
                        width: 180, height: 180, borderRadius: 12,
                        background: '#dcfce7', display: 'flex', alignItems: 'center', justifyContent: 'center',
                        color: 'var(--color-success)', fontWeight: 600, fontSize: 16,
                      }}>
                        已绑定 ✓
                      </div>
                    ) : (
                      <QRCodeSVG value={qr.qr_content} size={180} fgColor="#134e4a" />
                    )}
                    <div style={{ marginTop: 6, fontSize: 12, color: 'var(--color-text-secondary)' }}>{qr.key_id}</div>
                  </div>
                ))}
              </div>
              <div style={{ textAlign: 'center', marginTop: 16 }}>
                <button className="btn btn--ghost" onClick={load}>刷新状态</button>
              </div>
            </div>
          )}

          {/* 步骤3：生成 QR2 */}
          {currentStep === 3 && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title">步骤 3：生成注册二维码（QR2）</div>
              <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 16 }}>
                超级管理员已绑定，点击生成 QR2 后拿给 SFID 管理员扫码注册
              </div>
              <div style={{ textAlign: 'center' }}>
                <button className="btn btn--primary" onClick={handleGenerateQr2} disabled={qr2Generating}>
                  {qr2Generating ? '生成中...' : '生成 QR2'}
                </button>
              </div>
            </div>
          )}

          {/* 步骤4：展示 QR2 */}
          {currentStep === 4 && qr2Payload && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title">步骤 3：注册二维码（QR2）</div>
              <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 12 }}>
                将此二维码展示给 SFID 管理员扫码注册，拿到 QR3 后登录系统完成盲化
              </div>
              <div style={{ display: 'flex', justifyContent: 'center', margin: '12px 0' }}>
                <QRCodeSVG value={qr2Payload} size={220} fgColor="#134e4a" />
              </div>
            </div>
          )}

          {/* 步骤5：完成 */}
          {currentStep === 5 && (
            <div style={{ textAlign: 'center', padding: '20px 0' }}>
              <div style={{ fontSize: 36, marginBottom: 12 }}>✅</div>
              <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--color-success)', marginBottom: 8 }}>
                系统初始化完成
              </div>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 4 }}>
                站点 SFID: <strong>{status?.site_sfid}</strong>
              </div>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 20 }}>
                匿名证书已注册，系统已具备 QR4 生成能力
              </div>
            </div>
          )}

          {/* 前往登录（管理员已绑定 + QR2 已生成后可见） */}
          {initialized && adminBound && (
            <div className="mt-16 text-center">
              <a href="/login" className="btn btn--primary">前往登录</a>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
