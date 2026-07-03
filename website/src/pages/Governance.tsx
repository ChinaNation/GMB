import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'

const votingTiers = [
  {
    tier: '内部投票',
    name: '内部投票',
    scope: '各机构 / 机构多签 / 个人多签',
    desc: '各机构与多签账户内部事项（如内部转账等）的决策投票，由该主体管理员参与',
    voters: '主体内管理员',
    threshold: '管理员快照 + 阈值快照，达阈值通过',
    extraTag: '',
    color: 'gold',
  },
  {
    tier: '联合投票',
    name: '联合投票',
    scope: '仅治理机构：国家储委会 + 全部省储委会 + 全部省储行',
    desc: '仅治理机构之间的共同治理事项，需国家储委会（19票）、省储委会（43票）、省储行（43票）联合投票',
    voters: '105 票总计',
    threshold: '全票立即通过',
    extraTag: '非全票则联合公投',
    color: 'gold',
  },
  {
    tier: '立法投票',
    name: '立法投票',
    scope: '立法机构：市自治会 / 市教委会 / 市立法会 / 省·国家参议会众议会 / 国家教委会',
    desc: '仅立法机构的修法表决，严格按公民宪法的表决类型与阈值，含两院顺序、行政签署与强制公投',
    voters: '对应立法院成员',
    threshold: '依宪法表决类型与阈值',
    extraTag: '回调立法院写入新法律版本',
    color: 'gold',
  },
  {
    tier: '选举投票',
    name: '选举投票',
    scope: '公职人员选举：公民普选 + 公权机构成员互选',
    desc: '按公民宪法选举各类公职人员，既含公民普选，也含公权机构成员的内部互选',
    voters: '视职位：全体认证公民 / 特定机构现任成员',
    threshold: '按职位取选民快照，依宪法阈值通过',
    extraTag: '',
    color: 'gold',
  },
]

const rules = [
  {
    icon: (
      <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    title: '投票期限',
    desc: '每次投票最长 30 天，基于区块链高度计算截止时间',
  },
  {
    icon: (
      <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    title: '二元选择',
    desc: '投票仅允许"赞成"或"反对"，简洁明确，杜绝歧义',
  },
  {
    icon: (
      <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 3v11.25A2.25 2.25 0 006 16.5h2.25M3.75 3h-1.5m1.5 0h16.5m0 0h1.5m-1.5 0v11.25A2.25 2.25 0 0118 16.5h-2.25m-7.5 0h7.5m-7.5 0l-1 3m8.5-3l1 3m0 0l.5 1.5m-.5-1.5h-9.5m0 0l-.5 1.5" />
      </svg>
    ),
    title: '链上透明',
    desc: '所有投票记录与计票结果存储在区块链上，公开可验证',
  },
  {
    icon: (
      <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
      </svg>
    ),
    title: '链上公民身份',
    desc: '选举投票与联合公投读取 citizen-identity 快照，确保一人一票，防止重复投票',
  },
]

const proposalTypes = [
  { name: '货币增发提案', desc: '经联合投票通过后执行公民币增发' },
  { name: '货币销毁提案', desc: '经治理投票通过后销毁指定数量的公民币' },
  { name: 'GRANDPA 密钥轮换', desc: '权威节点 GRANDPA 终局性验证密钥的更新与轮换' },
  { name: '管理员变更', desc: '各级储备委员会管理员的增减与替换' },
  { name: '协议升级', desc: '通过链上 setCode 进行 WASM 运行时无分叉升级区块链协议' },
  { name: '参数调整', desc: '交易费率、出块奖励、难度参数等链上参数调整' },
]

export default function Governance() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden py-24 md:py-32">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/3 top-0 h-[500px] w-[600px] rounded-full bg-gold-500/6 blur-3xl" />
          <div className="absolute right-1/4 top-1/4 h-[400px] w-[500px] rounded-full bg-navy-500/10 blur-3xl" />
        </div>
        <div className="relative mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="治理体系"
            title="四种民主投票机制"
            description="从机构内部事项、治理机构联合决策、立法机构修法到公职人员选举，四种投票确保每一位公民都有权参与国家治理。"
          />
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Three Tiers */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle subtitle="投票分类" title="四种投票体系" />
        <div className="grid gap-8 md:grid-cols-3">
          {votingTiers.map((v) => (
            <GlowCard key={v.name} glow="gold" className="flex flex-col">
              <div className="mb-4 inline-flex items-center gap-2">
                <span className="rounded-full bg-gold-500/10 px-3 py-1 text-xs font-bold text-gold-400">{v.tier}</span>
              </div>
              <h3 className="mb-2 text-xl font-bold text-white">{v.name}</h3>
              <p className="mb-1 text-xs font-medium text-gold-400/80">{v.scope}</p>
              <p className="mb-6 flex-1 text-sm leading-relaxed text-slate-400">{v.desc}</p>
              <div className="flex flex-wrap gap-2 border-t border-white/10 pt-4">
                <span className="rounded-md bg-white/5 px-2 py-1 text-xs text-slate-300">{v.voters}</span>
                <span className="rounded-md bg-white/5 px-2 py-1 text-xs text-slate-300">{v.threshold}</span>
                {v.extraTag && (
                  <span className="rounded-md bg-white/5 px-2 py-1 text-xs text-slate-300">{v.extraTag}</span>
                )}
              </div>
            </GlowCard>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Voting Rules */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle subtitle="投票规则" title="链上治理基本规则" />
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          {rules.map((r) => (
            <GlowCard key={r.title} glow="blue" className="text-center">
              <div className="mx-auto mb-4 inline-flex rounded-xl bg-navy-500/20 p-3 text-navy-300">
                {r.icon}
              </div>
              <h3 className="mb-2 text-base font-semibold text-white">{r.title}</h3>
              <p className="text-sm leading-relaxed text-slate-400">{r.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Proposal Types */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="提案类型"
          title="链上治理提案"
          description="支持多种类型的链上治理提案，覆盖货币政策、技术升级与组织管理。"
        />
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {proposalTypes.map((p) => (
            <div key={p.name} className="rounded-xl border border-white/[0.08] bg-white/[0.03] p-6">
              <h3 className="mb-2 text-base font-semibold text-white">{p.name}</h3>
              <p className="text-sm text-slate-400">{p.desc}</p>
            </div>
          ))}
        </div>
      </section>

      {/* CTA */}
      <section className="border-t border-white/10 bg-gradient-to-b from-navy-900/40 to-navy-950 py-24">
        <div className="mx-auto max-w-3xl px-6 text-center">
          <h2 className="text-3xl font-bold text-white">公民的权利，链上的保障</h2>
          <p className="mt-6 text-lg text-slate-400">
            每一票都记录在区块链上，透明、不可篡改。通过 CitizenApp 客户端，
            每位认证公民都可以直接参与链上治理投票。
          </p>
        </div>
      </section>
    </>
  )
}
