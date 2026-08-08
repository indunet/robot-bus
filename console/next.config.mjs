/** @type {import('next').NextConfig} */
const brokerOrigin = process.env.ROBOT_BUS_BROKER_URL ?? 'http://127.0.0.1:15770'
const isProd = process.env.NODE_ENV === 'production'

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export only for `pnpm build` → assets/console / rust-embed.
  // `next dev` must NOT set output:'export' or rewrites are ignored (gRPC 404).
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

// Under `next dev`, proxy REST + gRPC-Web to the broker (same-origin browser calls).
// Use beforeFiles: dotted gRPC paths look like static files and skip afterFiles rewrites.
if (!isProd) {
  const grpcGateways = ['MessageGateway', 'ServiceGateway', 'ActionGateway']
  nextConfig.rewrites = async () => ({
    beforeFiles: [
      {
        source: '/api/:path*',
        destination: `${brokerOrigin}/api/:path*`,
      },
      ...grpcGateways.map((gateway) => ({
        source: `/robot_bus_interface.grpc.v1.${gateway}/:path*`,
        destination: `${brokerOrigin}/robot_bus_interface.grpc.v1.${gateway}/:path*`,
      })),
    ],
  })
}

export default nextConfig
