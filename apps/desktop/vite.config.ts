/// <reference types="node" />
// Vite 配置文件：为桌面应用指定根目录、构建输出与开发端口。
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

// 获取当前文件所在目录（ESM 方式）。
const __dirname = dirname(fileURLToPath(import.meta.url));

// 导出 Vite 配置。
export default defineConfig({
  // 前端源码根目录（相对 apps/desktop 目录）。
  root: resolve(__dirname, "frontend"),
  // 静态资源目录。
  publicDir: resolve(__dirname, "public"),
  // Vue 插件。
  plugins: [vue()],
  // 使用相对路径，便于打包后的文件加载。
  base: "./",
  build: {
    // 输出目录指向统一的 build/frontend。
    outDir: resolve(__dirname, "../..", "build/frontend"),
    // 构建前清空输出目录。
    emptyOutDir: true,
    // 调高 chunk 报警阈值，减少无意义警告。
    chunkSizeWarningLimit: 2000
  },
  server: {
    // 开发服务器端口（与 Tauri devUrl 保持一致）。
    port: 1420,
    // 端口被占用时直接报错，避免自动换端口导致 Tauri 加载失败。
    strictPort: true
  }
});
