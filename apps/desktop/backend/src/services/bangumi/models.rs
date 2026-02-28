/* 文件：models.rs | 用途：定义 Bangumi API 交互的数据结构 | 关键对象：Subject、SeasonAnime、各类 Response */
// 本行目的：引入反序列化支持。
use serde::Deserialize;
// 本行目的：引入序列化支持。
use serde::Serialize;

// 本行目的：引入 JSON 任意值类型。
use serde_json::Value;

// 本行目的：为分页条目响应结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义分页条目响应结构体。
pub(crate) struct PagedSubject {
    // 变量：total | 含义：总条目数 | 类型：u32 | 作用域：PagedSubject
    // 本行目的：记录总条目数量。
    pub total: u32,
    // 变量：limit | 含义：每页条目数量 | 类型：u32 | 作用域：PagedSubject
    // 本行目的：记录分页大小。
    pub limit: u32,
    // 变量：data | 含义：条目列表 | 类型：Vec<Subject> | 作用域：PagedSubject
    // 本行目的：承载当前页条目数据。
    pub data: Vec<Subject>,
// 本行目的：结束分页条目结构体定义。
}

// 本行目的：为条目结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义条目基础信息结构体。
pub(crate) struct Subject {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：Subject
    // 本行目的：记录条目唯一标识。
    pub id: u32,
    // 变量：name | 含义：原始名称 | 类型：String | 作用域：Subject
    // 本行目的：记录条目原名。
    pub name: String,
    // 变量：name_cn | 含义：中文名称 | 类型：String | 作用域：Subject
    // 本行目的：记录条目中文名。
    pub name_cn: String,
    // 变量：date | 含义：放送日期 | 类型：Option<String> | 作用域：Subject
    // 本行目的：记录条目日期信息。
    pub date: Option<String>,
    // 变量：images | 含义：图片资源信息 | 类型：Option<Images> | 作用域：Subject
    // 本行目的：记录条目图片集合。
    pub images: Option<Images>,
    // 变量：summary | 含义：条目简介 | 类型：Option<String> | 作用域：Subject
    // 本行目的：记录条目简介文本。
    pub summary: Option<String>,
    // 变量：rating | 含义：评分信息 | 类型：Option<Rating> | 作用域：Subject
    // 本行目的：记录条目评分信息。
    pub rating: Option<Rating>,
// 本行目的：结束条目结构体定义。
}

// 本行目的：为图片结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义图片资源结构体。
pub(crate) struct Images {
    // 变量：common | 含义：常用尺寸图片 URL | 类型：Option<String> | 作用域：Images
    // 本行目的：记录常用尺寸图片地址。
    pub common: Option<String>,
    // 变量：medium | 含义：中等尺寸图片 URL | 类型：Option<String> | 作用域：Images
    // 本行目的：记录中等尺寸图片地址。
    pub medium: Option<String>,
    // 变量：large | 含义：大尺寸图片 URL | 类型：Option<String> | 作用域：Images
    // 本行目的：记录大尺寸图片地址。
    pub large: Option<String>,
// 本行目的：结束图片结构体定义。
}

// 本行目的：为评分结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义评分结构体。
pub(crate) struct Rating {
    // 变量：score | 含义：评分数值 | 类型：Option<f64> | 作用域：Rating
    // 本行目的：记录评分。
    pub score: Option<f64>,
// 本行目的：结束评分结构体定义。
}

// 本行目的：为剧集分页结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义剧集分页响应结构体。
pub(crate) struct EpisodePage {
    // 变量：total | 含义：总剧集数 | 类型：u32 | 作用域：EpisodePage
    // 本行目的：记录总剧集数。
    pub total: u32,
    // 变量：limit | 含义：每页剧集数 | 类型：u32 | 作用域：EpisodePage
    // 本行目的：记录分页大小。
    pub limit: u32,
    // 变量：data | 含义：剧集列表 | 类型：Vec<Episode> | 作用域：EpisodePage
    // 本行目的：记录当前页剧集数据。
    pub data: Vec<Episode>,
// 本行目的：结束剧集分页结构体定义。
}

// 本行目的：为剧集结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义剧集信息结构体。
pub(crate) struct Episode {
    // 变量：airdate | 含义：播放日期 | 类型：Option<String> | 作用域：Episode
    // 本行目的：记录剧集播出日期。
    pub airdate: Option<String>,
    // 变量：status | 含义：播出状态 | 类型：Option<String> | 作用域：Episode
    // 本行目的：记录剧集状态字段。
    pub status: Option<String>,
// 本行目的：结束剧集结构体定义。
}

// 本行目的：为条目详情结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义条目详情结构体。
pub(crate) struct SubjectDetail {
    // 变量：summary | 含义：条目简介 | 类型：Option<String> | 作用域：SubjectDetail
    // 本行目的：记录条目简介文本。
    pub summary: Option<String>,
    // 变量：infobox | 含义：条目 infobox 列表 | 类型：Option<Vec<InfoboxItem>> | 作用域：SubjectDetail
    // 本行目的：记录 infobox 条目列表。
    pub infobox: Option<Vec<InfoboxItem>>,
    // 变量：tags | 含义：标签列表 | 类型：Option<Vec<SubjectTag>> | 作用域：SubjectDetail
    // 本行目的：记录条目标签列表。
    pub tags: Option<Vec<SubjectTag>>,
    // 变量：meta_tags | 含义：元标签列表 | 类型：Option<Vec<String>> | 作用域：SubjectDetail
    // 本行目的：记录元标签列表。
    pub meta_tags: Option<Vec<String>>,
// 本行目的：结束条目详情结构体定义。
}

// 本行目的：为标签结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义条目标签结构体。
pub(crate) struct SubjectTag {
    // 变量：name | 含义：标签名称 | 类型：String | 作用域：SubjectTag
    // 本行目的：记录标签名称。
    pub name: String,
// 本行目的：结束标签结构体定义。
}

// 本行目的：为 infobox 条目结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 infobox 条目结构体。
pub(crate) struct InfoboxItem {
    // 变量：key | 含义：条目键名 | 类型：String | 作用域：InfoboxItem
    // 本行目的：记录 infobox 条目键名。
    pub key: String,
    // 变量：value | 含义：条目值 | 类型：Value | 作用域：InfoboxItem
    // 本行目的：记录 infobox 条目值。
    pub value: Value,
// 本行目的：结束 infobox 条目结构体定义。
}

// 本行目的：为人物条目结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义人物条目结构体。
pub(crate) struct SubjectPersonItem {
    // 变量：id | 含义：人物 ID | 类型：u32 | 作用域：SubjectPersonItem
    // 本行目的：记录人物标识。
    pub id: u32,
    // 变量：name | 含义：人物名称 | 类型：String | 作用域：SubjectPersonItem
    // 本行目的：记录人物名称。
    pub name: String,
    // 变量：relation | 含义：人物关联关系 | 类型：Option<String> | 作用域：SubjectPersonItem
    // 本行目的：记录人物与条目关系。
    pub relation: Option<String>,
// 本行目的：结束人物条目结构体定义。
}

// 本行目的：为人物响应枚举启用反序列化。
#[derive(Deserialize)]
// 本行目的：使用 untagged 处理不同响应结构。
#[serde(untagged)]
// 本行目的：定义人物响应的枚举类型。
pub(crate) enum SubjectPersonPayload {
    // 本行目的：列表形式响应。
    List(Vec<SubjectPersonItem>),
    // 本行目的：分页形式响应。
    Page {
        // 变量：_total | 含义：总条目数 | 类型：u32 | 作用域：SubjectPersonPayload::Page
        // 本行目的：记录分页总数。
        _total: u32,
        // 变量：_limit | 含义：分页大小 | 类型：u32 | 作用域：SubjectPersonPayload::Page
        // 本行目的：记录分页大小。
        _limit: u32,
        // 变量：data | 含义：人物列表 | 类型：Vec<SubjectPersonItem> | 作用域：SubjectPersonPayload::Page
        // 本行目的：记录人物数据列表。
        data: Vec<SubjectPersonItem>,
    // 本行目的：结束 Page 变体定义。
    },
// 本行目的：结束人物响应枚举定义。
}

// 本行目的：为角色条目结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义角色条目结构体。
pub(crate) struct SubjectCharacterItem {
    // 变量：id | 含义：角色 ID | 类型：u32 | 作用域：SubjectCharacterItem
    // 本行目的：记录角色标识。
    pub id: u32,
    // 变量：name | 含义：角色名称 | 类型：String | 作用域：SubjectCharacterItem
    // 本行目的：记录角色名称。
    pub name: String,
    // 变量：name_cn | 含义：角色中文名 | 类型：Option<String> | 作用域：SubjectCharacterItem
    // 本行目的：记录角色中文名称。
    pub name_cn: Option<String>,
    // 变量：relation | 含义：角色关系 | 类型：Option<String> | 作用域：SubjectCharacterItem
    // 本行目的：记录角色与条目关系。
    pub relation: Option<String>,
// 本行目的：结束角色条目结构体定义。
}

// 本行目的：为角色响应枚举启用反序列化。
#[derive(Deserialize)]
// 本行目的：使用 untagged 处理不同响应结构。
#[serde(untagged)]
// 本行目的：定义角色响应的枚举类型。
pub(crate) enum SubjectCharacterPayload {
    // 本行目的：列表形式响应。
    List(Vec<SubjectCharacterItem>),
    // 本行目的：分页形式响应。
    Page {
        // 变量：_total | 含义：总条目数 | 类型：u32 | 作用域：SubjectCharacterPayload::Page
        // 本行目的：记录分页总数。
        _total: u32,
        // 变量：_limit | 含义：分页大小 | 类型：u32 | 作用域：SubjectCharacterPayload::Page
        // 本行目的：记录分页大小。
        _limit: u32,
        // 变量：data | 含义：角色列表 | 类型：Vec<SubjectCharacterItem> | 作用域：SubjectCharacterPayload::Page
        // 本行目的：记录角色数据列表。
        data: Vec<SubjectCharacterItem>,
    // 本行目的：结束 Page 变体定义。
    },
// 本行目的：结束角色响应枚举定义。
}

// 本行目的：为季度动画结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义季度动画输出结构体。
pub struct SeasonAnime {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SeasonAnime
    // 本行目的：记录条目标识。
    pub id: u32,
    // 变量：name | 含义：原始名称 | 类型：String | 作用域：SeasonAnime
    // 本行目的：记录原始名称。
    pub name: String,
    // 变量：name_cn | 含义：中文名称 | 类型：String | 作用域：SeasonAnime
    // 本行目的：记录中文名称。
    pub name_cn: String,
    // 变量：image | 含义：图片 URL | 类型：String | 作用域：SeasonAnime
    // 本行目的：记录图片地址。
    pub image: String,
    // 变量：date | 含义：日期 | 类型：String | 作用域：SeasonAnime
    // 本行目的：记录日期字符串。
    pub date: String,
    // 变量：rating | 含义：评分 | 类型：Option<f64> | 作用域：SeasonAnime
    // 本行目的：记录评分。
    pub rating: Option<f64>,
    // 变量：summary | 含义：简介 | 类型：String | 作用域：SeasonAnime
    // 本行目的：记录简介文本。
    pub summary: String,
    // 变量：url | 含义：详情页 URL | 类型：String | 作用域：SeasonAnime
    // 本行目的：记录条目链接。
    pub url: String,
// 本行目的：结束季度动画结构体定义。
}

// 本行目的：为季度月份结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义季度月份输出结构体。
pub struct SeasonMonth {
    // 变量：year | 含义：年份 | 类型：u32 | 作用域：SeasonMonth
    // 本行目的：记录年份。
    pub year: u32,
    // 变量：month | 含义：月份 | 类型：u32 | 作用域：SeasonMonth
    // 本行目的：记录月份。
    pub month: u32,
    // 变量：count | 含义：条目数量 | 类型：usize | 作用域：SeasonMonth
    // 本行目的：记录条目数量。
    pub count: usize,
    // 变量：list | 含义：动画列表 | 类型：Vec<SeasonAnime> | 作用域：SeasonMonth
    // 本行目的：记录动画列表。
    pub list: Vec<SeasonAnime>,
// 本行目的：结束季度月份结构体定义。
}

// 本行目的：为季度响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义季度响应结构体。
pub struct SeasonResponse {
    // 变量：year | 含义：年份 | 类型：u32 | 作用域：SeasonResponse
    // 本行目的：记录年份。
    pub year: u32,
    // 变量：season | 含义：季节字符串 | 类型：String | 作用域：SeasonResponse
    // 本行目的：记录季节名称。
    pub season: String,
    // 变量：fetched_at | 含义：抓取时间 | 类型：String | 作用域：SeasonResponse
    // 本行目的：记录抓取时间。
    pub fetched_at: String,
    // 变量：source | 含义：数据源 | 类型：String | 作用域：SeasonResponse
    // 本行目的：记录数据来源。
    pub source: String,
    // 变量：months | 含义：月份列表 | 类型：Vec<SeasonMonth> | 作用域：SeasonResponse
    // 本行目的：记录月份列表。
    pub months: Vec<SeasonMonth>,
// 本行目的：结束季度响应结构体定义。
}

// 本行目的：为原作响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义原作响应结构体。
pub struct SubjectOriginResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectOriginResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：origin | 含义：原作信息 | 类型：Option<String> | 作用域：SubjectOriginResponse
    // 本行目的：记录原作信息。
    pub origin: Option<String>,
// 本行目的：结束原作响应结构体定义。
}

// 本行目的：为已播统计响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义已播统计响应结构体。
pub struct SubjectAiredResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectAiredResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：aired_count | 含义：已播集数 | 类型：u32 | 作用域：SubjectAiredResponse
    // 本行目的：记录已播集数。
    pub aired_count: u32,
    // 变量：total_count | 含义：总集数 | 类型：u32 | 作用域：SubjectAiredResponse
    // 本行目的：记录总集数。
    pub total_count: u32,
// 本行目的：结束已播统计响应结构体定义。
}

// 本行目的：为制作人员结构启用序列化与克隆。
#[derive(Serialize, Clone)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义制作人员结构体。
pub struct StaffPersonResponse {
    // 变量：id | 含义：人物 ID | 类型：u32 | 作用域：StaffPersonResponse
    // 本行目的：记录人物 ID。
    pub id: u32,
    // 变量：name | 含义：人物名称 | 类型：String | 作用域：StaffPersonResponse
    // 本行目的：记录人物名称。
    pub name: String,
    // 变量：url | 含义：人物链接 | 类型：String | 作用域：StaffPersonResponse
    // 本行目的：记录人物链接。
    pub url: String,
// 本行目的：结束制作人员结构体定义。
}

// 本行目的：为制作人员分组结构启用序列化与克隆。
#[derive(Serialize, Clone)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义制作人员分组结构体。
pub struct StaffGroupResponse {
    // 变量：role | 含义：职务名称 | 类型：String | 作用域：StaffGroupResponse
    // 本行目的：记录职务名称。
    pub role: String,
    // 变量：people | 含义：人员列表 | 类型：Vec<StaffPersonResponse> | 作用域：StaffGroupResponse
    // 本行目的：记录人员列表。
    pub people: Vec<StaffPersonResponse>,
// 本行目的：结束制作人员分组结构体定义。
}

// 本行目的：为制作人员响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义制作人员响应结构体。
pub struct SubjectStaffResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectStaffResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：groups | 含义：分组列表 | 类型：Vec<StaffGroupResponse> | 作用域：SubjectStaffResponse
    // 本行目的：记录人员分组列表。
    pub groups: Vec<StaffGroupResponse>,
// 本行目的：结束制作人员响应结构体定义。
}

// 本行目的：为角色链接结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义角色链接结构体。
pub struct CharacterLinkResponse {
    // 变量：id | 含义：角色 ID | 类型：u32 | 作用域：CharacterLinkResponse
    // 本行目的：记录角色 ID。
    pub id: u32,
    // 变量：name | 含义：角色名称 | 类型：String | 作用域：CharacterLinkResponse
    // 本行目的：记录角色名称。
    pub name: String,
    // 变量：url | 含义：角色链接 | 类型：String | 作用域：CharacterLinkResponse
    // 本行目的：记录角色链接。
    pub url: String,
    // 变量：relation | 含义：角色关系 | 类型：Option<String> | 作用域：CharacterLinkResponse
    // 本行目的：记录角色关系。
    pub relation: Option<String>,
// 本行目的：结束角色链接结构体定义。
}

// 本行目的：为角色响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义角色响应结构体。
pub struct SubjectCharactersResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectCharactersResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：characters | 含义：角色列表 | 类型：Vec<CharacterLinkResponse> | 作用域：SubjectCharactersResponse
    // 本行目的：记录角色列表。
    pub characters: Vec<CharacterLinkResponse>,
// 本行目的：结束角色响应结构体定义。
}

// 本行目的：为别名响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义别名响应结构体。
pub struct SubjectAliasesResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectAliasesResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：aliases | 含义：别名列表 | 类型：Vec<String> | 作用域：SubjectAliasesResponse
    // 本行目的：记录别名列表。
    pub aliases: Vec<String>,
// 本行目的：结束别名响应结构体定义。
}

// 本行目的：为简介响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义简介响应结构体。
pub struct SubjectSummaryResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectSummaryResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：summary | 含义：简介文本 | 类型：String | 作用域：SubjectSummaryResponse
    // 本行目的：记录简介文本。
    pub summary: String,
    // 变量：translated | 含义：是否已翻译 | 类型：bool | 作用域：SubjectSummaryResponse
    // 本行目的：记录翻译状态。
    pub translated: bool,
    // 变量：error | 含义：错误信息 | 类型：Option<String> | 作用域：SubjectSummaryResponse
    // 本行目的：记录错误信息。
    pub error: Option<String>,
// 本行目的：结束简介响应结构体定义。
}

// 本行目的：为筛选响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义筛选响应结构体。
pub struct SubjectFiltersResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectFiltersResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：types | 含义：类型列表 | 类型：Vec<String> | 作用域：SubjectFiltersResponse
    // 本行目的：记录类型列表。
    pub types: Vec<String>,
    // 变量：regions | 含义：地区列表 | 类型：Vec<String> | 作用域：SubjectFiltersResponse
    // 本行目的：记录地区列表。
    pub regions: Vec<String>,
    // 变量：audiences | 含义：受众列表 | 类型：Vec<String> | 作用域：SubjectFiltersResponse
    // 本行目的：记录受众列表。
    pub audiences: Vec<String>,
// 本行目的：结束筛选响应结构体定义。
}

// 本行目的：为简要响应结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义简要响应结构体。
pub struct SubjectBriefResponse {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：SubjectBriefResponse
    // 本行目的：记录条目 ID。
    pub id: u32,
    // 变量：name | 含义：原始名称 | 类型：String | 作用域：SubjectBriefResponse
    // 本行目的：记录原始名称。
    pub name: String,
    // 变量：name_cn | 含义：中文名称 | 类型：String | 作用域：SubjectBriefResponse
    // 本行目的：记录中文名称。
    pub name_cn: String,
    // 变量：image | 含义：图片 URL | 类型：String | 作用域：SubjectBriefResponse
    // 本行目的：记录图片地址。
    pub image: String,
    // 变量：date | 含义：日期 | 类型：String | 作用域：SubjectBriefResponse
    // 本行目的：记录日期字符串。
    pub date: String,
    // 变量：rating | 含义：评分 | 类型：Option<f64> | 作用域：SubjectBriefResponse
    // 本行目的：记录评分。
    pub rating: Option<f64>,
    // 变量：summary | 含义：简介 | 类型：String | 作用域：SubjectBriefResponse
    // 本行目的：记录简介文本。
    pub summary: String,
    // 变量：url | 含义：详情页 URL | 类型：String | 作用域：SubjectBriefResponse
    // 本行目的：记录条目链接。
    pub url: String,
// 本行目的：结束简要响应结构体定义。
}
