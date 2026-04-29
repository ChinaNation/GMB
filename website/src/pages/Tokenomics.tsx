import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'

const allocations = [
  {
    name: '创世发行',
    amount: '1,443.49 亿',
    percent: 7.10,
    desc: '创世发行由国储会账户持有',
    color: 'bg-gold-400',
  },
  {
    name: '省储行创立发行',
    amount: '1,443.49 亿',
    percent: 7.10,
    desc: '永久质押省储行创立发行的公民币，不进入流通',
    color: 'bg-gold-600',
  },
  {
    name: '省储行利息发行',
    amount: '728.97 亿',
    percent: 3.58,
    desc: '省储行利息按质押基数 100 年线性释放',
    color: 'bg-navy-400',
  },
  {
    name: '全节点铸块发行',
    amount: '999.89 亿',
    percent: 4.92,
    desc: 'PoW 出块奖励，逐块释放约 1000 万个区块',
    color: 'bg-navy-300',
  },
]

const economics = [
  { label: '代币符号', value: 'GMB' },
  { label: '代币名称', value: '公民币' },
  { label: '基本单位', value: '元 (Yuan)' },
  { label: '最小单位', value: '分 (Fen)' },
  { label: '精度', value: '1 元 = 100 分' },
  { label: '流通总量', value: '~1443.49亿 GMB' },
  { label: '交易费率', value: '0.1% (最低 0.1 元)' },
  { label: '最小存款', value: '1.11 元' },
]

const feeDistribution = [
  { name: '全节点奖励', share: '80%', desc: '出块全节点获得交易手续费的 80%' },
  { name: '手续费账户', share: '10%', desc: '国储会手续费账户用于国储会运营' },
  { name: '安全基金', share: '10%', desc: '网络安全与应急储备基金' },
]

export default function Tokenomics() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden py-24 md:py-32">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/2 top-0 h-[500px] w-[700px] -translate-x-1/2 rounded-full bg-gradient-to-b from-gold-500/8 to-transparent blur-3xl" />
        </div>
        <div className="relative mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="公民币经济"
            title="公民币代币经济模型"
            description="基于《公民宪法》发行的法定数字货币，通过多渠道发行机制确保公平分配。"
          />
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Basic Info */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle subtitle="基本参数" title="公民币核心指标" />
        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
          {economics.map((e) => (
            <div key={e.label} className="rounded-xl border border-white/[0.08] bg-white/[0.03] p-5">
              <div className="text-xs font-medium uppercase tracking-wider text-gold-400">{e.label}</div>
              <div className="mt-2 text-sm font-semibold text-white">{e.value}</div>
            </div>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Token Allocation */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="代币分配"
          title="发行分配方案"
          description="公民币通过四种渠道发行，确保公平分配与长期可持续性。"
        />

        <div className="grid gap-6 md:grid-cols-2">
          {allocations.map((a) => (
            <GlowCard key={a.name} glow="gold">
              <div className="mb-4 flex items-center justify-between">
                <h3 className="text-lg font-semibold text-white">{a.name}</h3>
                <span className="text-2xl font-bold text-gold-400">{a.percent}%</span>
              </div>
              <div className="mb-4 h-2 overflow-hidden rounded-full bg-white/10">
                <div className={`h-full rounded-full ${a.color}`} style={{ width: `${a.percent}%` }} />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-slate-400">{a.desc}</span>
                <span className="text-sm font-semibold text-white">{a.amount}</span>
              </div>
            </GlowCard>
          ))}
        </div>

        {/* Additional: Citizen Light Node */}
        <GlowCard glow="gold" className="mt-6">
          <div className="mb-4 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">公民发行</h3>
            <span className="text-2xl font-bold text-gold-400">77.30%</span>
          </div>
          <div className="mb-4 h-2 overflow-hidden rounded-full bg-white/10">
            <div className="h-full rounded-full bg-gradient-to-r from-gold-300 to-gold-500" style={{ width: '77.30%' }} />
          </div>
          <div className="flex flex-col items-start justify-between gap-2 md:flex-row md:items-center">
            <span className="text-sm text-slate-400">经 SFID 认证的公民，按认证数量获得公民币认证奖励，共 1,443,497,378 个名额。</span>
            <div className="flex items-center gap-3">
              <span className="text-sm font-semibold text-white">15,719.81 亿</span>
              <span className="whitespace-nowrap rounded-full border border-gold-500/30 bg-gold-500/10 px-3 py-1 text-xs font-semibold text-gold-400">
                按节点认证发放
              </span>
            </div>
          </div>
        </GlowCard>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Fee Distribution */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="手续费分配"
          title="交易费用分配机制"
          description="交易手续费按 8:1:1 比例分配，激励全节点运营同时保障网络安全。"
        />
        <div className="grid gap-6 md:grid-cols-3">
          {feeDistribution.map((f) => (
            <GlowCard key={f.name} glow="blue" className="text-center">
              <div className="mb-3 text-4xl font-extrabold text-gold-400">{f.share}</div>
              <h3 className="mb-2 text-lg font-semibold text-white">{f.name}</h3>
              <p className="text-sm text-slate-400">{f.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>
    </>
  )
}
