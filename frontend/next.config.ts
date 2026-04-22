import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Disable Turbopack — use Webpack instead.
  // Turbopack's PostCSS worker pool leaks memory on Windows,
  // causing "Fatal process out of memory" OOM crashes (OS error 1450).
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: "http://127.0.0.1:8080/api/:path*", // Proxy to Backend
      },
    ];
  },
  webpack: (config) => {
    // Limit parallelism to prevent resource exhaustion on Windows
    config.parallelism = 20;
    return config;
  },
};

export default nextConfig;
