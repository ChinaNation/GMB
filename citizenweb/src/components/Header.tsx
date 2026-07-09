import { useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import flagEmblem from '../assets/flag-emblem.png'

const navItems = [
  { path: '/', label: '首页' },
  { path: '/technology', label: '区块链' },
  { path: '/tokenomics', label: '公民币' },
  { path: '/ecosystem', label: '产品' },
  { path: '/membership', label: '会员订阅' },
  { path: '/governance', label: '治理体系' },
  { path: '/whitepaper', label: '白皮书' },
  { path: '/about', label: '关于我们' },
]

export default function Header() {
  const location = useLocation()
  const [mobileOpen, setMobileOpen] = useState(false)

  return (
    <header className="fixed top-0 left-0 right-0 z-50 border-b border-white/10 bg-navy-950/80 backdrop-blur-xl">
      <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
        <Link to="/" className="flex items-center gap-3 no-underline">
          <img
            src={flagEmblem}
            alt="中华民族联邦共和国国徽"
            className="h-10 w-10 rounded-full object-cover ring-1 ring-white/15"
          />
          <div className="hidden sm:block">
            <div className="text-base font-semibold tracking-wide text-white">中华民族联邦共和国</div>
            <div className="text-xs tracking-wider text-gold-400">公民储备委员会</div>
          </div>
        </Link>

        <nav className="hidden items-center gap-1 lg:flex">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`rounded-lg px-3 py-2 text-sm font-medium no-underline transition-colors ${
                location.pathname === item.path
                  ? 'bg-white/10 text-gold-400'
                  : 'text-slate-300 hover:bg-white/5 hover:text-white'
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className="flex h-10 w-10 items-center justify-center rounded-lg text-slate-300 hover:bg-white/10 lg:hidden"
        >
          <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            {mobileOpen ? (
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
            ) : (
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
            )}
          </svg>
        </button>
      </div>

      {mobileOpen && (
        <nav className="border-t border-white/10 bg-navy-950/95 px-6 py-4 backdrop-blur-xl lg:hidden">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              onClick={() => setMobileOpen(false)}
              className={`block rounded-lg px-4 py-3 text-sm font-medium no-underline ${
                location.pathname === item.path
                  ? 'bg-white/10 text-gold-400'
                  : 'text-slate-300 hover:bg-white/5'
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>
      )}
    </header>
  )
}
