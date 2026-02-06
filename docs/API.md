
# API 文档

本项目包含前端调用的 Tauri 命令（Rust 后端），以及前端直接调用的 Tauri JS API 和外部 HTTP API。下面按类型列出，并给出简要说明与用法。

## Tauri 命令（Rust 后端）

说明：以下命令均通过 `@tauri-apps/api/core` 的 `invoke()` 调用。

### Bangumi 数据

- `get_season_subjects(year: number, season: string)`
	- 简介：按季度获取番剧列表。
	- 用法：`invoke("get_season_subjects", { year, season })`

- `get_subject_origin(id: number)`
	- 简介：获取番剧原作信息。
	- 用法：`invoke("get_subject_origin", { id })`

- `get_subject_aired_count(id: number)`
	- 简介：获取已播集数与总集数。
	- 用法：`invoke("get_subject_aired_count", { id })`

- `get_subject_filters(id: number)`
	- 简介：获取番剧筛选标签（类型/地区/受众）。
	- 用法：`invoke("get_subject_filters", { id })`

- `get_subject_staff(id: number)`
	- 简介：获取制作人员分组信息。
	- 用法：`invoke("get_subject_staff", { id })`

- `get_subject_characters(id: number)`
	- 简介：获取角色列表。
	- 用法：`invoke("get_subject_characters", { id })`

- `get_subject_summary_cn(id: number, summary: string)`
	- 简介：获取/翻译番剧简介为中文。
	- 用法：`invoke("get_subject_summary_cn", { id, summary })`

- `get_subject_brief(id: number)`
	- 简介：获取番剧简要信息（名称/图片/评分等）。
	- 用法：`invoke("get_subject_brief", { id })`

- `get_subject_aliases(id: number)`
	- 简介：获取番剧别名列表。
	- 用法：`invoke("get_subject_aliases", { id })`

- `fetch_search_html(url: string)`
	- 简介：抓取搜索页面 HTML（用于站内解析）。
	- 用法：`invoke("fetch_search_html", { url })`

### 媒体轨道解析/混流

- `parse_media_tracks(path: string, kind: "video" | "audio" | "subtitle")`
	- 简介：解析媒体文件轨道信息。
	- 用法：`invoke("parse_media_tracks", { path, kind })`

- `get_media_file_size(path: string)`
	- 简介：获取媒体文件大小（可读格式）。
	- 用法：`invoke("get_media_file_size", { path })`

- `mix_media_tracks(inputs: MixTrackInput[], outputPath: string)`
	- 简介：按指定轨道混流生成输出文件。
	- 用法：`invoke("mix_media_tracks", { inputs, outputPath })`

### 下载（Torrent）

- `start_torrent_download(url: string, outputDir: string)`
	- 简介：启动下载任务。
	- 用法：`invoke("start_torrent_download", { url, outputDir })`

- `get_torrent_status(id: number)`
	- 简介：获取下载任务状态。
	- 用法：`invoke("get_torrent_status", { id })`

- `pause_torrent(id: number)`
	- 简介：暂停下载任务。
	- 用法：`invoke("pause_torrent", { id })`

- `resume_torrent(id: number)`
	- 简介：恢复下载任务。
	- 用法：`invoke("resume_torrent", { id })`

- `delete_torrent(id: number)`
	- 简介：删除下载任务。
	- 用法：`invoke("delete_torrent", { id })`

- `finalize_torrent_download(tempFolder: string, finalFolder: string)`
	- 简介：完成下载后移动文件并清理临时目录。
	- 用法：`invoke("finalize_torrent_download", { tempFolder, finalFolder })`

### 本地存储

- `list_tracked_subjects()`
	- 简介：读取本地追番列表。
	- 用法：`invoke("list_tracked_subjects")`

- `save_tracked_subject(subject: TrackedSubject)`
	- 简介：保存/更新追番记录。
	- 用法：`invoke("save_tracked_subject", { subject })`

### 外部链接

- `open_external_link(url: string)`
	- 简介：使用系统默认浏览器打开链接。
	- 用法：`invoke("open_external_link", { url })`

## 前端 Tauri JS API

- `@tauri-apps/api/core.invoke()`
	- 简介：调用上面的后端命令。
	- 用法：`invoke("command_name", payload)`

- `@tauri-apps/api/window.getCurrentWindow()`
	- 简介：获取当前窗口实例。
	- 用法：`const appWindow = getCurrentWindow()`

- `Window.minimize()` / `Window.close()`
	- 简介：窗口最小化与关闭。
	- 用法：`await appWindow.minimize()` / `await appWindow.close()`

- `@tauri-apps/plugin-dialog.open()` / `save()`
	- 简介：打开文件/保存文件对话框。
	- 用法：`open({ directory: false, multiple: false })` / `save({ filters: [...] })`

## 外部 HTTP API

- Bangumi API
	- 基础地址：`https://api.bgm.tv`
	- 用途：番剧信息、角色、人员、别名、集数等。

- 搜索抓取（Nyaa）
	- 基础地址：`https://nyaa.vaciller.top/`
	- 用途：下载搜索页 HTML 解析。

- 百度翻译 AI
	- 地址：`https://fanyi-api.baidu.com/ait/api/aiTextTranslate`
	- 用途：番剧简介翻译。
	- 需要环境变量：`BAIDU_TRANSLATE_APP_ID`、`BAIDU_TRANSLATE_API_KEY`

## 本地工具（随应用打包）

- `ffprobe` / `ffmpeg` / `mkvmerge` / `mkvinfo`
	- 用途：媒体轨道解析、封装与混流。
	- 位置：`apps/desktop/public/tools/`（打包后内置到资源目录）。

## 更新记录
- 2026-02-06：初始第一版。

