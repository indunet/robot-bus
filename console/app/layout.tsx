import type { Metadata, Viewport } from 'next'
import localFont from 'next/font/local'
import Providers from './providers'
import './globals.css'

// Self-hosted (next/font/local) so `next build` does not fetch fonts.gstatic.com —
// CI / locked-down networks often fail with next/font/google.
const inter = localFont({
  src: '../fonts/inter-latin-wght-normal.woff2',
  variable: '--font-inter',
  weight: '100 900',
  display: 'swap',
})
const jetbrainsMono = localFont({
  src: '../fonts/jetbrains-mono-latin-wght-normal.woff2',
  variable: '--font-mono',
  weight: '100 900',
  display: 'swap',
})
const orbitron = localFont({
  src: [
    {
      path: '../fonts/orbitron-latin-600-normal.woff2',
      weight: '600',
      style: 'normal',
    },
    {
      path: '../fonts/orbitron-latin-700-normal.woff2',
      weight: '700',
      style: 'normal',
    },
  ],
  variable: '--font-brand',
  display: 'swap',
})

export const metadata: Metadata = {
  title: 'robot bus',
  description: 'robot-bus broker monitor — status, topic traffic, and event logs',
}

export const viewport: Viewport = {
  colorScheme: 'dark',
  themeColor: '#141618',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html
      lang="en"
      className={`${inter.variable} ${jetbrainsMono.variable} ${orbitron.variable} bg-bus-bg`}
    >
      <body className="antialiased font-sans">
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}
