use chrono::{NaiveDate, Utc};
use reqwest::Client;

use super::filters::{extract_aliases, extract_filter_groups, extract_origin, collect_subject_tags, map_tags_to_official};
use super::models::{
  CharacterLinkResponse,
  Episode,
  EpisodePage,
  PagedSubject,
  SeasonAnime,
  SeasonMonth,
  SeasonResponse,
  StaffGroupResponse,
  StaffPersonResponse,
  Subject,
  SubjectAiredResponse,
  SubjectAliasesResponse,
  SubjectBriefResponse,
  SubjectCharacterItem,
  SubjectCharacterPayload,
  SubjectCharactersResponse,
  SubjectDetail,
  SubjectFiltersResponse,
  SubjectOriginResponse,
  SubjectPersonItem,
  SubjectPersonPayload,
  SubjectStaffResponse,
};

const API_BASE: &str = "https://api.bgm.tv";
const SUBJECTS_PATH: &str = "/v0/subjects";
const EPISODES_PATH: &str = "/v0/episodes";
const DEFAULT_IMAGE: &str = "https://lain.bgm.tv/img/no_icon_subject.png";

pub(crate) fn season_months(season: &str) -> Result<Vec<u32>, String> {
  match season {
    "winter" => Ok(vec![1, 2, 3]),
    "spring" => Ok(vec![4, 5, 6]),
    "summer" => Ok(vec![7, 8, 9]),
    "autumn" => Ok(vec![10, 11, 12]),
    _ => Err("invalid season".into()),
  }
}

fn resolve_image(images: Option<super::models::Images>) -> String {
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

pub(crate) async fn fetch_subject_persons(
  client: &Client,
  id: u32,
) -> Result<Vec<SubjectPersonItem>, String> {
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

pub(crate) fn build_staff_groups(items: Vec<SubjectPersonItem>) -> Vec<StaffGroupResponse> {
  let mut grouped: std::collections::HashMap<String, Vec<StaffPersonResponse>> = std::collections::HashMap::new();
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

  let mut used = std::collections::HashSet::new();
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

pub(crate) async fn fetch_subject_characters(
  client: &Client,
  id: u32,
) -> Result<Vec<SubjectCharacterItem>, String> {
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

pub(crate) fn map_subject(subject: Subject) -> SeasonAnime {
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

pub(crate) async fn get_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
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

pub(crate) async fn fetch_month_subjects(
  client: &Client,
  year: u32,
  month: u32,
) -> Result<Vec<SeasonAnime>, String> {
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

pub(crate) async fn get_subject_origin_impl(id: u32) -> Result<SubjectOriginResponse, String> {
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

pub(crate) async fn get_subject_aired_count_impl(id: u32) -> Result<SubjectAiredResponse, String> {
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

pub(crate) async fn get_subject_filters_impl(id: u32) -> Result<SubjectFiltersResponse, String> {
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

pub(crate) async fn get_subject_aliases_impl(id: u32) -> Result<SubjectAliasesResponse, String> {
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

pub(crate) async fn get_subject_staff_impl(id: u32) -> Result<SubjectStaffResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let persons = fetch_subject_persons(&client, id).await?;
  let groups = build_staff_groups(persons);
  Ok(SubjectStaffResponse { id, groups })
}

pub(crate) async fn get_subject_characters_impl(id: u32) -> Result<SubjectCharactersResponse, String> {
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

pub(crate) async fn get_season_subjects_impl(
  year: u32,
  season: String,
) -> Result<SeasonResponse, String> {
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
