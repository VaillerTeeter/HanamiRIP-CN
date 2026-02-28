# API 文档

本文档基于当前后端命令注册（`apps/desktop/backend/src/main.rs`）整理，覆盖前端可调用能力与主要参数。

## 1. 调用约定

前端通过：

```ts
import { invoke } from "@tauri-apps/api/core";
```

统一调用：

```ts
await invoke("command_name", { /* payload */ });
```

返回值由 Rust `Result<T, String>` 序列化后返回；失败时抛出错误字符串。

## 2. Tauri 命令总览

### 2.1 Bangumi 模块（10 个）

1. `get_season_subjects(year: u32, season: String)`
2. `get_subject_origin(id: u32)`
3. `get_subject_aired_count(id: u32)`
4. `get_subject_filters(id: u32)`
5. `get_subject_staff(id: u32)`
6. `get_subject_characters(id: u32)`
7. `get_subject_summary_cn(id: u32, summary: String)`
8. `get_subject_brief(id: u32)`
9. `fetch_search_html(url: String)`
10. `get_subject_aliases(id: u32)`

说明：
- 数据来源主要是 Bangumi API。
- `get_subject_summary_cn` 会尝试翻译，失败时返回原文并带错误信息字段。
- `fetch_search_html` 返回原始 HTML 字符串，解析逻辑在前端。

### 2.2 Torrent 模块（10 个）

1. `start_torrent_download(app, url, output_dir, output_path, total_bytes?)`
2. `get_torrent_metadata(app, url)`
3. `list_pending_downloads(app)`
4. `resume_torrent_download(app, task_id)`
5. `discard_torrent_download(app, task_id)`
6. `get_torrent_status(app, id)`
7. `finalize_torrent_download(app, temp_folder, final_path, placeholder_path)`
8. `pause_torrent(app, id)`
9. `resume_torrent(app, id)`
10. `delete_torrent(app, id)`

说明：
- `start_torrent_download` 会创建临时目录与占位文件，并持久化任务记录。
- `list_pending_downloads` 会清理已失效任务并返回仍可恢复的任务。
- `finalize_torrent_download` 会移动产物并清理占位与临时目录。

### 2.3 Media 模块（3 个）

1. `parse_media_tracks(app, path, kind)`
2. `get_media_file_size(path)`
3. `mix_media_tracks(app, inputs, output_path)`

说明：
- `parse_media_tracks`：MKV 容器优先走 `mkvmerge -J`；其他格式走 `ffprobe`。
- `kind` 常用值：`video` / `audio` / `subtitle`。
- `mix_media_tracks` 返回最终输出路径字符串。

### 2.4 Storage 模块（3 个）

1. `list_tracked_subjects(app)`
2. `save_tracked_subject(app, subject)`
3. `get_local_weekday()`

说明：
- `save_tracked_subject` 在条目三个状态都为 false 时会删除该条目。

### 2.5 External 模块（1 个）

1. `open_external_link(url)`

说明：
- 使用系统默认浏览器打开链接。

## 3. 前端常用 Tauri JS API

- `@tauri-apps/api/core.invoke`
- `@tauri-apps/api/window.getCurrentWindow`
- `Window.minimize()` / `Window.close()`
- `@tauri-apps/plugin-dialog.open` / `save`

## 4. 外部依赖接口

- Bangumi API：`https://api.bgm.tv`
- 搜索抓取来源（HTML）：`https://nyaa.vaciller.top/`
- 百度翻译（可选）：`https://fanyi-api.baidu.com/ait/api/aiTextTranslate`

## 5. 示例

### 5.1 启动下载

```ts
const task = await invoke("start_torrent_download", {
  url: "magnet:?xt=...",
  outputDir: "D:/Anime",
  outputPath: "D:/Anime/MyShow.mkv",
  totalBytes: 1_073_741_824,
});
```

### 5.2 查询轨道

```ts
const result = await invoke("parse_media_tracks", {
  path: "D:/media/input.mkv",
  kind: "audio",
});
```