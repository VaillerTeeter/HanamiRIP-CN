/* 文件：mod.rs | 用途：管理种子下载与任务状态 | 关键对象：torrent_api、start_torrent_download、DownloadTaskRecord */
// 本行目的：引入 UTC 时间工具。
use chrono::Utc;
// 本行目的：引入 rqbit API 类型。
use librqbit::api::Api;
// 本行目的：引入 torrent 标识类型。
use librqbit::api::TorrentIdOrHash;
// 本行目的：引入添加种子类型。
use librqbit::AddTorrent;
// 本行目的：引入添加种子选项。
use librqbit::AddTorrentOptions;
// 本行目的：引入对等连接选项。
use librqbit::PeerConnectionOptions;
// 本行目的：引入会话类型。
use librqbit::Session;
// 本行目的：引入会话选项。
use librqbit::SessionOptions;
// 本行目的：引入反序列化支持。
use serde::Deserialize;
// 本行目的：引入序列化支持。
use serde::Serialize;
// 本行目的：引入哈希集合类型。
use std::collections::HashSet;
// 本行目的：引入文件系统模块。
use std::fs;
// 本行目的：引入路径类型。
use std::path::Path;
// 本行目的：引入路径缓冲类型。
use std::path::PathBuf;
// 本行目的：引入系统命令模块。
use std::process::Command;
// 本行目的：引入时间间隔类型。
use std::time::Duration;
// 本行目的：引入 Tauri 管理器 trait。
use tauri::Manager;
// 本行目的：引入异步一次性初始化类型。
use tokio::sync::OnceCell as AsyncOnceCell;

// 变量：TORRENT_OUTPUT_DIR | 含义：下载输出目录名 | 类型：&str | 作用域：模块级
// 本行目的：定义下载输出目录名常量。
const TORRENT_OUTPUT_DIR: &str = "downloads";

// 变量：TORRENT_API | 含义：全局 torrent API 单例 | 类型：AsyncOnceCell<Api> | 作用域：模块级
// 本行目的：定义异步初始化的 torrent API 单例。
static TORRENT_API: AsyncOnceCell<Api> = AsyncOnceCell::const_new();

// 本行目的：为启动响应结构启用序列化。
#[derive(serde::Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义下载启动响应结构体。
pub struct TorrentStartResponse {
    // 变量：id | 含义：下载任务 ID | 类型：usize | 作用域：TorrentStartResponse
    // 本行目的：记录下载任务 ID。
    pub id: usize,
    // 变量：task_id | 含义：本地任务标识 | 类型：String | 作用域：TorrentStartResponse
    // 本行目的：记录本地任务标识。
    pub task_id: String,
    // 变量：info_hash | 含义：种子哈希 | 类型：String | 作用域：TorrentStartResponse
    // 本行目的：记录种子哈希。
    pub info_hash: String,
    // 变量：name | 含义：种子名称 | 类型：Option<String> | 作用域：TorrentStartResponse
    // 本行目的：记录种子名称。
    pub name: Option<String>,
    // 变量：output_folder | 含义：实际输出目录 | 类型：String | 作用域：TorrentStartResponse
    // 本行目的：记录实际输出目录。
    pub output_folder: String,
    // 变量：final_folder | 含义：最终输出路径 | 类型：String | 作用域：TorrentStartResponse
    // 本行目的：记录最终输出路径。
    pub final_folder: String,
    // 变量：placeholder_path | 含义：占位文件路径 | 类型：String | 作用域：TorrentStartResponse
    // 本行目的：记录占位文件路径。
    pub placeholder_path: String,
// 本行目的：结束启动响应结构体定义。
}

// 本行目的：为状态响应结构启用序列化。
#[derive(serde::Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义下载状态响应结构体。
pub struct TorrentStatusResponse {
    // 变量：id | 含义：下载任务 ID | 类型：usize | 作用域：TorrentStatusResponse
    // 本行目的：记录下载任务 ID。
    pub id: usize,
    // 变量：state | 含义：下载状态 | 类型：String | 作用域：TorrentStatusResponse
    // 本行目的：记录下载状态。
    pub state: String,
    // 变量：progress_bytes | 含义：已下载字节数 | 类型：u64 | 作用域：TorrentStatusResponse
    // 本行目的：记录已下载字节数。
    pub progress_bytes: u64,
    // 变量：total_bytes | 含义：总字节数 | 类型：u64 | 作用域：TorrentStatusResponse
    // 本行目的：记录总字节数。
    pub total_bytes: u64,
    // 变量：uploaded_bytes | 含义：已上传字节数 | 类型：u64 | 作用域：TorrentStatusResponse
    // 本行目的：记录已上传字节数。
    pub uploaded_bytes: u64,
    // 变量：finished | 含义：是否完成 | 类型：bool | 作用域：TorrentStatusResponse
    // 本行目的：记录完成状态。
    pub finished: bool,
    // 变量：error | 含义：错误信息 | 类型：Option<String> | 作用域：TorrentStatusResponse
    // 本行目的：记录错误信息。
    pub error: Option<String>,
    // 变量：download_speed | 含义：下载速度 | 类型：Option<String> | 作用域：TorrentStatusResponse
    // 本行目的：记录下载速度。
    pub download_speed: Option<String>,
    // 变量：upload_speed | 含义：上传速度 | 类型：Option<String> | 作用域：TorrentStatusResponse
    // 本行目的：记录上传速度。
    pub upload_speed: Option<String>,
    // 变量：time_remaining | 含义：剩余时间 | 类型：Option<String> | 作用域：TorrentStatusResponse
    // 本行目的：记录剩余时间。
    pub time_remaining: Option<String>,
// 本行目的：结束状态响应结构体定义。
}

// 本行目的：为只读元数据响应结构启用序列化。
#[derive(serde::Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义种子元数据响应结构体。
pub struct TorrentListOnlyResponse {
    // 变量：info_hash | 含义：种子哈希 | 类型：String | 作用域：TorrentListOnlyResponse
    // 本行目的：记录种子哈希。
    pub info_hash: String,
    // 变量：name | 含义：种子名称 | 类型：Option<String> | 作用域：TorrentListOnlyResponse
    // 本行目的：记录种子名称。
    pub name: Option<String>,
    // 变量：total_bytes | 含义：总字节数 | 类型：u64 | 作用域：TorrentListOnlyResponse
    // 本行目的：记录总字节数。
    pub total_bytes: u64,
// 本行目的：结束元数据响应结构体定义。
}

/// 函数：build_add_target | 输入：种子 URL | 输出：AddTorrent 对象 | 可能失败：网络或读取失败
// 本行目的：根据 URL 构建添加种子对象。
async fn build_add_target(url: &str) -> Result<AddTorrent<'static>, String> {
    // 本行目的：若为 HTTP(S) 则先下载种子文件。
    if url.starts_with("http://") || url.starts_with("https://") {
        // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：build_add_target
        // 本行目的：下载种子文件。
        let response = reqwest::get(url).await.map_err(|e| format!("下载种子失败: {e}"))?;

        // 本行目的：检查响应是否成功。
        if !response.status().is_success() {
            // 本行目的：返回 HTTP 错误。
            return Err(format!("下载种子失败: HTTP {}", response.status()));
        // 本行目的：结束状态检查。
        }

        // 变量：bytes | 含义：种子字节内容 | 类型：bytes::Bytes | 作用域：build_add_target
        // 本行目的：读取响应字节。
        let bytes = response.bytes().await.map_err(|e| format!("读取种子失败: {e}"))?;

        // 本行目的：从字节构建 AddTorrent。
        Ok(AddTorrent::from_bytes(bytes))
    // 本行目的：处理非 HTTP(S) URL。
    } else {
        // 本行目的：直接从 URL 构建 AddTorrent。
        Ok(AddTorrent::from_url(url.to_string()))
    // 本行目的：结束 URL 判断。
    }
// 本行目的：结束 AddTorrent 构建函数。
}

// 本行目的：为下载任务记录结构启用序列化/反序列化与克隆。
#[derive(Serialize, Deserialize, Clone)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义下载任务记录结构体。
pub struct DownloadTaskRecord {
    // 变量：task_id | 含义：任务 ID | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录任务 ID。
    pub task_id: String,
    // 变量：url | 含义：种子 URL | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录种子 URL。
    pub url: String,
    // 变量：name | 含义：任务名称 | 类型：Option<String> | 作用域：DownloadTaskRecord
    // 本行目的：记录任务名称。
    pub name: Option<String>,
    // 变量：output_dir | 含义：输出目录 | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录输出目录。
    pub output_dir: String,
    // 变量：output_path | 含义：输出路径 | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录输出路径。
    pub output_path: String,
    // 变量：temp_folder | 含义：临时目录 | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录临时目录。
    pub temp_folder: String,
    // 变量：placeholder_path | 含义：占位文件路径 | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录占位文件路径。
    pub placeholder_path: String,
    // 变量：total_bytes | 含义：总字节数 | 类型：Option<u64> | 作用域：DownloadTaskRecord
    // 本行目的：记录总字节数。
    pub total_bytes: Option<u64>,
    // 变量：created_at | 含义：创建时间 | 类型：String | 作用域：DownloadTaskRecord
    // 本行目的：记录创建时间。
    pub created_at: String,
// 本行目的：结束下载任务记录结构体定义。
}

/// 函数：tasks_path | 输入：App 句柄引用 | 输出：任务文件路径 | 可能失败：无法创建目录
// 本行目的：获取下载任务记录文件路径。
fn tasks_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 变量：dir | 含义：数据目录路径 | 类型：PathBuf | 作用域：tasks_path
    // 本行目的：解析数据目录并追加子目录。
    let dir = app.path().app_data_dir().map_err(|e| format!("无法获取数据目录: {e}"))?.join("hanamirip-cn");
    // 本行目的：确保目录存在。
    fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
    // 本行目的：返回任务文件路径。
    Ok(dir.join("download-tasks.json"))
// 本行目的：结束任务路径函数。
}

/// 函数：load_tasks | 输入：App 句柄引用 | 输出：任务列表 | 可能失败：读取或解析失败
// 本行目的：加载下载任务列表。
fn load_tasks(app: &tauri::AppHandle) -> Result<Vec<DownloadTaskRecord>, String> {
    // 变量：path | 含义：任务文件路径 | 类型：PathBuf | 作用域：load_tasks
    // 本行目的：获取任务文件路径。
    let path = tasks_path(app)?;

    // 本行目的：若任务文件不存在则返回空列表。
    if !path.exists() {
        // 本行目的：返回空列表。
        return Ok(Vec::new());
    // 本行目的：结束文件存在判断。
    }

    // 变量：content | 含义：任务文件内容 | 类型：String | 作用域：load_tasks
    // 本行目的：读取任务文件内容。
    let content = fs::read_to_string(&path).map_err(|e| format!("读取下载任务失败: {e}"))?;

    // 本行目的：若内容为空则返回空列表。
    if content.trim().is_empty() {
        // 本行目的：返回空列表。
        return Ok(Vec::new());
    // 本行目的：结束空内容判断。
    }

    // 本行目的：解析 JSON 为任务列表。
    serde_json::from_str(&content).map_err(|e| format!("解析下载任务失败: {e}"))
// 本行目的：结束任务加载函数。
}

/// 函数：persist_tasks | 输入：App 句柄与任务列表 | 输出：空结果或错误 | 可能失败：序列化或写入失败
// 本行目的：保存下载任务列表。
fn persist_tasks(app: &tauri::AppHandle, tasks: &[DownloadTaskRecord]) -> Result<(), String> {
    // 变量：path | 含义：任务文件路径 | 类型：PathBuf | 作用域：persist_tasks
    // 本行目的：获取任务文件路径。
    let path = tasks_path(app)?;
    // 变量：payload | 含义：序列化后的 JSON 文本 | 类型：String | 作用域：persist_tasks
    // 本行目的：序列化任务列表。
    let payload = serde_json::to_string_pretty(tasks).map_err(|e| format!("序列化下载任务失败: {e}"))?;
    // 本行目的：写入任务文件并返回结果。
    fs::write(&path, payload).map_err(|e| format!("写入下载任务失败: {e}"))
// 本行目的：结束任务保存函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_torrent_metadata | 输入：App 句柄与种子 URL | 输出：种子元数据响应 | 可能失败：读取种子失败
// 本行目的：获取种子元数据信息。
pub async fn get_torrent_metadata(app: tauri::AppHandle, url: String) -> Result<TorrentListOnlyResponse, String> {
    // 变量：api | 含义：torrent API | 类型：Api | 作用域：get_torrent_metadata
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 变量：opts | 含义：添加种子选项 | 类型：AddTorrentOptions | 作用域：get_torrent_metadata
    // 本行目的：初始化添加种子选项。
    let mut opts = AddTorrentOptions::default();

    // 本行目的：设置为只读取列表信息。
    opts.list_only = true;

    // 变量：add_target | 含义：添加种子对象 | 类型：AddTorrent<'static> | 作用域：get_torrent_metadata
    // 本行目的：构建添加种子对象。
    let add_target = build_add_target(&url).await?;
    // 变量：response | 含义：添加种子响应 | 类型：librqbit::api::AddTorrentResponse | 作用域：get_torrent_metadata
    // 本行目的：调用 API 添加种子并读取元数据。
    let response = api.api_add_torrent(add_target, Some(opts)).await.map_err(|e| format!("读取种子信息失败: {e}"))?;
    // 变量：total_bytes | 含义：总字节数 | 类型：u64 | 作用域：get_torrent_metadata
    // 本行目的：统计文件总大小。
    let total_bytes = response.details.files.as_ref().map(|files| files.iter().map(|file| file.length).sum()).unwrap_or(0);

    // 本行目的：构建并返回元数据响应。
    Ok(TorrentListOnlyResponse {
        // 本行目的：写入种子哈希。
        info_hash: response.details.info_hash,
        // 本行目的：写入种子名称。
        name: response.details.name,
        // 本行目的：写入总字节数。
        total_bytes,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束元数据命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：list_pending_downloads | 输入：App 句柄 | 输出：待处理任务列表 | 可能失败：读取或写入失败
// 本行目的：列出仍存在占位或临时目录的任务。
pub fn list_pending_downloads(app: tauri::AppHandle) -> Result<Vec<DownloadTaskRecord>, String> {
    // 变量：tasks | 含义：任务列表 | 类型：Vec<DownloadTaskRecord> | 作用域：list_pending_downloads
    // 本行目的：加载任务列表。
    let tasks = load_tasks(&app)?;
    // 变量：active | 含义：仍在进行的任务列表 | 类型：Vec<DownloadTaskRecord> | 作用域：list_pending_downloads
    // 本行目的：初始化活动任务列表。
    let mut active = Vec::new();
    // 变量：changed | 含义：是否需要保存更新 | 类型：bool | 作用域：list_pending_downloads
    // 本行目的：初始化变化标记。
    let mut changed = false;

    // 本行目的：遍历任务并筛选仍存在的任务。
    for task in tasks {
        // 变量：placeholder_exists | 含义：占位文件是否存在 | 类型：bool | 作用域：list_pending_downloads
        // 本行目的：检查占位文件是否存在。
        let placeholder_exists = Path::new(&task.placeholder_path).exists();
        // 变量：temp_exists | 含义：临时目录是否存在 | 类型：bool | 作用域：list_pending_downloads
        // 本行目的：检查临时目录是否存在。
        let temp_exists = Path::new(&task.temp_folder).exists();

        // 本行目的：若仍存在相关文件则保留任务。
        if placeholder_exists || temp_exists {
            // 本行目的：加入活动任务列表。
            active.push(task);
        // 本行目的：处理不存在的任务。
        } else {
            // 本行目的：标记需要更新存储。
            changed = true;
        // 本行目的：结束存在判断。
        }
    // 本行目的：结束任务遍历。
    }

    // 本行目的：若列表发生变化则写回存储。
    if changed {
        // 本行目的：持久化活动任务列表。
        persist_tasks(&app, &active)?;
    // 本行目的：结束变化判断。
    }

    // 本行目的：返回活动任务列表。
    Ok(active)
// 本行目的：结束任务列表命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：resume_torrent_download | 输入：App 句柄与任务 ID | 输出：启动响应 | 可能失败：找不到任务或恢复失败
// 本行目的：恢复已存在的下载任务。
pub async fn resume_torrent_download(app: tauri::AppHandle, task_id: String) -> Result<TorrentStartResponse, String> {
    // 变量：tasks | 含义：任务列表 | 类型：Vec<DownloadTaskRecord> | 作用域：resume_torrent_download
    // 本行目的：加载任务列表。
    let tasks = load_tasks(&app)?;
    // 变量：task | 含义：匹配的任务记录 | 类型：DownloadTaskRecord | 作用域：resume_torrent_download
    // 本行目的：查找对应任务记录。
    let task = tasks.iter().find(|item| item.task_id == task_id).ok_or_else(|| "找不到下载任务".to_string())?.clone();

    // 本行目的：创建或清空占位文件以提示下载中。
    if let Ok(file) = fs::OpenOptions::new().create(true).write(true).truncate(true).open(&task.placeholder_path) {
        // 本行目的：若有总大小则设置占位文件长度。
        if let Some(size) = task.total_bytes {
            // 本行目的：仅在尺寸大于 0 时设置长度。
            if size > 0 {
                // 本行目的：忽略设置长度的错误。
                let _ = file.set_len(size);
            // 本行目的：结束大小判断。
            }
        // 本行目的：结束总大小判断。
        }
    // 本行目的：结束占位文件创建判断。
    }

    // 变量：api | 含义：torrent API | 类型：Api | 作用域：resume_torrent_download
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 变量：opts | 含义：添加种子选项 | 类型：AddTorrentOptions | 作用域：resume_torrent_download
    // 本行目的：初始化添加种子选项。
    let mut opts = AddTorrentOptions::default();

    // 本行目的：设置输出目录为临时目录。
    opts.output_folder = Some(task.temp_folder.clone());
    // 本行目的：允许覆盖已有数据。
    opts.overwrite = true;

    // 变量：add_target | 含义：添加种子对象 | 类型：AddTorrent<'static> | 作用域：resume_torrent_download
    // 本行目的：构建添加种子对象。
    let add_target = build_add_target(&task.url).await?;
    // 变量：response | 含义：添加种子响应 | 类型：librqbit::api::AddTorrentResponse | 作用域：resume_torrent_download
    // 本行目的：调用 API 添加种子恢复下载。
    let response = api.api_add_torrent(add_target, Some(opts)).await.map_err(|e| format!("恢复下载失败: {e}"))?;
    // 变量：id | 含义：下载任务 ID | 类型：usize | 作用域：resume_torrent_download
    // 本行目的：获取下载任务 ID。
    let id = response.id.or(response.details.id).ok_or_else(|| "无法获取下载任务 ID".to_string())?;

    // 本行目的：构建并返回启动响应。
    Ok(TorrentStartResponse {
        // 本行目的：写入任务 ID。
        id,
        // 本行目的：写入本地任务 ID。
        task_id: task_id.clone(),
        // 本行目的：写入种子哈希。
        info_hash: response.details.info_hash,
        // 本行目的：写入名称。
        name: response.details.name,
        // 本行目的：写入输出目录。
        output_folder: response.output_folder,
        // 本行目的：写入最终输出路径。
        final_folder: task.output_path,
        // 本行目的：写入占位文件路径。
        placeholder_path: task.placeholder_path,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束恢复下载命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：discard_torrent_download | 输入：App 句柄与任务 ID | 输出：空结果或错误 | 可能失败：读取或写入失败
// 本行目的：丢弃并清理下载任务。
pub fn discard_torrent_download(app: tauri::AppHandle, task_id: String) -> Result<(), String> {
    // 变量：tasks | 含义：任务列表 | 类型：Vec<DownloadTaskRecord> | 作用域：discard_torrent_download
    // 本行目的：加载任务列表。
    let mut tasks = load_tasks(&app)?;
    // 变量：removed | 含义：被移除的任务 | 类型：Option<DownloadTaskRecord> | 作用域：discard_torrent_download
    // 本行目的：初始化移除任务占位。
    let mut removed: Option<DownloadTaskRecord> = None;

    // 本行目的：过滤并移除指定任务。
    tasks.retain(|item| {
        // 本行目的：判断是否为目标任务。
        if item.task_id == task_id {
            // 本行目的：保存被移除任务。
            removed = Some(item.clone());
            // 本行目的：从列表中移除。
            false
        // 本行目的：保留非目标任务。
        } else {
            // 本行目的：保留当前任务。
            true
        // 本行目的：结束任务匹配判断。
        }
    // 本行目的：结束 retain 闭包。
    });

    // 本行目的：持久化更新后的任务列表。
    persist_tasks(&app, &tasks)?;

    // 本行目的：若存在被移除任务则清理文件。
    if let Some(task) = removed {
        // 本行目的：尝试删除占位文件。
        let _ = fs::remove_file(task.placeholder_path);
        // 本行目的：尝试删除临时目录。
        let _ = fs::remove_dir_all(task.temp_folder);
    // 本行目的：结束移除任务判断。
    }

    // 本行目的：返回成功。
    Ok(())
// 本行目的：结束丢弃下载命令。
}

/// 函数：default_tracker_set | 输入：无 | 输出：默认 tracker 集合 | 可能失败：无
// 本行目的：构建默认 tracker URL 集合。
fn default_tracker_set() -> HashSet<url::Url> {
    // 本行目的：定义 tracker URL 列表并解析为集合。
    [
        // 本行目的：tracker 地址。
        "udp://tracker.openbittorrent.com:80/announce",
        // 本行目的：tracker 地址。
        "udp://tracker.opentrackr.org:1337/announce",
        // 本行目的：tracker 地址。
        "udp://tracker.internetwarriors.net:1337/announce",
        // 本行目的：tracker 地址。
        "https://tracker2.ctix.cn/announce",
        // 本行目的：tracker 地址。
        "udp://tracker.torrent.eu.org:451/announce",
        // 本行目的：tracker 地址。
        "udp://tracker.moeking.me:6969/announce",
        // 本行目的：tracker 地址。
        "udp://tracker.bittor.pw:1337/announce",
        // 本行目的：tracker 地址。
        "udp://tracker1.bt.moack.co:80/announce",
        // 本行目的：tracker 地址。
        "udp://open.tracker.cl:1337/announce",
        // 本行目的：tracker 地址。
        "udp://exodus.desync.com:6969/announce",
        // 本行目的：tracker 地址。
        "udp://tracker-udp.gbitt.info:80/announce",
        // 本行目的：tracker 地址。
        "https://tracker1.520.jp/announce",
        // 本行目的：tracker 地址。
        "https://tracker2.itzmx.com/announce",
        // 本行目的：tracker 地址。
        "https://tracker.gbitt.info/announce",
    // 本行目的：结束 tracker 列表。
    ]
    // 本行目的：转换为迭代器。
    .into_iter()
    // 本行目的：解析 URL 并过滤失败项。
    .filter_map(|url| url::Url::parse(url).ok())
    // 本行目的：收集为集合。
    .collect()
// 本行目的：结束默认 tracker 集合函数。
}

/// 函数：torrent_api | 输入：App 句柄引用 | 输出：torrent API 实例 | 可能失败：初始化会话失败
// 本行目的：初始化并返回 torrent API 单例。
async fn torrent_api(app: &tauri::AppHandle) -> Result<Api, String> {
    // 本行目的：初始化或获取 API 单例。
    TORRENT_API
        // 本行目的：按需初始化 API。
        .get_or_try_init(|| {
            // 本行目的：创建初始化异步块。
            async {
                // 变量：base_dir | 含义：下载根目录 | 类型：PathBuf | 作用域：torrent_api
                // 本行目的：解析下载根目录。
                let base_dir = app.path().app_data_dir().map_err(|e| format!("无法获取下载目录: {e}"))?.join("hanamirip-cn").join(TORRENT_OUTPUT_DIR);
                // 本行目的：确保下载目录存在。
                fs::create_dir_all(&base_dir).map_err(|e| format!("创建下载目录失败: {e}"))?;

                // 变量：opts | 含义：会话选项 | 类型：SessionOptions | 作用域：torrent_api
                // 本行目的：初始化会话选项。
                let mut opts = SessionOptions::default();
                // 本行目的：设置监听端口范围。
                opts.listen_port_range = Some(40000 .. 50000);
                // 本行目的：启用 UPnP 端口映射。
                opts.enable_upnp_port_forwarding = true;
                // 本行目的：设置对等连接选项。
                opts.peer_opts = Some(PeerConnectionOptions {
                    // 本行目的：设置连接超时。
                    connect_timeout: Some(Duration::from_secs(4)),
                    // 本行目的：设置读写超时。
                    read_write_timeout: Some(Duration::from_secs(12)),
                    // 本行目的：设置保活间隔。
                    keep_alive_interval: Some(Duration::from_secs(45)),
                // 本行目的：结束对等连接选项。
                });
                // 本行目的：设置延迟写入阈值。
                opts.defer_writes_up_to = Some(256);
                // 本行目的：设置并发初始化限制。
                opts.concurrent_init_limit = Some(3);
                // 本行目的：设置 tracker 列表。
                opts.trackers = default_tracker_set();

                // 变量：session | 含义：下载会话 | 类型：Session | 作用域：torrent_api
                // 本行目的：创建下载会话。
                let session = Session::new_with_opts(base_dir, opts).await.map_err(|e| format!("初始化下载会话失败: {e}"))?;
                // 本行目的：构建 API 实例并返回。
                Ok(Api::new(session, None))
            // 本行目的：结束初始化异步块。
            }
        // 本行目的：结束 get_or_try_init 调用。
        })
        // 本行目的：等待单例初始化完成。
        .await
        // 本行目的：克隆 API 实例返回。
        .map(Clone::clone)
// 本行目的：结束 API 获取函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：start_torrent_download | 输入：App 句柄、URL、输出目录与路径、总大小 | 输出：启动响应 | 可能失败：添加或写入失败
// 本行目的：创建并启动下载任务。
pub async fn start_torrent_download(
    // 变量：app | 含义：App 句柄 | 类型：tauri::AppHandle | 作用域：start_torrent_download
    // 本行目的：传入 App 句柄。
    app: tauri::AppHandle,
    // 变量：url | 含义：种子 URL | 类型：String | 作用域：start_torrent_download
    // 本行目的：传入种子 URL。
    url: String,
    // 变量：output_dir | 含义：输出目录 | 类型：String | 作用域：start_torrent_download
    // 本行目的：传入输出目录。
    output_dir: String,
    // 变量：output_path | 含义：输出路径 | 类型：String | 作用域：start_torrent_download
    // 本行目的：传入输出路径。
    output_path: String,
    // 变量：total_bytes | 含义：总字节数 | 类型：Option<u64> | 作用域：start_torrent_download
    // 本行目的：传入总字节数。
    total_bytes: Option<u64>,
// 本行目的：结束函数签名。
) -> Result<TorrentStartResponse, String> {
    // 变量：api | 含义：torrent API | 类型：Api | 作用域：start_torrent_download
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 变量：output_folder | 含义：输出目录路径 | 类型：PathBuf | 作用域：start_torrent_download
    // 本行目的：构建输出目录路径。
    let output_folder = PathBuf::from(&output_dir);

    // 本行目的：确保输出目录存在。
    fs::create_dir_all(&output_folder).map_err(|e| format!("创建下载目录失败: {e}"))?;

    // 变量：placeholder_path | 含义：占位文件路径 | 类型：PathBuf | 作用域：start_torrent_download
    // 本行目的：构建占位文件路径。
    let placeholder_path = PathBuf::from(&output_path).with_extension(format!(
        // 本行目的：拼接原扩展名与占位后缀。
        "{}{}",
        // 本行目的：读取原扩展名并追加点号。
        Path::new(&output_path).extension().and_then(|ext| ext.to_str()).map(|ext| format!("{}.", ext)).unwrap_or_default(),
        // 本行目的：追加占位后缀。
        "hanamirip"
    // 本行目的：结束扩展名拼接。
    ));

    // 本行目的：创建或清空占位文件。
    if let Ok(file) = fs::OpenOptions::new().create(true).write(true).truncate(true).open(&placeholder_path) {
        // 本行目的：若有总大小则设置占位文件长度。
        if let Some(size) = total_bytes {
            // 本行目的：仅在尺寸大于 0 时设置长度。
            if size > 0 {
                // 本行目的：忽略设置长度的错误。
                let _ = file.set_len(size);
            // 本行目的：结束大小判断。
            }
        // 本行目的：结束总大小判断。
        }
    // 本行目的：结束占位文件创建判断。
    }

    // 变量：downloading_dir | 含义：临时下载目录 | 类型：PathBuf | 作用域：start_torrent_download
    // 本行目的：构建临时下载目录路径。
    let downloading_dir = output_folder.join(".downloading");

    // 本行目的：创建临时下载目录。
    fs::create_dir_all(&downloading_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;
    // 本行目的：将临时目录设为隐藏。
    set_hidden_dir(&downloading_dir);

    // 变量：task_id | 含义：任务 ID | 类型：String | 作用域：start_torrent_download
    // 本行目的：生成任务 ID。
    let task_id = Utc::now().timestamp_millis().to_string();
    // 变量：temp_folder | 含义：任务临时目录 | 类型：PathBuf | 作用域：start_torrent_download
    // 本行目的：构建任务临时目录路径。
    let temp_folder = downloading_dir.join(&task_id);

    // 本行目的：创建任务临时目录。
    fs::create_dir_all(&temp_folder).map_err(|e| format!("创建临时下载目录失败: {e}"))?;

    // 变量：opts | 含义：添加种子选项 | 类型：AddTorrentOptions | 作用域：start_torrent_download
    // 本行目的：初始化添加种子选项。
    let mut opts = AddTorrentOptions::default();
    // 变量：output_folder_str | 含义：输出目录字符串 | 类型：String | 作用域：start_torrent_download
    // 本行目的：将临时目录转换为字符串。
    let output_folder_str = temp_folder.to_string_lossy().to_string();

    // 本行目的：设置输出目录。
    opts.output_folder = Some(output_folder_str.clone());
    // 本行目的：允许覆盖已有数据。
    opts.overwrite = true;

    // 变量：add_target | 含义：添加种子对象 | 类型：AddTorrent<'static> | 作用域：start_torrent_download
    // 本行目的：构建添加种子对象。
    let add_target = build_add_target(&url).await?;
    // 变量：response | 含义：添加种子响应 | 类型：librqbit::api::AddTorrentResponse | 作用域：start_torrent_download
    // 本行目的：调用 API 添加种子。
    let mut response = api.api_add_torrent(add_target, Some(opts)).await.map_err(|e| format!("添加下载失败: {e}"))?;

    // 本行目的：若输出目录不一致则重试添加。
    if PathBuf::from(&response.output_folder) != PathBuf::from(&output_folder_str) {
        // 变量：existing_id | 含义：已存在任务 ID | 类型：usize | 作用域：start_torrent_download
        // 本行目的：获取已存在任务 ID。
        let existing_id = response.id.or(response.details.id).ok_or_else(|| "无法获取下载任务 ID".to_string())?;

        // 本行目的：移除已存在任务以重试。
        api.api_torrent_action_forget(TorrentIdOrHash::Id(existing_id)).await.map_err(|e| format!("移除已存在任务失败: {e}"))?;

        // 变量：retry_opts | 含义：重试添加选项 | 类型：AddTorrentOptions | 作用域：start_torrent_download
        // 本行目的：初始化重试选项。
        let mut retry_opts = AddTorrentOptions::default();

        // 本行目的：设置输出目录。
        retry_opts.output_folder = Some(output_folder_str.clone());
        // 本行目的：允许覆盖已有数据。
        retry_opts.overwrite = true;

        // 变量：retry_target | 含义：重试添加种子对象 | 类型：AddTorrent<'static> | 作用域：start_torrent_download
        // 本行目的：构建重试添加对象。
        let retry_target = build_add_target(&url).await?;

        // 本行目的：重试添加种子。
        response = api.api_add_torrent(retry_target, Some(retry_opts)).await.map_err(|e| format!("添加下载失败: {e}"))?;
    // 本行目的：结束输出目录一致性判断。
    }

    // 变量：id | 含义：下载任务 ID | 类型：usize | 作用域：start_torrent_download
    // 本行目的：获取下载任务 ID。
    let id = response.id.or(response.details.id).ok_or_else(|| "无法获取下载任务 ID".to_string())?;
    // 变量：tasks | 含义：任务列表 | 类型：Vec<DownloadTaskRecord> | 作用域：start_torrent_download
    // 本行目的：加载任务列表。
    let mut tasks = load_tasks(&app)?;

    // 本行目的：追加新的任务记录。
    tasks.push(DownloadTaskRecord {
        // 本行目的：写入任务 ID。
        task_id: task_id.clone(),
        // 本行目的：写入 URL。
        url,
        // 本行目的：写入任务名称。
        name: response.details.name.clone(),
        // 本行目的：写入输出目录。
        output_dir: output_folder.to_string_lossy().to_string(),
        // 本行目的：写入输出路径。
        output_path: output_path.clone(),
        // 本行目的：写入临时目录。
        temp_folder: temp_folder.to_string_lossy().to_string(),
        // 本行目的：写入占位文件路径。
        placeholder_path: placeholder_path.to_string_lossy().to_string(),
        // 本行目的：写入总字节数。
        total_bytes,
        // 本行目的：写入创建时间。
        created_at: Utc::now().to_rfc3339(),
    // 本行目的：结束任务记录构建。
    });

    // 本行目的：持久化任务列表。
    persist_tasks(&app, &tasks)?;

    // 本行目的：构建并返回启动响应。
    Ok(TorrentStartResponse {
        // 本行目的：写入任务 ID。
        id,
        // 本行目的：写入本地任务 ID。
        task_id,
        // 本行目的：写入种子哈希。
        info_hash: response.details.info_hash,
        // 本行目的：写入名称。
        name: response.details.name,
        // 本行目的：写入输出目录。
        output_folder: response.output_folder,
        // 本行目的：写入最终输出路径。
        final_folder: output_path,
        // 本行目的：写入占位文件路径。
        placeholder_path: placeholder_path.to_string_lossy().to_string(),
    // 本行目的：结束响应构建。
    })
// 本行目的：结束启动下载命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：finalize_torrent_download | 输入：App 句柄、临时目录、最终路径、占位路径 | 输出：空结果或错误 | 可能失败：文件操作失败
// 本行目的：将临时下载内容移动到最终目录并清理。
pub fn finalize_torrent_download(app: tauri::AppHandle, temp_folder: String, final_path: String, placeholder_path: String) -> Result<(), String> {
    // 变量：temp_path | 含义：临时目录路径 | 类型：PathBuf | 作用域：finalize_torrent_download
    // 本行目的：构建临时目录路径。
    let temp_path = PathBuf::from(temp_folder);

    // 本行目的：若临时目录不存在则直接返回。
    if !temp_path.exists() {
        // 本行目的：返回成功。
        return Ok(());
    // 本行目的：结束存在判断。
    }

    // 变量：entries | 含义：临时目录条目列表 | 类型：Vec<DirEntry> | 作用域：finalize_torrent_download
    // 本行目的：读取临时目录条目。
    let entries: Vec<_> = fs::read_dir(&temp_path).map_err(|e| format!("读取临时目录失败: {e}"))?.filter_map(|entry| entry.ok()).collect();
    // 变量：final_path | 含义：最终路径 | 类型：PathBuf | 作用域：finalize_torrent_download
    // 本行目的：构建最终路径。
    let final_path = PathBuf::from(final_path);

    // 本行目的：若只有一个文件则直接移动为最终文件。
    if entries.len() == 1 && entries[0].path().is_file() {
        // 本行目的：确保最终目录存在。
        if let Some(parent) = final_path.parent() {
            // 本行目的：创建最终目录。
            fs::create_dir_all(parent).map_err(|e| format!("创建目标目录失败: {e}"))?;
        // 本行目的：结束父目录判断。
        }

        // 本行目的：移动临时文件到最终路径。
        fs::rename(entries[0].path(), &final_path).map_err(|e| format!("移动下载文件失败: {e}"))?;
    // 本行目的：处理多文件或目录情况。
    } else {
        // 本行目的：创建最终目录。
        fs::create_dir_all(&final_path).map_err(|e| format!("创建目标目录失败: {e}"))?;

        // 本行目的：遍历条目并移动到最终目录。
        for entry in entries {
            // 变量：dest | 含义：目标路径 | 类型：PathBuf | 作用域：finalize_torrent_download
            // 本行目的：构建目标路径。
            let dest = final_path.join(entry.file_name());
            // 本行目的：移动条目到目标路径。
            fs::rename(entry.path(), dest).map_err(|e| format!("移动下载文件失败: {e}"))?;
        // 本行目的：结束条目遍历。
        }
    // 本行目的：结束单文件/多文件判断。
    }

    // 本行目的：删除占位文件，忽略错误。
    let _ = fs::remove_file(&placeholder_path);

    // 本行目的：删除临时目录。
    fs::remove_dir_all(&temp_path).map_err(|e| format!("清理临时目录失败: {e}"))?;

    // 本行目的：如父目录为空则清理。
    if let Some(parent) = temp_path.parent() {
        // 本行目的：检查父目录是否为空。
        if is_dir_empty(parent) {
            // 本行目的：删除空父目录，忽略错误。
            let _ = fs::remove_dir_all(parent);
        // 本行目的：结束父目录为空判断。
        }
    // 本行目的：结束父目录判断。
    }

    // 变量：tasks | 含义：任务列表 | 类型：Vec<DownloadTaskRecord> | 作用域：finalize_torrent_download
    // 本行目的：加载任务列表。
    let mut tasks = load_tasks(&app)?;

    // 本行目的：移除已完成任务记录。
    tasks.retain(|item| item.placeholder_path != placeholder_path);
    // 本行目的：持久化任务列表。
    persist_tasks(&app, &tasks)?;

    // 本行目的：返回成功。
    Ok(())
// 本行目的：结束完成下载命令。
}

/// 函数：is_dir_empty | 输入：目录路径 | 输出：是否为空 | 可能失败：无
// 本行目的：检查目录是否为空。
fn is_dir_empty(path: &Path) -> bool {
    // 本行目的：读取目录并判断是否有条目。
    fs::read_dir(path).map(|mut iter| iter.next().is_none()).unwrap_or(false)
// 本行目的：结束空目录判断函数。
}

/// 函数：set_hidden_dir | 输入：目录路径 | 输出：无 | 可能失败：无
// 本行目的：在 Windows 上设置目录为隐藏。
fn set_hidden_dir(path: &Path) {
    // 本行目的：仅在 Windows 平台执行。
    #[cfg(target_os = "windows")]
    // 本行目的：进入 Windows 平台作用域。
    {
        // 本行目的：调用 attrib 设置隐藏属性，忽略错误。
        let _ = Command::new("cmd").args(["/C", "attrib", "+h", path.to_string_lossy().as_ref()]).status();
    // 本行目的：结束 Windows 平台作用域。
    }
// 本行目的：结束设置隐藏目录函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_torrent_status | 输入：App 句柄与任务 ID | 输出：状态响应 | 可能失败：获取状态失败
// 本行目的：获取下载状态信息。
pub async fn get_torrent_status(app: tauri::AppHandle, id: usize) -> Result<TorrentStatusResponse, String> {
    // 变量：api | 含义：torrent API | 类型：Api | 作用域：get_torrent_status
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 变量：stats | 含义：下载状态 | 类型：librqbit::api::TorrentStats | 作用域：get_torrent_status
    // 本行目的：查询下载状态。
    let stats = api.api_stats_v1(TorrentIdOrHash::Id(id)).map_err(|e| format!("获取下载状态失败: {e}"))?;
    // 变量：(download_speed, upload_speed, time_remaining) | 含义：速度与剩余时间 | 类型：(Option<String>, Option<String>, Option<String>) | 作用域：get_torrent_status
    // 本行目的：从实时数据中提取速度与剩余时间。
    let (download_speed, upload_speed, time_remaining) = stats
        // 本行目的：读取实时数据。
        .live
        // 本行目的：转换为可选引用。
        .as_ref()
        // 本行目的：映射为速度与时间。
        .map(|live| {
            // 本行目的：构建速度与剩余时间元组。
            (
                // 本行目的：格式化下载速度。
                Some(format!("{}", live.download_speed)),
                // 本行目的：格式化上传速度。
                Some(format!("{}", live.upload_speed)),
                // 本行目的：格式化剩余时间。
                live.time_remaining.as_ref().map(|v| v.to_string()),
            )
        // 本行目的：结束映射闭包。
        })
        // 本行目的：无实时数据时返回空值。
        .unwrap_or((None, None, None));

    // 本行目的：构建并返回状态响应。
    Ok(TorrentStatusResponse {
        // 本行目的：写入任务 ID。
        id,
        // 本行目的：写入状态字符串。
        state: format!("{:?}", stats.state),
        // 本行目的：写入已下载字节数。
        progress_bytes: stats.progress_bytes,
        // 本行目的：写入总字节数。
        total_bytes: stats.total_bytes,
        // 本行目的：写入已上传字节数。
        uploaded_bytes: stats.uploaded_bytes,
        // 本行目的：写入完成状态。
        finished: stats.finished,
        // 本行目的：写入错误信息。
        error: stats.error,
        // 本行目的：写入下载速度。
        download_speed,
        // 本行目的：写入上传速度。
        upload_speed,
        // 本行目的：写入剩余时间。
        time_remaining,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束状态查询命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：pause_torrent | 输入：App 句柄与任务 ID | 输出：空结果或错误 | 可能失败：暂停失败
// 本行目的：暂停下载任务。
pub async fn pause_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
    // 变量：api | 含义：torrent API | 类型：Api | 作用域：pause_torrent
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 本行目的：调用暂停动作。
    api.api_torrent_action_pause(TorrentIdOrHash::Id(id)).await.map_err(|e| format!("暂停下载失败: {e}"))?;
    // 本行目的：返回成功。
    Ok(())
// 本行目的：结束暂停命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：resume_torrent | 输入：App 句柄与任务 ID | 输出：空结果或错误 | 可能失败：继续失败
// 本行目的：继续下载任务。
pub async fn resume_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
    // 变量：api | 含义：torrent API | 类型：Api | 作用域：resume_torrent
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 本行目的：调用继续动作。
    api.api_torrent_action_start(TorrentIdOrHash::Id(id)).await.map_err(|e| format!("继续下载失败: {e}"))?;
    // 本行目的：返回成功。
    Ok(())
// 本行目的：结束继续命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：delete_torrent | 输入：App 句柄与任务 ID | 输出：空结果或错误 | 可能失败：删除失败
// 本行目的：删除下载任务。
pub async fn delete_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
    // 变量：api | 含义：torrent API | 类型：Api | 作用域：delete_torrent
    // 本行目的：获取 torrent API 实例。
    let api = torrent_api(&app).await?;
    // 本行目的：调用删除动作。
    api.api_torrent_action_delete(TorrentIdOrHash::Id(id)).await.map_err(|e| format!("删除下载失败: {e}"))?;
    // 本行目的：返回成功。
    Ok(())
// 本行目的：结束删除命令。
}
