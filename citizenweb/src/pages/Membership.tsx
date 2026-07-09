import { useCallback, useMemo, useState } from 'react'
import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'
import QrScannerModal from '../components/QrScannerModal'

type MembershipLevel = 'visitor' | 'voting' | 'candidate'

interface Plan {
  level: MembershipLevel
  name: string
  price: string
  identity: string
  dynamic: string
  article: string
}

const plans: Plan[] = [
  {
    level: 'visitor',
    name: '访客会员',
    price: '$2.99 / 月',
    identity: '任意钱包账户',
    dynamic: '动态：300 字、9 张标清图片、1 分钟标清视频',
    article: '文章：20,000 字、50 张标清图片、1 张高清首图',
  },
  {
    level: 'voting',
    name: '投票公民会员',
    price: '$9.99 / 月',
    identity: '认证投票公民',
    dynamic: '动态：300 字、9 张高清图片、30 分钟高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
  {
    level: 'candidate',
    name: '竞选公民会员',
    price: '$99.99 / 月',
    identity: '认证选举公民',
    dynamic: '动态：300 字、9 张高清图片、3 小时高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
]

const apiBaseUrl =
  import.meta.env.VITE_CITIZENAPP_SQUARE_API_BASE_URL?.replace(/\/+$/, '') ??
  'https://citizenapp-square-api.stews87-fawn.workers.dev'

/**
 * 从二维码文本中提取钱包地址：兼容纯地址与 substrate:地址:哈希 之类的 URI。
 * 只接受完整的 40–64 位 base58 段（前后不能再连着 base58 字符，避免截断长串），无匹配返回 null。
 */
function extractWalletAddress(raw: string): string | null {
  const match = raw
    .trim()
    .match(/(?<![1-9A-HJ-NP-Za-km-z])[1-9A-HJ-NP-Za-km-z]{40,64}(?![1-9A-HJ-NP-Za-km-z])/)
  return match ? match[0] : null
}

export default function Membership() {
  const [ownerAccount, setOwnerAccount] = useState('')
  const [selectedLevel, setSelectedLevel] = useState<MembershipLevel>('visitor')
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [scannerOpen, setScannerOpen] = useState(false)

  const handleScanResult = useCallback((text: string) => {
    const address = extractWalletAddress(text)
    if (address) {
      setOwnerAccount(address)
      setMessage(null)
    } else {
      setMessage('二维码中未识别到钱包地址，请扫描 CitizenApp 钱包地址二维码')
    }
    setScannerOpen(false)
  }, [])

  const handleScannerClose = useCallback(() => {
    setScannerOpen(false)
  }, [])

  const selectedPlan = useMemo(
    () => plans.find((plan) => plan.level === selectedLevel) ?? plans[0],
    [selectedLevel],
  )

  async function startCheckout() {
    const account = ownerAccount.trim()
    if (!account) {
      setMessage('请输入钱包账户地址')
      return
    }
    setLoading(true)
    setMessage(null)
    try {
      const response = await fetch(`${apiBaseUrl}/v1/square/membership/stripe/checkout`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          owner_account: account,
          membership_level: selectedLevel,
        }),
      })
      const data = (await response.json()) as {
        checkout_url?: string
        message?: string
      }
      if (!response.ok || !data.checkout_url) {
        throw new Error(data.message ?? '订阅创建失败')
      }
      window.location.assign(data.checkout_url)
    } catch (error) {
      setMessage(error instanceof Error ? error.message : '订阅创建失败')
      setLoading(false)
    }
  }

  return (
    <>
      <section className="border-b border-white/10 bg-navy-900/35 py-10 md:py-12">
        <div className="mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="CitizenApp Membership"
            title="公民 App 会员订阅"
            description="三档会员统一按美元计价，订阅权益绑定到钱包账户；Stripe Checkout 负责银行卡、本地法币和已启用的 USDC 支付。"
          />
        </div>
      </section>

      <section className="mx-auto grid max-w-7xl gap-8 px-6 py-20 lg:grid-cols-[minmax(0,1fr)_380px]">
        <div className="grid gap-6 md:grid-cols-3">
          {plans.map((plan) => (
            <button
              key={plan.level}
              type="button"
              onClick={() => setSelectedLevel(plan.level)}
              className={`rounded-2xl border p-0 text-left transition-all ${
                selectedLevel === plan.level
                  ? 'border-gold-400 bg-gold-500/10'
                  : 'border-white/[0.08] bg-white/[0.03] hover:border-white/[0.18] hover:bg-white/[0.05]'
              }`}
            >
              <div className="p-6">
                <div className="text-sm font-medium text-gold-400">{plan.identity}</div>
                <h2 className="mt-3 text-2xl font-bold text-white">{plan.name}</h2>
                <div className="mt-4 text-3xl font-extrabold text-white">{plan.price}</div>
                <div className="mt-6 space-y-3 text-sm leading-relaxed text-slate-300">
                  <p>{plan.dynamic}</p>
                  <p>{plan.article}</p>
                </div>
              </div>
            </button>
          ))}
        </div>

        <GlowCard glow="gold" className="h-fit">
          <div className="text-sm font-medium text-gold-400">当前选择</div>
          <h2 className="mt-2 text-2xl font-bold text-white">{selectedPlan.name}</h2>
          <p className="mt-2 text-sm text-slate-400">{selectedPlan.price}</p>

          <label className="mt-8 block text-sm font-semibold text-slate-200" htmlFor="owner-account">
            钱包账户地址
          </label>
          <div className="relative mt-3">
            <input
              id="owner-account"
              value={ownerAccount}
              onChange={(event) => setOwnerAccount(event.target.value)}
              className="w-full rounded-lg border border-white/10 bg-navy-950 py-3 pl-4 pr-12 text-sm text-white outline-none transition-colors placeholder:text-slate-600 focus:border-gold-400"
              placeholder="输入 CitizenApp 钱包地址"
            />
            <button
              type="button"
              onClick={() => setScannerOpen(true)}
              aria-label="扫码识别钱包地址"
              title="扫码识别钱包地址"
              className="absolute right-1.5 top-1/2 flex h-9 w-9 -translate-y-1/2 items-center justify-center rounded-md text-slate-400 transition-colors hover:bg-white/5 hover:text-gold-400"
            >
              {/* 扫码图标：取景框四角 + 中间一条横线 */}
              <svg
                className="h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M4 8V6a2 2 0 0 1 2-2h2" />
                <path d="M16 4h2a2 2 0 0 1 2 2v2" />
                <path d="M20 16v2a2 2 0 0 1-2 2h-2" />
                <path d="M8 20H6a2 2 0 0 1-2-2v-2" />
                <path d="M4 12h16" />
              </svg>
            </button>
          </div>

          <button
            type="button"
            onClick={startCheckout}
            disabled={loading}
            className="mt-5 w-full rounded-lg bg-gold-500 px-5 py-3 text-sm font-bold text-navy-950 transition-colors hover:bg-gold-400 disabled:cursor-not-allowed disabled:bg-slate-600 disabled:text-slate-300"
          >
            {loading ? '正在创建订阅...' : '进入 Stripe 订阅'}
          </button>

          {message && (
            <div className="mt-4 rounded-lg border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-100">
              {message}
            </div>
          )}

          <div className="mt-6 border-t border-white/10 pt-5 text-xs leading-relaxed text-slate-500">
            App Store 和 Google Play 版本只显示订阅状态；订阅支付统一在官网完成。
          </div>
        </GlowCard>
      </section>

      {scannerOpen && <QrScannerModal onResult={handleScanResult} onClose={handleScannerClose} />}
    </>
  )
}
