import { useCallback, useMemo, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'
import QRScannerModal from '../components/QRScannerModal'
import { buildSquareActionSignRequest, parseSignResponseSignature } from '../lib/qr-v1'

type MembershipLevel = 'freedom' | 'democracy' | 'voting' | 'candidate'
// 链上身份档（与会员档解耦）：访客身份含自由/民主两档会员。
type IdentityLevel = 'visitor' | 'voting' | 'candidate'
type TabKey = 'subscribe' | 'cancel'
type Tone = 'error' | 'info' | 'success'

interface Plan {
  level: MembershipLevel
  requiredIdentity: IdentityLevel
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
    level: 'freedom',
    requiredIdentity: 'visitor',
    name: '自由会员',
    price: '$2.99 / 月',
    identity: '任意钱包账户',
    dynamic: '动态：300 字、9 张标清图片、1 分钟标清视频',
    article: '文章：20,000 字、50 张标清图片、1 张高清首图',
  },
  {
    // 民主会员：访客身份的 $9.99 高权益档，权益对齐投票公民会员，唯身份匿名。
    level: 'democracy',
    requiredIdentity: 'visitor',
    name: '民主会员',
    price: '$9.99 / 月',
    identity: '任意钱包账户',
    dynamic: '动态：300 字、9 张高清图片、30 分钟高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
  {
    level: 'voting',
    requiredIdentity: 'voting',
    name: '投票公民会员',
    price: '$9.99 / 月',
    identity: '认证投票公民',
    dynamic: '动态：300 字、9 张高清图片、30 分钟高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
  {
    level: 'candidate',
    requiredIdentity: 'candidate',
    name: '竞选公民会员',
    price: '$99.99 / 月',
    identity: '认证选举公民',
    dynamic: '动态：300 字、9 张高清图片、3 小时高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
]

// 3 张身份卡顺序：访客 / 投票公民 / 竞选公民。
const identityTierOrder: IdentityLevel[] = ['visitor', 'voting', 'candidate']

// 与CitizenApp 身份卡统一：档色 / 顶带前景色 / 档名 / 身份名 / 链上公开身份字段。
const tierColor: Record<IdentityLevel, string> = {
  visitor: '#E5A100',
  voting: '#3B82F6',
  candidate: '#EF4444',
}
// 访客金底用深棕保对比度，投票蓝/竞选红底用白字。
const tierOnColor: Record<IdentityLevel, string> = {
  visitor: '#4A3000',
  voting: '#FFFFFF',
  candidate: '#FFFFFF',
}
const tierCardName: Record<IdentityLevel, string> = {
  visitor: '访客轻节点',
  voting: '公民轻节点 · 投票',
  candidate: '公民轻节点 · 竞选',
}
const tierIdentityName: Record<IdentityLevel, string> = {
  visitor: '访客',
  voting: '投票公民',
  candidate: '竞选公民',
}
// 该档在链上公开的身份字段（通用模板，非某用户真实值）。访客单独走「完全匿名」。
const tierIdentityFields: Record<IdentityLevel, string[]> = {
  visitor: [],
  voting: ['公民身份 CID 号', '居住选区', '投票身份有效期'],
  candidate: ['公民身份 CID 号', '居住选区', '身份有效期', '真实姓名', '性别', '出生地'],
}

// 仓库扇贝勋章徽章（与 App identity_badge 一致）：档色底 + 中心白色小人（官网无
// 登录用户，恒显小人），套半透明白圆底才能从同色顶带浮出。
function IdentityBadge({ color, size = 40 }: { color: string; size?: number }) {
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: '50%',
        background: 'rgba(255,255,255,0.9)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        boxShadow: '0 2px 6px rgba(0,0,0,0.14)',
      }}
    >
      <svg width={Math.round(size * 0.78)} height={Math.round(size * 0.78)} viewBox="0 0 24 24" aria-hidden="true">
        <g fill={color}>
          <circle cx="18" cy="12" r="4.3" />
          <circle cx="16.24" cy="16.24" r="4.3" />
          <circle cx="12" cy="18" r="4.3" />
          <circle cx="7.76" cy="16.24" r="4.3" />
          <circle cx="6" cy="12" r="4.3" />
          <circle cx="7.76" cy="7.76" r="4.3" />
          <circle cx="12" cy="6" r="4.3" />
          <circle cx="16.24" cy="7.76" r="4.3" />
          <circle cx="12" cy="12" r="7.6" />
        </g>
        <circle cx="12" cy="9.7" r="2.3" fill="#fff" />
        <path d="M7.7 16.4 C7.7 14 9.6 12.7 12 12.7 C14.4 12.7 16.3 14 16.3 16.4 Z" fill="#fff" />
      </svg>
    </div>
  )
}

function SectionLabel({ color, text }: { color: string; text: string }) {
  return (
    <div
      className="flex items-center gap-1.5 text-[11px] font-bold tracking-wide"
      style={{ color }}
    >
      <span style={{ width: 6, height: 6, borderRadius: '50%', background: color, display: 'inline-block' }} />
      {text}
    </div>
  )
}

const apiBaseUrl =
  import.meta.env.VITE_API_URL?.replace(/\/+$/, '') ??
  '/api'

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
  // null=未选中任何会员卡（取消订阅态）；订阅态默认自由会员。
  const [selectedLevel, setSelectedLevel] = useState<MembershipLevel | null>('freedom')
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState<{ tone: Tone; text: string } | null>(null)
  const [signing, setSigning] = useState<Signing | null>(null)
  const [scannerMode, setScannerMode] = useState<'address' | 'signature' | null>(null)

  const selectedPlan = useMemo(
    () => (selectedLevel ? plans.find((plan) => plan.level === selectedLevel) ?? null : null),
    [selectedLevel],
  )

  // 选中会员卡片=进入订阅意图：高亮该卡并切到「订阅会员」。
  const handleSelectLevel = useCallback((level: MembershipLevel) => {
    setSelectedLevel(level)
    setActiveTab('subscribe')
    setSigning(null)
    setMessage(null)
  }, [])

  const switchTab = useCallback((tab: TabKey) => {
    setActiveTab(tab)
    setSigning(null)
    setMessage(null)
    // 取消订阅无需选中会员档：切到「取消订阅」即释放所有卡片选中；切回订阅默认自由。
    setSelectedLevel(tab === 'cancel' ? null : (prev) => prev ?? 'freedom')
  }, [])

  // 发起签名：取挑战 → 构建 signRequest 二维码等 CitizenApp 扫码签名。
  const beginSigning = useCallback(
    async (action: TabKey) => {
      const owner = ownerAccount.trim()
      if (!owner) {
        setMessage({ tone: 'error', text: '请输入钱包账户地址' })
        return
      }
      const level = selectedLevel
      if (action === 'subscribe' && !level) {
        setMessage({ tone: 'error', text: '请先选择会员档位' })
        return
      }
      setLoading(true)
      setMessage(null)
      setSigning(null)
      try {
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
          level: action === 'subscribe' ? level ?? undefined : undefined,
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
            title="CitizenApp 会员订阅"
            description="四档会员统一按美元计价，订阅、取消与续订都由 CitizenApp 钱包扫码签名授权；Stripe Checkout 负责银行卡、本地法币和已启用的 USDC 支付。"
          />
        </div>
      </section>

      <section className="mx-auto grid max-w-7xl auto-rows-fr gap-6 px-6 py-20 sm:grid-cols-2 lg:grid-cols-4">
        {identityTierOrder.map((tier) => {
          const tierPlans = plans.filter((plan) => plan.requiredIdentity === tier)
          const shown = tierPlans.find((plan) => plan.level === selectedLevel) ?? tierPlans[0]
          const isTierSelected =
            selectedLevel !== null && tierPlans.some((plan) => plan.level === selectedLevel)
          const hasToggle = tierPlans.length > 1
          const color = tierColor[tier]
          const onColor = tierOnColor[tier]
          return (
            <div
              key={tier}
              className="flex h-full flex-col overflow-hidden rounded-2xl bg-white"
              style={{
                border: isTierSelected ? `2px solid ${color}` : '1px solid rgba(255,255,255,0.12)',
                boxShadow: isTierSelected
                  ? `0 10px 30px ${color}55`
                  : '0 8px 24px rgba(0,0,0,0.28)',
              }}
            >
              {/* 档色顶带：身份名在顶 + 档名 + 右上角扇贝徽章 */}
              <div className="relative px-5 pb-4 pt-4" style={{ background: color }}>
                <div className="text-[11px] font-medium" style={{ color: onColor, opacity: 0.85 }}>
                  身份 · {tierIdentityName[tier]}
                </div>
                <div className="mt-1 text-lg font-bold" style={{ color: onColor }}>
                  {tierCardName[tier]}
                </div>
                <div className="absolute right-4 top-3.5">
                  <IdentityBadge color={color} />
                </div>
              </div>

              {/* 卡身：链上公开身份信息 + 会员权益 + 价签 + 切换 + 订阅 */}
              <div className="flex flex-1 flex-col p-5">
                <SectionLabel color={color} text="链上公开的身份信息" />
                <div className="mt-2.5">
                  {tier === 'visitor' ? (
                    <div className="flex items-center gap-2.5">
                      <svg
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="#94a3b8"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      >
                        <circle cx="10" cy="8" r="4" />
                        <path d="M4 21v-1a6 6 0 0 1 6-6h1" />
                        <path d="m16 16 5 5M21 16l-5 5" />
                      </svg>
                      <div>
                        <div className="text-[15px] font-bold text-slate-900">完全匿名</div>
                        <div className="text-[11px] text-slate-400">没有链上身份</div>
                      </div>
                    </div>
                  ) : (
                    <div className="space-y-1.5">
                      {tierIdentityFields[tier].map((field) => (
                        <div
                          key={field}
                          className="flex items-center gap-2 text-[13px] font-medium text-slate-800"
                        >
                          <span
                            style={{
                              width: 5,
                              height: 5,
                              borderRadius: '50%',
                              background: color,
                              display: 'inline-block',
                            }}
                          />
                          {field}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                <div className="mt-4">
                  <SectionLabel color={color} text="会员权益" />
                  <div className="mt-2.5 space-y-1.5 text-[12px] leading-relaxed text-slate-600">
                    <p>{shown.dynamic}</p>
                    <p>{shown.article}</p>
                  </div>
                </div>

                <div className="flex-1" />

                {/* 访客档：自由/民主分段切换（默认自由） */}
                {hasToggle && (
                  <div className="mt-4 flex rounded-lg bg-slate-100 p-1 text-[13px] font-bold">
                    {tierPlans.map((plan) => {
                      const active = selectedLevel === plan.level
                      return (
                        <button
                          key={plan.level}
                          type="button"
                          onClick={() => handleSelectLevel(plan.level)}
                          className="flex-1 rounded-md py-1.5 transition-colors"
                          style={active ? { background: color, color: onColor } : { color: '#64748b' }}
                        >
                          {plan.name}
                        </button>
                      )
                    })}
                  </div>
                )}

                {/* 价格：档色填充标签 */}
                <div className="mt-3">
                  <span
                    className="inline-block rounded-lg px-3 py-1 text-base font-extrabold"
                    style={{ background: color, color: onColor }}
                  >
                    {shown.price}
                  </span>
                </div>

                {/* 订阅：选中该档并切到订阅态（下方面板扫码完成） */}
                <button
                  type="button"
                  onClick={() => handleSelectLevel(shown.level)}
                  className="mt-3 w-full rounded-lg py-2.5 text-sm font-bold transition-opacity hover:opacity-90"
                  style={{ background: color, color: onColor }}
                >
                  订阅
                </button>
              </div>
            </div>
          )
        })}

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
            {activeTab === 'subscribe' && selectedPlan ? (
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
              placeholder="输入公民钱包地址"
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
              打开CitizenApp → 交易 → 扫一扫，扫描下方二维码并确认，再点「扫描签名结果」。
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
        <QRScannerModal
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
