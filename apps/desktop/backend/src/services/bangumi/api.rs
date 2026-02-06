/*
  这一文件负责“和 Bangumi 官方 API 打交道”。
  你可以把它理解为：所有网络请求、数据拉取、数据整理都在这里完成。
  其他模块只需要调用这里的函数，就能拿到整理好的结果。
*/

// 时间处理库：用来把“字符串日期”变成真正的日期对象，便于比较。
use chrono::{NaiveDate, Utc};
// HTTP 客户端：负责发起网络请求。
use reqwest::Client;

// 这些是“过滤/解析工具函数”，把 Bangumi 返回的杂乱数据整理成可用信息。
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

// Bangumi API 的基础地址。
const API_BASE: &str = "https://api.bgm.tv";
// 获取条目信息的路径。
const SUBJECTS_PATH: &str = "/v0/subjects";
// 获取分集信息的路径。
const EPISODES_PATH: &str = "/v0/episodes";
// 当条目没有图片时使用的默认图片。
const DEFAULT_IMAGE: &str = "https://lain.bgm.tv/img/no_icon_subject.png";

// 把“季节字符串”转换成对应月份列表。
// 例如 winter -> [1,2,3]，spring -> [4,5,6]。
// 返回 Result 是因为传入的季节可能不合法。
pub(crate) fn season_months(season: &str) -> Result<Vec<u32>, String> {
  match season {
    "winter" => Ok(vec![1, 2, 3]),
    "spring" => Ok(vec![4, 5, 6]),
    "summer" => Ok(vec![7, 8, 9]),
    "autumn" => Ok(vec![10, 11, 12]),
    _ => Err("invalid season".into()),
  }
}

// 从一组可选图片地址里挑一个“可用的”。
// 优先顺序：common -> medium -> large；都没有就返回默认图。
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

// 把字符串日期解析成 NaiveDate（没有时区的日期）。
// 如果格式不对或为空，就返回 None。
fn parse_airdate(value: &Option<String>) -> Option<NaiveDate> {
  let raw = value.as_ref()?;
  NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok()
}

// 判断“这一集是否已播出”。
// 规则：
// 1) 如果状态是 "air"，直接算已播出；
// 2) 否则尝试比较 airdate 是否 <= 今天。
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

// 获取条目的“制作人员/声优等人物列表”。
// 这里会处理分页/非分页两种返回格式。
pub(crate) async fn fetch_subject_persons(
  client: &Client,
  id: u32,
) -> Result<Vec<SubjectPersonItem>, String> {
  // 发起网络请求。
  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}/persons"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  // 非 2xx 都视为失败，并给出明确的错误信息。
  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  // 解析 JSON，如果失败直接把错误转成字符串返回。
  let payload: SubjectPersonPayload = response.json().await.map_err(|e| e.to_string())?;
  // Bangumi 可能返回“列表”或“分页结构”，这里统一成 Vec。
  let items = match payload {
    SubjectPersonPayload::List(list) => list,
    SubjectPersonPayload::Page { data, .. } => data,
  };

  Ok(items)
}

// 把“人物列表”整理成“按职位分组”的结构。
// 例如：导演、脚本、音乐各自成为一个分组，便于前端展示。
pub(crate) fn build_staff_groups(items: Vec<SubjectPersonItem>) -> Vec<StaffGroupResponse> {
  // 用 HashMap 按“职位名称”分组。
  let mut grouped: std::collections::HashMap<String, Vec<StaffPersonResponse>> = std::collections::HashMap::new();
  for item in items {
    // relation 为空或全空白就跳过，这些数据无法分类。
    let relation = match item.relation {
      Some(value) if !value.trim().is_empty() => value,
      _ => continue,
    };
    // 把原始条目转换成“前端需要的人员结构”。
    let person = StaffPersonResponse {
      id: item.id,
      name: item.name,
      url: format!("https://bgm.tv/person/{}", item.id),
    };
    grouped.entry(relation).or_default().push(person);
  }

  // 预设一个“优先排序”的职位列表，让常见职位排在前面。
  let ordered_roles = vec![
    "导演", "脚本", "分镜", "演出", "音乐", "人物设定", "系列构成", "美术监督", "色彩设计",
    "总作画监督", "作画监督", "摄影监督", "道具设计", "剪辑", "剪辑助手", "主题歌编曲",
    "主题歌作曲",
  ];

  // used 用来记录已经输出过的职位，避免重复。
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

  // 把剩余的职位按名称排序，保证输出稳定。
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

// 获取条目的“角色列表”。
// 逻辑与 fetch_subject_persons 类似：请求、校验、解析。
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

// 把 Bangumi 的 Subject 数据转换成我们前端更好用的 SeasonAnime。
// 这里会：
// - 补默认图
// - 处理可选字段
// - 生成条目 URL
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

// 获取“条目简介”。这是一个轻量级接口：
// 只保留前端最常用字段。
pub(crate) async fn get_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
  // 构建 HTTP 客户端，并设置 user_agent。
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  // 请求条目详情。
  let response = client
    .get(format!("{API_BASE}{SUBJECTS_PATH}/{id}"))
    .send()
    .await
    .map_err(|e| e.to_string())?;

  // 状态码不成功则直接返回错误。
  if !response.status().is_success() {
    return Err(format!("Bangumi API 请求失败: {}", response.status()));
  }

  // 把 JSON 解析成 Subject 结构体。
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

// 拉取“某年某月”的番剧列表。
// Bangumi API 有分页，所以这里需要循环拉取直到数据结束。
pub(crate) async fn fetch_month_subjects(
  client: &Client,
  year: u32,
  month: u32,
) -> Result<Vec<SeasonAnime>, String> {
  // offset：从第几条开始；limit：每次拉取数量。
  let mut offset = 0u32;
  let limit = 50u32;
  // items 用来累积所有结果。
  let mut items = Vec::new();

  loop {
    // 分页请求：带上 offset 和 limit。
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

    // 解析分页数据。
    let payload: PagedSubject = response.json().await.map_err(|e| e.to_string())?;
    if payload.data.is_empty() {
      break;
    }

    // 把每条 Subject 映射成 SeasonAnime 并累积。
    items.extend(payload.data.into_iter().map(map_subject));

    // 递增 offset，确保下一页不重复。
    offset += payload.limit.max(limit);
    if offset >= payload.total {
      break;
    }
  }

  Ok(items)
}

// 获取“原作来源”（例如：漫画、小说、原创）。
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

  // 解析完整详情，随后从 infobox 中提取“原作”。
  let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
  let origin = extract_origin(payload.infobox);
  Ok(SubjectOriginResponse { id, origin })
}

// 获取“已播出集数”。
// 逻辑：分页拉取分集列表，然后根据状态/日期判断是否已播出。
pub(crate) async fn get_subject_aired_count_impl(id: u32) -> Result<SubjectAiredResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  // 分页参数与统计计数。
  let mut offset = 0u32;
  let limit = 50u32;
  let mut aired_count = 0u32;
  let mut total_count = 0u32;
  // 以“今天”的日期作为比较基准。
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

    // 解析分集分页数据。
    let payload: EpisodePage = response.json().await.map_err(|e| e.to_string())?;
    if total_count == 0 {
      total_count = payload.total;
    }

    // 逐集判断是否已播出。
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

// 获取“条目的筛选标签”（类型/地区/受众）。
// 会把官方标签 + infobox 信息 + 别名统一规范化。
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
  // 从 infobox 中提取类型/地区/受众。
  let (info_types, info_regions, info_audiences) = extract_filter_groups(payload.infobox);
  // 收集原始标签，再把 infobox 的结果合并进去。
  let mut tags = collect_subject_tags(payload.tags, payload.meta_tags);
  tags.extend(info_types.clone());
  tags.extend(info_regions.clone());
  tags.extend(info_audiences.clone());

  // 官方规范“类型列表”。
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

  // 官方规范“地区列表”。
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

  // 官方规范“受众列表”。
  const OFFICIAL_AUDIENCES: &[&str] = &[
    "BL",
    "GL",
    "子供向",
    "女性向",
    "少女向",
    "少年向",
    "青年向",
  ];

  // 类型别名映射：把常见说法统一成官方标准。
  const TYPE_ALIASES: &[(&str, &str)] = &[
    ("搞笑", "喜剧"),
    ("恋愛", "恋爱"),
    ("日常系", "日常"),
    ("熱血", "战斗"),
    ("机甲", "机战"),
    ("萌", "萌系"),
  ];

  // 地区别名映射。
  const REGION_ALIASES: &[(&str, &str)] = &[
    ("大陆", "中国"),
    ("中国大陆", "中国"),
    ("香港", "中国香港"),
    ("台湾", "中国台湾"),
    ("欧美动画", "欧美"),
    ("欧洲", "欧美"),
    ("俄国", "俄罗斯"),
  ];

  // 受众别名映射。
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

  // 把所有标签归一化到官方列表里。
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

// 获取条目的“别名列表”。
// 别名可能来自 infobox 的多种字段（中英日等）。
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

// 获取并分组条目的“制作人员”。
pub(crate) async fn get_subject_staff_impl(id: u32) -> Result<SubjectStaffResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let persons = fetch_subject_persons(&client, id).await?;
  let groups = build_staff_groups(persons);
  Ok(SubjectStaffResponse { id, groups })
}

// 获取条目的“角色列表”，并转换为前端需要的结构。
pub(crate) async fn get_subject_characters_impl(id: u32) -> Result<SubjectCharactersResponse, String> {
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  let characters = fetch_subject_characters(&client, id).await?;
  // 映射角色名称：优先中文名，没有就用原名。
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

// 获取“某年某季”的番剧列表。
// 会先把季节转换成月份，再逐月拉取并汇总。
pub(crate) async fn get_season_subjects_impl(
  year: u32,
  season: String,
) -> Result<SeasonResponse, String> {
  // 如果季节字符串不合法，这里会直接返回错误。
  let months = season_months(&season)?;
  let client = reqwest::Client::builder()
    .user_agent("HanamiRIP-CN/0.1")
    .build()
    .map_err(|e| e.to_string())?;

  // month_payloads 用于保存每个月的结果。
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
