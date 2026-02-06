# package.json 注释说明

> 说明：package.json 为标准 JSON，不支持注释，因此注释放在此旁路文档中。

## 顶层字段
- name：项目名称。
- private：是否禁止发布到 npm（true 表示仅本地使用）。
- version：项目版本号（语义化版本）。
- type：模块类型（module 表示使用 ES Modules 语法）。

## scripts
- predev：开发前执行脚本（展示 banner）。
- dev：启动前端开发服务器（Vite）。
- clean：清理构建产物与打包输出（可选清理 node_modules）。
- setup:env：一键准备 Windows 开发环境。
- debug：启动 Tauri 开发模式并指定输出目录。
- build:baidu-dll：编译百度翻译动态库并复制到 dist 目录。
- build:windows:x64：构建 Windows x64 安装包（NSIS）。
- build:windows:x86：构建 Windows x86 安装包（NSIS）。
- build:windows：先执行 banner，再分别构建 x64/x86。
- package:windows:x64：拷贝 x64 安装包到 dist 目录。
- package:windows:x86：拷贝 x86 安装包到 dist 目录。
- package:windows：先执行 banner，再分别打包 x64/x86。

## dependencies
- @tauri-apps/api：Tauri 前端 API。
- @tauri-apps/plugin-dialog：对话框插件。
- @tauri-apps/plugin-fs：文件系统插件。
- naive-ui：UI 组件库。
- vue：前端框架。

## devDependencies
- @types/node：Node.js 类型定义。
- @tauri-apps/cli：Tauri 命令行工具。
- @vitejs/plugin-vue：Vite 的 Vue 插件。
- typescript：TypeScript 编译器。
- vite：前端构建工具。

## 更新记录
- 2026-02-06：初始第一版。
