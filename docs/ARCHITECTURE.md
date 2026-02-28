# 架构说明

本文档描述 HanamiRIP-CN 的代码组织、运行时边界和关键数据流。

## 1. 总体架构

项目采用 Monorepo 结构，桌面应用由两部分组成：

- 前端：`apps/desktop/frontend`（Vue 3 + TypeScript + Arco）
- 后端：`apps/desktop/backend`（Rust + Tauri Command）

前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用后端命令，后端负责：
- 外部 API 聚合（Bangumi）
- 本地数据持久化（追番状态、下载任务）
- 系统能力（文件、对话框、外链、窗口）
- 外部工具调用（FFmpeg / MKVToolNix）

## 2. 目录与职责

```text
apps/desktop/backend/src/
├─ main.rs                 # Tauri 启动与命令注册
└─ services/
   ├─ bangumi/             # Bangumi 数据与翻译
   ├─ torrent/             # Torrent 下载任务生命周期
   ├─ media/               # 轨道解析与混流
   ├─ storage/             # 追番本地存储
   └─ external/            # 外链打开

apps/desktop/frontend/
├─ modules/
│  ├─ query/               # 番剧详情查询
│  ├─ search/              # 搜索聚合
│  ├─ tracking/            # 追番状态管理
│  ├─ download/            # 下载任务页
│  └─ tracks/              # 混流页
└─ shared/
   ├─ components/          # 通用组件（含标题栏）
   ├─ composables/         # 通用逻辑
   ├─ i18n/                # 文案与国际化
   └─ utils/               # 工具函数
```

## 3. 运行时数据流

### 3.1 番剧查询流

1. 前端模块发起 `invoke("get_season_subjects" | "get_subject_*")`。
2. 后端 `services/bangumi` 请求 Bangumi API 并整理模型。
3. 可选调用百度翻译逻辑生成中文简介。
4. 返回 JSON 给前端渲染。

### 3.2 下载任务流

1. 前端调用 `get_torrent_metadata` 获取种子基础信息。
2. 前端确认输出路径后调用 `start_torrent_download`。
3. 后端创建 `.downloading` 临时目录和占位文件，落盘任务记录。
4. 前端轮询 `get_torrent_status` 更新进度。
5. 完成后调用 `finalize_torrent_download` 移动产物并清理临时目录。

### 3.3 轨道混流流

1. 前端调用 `parse_media_tracks(path, kind)` 拉取轨道。
2. 用户选择轨道后调用 `mix_media_tracks(inputs, output_path)`。
3. 后端调用 `mkvmerge` 执行混流，返回输出路径。

## 4. 状态与持久化

- 追番数据：`services/storage` 以 JSON 文件持久化。
- 下载任务：`services/torrent` 维护任务文件，支持恢复与清理。
- 应用数据目录：由 `main.rs` 在启动时保证可用（含 `user-data` 链接处理）。

## 5. 安全与权限边界

- 前端不能直接访问系统 API，必须经 Tauri command。
- 能力权限由 `tauri.conf.json` 的 `app.security.capabilities` 显式声明。
- 当前 `csp: null` 更偏开发友好，发布阶段可视需求收紧。

## 6. 构建与发布链路

- 前端构建：Vite 输出到 `build/frontend`
- 后端构建：Cargo 输出到 `build/tauri-target`
- 打包目标：NSIS（x64/x86）
- 发布整理：`package:windows:*` 脚本复制到 `dist/windows`