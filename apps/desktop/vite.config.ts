/** 文件：vite.config.ts | 用途：定义桌面前端 Vite 构建与开发服务器配置 | 关键对象：__dirname、defineConfig、build、server */
// 引入 Node.js 类型声明，保证本文件中 Node API 的类型可用。
/// <reference types="node" />
// 引入 Vite 配置工厂函数，用于获得类型安全的配置对象。
import { defineConfig } from "vite";
// 引入 Vue 插件，让 Vite 能处理 .vue 单文件组件。
import vue from "@vitejs/plugin-vue";
// 引入路径工具，用于构造跨平台绝对路径。
import { resolve, dirname } from "path";
// 引入 URL 转文件路径工具，用于在 ESM 环境下获取当前目录。
import { fileURLToPath } from "url";

// 变量：__dirname | 含义：当前配置文件所在目录的绝对路径 | 类型：string | 作用域：vite.config 模块
const __dirname = dirname(fileURLToPath(import.meta.url));

// 导出 Vite 配置对象，供开发与构建命令统一使用。
export default defineConfig({
    // 指定前端源代码根目录。
    root: resolve(__dirname, "frontend"),
    // 指定静态资源目录（字体、图标、工具等）。
    publicDir: resolve(__dirname, "public"),
    // 注册 Vue 插件。
    plugins: [vue()],
    // 使用相对基础路径，便于桌面应用从本地文件加载资源。
    base: "./",

    // 构建相关选项。
    build: {
        // 指定构建输出目录到仓库的 build/frontend。
        outDir: resolve(__dirname, "../..", "build/frontend"),
        // 构建前清空输出目录，避免旧产物残留。
        emptyOutDir: true,
        // 提高 chunk 体积警告阈值，减少大型桌面包场景的噪声警告。
        chunkSizeWarningLimit: 2000,
    },

    // 开发服务器选项。
    server: {
        // 固定开发端口，便于与 Tauri 侧配置保持一致。
        port: 1420,
        // 端口被占用时直接报错，不自动换端口，避免联调混乱。
        strictPort: true,
    },
});
