/** 文件：search.ts | 用途：定义搜索模块的核心类型契约，约束关键词、结果与逻辑选项结构 | 关键对象：LogicOp, SearchTerm, SearchResult, LogicOption */
/** 类型：LogicOp | 用途：约束关键词拼装时可用的逻辑运算符 | 取值：and（与）、or（或）、not（非） */
// 本行目的：导出逻辑运算符联合字面量类型，避免使用任意字符串。
export type LogicOp = "and" | "or" | "not";

/** 类型：SearchTerm | 用途：描述单个搜索关键词条目 | 字段：id、value、op、source */
// 本行目的：导出关键词条目对象类型。
export type SearchTerm = {
    // 本行目的：定义关键词条目的唯一标识，便于列表渲染与删除。
    id: number;
    // 本行目的：定义关键词的实际文本内容。
    value: string;
    // 本行目的：定义该关键词参与查询时使用的逻辑运算符。
    op: LogicOp;
    // 本行目的：标记关键词来源（预置、自定义、追踪别名）。
    source: "preset" | "custom" | "tracked";
};

/** 类型：SearchResult | 用途：描述从搜索页面解析出的单条结果 | 字段：标题、详情链接、下载链接与元信息 */
// 本行目的：导出搜索结果对象类型。
export type SearchResult = {
    // 本行目的：定义结果标题（展示主文本，通常必有）。
    title: string;
    // 本行目的：定义详情页链接（可能不存在，因此可选）。
    detailUrl?: string;
    // 本行目的：定义磁力链接（有些结果没有磁力链接，故为可选）。
    magnet?: string;
    // 本行目的：定义 torrent 下载链接（并非每条结果都提供）。
    download?: string;
    // 本行目的：定义文件大小文本（解析不到时为空）。
    size?: string;
    // 本行目的：定义发布时间文本（解析不到时为空）。
    date?: string;
};

/** 类型：LogicOption | 用途：描述界面中逻辑运算符下拉/按钮选项 | 字段：value、labelKey */
// 本行目的：导出逻辑选项对象类型。
export type LogicOption = {
    // 本行目的：定义选项对应的逻辑值。
    value: LogicOp;
    // 本行目的：定义国际化文案键，供界面按语言渲染。
    labelKey: string;
};
