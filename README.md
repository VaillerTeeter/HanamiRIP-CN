# HanamiRIP-CN

HanamiRIP-CN 是基于 **Tauri + Vue 3 + Vite + Naive UI** 的桌面应用，提供番剧查询、追番管理、搜索聚合、下载与轨道混流能力。

## 功能概览

- Bangumi 番剧查询与详情
- 追番状态管理（在追/补番/完结）
- 搜索聚合（Nyaa 抓取解析）
- Torrent 下载与队列管理
- 轨道解析与混流（ffprobe/mkvmerge）

## 快速开始

```bash
# 一键准备环境（Windows）
yarn setup:env

# 启动调试（Tauri）
yarn debug
```

## 常用命令

- 开发（前端）
	- `yarn dev`
- 调试（Tauri）
	- `yarn debug`
- 清理构建/打包产物
	- `yarn clean`
- 一键准备环境（Windows）
	- `yarn setup:env`
- 构建 Windows 产物
	- `yarn build:windows`
- 打包 Windows 安装包
	- `yarn package:windows`
- 构建百度翻译 DLL
	- `yarn build:baidu-dll`

## 文档

- [docs/API.md](docs/API.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)

## 目录结构

```
apps/desktop   # Tauri + Vue 桌面应用
crates         # Rust 动态库（baidu_verify）
docs           # 项目文档
scripts        # 开发脚本
dist           # 打包产物
```

## 更新记录
- 2026-02-06：初始第一版。
