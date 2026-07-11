import { Link } from 'react-router-dom'
import GlowCard from '../components/GlowCard'

const stats = [
  { value: '2.23万亿', label: '固定发行合计 (GMB)', suffix: '' },
  { value: '44个', label: '权威节点（国家储委会+省储委会）', suffix: '' },
  { value: '43个', label: '权益节点（省储行）', suffix: '' },
  { value: '4类', label: '投票引擎', suffix: '' },
]

const features = [
  {
    icon: (
      <svg className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
      </svg>
    ),
    title: '主权区块链',
    desc: 'PoW 出块与 GRANDPA 最终性共同维护主权区块链，保障公民币和链上治理规则',
  },
  {
    icon: (
      <svg className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
      </svg>
    ),
    title: '链上中国平台',
    desc: '区块链系统的链上中国平台，承接公民档案、机构注册、链上身份提交、立法提案、选举提案等链下上链业务',
  },
  {
    icon: (
      <svg className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M7.864 4.243A7.5 7.5 0 0119.5 10.5c0 2.92-.556 5.709-1.568 8.268M5.742 6.364A7.465 7.465 0 004.5 10.5a48.667 48.667 0 00-1.115 10.336M4.095 20.693A7.5 7.5 0 0112 3a7.5 7.5 0 017.905 8.807M12 12.75a.75.75 0 110-1.5.75.75 0 010 1.5z" />
      </svg>
    ),
    title: '立法与法律文库',
    desc: '公民宪法、普通法律、立法提案和法律版本历史可在链上中国中阅读与追溯',
  },
  {
    icon: (
      <svg className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 18.75a60.07 60.07 0 0115.797 2.101c.727.198 1.453-.342 1.453-1.096V18.75M3.75 4.5v.75A.75.75 0 013 6h-.75m0 0v-.375c0-.621.504-1.125 1.125-1.125H20.25M2.25 6v9m18-10.5v.75c0 .414.336.75.75.75h.75m-1.5-1.5h.375c.621 0 1.125.504 1.125 1.125v9.75c0 .621-.504 1.125-1.125 1.125h-.375m1.5-1.5H21a.75.75 0 00-.75.75v.75m0 0H3.75m0 0h-.375a1.125 1.125 0 01-1.125-1.125V15m1.5 1.5v-.75A.75.75 0 003 15h-.75M15 10.5a3 3 0 11-6 0 3 3 0 016 0zm3 0h.008v.008H18V10.5zm-12 0h.008v.008H6V10.5z" />
      </svg>
    ),
    title: '数字法定货币',
    desc: '固定发行合计 2,229,386,218,778 GMB，后续决议发行受联合投票和链上限额约束',
  },
  {
    icon: (
      <svg className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" />
      </svg>
    ),
    title: '投票引擎',
    desc: '内部投票、联合投票、立法投票、选举投票统一由链上投票引擎管理',
  },
  {
    icon: (
      <svg className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 21h16.5M4.5 3h15l-.75 18H5.25L4.5 3zm4.5 4.5h1.5m3 0H15m-6 4.5h1.5m3 0H15m-6 4.5h1.5m3 0H15" />
      </svg>
    ),
    title: '私权机构注册',
    desc: '个体经营、合伙企业、股权公司、股份公司、公益组织和注册协会等社会私权机构由链上中国平台注册局注册上链',
  },
]

export default function Home() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden">
        {/* Background effects */}
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/2 top-0 h-[600px] w-[800px] -translate-x-1/2 rounded-full bg-gradient-to-b from-gold-500/8 to-transparent blur-3xl" />
          <div className="absolute right-0 top-1/3 h-[400px] w-[400px] rounded-full bg-navy-500/10 blur-3xl" />
        </div>

        <div className="relative mx-auto max-w-7xl px-6 pb-24 pt-24 md:pt-32 lg:pt-40">
          <div className="mx-auto max-w-4xl text-center">
            <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-gold-500/20 bg-gold-500/5 px-4 py-2">
              <span className="h-2 w-2 animate-pulse rounded-full bg-gold-400" />
              <span className="text-xs font-medium tracking-wider text-gold-300">主网运行中</span>
            </div>

            <h1 className="text-4xl font-extrabold leading-tight tracking-tight text-white md:text-5xl lg:text-6xl">
              <span className="block text-lg font-medium tracking-widest text-gold-400 md:text-xl">
                中华民族联邦共和国
              </span>
              <span className="mt-2 block bg-gradient-to-r from-gold-300 via-gold-400 to-gold-500 bg-clip-text text-transparent">
                公民储备委员会
              </span>
            </h1>

            <p className="mx-auto mt-8 max-w-2xl text-lg leading-relaxed text-slate-400 md:text-xl">
              构建去中心化主权区块链，发行法定数字公民币，
              推动公民建国运动，建立自由民主的中华民族联邦共和国
            </p>

            <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
              <Link
                to="/about"
                className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-gold-500 to-gold-600 px-8 py-3.5 text-sm font-semibold text-navy-950 no-underline shadow-lg shadow-gold-500/25 transition-all hover:shadow-gold-500/40"
              >
                了解更多
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                </svg>
              </Link>
              <Link
                to="/technology"
                className="inline-flex items-center gap-2 rounded-xl border border-white/20 px-8 py-3.5 text-sm font-semibold text-white no-underline transition-all hover:border-white/40 hover:bg-white/5"
              >
                技术架构
              </Link>
            </div>
          </div>

          {/* Stats */}
          <div className="mx-auto mt-20 grid max-w-4xl grid-cols-2 gap-6 md:grid-cols-4">
            {stats.map((stat) => (
              <div key={stat.label} className="text-center">
                <div className="text-3xl font-bold text-white md:text-4xl">{stat.value}</div>
                <div className="mt-2 text-sm text-slate-400">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Divider */}
      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Features */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <div className="mx-auto mb-16 max-w-3xl text-center">
          <span className="mb-4 inline-block rounded-full border border-gold-500/30 bg-gold-500/10 px-4 py-1.5 text-xs font-semibold uppercase tracking-widest text-gold-400">
            核心特性
          </span>
          <h2 className="mt-4 text-3xl font-bold tracking-tight text-white md:text-4xl">
            为公民构建的主权区块链
          </h2>
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          {features.map((f) => (
            <GlowCard key={f.title} glow="gold">
              <div className="mb-4 inline-flex rounded-xl bg-gold-500/10 p-3 text-gold-400">
                {f.icon}
              </div>
              <h3 className="mb-3 text-xl font-semibold text-white">{f.title}</h3>
              <p className="leading-relaxed text-slate-400">{f.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>

      {/* CTA */}
      <section className="relative overflow-hidden border-t border-white/10 py-24">
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-navy-900/50 to-navy-950" />
        <div className="relative mx-auto max-w-3xl px-6 text-center">
          <h2 className="text-3xl font-bold text-white md:text-4xl">
            加入公民链生态
          </h2>
          <p className="mt-6 text-lg text-slate-400">
            下载 CitizenApp 移动客户端，成为公民轻节点，参与区块链治理
          </p>
          <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
            <Link
              to="/ecosystem"
              className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-gold-500 to-gold-600 px-8 py-3.5 text-sm font-semibold text-navy-950 no-underline shadow-lg shadow-gold-500/25 transition-all hover:shadow-gold-500/40"
            >
              探索产品体系
            </Link>
            <Link
              to="/governance"
              className="inline-flex items-center gap-2 rounded-xl border border-white/20 px-8 py-3.5 text-sm font-semibold text-white no-underline transition-all hover:border-white/40 hover:bg-white/5"
            >
              了解治理体系
            </Link>
          </div>
        </div>
      </section>
    </>
  )
}
