# package.json 配置说明

文件：package.json | 用途：定义前端依赖、开发脚本与打包流程 | 关键字段：name、version、scripts、dependencies、devDependencies

本文件无函数/变量（不适用）

## 1. 基础字段

**name** | 含义：项目包名 | 类型：string | 作用域：包管理标识
- 当前值：`"hanamirip-cn"`

**private** | 含义：禁止发布到公共 npm | 类型：boolean | 作用域：发布控制
- 当前值：`true`

**version** | 含义：项目版本号 | 类型：string | 作用域：构建与发布标识
- 当前值：`"0.1.0"`

**type** | 含义：模块系统 | 类型：string | 作用域：Node.js 解析规则
- 当前值：`"module"`

## 2. scripts 命令说明

### 2.1 环境与开发

**setup:env**
```bash
yarn setup:env
```
- 执行：`powershell ... scripts/setup-env.ps1`
- 用途：Windows 环境自动检查与安装

**predev**
```bash
yarn predev
```
- 执行：`powershell ... scripts/banner.ps1`
- 用途：输出项目 banner

**dev**
```bash
yarn dev
```
- 执行：`vite --config apps/desktop/vite.config.ts`
- 用途：前端开发服务

**debug**
```bash
yarn debug
```
- 执行：设置 `CARGO_TARGET_DIR` 后运行 `tauri dev`
- 用途：前后端联调

### 2.2 清理与构建

**clean**
```bash
yarn clean
```
- 执行：`yarn predev && node scripts/clean.js`
- 用途：清理 `build/` 与 `dist/`

**build:baidu-dll**
```bash
yarn build:baidu-dll
```
- 用途：编译 `crates/baidu_verify` 并复制到 `dist/baidu_verify/windows`
- 前置：`BAIDU_TRANSLATE_APP_ID`、`BAIDU_TRANSLATE_API_KEY`

**build:windows:x64** / **build:windows:x86**
```bash
yarn build:windows:x64
yarn build:windows:x86
```
- 执行：`tauri build --bundles nsis --target ...`
- 用途：分别构建 x64/x86 安装包

**build:windows**
```bash
yarn build:windows
```
- 执行：串行执行 x64 + x86 构建

### 2.3 安装包整理

**package:windows:x64** / **package:windows:x86**
```bash
yarn package:windows:x64
yarn package:windows:x86
```
- 用途：从 `build/tauri-target/.../bundle/nsis/*.exe` 复制并重命名到 `dist/windows`

**package:windows**
```bash
yarn package:windows
```
- 用途：一次性整理双架构安装包

## 3. dependencies（运行时依赖）

- `@tauri-apps/api`：前后端命令调用与窗口 API
- `@tauri-apps/plugin-dialog`：系统文件对话框
- `@tauri-apps/plugin-fs`：文件系统访问插件
- `@arco-design/web-vue`：UI 组件库
- `vue`：前端框架

## 4. devDependencies（开发依赖）

- `@tauri-apps/cli`：Tauri 开发与打包命令
- `vite`、`@vitejs/plugin-vue`：前端开发与构建
- `typescript`、`@types/node`：类型系统
- `prettier`、`prettier-plugin-powershell`：格式化工具链

## 5. 版本维护建议

- Tauri 相关版本（`@tauri-apps/api` 与 Rust 侧 `tauri`）尽量同主次版本。
- 大版本升级前先验证 `tauri.conf.json` 与权限配置。
- CI 或发布前建议固定 lockfile，避免依赖漂移。