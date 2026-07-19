import { useState } from 'react'

/// GitHub 最新发布直下基址：`releases/latest/download/<固定资产名>` 永远指向最新版对应资产，
/// 前提是每次发版都用相同的固定资产名（见下方 asset 命名）。
const RELEASE_BASE = 'https://github.com/ChinaNation/GMB/releases/latest/download'

/** 一个平台下载项：'store' 弹提示文案（如 iOS 去 App Store），'file' 直下 GitHub 最新资产。 */
export type DownloadOption =
  | { label: string; kind: 'store'; message: string }
  | { label: string; kind: 'file'; asset: string }

interface DownloadButtonProps {
  /** 无障碍标签，标明所属产品，如「CitizenApp」。 */
  productLabel: string
  options: DownloadOption[]
}

export default function DownloadButton({ productLabel, options }: DownloadButtonProps) {
  const [open, setOpen] = useState(false)

  return (
    <div className="relative shrink-0">
      <button
        type="button"
        aria-label={`下载 ${productLabel}`}
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
        className="flex items-center gap-1 text-xl font-bold text-gold-400 transition-colors hover:text-gold-300"
      >
        下载
        <span className={`text-sm transition-transform ${open ? 'rotate-180' : ''}`} aria-hidden="true">
          ▾
        </span>
      </button>

      {open && (
        <>
          {/* 点击菜单外任意处关闭。 */}
          <div className="fixed inset-0 z-40" aria-hidden="true" onClick={() => setOpen(false)} />
          <div
            role="menu"
            className="absolute right-0 z-50 mt-2 min-w-[168px] overflow-hidden rounded-xl border border-gold-500/30 bg-navy-950/95 shadow-2xl shadow-black/40 backdrop-blur-xl"
          >
            {options.map((option) =>
              option.kind === 'store' ? (
                <button
                  key={option.label}
                  type="button"
                  role="menuitem"
                  onClick={() => {
                    setOpen(false)
                    window.alert(option.message)
                  }}
                  className="block w-full px-4 py-3 text-left text-sm font-medium text-slate-200 transition-colors hover:bg-white/5 hover:text-gold-300"
                >
                  {option.label}
                </button>
              ) : (
                <a
                  key={option.label}
                  role="menuitem"
                  href={`${RELEASE_BASE}/${encodeURIComponent(option.asset)}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={() => setOpen(false)}
                  className="block w-full px-4 py-3 text-left text-sm font-medium text-slate-200 no-underline transition-colors hover:bg-white/5 hover:text-gold-300"
                >
                  {option.label}
                </a>
              ),
            )}
          </div>
        </>
      )}
    </div>
  )
}
