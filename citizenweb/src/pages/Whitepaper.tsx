import { useMemo, useState } from 'react'
import DOMPurify from 'dompurify'
import { marked } from 'marked'
import whitepaperMarkdown from '../whitepaper.md?raw'

// 白皮书正文归入 citizenweb/src/whitepaper.md；图片仍通过 Vite glob 打包 docs/assets/*，
// 渲染前把 markdown 里的 ./assets/<file> 替换成打包 URL（保证线上能加载，
// 同时保留现有白皮书图片资源目录）。
const whitepaperAssetUrls = import.meta.glob('../../../docs/assets/*', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>

const assetUrlByName = new Map<string, string>(
  Object.entries(whitepaperAssetUrls).map(([path, url]) => [path.split('/').pop() ?? path, url]),
)

type Heading = {
  id: string
  level: number
  title: string
  subtitle: string
}

type TocNode = Heading & {
  children: TocNode[]
}

const headingSubtitlePattern = /<span class="(whitepaper-title-en|whitepaper-heading-en)">([^<]+)<\/span>/

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

function cleanMarkdownText(value: string) {
  return value
    .replaceAll('**', '')
    .replaceAll('__', '')
    .replace(/<[^>]+>/g, '')
    .trim()
}

function parseHeadingContent(value: string) {
  const subtitleMatch = headingSubtitlePattern.exec(value)
  const titleSource = value
    .replace(/<br\s*\/?>\s*<span class="(?:whitepaper-title-en|whitepaper-heading-en)">[^<]+<\/span>/i, '')
    .replace(headingSubtitlePattern, '')

  return {
    title: cleanMarkdownText(titleSource),
    subtitleClass: subtitleMatch?.[1] ?? '',
    subtitle: subtitleMatch?.[2]?.trim() ?? '',
  }
}

function createHeadingId(title: string, usedIds: Map<string, number>) {
  const base = title
    .toLowerCase()
    .replace(/[^\p{Letter}\p{Number}]+/gu, '-')
    .replace(/^-+|-+$/g, '') || 'section'
  const usedCount = usedIds.get(base) ?? 0
  usedIds.set(base, usedCount + 1)
  return usedCount === 0 ? base : `${base}-${usedCount + 1}`
}

function removeSourceTableOfContents(markdown: string) {
  return markdown.replace(/\n# 目录(?:<br\s*\/?><span class="whitepaper-heading-en">Table of Contents<\/span>)?\n[\s\S]*?\n\*{4}\n/, '\n')
}

function extractHeadings(markdown: string) {
  const usedIds = new Map<string, number>()
  const lines = markdown.split('\n')
  const headings: Heading[] = []

  for (let index = 0; index < lines.length; index += 1) {
    const match = /^(#{1,3})\s+(.+)$/.exec(lines[index])
    if (!match) {
      continue
    }

    const { title, subtitle } = parseHeadingContent(match[2])
    headings.push({
      id: createHeadingId(title, usedIds),
      level: match[1].length,
      title,
      subtitle,
    })
  }

  return headings
}

function buildTocTree(headings: Heading[]) {
  const roots: TocNode[] = []
  const stack: TocNode[] = []

  for (const heading of headings) {
    if (heading.title.includes('公民链白皮书')) {
      continue
    }

    const node: TocNode = { ...heading, children: [] }

    while (stack.length > 0 && stack[stack.length - 1].level >= node.level) {
      stack.pop()
    }

    const parent = stack[stack.length - 1]
    if (parent) {
      parent.children.push(node)
    } else {
      roots.push(node)
    }

    stack.push(node)
  }

  return roots
}

function addHeadingIds(markdown: string, headings: Heading[]) {
  let headingIndex = 0
  const lines = markdown.split('\n')
  const result: string[] = []

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]
    const match = /^(#{1,3})\s+(.+)$/.exec(line)
    if (!match) {
      result.push(line)
      continue
    }

    const heading = headings[headingIndex]
    headingIndex += 1

    if (!heading) {
      result.push(line)
      continue
    }

    const { subtitleClass, subtitle } = parseHeadingContent(match[2])
    const subtitleHtml = subtitleClass && subtitle
      ? `<span class="${subtitleClass}">${escapeHtml(subtitle)}</span>`
      : ''

    result.push([
      `<h${heading.level} id="${escapeHtml(heading.id)}">`,
      `<span class="whitepaper-heading-title">${escapeHtml(heading.title)}</span>`,
      subtitleHtml,
      `</h${heading.level}>`,
    ].join(''))

  }

  return result.join('\n')
}

function splitTableRow(row: string) {
  return row
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((cell) => cell.trim())
}

function renderTableCodeBlock(code: string) {
  const rows = code
    .trim()
    .split('\n')
    .filter((row) => row.trim().startsWith('|') && row.includes('|'))

  if (rows.length < 2) {
    return null
  }

  const header = splitTableRow(rows[0])
  const divider = splitTableRow(rows[1])
  const isDivider = divider.length === header.length
    && divider.every((cell) => /^:?-{3,}:?$/.test(cell))

  if (!isDivider) {
    return null
  }

  const bodyRows = rows.slice(2).map(splitTableRow)
  const headerHtml = header
    .map((cell) => `<th>${escapeHtml(cell)}</th>`)
    .join('')
  const bodyHtml = bodyRows
    .map((row) => `<tr>${row.map((cell) => `<td>${escapeHtml(cell)}</td>`).join('')}</tr>`)
    .join('')

  return [
    '<div class="whitepaper-table-code">',
    '<table>',
    `<thead><tr>${headerHtml}</tr></thead>`,
    `<tbody>${bodyHtml}</tbody>`,
    '</table>',
    '</div>',
  ].join('')
}

function renderCodeTables(markdown: string) {
  return markdown.replace(/```([\s\S]*?)```/g, (block, code: string) => {
    const tableHtml = renderTableCodeBlock(code)
    return tableHtml ?? block
  })
}

function resolveAssetUrls(markdown: string) {
  return markdown.replace(
    /(["'(])(?:\.\/)?assets\/([A-Za-z0-9._-]+)/g,
    (whole, lead: string, file: string) => {
      const url = assetUrlByName.get(file)
      return url ? `${lead}${url}` : whole
    },
  )
}

function renderWhitepaper(markdown: string, headings: Heading[]) {
  const markdownWithIds = addHeadingIds(markdown, headings)
  const markdownWithTables = renderCodeTables(markdownWithIds)
  const markdownWithAssets = resolveAssetUrls(markdownWithTables)
  const html = marked.parse(markdownWithAssets, { async: false }) as string

  return DOMPurify.sanitize(html, {
    ADD_ATTR: ['class', 'id', 'width'],
    ADD_DATA_URI_TAGS: ['img'],
  })
}

function WhitepaperToc({
  items,
  openIds,
  onToggle,
}: {
  items: TocNode[]
  openIds: Set<string>
  onToggle: (id: string) => void
}) {
  return (
    <ul className="whitepaper-toc-tree">
      {items.map((item) => {
        const isOpen = openIds.has(item.id)
        const hasChildren = item.children.length > 0
        const content = (
          <>
            <span className="whitepaper-toc-caret" aria-hidden="true">
              {hasChildren ? (isOpen ? '-' : '+') : ''}
            </span>
            <span className="whitepaper-toc-text">
              <span>{item.title}</span>
              {item.subtitle && <small>{item.subtitle}</small>}
            </span>
          </>
        )

        return (
          <li key={item.id} className={`whitepaper-toc-item whitepaper-toc-level-${item.level}`}>
            {hasChildren ? (
              <>
                <button
                  type="button"
                  className="whitepaper-toc-trigger"
                  aria-expanded={isOpen}
                  aria-controls={`toc-children-${item.id}`}
                  onClick={() => onToggle(item.id)}
                >
                  {content}
                </button>
                {isOpen && (
                  <div id={`toc-children-${item.id}`}>
                    <WhitepaperToc items={item.children} openIds={openIds} onToggle={onToggle} />
                  </div>
                )}
              </>
            ) : (
              <a href={`#${item.id}`} className="whitepaper-toc-link">
                {content}
              </a>
            )}
          </li>
        )
      })}
    </ul>
  )
}

export default function Whitepaper() {
  const [openTocIds, setOpenTocIds] = useState<Set<string>>(() => new Set())
  const { html, tocTree } = useMemo(() => {
    const markdown = removeSourceTableOfContents(whitepaperMarkdown)
    const headings = extractHeadings(markdown)
    const renderedHtml = renderWhitepaper(markdown, headings)

    return {
      html: renderedHtml,
      tocTree: buildTocTree(headings),
    }
  }, [])
  const toggleTocItem = (id: string) => {
    setOpenTocIds((current) => {
      const next = new Set(current)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }
  const scrollToTop = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  return (
    <section className="whitepaper-page min-h-screen bg-navy-950">
      <div className="whitepaper-shell mx-auto grid max-w-7xl gap-8 px-6 py-12 lg:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="whitepaper-toc lg:sticky lg:top-24 lg:self-start">
          <div className="whitepaper-toc-title">
            <span>目录</span>
            <span>Table of Contents</span>
          </div>
          <nav className="whitepaper-toc-list" aria-label="白皮书目录">
            <WhitepaperToc items={tocTree} openIds={openTocIds} onToggle={toggleTocItem} />
          </nav>
        </aside>

        <article
          className="whitepaper-body"
          dangerouslySetInnerHTML={{ __html: html }}
        />
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
