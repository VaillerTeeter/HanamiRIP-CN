use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfoResponse {
  pub track_id: String,
  pub codec: String,
  pub lang: Option<String>,
  pub language_name: Option<String>,
  pub track_name: Option<String>,
  pub is_default: Option<bool>,
  pub is_forced: Option<bool>,
  pub charset: Option<String>,
  pub attributes: Option<String>,
  pub container: Option<String>,
  pub file_size: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackParseResponse {
  pub tracks: Vec<TrackInfoResponse>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MixTrackInput {
  pub path: String,
  pub kind: String,
  pub track_ids: Vec<String>,
  #[serde(default)]
  pub track_langs: HashMap<String, String>,
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
    let dev_bin = PathBuf::from("../public").join("tools");
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
pub async fn parse_media_tracks(
  app: tauri::AppHandle,
  path: String,
  kind: String,
) -> Result<TrackParseResponse, String> {
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
pub async fn get_media_file_size(path: String) -> Result<Option<String>, String> {
  let meta = fs::metadata(&path).map_err(|e| format!("读取文件大小失败: {e}"))?;
  Ok(Some(format_bytes_readable(meta.len())))
}

#[tauri::command]
pub async fn mix_media_tracks(
  app: tauri::AppHandle,
  inputs: Vec<MixTrackInput>,
  output_path: String,
) -> Result<String, String> {
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
    .join(chrono::Utc::now().timestamp_millis().to_string());
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
