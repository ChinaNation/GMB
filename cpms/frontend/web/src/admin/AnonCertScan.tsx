import { useState, useEffect, useRef } from 'react';
import QrScanner from 'qr-scanner';
import * as api from '../api';

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};
type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

export default function AnonCertScan() {
  const [scannerActive, setScannerActive] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [anonCertDone, setAnonCertDone] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scanCleanupRef = useRef<(() => void) | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    api.installStatus().then(res => {
      if (res.data?.anon_cert_done) setAnonCertDone(true);
    }).catch(() => {});
  }, []);

  const stopScanner = () => {
    if (scanCleanupRef.current) {
      scanCleanupRef.current();
      scanCleanupRef.current = null;
    }
    setScannerReady(false);
  };

  const handleQr3Scanned = async (raw: string) => {
    setError('');
    setSuccess('');
    setSubmitting(true);
    setScannerActive(false);
    stopScanner();
    try {
      await api.adminProcessAnonCert(raw);
      setSuccess('匿名证书注册完成，系统已具备 QR4 生成能力');
      setAnonCertDone(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : '处理 QR3 失败');
    }
    setSubmitting(false);
  };

  // 摄像头扫码用 BarcodeDetector
  useEffect(() => {
    if (!scannerActive || !videoRef.current) {
      stopScanner();
      return;
    }
    let stopped = false;
    let stream: MediaStream | null = null;
    let timer: number | undefined;
    const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
    if (!win.BarcodeDetector) {
      setError('当前浏览器不支持摄像头扫码');
      setScannerActive(false);
      return;
    }
    const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
    const video = videoRef.current;
    (async () => {
      try {
        stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' }, audio: false });
        if (stopped) { stream.getTracks().forEach(t => t.stop()); return; }
        video.srcObject = stream;
        await video.play();
        setScannerReady(true);
        timer = window.setInterval(async () => {
          if (stopped || submitting) return;
          try {
            const codes = await detector.detect(video);
            const raw = codes[0]?.rawValue?.trim();
            if (raw) { window.clearInterval(timer); await handleQr3Scanned(raw); }
          } catch { /* ignore */ }
        }, 500);
      } catch {
        setError('无法打开摄像头，请检查权限');
        setScannerActive(false);
      }
    })();
    scanCleanupRef.current = () => {
      stopped = true;
      if (timer !== undefined) window.clearInterval(timer);
      if (stream) stream.getTracks().forEach(t => t.stop());
    };
    return () => stopScanner();
  }, [scannerActive, submitting]);

  // 图片上传用 qr-scanner
  const onUploadQrImage = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (fileInputRef.current) fileInputRef.current.value = '';
    if (!file) return;
    try {
      const result = await QrScanner.scanImage(file, { returnDetailedScanResult: true });
      const raw = result.data?.trim();
      if (raw) {
        await handleQr3Scanned(raw);
      } else {
        setError('未识别到二维码，请确认图片中包含有效的二维码');
      }
    } catch {
      setError('未识别到二维码，请确认图片中包含有效的二维码');
    }
  };

  if (anonCertDone) {
    return (
      <div className="card">
        <div className="card__title">匿名证书</div>
        <div style={{ textAlign: 'center', padding: '20px 0' }}>
          <div style={{ fontSize: 36, marginBottom: 12 }}>✅</div>
          <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--color-success)' }}>
            匿名证书已注册完成
          </div>
          <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginTop: 8 }}>
            系统已具备 QR4 生成能力
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <div className="card__title">扫描匿名证书二维码（QR3）</div>
      <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 16 }}>
        将 SFID 返回的 QR3 二维码放到摄像头前扫描，完成匿名证书注册。
      </div>

      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}
      {success && <div style={{ color: 'var(--color-success)', fontSize: 13, marginBottom: 12 }}>{success}</div>}

      <div style={{
        width: '80%',
        maxWidth: 300,
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
            <div style={{ fontSize: 28, color: 'rgba(255,255,255,0.25)' }}>📷</div>
            <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
              点击开启摄像头扫码
            </div>
          </div>
        )}
      </div>
      <div style={{ textAlign: 'center', marginTop: 12 }}>
        <button
          className="btn btn--primary"
          onClick={() => setScannerActive(v => !v)}
          disabled={submitting}
        >
          {submitting ? '处理中...' : scannerActive ? '停止扫码' : '开启扫码'}
        </button>
        <input type="file" accept="image/*" ref={fileInputRef} style={{ display: 'none' }} onChange={onUploadQrImage} />
        <button
          className="btn btn--ghost"
          onClick={() => fileInputRef.current?.click()}
          disabled={submitting}
          style={{ marginLeft: 8 }}
        >
          上传二维码
        </button>
      </div>
    </div>
  );
}
