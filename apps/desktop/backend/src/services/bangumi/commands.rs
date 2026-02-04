use reqwest::Client;

use super::api::{
  get_season_subjects_impl,
  get_subject_aired_count_impl,
  get_subject_aliases_impl,
  get_subject_brief as get_subject_brief_impl,
  get_subject_characters_impl,
  get_subject_filters_impl,
  get_subject_origin_impl,
  get_subject_staff_impl,
};
use super::models::{
  SeasonResponse,
  SubjectAiredResponse,
  SubjectAliasesResponse,
  SubjectBriefResponse,
  SubjectCharactersResponse,
  SubjectFiltersResponse,
  SubjectOriginResponse,
  SubjectStaffResponse,
  SubjectSummaryResponse,
};
use super::translate::{is_chinese_text, translate_to_cn_baidu};

#[tauri::command]
pub async fn get_subject_origin(id: u32) -> Result<SubjectOriginResponse, String> {
  get_subject_origin_impl(id).await
}

#[tauri::command]
pub async fn get_subject_aired_count(id: u32) -> Result<SubjectAiredResponse, String> {
  get_subject_aired_count_impl(id).await
}

#[tauri::command]
pub async fn get_subject_filters(id: u32) -> Result<SubjectFiltersResponse, String> {
  get_subject_filters_impl(id).await
}

#[tauri::command]
pub async fn get_subject_aliases(id: u32) -> Result<SubjectAliasesResponse, String> {
  get_subject_aliases_impl(id).await
}

#[tauri::command]
pub async fn get_subject_staff(id: u32) -> Result<SubjectStaffResponse, String> {
  get_subject_staff_impl(id).await
}

#[tauri::command]
pub async fn get_subject_characters(id: u32) -> Result<SubjectCharactersResponse, String> {
  get_subject_characters_impl(id).await
}

#[tauri::command]
pub async fn get_subject_summary_cn(id: u32, summary: String) -> Result<SubjectSummaryResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let mut raw = summary;
  if raw.trim().is_empty() {
    let response = client
      .get(format!("https://api.bgm.tv/v0/subjects/{id}"))
      .send()
      .await
      .map_err(|e| e.to_string())?;

    if response.status().is_success() {
      if let Ok(payload) = response.json::<super::models::SubjectDetail>().await {
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
pub async fn get_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
  get_subject_brief_impl(id).await
}

#[tauri::command]
pub async fn fetch_search_html(url: String) -> Result<String, String> {
  let client = Client::builder()
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
pub async fn get_season_subjects(year: u32, season: String) -> Result<SeasonResponse, String> {
  get_season_subjects_impl(year, season).await
}
