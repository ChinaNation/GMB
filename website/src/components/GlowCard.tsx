import { type ReactNode } from 'react'

interface GlowCardProps {
  children: ReactNode
  className?: string
  glow?: 'gold' | 'blue' | 'none'
}

export default function GlowCard({ children, className = '', glow = 'none' }: GlowCardProps) {
  const glowStyles = {
    gold: 'hover:shadow-[0_0_40px_rgba(212,160,23,0.15)]',
    blue: 'hover:shadow-[0_0_40px_rgba(46,87,151,0.2)]',
    none: '',
  }

  return (
    <div
      className={`rounded-2xl border border-white/[0.08] bg-white/[0.03] p-8 backdrop-blur-sm transition-all duration-300 hover:border-white/[0.15] hover:bg-white/[0.05] ${glowStyles[glow]} ${className}`}
    >
      {children}
    </div>
  )
}
