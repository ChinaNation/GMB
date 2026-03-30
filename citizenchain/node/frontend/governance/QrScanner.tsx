// 摄像头 QR 扫描组件：使用 jsQR 做纯 JS 解码，兼容 Tauri WebView。
// 通过 getUserMedia 获取摄像头流，用 canvas 逐帧解码。
import { useEffect, useRef, useCallback } from 'react';
import jsQR from 'jsqr';

type Props = {
  onScan: (data: string) => void;
  onError: (error: string) => void;
};

export function QrScanner({ onScan, onError }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animFrameRef = useRef<number>(0);
  const streamRef = useRef<MediaStream | null>(null);
  const scannedRef = useRef(false);

  const stopCamera = useCallback(() => {
    if (animFrameRef.current) {
      cancelAnimationFrame(animFrameRef.current);
      animFrameRef.current = 0;
    }
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
  }, []);

  useEffect(() => {
    let mounted = true;

    async function start() {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { width: { ideal: 640 }, height: { ideal: 480 } },
        });
        if (!mounted) { stream.getTracks().forEach((t) => t.stop()); return; }
        streamRef.current = stream;
        const video = videoRef.current;
        if (!video) return;
        video.srcObject = stream;
        video.setAttribute('playsinline', 'true');
        await video.play();
        requestScan();
      } catch (e) {
        if (mounted) onError(`摄像头启动失败: ${e}`);
      }
    }

    function requestScan() {
      animFrameRef.current = requestAnimationFrame(scan);
    }

    function scan() {
      if (!mounted || scannedRef.current) return;
      const video = videoRef.current;
      const canvas = canvasRef.current;
      if (!video || !canvas || video.readyState !== video.HAVE_ENOUGH_DATA) {
        requestScan();
        return;
      }

      const ctx = canvas.getContext('2d', { willReadFrequently: true });
      if (!ctx) { requestScan(); return; }

      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);

      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const code = jsQR(imageData.data, imageData.width, imageData.height, {
        inversionAttempts: 'dontInvert',
      });

      if (code && code.data) {
        scannedRef.current = true;
        stopCamera();
        onScan(code.data);
        return;
      }

      requestScan();
    }

    start();

    return () => {
      mounted = false;
      stopCamera();
    };
  }, [onScan, onError, stopCamera]);

  return (
    <div className="qr-scanner-wrapper">
      <video ref={videoRef} className="qr-scanner-video" muted playsInline />
      <canvas ref={canvasRef} style={{ display: 'none' }} />
      <div className="qr-scanner-overlay">
        <div className="qr-scanner-frame" />
      </div>
    </div>
  );
}
