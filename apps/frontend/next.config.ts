import type { NextConfig } from "next";

const nextConfig: NextConfig = {
   /* config options here */
   // reactCompiler: true,
   images: {
      remotePatterns: [new URL("https://*.ytimg.com/vi/**")],
   },
};

export default nextConfig;
