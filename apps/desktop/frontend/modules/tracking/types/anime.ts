/**
 * 单月番剧条目（来自后端季节接口）。
 */
export interface MonthAnime {
  // 条目 ID。
  id: number;
  // 原始名称。
  name: string;
  // 中文名称（可能为空）。
  nameCn?: string;
  // 其他别名（可选）。
  alias?: string;
  // 原作来源（漫画/小说等）。
  origin?: string;
  // 已播出集数。
  airedCount?: number;
  // 总集数。
  totalCount?: number;
  // 中文简介。
  summaryCn?: string;
  // 是否使用翻译得到的简介。
  summaryTranslated?: boolean;
  // 翻译是否失败（用于 UI 提示）。
  summaryTranslateFailed?: boolean;
  // 类型标签。
  types?: string[];
  // 地区标签。
  regions?: string[];
  // 受众标签。
  audiences?: string[];
  // 封面图片 URL。
  image: string;
  // 首播日期。
  date?: string;
  // 评分。
  rating?: number | null;
  // 原始简介。
  summary?: string;
  // 条目链接。
  url?: string;
  // 所属月份（用于筛选）。
  month?: number;
}

/**
 * 制作人员信息。
 */
export interface StaffPerson {
  id: number;
  name: string;
  url: string;
}

/**
 * 制作人员分组（按职位）。
 */
export interface StaffGroup {
  role: string;
  people: StaffPerson[];
}

/**
 * 角色信息（用于详情展示）。
 */
export interface CharacterLink {
  id: number;
  name: string;
  url: string;
  relation?: string;
}

/**
 * 某个月的番剧列表。
 */
export interface SeasonMonthData {
  year: number;
  month: number;
  count: number;
  list: MonthAnime[];
}

/**
 * 季度查询结果。
 */
export interface SeasonResponse {
  year: number;
  season: string;
  fetchedAt: string;
  source: string;
  months: SeasonMonthData[];
}
