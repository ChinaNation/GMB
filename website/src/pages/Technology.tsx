import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'

const techStack = [
  { label: '核心语言', value: 'Rust 2021' },
  { label: '区块链框架', value: 'Substrate / Polkadot SDK' },
  { label: '网络协议', value: 'libp2p / litep2p' },
  { label: '签名算法', value: 'Sr25519' },
  { label: '哈希算法', value: 'Blake2' },
  { label: '存储引擎', value: 'RocksDB' },
  { label: '运行时', value: 'WASM' },
  { label: '终局性', value: 'GRANDPA' },
]

const pallets = [
  {
    name: 'PoW 共识',
    module: 'pow-difficulty-module',
    desc: '工作量证明挖矿机制，动态难度调整，保障全节点公平参与出块',
  },
  {
    name: 'GRANDPA 终局性',
    module: 'grandpa-key-gov',
    desc: '44 个权威节点参与 GRANDPA 终局性投票，确保区块不可回滚',
  },
  {
    name: 'SFID 身份认证',
    module: 'sfid-code-auth',
    desc: '链上公民身份绑定与验证，一人一链上身份，保障投票资格',
  },
  {
    name: '公民轻节点发行',
    module: 'citizen-lightnode-issuance',
    desc: '经 SFID 认证的公民轻节点获得公民币发行，按节点认证数量线性释放',
  },
  {
    name: '全节点 PoW 奖励',
    module: 'fullnode-pow-reward',
    desc: '全节点通过 PoW 出块获取区块奖励，奖励逐块释放，跨约 1000 万区块',
  },
  {
    name: '治理投票引擎',
    module: 'voting-engine-system',
    desc: '三级投票引擎：内部投票、联合投票、公民投票，链上透明计票',
  },
  {
    name: '发行决议治理',
    module: 'resolution-issuance-gov',
    desc: '货币增发提案投票机制，需经多级治理审批通过方可执行',
  },
  {
    name: '管理员治理',
    module: 'admins-origin-gov',
    desc: '多签管理员体系，国储 13/19、省储 6/9 签名门槛，保障去中心化决策',
  },
]

const nodeTypes = [
  {
    type: '国储会权威节点',
    count: '1',
    desc: '国家级货币发行控制，19 位管理员多签治理',
    features: ['国家铸币权', '全网治理', '13/19 多签'],
  },
  {
    type: '省储会权威节点',
    count: '43',
    desc: '省级储备管理，每省 9 位管理员',
    features: ['省铸币权', '省级治理', '联合投票', '6/9 多签'],
  },
  {
    type: '省储行权益节点',
    count: '43',
    desc: '省级金融服务执行，9 位董事管理',
    features: ['金融服务', '质押利息', '链下支付'],
  },
  {
    type: '全节点',
    count: '无限',
    desc: '任何组织或个人均可运行，参与 PoW 出块',
    features: ['PoW 出块', '交易验证', '去中心化'],
  },
]

export default function Technology() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden py-24 md:py-32">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/4 top-0 h-[500px] w-[600px] rounded-full bg-navy-500/10 blur-3xl" />
          <div className="absolute right-1/4 top-1/3 h-[400px] w-[500px] rounded-full bg-gold-500/5 blur-3xl" />
        </div>
        <div className="relative mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="区块链技术"
            title="基于 Substrate 的主权区块链"
            description="采用 Rust 语言与 Polkadot SDK 构建，PoW + GRANDPA 混合共识，WASM 可升级运行时，打造安全、可扩展、可治理的主权区块链。"
          />
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Tech Stack */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle subtitle="技术栈" title="核心技术选型" />
        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
          {techStack.map((t) => (
            <div key={t.label} className="rounded-xl border border-white/[0.08] bg-white/[0.03] p-5 text-center">
              <div className="text-xs font-medium uppercase tracking-wider text-gold-400">{t.label}</div>
              <div className="mt-2 text-sm font-semibold text-white">{t.value}</div>
            </div>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Runtime Pallets */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="运行时模块"
          title="链上 Pallet 架构"
          description="模块化的 WASM 运行时，支持链上无分叉升级，每个 Pallet 负责独立业务逻辑。"
        />
        <div className="grid gap-6 md:grid-cols-2">
          {pallets.map((p) => (
            <GlowCard key={p.name} glow="blue">
              <div className="mb-1 text-xs font-mono tracking-wider text-navy-300">{p.module}</div>
              <h3 className="mb-3 text-lg font-semibold text-white">{p.name}</h3>
              <p className="text-sm leading-relaxed text-slate-400">{p.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Node Architecture */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="节点体系"
          title="四类节点架构"
          description="从国家级权威节点到公民全节点，构成完整的去中心化网络拓扑。"
        />
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          {nodeTypes.map((n) => (
            <GlowCard key={n.type} glow="gold" className="flex flex-col">
              <div className="mb-4 text-4xl font-extrabold text-gold-400">{n.count}</div>
              <h3 className="mb-2 text-lg font-semibold text-white">{n.type}</h3>
              <p className="mb-4 flex-1 text-sm text-slate-400">{n.desc}</p>
              <div className="flex flex-wrap gap-2">
                {n.features.map((f) => (
                  <span key={f} className="rounded-md bg-gold-500/10 px-2 py-1 text-xs font-medium text-gold-300">
                    {f}
                  </span>
                ))}
              </div>
            </GlowCard>
          ))}
        </div>
      </section>

      {/* Consensus */}
      <section className="border-t border-white/10 bg-gradient-to-b from-navy-900/40 to-navy-950 py-24">
        <div className="mx-auto max-w-5xl px-6">
          <SectionTitle
            subtitle="共识机制"
            title="PoW + GRANDPA 混合共识"
          />
          <div className="grid gap-8 md:grid-cols-2">
            <GlowCard glow="gold">
              <h3 className="mb-4 text-xl font-semibold text-white">PoW 工作量证明</h3>
              <ul className="space-y-3 text-sm text-slate-400">
                <li className="flex gap-3">
                  <span className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-gold-400" />
                  全节点通过算力竞争出块权
                </li>
                <li className="flex gap-3">
                  <span className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-gold-400" />
                  动态难度调整确保稳定出块
                </li>
                <li className="flex gap-3">
                  <span className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-gold-400" />
                  任何人均可参与，去中心化保障
                </li>
              </ul>
            </GlowCard>
            <GlowCard glow="blue">
              <h3 className="mb-4 text-xl font-semibold text-white">GRANDPA 终局性</h3>
              <ul className="space-y-3 text-sm text-slate-400">
                <li className="flex gap-3">
                  <span className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-navy-300" />
                  44 个权威节点参与终局性投票
                </li>
                <li className="flex gap-3">
                  <span className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-navy-300" />
                  确保已确认区块不可回滚
                </li>
                <li className="flex gap-3">
                  <span className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-navy-300" />
                  拜占庭容错，2/3+ 诚实即安全
                </li>
              </ul>
            </GlowCard>
          </div>
        </div>
      </section>
    </>
  )
}
