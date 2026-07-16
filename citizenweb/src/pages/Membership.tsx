import { useCallback, useMemo, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'
import QRScannerModal from '../components/QRScannerModal'
import { buildSquareActionSignRequest, parseSignResponseSignature } from '../lib/qr-v1'

type MembershipLevel = 'freedom' | 'democracy' | 'spark'
type Tone = 'error' | 'info' | 'success'
type PrepaidDuration = 'quarter' | 'year'

// 卡上操作：银行卡订阅 / 加密货币预付 / 取消订阅。加密货币预付遇异档自动转换档。
type ActionKind = 'card-subscribe' | 'crypto-prepaid' | 'cancel'

interface Plan {
  level: MembershipLevel
  name: string
  price: string
  /// 月费（分），供加密货币季/年金额（月数×月费，无折扣）计算。
  priceCents: number
  /// 会员权益之一 = 单个聊天文件大小上限。
  chatFile: string
  dynamic: string
  article: string
}

interface TierChangePreview {
  kind: 'upgrade' | 'downgrade' | 'switch'
  amountCents: number
}

/// 加密货币换档预览：升档带补差价（分），降/平档带折算后剩余天数。
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
    name: '薪火会员',
    price: '$99.99 / 月',
    priceCents: 9999,
    chatFile: '聊天文件：单个 ≤ 5GB',
    dynamic: '动态：300 字、9 张高清图片、3 小时高清视频',
    article: '文章：30,000 字、100 张高清图片、1 张高清首图',
  },
]

// 会员三档固定顺序（价格升序）：自由 / 民主 / 薪火。
const tierOrder: MembershipLevel[] = ['freedom', 'democracy', 'spark']

// 与 CitizenApp 会员卡统一：档色 / 顶带前景色 / 描边按钮深色文字。
const tierColor: Record<MembershipLevel, string> = {
  freedom: '#E5A100',
  democracy: '#3B82F6',
  spark: '#EF4444',
}
// 自由金底用深棕保对比度，民主蓝/薪火红底用白字。
const tierOnColor: Record<MembershipLevel, string> = {
  freedom: '#4A3000',
  democracy: '#FFFFFF',
  spark: '#FFFFFF',
}
// 描边按钮（加密货币预付）白底上的深色文字，保证对比度。
const tierDeep: Record<MembershipLevel, string> = {
  freedom: '#8A6200',
  democracy: '#185FA5',
  spark: '#A32D2D',
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

/// 加密货币预付总额（分）= 月数 × 月费（无折扣，与 Worker 一致）。
function prepaidTotalCents(plan: Plan, duration: PrepaidDuration): number {
  return plan.priceCents * PREPAID_MONTHS[duration]
}

/// 解析加密货币换档预览（Worker `{ kind, amount_cents?, new_days? }`）。
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

/// 加密货币换档签名前文案：升档补差价、降/平档折算剩余天数。
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

/// 加密货币换档即时生效文案（降/平档本地切档、升档待付差价另跳转）。
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

/// 取消订阅结果文案：卡（连续订阅）到期取消 / 加密货币预付到期自然失效。
function cancelResultMessage(cancelKind: unknown): string {
  return cancelKind === 'usdc_prepaid'
    ? '加密货币预付无自动续费，将于到期日自然失效，无需取消'
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

/// 档位对应的会员名（换档预览文案用）。
function levelName(level: MembershipLevel | undefined): string {
  return plans.find((plan) => plan.level === level)?.name ?? '目标档'
}

/// 弹窗标题（按卡上操作类型 + 档名）。
function actionModalTitle(pending: { kind: ActionKind }, planName?: string): string {
  if (pending.kind === 'cancel') return '取消订阅'
  const prefix = pending.kind === 'card-subscribe' ? '银行卡订阅' : '加密货币预付'
  return planName ? `${prefix} · ${planName}` : prefix
}

/// 弹窗内触发按钮文案。
function actionButtonLabel(kind: ActionKind): string {
  if (kind === 'cancel') return '扫码签名并取消'
  return kind === 'card-subscribe' ? '扫码签名并订阅' : '扫码签名并购买'
}

/// 挑战请求体：卡订阅/换档带档位，加密货币购买另带时长，取消只带钱包。
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
  const [ownerAccount, setOwnerAccount] = useState('')
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState<{ tone: Tone; text: string } | null>(null)
  // 当前卡上发起的操作（银行卡订阅 / 加密货币预付 / 取消）；null=无弹窗。
  const [pending, setPending] = useState<{ kind: ActionKind; level: MembershipLevel | null } | null>(null)
  const [signing, setSigning] = useState<Signing | null>(null)
  const [scannerMode, setScannerMode] = useState<'address' | 'signature' | null>(null)
  const [prepaidDuration, setPrepaidDuration] = useState<PrepaidDuration>('quarter')

  const pendingPlan = useMemo(
    () => (pending?.level ? plans.find((plan) => plan.level === pending.level) ?? null : null),
    [pending],
  )

  // 点卡上按钮打开操作弹窗；关闭弹窗清空临时态。
  const openAction = useCallback((kind: ActionKind, level: MembershipLevel | null) => {
    setPending({ kind, level })
    setSigning(null)
    setMessage(null)
    setPrepaidDuration('quarter')
  }, [])

  const closeModal = useCallback(() => {
    setPending(null)
    setSigning(null)
  }, [])

  // 发起签名：银行卡订阅 / 加密货币预付 / 取消。加密货币预付遇「已有异档」自动改走换档。
  const beginSigning = useCallback(async () => {
    const active = pending
    if (!active) return
    const owner = ownerAccount.trim()
    if (!owner) {
      setMessage({ tone: 'error', text: '请输入钱包账户地址' })
      return
    }
    const level = active.level
    if (active.kind !== 'cancel' && !level) {
      setMessage({ tone: 'error', text: '请先选择会员档位' })
      return
    }
    const primaryKind: SigningKind =
      active.kind === 'cancel'
        ? 'cancel'
        : active.kind === 'card-subscribe'
          ? 'card-subscribe'
          : 'usdc-purchase'
    setLoading(true)
    setMessage(null)
    setSigning(null)
    try {
      let kind: SigningKind = primaryKind
      let data: Record<string, unknown>
      try {
        data = await postJson(challengePath(kind), challengeBody(kind, owner, level, prepaidDuration))
      } catch (error) {
        // 加密货币预付遇「已有其它档」→ 自动改走换档（补差价/折算），不让用户再选。
        if (
          error instanceof ApiError &&
          error.code === 'prepaid_tier_change_required' &&
          kind === 'usdc-purchase'
        ) {
          kind = 'usdc-change'
          data = await postJson(challengePath(kind), challengeBody(kind, owner, level, prepaidDuration))
        } else {
          throw error
        }
      }
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
        level: kind === 'cancel' ? undefined : level ?? undefined,
        duration: kind === 'usdc-purchase' ? prepaidDuration : undefined,
        preview: kind === 'card-subscribe' ? parseSubscribePreview(data.preview) : null,
        changePreview: kind === 'usdc-change' ? parsePrepaidChangePreview(data.preview) : null,
      })
    } catch (error) {
      setMessage({ tone: 'error', text: error instanceof Error ? error.message : '发起签名失败' })
    } finally {
      setLoading(false)
    }
  }, [pending, ownerAccount, prepaidDuration])

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
        // 加密货币购买 checkout_url / 加密货币升档待付 checkout_url。
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

        // 操作完成：关闭整个弹窗，仅保留底部浮层提示。
        setPending(null)
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
        // 挑战过期 / 已用 / 不存在：关掉扫码回到输入态，提示重新发起，
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

  return (
    <>
      <section className="border-b border-white/10 bg-navy-900/35 py-10 md:py-12">
        <div className="mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="CitizenApp Membership"
            title="CitizenApp 会员订阅"
            description="三档会员按月订阅，统一美元计价；订阅、取消与续订都由 CitizenApp 钱包扫码签名授权；Stripe Checkout 负责银行卡、本地法币和已启用的加密货币支付。"
          />
        </div>
      </section>

      <section className="mx-auto grid max-w-7xl auto-rows-fr gap-6 px-6 py-20 sm:grid-cols-2 lg:grid-cols-4">
        {tierOrder.map((level) => {
          const plan = plans.find((item) => item.level === level)!
          const color = tierColor[level]
          const onColor = tierOnColor[level]
          return (
            <div
              key={level}
              className="flex h-full min-h-[560px] flex-col overflow-hidden rounded-2xl bg-white"
              style={{
                border: '1px solid rgba(255,255,255,0.12)',
                boxShadow: '0 8px 24px rgba(0,0,0,0.28)',
              }}
            >
              {/* 档色顶带：档名 */}
              <div className="px-5 pb-4 pt-4" style={{ background: color }}>
                <div className="text-[11px] font-medium" style={{ color: onColor, opacity: 0.85 }}>
                  会员订阅
                </div>
                <div className="mt-1 text-lg font-bold" style={{ color: onColor }}>
                  {plan.name}
                </div>
              </div>

              {/* 卡身：会员权益 + 价签 + 两个操作按钮（点击弹窗完成） */}
              <div className="flex flex-1 flex-col p-5">
                <SectionLabel color={color} text="会员权益" />
                <div className="mt-2.5 space-y-1.5 text-[12px] leading-relaxed text-slate-600">
                  <p>{plan.chatFile}</p>
                  <p>{plan.dynamic}</p>
                  <p>{plan.article}</p>
                </div>

                <div className="flex-1" />

                <div className="mt-3">
                  <span
                    className="inline-block rounded-lg px-3 py-1 text-base font-extrabold"
                    style={{ background: color, color: onColor }}
                  >
                    {plan.price}
                  </span>
                </div>

                <button
                  type="button"
                  onClick={() => openAction('card-subscribe', level)}
                  className="mt-3 w-full rounded-lg py-2.5 text-sm font-bold transition-opacity hover:opacity-90"
                  style={{ background: color, color: onColor }}
                >
                  银行卡订阅
                </button>
                <button
                  type="button"
                  onClick={() => openAction('crypto-prepaid', level)}
                  className="mt-2 w-full rounded-lg border py-2.5 text-sm font-bold transition-colors hover:bg-slate-50"
                  style={{ borderColor: color, color: tierDeep[level] }}
                >
                  加密货币预付
                </button>
              </div>
            </div>
          )
        })}

        <GlowCard glow="gold" className="flex flex-col">
          <div className="text-sm font-medium text-gold-400">取消订阅</div>
          <p className="mt-3 text-sm leading-relaxed text-slate-300">
            取消订阅将在当前计费周期结束后生效；加密货币预付则到期自然失效（无自动续费）。
          </p>
          <div className="flex-1" />
          <button
            type="button"
            onClick={() => openAction('cancel', null)}
            className="mt-6 w-full rounded-lg border border-red-400/40 py-3 text-sm font-bold text-red-200 transition-colors hover:bg-red-500/10"
          >
            取消订阅
          </button>
          <div className="mt-6 border-t border-white/10 pt-5 text-xs leading-relaxed text-slate-500">
            App Store 和 Google Play 版本只显示订阅状态；订阅与取消的支付操作统一在官网完成。
          </div>
        </GlowCard>
      </section>

      {/* 操作弹窗：先输入钱包（加密货币再选时长），再扫码签名。三类操作共用一个弹窗。 */}
      {pending && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-navy-950/80 px-6 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label="会员操作"
          onClick={closeModal}
        >
          <div
            className="w-full max-w-sm rounded-2xl border border-white/10 bg-navy-900 p-6"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="mb-4 flex items-center justify-between">
              <h3 className="text-lg font-semibold text-white">{actionModalTitle(pending, pendingPlan?.name)}</h3>
              <button
                type="button"
                onClick={closeModal}
                aria-label="关闭"
                className="flex h-9 w-9 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-white/10 hover:text-white"
              >
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {!signing ? (
              <>
                {pending.kind === 'cancel' && (
                  <p className="mb-4 text-xs leading-relaxed text-slate-400">
                    卡订阅到期终止，加密货币预付到期自然失效；扫码签名后按当前支付方式识别。
                  </p>
                )}

                <label className="block text-sm font-semibold text-slate-200" htmlFor="owner-account">
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

                {pending.kind === 'crypto-prepaid' && pendingPlan && (
                  <div className="mt-4">
                    <div className="mb-2 text-xs font-semibold text-slate-400">预付时长</div>
                    <div className="flex gap-2">
                      {(['quarter', 'year'] as const).map((duration) => {
                        const label = duration === 'year' ? '年付 · 12 个月' : '季付 · 3 个月'
                        const total = prepaidTotalCents(pendingPlan, duration)
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
                            <div className="mt-0.5 text-sm font-extrabold text-gold-300">{formatUsd(total)}</div>
                          </button>
                        )
                      })}
                    </div>
                    <p className="mt-2 text-xs leading-relaxed text-slate-500">
                      已有其它档加密货币会员时，将自动按剩余时长折算换档（补差价或补时长）。
                    </p>
                  </div>
                )}

                <button
                  type="button"
                  onClick={beginSigning}
                  disabled={loading}
                  className="mt-6 w-full rounded-lg bg-gold-500 px-5 py-3 text-sm font-bold text-navy-950 transition-colors hover:bg-gold-400 disabled:cursor-not-allowed disabled:bg-slate-600 disabled:text-slate-300"
                >
                  {loading ? '正在处理...' : actionButtonLabel(pending.kind)}
                </button>
              </>
            ) : (
              <>
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
              </>
            )}
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
