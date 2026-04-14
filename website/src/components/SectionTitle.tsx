interface SectionTitleProps {
  subtitle: string
  title: string
  description?: string
}

export default function SectionTitle({ subtitle, title, description }: SectionTitleProps) {
  return (
    <div className="mx-auto mb-16 max-w-3xl text-center">
      <span className="mb-4 inline-block rounded-full border border-gold-500/30 bg-gold-500/10 px-4 py-1.5 text-xs font-semibold uppercase tracking-widest text-gold-400">
        {subtitle}
      </span>
      <h2 className="mt-4 text-3xl font-bold tracking-tight text-white md:text-4xl lg:text-5xl">
        {title}
      </h2>
      {description && (
        <p className="mt-6 text-lg leading-relaxed text-slate-400">{description}</p>
      )}
    </div>
  )
}
