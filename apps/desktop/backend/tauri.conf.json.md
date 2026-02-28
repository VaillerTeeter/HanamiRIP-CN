# tauri.conf.json 配置说明

文件：tauri.conf.json | 用途：Tauri 应用构建、窗口、权限和打包配置 | 关键字段：$schema、productName、version、identifier、build、app、bundle

本文件无函数/变量（不适用）

## 1. 当前配置摘要

- Tauri Schema：`https://schema.tauri.app/config/2`
- 产品名：`HanamiRIP CN`
- 版本：`0.1.0`
- 标识符：`com.hanamirip.cn`
- 开发地址：`http://localhost:1420`
- 前端产物目录：`../../../build/frontend`
- 打包目标：`nsis`

## 2. 顶层字段

**$schema** | 含义：JSON Schema 地址 | 类型：string | 作用域：编辑器校验
- 建议保持与 Tauri 主版本一致。

**productName** | 含义：应用展示名 | 类型：string | 作用域：窗口与安装包显示

**version** | 含义：应用版本号 | 类型：string | 作用域：打包版本标识

**identifier** | 含义：应用唯一 ID（反向域名） | 类型：string | 作用域：系统注册与升级链路

## 3. build 配置

**build.beforeDevCommand**
- 当前值：`yarn --cwd ../.. dev`
- 作用：`tauri dev` 前先启动前端开发服务器

**build.beforeBuildCommand**
- 当前值：`yarn --cwd ../.. vite build --config apps/desktop/vite.config.ts`
- 作用：`tauri build` 前先构建前端静态资源

**build.devUrl**
- 当前值：`http://localhost:1420`
- 作用：开发模式 WebView 加载地址

**build.frontendDist**
- 当前值：`../../../build/frontend`
- 作用：生产模式前端静态资源根目录

## 4. app.windows 配置

当前仅定义 1 个主窗口：

- `title`: `HanamiRIP CN`
- `width`: `1360`
- `height`: `820`
- `resizable`: `false`
- `maximizable`: `false`
- `decorations`: `false`

说明：
- `decorations=false` 表示使用自绘标题栏。
- 禁止缩放和最大化，有助于固定布局一致性。

## 5. app.security 配置

**csp: null**
- 当前为关闭 CSP（开发更灵活）。
- 发布阶段可按安全策略逐步收紧。

**capabilities[0]**
- `identifier`: `window-controls`
- `windows`: `["main"]`
- `permissions`:
  - `core:default`
  - `core:window:allow-close`
  - `core:window:allow-minimize`
  - `core:window:allow-start-dragging`
  - `dialog:allow-open`
  - `dialog:allow-save`

说明：
- 权限仅授予主窗口。
- 建议按最小权限原则维护。

## 6. bundle 配置

**bundle.icon**
- 当前值：`["../public/icons/icon.ico", "../public/icons/icon.png", "../public/icons/icon.svg"]`
- 用途：构建不同格式图标资源

**bundle.targets**
- 当前值：`["nsis"]`
- 用途：Windows 安装包输出为 NSIS

## 7. 常见变更点

### 修改开发端口

- 调整 Vite 端口后，需同步修改 `build.devUrl`。

### 修改窗口策略

- 允许缩放：将 `resizable` 改为 `true`。
- 允许最大化：将 `maximizable` 改为 `true`。

### 新增能力权限

- 在 `app.security.capabilities[*].permissions` 中新增。
- 新增后建议同步更新 API 文档并验证风险。