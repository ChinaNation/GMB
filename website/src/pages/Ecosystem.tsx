import SectionTitle from '../components/SectionTitle'
import GlowCard from '../components/GlowCard'

const systems = [
  {
    name: '链上中国平台',
    subtitle: '本地治理与注册平台',
    desc: '节点内置的链上中国平台，承接注册局业务、法律文库、立法入口、机构注册、公民档案和链上身份提交。',
    features: [
      '链上管理员鉴权',
      '扫码签名登录',
      '公民电子护照',
      '公权机构注册',
      '私权机构注册',
      '法律文库展示',
      '立法入口生成',
      '完整审计日志',
    ],
    tech: 'Rust / Axum / PostgreSQL / React',
    color: 'gold',
  },
  {
    name: 'CitizenApp',
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
    fullWidth: false,
  },
  {
    name: 'CitizenWallet 冷钱包',
    subtitle: '离线签名冷钱包',
    desc: '完全离线运行的签名钱包，用于关键身份与治理操作的气隙式 Sr25519 签名，与 CitizenApp 热钱包形成"冷热分离"的双钱包体系。',
    features: [
      '完全离线运行（气隙）',
      'Sr25519 离线签名',
      'QR_V1 统一扫码协议',
      '加密 keystore 私钥保管',
      '交易离线签名',
      '管理员激活签名',
      '治理提案离线签名',
      '登录签名请求/响应',
    ],
    tech: 'Dart / Flutter / 完全离线 / 独立应用',
    color: 'gold',
    fullWidth: true,
  },
]

const workflow = [
  {
    step: '01',
    title: '公民上链',
    desc: '注册局创建公民档案和电子护照 → 公民钱包签名确认 → citizen-identity 写入投票或参选字段',
  },
  {
    step: '02',
    title: '机构注册',
    desc: '链上中国锁定机构类型和管理员集合 → 注册局提交交易 → 公权、教育、私权机构上链生效',
  },
  {
    step: '03',
    title: '立法选举',
    desc: '法律文库组织资料 → 立法入口生成待签交易 → 投票引擎读取公民快照并推进表决或选举',
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
            description="从链上中国到移动钱包，从法律文库到投票引擎，公民链把注册、立法、选举、机构和资产操作连成完整闭环。"
          />
        </div>
      </section>

      <div className="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-gold-500/30 to-transparent" />

      {/* Systems */}
      <section className="mx-auto max-w-7xl px-6 py-24">
        <SectionTitle subtitle="核心产品" title="四大核心系统" />
        <div className="grid gap-8 md:grid-cols-2">
          {systems.map((s) => (
            <GlowCard
              key={s.name}
              glow={s.color === 'gold' ? 'gold' : 'blue'}
              className={`flex flex-col ${s.fullWidth ? 'md:col-span-2' : ''}`}
            >
              <div className="mb-1 text-xs font-medium uppercase tracking-wider text-gold-400">{s.subtitle}</div>
              <h3 className="mb-3 text-2xl font-bold text-white">{s.name}</h3>
              <p className="mb-6 text-sm leading-relaxed text-slate-400">{s.desc}</p>

              <div className={`mb-6 grid gap-2 grid-cols-2 ${s.fullWidth ? 'md:grid-cols-4' : ''}`}>
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
          description="各系统之间通过扫码签名、链上身份和投票引擎实现可信协作。"
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
