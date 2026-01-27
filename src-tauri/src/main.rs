#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{NaiveDate, Utc};
use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use std::env;
use std::os::raw::c_char;
use std::path::Path;
use std::time::Duration;
use open;
use tauri_plugin_dialog::init as dialog_plugin;
use tokio::sync::OnceCell as AsyncOnceCell;
use librqbit::api::{Api, TorrentIdOrHash};
use librqbit::{AddTorrent, AddTorrentOptions, PeerConnectionOptions, Session, SessionOptions};
use tokio::process::Command;

const API_BASE: &str = "https://api.bgm.tv";
const SUBJECTS_PATH: &str = "/v0/subjects";
const EPISODES_PATH: &str = "/v0/episodes";
const DEFAULT_IMAGE: &str = "https://lain.bgm.tv/img/no_icon_subject.png";
const TRACK_DB_FILE: &str = "watchlist.json";
const TORRENT_OUTPUT_DIR: &str = "downloads";

static TORRENT_API: AsyncOnceCell<Api> = AsyncOnceCell::const_new();

#[derive(Deserialize)]
struct PagedSubject {
  total: u32,
  limit: u32,
  data: Vec<Subject>,
}

#[derive(Deserialize)]
struct Subject {
  id: u32,
  name: String,
  name_cn: String,
  date: Option<String>,
  images: Option<Images>,
  summary: Option<String>,
  rating: Option<Rating>,
}

#[derive(Deserialize)]
struct Images {
  common: Option<String>,
  medium: Option<String>,
  large: Option<String>,
}

#[derive(Deserialize)]
struct Rating {
  score: Option<f64>,
}

#[derive(Deserialize)]
struct EpisodePage {
  total: u32,
  limit: u32,
  data: Vec<Episode>,
}

#[derive(Deserialize)]
struct Episode {
  airdate: Option<String>,
  status: Option<String>,
}

#[derive(Deserialize)]
struct SubjectDetail {
  summary: Option<String>,
  infobox: Option<Vec<InfoboxItem>>,
  tags: Option<Vec<SubjectTag>>,
  meta_tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct SubjectTag {
  name: String,
}

#[derive(Deserialize)]
struct InfoboxItem {
  key: String,
  value: Value,
}

#[derive(Deserialize)]
struct SubjectPersonItem {
  id: u32,
  name: String,
  relation: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SubjectPersonPayload {
  List(Vec<SubjectPersonItem>),
  Page {
    _total: u32,
    _limit: u32,
    data: Vec<SubjectPersonItem>,
  },
}

#[derive(Deserialize)]
struct SubjectCharacterItem {
  id: u32,
  name: String,
  name_cn: Option<String>,
  relation: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SubjectCharacterPayload {
  List(Vec<SubjectCharacterItem>),
  Page {
    _total: u32,
    _limit: u32,
    data: Vec<SubjectCharacterItem>,
  },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeasonAnime {
  id: u32,
  name: String,
  name_cn: String,
  image: String,
  date: String,
  rating: Option<f64>,
  summary: String,
  url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeasonMonth {
  year: u32,
  month: u32,
  count: usize,
  list: Vec<SeasonAnime>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeasonResponse {
  year: u32,
  season: String,
  fetched_at: String,
  source: String,
  months: Vec<SeasonMonth>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectOriginResponse {
  id: u32,
  origin: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectAiredResponse {
  id: u32,
  aired_count: u32,
  total_count: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StaffPersonResponse {
  id: u32,
  name: String,
  url: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StaffGroupResponse {
  role: String,
  people: Vec<StaffPersonResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectStaffResponse {
  id: u32,
  groups: Vec<StaffGroupResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CharacterLinkResponse {
  id: u32,
  name: String,
  url: String,
  relation: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectCharactersResponse {
  id: u32,
  characters: Vec<CharacterLinkResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectAliasesResponse {
  id: u32,
  aliases: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TrackedSubject {
  id: u32,
  name: String,
  name_cn: String,
  image: String,
  url: String,
  watching: bool,
  backlog: bool,
  watched: bool,
  date: String,
  rating: Option<f64>,
  summary: String,
  #[serde(default)]
  aliases: Option<Vec<String>>, // 英文名/罗马音等别名列表
  #[serde(default)]
  aired_count: Option<u32>,
  #[serde(default)]
  total_count: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectBriefResponse {
  id: u32,
  name: String,
  name_cn: String,
  image: String,
  date: String,
  rating: Option<f64>,
  summary: String,
  url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TorrentStartResponse {
  id: usize,
  info_hash: String,
  name: Option<String>,
  output_folder: String,
  final_folder: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TorrentStatusResponse {
  id: usize,
  state: String,
  progress_bytes: u64,
  total_bytes: u64,
  uploaded_bytes: u64,
  finished: bool,
  error: Option<String>,
  download_speed: Option<String>,
  upload_speed: Option<String>,
  time_remaining: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackInfoResponse {
  track_id: String,
  codec: String,
  lang: Option<String>,
  language_name: Option<String>,
  track_name: Option<String>,
  is_default: Option<bool>,
  is_forced: Option<bool>,
  charset: Option<String>,
  attributes: Option<String>,
  container: Option<String>,
  file_size: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackParseResponse {
  tracks: Vec<TrackInfoResponse>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MixTrackInput {
  path: String,
  kind: String,
  track_ids: Vec<String>,
  #[serde(default)]
  track_langs: HashMap<String, String>,
}


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectSummaryResponse {
  id: u32,
  summary: String,
  translated: bool,
  error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectFiltersResponse {
  id: u32,
  types: Vec<String>,
  regions: Vec<String>,
  audiences: Vec<String>,
}

fn season_months(season: &str) -> Result<Vec<u32>, String> {
  match season {
    "winter" => Ok(vec![1, 2, 3]),
    "spring" => Ok(vec![4, 5, 6]),
    "summer" => Ok(vec![7, 8, 9]),
    "autumn" => Ok(vec![10, 11, 12]),
    _ => Err("invalid season".into()),
  }
}

fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("无法获取数据目录: {e}"))?
    .join("hanamirip-cn");
  fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
  Ok(dir.join(TRACK_DB_FILE))
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

fn load_tracked(app: &tauri::AppHandle) -> Result<HashMap<u32, TrackedSubject>, String> {
  let path = db_path(app)?;
  if !path.exists() {
    return Ok(HashMap::new());
  }
  let content = fs::read_to_string(&path).map_err(|e| format!("读取追番数据失败: {e}"))?;
  if content.trim().is_empty() {
    return Ok(HashMap::new());
  }
  let parsed: Vec<TrackedSubject> = serde_json::from_str(&content)
    .map_err(|e| format!("解析追番数据失败: {e}"))?;
  let mut map = HashMap::new();
  for item in parsed {
    map.insert(item.id, item);
  }
  Ok(map)
}

fn persist_tracked(app: &tauri::AppHandle, data: &HashMap<u32, TrackedSubject>) -> Result<(), String> {
  let path = db_path(app)?;
  let list: Vec<_> = data.values().cloned().collect();
  let payload = serde_json::to_string_pretty(&list)
    .map_err(|e| format!("序列化追番数据失败: {e}"))?;
  fs::write(&path, payload).map_err(|e| format!("写入追番数据失败: {e}"))
}

fn resolve_image(images: Option<Images>) -> String {
  if let Some(images) = images {
    if let Some(url) = images.common {
      return url;
    }
    if let Some(url) = images.medium {
      return url;
    }
    if let Some(url) = images.large {
      return url;
    }
  }
  DEFAULT_IMAGE.to_string()
}

fn extract_value_from_object(map: &Map<String, Value>) -> Option<String> {
  for key in ["v", "value", "name", "text"] {
    if let Some(Value::String(text)) = map.get(key) {
      if !text.trim().is_empty() {
        return Some(text.clone());
      }
    }
  }
  None
}

fn extract_infobox_value(value: &Value) -> Option<String> {
  match value {
    Value::String(text) => {
      let trimmed = text.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    }
    Value::Object(map) => extract_value_from_object(map),
    Value::Array(items) => {
      let mut parts = Vec::new();
      for item in items {
        let extracted = match item {
          Value::Object(map) => extract_value_from_object(map),
          _ => extract_infobox_value(item),
        };
        if let Some(text) = extracted {
          if !text.trim().is_empty() {
            parts.push(text);
          }
        }
      }
      if parts.is_empty() {
        None
      } else {
        Some(parts.join(" / "))
      }
    }
    _ => None,
  }
}

fn format_bytes_readable(bytes: u64) -> String {
  const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
  let mut size = bytes as f64;
  let mut idx = 0;
  while size >= 1024.0 && idx < UNITS.len() - 1 {
    size /= 1024.0;
    idx += 1;
  }
  if idx == 0 {
    format!("{} {}", bytes, UNITS[idx])
  } else {
    format!("{:.2} {}", size, UNITS[idx])
  }
}

fn resolve_tool_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
  let resource_dir = app
    .path()
    .resource_dir()
    .map_err(|e| format!("无法获取资源目录: {e}"))?;
  let mut candidates = Vec::new();
  if cfg!(target_os = "windows") {
    candidates.push(resource_dir.join("bin").join(format!("{name}.exe")));
  }
  candidates.push(resource_dir.join("bin").join(name));

  if cfg!(debug_assertions) {
    let dev_bin = PathBuf::from("src-tauri").join("bin");
    if cfg!(target_os = "windows") {
      candidates.push(dev_bin.join(format!("{name}.exe")));
    }
    candidates.push(dev_bin.join(name));
  }

  for path in candidates {
    if path.exists() {
      return Ok(path);
    }
  }
  Err(format!("未找到内置工具 {name}，请检查打包资源是否包含对应文件"))
}

fn map_language_name(code: &str) -> Option<String> {
  let normalized = code.trim().to_lowercase();
  if normalized.starts_with("zh-hans") {
    return Some("简体中文".to_string());
  }
  if normalized.starts_with("zh-hant") || normalized.starts_with("zh-hk") || normalized.starts_with("zh-mo") {
    return Some("繁体中文".to_string());
  }
  let name = match normalized.as_str() {
    "jpn" | "ja" => "日语",
    "eng" | "en" => "英语",
    "chi" | "zho" | "zh" => "中文",
    "chs" | "zh-cn" | "cmn" => "简体中文",
    "cht" | "zh-tw" => "繁体中文",
    "kor" | "ko" => "韩语",
    "fra" | "fr" => "法语",
    "deu" | "ger" | "de" => "德语",
    "spa" | "es" => "西班牙语",
    _ => "",
  };
  if name.is_empty() {
    None
  } else {
    Some(name.to_string())
  }
}

#[derive(Deserialize)]
struct FFProbeFormat {
  format_name: Option<String>,
  size: Option<String>,
}

#[derive(Clone, Deserialize)]
struct FFProbeDisposition {
  default: Option<i32>,
  forced: Option<i32>,
}

#[derive(Deserialize)]
struct FFProbeStreamTags {
  language: Option<String>,
  title: Option<String>,
  encoding: Option<String>,
  charset: Option<String>,
}

#[derive(Deserialize)]
struct FFProbeStream {
  index: Option<u32>,
  codec_name: Option<String>,
  codec_type: Option<String>,
  width: Option<u32>,
  height: Option<u32>,
  r_frame_rate: Option<String>,
  channels: Option<u32>,
  channel_layout: Option<String>,
  disposition: Option<FFProbeDisposition>,
  tags: Option<FFProbeStreamTags>,
}

#[derive(Deserialize)]
struct FFProbeOutput {
  streams: Option<Vec<FFProbeStream>>,
  format: Option<FFProbeFormat>,
}

#[derive(Deserialize)]
struct MkvmergeContainerProperties {
  file_size: Option<u64>,
}

#[derive(Deserialize)]
struct MkvmergeContainer {
  r#type: Option<String>,
  properties: Option<MkvmergeContainerProperties>,
}

#[derive(Deserialize)]
struct MkvmergeTrackProperties {
  language: Option<String>,
  language_ietf: Option<String>,
  track_name: Option<String>,
  default_track: Option<bool>,
  forced_track: Option<bool>,
  codec_name: Option<String>,
  codec_id: Option<String>,
  encoding: Option<String>,
  pixel_dimensions: Option<String>,
  audio_channels: Option<u32>,
  audio_sampling_frequency: Option<f64>,
}

#[derive(Deserialize)]
struct MkvmergeTrack {
  id: u32,
  r#type: Option<String>,
  codec: Option<String>,
  properties: Option<MkvmergeTrackProperties>,
}

#[derive(Deserialize)]
struct MkvmergeOutput {
  container: Option<MkvmergeContainer>,
  tracks: Option<Vec<MkvmergeTrack>>,
}

fn build_attributes(stream: &FFProbeStream) -> Option<String> {
  if stream.codec_type.as_deref() == Some("video") {
    let mut parts = Vec::new();
    if let (Some(w), Some(h)) = (stream.width, stream.height) {
      parts.push(format!("{}x{}", w, h));
    }
    if let Some(rate) = stream.r_frame_rate.as_ref() {
      if rate != "0/0" {
        parts.push(rate.to_string());
      }
    }
    return if parts.is_empty() { None } else { Some(parts.join(" ")) };
  }
  if stream.codec_type.as_deref() == Some("audio") {
    let mut parts = Vec::new();
    if let Some(ch) = stream.channels {
      parts.push(format!("{}ch", ch));
    }
    if let Some(layout) = stream.channel_layout.as_ref() {
      parts.push(layout.to_string());
    }
    return if parts.is_empty() { None } else { Some(parts.join(" ")) };
  }
  stream.tags.as_ref().and_then(|t| t.title.clone())
}

fn build_mkvmerge_attributes(props: &MkvmergeTrackProperties, kind: &str) -> Option<String> {
  if kind == "video" {
    return props.pixel_dimensions.clone();
  }
  if kind == "audio" {
    let mut parts = Vec::new();
    if let Some(ch) = props.audio_channels {
      parts.push(format!("{}ch", ch));
    }
    if let Some(freq) = props.audio_sampling_frequency {
      parts.push(format!("{} Hz", freq.round() as u64));
    }
    return if parts.is_empty() { None } else { Some(parts.join(" ")) };
  }
  None
}

#[tauri::command]
async fn parse_media_tracks(app: tauri::AppHandle, path: String, kind: String) -> Result<TrackParseResponse, String> {
  let kind_lower = kind.to_lowercase();
  let ext = Path::new(&path)
    .extension()
    .and_then(|s| s.to_str())
    .unwrap_or("")
    .to_lowercase();

  if ["mkv", "mka", "mks"].contains(&ext.as_str()) {
    let mkvmerge_path = resolve_tool_path(&app, "mkvmerge")?;
    let output = Command::new(mkvmerge_path)
      .args(["-J", &path])
      .output()
      .await
      .map_err(|e| format!("调用 mkvmerge 失败: {e}"))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!("mkvmerge 执行失败: {stderr}"));
    }

    let parsed: MkvmergeOutput = serde_json::from_slice(&output.stdout)
      .map_err(|e| format!("解析 mkvmerge 输出失败: {e}"))?;

    let container = parsed
      .container
      .as_ref()
      .and_then(|c| c.r#type.clone());
    let file_size = parsed
      .container
      .as_ref()
      .and_then(|c| c.properties.as_ref())
      .and_then(|p| p.file_size)
      .map(format_bytes_readable);

    let tracks = parsed
      .tracks
      .unwrap_or_default()
      .into_iter()
      .filter(|track| {
        let track_type = track.r#type.as_deref().unwrap_or("");
        if kind_lower == "subtitle" {
          return track_type == "subtitles" || track_type == "subtitle";
        }
        track_type == kind_lower
      })
      .map(|track| {
        let props = track.properties.unwrap_or(MkvmergeTrackProperties {
          language: None,
          language_ietf: None,
          track_name: None,
          default_track: None,
          forced_track: None,
          codec_name: None,
          codec_id: None,
          encoding: None,
          pixel_dimensions: None,
          audio_channels: None,
          audio_sampling_frequency: None,
        });
        let lang = props.language_ietf.clone().or(props.language.clone());
        let language_name = lang
          .as_deref()
          .and_then(map_language_name);
        let codec = props
          .codec_name
          .clone()
          .or(track.codec.clone())
          .or(props.codec_id.clone())
          .unwrap_or_else(|| "unknown".to_string());
        TrackInfoResponse {
          track_id: track.id.to_string(),
          codec,
          lang,
          language_name,
          track_name: props.track_name.clone(),
          is_default: props.default_track,
          is_forced: props.forced_track,
          charset: props.encoding.clone(),
          attributes: build_mkvmerge_attributes(&props, &kind_lower),
          container: container.clone(),
          file_size: file_size.clone(),
        }
      })
      .collect();

    return Ok(TrackParseResponse { tracks });
  }

  let ffprobe_path = resolve_tool_path(&app, "ffprobe")?;
  let output = Command::new(ffprobe_path)
    .args([
      "-v",
      "error",
      "-print_format",
      "json",
      "-show_format",
      "-show_streams",
      &path,
    ])
    .output()
    .await
    .map_err(|e| format!("调用 ffprobe 失败: {e}"))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("ffprobe 执行失败: {stderr}"));
  }

  let parsed: FFProbeOutput = serde_json::from_slice(&output.stdout)
    .map_err(|e| format!("解析 ffprobe 输出失败: {e}"))?;

  let container = parsed
    .format
    .as_ref()
    .and_then(|f| f.format_name.clone());
  let file_size = parsed
    .format
    .as_ref()
    .and_then(|f| f.size.as_ref())
    .and_then(|s| s.parse::<u64>().ok())
    .map(format_bytes_readable);

  let streams = parsed.streams.unwrap_or_default();
  let tracks = streams
    .into_iter()
    .filter(|stream| stream.codec_type.as_deref() == Some(&kind_lower))
    .map(|stream| {
      let lang = stream.tags.as_ref().and_then(|t| t.language.clone());
      let language_name = lang.as_deref().and_then(map_language_name);
      let track_name = stream.tags.as_ref().and_then(|t| t.title.clone());
      let charset = stream
        .tags
        .as_ref()
        .and_then(|t| t.charset.clone().or_else(|| t.encoding.clone()));
      let codec = stream
        .codec_name
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
      let disposition = stream
        .disposition
        .clone()
        .unwrap_or(FFProbeDisposition {
          default: None,
          forced: None,
        });
      TrackInfoResponse {
        track_id: stream.index.unwrap_or(0).to_string(),
        codec,
        lang,
        language_name,
        track_name,
        is_default: disposition.default.map(|v| v == 1),
        is_forced: disposition.forced.map(|v| v == 1),
        charset,
        attributes: build_attributes(&stream),
        container: container.clone(),
        file_size: file_size.clone(),
      }
    })
    .collect();

  Ok(TrackParseResponse { tracks })
}

#[tauri::command]
async fn get_media_file_size(path: String) -> Result<Option<String>, String> {
  let meta = fs::metadata(&path).map_err(|e| format!("读取文件大小失败: {e}"))?;
  Ok(Some(format_bytes_readable(meta.len())))
}

#[tauri::command]
async fn mix_media_tracks(app: tauri::AppHandle, inputs: Vec<MixTrackInput>, output_path: String) -> Result<String, String> {
  if inputs.is_empty() {
    return Err("未提供可合成的轨道".to_string());
  }

  let mut output = PathBuf::from(&output_path);
  if output.extension().is_none() {
    output.set_extension("mkv");
  }
  if let Some(parent) = output.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败: {e}"))?;
    }
  }

  let mkvmerge_path = resolve_tool_path(&app, "mkvmerge")?;

  fn lang_for_kind(kind: &str) -> &'static str {
    match kind {
      "video" => "ja",
      "audio" => "ja",
      "subtitle" => "zh-Hans",
      _ => "und",
    }
  }

  fn format_arg(arg: &str) -> String {
    if arg.contains(' ') || arg.contains('\t') || arg.contains('"') {
      format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
      arg.to_string()
    }
  }

  fn build_cmdline(args: &[String]) -> String {
    let mut cmdline = Vec::new();
    cmdline.push(format_arg("mkvmerge"));
    for arg in args {
      cmdline.push(format_arg(arg));
    }
    cmdline.join(" ")
  }

  async fn run_mkvmerge(mkvmerge_path: &PathBuf, args: &[String]) -> Result<(), String> {
    let output_exec = Command::new(mkvmerge_path)
      .args(args.iter())
      .output()
      .await
      .map_err(|e| format!("调用 mkvmerge 失败: {e}"))?;

    if !output_exec.status.success() {
      let stderr = String::from_utf8_lossy(&output_exec.stderr);
      let stdout = String::from_utf8_lossy(&output_exec.stdout);
      return Err(format!(
        "mkvmerge 执行失败 (code {:?}): {} {}\n命令: {}",
        output_exec.status.code(),
        stdout.trim(),
        stderr.trim(),
        build_cmdline(args)
      ));
    }
    Ok(())
  }

  let mut kind_inputs: HashMap<String, MixTrackInput> = HashMap::new();
  for input in inputs {
    let path = input.path.trim();
    if path.is_empty() {
      return Err("轨道文件路径为空".to_string());
    }
    if !Path::new(path).exists() {
      return Err(format!("轨道文件不存在: {path}"));
    }
    let track_ids: Vec<String> = input
      .track_ids
      .into_iter()
      .map(|id| id.trim().to_string())
      .filter(|id| !id.is_empty())
      .collect();
    if track_ids.is_empty() {
      continue;
    }
    let kind_lower = input.kind.to_lowercase();
    let entry = kind_inputs.entry(kind_lower.clone()).or_insert(MixTrackInput {
      path: path.to_string(),
      kind: kind_lower.clone(),
      track_ids: Vec::new(),
      track_langs: HashMap::new(),
    });
    if entry.path != path {
      return Err(format!("同一类型只支持一个文件：{}", kind_lower));
    }
    for track_id in track_ids {
      if !entry.track_ids.iter().any(|id| id == &track_id) {
        entry.track_ids.push(track_id);
      }
    }
    for (track_id, lang) in input.track_langs {
      entry.track_langs.insert(track_id, lang);
    }
  }

  if !kind_inputs.contains_key("video") {
    return Err("请先检测并选择至少一个视频轨道".to_string());
  }

  let temp_root = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("无法获取数据目录: {e}"))?
    .join("hanamirip-cn")
    .join("mix-temp")
    .join(Utc::now().timestamp_millis().to_string());
  fs::create_dir_all(&temp_root).map_err(|e| format!("创建临时目录失败: {e}"))?;

  let mut temp_files: Vec<PathBuf> = Vec::new();

  async fn build_temp(
    kind: &str,
    input: &MixTrackInput,
    temp_root: &PathBuf,
    mkvmerge_path: &PathBuf,
  ) -> Result<PathBuf, String> {
    let ext = match kind {
      "video" => "mkv",
      "audio" => "mka",
      "subtitle" => "mks",
      _ => "mkv",
    };
    let temp_path = temp_root.join(format!("{kind}.{ext}"));
    let mut args: Vec<String> = Vec::new();
    args.push("-o".to_string());
    args.push(temp_path.to_string_lossy().to_string());

    match kind {
      "video" => {
        args.push("--video-tracks".to_string());
        args.push(input.track_ids.join(","));
        args.push("--audio-tracks".to_string());
        args.push("-1".to_string());
        args.push("--subtitle-tracks".to_string());
        args.push("-1".to_string());
      }
      "audio" => {
        args.push("--audio-tracks".to_string());
        args.push(input.track_ids.join(","));
        args.push("--video-tracks".to_string());
        args.push("-1".to_string());
        args.push("--subtitle-tracks".to_string());
        args.push("-1".to_string());
      }
      "subtitle" => {
        args.push("--subtitle-tracks".to_string());
        args.push(input.track_ids.join(","));
        args.push("--video-tracks".to_string());
        args.push("-1".to_string());
        args.push("--audio-tracks".to_string());
        args.push("-1".to_string());
      }
      _ => {}
    }

    let lang = lang_for_kind(kind);
    for track_id in &input.track_ids {
      args.push("--track-name".to_string());
      args.push(format!("{track_id}:"));
      args.push("--default-track-flag".to_string());
      args.push(format!("{track_id}:yes"));
      args.push("--forced-display-flag".to_string());
      args.push(format!("{track_id}:no"));
      args.push("--language".to_string());
      let custom_lang = input.track_langs.get(track_id).map(|v| v.as_str());
      let final_lang = custom_lang.unwrap_or(lang);
      args.push(format!("{track_id}:{final_lang}"));
    }

    args.push(input.path.clone());
    run_mkvmerge(mkvmerge_path, &args).await?;
    Ok::<PathBuf, String>(temp_path)
  }

  let mut video_temp = None;
  let mut audio_temp = None;
  let mut subtitle_temp = None;

  if let Some(input) = kind_inputs.get("video") {
    let path = build_temp("video", input, &temp_root, &mkvmerge_path).await?;
    temp_files.push(path.clone());
    video_temp = Some(path);
  }
  if let Some(input) = kind_inputs.get("audio") {
    let path = build_temp("audio", input, &temp_root, &mkvmerge_path).await?;
    temp_files.push(path.clone());
    audio_temp = Some(path);
  }
  if let Some(input) = kind_inputs.get("subtitle") {
    let path = build_temp("subtitle", input, &temp_root, &mkvmerge_path).await?;
    temp_files.push(path.clone());
    subtitle_temp = Some(path);
  }

  let mut merge_args: Vec<String> = Vec::new();
  merge_args.push("-o".to_string());
  merge_args.push(output.to_string_lossy().to_string());

  if let Some(path) = video_temp.as_ref() {
    merge_args.push(path.to_string_lossy().to_string());
  }
  if let Some(path) = audio_temp.as_ref() {
    merge_args.push(path.to_string_lossy().to_string());
  }
  if let Some(path) = subtitle_temp.as_ref() {
    merge_args.push(path.to_string_lossy().to_string());
  }

  run_mkvmerge(&mkvmerge_path, &merge_args).await?;

  for path in temp_files {
    let _ = fs::remove_file(path);
  }

  Ok(output.to_string_lossy().to_string())
}

fn split_infobox_terms(value: &str) -> Vec<String> {
  value
    .split(|ch| matches!(ch, '/' | '／' | '、' | ',' | '，' | '|'))
    .map(|item| item.trim())
    .filter(|item| !item.is_empty())
    .map(|item| item.to_string())
    .collect()
}

fn extract_infobox_values(value: &Value) -> Vec<String> {
  match value {
    Value::String(text) => split_infobox_terms(text),
    Value::Object(map) => extract_value_from_object(map)
      .map(|text| split_infobox_terms(&text))
      .unwrap_or_default(),
    Value::Array(items) => {
      let mut output = Vec::new();
      for item in items {
        let extracted = match item {
          Value::Object(map) => extract_value_from_object(map)
            .map(|text| split_infobox_terms(&text))
            .unwrap_or_default(),
          _ => extract_infobox_values(item),
        };
        output.extend(extracted);
      }
      output
    }
    _ => Vec::new(),
  }
}

fn collect_subject_tags(tags: Option<Vec<SubjectTag>>, meta_tags: Option<Vec<String>>) -> Vec<String> {
  let mut output = Vec::new();
  if let Some(items) = tags {
    for item in items {
      let name = item.name.trim().to_string();
      if !name.is_empty() {
        output.push(name);
      }
    }
  }
  if let Some(items) = meta_tags {
    for name in items {
      let name = name.trim().to_string();
      if !name.is_empty() {
        output.push(name);
      }
    }
  }
  dedupe_terms(output)
}

fn normalize_tag(value: &str) -> String {
  value.trim().to_lowercase()
}

fn map_tags_to_official(tags: &[String], official: &[&str], aliases: &[(&str, &str)]) -> Vec<String> {
  let mut official_map = HashMap::new();
  for name in official {
    official_map.insert(normalize_tag(name), (*name).to_string());
  }
  let mut alias_map = HashMap::new();
  for (from, to) in aliases {
    alias_map.insert(normalize_tag(from), (*to).to_string());
  }
  let mut output = Vec::new();
  let mut seen = HashSet::new();
  for tag in tags {
    let normalized = normalize_tag(tag);
    if let Some(value) = official_map.get(&normalized) {
      if seen.insert(value.clone()) {
        output.push(value.clone());
      }
      continue;
    }
    if let Some(value) = alias_map.get(&normalized) {
      if seen.insert(value.clone()) {
        output.push(value.clone());
      }
    }
  }
  output
}

fn dedupe_terms(values: Vec<String>) -> Vec<String> {
  let mut seen = HashSet::new();
  let mut output = Vec::new();
  for value in values {
    if seen.insert(value.clone()) {
      output.push(value);
    }
  }
  output
}

fn extract_filter_groups(infobox: Option<Vec<InfoboxItem>>) -> (Vec<String>, Vec<String>, Vec<String>) {
  let mut types = Vec::new();
  let mut regions = Vec::new();
  let mut audiences = Vec::new();
  let items = match infobox {
    Some(items) => items,
    None => return (types, regions, audiences),
  };
  for item in items {
    let values = extract_infobox_values(&item.value);
    if values.is_empty() {
      continue;
    }
    match item.key.as_str() {
      "类型" | "题材" | "动画类型" | "分类" | "类别" => types.extend(values),
      "地区" | "国家/地区" | "国家地区" | "国家" | "发行地区" => regions.extend(values),
      "受众" | "对象" | "读者对象" => audiences.extend(values),
      _ => {}
    }
  }
  (dedupe_terms(types), dedupe_terms(regions), dedupe_terms(audiences))
}

fn extract_origin(infobox: Option<Vec<InfoboxItem>>) -> Option<String> {
  let items = infobox?;
  for item in items {
    if item.key == "原作" {
      if let Some(value) = extract_infobox_value(&item.value) {
        return Some(value);
      }
    }
  }
  None
}

fn extract_aliases(infobox: Option<Vec<InfoboxItem>>) -> Vec<String> {
  let mut output = Vec::new();
  let items = match infobox { Some(items) => items, None => return output };
  for item in items {
    let key = item.key.as_str();
    // 兼容多种可能的别名字段
    let is_alias_key = key.contains("别名")
      || key.contains("又名")
      || key.contains("英文")
      || key.contains("罗马")
      || key.eq_ignore_ascii_case("romaji")
      || key.eq_ignore_ascii_case("english");
    if !is_alias_key { continue; }
    let values = extract_infobox_values(&item.value);
    for v in values { output.push(v); }
  }
  dedupe_terms(output)
}

fn parse_airdate(value: &Option<String>) -> Option<NaiveDate> {
  let raw = value.as_ref()?;
  NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok()
}

fn is_aired(episode: &Episode, today: NaiveDate) -> bool {
  if let Some(status) = &episode.status {
    if status == "air" {
      return true;
    }
  }
  if let Some(date) = parse_airdate(&episode.airdate) {
    return date <= today;
  }
  false
}

fn contains_kana(text: &str) -> bool {
  text.chars()
    .any(|ch| matches!(ch as u32, 0x3040..=0x30FF | 0x31F0..=0x31FF))
}

fn contains_cjk(text: &str) -> bool {
  text.chars().any(|ch| matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF))
}

fn is_chinese_text(text: &str) -> bool {
  contains_cjk(text) && !contains_kana(text)
}

struct BaiduVerifier {
  _lib: Library,
  get_app_id: unsafe extern "C" fn() -> *const c_char,
  get_api_key: unsafe extern "C" fn() -> *const c_char,
}

static BAIDU_VERIFIER: OnceCell<BaiduVerifier> = OnceCell::new();

fn load_baidu_verifier() -> Result<&'static BaiduVerifier, String> {
  BAIDU_VERIFIER.get_or_try_init(|| {
    let lib_path = env::var("BAIDU_VERIFY_SO").unwrap_or_else(|_| {
      // Windows uses .dll, Linux/macOS use .so/.dylib
      #[cfg(target_os = "windows")]
      let candidates = [
        "src-tauri/baidu_verify/baidu_verify.dll",
        "baidu_verify/baidu_verify.dll",
        "src-tauri\\baidu_verify\\baidu_verify.dll",
        "baidu_verify\\baidu_verify.dll",
      ];
      
      #[cfg(target_os = "macos")]
      let candidates = [
        "src-tauri/baidu_verify/libbaidu_verify.dylib",
        "baidu_verify/libbaidu_verify.dylib",
      ];
      
      #[cfg(not(any(target_os = "windows", target_os = "macos")))]
      let candidates = [
        "src-tauri/baidu_verify/libbaidu_verify.so",
        "baidu_verify/libbaidu_verify.so",
      ];
      
      candidates
        .iter()
        .find(|path| Path::new(path).exists())
        .unwrap_or(&candidates[0])
        .to_string()
    });
    
    // Check if the library file exists before attempting to load
    if !Path::new(&lib_path).exists() {
      #[cfg(target_os = "windows")]
      let lib_name = "baidu_verify.dll";
      #[cfg(target_os = "macos")]
      let lib_name = "libbaidu_verify.dylib";
      #[cfg(not(any(target_os = "windows", target_os = "macos")))]
      let lib_name = "libbaidu_verify.so";
      
      return Err(format!(
        "找不到百度翻译动态库文件: {}\n\
         请先构建动态库：\n\
         1. 设置环境变量 BAIDU_TRANSLATE_APP_ID 和 BAIDU_TRANSLATE_API_KEY\n\
         2. 运行命令: yarn run build:baidu-so:windows (Windows) 或 yarn run build:baidu-so:linux (Linux)\n\
         3. 确保生成的 {} 文件位于 src-tauri/baidu_verify/ 目录下",
        lib_path, lib_name
      ));
    }
    
    let lib = unsafe { Library::new(&lib_path) }
      .map_err(|e| format!("加载百度翻译校验库失败 ({}): {}", lib_path, e))?;
    let get_app_id = {
      let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
        unsafe { lib.get(b"baidu_get_app_id") }
          .map_err(|e| format!("解析百度翻译 APP ID 读取函数失败: {e}"))?;
      *symbol
    };
    let get_api_key = {
      let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
        unsafe { lib.get(b"baidu_get_api_key") }
          .map_err(|e| format!("解析百度翻译 API Key 读取函数失败: {e}"))?;
      *symbol
    };
    Ok(BaiduVerifier {
      _lib: lib,
      get_app_id,
      get_api_key,
    })
  })
}

fn get_baidu_credentials() -> Result<(String, String), String> {
  let verifier = load_baidu_verifier()?;
  let app_id_ptr = unsafe { (verifier.get_app_id)() };
  let api_key_ptr = unsafe { (verifier.get_api_key)() };
  if app_id_ptr.is_null() || api_key_ptr.is_null() {
    return Err("百度翻译密钥读取失败".to_string());
  }
  let app_id = unsafe { std::ffi::CStr::from_ptr(app_id_ptr) }
    .to_string_lossy()
    .trim()
    .to_string();
  let api_key = unsafe { std::ffi::CStr::from_ptr(api_key_ptr) }
    .to_string_lossy()
    .trim()
    .to_string();
  if app_id.is_empty() || api_key.is_empty() {
    return Err("百度翻译密钥为空".to_string());
  }
  Ok((app_id, api_key))
}

async fn translate_to_cn_baidu(client: &Client, text: &str) -> Result<String, String> {
  let (app_id, api_key) = get_baidu_credentials()?;

  let parse_translation = |body: &str| -> Result<Option<String>, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| format!("响应解析失败: {e}"))?;
    if let Some(code) = value.get("error_code").and_then(|v| v.as_str()) {
      let msg = value
        .get("error_msg")
        .and_then(|v| v.as_str())
        .unwrap_or("");
      return Err(format!("百度翻译错误: {code} {msg}"));
    }
    let extract_list = |list: &Value| {
      list.as_array()
        .map(|items| {
          items
            .iter()
            .filter_map(|item| item.get("dst").and_then(|v| v.as_str()))
            .collect::<String>()
        })
        .filter(|output| !output.trim().is_empty())
    };

    if let Some(output) = value.get("trans_result").and_then(extract_list) {
      return Ok(Some(output));
    }
    if let Some(output) = value.get("data").and_then(extract_list) {
      return Ok(Some(output));
    }
    if let Some(result) = value.get("result") {
      if let Some(output) = result.get("trans_result").and_then(extract_list) {
        return Ok(Some(output));
      }
      if let Some(output) = result.get("data").and_then(extract_list) {
        return Ok(Some(output));
      }
    }
    Ok(None)
  };

  let ai_request = client
    .post("https://fanyi-api.baidu.com/ait/api/aiTextTranslate")
    .json(&serde_json::json!({
      "appid": app_id,
      "q": text,
      "from": "auto",
      "to": "zh",
      "model_type": "llm"
    }))
    .bearer_auth(&api_key);

  let ai_response = ai_request.send().await.map_err(|e| e.to_string())?;

  if ai_response.status().is_success() {
    let body = ai_response.text().await.map_err(|e| e.to_string())?;
    match parse_translation(&body) {
      Ok(Some(output)) => return Ok(output),
      Ok(None) => {}
      Err(err) => {
        if !err.contains("52003") {
          return Err(err);
        }
      }
    }
  }

  Err("翻译结果为空".to_string())
}

async fn fetch_subject_persons(client: &Client, id: u32) -> Result<Vec<SubjectPersonItem>, String> {
  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}/persons"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  let payload: SubjectPersonPayload = response.json().await.map_err(|e| e.to_string())?;
  let items = match payload {
    SubjectPersonPayload::List(list) => list,
    SubjectPersonPayload::Page { data, .. } => data,
  };

  Ok(items)
}

fn build_staff_groups(items: Vec<SubjectPersonItem>) -> Vec<StaffGroupResponse> {
  let mut grouped: HashMap<String, Vec<StaffPersonResponse>> = HashMap::new();
  for item in items {
    let relation = match item.relation {
      Some(value) if !value.trim().is_empty() => value,
      _ => continue,
    };
    let person = StaffPersonResponse {
      id: item.id,
      name: item.name,
      url: format!("https://bgm.tv/person/{}", item.id),
    };
    grouped.entry(relation).or_default().push(person);
  }

  let ordered_roles = vec![
    "导演", "脚本", "分镜", "演出", "音乐", "人物设定", "系列构成", "美术监督", "色彩设计",
    "总作画监督", "作画监督", "摄影监督", "道具设计", "剪辑", "剪辑助手", "主题歌编曲",
    "主题歌作曲",
  ];

  let mut used = HashSet::new();
  let mut groups = Vec::new();
  for role in ordered_roles {
    if let Some(people) = grouped.get(role) {
      used.insert(role.to_string());
      groups.push(StaffGroupResponse {
        role: role.to_string(),
        people: people.clone(),
      });
    }
  }

  let mut remaining: Vec<_> = grouped
    .into_iter()
    .filter(|(role, _)| !used.contains(role))
    .collect();
  remaining.sort_by(|a, b| a.0.cmp(&b.0));
  for (role, people) in remaining {
    groups.push(StaffGroupResponse { role, people });
  }

  groups
}

async fn fetch_subject_characters(client: &Client, id: u32) -> Result<Vec<SubjectCharacterItem>, String> {
  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}/characters"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  let payload: SubjectCharacterPayload = response.json().await.map_err(|e| e.to_string())?;
  let items = match payload {
    SubjectCharacterPayload::List(list) => list,
    SubjectCharacterPayload::Page { data, .. } => data,
  };

  Ok(items)
}

fn map_subject(subject: Subject) -> SeasonAnime {
  let image = resolve_image(subject.images);
  let summary = subject.summary.unwrap_or_default();
  let date = subject.date.unwrap_or_default();
  let rating = subject.rating.and_then(|value| value.score);
  let url = format!("https://bgm.tv/subject/{}", subject.id);
  SeasonAnime {
    id: subject.id,
    name: subject.name,
    name_cn: subject.name_cn,
    image,
    date,
    rating,
    summary,
    url,
  }
}

async fn fetch_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  let subject: Subject = response.json().await.map_err(|e| e.to_string())?;
  let mapped = map_subject(subject);
  Ok(SubjectBriefResponse {
    id: mapped.id,
    name: mapped.name,
    name_cn: mapped.name_cn,
    image: mapped.image,
    date: mapped.date,
    rating: mapped.rating,
    summary: mapped.summary,
    url: mapped.url,
  })
}

async fn fetch_month_subjects(client: &Client, year: u32, month: u32) -> Result<Vec<SeasonAnime>, String> {
  let mut offset = 0u32;
  let limit = 50u32;
  let mut items = Vec::new();

  loop {
    let response = client
      .get(format!("{API_BASE}{SUBJECTS_PATH}"))
      .query(&[
        ("type", "2"),
        ("year", &year.to_string()),
        ("month", &month.to_string()),
        ("limit", &limit.to_string()),
        ("offset", &offset.to_string()),
      ])
      .send()
      .await
      .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
      return Err(format!("Bangumi API 请求失败: {}", response.status()));
    }

    let payload: PagedSubject = response.json().await.map_err(|e| e.to_string())?;
    if payload.data.is_empty() {
      break;
    }

    items.extend(payload.data.into_iter().map(map_subject));

    offset += payload.limit.max(limit);
    if offset >= payload.total {
      break;
    }
  }

  Ok(items)
}

#[tauri::command]
async fn get_subject_origin(id: u32) -> Result<SubjectOriginResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
  let origin = extract_origin(payload.infobox);
  Ok(SubjectOriginResponse { id, origin })
}

#[tauri::command]
async fn get_subject_aired_count(id: u32) -> Result<SubjectAiredResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let mut offset = 0u32;
  let limit = 50u32;
  let mut aired_count = 0u32;
  let mut total_count = 0u32;
  let today = Utc::now().date_naive();

  loop {
    let response = client
      .get(format!("{API_BASE}{EPISODES_PATH}"))
      .query(&[
        ("subject_id", &id.to_string()),
        ("limit", &limit.to_string()),
        ("offset", &offset.to_string()),
      ])
      .send()
      .await
      .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
      return Err(format!("Bangumi API 请求失败: {}", response.status()));
    }

    let payload: EpisodePage = response.json().await.map_err(|e| e.to_string())?;
    if total_count == 0 {
      total_count = payload.total;
    }

    for episode in payload.data.iter() {
      if is_aired(episode, today) {
        aired_count += 1;
      }
    }

    if payload.data.is_empty() {
      break;
    }

    offset += payload.limit.max(limit);
    if offset >= payload.total {
      break;
    }
  }

  Ok(SubjectAiredResponse {
    id,
    aired_count,
    total_count,
  })
}

#[tauri::command]
async fn get_subject_filters(id: u32) -> Result<SubjectFiltersResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
  let (info_types, info_regions, info_audiences) = extract_filter_groups(payload.infobox);
  let mut tags = collect_subject_tags(payload.tags, payload.meta_tags);
  tags.extend(info_types.clone());
  tags.extend(info_regions.clone());
  tags.extend(info_audiences.clone());

  const OFFICIAL_TYPES: &[&str] = &[
    "科幻",
    "喜剧",
    "同人",
    "百合",
    "校园",
    "惊悚",
    "后宫",
    "机战",
    "悬疑",
    "恋爱",
    "奇幻",
    "推理",
    "运动",
    "耽美",
    "音乐",
    "战斗",
    "冒险",
    "萌系",
    "穿越",
    "玄幻",
    "乙女",
    "恐怖",
    "历史",
    "日常",
    "剧情",
    "武侠",
    "美食",
    "职场",
  ];
  const OFFICIAL_REGIONS: &[&str] = &[
    "日本",
    "欧美",
    "中国",
    "美国",
    "韩国",
    "法国",
    "中国香港",
    "英国",
    "俄罗斯",
    "苏联",
    "捷克",
    "中国台湾",
    "马来西亚",
  ];
  const OFFICIAL_AUDIENCES: &[&str] = &[
    "BL",
    "GL",
    "子供向",
    "女性向",
    "少女向",
    "少年向",
    "青年向",
  ];
  const TYPE_ALIASES: &[(&str, &str)] = &[
    ("搞笑", "喜剧"),
    ("恋愛", "恋爱"),
    ("日常系", "日常"),
    ("熱血", "战斗"),
    ("机甲", "机战"),
    ("萌", "萌系"),
  ];
  const REGION_ALIASES: &[(&str, &str)] = &[
    ("大陆", "中国"),
    ("中国大陆", "中国"),
    ("香港", "中国香港"),
    ("台湾", "中国台湾"),
    ("欧美动画", "欧美"),
    ("欧洲", "欧美"),
    ("俄国", "俄罗斯"),
  ];
  const AUDIENCE_ALIASES: &[(&str, &str)] = &[
    ("少年", "少年向"),
    ("少女", "少女向"),
    ("青年", "青年向"),
    ("儿童", "子供向"),
    ("儿童向", "子供向"),
    ("女性向", "女性向"),
    ("男性向", "男性向"),
    ("男向", "男性向"),
    ("女向", "女性向"),
    ("百合", "GL"),
    ("耽美", "BL"),
    ("腐向", "BL"),
  ];

  let types = map_tags_to_official(&tags, OFFICIAL_TYPES, TYPE_ALIASES);
  let regions = map_tags_to_official(&tags, OFFICIAL_REGIONS, REGION_ALIASES);
  let audiences = map_tags_to_official(&tags, OFFICIAL_AUDIENCES, AUDIENCE_ALIASES);
  Ok(SubjectFiltersResponse {
    id,
    types,
    regions,
    audiences,
  })
}

#[tauri::command]
async fn get_subject_aliases(id: u32) -> Result<SubjectAliasesResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
  let aliases = extract_aliases(payload.infobox);
  Ok(SubjectAliasesResponse { id, aliases })
}

#[tauri::command]
async fn get_subject_staff(id: u32) -> Result<SubjectStaffResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let persons = fetch_subject_persons(&client, id).await?;
  let groups = build_staff_groups(persons);
  Ok(SubjectStaffResponse { id, groups })
}

#[tauri::command]
async fn get_subject_characters(id: u32) -> Result<SubjectCharactersResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let characters = fetch_subject_characters(&client, id).await?;
  let mapped = characters
    .into_iter()
    .map(|item| CharacterLinkResponse {
      id: item.id,
      name: item.name_cn.clone().unwrap_or(item.name),
      url: format!("https://bgm.tv/character/{}", item.id),
      relation: item.relation,
    })
    .collect();

  Ok(SubjectCharactersResponse {
    id,
    characters: mapped,
  })
}

#[tauri::command]
async fn get_subject_summary_cn(id: u32, summary: String) -> Result<SubjectSummaryResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let mut raw = summary;
  if raw.trim().is_empty() {
    let response = client
      .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}"))
      .send()
      .await
      .map_err(|e| e.to_string())?;

    if response.status().is_success() {
      if let Ok(payload) = response.json::<SubjectDetail>().await {
        raw = payload.summary.unwrap_or_default();
      }
    }
  }

  if raw.trim().is_empty() {
    return Ok(SubjectSummaryResponse {
      id,
      summary: "".to_string(),
      translated: false,
      error: None,
    });
  }

  if is_chinese_text(&raw) {
    return Ok(SubjectSummaryResponse {
      id,
      summary: raw,
      translated: false,
      error: None,
    });
  }

  match translate_to_cn_baidu(&client, &raw).await {
    Ok(result) => {
      let normalized = result.trim().to_string();
      if normalized.is_empty() || normalized == raw || !is_chinese_text(&normalized) {
        Ok(SubjectSummaryResponse {
          id,
          summary: raw,
          translated: false,
          error: Some("翻译失败或未产出中文结果".to_string()),
        })
      } else {
        Ok(SubjectSummaryResponse {
          id,
          summary: normalized,
          translated: true,
          error: None,
        })
      }
    }
    Err(error) => Ok(SubjectSummaryResponse {
      id,
      summary: raw,
      translated: false,
      error: Some(error),
    }),
  }
}

#[tauri::command]
async fn get_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
  fetch_subject_brief(id).await
}

#[tauri::command]
async fn fetch_search_html(url: String) -> Result<String, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
  if !response.status().is_success() {
    return Err(format!("搜索站点请求失败: {}", response.status()));
  }

  let body = response.text().await.map_err(|e| e.to_string())?;
  Ok(body)
}

#[tauri::command]
async fn start_torrent_download(app: tauri::AppHandle, url: String, output_dir: String) -> Result<TorrentStartResponse, String> {
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
fn finalize_torrent_download(temp_folder: String, final_folder: String) -> Result<(), String> {
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
async fn get_torrent_status(app: tauri::AppHandle, id: usize) -> Result<TorrentStatusResponse, String> {
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
async fn pause_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_pause(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("暂停下载失败: {e}"))?;
  Ok(())
}

#[tauri::command]
async fn resume_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_start(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("继续下载失败: {e}"))?;
  Ok(())
}

#[tauri::command]
async fn delete_torrent(app: tauri::AppHandle, id: usize) -> Result<(), String> {
  let api = torrent_api(&app).await?;
  api
    .api_torrent_action_delete(TorrentIdOrHash::Id(id))
    .await
    .map_err(|e| format!("删除下载失败: {e}"))?;
  Ok(())
}

#[tauri::command]
fn open_external_link(url: String) -> Result<(), String> {
  open::that(url).map_err(|err| format!("打开外部链接失败: {err}"))
}

#[tauri::command]
fn list_tracked_subjects(app: tauri::AppHandle) -> Result<Vec<TrackedSubject>, String> {
  let data = load_tracked(&app)?;
  Ok(data.values().cloned().collect())
}

#[tauri::command]
fn save_tracked_subject(app: tauri::AppHandle, subject: TrackedSubject) -> Result<Vec<TrackedSubject>, String> {
  let mut data = load_tracked(&app)?;
  if !subject.watching && !subject.backlog && !subject.watched {
    data.remove(&subject.id);
  } else {
    data.insert(subject.id, subject);
  }
  persist_tracked(&app, &data)?;
  Ok(data.values().cloned().collect())
}

#[tauri::command]
async fn get_season_subjects(year: u32, season: String) -> Result<SeasonResponse, String> {
  let months = season_months(&season)?;
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let mut month_payloads = Vec::new();
  for month in months {
    let list = fetch_month_subjects(&client, year, month).await?;
    let count = list.len();
    month_payloads.push(SeasonMonth {
      year,
      month,
      count,
      list,
    });
  }

  Ok(SeasonResponse {
    year,
    season,
    fetched_at: Utc::now().to_rfc3339(),
    source: format!("{API_BASE}{SUBJECTS_PATH}"),
    months: month_payloads,
  })
}

fn main() {
  tauri::Builder::default()
    .plugin(dialog_plugin())
    .invoke_handler(tauri::generate_handler![
      get_season_subjects,
      get_subject_origin,
      get_subject_aired_count,
      get_subject_filters,
      get_subject_staff,
      get_subject_characters,
      get_subject_summary_cn,
      get_subject_brief,
      fetch_search_html,
      start_torrent_download,
      get_torrent_status,
      finalize_torrent_download,
      parse_media_tracks,
      get_media_file_size,
      mix_media_tracks,
      pause_torrent,
      resume_torrent,
      delete_torrent,
      open_external_link,
      get_subject_aliases,
      list_tracked_subjects,
      save_tracked_subject
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
