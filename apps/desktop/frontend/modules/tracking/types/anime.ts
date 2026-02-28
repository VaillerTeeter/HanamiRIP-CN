/** 文件：anime.ts | 用途：定义追踪模块中动画条目、制作人员与季度数据的类型契约 | 关键对象：MonthAnime, StaffPerson, StaffGroup, CharacterLink, SeasonMonthData, SeasonResponse */
/** 类型：MonthAnime | 用途：描述月度动画条目的完整信息结构 | 字段：基础信息、统计信息、翻译状态与展示信息 */
// 本行目的：导出月度动画条目接口。
export interface MonthAnime {
    // 本行目的：定义条目唯一 ID。
    id: number;
    // 本行目的：定义条目原始名称。
    name: string;
    // 本行目的：定义中文名称（可选）。
    nameCn?: string;
    // 本行目的：定义别名文本（可选）。
    alias?: string;
    // 本行目的：定义原始简介或原文信息（可选）。
    origin?: string;
    // 本行目的：定义已放送集数（可选）。
    airedCount?: number;
    // 本行目的：定义总集数（可选）。
    totalCount?: number;
    // 本行目的：定义中文简介文本（可选）。
    summaryCn?: string;
    // 本行目的：定义中文简介是否翻译完成标记（可选）。
    summaryTranslated?: boolean;
    // 本行目的：定义中文简介翻译是否失败标记（可选）。
    summaryTranslateFailed?: boolean;
    // 本行目的：定义题材类型列表（可选）。
    types?: string[];
    // 本行目的：定义地区列表（可选）。
    regions?: string[];
    // 本行目的：定义受众标签列表（可选）。
    audiences?: string[];
    // 本行目的：定义封面图片地址。
    image: string;
    // 本行目的：定义放送日期（可选）。
    date?: string;
    // 本行目的：定义评分（可选且允许 null）。
    rating?: number | null;
    // 本行目的：定义简介文本（可选）。
    summary?: string;
    // 本行目的：定义条目详情链接（可选）。
    url?: string;
    // 本行目的：定义所属月份（可选）。
    month?: number;
}

/** 类型：StaffPerson | 用途：描述单个制作人员基础信息 | 字段：id、name、url */
// 本行目的：导出制作人员接口。
export interface StaffPerson {
    // 本行目的：定义人员唯一 ID。
    id: number;
    // 本行目的：定义人员名称。
    name: string;
    // 本行目的：定义人员详情链接。
    url: string;
}

/** 类型：StaffGroup | 用途：描述按职责分组的制作人员集合 | 字段：role、people */
// 本行目的：导出制作组接口。
export interface StaffGroup {
    // 本行目的：定义职责名称。
    role: string;
    // 本行目的：定义该职责下的人员列表。
    people: StaffPerson[];
}

/** 类型：CharacterLink | 用途：描述角色条目及其关联关系信息 | 字段：id、name、url、relation */
// 本行目的：导出角色关联接口。
export interface CharacterLink {
    // 本行目的：定义角色唯一 ID。
    id: number;
    // 本行目的：定义角色名称。
    name: string;
    // 本行目的：定义角色详情链接。
    url: string;
    // 本行目的：定义角色关系描述（可选）。
    relation?: string;
}

/** 类型：SeasonMonthData | 用途：描述季度中某个月的数据聚合 | 字段：year、month、count、list */
// 本行目的：导出季度月份数据接口。
export interface SeasonMonthData {
    // 本行目的：定义年份。
    year: number;
    // 本行目的：定义月份（1-12）。
    month: number;
    // 本行目的：定义该月条目总数。
    count: number;
    // 本行目的：定义该月动画条目列表。
    list: MonthAnime[];
}

/** 类型：SeasonResponse | 用途：描述季度接口完整响应结构 | 字段：year、season、fetchedAt、source、months */
// 本行目的：导出季度响应接口。
export interface SeasonResponse {
    // 本行目的：定义响应对应年份。
    year: number;
    // 本行目的：定义季度标识（如 winter/spring）。
    season: string;
    // 本行目的：定义数据抓取时间戳。
    fetchedAt: string;
    // 本行目的：定义数据来源标识。
    source: string;
    // 本行目的：定义季度包含的月份数据列表。
    months: SeasonMonthData[];
}
