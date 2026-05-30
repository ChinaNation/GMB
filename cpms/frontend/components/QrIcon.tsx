// 统一 QR 码图标组件。全仓库 CPMS 前端所有扫码/QR 相关图标统一使用此组件。

export function QrIcon({ size = 20, color = 'currentColor' }: { size?: number; color?: string }) {
  return (
    <svg viewBox="0 0 24 24" width={size} height={size} fill="none" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="7" rx="1" />
      <rect x="14" y="3" width="7" height="7" rx="1" />
      <rect x="3" y="14" width="7" height="7" rx="1" />
      <rect x="14" y="14" width="3" height="3" />
      <rect x="18" y="18" width="3" height="3" />
      <rect x="14" y="18" width="3" height="3" />
      <rect x="18" y="14" width="3" height="3" />
    </svg>
  );
}
