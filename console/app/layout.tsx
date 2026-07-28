import type { Metadata, Viewport } from 'next'
import { Inter, JetBrains_Mono, Orbitron } from 'next/font/google'
import Providers from './providers'
import './globals.css'

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' })
const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
})
const orbitron = Orbitron({
  subsets: ['latin'],
  variable: '--font-brand',
  weight: ['600', '700'],
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
