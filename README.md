# HanamiRIP-CN

HanamiRIP-CN 是一个 Windows 桌面端番剧工具，基于 **Tauri 2 + Rust + Vue 3 + Vite + Arco Design Vue**，覆盖番剧检索、追番管理、下载与轨道混流。

## 核心能力

- 番剧数据：基于 Bangumi API 获取条目、别名、角色、Staff、季度番组。
- 追番管理：本地保存在追/补番/完结状态与观看进度。
- 下载管理：基于 `librqbit` 提供 Torrent 元数据读取、任务恢复、暂停/继续、状态查询。
- 轨道混流：调用 `mkvmerge` / `ffprobe` / `ffmpeg` 解析轨道并输出混流文件。
- 桌面集成：自绘标题栏、窗口控制、系统对话框、外链打开。

## 快速开始

### 1) 准备环境（Windows）

```bash
yarn setup:env
```

该命令会执行 `scripts/setup-env.ps1`，自动检查/安装：
- Node.js、Yarn、Rust、Rust targets（x64/x86）
- MSVC Build Tools（link.exe）
- NSIS 与 `nsis_tauri_utils.dll`
- FFmpeg、MKVToolNix 工具

### 2) 开发模式

```bash
# 仅前端（不含 Tauri 后端命令）
yarn dev

# 全栈（前端 + Tauri 后端）
yarn debug
```

### 3) 生产构建与打包

```bash
# 构建 x64 + x86
yarn build:windows

# 复制安装包到 dist/windows
yarn package:windows
```

## 常用命令

- `yarn setup:env`：初始化 Windows 开发环境
- `yarn dev`：启动 Vite 开发服务器（http://localhost:1420）
- `yarn debug`：启动 Tauri 开发模式
- `yarn clean`：清理 `build/` 与 `dist/`
- `yarn build:windows[:x64|:x86]`：构建安装程序
- `yarn package:windows[:x64|:x86]`：整理安装包到 `dist/windows`
- `yarn build:baidu-dll`：编译 `baidu_verify.dll`

## 项目结构

```text
apps/desktop/
├─ backend/    # Rust + Tauri 命令与服务
├─ frontend/   # Vue 3 前端模块
└─ public/     # 字体、图标、FFmpeg/MKVToolNix 等资源

crates/
└─ baidu_verify/  # 百度翻译校验动态库

docs/             # 项目文档
scripts/          # 环境初始化、清理、banner 脚本
```

## 文档入口

- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)：开发流程与排障
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)：模块边界与数据流
- [docs/API.md](docs/API.md)：前后端命令与 API 说明
- [package.json.md](package.json.md)：脚本与依赖说明
- [apps/desktop/backend/tauri.conf.json.md](apps/desktop/backend/tauri.conf.json.md)：Tauri 配置说明

## 平台说明

- 当前主要目标平台：**Windows 10/11**。
- 构建产物：NSIS 安装程序（x64/x86）。