// 摄像头 QR 扫描组件。底层由 cameraScanner.ts 统一封装:
// Chromium WebView 用 BarcodeDetector,WebKit 用 jsqr + canvas。
import { useEffect, useRef, useCallback, useState } from 'react';
import { startCameraScanner } from '../utils/cameraScanner';

type Props = {
  onScan: (data: string) => void;
  onError: (error: string) => void;
};

export function QrScanner({ onScan, onError }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const cleanupRef = useRef<(() => void) | null>(null);
  const [ready, setReady] = useState(false);

  const stop = useCallback(() => {
    if (cleanupRef.current) {
      cleanupRef.current();
      cleanupRef.current = null;
    }
  }, []);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const cleanup = startCameraScanner(
      video,
      (raw) => {
        stop();
        onScan(raw);
      },
      () => { setReady(true); },
      (msg) => { onError(msg); },
    );
    cleanupRef.current = cleanup;

    return () => stop();
  }, [onScan, onError, stop]);

  return (
    <div className="qr-scanner-wrapper">
      <video ref={videoRef} className="qr-scanner-video" muted playsInline />
      <div className="qr-scanner-overlay">
        <div className="qr-scanner-frame" />
      </div>
    </div>
  );
}
