'use client'

import { useMemo, useState, type ReactNode } from 'react'
import Markdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import { BookOpen } from 'lucide-react'
import { PanelHeader } from './BrokerOverview'
import { useI18n, type Locale } from '@/lib/i18n'
import { bundledDocs } from '@/lib/bundled-docs.generated'
import { rewriteDocHref, GITHUB_BLOB } from '@/lib/docs-links'

const remarkPlugins = [remarkGfm]
const rehypePlugins = [rehypeHighlight]

function displayTitle(title: string): string {
  return title.replace(/`/g, '')
}

function flattenText(node: ReactNode): string {
  if (node == null || typeof node === 'boolean') return ''
  if (typeof node === 'string' || typeof node === 'number') return String(node)
  if (Array.isArray(node)) return node.map(flattenText).join('')
  if (typeof node === 'object' && node !== null && 'props' in node) {
    return flattenText((node as { props: { children?: ReactNode } }).props.children)
  }
  return ''
}

function headingId(children: ReactNode): string {
  return flattenText(children)
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, '-')
    .replace(/^-|-$/g, '')
}

export default function DocsView() {
  const { t, locale, setLocale } = useI18n()
  const [active, setActive] = useState(bundledDocs[0]?.slug ?? 'rust-api')

  const current = bundledDocs.find((d) => d.slug === active) ?? bundledDocs[0]
  const markdown = locale === 'zh' ? current?.zh ?? '' : current?.en ?? ''
  const headerTitle = displayTitle(
    locale === 'zh' ? current?.titleZh ?? t('docsTitle') : current?.titleEn ?? t('docsTitle'),
  )

  const knownSlugs = useMemo(() => new Set(bundledDocs.map((d) => d.slug)), [])

  const goDoc = (slug: string, nextLocale?: Locale) => {
    if (!knownSlugs.has(slug)) return
    setActive(slug)
    if (nextLocale && nextLocale !== locale) setLocale(nextLocale)
  }

  return (
    <div className="flex flex-1 min-h-0 gap-3">
      <aside className="w-52 shrink-0 flex flex-col rounded border border-bus-border bg-bus-panel overflow-hidden">
        <PanelHeader icon={<BookOpen size={14} />} title={t('docsToc')} />
        <nav className="flex-1 overflow-y-auto py-1 bus-scroll">
          {bundledDocs.map((doc) => {
            const label = displayTitle(locale === 'zh' ? doc.titleZh : doc.titleEn)
            const selected = doc.slug === current?.slug
            return (
              <button
                key={doc.slug}
                type="button"
                title={label}
                onClick={() => setActive(doc.slug)}
                className={`w-full text-left px-3 py-2 font-mono text-[11px] tracking-wide transition-colors truncate ${
                  selected
                    ? 'bg-bus-cyan/15 text-bus-cyan border-l-2 border-bus-cyan'
                    : 'text-bus-muted hover:text-bus-text hover:bg-[#1f2428] border-l-2 border-transparent'
                }`}
              >
                {label}
              </button>
            )
          })}
        </nav>
      </aside>

      <section className="flex-1 min-w-0 flex flex-col rounded border border-bus-border bg-bus-panel overflow-hidden">
        <PanelHeader title={headerTitle} />
        <div className="flex-1 overflow-y-auto overflow-x-hidden bus-scroll bg-[#0d1117]">
          <article className="markdown-body docs-md">
            <Markdown
              remarkPlugins={remarkPlugins}
              rehypePlugins={rehypePlugins}
              components={{
                a: ({ href, children }) => {
                  const link = rewriteDocHref(href, locale)
                  if (link.kind === 'doc') {
                    if (knownSlugs.has(link.slug)) {
                      return (
                        <a
                          href={`#${link.slug}`}
                          onClick={(event) => {
                            event.preventDefault()
                            goDoc(link.slug, link.locale)
                          }}
                        >
                          {children}
                        </a>
                      )
                    }
                    const loc = link.locale ?? locale
                    return (
                      <a
                        href={`${GITHUB_BLOB}/docs/${loc}/${link.slug}.md${link.hash}`}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {children}
                      </a>
                    )
                  }
                  if (link.kind === 'hash') {
                    return <a href={link.href}>{children}</a>
                  }
                  const external = /^https?:/i.test(link.href)
                  return (
                    <a
                      href={link.href}
                      {...(external ? { target: '_blank', rel: 'noreferrer' } : {})}
                    >
                      {children}
                    </a>
                  )
                },
                h1: ({ children }) => <h1 id={headingId(children)}>{children}</h1>,
                h2: ({ children }) => <h2 id={headingId(children)}>{children}</h2>,
                h3: ({ children }) => <h3 id={headingId(children)}>{children}</h3>,
                h4: ({ children }) => <h4 id={headingId(children)}>{children}</h4>,
                table: ({ children }) => (
                  <div className="docs-md-table-wrap bus-scroll">
                    <table>{children}</table>
                  </div>
                ),
              }}
            >
              {markdown}
            </Markdown>
          </article>
        </div>
      </section>
    </div>
  )
}
