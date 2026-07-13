import { useEffect, useState } from 'react'

/// 官网「公民宪法」tab：经 Cloudflare Worker 读链上唯一真源宪法（GET /v1/constitution），
/// 用白皮书样式自渲染（左目录树 + 右正文 + 不可修改徽章 + 版本标签 + 中英双语）。
/// 结构化 JSON 直接 JSX 渲染，不用 dangerouslySetInnerHTML。修宪后一个缓存 TTL 内自动更新。

const apiBaseUrl = import.meta.env.VITE_API_URL?.replace(/\/+$/, '') ?? '/api'

interface ConstitutionClause {
  text_cn: string
  text_en: string | null
}

interface ConstitutionArticle {
  number: number
  title_cn: string
  title_en: string | null
  body_cn: string
  body_en: string | null
  immutable: boolean
  clauses: ConstitutionClause[]
}

interface ConstitutionSection {
  number: number
  title_cn: string
  title_en: string | null
  articles: ConstitutionArticle[]
}

interface ConstitutionChapter {
  number: number
  title_cn: string
  title_en: string | null
  sections: ConstitutionSection[]
}

interface ConstitutionDocument {
  version: number
  content_hash: string
  version_label: { cn: string; en: string | null } | null
  immutable_articles: number[]
  chapters: ConstitutionChapter[]
}

const chapterAnchor = (chapter: number) => `chapter-${chapter}`
const sectionAnchor = (chapter: number, section: number) => `chapter-${chapter}-section-${section}`
const articleAnchor = (article: number) => `article-${article}`

/** 中英并列的标题：中文主行 + 英文副行（复用白皮书双语样式类）。 */
function BilingualHeading({ cn, en }: { cn: string; en: string | null }) {
  return (
    <>
      <span className="whitepaper-heading-title">{cn}</span>
      {en && <span className="whitepaper-heading-en">{en}</span>}
    </>
  )
}

/** 一段正文：中文 + 英文（英文缩进副行）。 */
function BilingualParagraph({ cn, en }: { cn: string; en: string | null }) {
  return (
    <p>
      <span className="whitepaper-heading-title">{cn}</span>
      {en && <span className="whitepaper-en">{en}</span>}
    </p>
  )
}

interface TocProps {
  chapters: ConstitutionChapter[]
  openKeys: Set<string>
  onToggle: (key: string) => void
}

function ConstitutionToc({ chapters, openKeys, onToggle }: TocProps) {
  return (
    <ul className="whitepaper-toc-tree">
      {chapters.map((chapter) => {
        const chapterKey = `c${chapter.number}`
        const chapterOpen = openKeys.has(chapterKey)
        return (
          <li key={chapterKey} className="whitepaper-toc-item whitepaper-toc-level-1">
            <button
              type="button"
              className="whitepaper-toc-trigger"
              aria-expanded={chapterOpen}
              onClick={() => onToggle(chapterKey)}
            >
              <span className="whitepaper-toc-caret" aria-hidden="true">
                {chapterOpen ? '-' : '+'}
              </span>
              <span className="whitepaper-toc-text">
                <span>{chapter.title_cn}</span>
                {chapter.title_en && <small>{chapter.title_en}</small>}
              </span>
            </button>
            {chapterOpen && (
              <ul className="whitepaper-toc-tree">
                {chapter.sections.map((section) => {
                  const sectionKey = `c${chapter.number}s${section.number}`
                  const sectionOpen = openKeys.has(sectionKey)
                  return (
                    <li key={sectionKey} className="whitepaper-toc-item whitepaper-toc-level-2">
                      <button
                        type="button"
                        className="whitepaper-toc-trigger"
                        aria-expanded={sectionOpen}
                        onClick={() => onToggle(sectionKey)}
                      >
                        <span className="whitepaper-toc-caret" aria-hidden="true">
                          {sectionOpen ? '-' : '+'}
                        </span>
                        <span className="whitepaper-toc-text">
                          <span>{section.title_cn}</span>
                          {section.title_en && <small>{section.title_en}</small>}
                        </span>
                      </button>
                      {sectionOpen && (
                        <ul className="whitepaper-toc-tree">
                          {section.articles.map((article) => (
                            <li
                              key={article.number}
                              className="whitepaper-toc-item whitepaper-toc-level-3"
                            >
                              <a
                                href={`#${articleAnchor(article.number)}`}
                                className="whitepaper-toc-link"
                              >
                                <span className="whitepaper-toc-caret" aria-hidden="true" />
                                <span className="whitepaper-toc-text">
                                  <span>{article.title_cn}</span>
                                  {article.title_en && <small>{article.title_en}</small>}
                                </span>
                              </a>
                            </li>
                          ))}
                        </ul>
                      )}
                    </li>
                  )
                })}
              </ul>
            )}
          </li>
        )
      })}
    </ul>
  )
}

function ConstitutionArticleBlock({ article }: { article: ConstitutionArticle }) {
  return (
    <article id={articleAnchor(article.number)} className="constitution-article">
      <h3>
        <span className="whitepaper-heading-title">
          {article.title_cn}
          {article.immutable && (
            <span className="constitution-immutable-badge">不可修改条款 · Immutable</span>
          )}
        </span>
        {article.title_en && <span className="whitepaper-heading-en">{article.title_en}</span>}
      </h3>
      <BilingualParagraph cn={article.body_cn} en={article.body_en} />
      {article.clauses.map((clause, index) => (
        <BilingualParagraph key={index} cn={clause.text_cn} en={clause.text_en} />
      ))}
    </article>
  )
}

export default function Constitution() {
  const [doc, setDoc] = useState<ConstitutionDocument | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [openKeys, setOpenKeys] = useState<Set<string>>(() => new Set())

  useEffect(() => {
    const controller = new AbortController()
    fetch(`${apiBaseUrl}/v1/constitution`, { signal: controller.signal })
      .then(async (response) => {
        const data = (await response.json().catch(() => ({}))) as Record<string, unknown>
        if (!response.ok) {
          throw new Error(typeof data.message === 'string' ? data.message : '宪法加载失败')
        }
        const document = data as unknown as ConstitutionDocument
        setDoc(document)
        // 首屏默认展开第一章目录，避免目录空荡。
        if (document.chapters.length > 0) {
          setOpenKeys(new Set([`c${document.chapters[0].number}`]))
        }
      })
      .catch((cause: unknown) => {
        if (cause instanceof DOMException && cause.name === 'AbortError') return
        setError(cause instanceof Error ? cause.message : '宪法加载失败')
      })
    return () => controller.abort()
  }, [])

  const toggleKey = (key: string) => {
    setOpenKeys((current) => {
      const next = new Set(current)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  }

  const scrollToTop = () => window.scrollTo({ top: 0, behavior: 'smooth' })

  if (error) {
    return (
      <section className="whitepaper-page constitution-page min-h-screen bg-navy-950">
        <div className="constitution-message">
          <p>公民宪法加载失败：{error}</p>
        </div>
      </section>
    )
  }

  if (!doc) {
    return (
      <section className="whitepaper-page constitution-page min-h-screen bg-navy-950">
        <div className="constitution-message">
          <p>公民宪法加载中...</p>
        </div>
      </section>
    )
  }

  const versionLabel = doc.version_label

  return (
    <section className="whitepaper-page constitution-page min-h-screen bg-navy-950">
      <div className="whitepaper-shell mx-auto grid max-w-7xl gap-8 px-6 py-12 lg:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="whitepaper-toc lg:sticky lg:top-24 lg:self-start">
          <div className="whitepaper-toc-title">
            <span>目录</span>
            <span>Table of Contents</span>
          </div>
          <nav className="whitepaper-toc-list" aria-label="公民宪法目录">
            <ConstitutionToc chapters={doc.chapters} openKeys={openKeys} onToggle={toggleKey} />
          </nav>
        </aside>

        <article className="whitepaper-body">
          <h1>
            <span className="whitepaper-heading-title">公民宪法</span>
            <span className="whitepaper-title-en">Citizen Constitution</span>
          </h1>
          <p className="constitution-version">
            <span>
              {versionLabel?.cn ?? `第 ${doc.version} 版`}
              <span className="constitution-version-no">第 {doc.version} 版</span>
            </span>
            {versionLabel?.en && <span className="whitepaper-en">{versionLabel.en}</span>}
          </p>

          {doc.chapters.map((chapter) => (
            <section
              key={chapter.number}
              id={chapterAnchor(chapter.number)}
              className="constitution-chapter"
            >
              <h1>
                <BilingualHeading cn={chapter.title_cn} en={chapter.title_en} />
              </h1>
              {chapter.sections.map((section) => (
                <section
                  key={section.number}
                  id={sectionAnchor(chapter.number, section.number)}
                  className="constitution-section"
                >
                  <h2>
                    <BilingualHeading cn={section.title_cn} en={section.title_en} />
                  </h2>
                  {section.articles.map((article) => (
                    <ConstitutionArticleBlock key={article.number} article={article} />
                  ))}
                </section>
              ))}
            </section>
          ))}

          <p className="constitution-hash" title="链上宪法内容摘要（blake2_256）">
            链上内容摘要 {doc.content_hash}
          </p>
        </article>
      </div>

      <button
        type="button"
        className="whitepaper-back-top"
        aria-label="回到顶部"
        title="回到顶部"
        onClick={scrollToTop}
      >
        ↑
      </button>
    </section>
  )
}
