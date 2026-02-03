use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub(crate) struct PagedSubject {
  pub total: u32,
  pub limit: u32,
  pub data: Vec<Subject>,
}

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

#[derive(Deserialize)]
pub(crate) struct Images {
  pub common: Option<String>,
  pub medium: Option<String>,
  pub large: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Rating {
  pub score: Option<f64>,
}

#[derive(Deserialize)]
pub(crate) struct EpisodePage {
  pub total: u32,
  pub limit: u32,
  pub data: Vec<Episode>,
}

#[derive(Deserialize)]
pub(crate) struct Episode {
  pub airdate: Option<String>,
  pub status: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SubjectDetail {
  pub summary: Option<String>,
  pub infobox: Option<Vec<InfoboxItem>>,
  pub tags: Option<Vec<SubjectTag>>,
  pub meta_tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct SubjectTag {
  pub name: String,
}

#[derive(Deserialize)]
pub(crate) struct InfoboxItem {
  pub key: String,
  pub value: Value,
}

#[derive(Deserialize)]
pub(crate) struct SubjectPersonItem {
  pub id: u32,
  pub name: String,
  pub relation: Option<String>,
}

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

#[derive(Deserialize)]
pub(crate) struct SubjectCharacterItem {
  pub id: u32,
  pub name: String,
  pub name_cn: Option<String>,
  pub relation: Option<String>,
}

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonMonth {
  pub year: u32,
  pub month: u32,
  pub count: usize,
  pub list: Vec<SeasonAnime>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonResponse {
  pub year: u32,
  pub season: String,
  pub fetched_at: String,
  pub source: String,
  pub months: Vec<SeasonMonth>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectOriginResponse {
  pub id: u32,
  pub origin: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectAiredResponse {
  pub id: u32,
  pub aired_count: u32,
  pub total_count: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaffPersonResponse {
  pub id: u32,
  pub name: String,
  pub url: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaffGroupResponse {
  pub role: String,
  pub people: Vec<StaffPersonResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectStaffResponse {
  pub id: u32,
  pub groups: Vec<StaffGroupResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterLinkResponse {
  pub id: u32,
  pub name: String,
  pub url: String,
  pub relation: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectCharactersResponse {
  pub id: u32,
  pub characters: Vec<CharacterLinkResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectAliasesResponse {
  pub id: u32,
  pub aliases: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSummaryResponse {
  pub id: u32,
  pub summary: String,
  pub translated: bool,
  pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectFiltersResponse {
  pub id: u32,
  pub types: Vec<String>,
  pub regions: Vec<String>,
  pub audiences: Vec<String>,
}

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