# HanamiRIP CN

一个基于 Tauri v2 + Vite + Vue 3 + Naive UI 的桌面应用，用于查询季度番剧并进行筛选与详情查看。后端通过 Tauri 命令与 Rust 实现，调用 Bangumi 公共 API 获取数据；可选的中文简介翻译通过本地动态库驱动的百度翻译接口完成（不在仓库中存储任何密钥）。

## 功能概览
- 季度番剧查询（已实现）：按年份与季度拉取番剧列表，显示缩略图、评分、首播日期等，支持多维筛选与条目详情。
- 追番状态管理（已实现）：正在追番 / 补番计划 / 已完番剧三态互斥，支持一键切换；按照放送日期升序排列，正在追番页按周一开头分栏展示，数据持久化到本机 Tauri 数据目录的 `watchlist.json`。
- 搜索资源（已实现）：内置 Nyaa 查询拼装器，支持预设短语（SubsPlease、LoliHouse、内封、外挂、480、720、1080）、逻辑运算（与/或/非）、自定义精确短语、追番条目下拉选择，并可为选中的番剧单独勾选名称/别名后加入查询。

## 追番状态使用说明
- 入口：在季度查询详情面板内使用“正在追番 / 补番计划 / 已完番剧”按钮进行状态切换，三态互斥；在列表页也可直接切换。
- 排序与分组：三个列表均按放送日期升序；“正在追番”按周一开头的星期分栏展示（未知日期单独分组）。
- 清空状态：在“已完番剧”页可点“变为未观看”清除状态；任意状态全部关闭后会从列表移除。
- 持久化：状态与条目快照保存在 Tauri 数据目录下的 `watchlist.json`（自动创建，无需手动配置）。

## 搜索资源使用说明
- 入口：顶部导航“搜索资源”。
- 逻辑与预设：先选择逻辑（与/或/非），再点击预设短语或自定义短语添加到查询；拼接结果区仅展示最终关键词。
- 番剧下拉与别名选择：选择追番/补番/已完番剧时，会弹出名称/别名选择框（默认全不选），可勾选任意名称/别名后加入查询；当前以 AND 方式连接。
- 打开搜索：点击“打开搜索”后，会在当前页面下方直接展示解析好的 Nyaa 结果列表（名称 + 磁链/种子 + Size/Date），无需跳转外部浏览器；仍保留 URL 作为备用外链。
- 持久化：别名字段已在后端持久化结构中预留（`aliases`），后续前端传入时会写入 `watchlist.json`。

## 下载操作说明
- 点击“磁链/种子”会先弹出系统目录选择窗口，用于记录期望的下载位置。
- 选择完成后会使用内置 BitTorrent 引擎进行下载，不再调用外部下载器。
- 下载页支持单条暂停/继续/删除与“全部暂停/全部继续”，并显示总下载/上传速率。
- 下载速度受种子健康度与网络环境影响，做种越多通常速度越快。

## 轨道工坊（轨道解析/混合）
- 入口：顶部导航“轨道工坊”。
- 轨道检测：分别为视频/音频/字幕添加文件后点击“检测”，会解析轨道信息并支持勾选需要的轨道。
- 默认语言：在“重新混合为新视频”按钮下方可设置视频/音频/字幕的默认语言，合成时会写入对应轨道语言属性。
- 合成流程：会先将选中的视频、音频、字幕轨道分别导出为临时文件，再合并为新的 MKV 文件。
- 混流任务队列：点击“添加到混流任务队列”可加入任务；在队列中可“开始任务队列”顺序执行，也可“清除所有任务”；点击任务可查看详情。

## 目录结构（关键）
- 前端入口：见 [src/main.ts](src/main.ts)
- 应用外壳（页面切换与弹窗组合）：见 [src/App.vue](src/App.vue)
- 页面与组件：见 [src/pages](src/pages) 、[src/components](src/components)
- 组合式逻辑（状态/请求/交互）：见 [src/composables](src/composables)
- 类型与工具：见 [src/types](src/types)、[src/utils](src/utils)
- 样式：见 [src/style.css](src/style.css)
- 静态资源：见 [public](public)（fonts、tools、icons）
- 后端入口（Tauri）：见 [src-tauri/src/main.rs](src-tauri/src/main.rs)
- 后端模块：见 [src-tauri/src](src-tauri/src)（如 bangumi/、media.rs、torrent.rs、storage.rs、external.rs）
- 百度翻译校验动态库：见 [src-tauri/baidu_verify/src/lib.rs](src-tauri/baidu_verify/src/lib.rs)
- 环境一键安装：
  - Windows：见 [scripts/setup-env.ps1](scripts/setup-env.ps1)
  - Linux：见 [scripts/setup-env.sh](scripts/setup-env.sh)

## 环境与依赖

### Windows 环境
- Windows 10 1809+ 或 Windows 11
- 系统依赖：自动安装（通过 winget）
- Node.js 24 LTS + Yarn
- Rust 工具链（stable，含 x86_64 与 i686 目标）
- **百度翻译插件**：需 Visual Studio Build Tools（C++ 工作负载）提供 `link.exe`；脚本会检测并在**管理员**下尝试自动安装，非管理员时仅提示，其余步骤照常执行
- 字体：已内置 Noto Sans SC/JP/KR/TC（400/600），位于 [public/fonts](public/fonts)，无需联网获取 Google Fonts

**一键安装**（使用 PowerShell，会自动安装 Node 24、Yarn、Rust、检测/安装 MSVC 并安装项目依赖）：

```powershell
.\scripts\setup-env.ps1
```

### Linux 环境（Ubuntu）
- Ubuntu（已在 WSL/Ubuntu 环境验证）
- 系统依赖（GTK/WebKit 等，脚本会安装）：`build-essential`、`pkg-config`、`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`librsvg2-dev`、`libssl-dev`、`curl`、`ca-certificates`
- **百度翻译插件**：Rust 编译依赖 gcc/cc，由 `build-essential` 提供；脚本会检测，缺失时提示安装
- Node.js 24 + Yarn（通过 corepack 管理）
- Rust 工具链（stable）

**一键安装**（会安装系统依赖、Node 24、Yarn、Rust、检测链接器并安装项目依赖）：

```bash
./scripts/setup-env.sh
```

## 内置 FFmpeg/MKVToolNix 工具（轨道解析/分离/合成）
- Windows 与 Ubuntu 会在环境脚本中自动下载并放入 [public/tools](public/tools)。
- 若需手动放置，请将 ffmpeg/ffprobe/mkvmerge/mkvinfo 放到该目录中（Windows 为 .exe）。

## 开发

项目使用 Yarn 脚本进行开发（参考 [package.json](package.json)）：

- 开发（桌面应用，Tauri 调试）：

```bash
yarn debug
```

- 前端本地开发（仅 Vite Web 服务）：

```bash
yarn dev
```

- 生成 Windows 安装包（.exe）：

```bash
yarn package:windows
```

输出内容：
- dist/windows/HanamiRIP-CN_x64-setup.exe：64 位 NSIS 安装包
- dist/windows/HanamiRIP-CN_x86-setup.exe：32 位 NSIS 安装包

- 仅生成 64 位安装包：

```bash
yarn package:windows:x64
```

- 仅生成 32 位安装包：

```bash
yarn package:windows:x86
```

- 清理构建产物：

```bash
yarn clean
```

## 可选：生成百度翻译校验动态库

为了在本地安全地读取百度翻译密钥，项目通过一个动态库在运行时以只读方式提供凭据，避免密钥出现在源码和产物中。

动态库的源码在 [src-tauri/baidu_verify/src/lib.rs](src-tauri/baidu_verify/src/lib.rs)。编译该动态库需要在「编译期」提供环境变量（仅在本机导出占位，绝不提交到仓库）。

- **Windows**：需已安装 Visual Studio Build Tools（C++ 工作负载），否则会报 `link.exe not found`；运行 `.\scripts\setup-env.ps1` 可检测并在管理员下安装。
- **Linux**：需已安装 `build-essential`（gcc/cc）；运行 `./scripts/setup-env.sh` 会安装并检测。

### Windows 环境

```powershell
# 注意：以下为占位示例，请替换为你本机的真实值；不要提交或分享！
$env:BAIDU_TRANSLATE_APP_ID = "<your-app-id>"
$env:BAIDU_TRANSLATE_API_KEY = "<your-api-key>"

# 方式1：使用 npm script（推荐）
yarn run build:baidu-so:windows

# 验证文件已生成
Test-Path "dist\baidu_verify\windows\baidu_verify.dll"
# 应该返回 True

```

### Linux 环境

```bash
# 注意：以下为占位示例，请替换为你本机的真实值；不要提交或分享！
export BAIDU_TRANSLATE_APP_ID="<your-app-id>"
export BAIDU_TRANSLATE_API_KEY="<your-api-key>"

# 方式1：使用 npm script（推荐）
yarn run build:baidu-so:linux

# 验证文件已生成
test -f "dist/baidu_verify/linux/libbaidu_verify.so" && echo "OK"

```

运行时，后端只会从 dist/baidu_verify/<platform>/ 目录加载该动态库。

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

### Windows 环境
- **winget 不可用**：请确保使用 Windows 10 1809+ 或 Windows 11，并从 Microsoft Store 安装「应用安装程序」。
- **link.exe not found / 百度插件编译失败**：需安装 Visual Studio Build Tools（C++ 工作负载）。以**管理员**运行 `.\scripts\setup-env.ps1` 可自动安装；或手动执行：`winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended"`，安装后重新打开终端再运行一次脚本以写入 PATH。
- **Node 版本不匹配**：项目需要 Node 24（脚本会自动安装）；如需自管，请确保 `yarn dev` 与 `yarn tauri dev` 可正常运行。
- **数据目录位置**：应用运行数据默认存放在系统应用数据目录，同时在安装目录创建 `user-data` 目录联接指向该位置，方便查看与备份。
- **百度翻译不可用或返回为空**：请检查本地是否已正确编译并加载动态库（Windows 为 `.dll` 格式），以及密钥是否通过环境变量在构建期注入（不存储到仓库）。
- **缺少图标文件**：运行 `yarn tauri icon public/icons/icon.png` 重新生成 Windows 所需的 `.ico` 文件。

### Linux 环境
- **Tauri 依赖缺失**：请先执行 `./scripts/setup-env.sh` 或手动安装 GTK/WebKit 相关依赖。
- **百度插件编译失败 / gcc 未找到**：脚本会安装 `build-essential`；若未以脚本安装，可执行 `sudo apt-get install -y build-essential` 后重跑脚本或直接编译。
- **Node 版本不匹配**：项目默认使用 Node 24（脚本会安装）；如需自管，请确保 `yarn dev` 与 `yarn tauri dev` 可正常运行。
- **百度翻译不可用或返回为空**：请检查本地是否已正确编译并加载 `libbaidu_verify.so`，以及密钥是否通过环境变量在构建期注入（不存储到仓库）。
