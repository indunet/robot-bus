/** @type {import('next').NextConfig} */
const brokerOrigin = process.env.ROBOT_BUS_BROKER_URL ?? 'http://127.0.0.1:15560'
const isProd = process.env.NODE_ENV === 'production'

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export only for `pnpm build` → assets/console / rust-embed.
  // `next dev` must NOT set output:'export' or rewrites are ignored.
  ...(isProd ? { output: 'export' } : {}),
  // The local SDK package contains generated protobuf TypeScript sources.
  transpilePackages: ['robot-bus'],
  typescript: {
    ignoreBuildErrors: true,
  },
  webpack(config) {
    // The workspace maps robot-bus to TypeScript sources whose ESM imports use
    // emitted `.js` suffixes. Resolve those suffixes back to source files.
    config.resolve.extensionAlias = {
      ...config.resolve.extensionAlias,
      '.js': ['.ts', '.tsx', '.js'],
      '.mjs': ['.mts', '.mjs'],
    }
    return config
  },
  images: {
    unoptimized: true,
  },
}

// Under `pnpm dev`, proxy REST to the broker. Browser WebSocket RPC should talk
// to the broker directly (`resolveBusUrl` → :15560/ws-rpc) — Next rewrites are unreliable for WS.
if (!isProd) {
  nextConfig.rewrites = async () => ({
    beforeFiles: [
      {
        source: '/api/:path*',
        destination: `${brokerOrigin}/api/:path*`,
      },
    ],
  })
}

export default nextConfig
