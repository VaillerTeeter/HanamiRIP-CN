/* 文件：commands.rs | 用途：暴露 Bangumi 相关命令供前端调用 | 关键对象：各 get_subject_* 命令与翻译逻辑 */
// 本行目的：引入季度条目查询实现。
use super::api::get_season_subjects_impl;
// 本行目的：引入已播集数统计实现。
use super::api::get_subject_aired_count_impl;
// 本行目的：引入别名解析实现。
use super::api::get_subject_aliases_impl;
// 本行目的：引入简要信息查询实现并重命名。
use super::api::get_subject_brief as get_subject_brief_impl;
// 本行目的：引入角色列表查询实现。
use super::api::get_subject_characters_impl;
// 本行目的：引入筛选标签计算实现。
use super::api::get_subject_filters_impl;
// 本行目的：引入原作信息查询实现。
use super::api::get_subject_origin_impl;
// 本行目的：引入制作人员查询实现。
use super::api::get_subject_staff_impl;
// 本行目的：引入季度响应模型。
use super::models::SeasonResponse;
// 本行目的：引入已播集数响应模型。
use super::models::SubjectAiredResponse;
// 本行目的：引入别名响应模型。
use super::models::SubjectAliasesResponse;
// 本行目的：引入简要信息响应模型。
use super::models::SubjectBriefResponse;
// 本行目的：引入角色列表响应模型。
use super::models::SubjectCharactersResponse;
// 本行目的：引入筛选标签响应模型。
use super::models::SubjectFiltersResponse;
// 本行目的：引入原作响应模型。
use super::models::SubjectOriginResponse;
// 本行目的：引入制作人员响应模型。
use super::models::SubjectStaffResponse;
// 本行目的：引入简介翻译响应模型。
use super::models::SubjectSummaryResponse;
// 本行目的：引入中文检测辅助函数。
use super::translate::is_chinese_text;
// 本行目的：引入百度翻译调用函数。
use super::translate::translate_to_cn_baidu;
// 本行目的：引入 HTTP 客户端类型。
use reqwest::Client;

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_origin | 输入：条目 ID | 输出：原作信息响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的原作信息。
pub async fn get_subject_origin(id: u32) -> Result<SubjectOriginResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_origin_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_aired_count | 输入：条目 ID | 输出：已播集数响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的已播集数统计。
pub async fn get_subject_aired_count(id: u32) -> Result<SubjectAiredResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_aired_count_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_filters | 输入：条目 ID | 输出：筛选标签响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的筛选标签。
pub async fn get_subject_filters(id: u32) -> Result<SubjectFiltersResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_filters_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_aliases | 输入：条目 ID | 输出：别名响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的别名列表。
pub async fn get_subject_aliases(id: u32) -> Result<SubjectAliasesResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_aliases_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_staff | 输入：条目 ID | 输出：制作人员响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的制作人员分组。
pub async fn get_subject_staff(id: u32) -> Result<SubjectStaffResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_staff_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_characters | 输入：条目 ID | 输出：角色列表响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的角色列表。
pub async fn get_subject_characters(id: u32) -> Result<SubjectCharactersResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_characters_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_summary_cn | 输入：条目 ID 与原始简介 | 输出：中文简介响应 | 可能失败：网络请求或翻译失败
// 本行目的：获取或翻译条目简介为中文。
pub async fn get_subject_summary_cn(id: u32, summary: String) -> Result<SubjectSummaryResponse, String> {
    // 变量：client | 含义：Bangumi 与翻译请求的 HTTP 客户端 | 类型：Client | 作用域：get_subject_summary_cn
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：raw | 含义：待翻译或返回的简介文本 | 类型：String | 作用域：get_subject_summary_cn
    // 本行目的：接收入参简介作为初始原文。
    let mut raw = summary;

    // 本行目的：当传入简介为空时尝试从接口补齐。
    if raw.trim().is_empty() {
        // 变量：response | 含义：获取条目详情的响应 | 类型：reqwest::Response | 作用域：get_subject_summary_cn
        // 本行目的：请求条目详情接口以获取简介。
        let response = client.get(format!("https://api.bgm.tv/v0/subjects/{id}")).send().await.map_err(|e| e.to_string())?;

        // 本行目的：仅在请求成功时继续解析。
        if response.status().is_success() {
            // 本行目的：尝试解析条目详情以读取简介字段。
            if let Ok(payload) = response.json::<super::models::SubjectDetail>().await {
                // 本行目的：使用接口返回的简介作为原文。
                raw = payload.summary.unwrap_or_default();
            // 本行目的：结束解析成功分支。
            }
        // 本行目的：结束状态检查。
        }
    // 本行目的：结束空简介补齐逻辑。
    }

    // 本行目的：如果仍为空则返回空结果。
    if raw.trim().is_empty() {
        // 本行目的：返回空简介响应并标记未翻译。
        return Ok(SubjectSummaryResponse {
            // 本行目的：写入条目 ID。
            id,
            // 本行目的：写入空简介字符串。
            summary: "".to_string(),
            // 本行目的：标记未发生翻译。
            translated: false,
            // 本行目的：不包含错误信息。
            error: None,
        // 本行目的：结束响应构建。
        });
    // 本行目的：结束空内容返回逻辑。
    }

    // 本行目的：若原文已是中文则直接返回。
    if is_chinese_text(&raw) {
        // 本行目的：返回原文并标记未翻译。
        return Ok(SubjectSummaryResponse {
            // 本行目的：写入条目 ID。
            id,
            // 本行目的：写入原始简介。
            summary: raw,
            // 本行目的：标记未发生翻译。
            translated: false,
            // 本行目的：不包含错误信息。
            error: None,
        // 本行目的：结束响应构建。
        });
    // 本行目的：结束中文检测分支。
    }

    // 本行目的：调用百度翻译并根据结果构建响应。
    match translate_to_cn_baidu(&client, &raw).await {
        // 本行目的：处理翻译成功的情况。
        Ok(result) => {
            // 变量：normalized | 含义：去除首尾空白后的翻译结果 | 类型：String | 作用域：get_subject_summary_cn
            // 本行目的：规范化翻译结果以便判断有效性。
            let normalized = result.trim().to_string();

            // 本行目的：判断翻译是否为空、无变化或非中文。
            if normalized.is_empty() || normalized == raw || !is_chinese_text(&normalized) {
                // 本行目的：返回原文并标记翻译失败。
                Ok(SubjectSummaryResponse {
                    // 本行目的：写入条目 ID。
                    id,
                    // 本行目的：保留原始简介。
                    summary: raw,
                    // 本行目的：标记未成功翻译。
                    translated: false,
                    // 本行目的：填充错误提示。
                    error: Some("翻译失败或未产出中文结果".to_string()),
                // 本行目的：结束响应构建。
                })
            // 本行目的：处理翻译有效的情况。
            } else {
                // 本行目的：返回翻译结果并标记已翻译。
                Ok(SubjectSummaryResponse {
                    // 本行目的：写入条目 ID。
                    id,
                    // 本行目的：写入翻译后的简介。
                    summary: normalized,
                    // 本行目的：标记已成功翻译。
                    translated: true,
                    // 本行目的：不包含错误信息。
                    error: None,
                // 本行目的：结束响应构建。
                })
            // 本行目的：结束翻译有效性判断。
            }
        // 本行目的：结束翻译成功分支。
        }
        // 本行目的：处理翻译失败的情况。
        Err(error) => {
            // 本行目的：返回原文并附带错误信息。
            Ok(SubjectSummaryResponse {
                // 本行目的：写入条目 ID。
                id,
                // 本行目的：保留原始简介。
                summary: raw,
                // 本行目的：标记未成功翻译。
                translated: false,
                // 本行目的：写入错误详情。
                error: Some(error),
            // 本行目的：结束响应构建。
            })
        // 本行目的：结束翻译失败分支。
        }
    // 本行目的：结束翻译结果匹配。
    }
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_subject_brief | 输入：条目 ID | 输出：简要信息响应 | 可能失败：网络请求或解析失败
// 本行目的：获取条目的简要信息。
pub async fn get_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_subject_brief_impl(id).await
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：fetch_search_html | 输入：搜索站点 URL | 输出：HTML 字符串 | 可能失败：网络请求失败
// 本行目的：抓取搜索站点 HTML 内容。
pub async fn fetch_search_html(url: String) -> Result<String, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：fetch_search_html
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：fetch_search_html
    // 本行目的：请求指定 URL。
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查响应是否成功。
    if !response.status().is_success() {
        // 本行目的：返回失败原因包含状态码。
        return Err(format!("搜索站点请求失败: {}", response.status()));
    // 本行目的：结束响应状态检查。
    }

    // 变量：body | 含义：响应体 HTML 文本 | 类型：String | 作用域：fetch_search_html
    // 本行目的：读取响应体为字符串。
    let body = response.text().await.map_err(|e| e.to_string())?;

    // 本行目的：返回抓取到的 HTML 内容。
    Ok(body)
// 本行目的：结束命令函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_season_subjects | 输入：年份与季节 | 输出：季度响应 | 可能失败：网络请求或解析失败
// 本行目的：获取指定季度的番剧列表。
pub async fn get_season_subjects(year: u32, season: String) -> Result<SeasonResponse, String> {
    // 本行目的：调用实现层并透传结果。
    get_season_subjects_impl(year, season).await
// 本行目的：结束命令函数。
}
