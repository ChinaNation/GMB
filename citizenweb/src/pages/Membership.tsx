import { useCallback, useMemo, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'
import QrScannerModal from '../components/QrScannerModal'
import { buildSquareActionSignRequest, parseSignResponseSignature } from '../lib/qrV1'

type MembershipLevel = 'visitor' | 'voting' | 'candidate'
type TabKey = 'subscribe' | 'cancel'
type Tone = 'error' | 'info' | 'success'

interface Plan {
  level: MembershipLevel
  name: string
  price: string
  identity: string
  dynamic: string
  article: string
}

interface Signing {
  action: TabKey
  challengeId: string
  requestQr: string
  level?: MembershipLevel
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

async function postJson(path: string, body: unknown): Promise<Record<string, unknown>> {
  const response = await fetch(`${apiBaseUrl}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  const data = (await response.json().catch(() => ({}))) as Record<string, unknown>
  if (!response.ok) {
    throw new Error(typeof data.message === 'string' ? data.message : '请求失败')
  }
  return data
}

export default function Membership() {
  const [activeTab, setActiveTab] = useState<TabKey>('subscribe')
  const [ownerAccount, setOwnerAccount] = useState('')
  const [selectedLevel, setSelectedLevel] = useState<MembershipLevel>('visitor')
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState<{ tone: Tone; text: string } | null>(null)
  const [signing, setSigning] = useState<Signing | null>(null)
  const [scannerMode, setScannerMode] = useState<'address' | 'signature' | null>(null)

  const selectedPlan = useMemo(
    () => plans.find((plan) => plan.level === selectedLevel) ?? plans[0],
    [selectedLevel],
  )

  const switchTab = useCallback((tab: TabKey) => {
    setActiveTab(tab)
    setSigning(null)
    setMessage(null)
  }, [])

  // 发起签名：取挑战 → 构建 signRequest 二维码等 CitizenApp 扫码签名。
  const beginSigning = useCallback(
    async (action: TabKey) => {
      const owner = ownerAccount.trim()
      if (!owner) {
        setMessage({ tone: 'error', text: '请输入钱包账户地址' })
        return
      }
      setLoading(true)
      setMessage(null)
      setSigning(null)
      try {
        const level = selectedLevel
        const data =
          action === 'subscribe'
            ? await postJson('/v1/square/membership/subscribe/challenge', {
                owner_account: owner,
                membership_level: level,
              })
            : await postJson('/v1/square/membership/cancel/challenge', {
                owner_account: owner,
              })
        const challengeId = data.challenge_id
        const signingPayloadHex = data.signing_payload_hex
        const ownerPubkeyHex = data.owner_pubkey_hex
        if (
          typeof challengeId !== 'string' ||
          typeof signingPayloadHex !== 'string' ||
          typeof ownerPubkeyHex !== 'string'
        ) {
          throw new Error('签名挑战响应不完整')
        }
        const requestQr = buildSquareActionSignRequest({
          challengeId,
          ownerPubkeyHex,
          signingPayloadHex,
        })
        setSigning({
          action,
          challengeId,
          requestQr,
          level: action === 'subscribe' ? level : undefined,
        })
      } catch (error) {
        setMessage({ tone: 'error', text: error instanceof Error ? error.message : '发起签名失败' })
      } finally {
        setLoading(false)
      }
    },
    [ownerAccount, selectedLevel],
  )

  // 扫回 App 的 signResponse → 提交 Worker 验签 → 订阅转 Stripe / 取消提示成功。
  const submitSignature = useCallback(
    async (raw: string) => {
      const active = signing
      if (!active) return
      const signature = parseSignResponseSignature(raw)
      if (!signature) {
        setMessage({ tone: 'error', text: '未识别到签名二维码，请扫描 CitizenApp 生成的签名结果' })
        return
      }
      const owner = ownerAccount.trim()
      setLoading(true)
      setMessage(null)
      try {
        if (active.action === 'subscribe') {
          const data = await postJson('/v1/square/membership/subscribe', {
            owner_account: owner,
            membership_level: active.level,
            challenge_id: active.challengeId,
            signature,
          })
          if (typeof data.checkout_url !== 'string') {
            throw new Error('订阅创建失败')
          }
          window.location.assign(data.checkout_url)
          return
        }
        await postJson('/v1/square/membership/cancel', {
          owner_account: owner,
          challenge_id: active.challengeId,
          signature,
        })
        setSigning(null)
        setMessage({ tone: 'success', text: '已提交取消订阅，当期结束后生效' })
      } catch (error) {
        setMessage({ tone: 'error', text: error instanceof Error ? error.message : '提交失败' })
      } finally {
        setLoading(false)
      }
    },
    [signing, ownerAccount],
  )

  const handleScanResult = useCallback(
    (text: string) => {
      const mode = scannerMode
      setScannerMode(null)
      if (mode === 'signature') {
        void submitSignature(text)
        return
      }
      const address = extractWalletAddress(text)
      if (address) {
        setOwnerAccount(address)
        setMessage(null)
      } else {
        setMessage({ tone: 'error', text: '二维码中未识别到钱包地址，请扫描 CitizenApp 钱包地址二维码' })
      }
    },
    [scannerMode, submitSignature],
  )

  const handleScannerClose = useCallback(() => setScannerMode(null), [])

  const messageClass =
    message?.tone === 'success'
      ? 'border-emerald-400/30 bg-emerald-500/10 text-emerald-100'
      : message?.tone === 'info'
        ? 'border-white/15 bg-white/5 text-slate-200'
        : 'border-red-400/30 bg-red-500/10 text-red-100'

  return (
    <>
      <section className="border-b border-white/10 bg-navy-900/35 py-10 md:py-12">
        <div className="mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="CitizenApp Membership"
            title="公民 App 会员订阅"
            description="三档会员统一按美元计价，订阅与取消都由 CitizenApp 钱包扫码签名授权；Stripe Checkout 负责银行卡、本地法币和已启用的 USDC 支付。"
          />
        </div>
      </section>

      <section className="mx-auto grid max-w-7xl auto-rows-fr gap-6 px-6 py-20 sm:grid-cols-2 lg:grid-cols-4">
        {plans.map((plan) => (
            <button
              key={plan.level}
              type="button"
              onClick={() => setSelectedLevel(plan.level)}
              className={`h-full rounded-2xl border p-0 text-left transition-all ${
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

        <GlowCard glow="gold" className="flex flex-col">
          {/* 订阅 / 取消订阅 分段切换 */}
          <div className="flex rounded-lg border border-white/10 bg-navy-950 p-1 text-sm font-semibold">
            {(['subscribe', 'cancel'] as const).map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => switchTab(tab)}
                className={`flex-1 rounded-md py-2 transition-colors ${
                  activeTab === tab ? 'bg-gold-500 text-navy-950' : 'text-slate-300 hover:text-white'
                }`}
              >
                {tab === 'subscribe' ? '订阅会员' : '取消订阅'}
              </button>
            ))}
          </div>

          {/* 固定高度：订阅(当前选择/档位/价格)与取消(说明)内容行数不同，
              锁死高度后切换 tab 不会撑高卡片。 */}
          <div className="mt-6 min-h-[104px]">
            {activeTab === 'subscribe' ? (
              <>
                <div className="text-sm font-medium text-gold-400">当前选择</div>
                <h2 className="mt-2 text-2xl font-bold text-white">{selectedPlan.name}</h2>
                <p className="mt-2 text-sm text-slate-400">{selectedPlan.price}</p>
              </>
            ) : (
              <p className="text-sm leading-relaxed text-slate-300">
                取消订阅将在当前计费周期结束后生效；期间会员权益不变。
              </p>
            )}
          </div>

          <label className="mt-8 block text-sm font-semibold text-slate-200" htmlFor="owner-account">
            钱包账户地址
          </label>
          <div className="relative mt-3">
            <input
              id="owner-account"
              value={ownerAccount}
              onChange={(event) => setOwnerAccount(event.target.value)}
              className="w-full rounded-lg border border-white/10 bg-navy-950 py-3 pl-4 pr-12 text-sm text-white outline-none transition-colors placeholder:text-slate-600 focus:border-gold-400"
              placeholder="输入 公民 钱包地址"
            />
            <button
              type="button"
              onClick={() => setScannerMode('address')}
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
            onClick={() => beginSigning(activeTab)}
            disabled={loading || signing !== null}
            className="mt-5 w-full rounded-lg bg-gold-500 px-5 py-3 text-sm font-bold text-navy-950 transition-colors hover:bg-gold-400 disabled:cursor-not-allowed disabled:bg-slate-600 disabled:text-slate-300"
          >
            {loading
              ? '正在处理...'
              : activeTab === 'subscribe'
                ? '扫码签名并订阅'
                : '扫码签名并取消'}
          </button>

          <div className="mt-auto border-t border-white/10 pt-5 text-xs leading-relaxed text-slate-500">
            App Store 和 Google Play 版本只显示订阅状态；订阅与取消的支付操作统一在官网完成。
          </div>
        </GlowCard>
      </section>

      {/* 扫码签名弹层：QR 与扫描独立成模态，保证订阅/取消卡片高度始终统一。 */}
      {signing && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-navy-950/80 px-6 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label="扫码签名"
          onClick={() => setSigning(null)}
        >
          <div
            className="w-full max-w-sm rounded-2xl border border-white/10 bg-navy-900 p-6"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="mb-2 flex items-center justify-between">
              <h3 className="text-lg font-semibold text-white">
                {signing.action === 'subscribe' ? '扫码签名并订阅' : '扫码签名并取消'}
              </h3>
              <button
                type="button"
                onClick={() => setSigning(null)}
                aria-label="关闭"
                className="flex h-9 w-9 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-white/10 hover:text-white"
              >
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <p className="text-xs leading-relaxed text-slate-400">
              打开公民 App → 交易 → 扫一扫，扫描下方二维码并确认，再点「扫描签名结果」。
            </p>
            <div className="mt-4 flex justify-center">
              <div className="rounded-xl bg-white p-3">
                <QRCodeSVG value={signing.requestQr} size={220} level="M" />
              </div>
            </div>
            <button
              type="button"
              onClick={() => setScannerMode('signature')}
              disabled={loading}
              className="mt-5 w-full rounded-lg bg-gold-500 px-5 py-3 text-sm font-bold text-navy-950 transition-colors hover:bg-gold-400 disabled:cursor-not-allowed disabled:bg-slate-600 disabled:text-slate-300"
            >
              {loading ? '正在提交...' : '扫描签名结果'}
            </button>
          </div>
        </div>
      )}

      {scannerMode && (
        <QrScannerModal
          onResult={handleScanResult}
          onClose={handleScannerClose}
          title={scannerMode === 'signature' ? '扫描签名结果' : '扫码识别钱包地址'}
          hint={
            scannerMode === 'signature'
              ? '将 CitizenApp 生成的签名结果二维码对准取景框'
              : '将 CitizenApp 钱包地址二维码对准取景框，识别后自动填入'
          }
        />
      )}

      {/* 提示以浮层呈现，不占卡片高度（否则会随内容撑高卡片、切换时跳动）。 */}
      {message && (
        <div className="fixed inset-x-0 bottom-6 z-40 mx-auto max-w-sm px-6">
          <div className={`rounded-lg border px-4 py-3 text-sm shadow-lg ${messageClass}`}>
            {message.text}
          </div>
        </div>
      )}
    </>
  )
}
