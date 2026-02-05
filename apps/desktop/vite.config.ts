/// <reference types="node" />
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root: resolve(__dirname, "frontend"),
  publicDir: resolve(__dirname, "public"),
  plugins: [vue()],
  base: "./",
  build: {
    outDir: resolve(__dirname, "../..", "build/frontend"),
    emptyOutDir: true,
    chunkSizeWarningLimit: 2000
  },
  server: {
    port: 1420,
    strictPort: true
  }
});
