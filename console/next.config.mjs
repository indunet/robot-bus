/** @type {import('next').NextConfig} */
const brokerOrigin = process.env.ROBOT_BUS_BROKER_URL ?? 'http://127.0.0.1:15771'

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export → console/out/, synced into assets/console for rust-embed.
  output: 'export',
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

// Rewrites are not applied to `output: 'export'` production builds.
// Under `next dev` they proxy /api to a running broker.
if (process.env.NODE_ENV !== 'production') {
  nextConfig.rewrites = async () => [
    {
      source: '/api/:path*',
      destination: `${brokerOrigin}/api/:path*`,
    },
  ]
}

export default nextConfig
