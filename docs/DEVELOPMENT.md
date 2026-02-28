# 开发指南

本文档用于日常开发、联调、构建和常见问题排查。

## 1. 环境要求

- Windows 10/11
- Node.js 24+
- Yarn 1.x（或 corepack 提供）
- Rust（含 `x86_64-pc-windows-msvc`、`i686-pc-windows-msvc` 目标）
- MSVC Build Tools（`link.exe`）
- NSIS（用于安装包）

## 2. 一键初始化

在仓库根目录执行：

```bash
yarn setup:env
```

执行内容：
- 运行 `scripts/setup-env.ps1`
- 检查/安装 Node、Yarn、Rust、Rust targets、MSVC、NSIS
- 下载 `nsis_tauri_utils.dll`
- 安装前端依赖
- 下载 FFmpeg/MKVToolNix 到 `apps/desktop/public/tools`

## 3. 开发模式

### 3.1 仅前端

```bash
yarn dev
```

- 启动 Vite（`http://localhost:1420`）
- 适合 UI 开发
- 不可调用 Tauri command（无后端运行时）

### 3.2 前后端联调

```bash
yarn debug
```

- 设置 `CARGO_TARGET_DIR=../../../build/tauri-target`
- 执行 `tauri dev --config apps/desktop/backend/tauri.conf.json`
- 前端可正常 `invoke()` 后端命令

## 4. 清理、构建、打包

### 清理

```bash
yarn clean
```

清理：`build/`、`dist/`

### 构建

```bash
# 双架构
yarn build:windows

# 单架构
yarn build:windows:x64
yarn build:windows:x86
```

### 打包整理

```bash
# 双架构整理
yarn package:windows

# 单架构整理
yarn package:windows:x64
yarn package:windows:x86
```

目标文件：`dist/windows/HanamiRIP-CN_{x64|x86}-setup.exe`

## 5. 百度翻译 DLL（可选）

如需启用翻译链路，先设置环境变量：

```powershell
$env:BAIDU_TRANSLATE_APP_ID = "your_app_id"
$env:BAIDU_TRANSLATE_API_KEY = "your_api_key"
```

执行：

```bash
yarn build:baidu-dll
```

输出：`dist/baidu_verify/windows/baidu_verify.dll`

## 6. 调试建议

- 前端调试：优先使用 `yarn dev` 快速迭代样式与交互。
- 命令联调：涉及 `invoke()` 时使用 `yarn debug`。
- 下载问题：优先检查 `list_pending_downloads` 与任务落盘文件。
- 混流问题：检查 `apps/desktop/public/tools` 下工具是否齐全。

## 7. 常见问题

### Q1: 打包提示 `link.exe` 不存在

- 说明：MSVC 工具链缺失。
- 处理：以管理员权限重新执行 `yarn setup:env`。

### Q2: 打包提示 NSIS 相关错误

- 说明：`makensis.exe` 或 `nsis_tauri_utils.dll` 缺失。
- 处理：重新执行 `yarn setup:env` 并重启终端。

### Q3: `invoke()` 报错

- 说明：通常在 `yarn dev` 模式下调用了后端命令。
- 处理：改用 `yarn debug`。

### Q4: 混流失败

- 说明：输入轨道不完整或工具不可用。
- 处理：先调用 `parse_media_tracks` 验证轨道，再检查工具文件是否存在。

## 8. 开发约定

- 前端按模块目录组织：`pages/components/composables/types`
- 后端能力统一通过 `#[tauri::command]` 暴露
- 文档修改后请同步校验对应配置/命令是否一致