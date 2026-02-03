use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const TRACK_DB_FILE: &str = "watchlist.json";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackedSubject {
  pub id: u32,
  pub name: String,
  pub name_cn: String,
  pub image: String,
  pub url: String,
  pub watching: bool,
  pub backlog: bool,
  pub watched: bool,
  pub date: String,
  pub rating: Option<f64>,
  pub summary: String,
  #[serde(default)]
  pub aliases: Option<Vec<String>>, // 英文名/罗马音等别名列表
  #[serde(default)]
  pub aired_count: Option<u32>,
  #[serde(default)]
  pub total_count: Option<u32>,
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

#[tauri::command]
pub fn list_tracked_subjects(app: tauri::AppHandle) -> Result<Vec<TrackedSubject>, String> {
  let data = load_tracked(&app)?;
  Ok(data.values().cloned().collect())
}

#[tauri::command]
pub fn save_tracked_subject(
  app: tauri::AppHandle,
  subject: TrackedSubject,
) -> Result<Vec<TrackedSubject>, String> {
  let mut data = load_tracked(&app)?;
  if !subject.watching && !subject.backlog && !subject.watched {
    data.remove(&subject.id);
  } else {
    data.insert(subject.id, subject);
  }
  persist_tracked(&app, &data)?;
  Ok(data.values().cloned().collect())
}