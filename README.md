# HanamiRIP CN

一个基于 Tauri v2 + Vite + Vue 3 + Naive UI 的桌面应用，用于查询季度番剧并进行筛选与详情查看。后端通过 Tauri 命令与 Rust 实现，调用 Bangumi 公共 API 获取数据；可选的中文简介翻译通过本地动态库驱动的百度翻译接口完成（不在仓库中存储任何密钥）。

## 功能概览
- 季度番剧查询（已实现）：按年份与季度拉取番剧列表，显示缩略图、评分、首播日期等，支持多维筛选与条目详情。
- 追番状态管理（已实现）：正在追番 / 补番计划 / 已完番剧三态互斥，支持一键切换；按照放送日期升序排列，正在追番页按周一开头分栏展示，数据持久化到本机 Tauri 数据目录的 `watchlist.json`。
- 搜索资源（已实现）：内置 Nyaa 查询拼装器，支持预设短语（SubsPlease、LoliHouse、内封、外挂）、逻辑运算（与/或/非）、自定义精确短语、追番条目下拉选择，并可为选中的番剧单独勾选名称/别名后加入查询。

## 追番状态使用说明
- 入口：在季度查询详情面板内使用“正在追番 / 补番计划 / 已完番剧”按钮进行状态切换，三态互斥；在列表页也可直接切换。
- 排序与分组：三个列表均按放送日期升序；“正在追番”按周一开头的星期分栏展示（未知日期单独分组）。
- 清空状态：在“已完番剧”页可点“变为未观看”清除状态；任意状态全部关闭后会从列表移除。
- 持久化：状态与条目快照保存在 Tauri 数据目录下的 `watchlist.json`（自动创建，无需手动配置）。

## 搜索资源使用说明
- 入口：顶部导航“搜索资源”。
- 逻辑与预设：先选择逻辑（与/或/非），再点击预设短语或自定义短语添加到查询；拼接结果区展示最终关键词和可直接打开的 Nyaa URL。
- 番剧下拉与别名选择：选择追番/补番/已完番剧时，会弹出名称/别名选择框（默认全不选），可勾选任意名称/别名后加入查询；当前以 AND 方式连接。
- 持久化：别名字段已在后端持久化结构中预留（`aliases`），后续前端传入时会写入 `watchlist.json`。

## 目录结构（关键）
- 前端：见 [src/main.ts](src/main.ts)、[src/App.vue](src/App.vue)、[vite.config.ts](vite.config.ts)
- 样式：见 [src/style.css](src/style.css)
- 后端（Tauri）：见 [src-tauri/src/main.rs](src-tauri/src/main.rs)、[src-tauri/Cargo.toml](src-tauri/Cargo.toml)、[src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)
- 百度翻译校验动态库：见 [src-tauri/baidu_verify/src/lib.rs](src-tauri/baidu_verify/src/lib.rs)
- 环境一键安装：见 [scripts/setup-env.sh](scripts/setup-env.sh)

## 环境与依赖
- Linux（已在 WSL/Ubuntu 环境验证）
- 系统依赖（GTK/WebKit 等，脚本会安装）：`build-essential`、`pkg-config`、`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`librsvg2-dev`、`libssl-dev`、`curl`、`ca-certificates`
- Node.js 24 + Yarn（通过 corepack 管理）
- Rust 工具链（stable）
- 字体：已内置 Noto Sans SC/JP/KR/TC（400/600），位于 [src/assets/fonts](src/assets/fonts)，无需联网获取 Google Fonts

一键安装（会安装系统依赖、Node 24、Yarn、Rust 并安装项目依赖）：

```bash
./scripts/setup-env.sh
```

## 开发与构建

项目使用 Yarn 脚本进行开发与打包（参考 [package.json](package.json)）：

- 开发（桌面应用，Tauri 调试）：

```bash
yarn tauri dev
```

- 前端本地开发（仅 Vite Web 服务）：

```bash
yarn dev
```

- 构建桌面应用（打包）：

```bash
yarn build
```

- 清理构建产物：

```bash
yarn run clean:all
```

## 可选：生成百度翻译校验动态库

为了在本地安全地读取百度翻译密钥，项目通过一个动态库 `libbaidu_verify.so` 在运行时以只读方式提供凭据，避免密钥出现在源码和产物中。

动态库的源码在 [src-tauri/baidu_verify/src/lib.rs](src-tauri/baidu_verify/src/lib.rs)。编译该动态库需要在“编译期”提供环境变量（仅在本机导出占位，绝不提交到仓库）：

```bash
# 注意：以下为占位示例，请替换为你本机的真实值；不要提交或分享！
export BAIDU_TRANSLATE_APP_ID="<your-app-id>"
export BAIDU_TRANSLATE_API_KEY="<your-api-key>"

# 生成本地动态库（默认输出到 src-tauri/baidu_verify/libbaidu_verify.so）
yarn run build:baidu-so
```

运行时，后端会尝试从默认路径加载该动态库；也可通过环境变量覆盖路径：

```bash
export BAIDU_VERIFY_SO="/absolute/path/to/libbaidu_verify.so"
```

## 运行机制简述
- 前端通过 `@tauri-apps/api` 的 `invoke()` 调用后端命令，如：`get_season_subjects`、`get_subject_filters`、`get_subject_origin`、`get_subject_aired_count`、`get_subject_staff`、`get_subject_characters`、`get_subject_summary_cn` 等。
- 后端使用 Rust 的 `reqwest` 调用 Bangumi 公共 API 并进行数据加工与聚合（分页、筛选、映射等）。
- 中文简介翻译：后端会尝试通过本地动态库读取百度翻译的凭据并调用官方接口；若未配置或失败则不返回翻译结果，前端会安全降级。

## 配置与安全
- 本项目不在仓库中保存任何第三方 API 密钥，也不会在 README 中展示真实凭据。
- 请在本机通过环境变量注入所需密钥，并确保 `.env`、密钥脚本或生成的动态库路径不被提交到版本库。
- 如需使用代理联网，可在 shell 中设置 `HTTP(S)_PROXY`/`ALL_PROXY` 等环境变量；后端基于 `reqwest`，可遵循系统环境变量。

## 许可证
见 [LICENSE](LICENSE)。

## 常见问题
- Tauri 依赖缺失：请先执行 `./scripts/setup-env.sh` 或手动安装 GTK/WebKit 相关依赖。
- Node 版本不匹配：项目默认使用 Node 24（脚本会安装）；如需自管，请确保 `yarn dev` 与 `yarn tauri dev` 可正常运行。
- 百度翻译不可用或返回为空：请检查本地是否已正确编译并加载 `libbaidu_verify.so`，以及密钥是否通过环境变量在构建期注入（不存储到仓库）。
