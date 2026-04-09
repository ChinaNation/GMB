// 中文注释:把 AntD <QRCode /> 渲染出的 canvas 导出成带白边 PNG 下载。
// 原实现散在 CpmsSitePanel.tsx 里,步 0 抽成通用 util,让后续 SFID 二维码、
// CPMS 二维码、多签二维码都能复用,避免每个组件复制一份。

export interface DownloadQrOptions {
  // QR 外层 DOM 容器(通常是 ref 指向的 <div>),里面必须包含一个 <canvas>
  container: HTMLElement | null;
  // 文件名(不含扩展名),内部会做一次 [^\w.-]+ -> _ 的安全化
  filename: string;
  // 白边宽度,默认 32px
  padding?: number;
  // 失败时的回调(比如 message.error),不抛异常
  onError?: (msg: string) => void;
}

function safeName(name: string): string {
  return name.replace(/[^\w.-]+/g, '_');
}

export function downloadQr(opts: DownloadQrOptions): void {
  const { container, filename, padding = 32, onError } = opts;
  if (!container) {
    onError?.('二维码未就绪');
    return;
  }
  const sourceCanvas = container.querySelector('canvas');
  if (!sourceCanvas) {
    onError?.('未找到二维码画布');
    return;
  }
  const w = sourceCanvas.width;
  const h = sourceCanvas.height;
  const outCanvas = document.createElement('canvas');
  outCanvas.width = w + padding * 2;
  outCanvas.height = h + padding * 2;
  const ctx = outCanvas.getContext('2d');
  if (!ctx) {
    onError?.('导出失败');
    return;
  }
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, outCanvas.width, outCanvas.height);
  ctx.drawImage(sourceCanvas, padding, padding);
  const link = document.createElement('a');
  link.href = outCanvas.toDataURL('image/png');
  link.download = `${safeName(filename)}.png`;
  link.click();
}
