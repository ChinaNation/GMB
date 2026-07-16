import { useCallback, useMemo, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'
import QRScannerModal from '../components/QRScannerModal'
import { buildSquareActionSignRequest, parseSignResponseSignature } from '../lib/qr-v1'

// 会员三档（ADR-036，与身份彻底解耦）：任意身份可订任意档，全组合放行。
type MembershipLevel = 'freedom' | 'democracy' | 'spark'
type TabKey = 'subscribe' | 'cancel'
type Tone = 'error' | 'info' | 'success'
// 支付方式：银行卡（Stripe 自动续订）或 USDC（预付固定时长、无自动续）。
type PaymentMethod = 'card' | 'usdc'
// USDC 两类操作：购买/续费（选季/年）或换档（补钱/补时长，不选时长）。
type UsdcMode = 'purchase' | 'change'
type PrepaidDuration = 'quarter' | 'year'

interface Plan {
  level: MembershipLevel
  name: string
  price: string
  /// 月费（分），供 USDC 季/年金额（月数×月费，无折扣）计算。
  priceCents: number
  /// 会员权益之一 = 单个聊天文件大小上限（ADR-036）。
  chatFile: string
  dynamic: string
  article: string
}

interface TierChangePreview {
  kind: 'upgrade' | 'downgrade' | 'switch'
  amountCents: number
}

/// USDC 换档预览：升档带补差价（分），降/平档带折算后剩余天数。
interface PrepaidChangePreview {
  kind: 'upgrade' | 'downgrade' | 'switch'
  amountCents?: number
  newDays?: number
}

/// 签名弹窗承载的操作类型，决定验签后走哪条提交路径。
type SigningKind = 'card-subscribe' | 'usdc-purchase' | 'usdc-change' | 'cancel'

interface Signing {
  kind: SigningKind
  challengeId: string
  requestQr: string
  level?: MembershipLevel
  duration?: PrepaidDuration
  preview?: TierChangePreview | null
  changePreview?: PrepaidChangePreview | null
}

const plans: Plan[] = [
  {
    level: 'freedom',
    name: '自由会员',
    price: '$2.99 / 月',
    priceCents: 299,
    chatFile: '聊天文件：单个 ≤ 10MB',
    dynamic: '动态：300 字、9 张标清图片、1 分钟标清视频',
    article: '文章：20,000 字、50 张标清图片、1 张高清首图',
  },
  {
    level: 'democracy',
    name: '民主会员',
    price: '$9.99 / 月',
    priceCents: 999,
    chatFile: '聊天文件：单个 ≤ 100MB',
    dynamic: '动态：300 字、9 张高清图片、30 分钟高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
  {
    level: 'spark',
    name: '星火会员',
    price: '$99.99 / 月',
    priceCents: 9999,
    chatFile: '聊天文件：单个 ≤ 5GB',
    dynamic: '动态：300 字、9 张高清图片、3 小时高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
]

// 会员三档固定顺序（价格升序，ADR-036 与身份彻底解耦）：自由 / 民主 / 星火。
const tierOrder: MembershipLevel[] = ['freedom', 'democracy', 'spark']

// 与 CitizenApp 会员卡统一：档色 / 顶带前景色。
const tierColor: Record<MembershipLevel, string> = {
  freedom: '#E5A100',
  democracy: '#3B82F6',
  spark: '#EF4444',
}
// 自由金底用深棕保对比度，民主蓝/星火红底用白字。
const tierOnColor: Record<MembershipLevel, string> = {
  freedom: '#4A3000',
  democracy: '#FFFFFF',
  spark: '#FFFFFF',
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

/** 携带 Worker error_code 的请求错误，供上层按 code 分支（如挑战过期回退）。 */
class ApiError extends Error {
  readonly code: string

  constructor(code: string, message: string) {
    super(message)
    this.code = code
  }
}

async function postJson(path: string, body: unknown): Promise<Record<string, unknown>> {
  const response = await fetch(`${apiBaseUrl}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  const data = (await response.json().catch(() => ({}))) as Record<string, unknown>
  if (!response.ok) {
    const code = typeof data.error_code === 'string' ? data.error_code : 'error'
    throw new ApiError(code, typeof data.message === 'string' ? data.message : '请求失败')
  }
  return data
}

/// 解析挑战响应里的换档金额预览（Worker 本地估算 { kind, amount_cents }）。
function parseSubscribePreview(raw: unknown): TierChangePreview | null {
  if (typeof raw !== 'object' || raw === null) return null
  const record = raw as Record<string, unknown>
  const kind = record.kind
  const amount = record.amount_cents
  if (
    (kind === 'upgrade' || kind === 'downgrade' || kind === 'switch') &&
    typeof amount === 'number'
  ) {
    return { kind, amountCents: amount }
  }
  return null
}

/// 签名前展示的换档金额文案（估算，实际以 Stripe proration 为准）。
function previewText(preview: TierChangePreview): string | null {
  const dollars = `$${(preview.amountCents / 100).toFixed(2)}`
  if (preview.kind === 'upgrade') return `升档：需按当期剩余天数补差价约 ${dollars}`
  if (preview.kind === 'downgrade') return `降档：约 ${dollars} 剩余价值将转为会员权益抵扣后续账单`
  return null
}

/// 换档即时生效时的成功文案（升档已扣 / 降档转权益 / 续订 / 无操作）。
function subscribeResultMessage(action: unknown): string {
  switch (action) {
    case 'upgraded':
      return '已升档，差价已按剩余天数结算'
    case 'downgraded':
      return '已降档，剩余价值转为会员权益抵扣后续账单'
    case 'resumed':
      return '已续订，订阅恢复'
    case 'already_subscribed':
      return '你已是该会员'
    default:
      return '订阅已更新'
  }
}

const PREPAID_MONTHS: Record<PrepaidDuration, number> = { quarter: 3, year: 12 }

/// 分转美元展示。
function formatUsd(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`
}

/// USDC 预付总额（分）= 月数 × 月费（无折扣，与 Worker 一致）。
function prepaidTotalCents(plan: Plan, duration: PrepaidDuration): number {
  return plan.priceCents * PREPAID_MONTHS[duration]
}

/// 解析 USDC 换档预览（Worker `{ kind, amount_cents?, new_days? }`）。
function parsePrepaidChangePreview(raw: unknown): PrepaidChangePreview | null {
  if (typeof raw !== 'object' || raw === null) return null
  const record = raw as Record<string, unknown>
  const kind = record.kind
  if (kind !== 'upgrade' && kind !== 'downgrade' && kind !== 'switch') return null
  return {
    kind,
    amountCents: typeof record.amount_cents === 'number' ? record.amount_cents : undefined,
    newDays: typeof record.new_days === 'number' ? record.new_days : undefined,
  }
}

/// USDC 换档签名前文案：升档补差价、降/平档折算剩余天数。
function prepaidChangePreviewText(preview: PrepaidChangePreview, targetName: string): string | null {
  if (preview.kind === 'upgrade' && typeof preview.amountCents === 'number') {
    return `升档到${targetName}：需补差价约 ${formatUsd(preview.amountCents)}`
  }
  if (preview.kind === 'downgrade' && typeof preview.newDays === 'number') {
    return `降档到${targetName}：剩余价值折算为约 ${preview.newDays} 天`
  }
  if (preview.kind === 'switch' && typeof preview.newDays === 'number') {
    return `平价换档到${targetName}：保留剩余约 ${preview.newDays} 天`
  }
  return null
}

/// USDC 换档即时生效文案（降/平档本地切档、升档待付差价另跳转）。
function prepaidChangeResultMessage(action: unknown): string {
  switch (action) {
    case 'downgraded':
      return '已降档，剩余价值已折算为更长会员时长'
    case 'switched':
      return '已平价换档，剩余时长保留'
    case 'upgraded':
      return '已升档'
    default:
      return '换档已完成'
  }
}

/// 取消订阅结果文案：卡（连续订阅）到期取消 / USDC 预付到期自然失效。
function cancelResultMessage(cancelKind: unknown): string {
  return cancelKind === 'usdc_prepaid'
    ? 'USDC 预付无自动续费，将于到期日自然失效，无需取消'
    : '已提交取消订阅，当前计费周期结束后生效'
}

/// 各操作对应的挑战 / 确认接口路径。
const CHALLENGE_PATH: Record<SigningKind, string> = {
  'card-subscribe': '/v1/square/membership/subscribe/challenge',
  'usdc-purchase': '/v1/square/membership/prepaid/challenge',
  'usdc-change': '/v1/square/membership/prepaid/change/challenge',
  cancel: '/v1/square/membership/cancel/challenge',
}
const CONFIRM_PATH: Record<SigningKind, string> = {
  'card-subscribe': '/v1/square/membership/subscribe',
  'usdc-purchase': '/v1/square/membership/prepaid',
  'usdc-change': '/v1/square/membership/prepaid/change',
  cancel: '/v1/square/membership/cancel',
}

function challengePath(kind: SigningKind): string {
  return CHALLENGE_PATH[kind]
}

/// 签名弹窗与按钮标题（按操作类型）。
function signingTitle(kind: SigningKind): string {
  switch (kind) {
    case 'usdc-purchase':
      return '扫码签名并购买'
    case 'usdc-change':
      return '扫码签名并换档'
    case 'cancel':
      return '扫码签名并取消'
    default:
      return '扫码签名并订阅'
  }
}

/// 档位对应的会员名（换档预览文案用）。
function levelName(level: MembershipLevel | undefined): string {
  return plans.find((plan) => plan.level === level)?.name ?? '目标档'
}

/// 挑战请求体：卡订阅/换档带档位，USDC 购买另带时长，取消只带钱包。
function challengeBody(
  kind: SigningKind,
  owner: string,
  level: MembershipLevel | null,
  duration: PrepaidDuration,
): Record<string, unknown> {
  if (kind === 'cancel') return { owner_account: owner }
  if (kind === 'usdc-purchase') {
    return { owner_account: owner, membership_level: level, duration }
  }
  return { owner_account: owner, membership_level: level }
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
  // 支付方式 / USDC 操作 / 预付时长（仅订阅态用）。
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>('card')
  const [usdcMode, setUsdcMode] = useState<UsdcMode>('purchase')
  const [prepaidDuration, setPrepaidDuration] = useState<PrepaidDuration>('quarter')

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

  // 发起签名：按 tab + 支付方式取对应挑战 → 构建 signRequest 二维码等 CitizenApp 扫码签名。
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
      // 订阅态按 支付方式 + USDC 操作 决定路径；取消态统一走 cancel。
      const kind: SigningKind =
        action === 'cancel'
          ? 'cancel'
          : paymentMethod === 'card'
            ? 'card-subscribe'
            : usdcMode === 'change'
              ? 'usdc-change'
              : 'usdc-purchase'
      setLoading(true)
      setMessage(null)
      setSigning(null)
      try {
        const data = await postJson(challengePath(kind), challengeBody(kind, owner, level, prepaidDuration))
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
          kind,
          challengeId,
          requestQr,
          level: action === 'subscribe' ? level ?? undefined : undefined,
          duration: kind === 'usdc-purchase' ? prepaidDuration : undefined,
          preview: kind === 'card-subscribe' ? parseSubscribePreview(data.preview) : null,
          changePreview: kind === 'usdc-change' ? parsePrepaidChangePreview(data.preview) : null,
        })
      } catch (error) {
        setMessage({ tone: 'error', text: error instanceof Error ? error.message : '发起签名失败' })
      } finally {
        setLoading(false)
      }
    },
    [ownerAccount, selectedLevel, paymentMethod, usdcMode, prepaidDuration],
  )

  // 扫回 App 的 signResponse → 提交 Worker 验签 → 按操作类型跳付款 / 提示结果。
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
        const body: Record<string, unknown> = {
          owner_account: owner,
          challenge_id: active.challengeId,
          signature,
        }
        if (active.kind !== 'cancel') body.membership_level = active.level
        if (active.kind === 'usdc-purchase') body.duration = active.duration
        const data = await postJson(CONFIRM_PATH[active.kind], body)

        // 需付款则跳转：卡全新订阅 checkout_url / 卡升档待付 payment_url /
        // USDC 购买 checkout_url / USDC 升档待付 checkout_url。
        const redirectUrl =
          typeof data.checkout_url === 'string'
            ? data.checkout_url
            : data.action === 'upgrade_pending' && typeof data.payment_url === 'string'
              ? data.payment_url
              : null
        if (redirectUrl) {
          window.location.assign(redirectUrl)
          return
        }

        setSigning(null)
        // 待付差价却没拿到付款链接：报错而非误报成功，避免用户以为升档完成却从未去付账单。
        if (data.action === 'upgrade_pending') {
          setMessage({ tone: 'error', text: '升档需支付差价，但未获取到付款链接，请重新发起' })
          return
        }
        // 无需跳转：即时生效 / 信息提示。按操作类型出对应文案。
        if (active.kind === 'cancel') {
          setMessage({ tone: 'success', text: cancelResultMessage(data.cancel_kind) })
        } else if (active.kind === 'usdc-change') {
          setMessage({ tone: 'success', text: prepaidChangeResultMessage(data.action) })
        } else {
          setMessage({ tone: 'success', text: subscribeResultMessage(data.action) })
        }
        return
      } catch (error) {
        // 挑战过期 / 已用 / 不存在：关掉扫码弹层回到发起态，提示重新发起，
        // 而不是停在「等扫码」界面让用户反复扫一张已作废的二维码。
        if (
          error instanceof ApiError &&
          ['expired_challenge', 'used_challenge', 'invalid_challenge'].includes(error.code)
        ) {
          setSigning(null)
          setMessage({ tone: 'error', text: '签名挑战已过期或已使用，请重新发起' })
        } else {
          setMessage({ tone: 'error', text: error instanceof Error ? error.message : '提交失败' })
        }
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

  // 「当前选择」价格行：卡按月费；USDC 购买按所选时长总额；USDC 换档按剩余折算。
  const priceLine =
    paymentMethod === 'card'
      ? selectedPlan?.price ?? ''
      : usdcMode === 'change'
        ? 'USDC 换档 · 按当前剩余时长折算'
        : selectedPlan
          ? `USDC ${prepaidDuration === 'year' ? '年付' : '季付'} · ${formatUsd(prepaidTotalCents(selectedPlan, prepaidDuration))}`
          : ''

  const subscribeButtonLabel =
    paymentMethod === 'card'
      ? '扫码签名并订阅'
      : usdcMode === 'change'
        ? '扫码签名并换档'
        : '扫码签名并购买'

  return (
    <>
      <section className="border-b border-white/10 bg-navy-900/35 py-10 md:py-12">
        <div className="mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="CitizenApp Membership"
            title="CitizenApp 会员订阅"
            description="三档会员与身份彻底解耦，任意身份都可订阅任意档，统一按美元计价；订阅、取消与续订都由 CitizenApp 钱包扫码签名授权；Stripe Checkout 负责银行卡、本地法币和已启用的 USDC 支付。"
          />
        </div>
      </section>

      <section className="mx-auto grid max-w-7xl auto-rows-fr gap-6 px-6 py-20 sm:grid-cols-2 lg:grid-cols-4">
        {tierOrder.map((level) => {
          const plan = plans.find((item) => item.level === level)!
          const isSelected = selectedLevel === level
          const color = tierColor[level]
          const onColor = tierOnColor[level]
          return (
            <div
              key={level}
              className="flex h-full flex-col overflow-hidden rounded-2xl bg-white"
              style={{
                border: isSelected ? `2px solid ${color}` : '1px solid rgba(255,255,255,0.12)',
                boxShadow: isSelected
                  ? `0 10px 30px ${color}55`
                  : '0 8px 24px rgba(0,0,0,0.28)',
              }}
            >
              {/* 档色顶带：档名（无身份字段，会员与身份已解耦） */}
              <div className="relative px-5 pb-4 pt-4" style={{ background: color }}>
                <div className="text-[11px] font-medium" style={{ color: onColor, opacity: 0.85 }}>
                  会员订阅
                </div>
                <div className="mt-1 text-lg font-bold" style={{ color: onColor }}>
                  {plan.name}
                </div>
              </div>

              {/* 卡身：会员权益 + 价签 + 订阅 */}
              <div className="flex flex-1 flex-col p-5">
                <SectionLabel color={color} text="会员权益" />
                <div className="mt-2.5 space-y-1.5 text-[12px] leading-relaxed text-slate-600">
                  <p>{plan.chatFile}</p>
                  <p>{plan.dynamic}</p>
                  <p>{plan.article}</p>
                </div>

                <div className="flex-1" />

                {/* 价格：档色填充标签 */}
                <div className="mt-3">
                  <span
                    className="inline-block rounded-lg px-3 py-1 text-base font-extrabold"
                    style={{ background: color, color: onColor }}
                  >
                    {plan.price}
                  </span>
                </div>

                {/* 订阅：选中该档并切到订阅态（下方面板扫码完成） */}
                <button
                  type="button"
                  onClick={() => handleSelectLevel(level)}
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
                <p className="mt-2 text-sm text-slate-400">{priceLine}</p>
              </>
            ) : (
              <p className="text-sm leading-relaxed text-slate-300">
                取消订阅将在当前计费周期结束后生效；USDC 预付则到期自然失效（无自动续费）。
              </p>
            )}
          </div>

          {/* 支付方式 / USDC 操作 / 预付时长：仅订阅态显示 */}
          {activeTab === 'subscribe' && selectedPlan && (
            <div className="mt-5 space-y-4">
              <div>
                <div className="mb-2 text-xs font-semibold text-slate-400">支付方式</div>
                <div className="flex rounded-lg border border-white/10 bg-navy-950 p-1 text-sm font-semibold">
                  {(
                    [
                      ['card', '银行卡 · 自动续订'],
                      ['usdc', 'USDC · 预付'],
                    ] as const
                  ).map(([method, label]) => (
                    <button
                      key={method}
                      type="button"
                      onClick={() => setPaymentMethod(method)}
                      className={`flex-1 rounded-md py-2 transition-colors ${
                        paymentMethod === method
                          ? 'bg-gold-500 text-navy-950'
                          : 'text-slate-300 hover:text-white'
                      }`}
                    >
                      {label}
                    </button>
                  ))}
                </div>
              </div>

              {paymentMethod === 'usdc' && (
                <div>
                  <div className="mb-2 text-xs font-semibold text-slate-400">USDC 操作</div>
                  <div className="flex rounded-lg border border-white/10 bg-navy-950 p-1 text-sm font-semibold">
                    {(
                      [
                        ['purchase', '购买 / 续费'],
                        ['change', '换档'],
                      ] as const
                    ).map(([mode, label]) => (
                      <button
                        key={mode}
                        type="button"
                        onClick={() => setUsdcMode(mode)}
                        className={`flex-1 rounded-md py-2 transition-colors ${
                          usdcMode === mode
                            ? 'bg-gold-500 text-navy-950'
                            : 'text-slate-300 hover:text-white'
                        }`}
                      >
                        {label}
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {paymentMethod === 'usdc' && usdcMode === 'purchase' && (
                <div>
                  <div className="mb-2 text-xs font-semibold text-slate-400">预付时长（无折扣）</div>
                  <div className="flex gap-2">
                    {(['quarter', 'year'] as const).map((duration) => {
                      const label = duration === 'year' ? '年付 · 12 个月' : '季付 · 3 个月'
                      const total = prepaidTotalCents(selectedPlan, duration)
                      const active = prepaidDuration === duration
                      return (
                        <button
                          key={duration}
                          type="button"
                          onClick={() => setPrepaidDuration(duration)}
                          className={`flex-1 rounded-lg border px-3 py-2.5 text-left transition-colors ${
                            active
                              ? 'border-gold-400 bg-gold-500/10'
                              : 'border-white/10 bg-navy-950 hover:border-white/20'
                          }`}
                        >
                          <div className="text-[13px] font-bold text-white">{label}</div>
                          <div className="mt-0.5 text-sm font-extrabold text-gold-300">
                            {formatUsd(total)}
                          </div>
                        </button>
                      )
                    })}
                  </div>
                </div>
              )}

              {paymentMethod === 'usdc' && usdcMode === 'change' && (
                <p className="text-xs leading-relaxed text-slate-400">
                  换档基于当前 USDC 会员剩余时长折算：升档补差价、降档补时长，金额在扫码前显示；仅对已有有效 USDC 会员生效。
                </p>
              )}
            </div>
          )}

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
                ? subscribeButtonLabel
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
              <h3 className="text-lg font-semibold text-white">{signingTitle(signing.kind)}</h3>
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
            {(() => {
              const text = signing.preview
                ? previewText(signing.preview)
                : signing.changePreview
                  ? prepaidChangePreviewText(signing.changePreview, levelName(signing.level))
                  : null
              return text ? (
                <div className="mt-3 rounded-lg border border-gold-400/30 bg-gold-500/10 px-3 py-2 text-xs leading-relaxed text-gold-200">
                  {text}
                </div>
              ) : null
            })()}
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
