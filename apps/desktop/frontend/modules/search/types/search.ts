/**
 * 搜索逻辑运算符：
 * - and：必须同时满足
 * - or：满足其一
 * - not：排除
 */
export type LogicOp = "and" | "or" | "not";

/**
 * 单个搜索词：
 * - value：关键词文本
 * - op：逻辑运算
 * - source：来源（预设/自定义/追番别名）
 */
export type SearchTerm = { value: string; op: LogicOp; source: "preset" | "custom" | "tracked" };

/**
 * 搜索结果条目（从 HTML 解析而来）。
 */
export type SearchResult = {
  // 标题文本。
  title: string;
  // 详情页链接。
  detailUrl?: string;
  // 磁力链接。
  magnet?: string;
  // 种子下载链接。
  download?: string;
  // 文件大小文本。
  size?: string;
  // 发布日期文本。
  date?: string;
};
