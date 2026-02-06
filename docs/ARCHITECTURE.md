
# 架构说明

## 总览

项目采用 **Monorepo** 结构：

- `.github/`：工程规范与注释说明
- `apps/desktop`：桌面应用（Tauri + Vue）
- `crates/baidu_verify`：百度翻译密钥校验动态库
- `docs/`：文档
- `scripts/`：开发脚本
- `dist/`：构建产物输出

## 运行时结构

- 前端（Vue + Naive UI）通过 Tauri JS API 调用后端命令。
- 后端（Rust/Tauri）负责：
	- Bangumi API 数据聚合
	- 本地追番数据保存
	- torrent 下载与状态查询
	- 媒体轨道解析与混流
	- 外部链接打开
- 工具链：`ffprobe/ffmpeg/mkvmerge/mkvinfo` 打包进应用资源目录。

## 项目树（非 .gitignore 忽略的所有文件）
```bash
.
├─ .github/
│  └─ instructions/ - 注释与格式规范
│     ├─ css-comments.instructions.md - CSS 注释规范
│     ├─ html-comments.instructions.md - HTML 注释规范
│     ├─ json-comments.instructions.md - JSON 注释规范
│     ├─ ps1-comments.instructions.md - PowerShell 注释规范
│     ├─ rust-comments.instructions.md - Rust 注释规范
│     ├─ toml-comments.instructions.md - TOML 注释规范
│     ├─ ts-comments.instructions.md - TypeScript 注释规范
│     └─ vue-comments.instructions.md - Vue 注释规范
├─ .gitignore - 代码忽略规则
├─ .node-version - Node.js 版本约束
├─ .vscode/
│  └─ settings.json - 工作区编辑器配置
├─ apps/
│  └─ desktop/
│     ├─ backend/
│     │  ├─ build.rs - Tauri 构建脚本（资源打包）
│     │  ├─ Cargo.lock - Rust 依赖锁定
│     │  ├─ Cargo.toml - 后端依赖与编译配置
│     │  ├─ tauri.conf.json - Tauri 应用配置
│     │  └─ src/
│     │     ├─ main.rs - Tauri 入口与命令注册
│     │     └─ services/
│     │        ├─ mod.rs - 服务模块聚合
│     │        ├─ bangumi/
│     │        │  ├─ api.rs - Bangumi API 聚合逻辑
│     │        │  ├─ commands.rs - Tauri 命令定义
│     │        │  ├─ filters.rs - 标签/过滤辅助逻辑
│     │        │  ├─ mod.rs - bangumi 模块入口
│     │        │  ├─ models.rs - Bangumi 数据模型
│     │        │  └─ translate.rs - 百度翻译与校验库加载
│     │        ├─ external/
│     │        │  └─ mod.rs - 外部链接打开命令
│     │        ├─ media/
│     │        │  └─ mod.rs - 轨道解析/混流逻辑
│     │        ├─ storage/
│     │        │  └─ mod.rs - 本地追番存储
│     │        └─ torrent/
│     │           └─ mod.rs - Torrent 下载逻辑
│     ├─ frontend/
│     │  ├─ App.vue - 应用根组件
│     │  ├─ env.d.ts - Vite 类型声明
│     │  ├─ index.html - 前端入口 HTML
│     │  ├─ main.ts - Vue 启动入口
│     │  ├─ style.css - 全局样式
│     │  ├─ modules/
│     │  │  ├─ download/
│     │  │  │  ├─ composables/useDownloadPage.ts - 下载页逻辑
│     │  │  │  ├─ pages/DownloadPage.vue - 下载页 UI
│     │  │  │  └─ types/download.ts - 下载类型定义
│     │  │  ├─ query/
│     │  │  │  ├─ components/StaffModal.vue - 人员弹窗组件
│     │  │  │  ├─ composables/useQueryPage.ts - 查询页逻辑
│     │  │  │  └─ pages/QueryPage.vue - 查询页 UI
│     │  │  ├─ search/
│     │  │  │  ├─ components/AliasModal.vue - 别名选择弹窗
│     │  │  │  ├─ composables/useSearchPage.ts - 搜索页逻辑
│     │  │  │  ├─ pages/SearchPage.vue - 搜索页 UI
│     │  │  │  └─ types/search.ts - 搜索类型定义
│     │  │  ├─ tracking/
│     │  │  │  ├─ composables/useTracking.ts - 追番状态逻辑
│     │  │  │  ├─ pages/BacklogPage.vue - 补番页 UI
│     │  │  │  ├─ pages/FinishedPage.vue - 完结页 UI
│     │  │  │  ├─ pages/WatchingPage.vue - 正在追页 UI
│     │  │  │  └─ types/
│     │  │  │     ├─ anime.ts - 番剧类型定义
│     │  │  │     └─ tracking.ts - 追番状态类型定义
│     │  │  └─ tracks/
│     │  │     ├─ components/MixQueueDetailModal.vue - 混流队列详情弹窗
│     │  │     ├─ composables/useTracksPage.ts - 轨道混流逻辑
│     │  │     ├─ pages/TracksPage.vue - 混流页 UI
│     │  │     └─ types/tracks.ts - 轨道类型定义
│     │  └─ shared/
│     │     ├─ components/AppTitlebar.vue - 窗口标题栏
│     │     ├─ composables/useExternalLink.ts - 外链打开封装
│     │     ├─ composables/useWindowControls.ts - 窗口控制封装
│     │     ├─ types/page.ts - 页面类型定义
│     │     └─ utils/
│     │        ├─ format.ts - 格式化工具
│     │        └─ tauri.ts - Tauri 运行环境判断
│     ├─ public/
│     │  ├─ fonts/ - 字体资源
│     │  │  ├─ NotoSansJP-Regular.ttf - 日文字体常规
│     │  │  ├─ NotoSansJP-SemiBold.ttf - 日文字体加粗
│     │  │  ├─ NotoSansKR-Regular.ttf - 韩文字体常规
│     │  │  ├─ NotoSansKR-SemiBold.ttf - 韩文字体加粗
│     │  │  ├─ NotoSansSC-Regular.ttf - 简中文字体常规
│     │  │  ├─ NotoSansSC-SemiBold.ttf - 简中文字体加粗
│     │  │  ├─ NotoSansTC-Regular.ttf - 繁中文字体常规
│     │  │  └─ NotoSansTC-SemiBold.ttf - 繁中文字体加粗
│     │  ├─ icons/ - 应用图标资源
│     │  │  ├─ icon.ico - Windows 图标
│     │  │  ├─ icon.png - PNG 图标
│     │  │  └─ icon.svg - SVG 图标
│     │  └─ tools/ - 内置工具
│     │     ├─ ffmpeg.exe - 媒体处理
│     │     ├─ ffprobe.exe - 媒体信息分析
│     │     ├─ mkvinfo.exe - MKV 信息分析
│     │     └─ mkvmerge.exe - MKV 混流
│     └─ vite.config.ts - Vite 配置
├─ crates/
│  └─ baidu_verify/
│     ├─ Cargo.lock - 动态库依赖锁定
│     ├─ Cargo.toml - 动态库编译配置
│     └─ src/
│        └─ lib.rs - 百度翻译密钥导出
├─ dist/
│  └─ baidu_verify/
│     └─ windows/
│        └─ baidu_verify.dll - 构建后的动态库
├─ docs/
│  ├─ API.md - API 文档
│  ├─ ARCHITECTURE.md - 架构说明
│  └─ DEVELOPMENT.md - 开发指南
├─ LICENSE - 许可证
├─ package.json - Node 脚本与依赖
├─ README.md - 项目说明
├─ scripts/
│  ├─ banner.ps1 - 命令行 banner
│  ├─ clean.js - 清理脚本
│  └─ setup-env.ps1 - Windows 环境初始化脚本
└─ yarn.lock - 依赖锁定
```

## 模块说明

- `apps/desktop/backend/src/services/bangumi`：Bangumi 数据聚合与翻译
- `apps/desktop/backend/src/services/media`：轨道解析/混流
- `apps/desktop/backend/src/services/torrent`：下载管理
- `apps/desktop/backend/src/services/storage`：本地追番数据存储
- `apps/desktop/backend/src/services/external`：外链打开
- `apps/desktop/frontend/modules/*`：按业务模块拆分的 UI 逻辑
- `apps/desktop/frontend/shared/*`：复用组件/工具

## 更新记录
- 2026-02-06：初始第一版。

