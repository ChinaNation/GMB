import { Link } from 'react-router-dom'

export default function Footer() {
  return (
    <footer className="border-t border-white/10 bg-navy-950">
      <div className="mx-auto max-w-7xl px-6 py-16">
        <div className="grid gap-12 md:grid-cols-4">
          <div className="md:col-span-2">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-gold-400 to-gold-600 text-lg font-bold text-navy-950">
                G
              </div>
              <div>
                <div className="text-sm font-semibold text-white">公民币区块链</div>
                <div className="text-[10px] tracking-wider text-gold-400">CITIZENCHAIN</div>
              </div>
            </div>
            <p className="mt-4 max-w-md text-sm leading-relaxed text-slate-400">
              中华民族联邦共和国公民储备委员会，致力于构建主权区块链，
              服务于公民建国运动，建立自由民主的中华民族联邦共和国。
            </p>
          </div>

          <div>
            <h4 className="mb-4 text-sm font-semibold tracking-wider text-gold-400">快速链接</h4>
            <ul className="space-y-2 text-sm">
              <li><Link to="/about" className="text-slate-400 no-underline transition-colors hover:text-white">关于我们</Link></li>
              <li><Link to="/technology" className="text-slate-400 no-underline transition-colors hover:text-white">区块链技术</Link></li>
              <li><Link to="/tokenomics" className="text-slate-400 no-underline transition-colors hover:text-white">公民币经济</Link></li>
              <li><Link to="/governance" className="text-slate-400 no-underline transition-colors hover:text-white">治理体系</Link></li>
            </ul>
          </div>

          <div>
            <h4 className="mb-4 text-sm font-semibold tracking-wider text-gold-400">生态系统</h4>
            <ul className="space-y-2 text-sm">
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">SFID 身份系统</Link></li>
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">CPMS 护照系统</Link></li>
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">WuminApp 移动端</Link></li>
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">全节点网络</Link></li>
            </ul>
          </div>
        </div>

        <div className="mt-12 flex flex-col items-center justify-between gap-4 border-t border-white/10 pt-8 md:flex-row">
          <p className="text-xs text-slate-500">
            &copy; {new Date().getFullYear()} 中华民族联邦共和国公民储备委员会 &mdash; 版权所有
          </p>
          <p className="text-xs text-slate-500">
            基于 Substrate 构建的主权区块链
          </p>
        </div>
      </div>
    </footer>
  )
}
