/**
 * 轨道类型：视频/音频/字幕。
 */
export type TrackType = "video" | "audio" | "subtitle";

/**
 * 用户选择的媒体文件。
 */
export type TrackItem = {
  id: number;
  name: string;
  path: string;
  fileSize?: string;
};

/**
 * 解析出的单条轨道信息。
 */
export type TrackInfo = {
  trackId: string;
  codec: string;
  lang?: string;
  languageName?: string;
  trackName?: string;
  isDefault?: boolean;
  isForced?: boolean;
  charset?: string;
  attributes?: string;
  container?: string;
  fileSize?: string;
  selected?: boolean;
  langOverride?: string;
};

/**
 * 某个文件的解析结果（含多条轨道）。
 */
export type TrackFileResult = {
  file: TrackItem;
  tracks: TrackInfo[];
};

/**
 * 发送给后端的混流输入。
 */
export type MixTrackInput = {
  path: string;
  kind: TrackType;
  trackIds: string[];
  trackLangs?: Record<string, string>;
};

/**
 * 混流队列状态。
 */
export type MixQueueStatus = "queued" | "running" | "success" | "failed";

/**
 * 队列中的单个混流任务。
 */
export type MixQueueItem = {
  id: number;
  createdAt: string;
  outputPath: string;
  inputs: MixTrackInput[];
  status: MixQueueStatus;
  message?: string;
};
