/*
  这里集中定义“数据模型”。
  作用：
  1) 把 Bangumi 返回的 JSON 结构映射成 Rust 类型（用于反序列化）。
  2) 把后端要返回给前端的结构定义好（用于序列化）。
  这样前后端交互时就有清晰的“数据契约”。
*/

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 分页条目列表：Bangumi 返回的“分页结构”。
/// total = 总条目数；limit = 每页条数；data = 当前页数据。
#[derive(Deserialize)]
pub(crate) struct PagedSubject {
  pub total: u32,
  pub limit: u32,
  pub data: Vec<Subject>,
}

/// Bangumi 条目（原始结构）。
/// 注意：很多字段是可选的，因为 API 可能不返回。
#[derive(Deserialize)]
pub(crate) struct Subject {
  pub id: u32,
  pub name: String,
  pub name_cn: String,
  pub date: Option<String>,
  pub images: Option<Images>,
  pub summary: Option<String>,
  pub rating: Option<Rating>,
}

/// 条目图片集合（可能为空）。
#[derive(Deserialize)]
pub(crate) struct Images {
  pub common: Option<String>,
  pub medium: Option<String>,
  pub large: Option<String>,
}

/// 评分结构（有些条目没有评分）。
#[derive(Deserialize)]
pub(crate) struct Rating {
  pub score: Option<f64>,
}

/// 分集分页数据。
#[derive(Deserialize)]
pub(crate) struct EpisodePage {
  pub total: u32,
  pub limit: u32,
  pub data: Vec<Episode>,
}

/// 分集信息：只用到“播出日期”和“播出状态”。
#[derive(Deserialize)]
pub(crate) struct Episode {
  pub airdate: Option<String>,
  pub status: Option<String>,
}

/// 条目详情（用于提取标签、原作、别名等）。
#[derive(Deserialize)]
pub(crate) struct SubjectDetail {
  pub summary: Option<String>,
  pub infobox: Option<Vec<InfoboxItem>>,
  pub tags: Option<Vec<SubjectTag>>,
  pub meta_tags: Option<Vec<String>>,
}

/// 标准标签结构。
#[derive(Deserialize)]
pub(crate) struct SubjectTag {
  pub name: String,
}

/// infobox 里的一个键值项。
/// value 是通用 JSON 类型，可能是字符串、数组或对象。
#[derive(Deserialize)]
pub(crate) struct InfoboxItem {
  pub key: String,
  pub value: Value,
}

/// 人物信息（制作人员/声优等）。
#[derive(Deserialize)]
pub(crate) struct SubjectPersonItem {
  pub id: u32,
  pub name: String,
  pub relation: Option<String>,
}

/// 人物接口返回可能是两种形态：
/// - 直接列表
/// - 带分页信息的结构
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum SubjectPersonPayload {
  List(Vec<SubjectPersonItem>),
  Page {
    _total: u32,
    _limit: u32,
    data: Vec<SubjectPersonItem>,
  },
}

/// 角色信息。
#[derive(Deserialize)]
pub(crate) struct SubjectCharacterItem {
  pub id: u32,
  pub name: String,
  pub name_cn: Option<String>,
  pub relation: Option<String>,
}

/// 角色接口返回结构（同样可能是列表或分页）。
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum SubjectCharacterPayload {
  List(Vec<SubjectCharacterItem>),
  Page {
    _total: u32,
    _limit: u32,
    data: Vec<SubjectCharacterItem>,
  },
}

/// 前端使用的“季度番剧条目”。
/// 使用 camelCase 以符合前端常用命名习惯。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonAnime {
  pub id: u32,
  pub name: String,
  pub name_cn: String,
  pub image: String,
  pub date: String,
  pub rating: Option<f64>,
  pub summary: String,
  pub url: String,
}

/// 某个月的番剧列表及统计信息。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonMonth {
  pub year: u32,
  pub month: u32,
  pub count: usize,
  pub list: Vec<SeasonAnime>,
}

/// 某一季的完整响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonResponse {
  pub year: u32,
  pub season: String,
  pub fetched_at: String,
  pub source: String,
  pub months: Vec<SeasonMonth>,
}

/// 原作信息响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectOriginResponse {
  pub id: u32,
  pub origin: Option<String>,
}

/// 已播出集数统计响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectAiredResponse {
  pub id: u32,
  pub aired_count: u32,
  pub total_count: u32,
}

/// 单个制作人员信息（输出给前端）。
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaffPersonResponse {
  pub id: u32,
  pub name: String,
  pub url: String,
}

/// 制作人员分组（按职位）。
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaffGroupResponse {
  pub role: String,
  pub people: Vec<StaffPersonResponse>,
}

/// 制作人员列表响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectStaffResponse {
  pub id: u32,
  pub groups: Vec<StaffGroupResponse>,
}

/// 角色链接信息（用于前端展示）。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterLinkResponse {
  pub id: u32,
  pub name: String,
  pub url: String,
  pub relation: Option<String>,
}

/// 角色列表响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectCharactersResponse {
  pub id: u32,
  pub characters: Vec<CharacterLinkResponse>,
}

/// 别名列表响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectAliasesResponse {
  pub id: u32,
  pub aliases: Vec<String>,
}

/// 简介翻译响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSummaryResponse {
  pub id: u32,
  pub summary: String,
  pub translated: bool,
  pub error: Option<String>,
}

/// 筛选标签响应（类型/地区/受众）。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectFiltersResponse {
  pub id: u32,
  pub types: Vec<String>,
  pub regions: Vec<String>,
  pub audiences: Vec<String>,
}

/// 条目简要信息响应。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectBriefResponse {
  pub id: u32,
  pub name: String,
  pub name_cn: String,
  pub image: String,
  pub date: String,
  pub rating: Option<f64>,
  pub summary: String,
  pub url: String,
}
