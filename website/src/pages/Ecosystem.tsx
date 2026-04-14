import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'

const systems = [
  {
    name: 'SFID 身份系统',
    subtitle: '身份识别码系统',
    desc: '中央身份绑定与验证系统，将公民身份一对一映射到区块链公钥，管理投票资格与公民快照。',
    features: [
      'Sr25519 挑战-响应认证',
      'QR 码扫描登录',
      '三级管理员体系',
      '公民身份快照',
      '投票资格验证',
      '完整审计日志',
    ],
    tech: 'Rust / Axum / PostgreSQL / React',
    color: 'gold',
  },
  {
    name: 'CPMS 护照系统',
    subtitle: '公民护照管理系统',
    desc: '离线公民档案与 QR 码签发系统，管理公民护照文档号码（V3格式），生成签名 QR 码用于公民绑定。',
    features: [
      '公民档案创建',
      '签名 QR 码生成',
      'V3 格式护照号管理',
      '机构公钥注册 QR',
      'Sr25519 数字签名',
      '操作审计追踪',
    ],
    tech: 'Rust / Axum / PostgreSQL',
    color: 'blue',
  },
  {
    name: 'WuminApp',
    subtitle: '公民移动客户端',
    desc: '面向全体公民的移动端应用，集成钱包管理、交易转账、治理投票、QR 码扫描等核心功能。',
    features: [
      '助记词钱包管理',
      '公民币转账交易',
      '链上治理投票',
      'QR 码扫描绑定',
      '生物识别认证',
      '内置轻节点 (Smoldot)',
    ],
    tech: 'Dart / Flutter / iOS / Android',
    color: 'gold',
  },
  {
    name: 'NodeUI 桌面端',
    subtitle: '全节点桌面应用',
    desc: '基于 Tauri 构建的桌面节点管理界面，集成原生区块链节点程序，提供可视化的节点运维体验。',
    features: [
      'Tauri 原生桌面应用',
      '集成全节点程序',
      '可视化节点状态',
      '区块浏览与查询',
      'PoW 挖矿管理',
      '网络对等节点监控',
    ],
    tech: 'Rust (Tauri) / React / TypeScript',
    color: 'blue',
  },
]

const workflow = [
  {
    step: '01',
    title: '公民绑定',
    desc: 'CitizenChain 发起请求 → CPMS 离线生成签名 QR 码 → SFID 扫描验签 → 链上绑定完成',
  },
  {
    step: '02',
    title: '治理投票',
    desc: '链上创建提案 → SFID 提供选民快照与资格验证 → WuminApp 移动端投票 → 链上记录与执行',
  },
  {
    step: '03',
    title: '管理员登录',
    desc: 'QR 挑战码生成 → 手机端扫描并 Sr25519 签名 → 签名验证通过 → 创建安全会话',
  },
]

export default function Ecosystem() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden py-24 md:py-32">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/2 top-0 h-[500px] w-[700px] -translate-x-1/2 rounded-full bg-gradient-to-b from-gold-500/6 to-transparent blur-3xl" />
        </div>
        <div className="relative mx-auto max-w-7xl px-6">
          <SectionTitle
            subtitle="生态系统"
            title="完整的公民链生态"
            description="从身份认证到移动钱包，从护照签发到桌面节点，公民币区块链构建了完整的生态系统闭环。"
          />
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Systems */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle subtitle="核心产品" title="四大核心系统" />
        <div className="grid gap-8 md:grid-cols-2">
          {systems.map((s) => (
            <GlowCard key={s.name} glow={s.color === 'gold' ? 'gold' : 'blue'} className="flex flex-col">
              <div className="mb-1 text-xs font-medium uppercase tracking-wider text-gold-400">{s.subtitle}</div>
              <h3 className="mb-3 text-2xl font-bold text-white">{s.name}</h3>
              <p className="mb-6 text-sm leading-relaxed text-slate-400">{s.desc}</p>

              <div className="mb-6 grid grid-cols-2 gap-2">
                {s.features.map((f) => (
                  <div key={f} className="flex items-center gap-2 text-sm text-slate-300">
                    <span className="h-1 w-1 rounded-full bg-gold-400" />
                    {f}
                  </div>
                ))}
              </div>

              <div className="mt-auto border-t border-white/10 pt-4">
                <span className="text-xs font-medium text-slate-500">技术栈：</span>
                <span className="ml-1 text-xs font-mono text-slate-400">{s.tech}</span>
              </div>
            </GlowCard>
          ))}
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-white/10 to-transparent" />

      {/* Cross-product workflow */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle
          subtitle="协作流程"
          title="跨产品协作工作流"
          description="各系统之间通过加密签名与链上验证实现无缝协作。"
        />
        <div className="grid gap-8 md:grid-cols-3">
          {workflow.map((w) => (
            <GlowCard key={w.step} glow="gold">
              <div className="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-gold-400/20 to-gold-600/20 text-lg font-bold text-gold-400">
                {w.step}
              </div>
              <h3 className="mb-3 text-lg font-semibold text-white">{w.title}</h3>
              <p className="text-sm leading-relaxed text-slate-400">{w.desc}</p>
            </GlowCard>
          ))}
        </div>
      </section>

      {/* Security */}
      <section className="border-t border-white/10 bg-gradient-to-b from-navy-900/40 to-navy-950 py-24">
        <div className="mx-auto max-w-5xl px-6">
          <SectionTitle subtitle="安全保障" title="端到端密码学安全" />
          <div className="grid gap-6 md:grid-cols-3">
            <GlowCard glow="blue" className="text-center">
              <div className="mb-3 text-3xl font-bold text-gold-400">Sr25519</div>
              <h3 className="mb-2 text-base font-semibold text-white">数字签名</h3>
              <p className="text-sm text-slate-400">Schnorr 签名方案，提供高效安全的身份认证与交易签名</p>
            </GlowCard>
            <GlowCard glow="blue" className="text-center">
              <div className="mb-3 text-3xl font-bold text-gold-400">Blake2</div>
              <h3 className="mb-2 text-base font-semibold text-white">哈希算法</h3>
              <p className="text-sm text-slate-400">高性能密码学哈希，用于区块哈希、默克尔树与数据完整性验证</p>
            </GlowCard>
            <GlowCard glow="blue" className="text-center">
              <div className="mb-3 text-3xl font-bold text-gold-400">AES-256</div>
              <h3 className="mb-2 text-base font-semibold text-white">数据加密</h3>
              <p className="text-sm text-slate-400">AES-256-GCM 加密保护省级签名密钥，HKDF 密钥派生确保安全</p>
            </GlowCard>
          </div>
        </div>
      </section>
    </>
  )
}
