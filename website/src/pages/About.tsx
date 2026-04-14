import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'

const principles = [
  { title: '民治', desc: '公民治理国家事务，建立公民自治的政府体系', icon: '01' },
  { title: '民主', desc: '民主共和制度，保障公民选举与被选举的权利', icon: '02' },
  { title: '民权', desc: '维护公民基本权利与自由，法律面前人人平等', icon: '03' },
  { title: '民生', desc: '建设公民社会，提升全民福祉与生活品质', icon: '04' },
  { title: '民族', desc: '复兴中华民族文化，凝聚民族精神与认同', icon: '05' },
]

const orgStructure = [
  {
    level: '国家储备委员会',
    count: '1',
    admins: '19 位委员',
    threshold: '13/19 多签管理',
    desc: '负责行使国家铸币权与全网治理决策',
    color: 'gold',
  },
  {
    level: '省级储备委员会',
    count: '43',
    admins: '9 位委员/省',
    threshold: '6/9 多签管理',
    desc: '行使省铸币权，负责省储备事务与区域治理',
    color: 'blue',
  },
  {
    level: '省级银行',
    count: '43',
    admins: '9 位董事/行',
    threshold: '6/9 多签管理',
    desc: '执行省级金融服务与公民币流通',
    color: 'blue',
  },
]

export default function About() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden py-24 md:py-32">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/2 top-0 h-[500px] w-[700px] -translate-x-1/2 rounded-full bg-gradient-to-b from-gold-500/8 to-transparent blur-3xl" />
        </div>
        <div className="relative mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="关于我们"
            title="中华民族联邦共和国公民储备委员会"
            description="致力于构建主权区块链基础设施，发行公民币法定数字货币，服务公民建国运动，建立自由民主的中华民族联邦共和国。"
          />
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Five Principles */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="核心理念"
          title="五民主义"
          description="公民币区块链以五民主义为核心思想指导，构建自由、民主、公正的公民社会。"
        />

        <div className="grid gap-6 md:grid-cols-3 lg:grid-cols-5">
          {principles.map((p) => (
            <GlowCard key={p.title} glow="gold" className="text-center">
              <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-gradient-to-br from-gold-400/20 to-gold-600/20 text-sm font-bold text-gold-400">
                {p.icon}
              </div>
              <h3 className="mb-2 text-xl font-bold text-white">{p.title}</h3>
              <p className="text-sm leading-relaxed text-slate-400">{p.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Organization Structure */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="组织架构"
          title="三级储备体系"
          description="国家储备委员会、省储备委员会、省储备银行构成完整的三级治理与治理架构。"
        />

        <div className="grid gap-8 md:grid-cols-3">
          {orgStructure.map((org) => (
            <GlowCard key={org.level} glow={org.color === 'gold' ? 'gold' : 'blue'}>
              <div className={`mb-4 inline-flex rounded-xl p-3 ${org.color === 'gold' ? 'bg-gold-500/10 text-gold-400' : 'bg-navy-500/20 text-navy-300'}`}>
                <span className="text-3xl font-extrabold">{org.count}</span>
              </div>
              <h3 className="mb-2 text-xl font-semibold text-white">{org.level}</h3>
              <div className="mb-4 flex flex-wrap gap-2">
                <span className="rounded-md bg-white/5 px-2 py-1 text-xs text-slate-300">{org.admins}</span>
                <span className="rounded-md bg-white/5 px-2 py-1 text-xs text-slate-300">{org.threshold}</span>
              </div>
              <p className="text-sm leading-relaxed text-slate-400">{org.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>

      {/* Mission */}
      <section className="border-t border-white/10 bg-gradient-to-b from-navy-900/40 to-navy-950 py-24">
        <div className="mx-auto max-w-4xl px-6 text-center">
          <h2 className="text-3xl font-bold text-white md:text-4xl">我们的使命</h2>
          <p className="mt-8 text-lg leading-relaxed text-slate-400">
            建立一套不受任何单一机构控制的去中心化数字货币体系，让每一位公民都能平等地参与国家治理与经济建设。
            通过区块链技术保障交易透明、身份自主、治理民主，推动公民建国运动，最终建立自由民主的中华民族联邦共和国。
          </p>
        </div>
      </section>
    </>
  )
}
