import type { Locale } from '@/lib/i18n'

export const GITHUB_BLOB = 'https://github.com/indunet/robot-bus/blob/main'

export type DocLink =
  | { kind: 'doc'; slug: string; locale?: Locale; hash: string }
  | { kind: 'hash'; href: string }
  | { kind: 'external'; href: string }

function splitHash(href: string): { path: string; hash: string } {
  const i = href.indexOf('#')
  if (i < 0) return { path: href, hash: '' }
  return { path: href.slice(0, i), hash: href.slice(i) }
}

function normalizePosix(path: string): string {
  const parts: string[] = []
  for (const seg of path.split('/')) {
    if (!seg || seg === '.') continue
    if (seg === '..') {
      parts.pop()
      continue
    }
    parts.push(seg)
  }
  return parts.join('/')
}

/** `../en/rust-api.md`, `./cpp-api.md`, `ros2-bridge.md` */
export function parseDocMdLink(href: string): { slug: string; locale?: Locale } | null {
  const { path } = splitHash(href)
  const cross = path.match(/^\.\.\/(zh|en)\/([a-z0-9-]+)\.md$/i)
  if (cross) return { locale: cross[1] as Locale, slug: cross[2] }
  const local = path.match(/^(?:\.\/)?([a-z0-9-]+)\.md$/i)
  if (local) return { slug: local[1] }
  return null
}

export function rewriteDocHref(href: string | undefined, locale: Locale): DocLink {
  if (!href) return { kind: 'external', href: '#' }
  if (/^(https?:|mailto:|tel:)/i.test(href)) return { kind: 'external', href }
  if (href.startsWith('#')) return { kind: 'hash', href }

  const md = parseDocMdLink(href)
  if (md) {
    const { hash } = splitHash(href)
    return { kind: 'doc', slug: md.slug, locale: md.locale, hash }
  }

  const { path, hash } = splitHash(href)
  const resolved = normalizePosix(`docs/${locale}/${path}`)
  if (!resolved || resolved.startsWith('..')) return { kind: 'external', href }
  return { kind: 'external', href: `${GITHUB_BLOB}/${resolved}${hash}` }
}
