import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { fileURLToPath } from "node:url";

const devHost = process.env.TAURI_DEV_HOST;

export default defineConfig({
  root: "ui",
  plugins: [
    paraglideVitePlugin({
      project: fileURLToPath(new URL("./project.inlang", import.meta.url)),
      outdir: fileURLToPath(new URL("./ui/src/lib/paraglide", import.meta.url)),
      strategy: ["globalVariable", "baseLocale"],
      emitTsDeclarations: true,
    }),
    svelte(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./ui/src/lib", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: devHost || false,
    hmr: devHost ? { protocol: "ws", host: devHost, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    outDir: "dist",
    // ! The 29 locales are compiled into the bundle, read from disk, not the network.
    chunkSizeWarningLimit: 2000,
    emptyOutDir: true,
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
});
