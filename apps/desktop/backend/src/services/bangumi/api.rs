/* 文件：api.rs | 用途：封装 Bangumi API 访问与数据映射 | 关键对象：fetch_*、get_*、map_subject */
// 本行目的：引入标签收集函数。
use super::filters::collect_subject_tags;
// 本行目的：引入别名提取函数。
use super::filters::extract_aliases;
// 本行目的：引入筛选分组提取函数。
use super::filters::extract_filter_groups;
// 本行目的：引入原作提取函数。
use super::filters::extract_origin;
// 本行目的：引入官方标签映射函数。
use super::filters::map_tags_to_official;
// 本行目的：引入角色链接响应模型。
use super::models::CharacterLinkResponse;
// 本行目的：引入剧集模型。
use super::models::Episode;
// 本行目的：引入剧集分页模型。
use super::models::EpisodePage;
// 本行目的：引入分页条目模型。
use super::models::PagedSubject;
// 本行目的：引入季度动画模型。
use super::models::SeasonAnime;
// 本行目的：引入季度月份模型。
use super::models::SeasonMonth;
// 本行目的：引入季度响应模型。
use super::models::SeasonResponse;
// 本行目的：引入制作人员分组响应模型。
use super::models::StaffGroupResponse;
// 本行目的：引入制作人员响应模型。
use super::models::StaffPersonResponse;
// 本行目的：引入条目基础模型。
use super::models::Subject;
// 本行目的：引入已播统计响应模型。
use super::models::SubjectAiredResponse;
// 本行目的：引入别名响应模型。
use super::models::SubjectAliasesResponse;
// 本行目的：引入简要响应模型。
use super::models::SubjectBriefResponse;
// 本行目的：引入角色条目模型。
use super::models::SubjectCharacterItem;
// 本行目的：引入角色响应载荷模型。
use super::models::SubjectCharacterPayload;
// 本行目的：引入角色响应模型。
use super::models::SubjectCharactersResponse;
// 本行目的：引入条目详情模型。
use super::models::SubjectDetail;
// 本行目的：引入筛选响应模型。
use super::models::SubjectFiltersResponse;
// 本行目的：引入原作响应模型。
use super::models::SubjectOriginResponse;
// 本行目的：引入人员条目模型。
use super::models::SubjectPersonItem;
// 本行目的：引入人员响应载荷模型。
use super::models::SubjectPersonPayload;
// 本行目的：引入制作人员响应模型。
use super::models::SubjectStaffResponse;
// 本行目的：引入日期类型。
use chrono::NaiveDate;
// 本行目的：引入 UTC 时间工具。
use chrono::Utc;
// 本行目的：引入 HTTP 客户端类型。
use reqwest::Client;

// 变量：API_BASE | 含义：Bangumi API 基础地址 | 类型：&str | 作用域：模块级
// 本行目的：定义 API 基础地址常量。
const API_BASE: &str = "https://api.bgm.tv";
// 变量：SUBJECTS_PATH | 含义：条目 API 路径 | 类型：&str | 作用域：模块级
// 本行目的：定义条目接口路径常量。
const SUBJECTS_PATH: &str = "/v0/subjects";
// 变量：EPISODES_PATH | 含义：剧集 API 路径 | 类型：&str | 作用域：模块级
// 本行目的：定义剧集接口路径常量。
const EPISODES_PATH: &str = "/v0/episodes";
// 变量：DEFAULT_IMAGE | 含义：默认条目图片 URL | 类型：&str | 作用域：模块级
// 本行目的：定义默认图片地址常量。
const DEFAULT_IMAGE: &str = "https://lain.bgm.tv/img/no_icon_subject.png";

/// 函数：season_months | 输入：季节字符串 | 输出：月份列表 | 可能失败：季节字符串无效
// 本行目的：将季节映射为对应的月份列表。
pub(crate) fn season_months(season: &str) -> Result<Vec<u32>, String> {
    // 本行目的：根据季节名称返回对应月份。
    match season {
        // 本行目的：冬季对应 1-3 月。
        "winter" => Ok(vec![1, 2, 3]),
        // 本行目的：春季对应 4-6 月。
        "spring" => Ok(vec![4, 5, 6]),
        // 本行目的：夏季对应 7-9 月。
        "summer" => Ok(vec![7, 8, 9]),
        // 本行目的：秋季对应 10-12 月。
        "autumn" => Ok(vec![10, 11, 12]),
        // 本行目的：其他值返回错误。
        _ => Err("invalid season".into()),
    // 本行目的：结束季节匹配。
    }
// 本行目的：结束季节月份映射函数。
}

/// 函数：resolve_image | 输入：可选图片集合 | 输出：图片 URL | 可能失败：无
// 本行目的：按优先级选择可用图片 URL。
fn resolve_image(images: Option<super::models::Images>) -> String {
    // 本行目的：处理存在图片集合的情况。
    if let Some(images) = images {
        // 本行目的：优先返回 common 图片。
        if let Some(url) = images.common {
            // 本行目的：返回 common 图片 URL。
            return url;
        // 本行目的：结束 common 判断。
        }

        // 本行目的：次选 medium 图片。
        if let Some(url) = images.medium {
            // 本行目的：返回 medium 图片 URL。
            return url;
        // 本行目的：结束 medium 判断。
        }

        // 本行目的：最后选 large 图片。
        if let Some(url) = images.large {
            // 本行目的：返回 large 图片 URL。
            return url;
        // 本行目的：结束 large 判断。
        }
    // 本行目的：结束图片集合存在判断。
    }

    // 本行目的：返回默认图片 URL。
    DEFAULT_IMAGE.to_string()
// 本行目的：结束图片解析函数。
}

/// 函数：parse_airdate | 输入：可选日期字符串 | 输出：可选日期 | 可能失败：日期格式不正确
// 本行目的：将日期字符串解析为 NaiveDate。
fn parse_airdate(value: &Option<String>) -> Option<NaiveDate> {
    // 变量：raw | 含义：原始日期字符串引用 | 类型：&String | 作用域：parse_airdate
    // 本行目的：提取日期字符串引用。
    let raw = value.as_ref()?;
    // 本行目的：按 YYYY-MM-DD 格式解析日期。
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok()
// 本行目的：结束日期解析函数。
}

/// 函数：is_aired | 输入：剧集与当前日期 | 输出：是否已播 | 可能失败：无
// 本行目的：判断剧集是否已经播出。
fn is_aired(episode: &Episode, today: NaiveDate) -> bool {
    // 本行目的：如果状态字段明确为已播则返回 true。
    if let Some(status) = &episode.status {
        // 本行目的：检查状态值是否为 air。
        if status == "air" {
            // 本行目的：状态为 air 时认为已播。
            return true;
        // 本行目的：结束状态判断。
        }
    // 本行目的：结束状态存在判断。
    }

    // 本行目的：若有播出日期则比较是否早于今日。
    if let Some(date) = parse_airdate(&episode.airdate) {
        // 本行目的：返回日期是否小于等于今日。
        return date <= today;
    // 本行目的：结束日期解析判断。
    }

    // 本行目的：默认认为未播出。
    false
// 本行目的：结束已播判断函数。
}

/// 函数：fetch_subject_persons | 输入：HTTP 客户端与条目 ID | 输出：人物列表 | 可能失败：网络或解析失败
// 本行目的：请求条目人物信息。
pub(crate) async fn fetch_subject_persons(client: &Client, id: u32) -> Result<Vec<SubjectPersonItem>, String> {
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：fetch_subject_persons
    // 本行目的：请求条目人物接口。
    let response = client.get(format!("{API_BASE}{SUBJECTS_PATH}/{id}/persons")).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查请求是否成功。
    if !response.status().is_success() {
        // 本行目的：返回包含状态码的错误信息。
        return Err(format!("Bangumi API 请求失败: {}", response.status()));
    // 本行目的：结束状态检查。
    }

    // 变量：payload | 含义：人物响应载荷 | 类型：SubjectPersonPayload | 作用域：fetch_subject_persons
    // 本行目的：解析响应 JSON 为载荷。
    let payload: SubjectPersonPayload = response.json().await.map_err(|e| e.to_string())?;
    // 变量：items | 含义：人物列表 | 类型：Vec<SubjectPersonItem> | 作用域：fetch_subject_persons
    // 本行目的：根据载荷类型提取人物列表。
    let items = match payload {
        // 本行目的：处理列表载荷。
        SubjectPersonPayload::List(list) => list,
        // 本行目的：处理分页载荷并取 data 字段。
        SubjectPersonPayload::Page {
            // 本行目的：提取 data 字段。
            data, ..
        } => data,
    // 本行目的：结束载荷匹配。
    };

    // 本行目的：返回人物列表。
    Ok(items)
// 本行目的：结束人物请求函数。
}

/// 函数：build_staff_groups | 输入：人物列表 | 输出：按职务分组的人员列表 | 可能失败：无
// 本行目的：将人员按职务分组并排序。
pub(crate) fn build_staff_groups(items: Vec<SubjectPersonItem>) -> Vec<StaffGroupResponse> {
    // 变量：grouped | 含义：职务到人员列表的映射 | 类型：HashMap<String, Vec<StaffPersonResponse>> | 作用域：build_staff_groups
    // 本行目的：初始化分组映射。
    let mut grouped: std::collections::HashMap<String, Vec<StaffPersonResponse>> = std::collections::HashMap::new();

    // 本行目的：遍历人物列表并构建分组。
    for item in items {
        // 变量：relation | 含义：职务名称 | 类型：String | 作用域：build_staff_groups
        // 本行目的：筛选出有效职务名称。
        let relation = match item.relation {
            // 本行目的：保留非空职务名称。
            Some(value) if !value.trim().is_empty() => value,
            // 本行目的：无效职务则跳过该人物。
            _ => continue,
        // 本行目的：结束职务匹配。
        };

        // 变量：person | 含义：人员响应对象 | 类型：StaffPersonResponse | 作用域：build_staff_groups
        // 本行目的：构建人员响应对象。
        let person = StaffPersonResponse {
            // 本行目的：写入人物 ID。
            id: item.id,
            // 本行目的：写入人物名称。
            name: item.name,
            // 本行目的：构建人物详情链接。
            url: format!("https://bgm.tv/person/{}", item.id),
        // 本行目的：结束人员对象构建。
        };

        // 本行目的：将人员加入对应职务分组。
        grouped.entry(relation).or_default().push(person);
    // 本行目的：结束人物遍历。
    }

    // 变量：ordered_roles | 含义：预定义职务排序列表 | 类型：Vec<&str> | 作用域：build_staff_groups
    // 本行目的：定义优先排序的职务名称。
    let ordered_roles = vec![
        // 本行目的：导演职务。
        "导演",
        // 本行目的：脚本职务。
        "脚本",
        // 本行目的：分镜职务。
        "分镜",
        // 本行目的：演出职务。
        "演出",
        // 本行目的：音乐职务。
        "音乐",
        // 本行目的：人物设定职务。
        "人物设定",
        // 本行目的：系列构成职务。
        "系列构成",
        // 本行目的：美术监督职务。
        "美术监督",
        // 本行目的：色彩设计职务。
        "色彩设计",
        // 本行目的：总作画监督职务。
        "总作画监督",
        // 本行目的：作画监督职务。
        "作画监督",
        // 本行目的：摄影监督职务。
        "摄影监督",
        // 本行目的：道具设计职务。
        "道具设计",
        // 本行目的：剪辑职务。
        "剪辑",
        // 本行目的：剪辑助手职务。
        "剪辑助手",
        // 本行目的：主题歌编曲职务。
        "主题歌编曲",
        // 本行目的：主题歌作曲职务。
        "主题歌作曲",
    // 本行目的：结束排序列表定义。
    ];

    // 变量：used | 含义：已使用的职务集合 | 类型：HashSet<String> | 作用域：build_staff_groups
    // 本行目的：初始化已使用职务集合。
    let mut used = std::collections::HashSet::new();
    // 变量：groups | 含义：最终分组列表 | 类型：Vec<StaffGroupResponse> | 作用域：build_staff_groups
    // 本行目的：初始化分组列表。
    let mut groups = Vec::new();

    // 本行目的：按预定义顺序生成分组。
    for role in ordered_roles {
        // 本行目的：如果分组存在则加入输出。
        if let Some(people) = grouped.get(role) {
            // 本行目的：记录该职务已被使用。
            used.insert(role.to_string());
            // 本行目的：追加该职务分组到列表。
            groups.push(StaffGroupResponse {
                // 本行目的：写入职务名称。
                role: role.to_string(),
                // 本行目的：写入人员列表副本。
                people: people.clone(),
            // 本行目的：结束分组构建。
            });
        // 本行目的：结束分组存在判断。
        }
    // 本行目的：结束预排序遍历。
    }

    // 变量：remaining | 含义：未在预定义列表中的分组 | 类型：Vec<(String, Vec<StaffPersonResponse>)> | 作用域：build_staff_groups
    // 本行目的：筛选出剩余分组并收集为列表。
    let mut remaining: Vec<_> = grouped.into_iter().filter(|(role, _)| !used.contains(role)).collect();

    // 本行目的：按职务名称排序剩余分组。
    remaining.sort_by(|a, b| a.0.cmp(&b.0));

    // 本行目的：追加剩余分组到结果列表。
    for (role, people) in remaining {
        // 本行目的：追加分组到输出列表。
        groups.push(StaffGroupResponse {
            // 本行目的：写入职务名称。
            role,
            // 本行目的：写入人员列表。
            people,
        // 本行目的：结束分组构建。
        });
    // 本行目的：结束剩余分组遍历。
    }

    // 本行目的：返回最终分组列表。
    groups
// 本行目的：结束制作人员分组函数。
}

/// 函数：fetch_subject_characters | 输入：HTTP 客户端与条目 ID | 输出：角色列表 | 可能失败：网络或解析失败
// 本行目的：请求条目角色信息。
pub(crate) async fn fetch_subject_characters(client: &Client, id: u32) -> Result<Vec<SubjectCharacterItem>, String> {
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：fetch_subject_characters
    // 本行目的：请求条目角色接口。
    let response = client.get(format!("{API_BASE}{SUBJECTS_PATH}/{id}/characters")).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查请求是否成功。
    if !response.status().is_success() {
        // 本行目的：返回包含状态码的错误信息。
        return Err(format!("Bangumi API 请求失败: {}", response.status()));
    // 本行目的：结束状态检查。
    }

    // 变量：payload | 含义：角色响应载荷 | 类型：SubjectCharacterPayload | 作用域：fetch_subject_characters
    // 本行目的：解析响应 JSON 为载荷。
    let payload: SubjectCharacterPayload = response.json().await.map_err(|e| e.to_string())?;
    // 变量：items | 含义：角色列表 | 类型：Vec<SubjectCharacterItem> | 作用域：fetch_subject_characters
    // 本行目的：根据载荷类型提取角色列表。
    let items = match payload {
        // 本行目的：处理列表载荷。
        SubjectCharacterPayload::List(list) => list,
        // 本行目的：处理分页载荷并取 data 字段。
        SubjectCharacterPayload::Page {
            // 本行目的：提取 data 字段。
            data, ..
        } => data,
    // 本行目的：结束载荷匹配。
    };

    // 本行目的：返回角色列表。
    Ok(items)
// 本行目的：结束角色请求函数。
}

/// 函数：map_subject | 输入：条目模型 | 输出：季度动画模型 | 可能失败：无
// 本行目的：将条目模型映射为季度动画输出。
pub(crate) fn map_subject(subject: Subject) -> SeasonAnime {
    // 变量：image | 含义：解析后的图片 URL | 类型：String | 作用域：map_subject
    // 本行目的：解析条目图片 URL。
    let image = resolve_image(subject.images);
    // 变量：summary | 含义：简介文本 | 类型：String | 作用域：map_subject
    // 本行目的：提取简介或默认空字符串。
    let summary = subject.summary.unwrap_or_default();
    // 变量：date | 含义：日期文本 | 类型：String | 作用域：map_subject
    // 本行目的：提取日期或默认空字符串。
    let date = subject.date.unwrap_or_default();
    // 变量：rating | 含义：评分值 | 类型：Option<f64> | 作用域：map_subject
    // 本行目的：提取评分数值。
    let rating = subject.rating.and_then(|value| value.score);
    // 变量：url | 含义：条目详情 URL | 类型：String | 作用域：map_subject
    // 本行目的：构建条目详情链接。
    let url = format!("https://bgm.tv/subject/{}", subject.id);

    // 本行目的：构建季度动画输出结构体。
    SeasonAnime {
        // 本行目的：写入条目 ID。
        id: subject.id,
        // 本行目的：写入原始名称。
        name: subject.name,
        // 本行目的：写入中文名称。
        name_cn: subject.name_cn,
        // 本行目的：写入图片 URL。
        image,
        // 本行目的：写入日期。
        date,
        // 本行目的：写入评分。
        rating,
        // 本行目的：写入简介。
        summary,
        // 本行目的：写入详情链接。
        url,
    // 本行目的：结束结构体构建。
    }
// 本行目的：结束条目映射函数。
}

/// 函数：get_subject_brief | 输入：条目 ID | 输出：简要响应 | 可能失败：网络或解析失败
// 本行目的：获取条目简要信息。
pub(crate) async fn get_subject_brief(id: u32) -> Result<SubjectBriefResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_brief
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：get_subject_brief
    // 本行目的：请求条目详情接口。
    let response = client.get(format!("{API_BASE}{SUBJECTS_PATH}/{id}")).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查请求是否成功。
    if !response.status().is_success() {
        // 本行目的：返回包含状态码的错误信息。
        return Err(format!("Bangumi API 请求失败: {}", response.status()));
    // 本行目的：结束状态检查。
    }

    // 变量：subject | 含义：条目模型 | 类型：Subject | 作用域：get_subject_brief
    // 本行目的：解析响应 JSON 为条目模型。
    let subject: Subject = response.json().await.map_err(|e| e.to_string())?;
    // 变量：mapped | 含义：映射后的季度动画信息 | 类型：SeasonAnime | 作用域：get_subject_brief
    // 本行目的：将条目模型映射为输出模型。
    let mapped = map_subject(subject);

    // 本行目的：构建并返回简要响应。
    Ok(SubjectBriefResponse {
        // 本行目的：写入条目 ID。
        id: mapped.id,
        // 本行目的：写入原始名称。
        name: mapped.name,
        // 本行目的：写入中文名称。
        name_cn: mapped.name_cn,
        // 本行目的：写入图片 URL。
        image: mapped.image,
        // 本行目的：写入日期。
        date: mapped.date,
        // 本行目的：写入评分。
        rating: mapped.rating,
        // 本行目的：写入简介。
        summary: mapped.summary,
        // 本行目的：写入详情链接。
        url: mapped.url,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束简要信息函数。
}

/// 函数：fetch_month_subjects | 输入：HTTP 客户端、年份、月份 | 输出：季度动画列表 | 可能失败：网络或解析失败
// 本行目的：分页获取指定年月的条目列表。
pub(crate) async fn fetch_month_subjects(client: &Client, year: u32, month: u32) -> Result<Vec<SeasonAnime>, String> {
    // 变量：offset | 含义：分页偏移量 | 类型：u32 | 作用域：fetch_month_subjects
    // 本行目的：初始化分页偏移量。
    let mut offset = 0u32;
    // 变量：limit | 含义：分页大小 | 类型：u32 | 作用域：fetch_month_subjects
    // 本行目的：设置每页大小。
    let limit = 50u32;
    // 变量：items | 含义：累计条目列表 | 类型：Vec<SeasonAnime> | 作用域：fetch_month_subjects
    // 本行目的：初始化条目列表。
    let mut items = Vec::new();

    // 本行目的：循环分页获取数据。
    loop {
        // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：fetch_month_subjects
        // 本行目的：构建并发送分页请求。
        let response = client
            // 本行目的：设置请求地址。
            .get(format!("{API_BASE}{SUBJECTS_PATH}"))
            // 本行目的：设置查询参数。
            .query(&[
                // 本行目的：设置条目类型为动画。
                ("type", "2"),
                // 本行目的：设置年份参数。
                ("year", &year.to_string()),
                // 本行目的：设置月份参数。
                ("month", &month.to_string()),
                // 本行目的：设置分页大小。
                ("limit", &limit.to_string()),
                // 本行目的：设置分页偏移量。
                ("offset", &offset.to_string()),
            // 本行目的：结束查询参数数组。
            ])
            // 本行目的：发送请求。
            .send()
            // 本行目的：等待响应。
            .await
            // 本行目的：转换请求错误为字符串。
            .map_err(|e| e.to_string())?;

        // 本行目的：检查请求是否成功。
        if !response.status().is_success() {
            // 本行目的：返回包含状态码的错误信息。
            return Err(format!("Bangumi API 请求失败: {}", response.status()));
        // 本行目的：结束状态检查。
        }

        // 变量：payload | 含义：分页载荷 | 类型：PagedSubject | 作用域：fetch_month_subjects
        // 本行目的：解析响应 JSON 为分页载荷。
        let payload: PagedSubject = response.json().await.map_err(|e| e.to_string())?;

        // 本行目的：如果当前页没有数据则退出循环。
        if payload.data.is_empty() {
            // 本行目的：跳出分页循环。
            break;
        // 本行目的：结束空数据判断。
        }

        // 本行目的：将当前页数据映射并追加到列表。
        items.extend(payload.data.into_iter().map(map_subject));
        // 本行目的：更新分页偏移量。
        offset += payload.limit.max(limit);

        // 本行目的：如果已达到总数则退出循环。
        if offset >= payload.total {
            // 本行目的：跳出分页循环。
            break;
        // 本行目的：结束总数判断。
        }
    // 本行目的：结束分页循环。
    }

    // 本行目的：返回累计的条目列表。
    Ok(items)
// 本行目的：结束月份条目获取函数。
}

/// 函数：get_subject_origin_impl | 输入：条目 ID | 输出：原作响应 | 可能失败：网络或解析失败
// 本行目的：获取条目原作信息。
pub(crate) async fn get_subject_origin_impl(id: u32) -> Result<SubjectOriginResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_origin_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：get_subject_origin_impl
    // 本行目的：请求条目详情接口。
    let response = client.get(format!("{API_BASE}{SUBJECTS_PATH}/{id}")).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查请求是否成功。
    if !response.status().is_success() {
        // 本行目的：返回包含状态码的错误信息。
        return Err(format!("Bangumi API 请求失败: {}", response.status()));
    // 本行目的：结束状态检查。
    }

    // 变量：payload | 含义：条目详情载荷 | 类型：SubjectDetail | 作用域：get_subject_origin_impl
    // 本行目的：解析响应 JSON 为条目详情。
    let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
    // 变量：origin | 含义：原作信息 | 类型：Option<String> | 作用域：get_subject_origin_impl
    // 本行目的：从 infobox 中提取原作信息。
    let origin = extract_origin(payload.infobox);

    // 本行目的：构建并返回原作响应。
    Ok(SubjectOriginResponse {
        // 本行目的：写入条目 ID。
        id,
        // 本行目的：写入原作信息。
        origin,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束原作获取函数。
}

/// 函数：get_subject_aired_count_impl | 输入：条目 ID | 输出：已播统计响应 | 可能失败：网络或解析失败
// 本行目的：统计条目已播集数。
pub(crate) async fn get_subject_aired_count_impl(id: u32) -> Result<SubjectAiredResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_aired_count_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：offset | 含义：分页偏移量 | 类型：u32 | 作用域：get_subject_aired_count_impl
    // 本行目的：初始化分页偏移量。
    let mut offset = 0u32;
    // 变量：limit | 含义：分页大小 | 类型：u32 | 作用域：get_subject_aired_count_impl
    // 本行目的：设置每页大小。
    let limit = 50u32;
    // 变量：aired_count | 含义：已播集数 | 类型：u32 | 作用域：get_subject_aired_count_impl
    // 本行目的：初始化已播计数。
    let mut aired_count = 0u32;
    // 变量：total_count | 含义：总集数 | 类型：u32 | 作用域：get_subject_aired_count_impl
    // 本行目的：初始化总集数。
    let mut total_count = 0u32;
    // 变量：today | 含义：当前日期 | 类型：NaiveDate | 作用域：get_subject_aired_count_impl
    // 本行目的：获取当前日期用于比较。
    let today = Utc::now().date_naive();

    // 本行目的：循环分页获取剧集并统计。
    loop {
        // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：get_subject_aired_count_impl
        // 本行目的：构建并发送剧集分页请求。
        let response = client
            // 本行目的：设置请求地址。
            .get(format!("{API_BASE}{EPISODES_PATH}"))
            // 本行目的：设置查询参数。
            .query(&[("subject_id", &id.to_string()), ("limit", &limit.to_string()), ("offset", &offset.to_string())])
            // 本行目的：发送请求。
            .send()
            // 本行目的：等待响应。
            .await
            // 本行目的：转换请求错误为字符串。
            .map_err(|e| e.to_string())?;

        // 本行目的：检查请求是否成功。
        if !response.status().is_success() {
            // 本行目的：返回包含状态码的错误信息。
            return Err(format!("Bangumi API 请求失败: {}", response.status()));
        // 本行目的：结束状态检查。
        }

        // 变量：payload | 含义：剧集分页载荷 | 类型：EpisodePage | 作用域：get_subject_aired_count_impl
        // 本行目的：解析响应 JSON 为分页载荷。
        let payload: EpisodePage = response.json().await.map_err(|e| e.to_string())?;

        // 本行目的：首次获取时记录总集数。
        if total_count == 0 {
            // 本行目的：设置总集数。
            total_count = payload.total;
        // 本行目的：结束总数初始化。
        }

        // 本行目的：遍历当前页剧集并统计已播。
        for episode in payload.data.iter() {
            // 本行目的：如果已播则增加计数。
            if is_aired(episode, today) {
                // 本行目的：已播计数加一。
                aired_count += 1;
            // 本行目的：结束已播判断。
            }
        // 本行目的：结束剧集遍历。
        }

        // 本行目的：若当前页无数据则结束循环。
        if payload.data.is_empty() {
            // 本行目的：跳出分页循环。
            break;
        // 本行目的：结束空数据判断。
        }

        // 本行目的：更新分页偏移量。
        offset += payload.limit.max(limit);

        // 本行目的：若已超过总数则结束循环。
        if offset >= payload.total {
            // 本行目的：跳出分页循环。
            break;
        // 本行目的：结束总数判断。
        }
    // 本行目的：结束分页循环。
    }

    // 本行目的：构建并返回已播统计响应。
    Ok(SubjectAiredResponse {
        // 本行目的：写入条目 ID。
        id,
        // 本行目的：写入已播集数。
        aired_count,
        // 本行目的：写入总集数。
        total_count,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束已播统计函数。
}

/// 函数：get_subject_filters_impl | 输入：条目 ID | 输出：筛选响应 | 可能失败：网络或解析失败
// 本行目的：计算条目的类型、地区和受众筛选标签。
pub(crate) async fn get_subject_filters_impl(id: u32) -> Result<SubjectFiltersResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_filters_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：get_subject_filters_impl
    // 本行目的：请求条目详情接口。
    let response = client.get(format!("{API_BASE}{SUBJECTS_PATH}/{id}")).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查请求是否成功。
    if !response.status().is_success() {
        // 本行目的：返回包含状态码的错误信息。
        return Err(format!("Bangumi API 请求失败: {}", response.status()));
    // 本行目的：结束状态检查。
    }

    // 变量：payload | 含义：条目详情载荷 | 类型：SubjectDetail | 作用域：get_subject_filters_impl
    // 本行目的：解析响应 JSON 为条目详情。
    let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
    // 变量：(info_types, info_regions, info_audiences) | 含义：infobox 提取的筛选项 | 类型：(Vec<String>, Vec<String>, Vec<String>) | 作用域：get_subject_filters_impl
    // 本行目的：从 infobox 提取筛选分组。
    let (info_types, info_regions, info_audiences) = extract_filter_groups(payload.infobox);
    // 变量：tags | 含义：合并后的标签列表 | 类型：Vec<String> | 作用域：get_subject_filters_impl
    // 本行目的：收集 tags 与 meta_tags。
    let mut tags = collect_subject_tags(payload.tags, payload.meta_tags);

    // 本行目的：合并 infobox 类型标签。
    tags.extend(info_types.clone());
    // 本行目的：合并 infobox 地区标签。
    tags.extend(info_regions.clone());
    // 本行目的：合并 infobox 受众标签。
    tags.extend(info_audiences.clone());

    // 变量：OFFICIAL_TYPES | 含义：官方类型列表 | 类型：&[&str] | 作用域：模块内
    // 本行目的：定义官方类型标签清单。
    const OFFICIAL_TYPES: &[&str] = &[
        // 本行目的：类型标签。
        "科幻",
        // 本行目的：类型标签。
        "喜剧",
        // 本行目的：类型标签。
        "同人",
        // 本行目的：类型标签。
        "百合",
        // 本行目的：类型标签。
        "校园",
        // 本行目的：类型标签。
        "惊悚",
        // 本行目的：类型标签。
        "后宫",
        // 本行目的：类型标签。
        "机战",
        // 本行目的：类型标签。
        "悬疑",
        // 本行目的：类型标签。
        "恋爱",
        // 本行目的：类型标签。
        "奇幻",
        // 本行目的：类型标签。
        "推理",
        // 本行目的：类型标签。
        "运动",
        // 本行目的：类型标签。
        "耽美",
        // 本行目的：类型标签。
        "音乐",
        // 本行目的：类型标签。
        "战斗",
        // 本行目的：类型标签。
        "冒险",
        // 本行目的：类型标签。
        "萌系",
        // 本行目的：类型标签。
        "穿越",
        // 本行目的：类型标签。
        "玄幻",
        // 本行目的：类型标签。
        "乙女",
        // 本行目的：类型标签。
        "恐怖",
        // 本行目的：类型标签。
        "历史",
        // 本行目的：类型标签。
        "日常",
        // 本行目的：类型标签。
        "剧情",
        // 本行目的：类型标签。
        "武侠",
        // 本行目的：类型标签。
        "美食",
        // 本行目的：类型标签。
        "职场",
    // 本行目的：结束官方类型列表。
    ];

    // 变量：OFFICIAL_REGIONS | 含义：官方地区列表 | 类型：&[&str] | 作用域：模块内
    // 本行目的：定义官方地区标签清单。
    const OFFICIAL_REGIONS: &[&str] = &[
        // 本行目的：地区标签。
        "日本",
        // 本行目的：地区标签。
        "欧美",
        // 本行目的：地区标签。
        "中国",
        // 本行目的：地区标签。
        "美国",
        // 本行目的：地区标签。
        "韩国",
        // 本行目的：地区标签。
        "法国",
        // 本行目的：地区标签。
        "中国香港",
        // 本行目的：地区标签。
        "英国",
        // 本行目的：地区标签。
        "俄罗斯",
        // 本行目的：地区标签。
        "苏联",
        // 本行目的：地区标签。
        "捷克",
        // 本行目的：地区标签。
        "中国台湾",
        // 本行目的：地区标签。
        "马来西亚",
    // 本行目的：结束官方地区列表。
    ];

    // 变量：OFFICIAL_AUDIENCES | 含义：官方受众列表 | 类型：&[&str] | 作用域：模块内
    // 本行目的：定义官方受众标签清单。
    const OFFICIAL_AUDIENCES: &[&str] = &[
        // 本行目的：受众标签。
        "BL",
        // 本行目的：受众标签。
        "GL",
        // 本行目的：受众标签。
        "子供向",
        // 本行目的：受众标签。
        "女性向",
        // 本行目的：受众标签。
        "少女向",
        // 本行目的：受众标签。
        "少年向",
        // 本行目的：受众标签。
        "青年向",
    // 本行目的：结束官方受众列表。
    ];

    // 变量：TYPE_ALIASES | 含义：类型别名映射 | 类型：&[(&str, &str)] | 作用域：模块内
    // 本行目的：定义类型别名映射。
    const TYPE_ALIASES: &[(&str, &str)] = &[
        // 本行目的：将“搞笑”映射为“喜剧”。
        ("搞笑", "喜剧"),
        // 本行目的：将“恋愛”映射为“恋爱”。
        ("恋愛", "恋爱"),
        // 本行目的：将“日常系”映射为“日常”。
        ("日常系", "日常"),
        // 本行目的：将“熱血”映射为“战斗”。
        ("熱血", "战斗"),
        // 本行目的：将“机甲”映射为“机战”。
        ("机甲", "机战"),
        // 本行目的：将“萌”映射为“萌系”。
        ("萌", "萌系"),
    // 本行目的：结束类型别名映射。
    ];

    // 变量：REGION_ALIASES | 含义：地区别名映射 | 类型：&[(&str, &str)] | 作用域：模块内
    // 本行目的：定义地区别名映射。
    const REGION_ALIASES: &[(&str, &str)] = &[
        // 本行目的：将“大陆”映射为“中国”。
        ("大陆", "中国"),
        // 本行目的：将“中国大陆”映射为“中国”。
        ("中国大陆", "中国"),
        // 本行目的：将“香港”映射为“中国香港”。
        ("香港", "中国香港"),
        // 本行目的：将“台湾”映射为“中国台湾”。
        ("台湾", "中国台湾"),
        // 本行目的：将“欧美动画”映射为“欧美”。
        ("欧美动画", "欧美"),
        // 本行目的：将“欧洲”映射为“欧美”。
        ("欧洲", "欧美"),
        // 本行目的：将“俄国”映射为“俄罗斯”。
        ("俄国", "俄罗斯"),
    // 本行目的：结束地区别名映射。
    ];

    // 变量：AUDIENCE_ALIASES | 含义：受众别名映射 | 类型：&[(&str, &str)] | 作用域：模块内
    // 本行目的：定义受众别名映射。
    const AUDIENCE_ALIASES: &[(&str, &str)] = &[
        // 本行目的：将“少年”映射为“少年向”。
        ("少年", "少年向"),
        // 本行目的：将“少女”映射为“少女向”。
        ("少女", "少女向"),
        // 本行目的：将“青年”映射为“青年向”。
        ("青年", "青年向"),
        // 本行目的：将“儿童”映射为“子供向”。
        ("儿童", "子供向"),
        // 本行目的：将“儿童向”映射为“子供向”。
        ("儿童向", "子供向"),
        // 本行目的：将“女性向”映射为“女性向”。
        ("女性向", "女性向"),
        // 本行目的：将“男性向”映射为“男性向”。
        ("男性向", "男性向"),
        // 本行目的：将“男向”映射为“男性向”。
        ("男向", "男性向"),
        // 本行目的：将“女向”映射为“女性向”。
        ("女向", "女性向"),
        // 本行目的：将“百合”映射为“GL”。
        ("百合", "GL"),
        // 本行目的：将“耽美”映射为“BL”。
        ("耽美", "BL"),
        // 本行目的：将“腐向”映射为“BL”。
        ("腐向", "BL"),
    // 本行目的：结束受众别名映射。
    ];

    // 变量：types | 含义：官方类型列表 | 类型：Vec<String> | 作用域：get_subject_filters_impl
    // 本行目的：将标签映射为官方类型。
    let types = map_tags_to_official(&tags, OFFICIAL_TYPES, TYPE_ALIASES);
    // 变量：regions | 含义：官方地区列表 | 类型：Vec<String> | 作用域：get_subject_filters_impl
    // 本行目的：将标签映射为官方地区。
    let regions = map_tags_to_official(&tags, OFFICIAL_REGIONS, REGION_ALIASES);
    // 变量：audiences | 含义：官方受众列表 | 类型：Vec<String> | 作用域：get_subject_filters_impl
    // 本行目的：将标签映射为官方受众。
    let audiences = map_tags_to_official(&tags, OFFICIAL_AUDIENCES, AUDIENCE_ALIASES);

    // 本行目的：构建并返回筛选响应。
    Ok(SubjectFiltersResponse {
        // 本行目的：写入条目 ID。
        id,
        // 本行目的：写入类型列表。
        types,
        // 本行目的：写入地区列表。
        regions,
        // 本行目的：写入受众列表。
        audiences,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束筛选响应函数。
}

/// 函数：get_subject_aliases_impl | 输入：条目 ID | 输出：别名响应 | 可能失败：网络或解析失败
// 本行目的：获取条目别名列表。
pub(crate) async fn get_subject_aliases_impl(id: u32) -> Result<SubjectAliasesResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_aliases_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：response | 含义：HTTP 响应 | 类型：reqwest::Response | 作用域：get_subject_aliases_impl
    // 本行目的：请求条目详情接口。
    let response = client.get(format!("{API_BASE}{SUBJECTS_PATH}/{id}")).send().await.map_err(|e| e.to_string())?;

    // 本行目的：检查请求是否成功。
    if !response.status().is_success() {
        // 本行目的：返回包含状态码的错误信息。
        return Err(format!("Bangumi API 请求失败: {}", response.status()));
    // 本行目的：结束状态检查。
    }

    // 变量：payload | 含义：条目详情载荷 | 类型：SubjectDetail | 作用域：get_subject_aliases_impl
    // 本行目的：解析响应 JSON 为条目详情。
    let payload: SubjectDetail = response.json().await.map_err(|e| e.to_string())?;
    // 变量：aliases | 含义：别名列表 | 类型：Vec<String> | 作用域：get_subject_aliases_impl
    // 本行目的：从 infobox 中提取别名。
    let aliases = extract_aliases(payload.infobox);

    // 本行目的：构建并返回别名响应。
    Ok(SubjectAliasesResponse {
        // 本行目的：写入条目 ID。
        id,
        // 本行目的：写入别名列表。
        aliases,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束别名响应函数。
}

/// 函数：get_subject_staff_impl | 输入：条目 ID | 输出：制作人员响应 | 可能失败：网络或解析失败
// 本行目的：获取条目制作人员分组信息。
pub(crate) async fn get_subject_staff_impl(id: u32) -> Result<SubjectStaffResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_staff_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：persons | 含义：人物列表 | 类型：Vec<SubjectPersonItem> | 作用域：get_subject_staff_impl
    // 本行目的：请求人物列表。
    let persons = fetch_subject_persons(&client, id).await?;
    // 变量：groups | 含义：人员分组列表 | 类型：Vec<StaffGroupResponse> | 作用域：get_subject_staff_impl
    // 本行目的：按职务分组人员列表。
    let groups = build_staff_groups(persons);

    // 本行目的：构建并返回制作人员响应。
    Ok(SubjectStaffResponse {
        // 本行目的：写入条目 ID。
        id,
        // 本行目的：写入人员分组列表。
        groups,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束制作人员响应函数。
}

/// 函数：get_subject_characters_impl | 输入：条目 ID | 输出：角色响应 | 可能失败：网络或解析失败
// 本行目的：获取条目角色列表并映射为链接形式。
pub(crate) async fn get_subject_characters_impl(id: u32) -> Result<SubjectCharactersResponse, String> {
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_subject_characters_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：characters | 含义：角色列表 | 类型：Vec<SubjectCharacterItem> | 作用域：get_subject_characters_impl
    // 本行目的：请求角色列表。
    let characters = fetch_subject_characters(&client, id).await?;
    // 变量：mapped | 含义：角色链接列表 | 类型：Vec<CharacterLinkResponse> | 作用域：get_subject_characters_impl
    // 本行目的：将角色列表映射为响应结构。
    let mapped = characters
        // 本行目的：将角色迭代器转为迭代。
        .into_iter()
        // 本行目的：映射每个角色条目。
        .map(|item| {
            // 本行目的：构建角色链接响应。
            CharacterLinkResponse {
                // 本行目的：写入角色 ID。
                id: item.id,
                // 本行目的：优先使用中文名，否则使用原名。
                name: item.name_cn.clone().unwrap_or(item.name),
                // 本行目的：构建角色详情链接。
                url: format!("https://bgm.tv/character/{}", item.id),
                // 本行目的：写入角色关系。
                relation: item.relation,
            // 本行目的：结束角色响应构建。
            }
        // 本行目的：结束映射闭包。
        })
        // 本行目的：收集为向量。
        .collect();

    // 本行目的：构建并返回角色响应。
    Ok(SubjectCharactersResponse {
        // 本行目的：写入条目 ID。
        id,
        // 本行目的：写入角色列表。
        characters: mapped,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束角色响应函数。
}

/// 函数：get_season_subjects_impl | 输入：年份与季节 | 输出：季度响应 | 可能失败：网络或解析失败
// 本行目的：按季度汇总月份番剧列表。
pub(crate) async fn get_season_subjects_impl(year: u32, season: String) -> Result<SeasonResponse, String> {
    // 变量：months | 含义：季度月份列表 | 类型：Vec<u32> | 作用域：get_season_subjects_impl
    // 本行目的：将季节转换为月份列表。
    let months = season_months(&season)?;
    // 变量：client | 含义：HTTP 客户端 | 类型：Client | 作用域：get_season_subjects_impl
    // 本行目的：构建带 UA 的 HTTP 客户端。
    let client = reqwest::Client::builder().user_agent("HanamiRIP-CN/0.1").build().map_err(|e| e.to_string())?;
    // 变量：month_payloads | 含义：月份响应列表 | 类型：Vec<SeasonMonth> | 作用域：get_season_subjects_impl
    // 本行目的：初始化月份响应列表。
    let mut month_payloads = Vec::new();

    // 本行目的：遍历季度内的每个月份。
    for month in months {
        // 变量：list | 含义：当前月份的条目列表 | 类型：Vec<SeasonAnime> | 作用域：get_season_subjects_impl
        // 本行目的：请求当前月份的条目列表。
        let list = fetch_month_subjects(&client, year, month).await?;
        // 变量：count | 含义：条目数量 | 类型：usize | 作用域：get_season_subjects_impl
        // 本行目的：统计当前月份条目数量。
        let count = list.len();

        // 本行目的：追加当前月份的响应数据。
        month_payloads.push(SeasonMonth {
            // 本行目的：写入年份。
            year,
            // 本行目的：写入月份。
            month,
            // 本行目的：写入条目数量。
            count,
            // 本行目的：写入条目列表。
            list,
        // 本行目的：结束月份结构体构建。
        });
    // 本行目的：结束月份遍历。
    }

    // 本行目的：构建并返回季度响应。
    Ok(SeasonResponse {
        // 本行目的：写入年份。
        year,
        // 本行目的：写入季节字符串。
        season,
        // 本行目的：写入抓取时间。
        fetched_at: Utc::now().to_rfc3339(),
        // 本行目的：写入数据源地址。
        source: format!("{API_BASE}{SUBJECTS_PATH}"),
        // 本行目的：写入月份列表。
        months: month_payloads,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束季度响应函数。
}
