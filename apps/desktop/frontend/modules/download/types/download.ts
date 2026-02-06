/**
 * 下载任务在前端的完整状态结构。
 * 用于列表渲染与状态轮询。
 */
export type DownloadItem = {
  // 前端自增 ID（仅用于 UI 标识）。
  id: number;
  // 下载标题（通常是条目名称）。
  title: string;
  // 下载链接（磁链或种子链接）。
  link: string;
  // 下载类型：磁链或种子。
  kind: "magnet" | "torrent";
  // 开始时间（ISO 字符串）。
  startedAt: string;
  // 当前状态：开始/失败/完成/暂停。
  status: "started" | "failed" | "completed" | "paused";
  // 用户选择的保存路径。
  path?: string;
  // 临时下载目录（后端使用）。
  tempPath?: string;
  // 最终下载目录（后端使用）。
  finalPath?: string;
  // 是否已执行“完成后移动文件”的收尾动作。
  finalized?: boolean;
  // 后端下载任务 ID。
  torrentId?: number;
  // 种子 infoHash。
  infoHash?: string;
  // 已下载字节数。
  progressBytes?: number;
  // 总字节数。
  totalBytes?: number;
  // 已上传字节数。
  uploadedBytes?: number;
  // 后端状态文本（用于判定暂停/停止）。
  state?: string;
  // 失败原因。
  error?: string;
  // 下载速度（字符串，如 "1.2 MB/s"）。
  downloadSpeed?: string;
  // 上传速度（字符串）。
  uploadSpeed?: string;
  // 剩余时间描述（字符串）。
  timeRemaining?: string;
};
