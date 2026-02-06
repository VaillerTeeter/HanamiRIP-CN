/*
  种子下载模块：
  - 使用 librqbit 作为下载引擎；
  - 提供开始/暂停/继续/删除等命令；
  - 管理临时下载目录与最终输出目录。
*/

use chrono::Utc;
use librqbit::api::{Api, TorrentIdOrHash};
use librqbit::{AddTorrent, AddTorrentOptions, PeerConnectionOptions, Session, SessionOptions};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::Manager;
use tokio::sync::OnceCell as AsyncOnceCell;

// 默认下载目录名（相对于应用数据目录）。
const TORRENT_OUTPUT_DIR: &str = "downloads";

// 全局共享的下载 API（异步懒加载，线程安全）。
static TORRENT_API: AsyncOnceCell<Api> = AsyncOnceCell::const_new();

/// 开始下载后返回给前端的信息。
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStartResponse {
  pub id: usize,
  pub info_hash: String,
  pub name: Option<String>,
  pub output_folder: String,
  pub final_folder: String,
}

/// 下载状态信息。
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStatusResponse {
  pub id: usize,
  pub state: String,
  pub progress_bytes: u64,
  pub total_bytes: u64,
  pub uploaded_bytes: u64,
  pub finished: bool,
  pub error: Option<String>,
  pub download_speed: Option<String>,
  pub upload_speed: Option<String>,
  pub time_remaining: Option<String>,
}

// 默认 tracker 列表（用于提高种子可连接性）。
fn default_tracker_set() -> HashSet<url::Url> {
  [
    "udp://tracker.openbittorrent.com:80/announce",
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.internetwarriors.net:1337/announce",
    "udp://tracker.coppersurfer.tk:6969/announce",
    "udp://tracker.cyberia.is:6969/announce",
    "https://tracker2.ctix.cn/announce",
  ]
  .into_iter()
  .filter_map(|url| url::Url::parse(url).ok())
  .collect()
}

// 懒加载 Torrent API：首次调用时初始化 session，之后复用。
async fn torrent_api(app: &tauri::AppHandle) -> Result<Api, String> {
  TORRENT_API
    .get_or_try_init(|| async {
      // 下载文件保存目录（应用数据目录 / hanamirip-cn / downloads）。
      let base_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取下载目录: {e}"))?
        .join("hanamirip-cn")
        .join(TORRENT_OUTPUT_DIR);
      fs::create_dir_all(&base_dir).map_err(|e| format!("创建下载目录失败: {e}"))?;

      // 配置下载会话：端口、UPnP、超时、并发等。
      let mut opts = SessionOptions::default();
      opts.listen_port_range = Some(40000..50000);
      opts.enable_upnp_port_forwarding = true;
      opts.peer_opts = Some(PeerConnectionOptions {
        connect_timeout: Some(Duration::from_secs(5)),
        read_write_timeout: Some(Duration::from_secs(15)),
        keep_alive_interval: Some(Duration::from_secs(60)),
      });
      opts.defer_writes_up_to = Some(128);
      opts.concurrent_init_limit = Some(8);
      opts.trackers = default_tracker_set();

      // 创建下载会话。
      let session = Session::new_with_opts(base_dir, opts)
        .await
        .map_err(|e| format!("初始化下载会话失败: {e}"))?;
      Ok(Api::new(session, None))
    })
    .await
    .map(Clone::clone)
}

// 开始一个新的下载任务。
// 会把文件先下载到临时目录，完成后再移动到最终目录。
#[tauri::command]
pub async fn start_torrent_download(
  app: tauri::AppHandle,
  url: String,
  output_dir: String,
) -> Result<TorrentStartResponse, String> {
  // 获取下载 API（如未初始化则初始化）。
  let api = torrent_api(&app).await?;
  // 最终输出目录由前端决定。
  let final_folder = PathBuf::from(&output_dir);
  // 临时目录：放在最终目录下的 .downloading 子目录。
  let temp_folder = final_folder
    .join(".downloading")
    .join(Utc::now().timestamp_millis().to_string());
  fs::create_dir_all(&temp_folder).map_err(|e| format!("创建临时下载目录失败: {e}"))?;
  // 下载选项：指定输出目录并允许覆盖。
  let mut opts = AddTorrentOptions::default();
  opts.output_folder = Some(temp_folder.to_string_lossy().to_string());
  opts.overwrite = true;

  // 添加下载任务。
  let response = api
    .api_add_torrent(AddTorrent::from_url(url), Some(opts))
    .await
    .map_err(|e| format!("添加下载失败: {e}"))?;

  // 任务 ID 可能在不同字段里，做一次兜底处理。
  let id = response
    .id
    .or(response.details.id)
    .ok_or_else(|| "无法获取下载任务 ID".to_string())?;

  Ok(TorrentStartResponse {
    id,
    info_hash: response.details.info_hash,
    name: response.details.name,
    output_folder: response.output_folder,
    final_folder: final_folder.to_string_lossy().to_string(),
  })
}

// 将临时目录内文件移动到最终目录，并清理临时目录。
#[tauri::command]
pub fn finalize_torrent_download(temp_folder: String, final_folder: String) -> Result<(), String> {
  let temp_path = PathBuf::from(temp_folder);
  let final_path = PathBuf::from(final_folder);
  // 临时目录不存在，说明无需处理。
  if !temp_path.exists() {
    return Ok(());
  }
  // 确保最终目录存在。
  fs::create_dir_all(&final_path).map_err(|e| format!("创建最终目录失败: {e}"))?;

  // 把临时目录里的文件移动到最终目录。
  for entry in fs::read_dir(&temp_path).map_err(|e| format!("读取临时目录失败: {e}"))? {
    let entry = entry.map_err(|e| format!("读取临时目录失败: {e}"))?;
    let dest = final_path.join(entry.file_name());
    fs::rename(entry.path(), dest).map_err(|e| format!("移动下载文件失败: {e}"))?;
  }

  fs::remove_dir_all(&temp_path).map_err(|e| format!("清理临时目录失败: {e}"))?;
  Ok(())
}

// 查询下载状态。
#[tauri::command]
pub async fn get_torrent_status(
  app: tauri::AppHandle,
  id: usize,
) -> Result<TorrentStatusResponse, String> {
  let api = torrent_api(&app).await?;
  // 读取下载统计信息。
  let stats = api
    .api_stats_v1(TorrentIdOrHash::Id(id))
    .map_err(|e| format!("获取下载状态失败: {e}"))?;

  // 可能没有实时数据，所以这里用 Option 包裹。
  let (download_speed, upload_speed, time_remaining) = stats
    .live
    .as_ref()
    .map(|live| {
      (
        Some(format!("{}", live.download_speed)),
        Some(format!("{}", live.upload_speed)),
        live.time_remaining.as_ref().map(|v| v.to_string()),
      )
    })
    .unwrap_or((None, None, None));

  Ok(TorrentStatusResponse {
    id,
    state: format!("{:?}", stats.state),
    progress_bytes: stats.progress_bytes,
    total_bytes: stats.total_bytes,
    uploaded_bytes: stats.uploaded_bytes,
    finished: stats.finished,
    error: stats.error,
    download_speed,
    upload_speed,
    time_remaining,
  })
}

// 暂停下载任务。
#[tauri::command]
pub async fn pause_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_pause(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("暂停下载失败: {e}"))?;
  Ok(())
}

// 继续下载任务。
#[tauri::command]
pub async fn resume_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_start(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("继续下载失败: {e}"))?;
  Ok(())
}

// 删除下载任务（只删除任务，不一定删除文件）。
#[tauri::command]
pub async fn delete_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_delete(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("删除下载失败: {e}"))?;
  Ok(())
}
