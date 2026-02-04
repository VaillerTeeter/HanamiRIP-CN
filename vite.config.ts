import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  base: "./",
  build: {
    outDir: "build/frontend",
    chunkSizeWarningLimit: 2000
  },
  server: {
    port: 1420,
    strictPort: true
  }
});
