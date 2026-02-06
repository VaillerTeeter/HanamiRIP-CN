# tauri.conf.json 注释说明

> 说明：Tauri 的 JSON 配置不允许自定义字段，因此注释放在此旁路文档中。

## 顶层字段
- $schema：配置 schema 地址，用于编辑器校验与提示。
- productName：产品名称，会显示在窗口标题、安装包等位置。
- version：应用版本号（语义化版本）。
- identifier：应用唯一标识（类似包名/反向域名）。

## build
- beforeDevCommand：开发模式启动前执行的命令（用于启动前端 dev server）。
- beforeBuildCommand：打包前执行的命令（用于构建前端静态资源）。
- devUrl：开发模式下加载的前端地址。
- frontendDist：生产模式下前端静态资源目录。

## app.windows
- title：窗口标题。
- width / height：窗口初始大小（像素）。
- resizable：是否允许拖拽调整大小。
- maximizable：是否允许最大化。
- decorations：是否显示系统标题栏/边框（false 通常表示自绘标题栏）。

## app.security
- csp：内容安全策略（CSP）。null 表示禁用，灵活但风险更高。
- capabilities：权限能力集，限定窗口可用的 API 权限。
  - identifier：能力集名称。
  - windows：适用窗口列表。
  - permissions：允许的权限集合（窗口控制、对话框等）。

## bundle
- icon：应用图标文件列表（不同平台选用不同格式）。
- targets：打包目标类型（这里使用 NSIS 安装包）。

## 更新记录
- 2026-02-06：初始第一版。
