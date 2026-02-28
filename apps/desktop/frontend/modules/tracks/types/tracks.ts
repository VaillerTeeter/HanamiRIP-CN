/** 文件：tracks.ts | 用途：定义轨道解析与混流队列模块的核心类型 | 关键对象：TrackType, TrackItem, TrackInfo, TrackFileResult, MixTrackInput, MixQueueStatus, MixQueueItem */
/** 类型：TrackType | 用途：约束轨道类别枚举值 | 取值：video、audio、subtitle */
// 本行目的：导出轨道类型联合字面量。
export type TrackType = "video" | "audio" | "subtitle";

/** 类型：TrackItem | 用途：描述用户选择的源文件基础信息 | 字段：id、name、path、fileSize */
// 本行目的：导出轨道文件条目类型。
export type TrackItem = {
    // 本行目的：定义文件条目唯一ID。
    id: number;
    // 本行目的：定义文件名（用于列表展示）。
    name: string;
    // 本行目的：定义文件绝对路径。
    path: string;
    // 本行目的：定义文件大小文本（可选）。
    fileSize?: string;
};

/** 类型：TrackInfo | 用途：描述单条媒体轨道的解析信息与前端选择状态 | 字段：轨道元信息 + 前端扩展字段 */
// 本行目的：导出媒体轨道信息类型。
export type TrackInfo = {
    // 本行目的：定义轨道ID。
    trackId: string;
    // 本行目的：定义编解码器标识。
    codec: string;
    // 本行目的：定义语言码（可选）。
    lang?: string;
    // 本行目的：定义语言名称（可选）。
    languageName?: string;
    // 本行目的：定义轨道名称（可选）。
    trackName?: string;
    // 本行目的：定义是否默认轨（可选）。
    isDefault?: boolean;
    // 本行目的：定义是否强制轨（可选）。
    isForced?: boolean;
    // 本行目的：定义字符集信息（可选）。
    charset?: string;
    // 本行目的：定义附加属性文本（可选）。
    attributes?: string;
    // 本行目的：定义容器信息（可选）。
    container?: string;
    // 本行目的：定义轨道对应文件大小（可选）。
    fileSize?: string;
    // 本行目的：定义前端是否选中该轨道（可选）。
    selected?: boolean;
    // 本行目的：定义前端覆盖语言码（可选）。
    langOverride?: string;
};

/** 类型：TrackFileResult | 用途：描述单个文件及其解析得到的轨道列表 | 字段：file、tracks */
// 本行目的：导出文件解析结果类型。
export type TrackFileResult = {
    // 本行目的：定义源文件信息。
    file: TrackItem;
    // 本行目的：定义该文件对应的轨道列表。
    tracks: TrackInfo[];
};

/** 类型：MixTrackInput | 用途：描述后端混流命令的单个输入源参数 | 字段：path、kind、trackIds、trackLangs */
// 本行目的：导出混流输入参数类型。
export type MixTrackInput = {
    // 本行目的：定义输入文件路径。
    path: string;
    // 本行目的：定义输入类型（视频/音频/字幕）。
    kind: TrackType;
    // 本行目的：定义参与混流的轨道ID列表。
    trackIds: string[];
    // 本行目的：定义轨道ID到语言码映射（可选）。
    trackLangs?: Record<string, string>;
};

/** 类型：MixQueueStatus | 用途：约束混流任务状态枚举 | 取值：queued、running、success、failed */
// 本行目的：导出混流队列状态联合字面量。
export type MixQueueStatus = "queued" | "running" | "success" | "failed";

/** 类型：MixQueueItem | 用途：描述混流队列中的单个任务对象 | 字段：任务元信息、输入数组、状态与消息 */
// 本行目的：导出混流队列任务类型。
export type MixQueueItem = {
    // 本行目的：定义任务唯一ID。
    id: number;
    // 本行目的：定义任务创建时间文本。
    createdAt: string;
    // 本行目的：定义输出文件路径。
    outputPath: string;
    // 本行目的：定义混流输入源列表。
    inputs: MixTrackInput[];
    // 本行目的：定义任务当前状态。
    status: MixQueueStatus;
    // 本行目的：定义任务执行消息（可选，成功输出或失败原因）。
    message?: string;
};
