// 摄像头 QR 扫码 + 图片 QR 识别 通用工具。
// 基于浏览器原生 BarcodeDetector API,不依赖任何第三方库。

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};
type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

/**
 * 启动摄像头 BarcodeDetector 扫码。
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
  if (!win.BarcodeDetector) {
    onError('当前浏览器不支持摄像头扫码');
    return () => {};
  }
  const detector = new win.BarcodeDetector({ formats: ['qr_code'] });

  (async () => {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'environment' },
        audio: false,
      });
      if (stopped) {
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      videoEl.srcObject = stream;
      await videoEl.play();
      onReady();
      timer = window.setInterval(async () => {
        if (stopped) return;
        try {
          const codes = await detector.detect(videoEl);
          const raw = codes[0]?.rawValue?.trim();
          if (raw) {
            window.clearInterval(timer);
            onDetected(raw);
          }
        } catch {
          /* ignore frame errors */
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

/**
 * 从图片文件识别 QR 码(用于"上传二维码"场景)。
 * 基于 BarcodeDetector,不依赖 qr-scanner 库。
 */
export async function scanImageQr(file: File): Promise<string> {
  const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
  if (!win.BarcodeDetector) {
    throw new Error('当前浏览器不支持二维码识别');
  }
  const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
  const bitmap = await createImageBitmap(file);
  try {
    const codes = await detector.detect(bitmap);
    const raw = codes[0]?.rawValue?.trim();
    if (!raw) throw new Error('未识别到二维码，请确认图片中包含有效的二维码');
    return raw;
  } finally {
    bitmap.close();
  }
}
