import { useState, useEffect, useRef } from 'react';
import * as api from '../api';
import type { InstallStatus } from '../types';
import { startCameraScanner, scanImageQr } from '../utils/cameraScanner';
import { parseQrEnvelope, QrParseError } from '../qr/wuminQr';
import type { UserContactBody } from '../qr/wuminQr';
import { ScanIcon } from '../components/ScanIcon';

// CPMS 初始化页面。
// 三个事实状态步骤：1.扫 QR1  2.绑定管理员  3.完成（前往登录）
// CPMS 是离线系统，不判断授权是否失效——只有 SFID 侧能判断。

export default function InstallPage() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [msg, setMsg] = useState('');
  const [scannerActive, setScannerActive] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindScannerReady, setBindScannerReady] = useState(false);
  const [bindLoading, setBindLoading] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scanCleanupRef = useRef<(() => void) | null>(null);
  const bindVideoRef = useRef<HTMLVideoElement | null>(null);
  const bindScanCleanupRef = useRef<(() => void) | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const load = async () => {
    try {
      const res = await api.installStatus();
      if (res.data) {
        setStatus(res.data);
      }
    } catch { /* ignore */ }
  };

  useEffect(() => { load(); }, []);

  const stopScanner = () => {
    if (scanCleanupRef.current) {
      scanCleanupRef.current();
      scanCleanupRef.current = null;
    }
    setScannerReady(false);
  };

  useEffect(() => {
    if (!scannerActive || !videoRef.current) {
      stopScanner();
      return;
    }
    const video = videoRef.current;
    const cleanup = startCameraScanner(
      video,
      (raw) => { handleQr1Scanned(raw); },
      () => { setScannerReady(true); },
      (msg) => { setError(msg); setScannerActive(false); },
    );
    scanCleanupRef.current = cleanup;
    return () => stopScanner();
  }, [scannerActive]);

  const stopBindScanner = () => {
    if (bindScanCleanupRef.current) {
      bindScanCleanupRef.current();
      bindScanCleanupRef.current = null;
    }
    setBindScannerReady(false);
  };

  useEffect(() => {
    if (!bindScannerActive || !bindVideoRef.current) {
      stopBindScanner();
      return;
    }
    const video = bindVideoRef.current;
    const cleanup = startCameraScanner(
      video,
      (raw) => { handleBindScanned(raw); },
      () => { setBindScannerReady(true); },
      (msg) => { setError(msg); setBindScannerActive(false); },
    );
    bindScanCleanupRef.current = cleanup;
    return () => stopBindScanner();
  }, [bindScannerActive]);

  const handleBindScanned = async (raw: string) => {
    setError('');
    setMsg('');
    setBindLoading(true);
    setBindScannerActive(false);
    stopBindScanner();
    try {
      // WUMIN_QR_V1 统一协议：解析 user_contact envelope，取 address（SS58）
      const env = parseQrEnvelope(raw);
      if (env.kind !== 'user_contact') {
        throw new Error(`需要扫描公民名片二维码（user_contact），当前为 ${env.kind}`);
      }
      const { address } = env.body as UserContactBody;
      // SS58 address 传后端，后端做 SS58→hex 解码
      await api.bindSuperAdmin(address.trim());
      setMsg('超级管理员绑定成功');
      await load();
    } catch (e) {
      if (e instanceof QrParseError) {
        setError(`二维码格式错误：${e.message}`);
      } else {
        setError(e instanceof Error ? e.message : '绑定失败');
      }
    }
    setBindLoading(false);
  };

  const onUploadQrImage = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (fileInputRef.current) fileInputRef.current.value = '';
    if (!file) return;
    try {
      const raw = await scanImageQr(file);
      await handleQr1Scanned(raw);
    } catch {
      setError('未识别到二维码，请确认图片中包含有效的二维码');
    }
  };

  const handleQr1Scanned = async (qrContent: string) => {
    setError('');
    setMsg('');
    setLoading(true);
    setScannerActive(false);
    stopScanner();
    try {
      const res = await api.installInitialize(qrContent);
      if (res.data) {
        setMsg(`站点 SFID: ${res.data.site_sfid}`);
      }
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '初始化失败');
    }
    setLoading(false);
  };

  // 三个事实状态：1.未初始化  2.已初始化未绑定管理员  3.已初始化已绑定管理员
  const initialized = status?.initialized ?? false;
  const adminBound = (status?.super_admin_bound_count ?? 0) >= 1;

  let currentStep = 1;
  if (initialized && !adminBound) currentStep = 2;
  if (initialized && adminBound) currentStep = 3;

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

          <div style={{ display: 'flex', gap: 8, marginBottom: 20, justifyContent: 'center' }}>
            {['扫码QR1', '绑定管理员', '完成'].map((label, i) => (
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

          {currentStep === 1 && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title">扫描 SFID 安装授权二维码（QR1）</div>
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
                {!scannerReady && !scannerActive && (
                  <div style={{
                    position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column',
                    alignItems: 'center', justifyContent: 'center', gap: 8,
                    cursor: 'pointer', userSelect: 'none',
                  }} onClick={() => setScannerActive(true)}>
                    <ScanIcon size={32} color="rgba(255,255,255,0.25)" />
                    <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      点击开启摄像头扫码
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
                <input type="file" accept="image/*" ref={fileInputRef} style={{ display: 'none' }} onChange={onUploadQrImage} />
                <button
                  className="btn btn--ghost"
                  onClick={() => fileInputRef.current?.click()}
                  disabled={loading}
                  style={{ marginLeft: 8 }}
                >
                  上传二维码
                </button>
              </div>
            </div>
          )}

          {currentStep === 2 && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title">绑定超级管理员</div>
              <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 12 }}>
                打开手机公民钱包，展示用户名片二维码，用摄像头扫码读取公钥
              </div>
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
                <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                {!bindScannerReady && !bindScannerActive && (
                  <div style={{
                    position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column',
                    alignItems: 'center', justifyContent: 'center', gap: 8,
                    cursor: 'pointer', userSelect: 'none',
                  }} onClick={() => setBindScannerActive(true)}>
                    <ScanIcon size={32} color="rgba(255,255,255,0.25)" />
                    <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      点击开启摄像头扫码
                    </div>
                  </div>
                )}
              </div>
              <div style={{ textAlign: 'center', marginTop: 8 }}>
                <button
                  className="btn btn--primary"
                  onClick={() => setBindScannerActive(v => !v)}
                  disabled={bindLoading}
                >
                  {bindLoading ? '绑定中...' : bindScannerActive ? '停止扫码' : '开启扫码'}
                </button>
              </div>
            </div>
          )}

          {currentStep === 3 && (
            <div style={{ textAlign: 'center', padding: '20px 0' }}>
              <div style={{ fontSize: 36, marginBottom: 12 }}>✅</div>
              <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--color-success)', marginBottom: 8 }}>
                初始化完成
              </div>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 4 }}>
                站点 SFID: <strong>{status?.site_sfid}</strong>
              </div>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 20 }}>
                请登录系统完成后续配置（生成 QR2、扫描 QR3）
              </div>
              <a href="/login" className="btn btn--primary">前往登录</a>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
