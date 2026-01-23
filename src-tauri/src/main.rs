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

const API_BASE: &str = "https://api.bgm.tv";
const SUBJECTS_PATH: &str = "/v0/subjects";
const EPISODES_PATH: &str = "/v0/episodes";
const DEFAULT_IMAGE: &str = "https://lain.bgm.tv/img/no_icon_subject.png";
const TRACK_DB_FILE: &str = "watchlist.json";

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
    let lib = unsafe { Library::new(&lib_path) }
      .map_err(|e| format!("加载百度翻译校验库失败: {e}"))?;
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

  Ok(SubjectAiredResponse { id, aired_count })
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
    .invoke_handler(tauri::generate_handler![
      get_season_subjects,
      get_subject_origin,
      get_subject_aired_count,
      get_subject_filters,
      get_subject_staff,
      get_subject_characters,
      get_subject_summary_cn,
      get_subject_brief,
      list_tracked_subjects,
      save_tracked_subject
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
