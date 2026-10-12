import babelPlugin from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact, { reactCompilerPreset } from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";

const config = defineConfig({
   resolve: { tsconfigPaths: true },
   server: { port: 3000 },
   plugins: [
      devtools(),
      nitro({ rollupConfig: { external: [/^@sentry\//] } }),
      tailwindcss(),
      tanstackStart(),
      viteReact(),
      babelPlugin({
         presets: [reactCompilerPreset()],
      }),
   ],
});

export default config;
