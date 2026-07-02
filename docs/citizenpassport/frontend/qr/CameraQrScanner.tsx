import { useEffect, useRef, useState } from 'react';
import { ScanIcon } from '../components/ScanIcon';
import { decodeQrImageFile, startCameraScanner } from './cameraScanner';

type CameraQrScannerProps = {
  active: boolean;
  onActiveChange: (active: boolean) => void;
  onDetected: (raw: string) => boolean | void | Promise<void>;
  onError: (message: string) => void;
  buttonLabel?: string;
  stopLabel?: string;
  idleText?: string;
  loadingText?: string;
  hint?: string;
  busy?: boolean;
  size?: number;
  showButton?: boolean;
  // 开启后在统一组件内渲染「上传二维码」入口，与摄像头共用 BarcodeDetector 和 onDetected，
  // 不在页面里另起第二套扫码逻辑；图片只在前端本地解析。
  allowUpload?: boolean;
  uploadLabel?: string;
};

// CPMS 所有二维码读取统一走摄像头，禁止在页面内分散实现第二套扫码入口。
export default function CameraQrScanner({
  active,
  onActiveChange,
  onDetected,
  onError,
  buttonLabel = '开启扫码',
  stopLabel = '停止扫码',
  idleText = '等待开启摄像头',
  loadingText = '摄像头初始化中...',
  hint,
  busy = false,
  size = 260,
  showButton = true,
  allowUpload = false,
  uploadLabel = '上传二维码',
}: CameraQrScannerProps) {
  const [ready, setReady] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const stopScanner = () => {
    cleanupRef.current?.();
    cleanupRef.current = null;
    setReady(false);
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = ''; // 允许重复选同一文件
    if (!file || busy) return;
    if (active) onActiveChange(false); // 上传解码与摄像头不同时推进
    try {
      const raw = await decodeQrImageFile(file);
      onDetected(raw);
    } catch (err) {
      onError(err instanceof Error ? err.message : '二维码图片识别失败');
    }
  };

  useEffect(() => {
    if (!active || !videoRef.current) {
      stopScanner();
      return;
    }
    cleanupRef.current?.();
    cleanupRef.current = startCameraScanner(
      videoRef.current,
      (raw) => {
        const accepted = onDetected(raw);
        if (accepted !== false) {
          onActiveChange(false);
        }
        return accepted === false ? false : undefined;
      },
      () => setReady(true),
      (message) => {
        setReady(false);
        onError(message);
      },
    );
    return () => stopScanner();
  }, [active]);

  useEffect(() => () => stopScanner(), []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
      <div style={{
        width: size,
        height: size,
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
        {!ready && (
          <button
            type="button"
            onClick={() => !busy && onActiveChange(true)}
            disabled={busy}
            style={{
              position: 'absolute',
              inset: 0,
              border: 0,
              background: 'transparent',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 8,
              cursor: busy ? 'default' : 'pointer',
            }}
            aria-label={active ? loadingText : idleText}
          >
            <ScanIcon size={32} color="rgba(255,255,255,0.25)" />
            <span style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
              {active ? loadingText : idleText}
            </span>
          </button>
        )}
      </div>
      {hint && (
        <div style={{ marginTop: 10, textAlign: 'center', fontSize: 12, color: 'var(--color-text-secondary)' }}>
          {hint}
        </div>
      )}
      {showButton && (
        <div style={{ display: 'flex', gap: 8, marginTop: 10, justifyContent: 'center' }}>
          <button
            className="btn btn--ghost"
            style={{ width: allowUpload ? 140 : 200 }}
            onClick={() => onActiveChange(!active)}
            disabled={busy}
          >
            {active ? stopLabel : buttonLabel}
          </button>
          {allowUpload && (
            <>
              <button
                className="btn btn--ghost"
                style={{ width: 140 }}
                onClick={() => fileInputRef.current?.click()}
                disabled={busy}
              >
                {uploadLabel}
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/png,image/jpeg,image/webp,image/gif,image/bmp"
                style={{ display: 'none' }}
                onChange={handleUpload}
              />
            </>
          )}
        </div>
      )}
    </div>
  );
}
