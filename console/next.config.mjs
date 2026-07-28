/** @type {import('next').NextConfig} */
const brokerOrigin = process.env.ROBOT_BUS_BROKER_URL ?? 'http://127.0.0.1:15771'

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export → console/out/, synced into assets/console for rust-embed.
  output: 'export',
  typescript: {
    ignoreBuildErrors: true,
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
