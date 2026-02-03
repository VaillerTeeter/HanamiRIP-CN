use chrono::Utc;
use librqbit::api::{Api, TorrentIdOrHash};
use librqbit::{AddTorrent, AddTorrentOptions, PeerConnectionOptions, Session, SessionOptions};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::Manager;
use tokio::sync::OnceCell as AsyncOnceCell;

const TORRENT_OUTPUT_DIR: &str = "downloads";

static TORRENT_API: AsyncOnceCell<Api> = AsyncOnceCell::const_new();

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStartResponse {
  pub id: usize,
  pub info_hash: String,
  pub name: Option<String>,
  pub output_folder: String,
  pub final_folder: String,
}

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

async fn torrent_api(app: &tauri::AppHandle) -> Result<Api, String> {
  TORRENT_API
    .get_or_try_init(|| async {
      let base_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取下载目录: {e}"))?
        .join("hanamirip-cn")
        .join(TORRENT_OUTPUT_DIR);
      fs::create_dir_all(&base_dir).map_err(|e| format!("创建下载目录失败: {e}"))?;

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

      let session = Session::new_with_opts(base_dir, opts)
        .await
        .map_err(|e| format!("初始化下载会话失败: {e}"))?;
      Ok(Api::new(session, None))
    })
    .await
    .map(Clone::clone)
}

#[tauri::command]
pub async fn start_torrent_download(
  app: tauri::AppHandle,
  url: String,
  output_dir: String,
) -> Result<TorrentStartResponse, String> {
  let api = torrent_api(&app).await?;
  let final_folder = PathBuf::from(&output_dir);
  let temp_folder = final_folder
    .join(".downloading")
    .join(Utc::now().timestamp_millis().to_string());
  fs::create_dir_all(&temp_folder).map_err(|e| format!("创建临时下载目录失败: {e}"))?;
  let mut opts = AddTorrentOptions::default();
  opts.output_folder = Some(temp_folder.to_string_lossy().to_string());
  opts.overwrite = true;

  let response = api
    .api_add_torrent(AddTorrent::from_url(url), Some(opts))
    .await
    .map_err(|e| format!("添加下载失败: {e}"))?;

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

#[tauri::command]
pub fn finalize_torrent_download(temp_folder: String, final_folder: String) -> Result<(), String> {
  let temp_path = PathBuf::from(temp_folder);
  let final_path = PathBuf::from(final_folder);
  if !temp_path.exists() {
    return Ok(());
  }
  fs::create_dir_all(&final_path).map_err(|e| format!("创建最终目录失败: {e}"))?;

  for entry in fs::read_dir(&temp_path).map_err(|e| format!("读取临时目录失败: {e}"))? {
    let entry = entry.map_err(|e| format!("读取临时目录失败: {e}"))?;
    let dest = final_path.join(entry.file_name());
    fs::rename(entry.path(), dest).map_err(|e| format!("移动下载文件失败: {e}"))?;
  }

  fs::remove_dir_all(&temp_path).map_err(|e| format!("清理临时目录失败: {e}"))?;
  Ok(())
}

#[tauri::command]
pub async fn get_torrent_status(
  app: tauri::AppHandle,
  id: usize,
) -> Result<TorrentStatusResponse, String> {
  let api = torrent_api(&app).await?;
  let stats = api
    .api_stats_v1(TorrentIdOrHash::Id(id))
    .map_err(|e| format!("获取下载状态失败: {e}"))?;

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

#[tauri::command]
pub async fn pause_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_pause(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("暂停下载失败: {e}"))?;
  Ok(())
}

#[tauri::command]
pub async fn resume_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_start(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("继续下载失败: {e}"))?;
  Ok(())
}

#[tauri::command]
pub async fn delete_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_delete(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("删除下载失败: {e}"))?;
  Ok(())
}