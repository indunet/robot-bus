# Self-hosted fonts (console)

Latin WOFF2 subsets used by `app/layout.tsx` via `next/font/local`.

Vendored so CI / offline builds do not call `fonts.gstatic.com` (see Next.js
`next/font/google` build-time fetch failures).

| File | Family | Source |
|------|--------|--------|
| `inter-latin-wght-normal.woff2` | Inter (variable) | [Fontsource](https://fontsource.org/fonts/inter) |
| `jetbrains-mono-latin-wght-normal.woff2` | JetBrains Mono (variable) | [Fontsource](https://fontsource.org/fonts/jetbrains-mono) |
| `orbitron-latin-600-normal.woff2` | Orbitron 600 | [Fontsource](https://fontsource.org/fonts/orbitron) |
| `orbitron-latin-700-normal.woff2` | Orbitron 700 | same |

Licenses: Inter (OFL), JetBrains Mono (OFL), Orbitron (OFL).
