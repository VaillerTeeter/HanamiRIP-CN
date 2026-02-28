/** 文件：format.ts | 用途：提供评分、日期、字节与速率等通用格式化工具函数 | 关键对象：formatRating, formatStars, formatAirDate, formatBytes, parseSpeedToBps, formatSpeed */
// 变量：weekdayLabels | 含义：星期索引到中文星期文本映射数组 | 类型：string[] | 作用域：模块级
// 本行目的：定义日期格式化时使用的星期标签。
const weekdayLabels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

// 变量：formatRating | 含义：评分格式化函数 | 类型：(value?: number | null) => string | 作用域：模块级
/** 函数：formatRating | 输入：可选评分数值 | 输出：一位小数字符串 | 可能失败：无（非数字回退 0.0） */
// 本行目的：导出评分格式化方法。
export const formatRating = (value?: number | null) => (typeof value === "number" ? value.toFixed(1) : "0.0");

// 变量：formatStars | 含义：把评分转换为星级字符 | 类型：(value?: number | null) => string | 作用域：模块级
/** 函数：formatStars | 输入：可选评分数值（0-10） | 输出：五星制星级字符串 | 可能失败：无效输入回退空星 */
// 本行目的：导出星级格式化方法。
export const formatStars = (value?: number | null) => {
    // 本行目的：非数字时返回全空星。
    if (typeof value !== "number") return "☆☆☆☆☆";
    // 变量：normalized | 含义：限制到 0-10 的评分值 | 类型：number | 作用域：formatStars 内部
    // 本行目的：对评分进行边界裁剪。
    const normalized = Math.max(0, Math.min(10, value));
    // 变量：filled | 含义：应显示的实心星数量 | 类型：number | 作用域：formatStars 内部
    // 本行目的：把 10 分制转换为 5 星制。
    const filled = Math.round(normalized / 2);
    // 本行目的：拼接实心星与空心星文本。
    return "★".repeat(filled) + "☆".repeat(5 - filled);
};

// 变量：formatAirDate | 含义：放送日期格式化函数 | 类型：(value?: string) => string | 作用域：模块级
/** 函数：formatAirDate | 输入：可选日期字符串（yyyy-mm-dd） | 输出：日期+星期文本 | 可能失败：日期非法时回退原值 */
// 本行目的：导出放送日期格式化方法。
export const formatAirDate = (value?: string) => {
    // 本行目的：无输入时返回“未知”。
    if (!value) return "未知";
    // 本行目的：解析日期字符串。
    const parsed = new Date(`${value}T00:00:00`);
    // 本行目的：解析失败时返回原始值。
    if (Number.isNaN(parsed.getTime())) return value;
    // 本行目的：返回带星期后缀的日期文本。
    return `${value}（${weekdayLabels[parsed.getDay()]}）`;
};

// 变量：formatBytes | 含义：字节数格式化函数 | 类型：(value?: number) => string | 作用域：模块级
/** 函数：formatBytes | 输入：可选字节数 | 输出：带单位的人类可读文本 | 可能失败：无效输入回退 "-" */
// 本行目的：导出字节格式化方法。
export const formatBytes = (value?: number) => {
    // 本行目的：无效输入返回占位符。
    if (value == null || Number.isNaN(value)) return "-";

    // 变量：units | 含义：字节单位数组 | 类型：string[] | 作用域：formatBytes 内部
    // 本行目的：定义单位换算序列。
    const units = ["B", "KB", "MB", "GB", "TB"];

    // 变量：size | 含义：当前换算中的数值 | 类型：number | 作用域：formatBytes 内部
    // 本行目的：初始化待换算大小。
    let size = value;
    // 变量：unitIndex | 含义：当前单位索引 | 类型：number | 作用域：formatBytes 内部
    // 本行目的：初始化单位索引为 B。
    let unitIndex = 0;

    // 本行目的：按 1024 逐级缩放到合适单位。
    while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex += 1;
    }

    // 本行目的：返回格式化后的大小文本。
    return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
};

// 变量：parseSpeedToBps | 含义：把速率文本解析为字节每秒数值 | 类型：(value?: string) => number | 作用域：模块级
/** 函数：parseSpeedToBps | 输入：速率字符串（如 "1.2 MB/s"） | 输出：B/s 数值 | 可能失败：解析失败回退 0 */
// 本行目的：导出速率文本解析方法。
export const parseSpeedToBps = (value?: string) => {
    // 本行目的：空输入返回 0。
    if (!value) return 0;

    // 变量：match | 含义：速率数值与单位匹配结果 | 类型：RegExpMatchArray | null | 作用域：parseSpeedToBps 内部
    // 本行目的：使用正则提取数字与单位。
    const match = value.match(/([\d.]+)\s*([a-zA-Z/]+)?/);
    // 本行目的：匹配失败返回 0。
    if (!match) return 0;

    // 变量：num | 含义：解析后的数值部分 | 类型：number | 作用域：parseSpeedToBps 内部
    // 本行目的：把匹配到的数值转成浮点数。
    const num = Number.parseFloat(match[1]);
    // 本行目的：数值非法时返回 0。
    if (!Number.isFinite(num)) return 0;

    // 变量：unitRaw | 含义：原始单位文本（标准化前） | 类型：string | 作用域：parseSpeedToBps 内部
    // 本行目的：去空白、去 /s 并统一大写。
    const unitRaw = (match[2] || "B").replace(/\s*/g, "").replace(/\/s/i, "").toUpperCase();
    // 变量：unit | 含义：标准化后的单位文本 | 类型：string | 作用域：parseSpeedToBps 内部
    // 本行目的：兼容处理 /S 后缀。
    const unit = unitRaw.endsWith("/S") ? unitRaw.slice(0, -2) : unitRaw;

    // 变量：factorMap | 含义：单位到字节倍率映射 | 类型：Record<string, number> | 作用域：parseSpeedToBps 内部
    // 本行目的：定义常见单位倍率表。
    const factorMap: Record<string, number> = {
        B: 1,
        KB: 1024,
        KIB: 1024,
        MB: 1024 ** 2,
        MIB: 1024 ** 2,
        GB: 1024 ** 3,
        GIB: 1024 ** 3,
        TB: 1024 ** 4,
        TIB: 1024 ** 4,
    };

    // 变量：factor | 含义：当前单位对应倍率 | 类型：number | 作用域：parseSpeedToBps 内部
    // 本行目的：读取单位倍率，未知单位回退 1。
    const factor = factorMap[unit] ?? 1;

    // 本行目的：返回换算后的字节每秒。
    return num * factor;
};

// 变量：formatSpeed | 含义：把 B/s 数值格式化为可读速率文本 | 类型：(bps: number) => string | 作用域：模块级
/** 函数：formatSpeed | 输入：字节每秒数值 | 输出：带单位速率文本 | 可能失败：无效输入回退 "0 B/s" */
// 本行目的：导出速率格式化方法。
export const formatSpeed = (bps: number) => {
    // 本行目的：非法或非正速率回退默认文本。
    if (!Number.isFinite(bps) || bps <= 0) return "0 B/s";

    // 变量：units | 含义：速率单位数组 | 类型：string[] | 作用域：formatSpeed 内部
    // 本行目的：定义速率单位序列。
    const units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];

    // 变量：size | 含义：当前换算中的速率值 | 类型：number | 作用域：formatSpeed 内部
    // 本行目的：初始化待换算速率。
    let size = bps;
    // 变量：unitIndex | 含义：当前单位索引 | 类型：number | 作用域：formatSpeed 内部
    // 本行目的：初始化单位索引。
    let unitIndex = 0;

    // 本行目的：按 1024 逐级缩放到合适速率单位。
    while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex += 1;
    }

    // 本行目的：返回格式化后的速率文本。
    return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
};
