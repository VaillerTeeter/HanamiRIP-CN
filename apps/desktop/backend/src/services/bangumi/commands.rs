/*
  这个文件定义的是 Tauri 的“命令入口”。
  前端调用这些命令时，会进入这里，然后再调用真正的业务实现。
  简单理解：这里是“前端到后端”的桥梁。
*/

// HTTP 客户端，用于少量需要直接请求的命令。
use reqwest::Client;

// 这些是实际业务实现（真正干活的函数）。
// commands 里只是“转发/包装”。
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

// 对外返回的数据结构。
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

// 文本语言判断与翻译工具。
use super::translate::{is_chinese_text, translate_to_cn_baidu};

// 下面的函数带有 #[tauri::command]，表示它们能被前端调用。
// 每个命令都尽量保持“薄封装”：不做复杂逻辑，直接调用实现层。

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

// 获取条目简介的中文摘要：
// - 如果传入摘要为空，则先从 Bangumi 获取原始摘要；
// - 如果摘要已经是中文，就直接返回；
// - 否则调用百度翻译，得到中文结果。
#[tauri::command]
pub async fn get_subject_summary_cn(id: u32, summary: String) -> Result<SubjectSummaryResponse, String> {
  // 构建 HTTP 客户端。
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  // raw 是“待处理的摘要”。
  let mut raw = summary;
  if raw.trim().is_empty() {
    // 如果前端没有传摘要，就从 Bangumi 获取。
    let response = client
      .get(format!("https://api.bgm.tv/v0/subjects/{id}"))
      .send()
      .await
      .map_err(|e| e.to_string())?;

    // 只有请求成功才尝试解析 JSON。
    if response.status().is_success() {
      if let Ok(payload) = response.json::<super::models::SubjectDetail>().await {
        raw = payload.summary.unwrap_or_default();
      }
    }
  }

  // 仍然为空：直接返回空摘要，标记未翻译。
  if raw.trim().is_empty() {
    return Ok(SubjectSummaryResponse {
      id,
      summary: "".to_string(),
      translated: false,
      error: None,
    });
  }

  // 如果已经是中文，就不翻译，避免多余请求。
  if is_chinese_text(&raw) {
    return Ok(SubjectSummaryResponse {
      id,
      summary: raw,
      translated: false,
      error: None,
    });
  }

  // 非中文则尝试翻译。
  match translate_to_cn_baidu(&client, &raw).await {
    Ok(result) => {
      // 去掉首尾空白，避免“看起来有内容但其实空”。
      let normalized = result.trim().to_string();
      if normalized.is_empty() || normalized == raw || !is_chinese_text(&normalized) {
        // 翻译失败：没有有效中文输出。
        Ok(SubjectSummaryResponse {
          id,
          summary: raw,
          translated: false,
          error: Some("翻译失败或未产出中文结果".to_string()),
        })
      } else {
        // 翻译成功。
        Ok(SubjectSummaryResponse {
          id,
          summary: normalized,
          translated: true,
          error: None,
        })
      }
    }
    Err(error) => Ok(SubjectSummaryResponse {
      // 翻译接口报错时，保留原文并带上错误信息。
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

// 获取搜索站点的 HTML 原文，通常用于爬取/解析。
#[tauri::command]
pub async fn fetch_search_html(url: String) -> Result<String, String> {
  // 构建 HTTP 客户端。
  let client = Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  // 拉取目标页面内容。
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
