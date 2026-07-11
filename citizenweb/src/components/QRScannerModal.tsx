import { useEffect, useRef, useState } from 'react'
import jsQR from 'jsqr'

interface QRScannerModalProps {
  onResult: (text: string) => void
  onClose: () => void
  /** 弹层标题与提示（默认扫钱包地址；签名往返时传「扫描签名结果」）。 */
  title?: string
  hint?: string
}

/** 全分辨率逐帧解码会阻塞主线程，按此间隔节流 */
const DECODE_INTERVAL_MS = 150
/** 解码用画布的最大边长，超过则等比降采样 */
const DECODE_MAX_DIMENSION = 640

/** 摄像头扫码弹层：识别到二维码后回传解码文本并由父组件关闭 */
export default function QRScannerModal({
  onResult,
  onClose,
  title = '扫码识别钱包地址',
  hint = '将 CitizenApp 钱包地址二维码对准取景框，识别后自动填入',
}: QRScannerModalProps) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const dialogRef = useRef<HTMLDivElement>(null)
  // getUserMedia 只在 HTTPS 或 localhost 下可用，环境能力在渲染期即可判定
  const [error, setError] = useState<string | null>(() =>
    window.isSecureContext && typeof navigator.mediaDevices?.getUserMedia === 'function'
      ? null
      : '当前环境不支持摄像头，请在 HTTPS 环境下使用最新版浏览器访问',
  )

  // dialog 键盘语义：Esc 关闭、焦点移入、Tab 圈闭、关闭后归还焦点
  useEffect(() => {
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null
    const dialog = dialogRef.current
    dialog?.querySelector<HTMLElement>('button')?.focus()

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.stopPropagation()
        onClose()
        return
      }
      if (event.key !== 'Tab' || !dialog) return
      const focusables = dialog.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      )
      if (focusables.length === 0) return
      const first = focusables[0]
      const last = focusables[focusables.length - 1]
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault()
        last.focus()
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      opener?.focus()
    }
  }, [onClose])

  useEffect(() => {
    if (!window.isSecureContext || typeof navigator.mediaDevices?.getUserMedia !== 'function') {
      return
    }

    let stream: MediaStream | null = null
    let rafId = 0
    let stopped = false
    let lastDecodeAt = 0
    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d', { willReadFrequently: true })

    function scanFrame(now: number) {
      if (stopped) return
      const video = videoRef.current
      if (
        video &&
        ctx &&
        video.readyState >= video.HAVE_ENOUGH_DATA &&
        now - lastDecodeAt >= DECODE_INTERVAL_MS
      ) {
        lastDecodeAt = now
        const scale = Math.min(1, DECODE_MAX_DIMENSION / Math.max(video.videoWidth, video.videoHeight))
        canvas.width = Math.round(video.videoWidth * scale)
        canvas.height = Math.round(video.videoHeight * scale)
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height)
        const image = ctx.getImageData(0, 0, canvas.width, canvas.height)
        const code = jsQR(image.data, image.width, image.height)
        if (code?.data) {
          stopped = true
          onResult(code.data)
          return
        }
      }
      rafId = requestAnimationFrame(scanFrame)
    }

    navigator.mediaDevices
      .getUserMedia({ video: { facingMode: 'environment' } })
      .then((mediaStream) => {
        if (stopped) {
          mediaStream.getTracks().forEach((track) => track.stop())
          return
        }
        stream = mediaStream
        const video = videoRef.current
        if (video) {
          video.srcObject = mediaStream
          // 弹层在播放开始前被关闭会让 play() 以 AbortError 拒绝，属预期，静默吞掉
          video.play().catch(() => {})
          rafId = requestAnimationFrame(scanFrame)
        }
      })
      .catch((cause: unknown) => {
        const name = cause instanceof DOMException ? cause.name : ''
        if (name === 'NotAllowedError') {
          setError('摄像头权限被拒绝，请在浏览器设置中允许本站访问摄像头后重试')
        } else if (name === 'NotFoundError' || name === 'OverconstrainedError') {
          setError('未检测到可用摄像头')
        } else {
          setError('摄像头启动失败，请重试')
        }
      })

    return () => {
      stopped = true
      cancelAnimationFrame(rafId)
      stream?.getTracks().forEach((track) => track.stop())
    }
  }, [onResult])

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-navy-950/80 px-6 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      onClick={onClose}
    >
      <div
        ref={dialogRef}
        className="w-full max-w-xs rounded-2xl border border-white/10 bg-navy-900 p-5"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-lg font-semibold text-white">{title}</h3>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            className="flex h-9 w-9 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-white/10 hover:text-white"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {error ? (
          <div
            role="alert"
            className="rounded-lg border border-red-400/30 bg-red-500/10 px-4 py-6 text-center text-sm text-red-100"
          >
            {error}
          </div>
        ) : (
          <div className="relative mx-auto w-full max-w-[240px] overflow-hidden rounded-xl bg-black">
            <video ref={videoRef} className="aspect-square w-full object-cover" muted playsInline />
            {/* 取景框：四角 + 中间扫描线 */}
            <div className="pointer-events-none absolute inset-8">
              <span className="absolute left-0 top-0 h-6 w-6 rounded-tl border-l-2 border-t-2 border-gold-400" />
              <span className="absolute right-0 top-0 h-6 w-6 rounded-tr border-r-2 border-t-2 border-gold-400" />
              <span className="absolute bottom-0 left-0 h-6 w-6 rounded-bl border-b-2 border-l-2 border-gold-400" />
              <span className="absolute bottom-0 right-0 h-6 w-6 rounded-br border-b-2 border-r-2 border-gold-400" />
              <span className="absolute left-2 right-2 top-1/2 h-px -translate-y-1/2 bg-gold-400/80 shadow-[0_0_8px_rgba(212,160,23,0.8)]" />
            </div>
          </div>
        )}

        <p className="mt-4 text-center text-xs text-slate-500">{hint}</p>
      </div>
    </div>
  )
}
