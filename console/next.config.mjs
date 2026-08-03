import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const tsSdkRoot = path.resolve(__dirname, '../bindings/typescript')

/** @type {import('next').NextConfig} */
const brokerOrigin = process.env.ROBOT_BUS_BROKER_URL ?? 'http://127.0.0.1:15771'

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export → console/out/, synced into assets/console for rust-embed.
  output: 'export',
  // Allow importing generated protobuf stubs from ../bindings/typescript.
  experimental: {
    externalDir: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    unoptimized: true,
  },
  webpack: (config) => {
    config.resolve.alias = {
      ...config.resolve.alias,
      // Browser entry of the in-repo TypeScript SDK (gRPC-Web).
      'robot-bus$': path.join(tsSdkRoot, 'dist/index.browser.js'),
      'robot-bus': path.join(tsSdkRoot, 'generated'),
    }
    // generated stubs are .ts but SDK consumers import with .js (NodeNext style).
    config.resolve.extensionAlias = {
      ...(config.resolve.extensionAlias || {}),
      '.js': ['.ts', '.tsx', '.js', '.jsx'],
    }
    config.resolve.modules = [
      ...(config.resolve.modules || ['node_modules']),
      path.join(tsSdkRoot, 'node_modules'),
    ]
    return config
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
