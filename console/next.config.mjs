/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export → console/out/, embedded and served by robot_bus_broker (:15771).
  output: 'export',
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    unoptimized: true,
  },
}

export default nextConfig
