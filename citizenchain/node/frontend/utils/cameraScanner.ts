// 摄像头 QR 扫码通用工具。
//
// 策略:优先使用浏览器原生 BarcodeDetector(Chromium 系,如 Windows Tauri/WebView2),
// 不支持时 fallback 到 jsqr + canvas 逐帧解码(macOS WKWebView / Linux WebKitGTK)。
// 对外接口统一,调用方无需关心底层实现。

import jsQR from 'jsqr';

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};
type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

/**
 * 启动摄像头 QR 扫码。
 *
 * @param videoEl 已挂载的 `<video>` 元素
 * @param onDetected 扫到 QR 时触发(调用后自动停止轮询,调用方决定是否 cleanup)
 * @param onReady 摄像头流就绪时触发
 * @param onError 摄像头打开失败 / 浏览器不支持时触发
 * @returns cleanup 函数
 */
export function startCameraScanner(
  videoEl: HTMLVideoElement,
  onDetected: (raw: string) => void,
  onReady: () => void,
  onError: (msg: string) => void,
): () => void {
  let stopped = false;
  let stream: MediaStream | null = null;
  let timer: number | undefined;

  const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
  const hasBarcodeDetector = Boolean(win.BarcodeDetector);

  const detector = hasBarcodeDetector
    ? new win.BarcodeDetector!({ formats: ['qr_code'] })
    : null;

  // jsqr fallback 用的 canvas(离屏,不挂 DOM)
  let canvas: HTMLCanvasElement | null = null;
  let ctx: CanvasRenderingContext2D | null = null;
  if (!detector) {
    canvas = document.createElement('canvas');
    ctx = canvas.getContext('2d', { willReadFrequently: true });
  }

  (async () => {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: hasBarcodeDetector
          ? { facingMode: 'environment' }
          : { facingMode: 'environment', width: { ideal: 1280 }, height: { ideal: 720 } },
        audio: false,
      });
      if (stopped) {
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      videoEl.srcObject = stream;
      await videoEl.play();
      onReady();

      timer = window.setInterval(() => {
        if (stopped) return;

        if (detector) {
          // ── BarcodeDetector 路径 ──
          detector.detect(videoEl).then((codes) => {
            const raw = codes[0]?.rawValue?.trim();
            if (raw) {
              window.clearInterval(timer);
              onDetected(raw);
            }
          }).catch(() => { /* ignore frame errors */ });
        } else if (canvas && ctx) {
          // ── jsqr + canvas fallback ──
          if (videoEl.readyState !== videoEl.HAVE_ENOUGH_DATA) return;
          canvas.width = videoEl.videoWidth;
          canvas.height = videoEl.videoHeight;
          ctx.drawImage(videoEl, 0, 0, canvas.width, canvas.height);
          const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
          const code = jsQR(imageData.data, imageData.width, imageData.height, {
            inversionAttempts: 'attemptBoth',
          });
          if (code?.data) {
            window.clearInterval(timer);
            onDetected(code.data);
          }
        }
      }, 500);
    } catch {
      onError('无法打开摄像头,请检查权限');
    }
  })();

  return () => {
    stopped = true;
    if (timer !== undefined) window.clearInterval(timer);
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
    }
  };
}
