# 开发指南

## 环境要求

- Windows 10/11
- Node.js 24+
- Yarn 1.x
- Rust（含 MSVC 工具链）
- NSIS（Windows 打包）

## 一键环境准备

在项目根目录执行：

```
yarn setup:env
```

脚本会自动：

- 安装/校验 Node、Yarn、Rust、MSVC
- 安装/校验 NSIS，并下载 `nsis_tauri_utils.dll`
- 安装前端依赖
- 下载 FFmpeg/MKVToolNix 工具到 `apps/desktop/public/tools`

## 开发运行

仅前端（Vite）：

```
yarn dev
```

Tauri 调试（前端 + 后端）：

```
yarn debug
```

## 清理

清理构建产物与打包输出：

```
yarn clean
```

## 构建与打包

仅构建 Windows 可执行与打包所需产物：

```
yarn build:windows
```

将 NSIS 安装包复制到 `dist/windows`：

```
yarn package:windows
```

如需单独构建/打包：

```
yarn build:windows:x64
yarn build:windows:x86
yarn package:windows:x64
yarn package:windows:x86
```

## 百度翻译 DLL

用于翻译简介的动态库需要手动构建：

1. 设置环境变量：
	- `BAIDU_TRANSLATE_APP_ID`
	- `BAIDU_TRANSLATE_API_KEY`
2. 执行：
	```
	yarn build:baidu-dll
	```
3. 生成文件：
	- `dist/baidu_verify/windows/baidu_verify.dll`

## 目录约定

- 前端：`apps/desktop/frontend`
- 后端：`apps/desktop/backend`
- 公共静态资源：`apps/desktop/public`
- 打包产物：`dist/`

## 常见问题

- NSIS 相关错误：先执行 `scripts/setup-env.ps1` 重新检测。
- Tauri 版本不一致：确保 `apps/desktop/backend/Cargo.toml` 与 `package.json` 版本匹配。
- 找不到百度 DLL：确认 `dist/baidu_verify/windows/baidu_verify.dll` 存在。

## 更新记录
- 2026-02-06：初始第一版。
