import { Link } from 'react-router-dom'
import flagEmblem from '../assets/flag-emblem.png'

export default function Footer() {
  return (
    <footer className="border-t border-white/10 bg-navy-950">
      <div className="mx-auto max-w-7xl px-6 py-16">
        <div className="grid gap-12 md:grid-cols-4">
          <div className="md:col-span-2">
            <div className="flex items-center gap-3">
              <img
                src={flagEmblem}
                alt="中华民族联邦共和国国徽"
                className="h-10 w-10 rounded-full object-cover ring-1 ring-white/15"
              />
              <div>
                <div className="text-sm font-semibold text-white">中华民族联邦共和国</div>
                <div className="text-xs tracking-wider text-gold-400">公民储备委员会</div>
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
              <li><Link to="/technology" className="text-slate-400 no-underline transition-colors hover:text-white">区块链</Link></li>
              <li><Link to="/tokenomics" className="text-slate-400 no-underline transition-colors hover:text-white">公民币</Link></li>
              <li><Link to="/governance" className="text-slate-400 no-underline transition-colors hover:text-white">治理体系</Link></li>
              <li><Link to="/whitepaper" className="text-slate-400 no-underline transition-colors hover:text-white">白皮书</Link></li>
            </ul>
          </div>

          <div>
            <h4 className="mb-4 text-sm font-semibold tracking-wider text-gold-400">产品</h4>
            <ul className="space-y-2 text-sm">
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">CitizenApp 公民</Link></li>
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">CitizenWallet 公民钱包</Link></li>
              <li><Link to="/ecosystem" className="text-slate-400 no-underline transition-colors hover:text-white">CitizenChain 公民链</Link></li>
            </ul>
          </div>
        </div>

        <div className="mt-12 flex flex-col items-center justify-between gap-4 border-t border-white/10 pt-8 md:flex-row">
          <p className="text-xs text-slate-500">
            &copy; {new Date().getFullYear()} 中华民族联邦共和国公民储备委员会 &mdash; 版权所有
          </p>
        </div>
      </div>
    </footer>
  )
}
