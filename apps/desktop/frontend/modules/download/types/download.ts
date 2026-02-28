/** 文件：download.ts | 用途：定义下载模块核心数据结构类型 | 关键对象：DownloadItem */
// 变量：DownloadItem | 含义：下载列表单条记录的统一类型定义 | 类型：type alias | 作用域：模块级
/** 类型：DownloadItem | 用途：描述下载任务在前端展示与控制所需的全部字段 | 字段：标识、状态、路径、进度、速度与错误信息 */
// 本行目的：导出下载条目类型，供下载页面与组合逻辑共享。
export type DownloadItem = {
    // 本行目的：前端本地唯一标识，用于列表渲染 key 与条目定位。
    id: number;
    // 本行目的：下载标题，通常来自搜索结果或元数据名称。
    title: string;
    // 本行目的：原始下载链接（磁力或种子 URL）。
    link: string;
    // 本行目的：下载类型枚举，区分 magnet 与 torrent。
    kind: "magnet" | "torrent";
    // 本行目的：任务开始时间（ISO 字符串），用于展示与排序。
    startedAt: string;
    // 本行目的：下载状态枚举，驱动按钮可用性与状态文案。
    status: "started" | "paused" | "completed" | "failed";
    // 本行目的：后端 torrent 任务 ID（可选），用于暂停/恢复/删除。
    torrentId?: number;
    // 本行目的：torrent 信息哈希（可选），用于任务识别与追踪。
    infoHash?: string;
    // 本行目的：可恢复任务 ID（可选），用于断点恢复与丢弃。
    taskId?: string;
    // 本行目的：下载临时目录路径（可选），用于完成后归档。
    tempPath?: string;
    // 本行目的：下载最终目录路径（可选），用于归档目标位置。
    finalPath?: string;
    // 本行目的：占位文件路径（可选），用于冲突与替换流程。
    placeholderPath?: string;
    // 本行目的：是否已执行最终归档（可选），避免重复 finalize。
    finalized?: boolean;
    // 本行目的：是否被归档冲突阻塞（可选），用于提示用户处理。
    finalizeBlocked?: boolean;
    // 本行目的：冲突文件列表（可选），用于展示阻塞详情。
    conflictFiles?: string[];
    // 本行目的：建议的最终路径（可选），用于冲突时给出替代方案。
    suggestedFinalPath?: string;
    // 本行目的：用户选择的保存路径（可选），用于界面展示。
    path?: string;
    // 本行目的：后端原始状态文本（可选），用于调试与展示。
    state?: string;
    // 本行目的：当前已下载字节数（可选），用于进度计算。
    progressBytes?: number;
    // 本行目的：任务总字节数（可选），用于进度条比例。
    totalBytes?: number;
    // 本行目的：已上传字节数（可选），用于做种或上传统计。
    uploadedBytes?: number;
    // 本行目的：错误信息（可选），失败时用于提示用户。
    error?: string;
    // 本行目的：当前下载速度文本（可选），例如 "1.2 MB/s"。
    downloadSpeed?: string;
    // 本行目的：当前上传速度文本（可选），例如 "200 KB/s"。
    uploadSpeed?: string;
    // 本行目的：预计剩余时间文本（可选），用于进度预估展示。
    timeRemaining?: string;
    // 本行目的：结束 DownloadItem 类型定义。
};
