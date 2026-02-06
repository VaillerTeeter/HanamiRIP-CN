/*
  本地存储模块：
  用于保存“追番列表”（watchlist）。
  数据以 JSON 文件形式落地在应用数据目录中。
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

// 追番数据库文件名。
const TRACK_DB_FILE: &str = "watchlist.json";

/// 单个追番条目的存储结构。
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

// 获取追番数据库文件的完整路径。
// 如果目录不存在会自动创建。
fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("无法获取数据目录: {e}"))?
    .join("hanamirip-cn");
  fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
  Ok(dir.join(TRACK_DB_FILE))
}

// 从 JSON 文件读取追番数据，转换成 HashMap（key=条目 ID）。
fn load_tracked(app: &tauri::AppHandle) -> Result<HashMap<u32, TrackedSubject>, String> {
  let path = db_path(app)?;
  // 文件不存在表示暂无数据。
  if !path.exists() {
    return Ok(HashMap::new());
  }
  let content = fs::read_to_string(&path).map_err(|e| format!("读取追番数据失败: {e}"))?;
  // 空文件也视为无数据。
  if content.trim().is_empty() {
    return Ok(HashMap::new());
  }
  // 读取的是数组，所以先解析成 Vec，再转 HashMap。
  let parsed: Vec<TrackedSubject> = serde_json::from_str(&content)
    .map_err(|e| format!("解析追番数据失败: {e}"))?;
  let mut map = HashMap::new();
  for item in parsed {
    map.insert(item.id, item);
  }
  Ok(map)
}

// 把追番数据写回 JSON 文件（格式化输出）。
fn persist_tracked(app: &tauri::AppHandle, data: &HashMap<u32, TrackedSubject>) -> Result<(), String> {
  let path = db_path(app)?;
  // HashMap 没有顺序，这里只需要保存内容即可。
  let list: Vec<_> = data.values().cloned().collect();
  let payload = serde_json::to_string_pretty(&list)
    .map_err(|e| format!("序列化追番数据失败: {e}"))?;
  fs::write(&path, payload).map_err(|e| format!("写入追番数据失败: {e}"))
}

// 返回所有追番条目。
#[tauri::command]
pub fn list_tracked_subjects(app: tauri::AppHandle) -> Result<Vec<TrackedSubject>, String> {
  let data = load_tracked(&app)?;
  Ok(data.values().cloned().collect())
}

// 新增或更新一个追番条目。
// 如果该条目三个状态都为 false，则视为“移除”。
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
