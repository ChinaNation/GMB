// 摄像头 QR 扫码通用工具。
// 基于浏览器原生 BarcodeDetector API,不依赖任何第三方库。

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};
type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

function createQrDetector(unsupportedMessage: string): BarcodeDetectorLike {
  const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
  if (!win.BarcodeDetector) {
    throw new Error(unsupportedMessage);
  }
  return new win.BarcodeDetector({ formats: ['qr_code'] });
}

/**
 * 启动摄像头 BarcodeDetector 扫码。
 *
 * @param videoEl 已挂载的 `<video>` 元素
 * @param onDetected 扫到 QR 时触发；返回 false 时继续轮询，其他返回值表示本次扫码已被消费
 * @param onReady 摄像头流就绪时触发
 * @param onError 摄像头打开失败 / 浏览器不支持时触发
 * @returns cleanup 函数
 */
export function startCameraScanner(
  videoEl: HTMLVideoElement,
  onDetected: (raw: string) => boolean | void,
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
            const accepted = onDetected(raw);
            if (accepted !== false) {
              window.clearInterval(timer);
            }
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
 * 从用户上传的图片文件中识别二维码内容。
 *
 * 中文注释：上传二维码与摄像头扫码共用 BarcodeDetector，只产生一份二维码原文，
 * 仍交给调用方已有的 onDetected 流程，不新增第二套业务逻辑；图片只在前端本地解析。
 */
export async function decodeQrImageFile(file: File): Promise<string> {
  const isImage =
    file.type.startsWith('image/') ||
    /\.(png|jpe?g|webp|gif|bmp)$/i.test(file.name);
  if (!isImage) {
    throw new Error('请上传二维码图片文件');
  }
  if (typeof createImageBitmap !== 'function') {
    throw new Error('当前浏览器不支持二维码图片解析');
  }

  const detector = createQrDetector('当前浏览器不支持二维码图片识别');
  let bitmap: ImageBitmap | null = null;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    throw new Error('二维码图片读取失败，请上传清晰的图片文件');
  }

  try {
    const codes = await detector.detect(bitmap);
    const raw = codes.find((code) => code.rawValue?.trim())?.rawValue?.trim();
    if (!raw) {
      throw new Error('未识别到二维码，请上传清晰的二维码图片');
    }
    return raw;
  } catch (err) {
    if (err instanceof Error && err.message.startsWith('未识别到二维码')) {
      throw err;
    }
    throw new Error('二维码图片识别失败，请换用清晰的二维码图片');
  } finally {
    bitmap.close();
  }
}
