export type TrackType = "video" | "audio" | "subtitle";

export type TrackItem = {
  id: number;
  name: string;
  path: string;
  fileSize?: string;
};

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

export type TrackFileResult = {
  file: TrackItem;
  tracks: TrackInfo[];
};

export type MixTrackInput = {
  path: string;
  kind: TrackType;
  trackIds: string[];
  trackLangs?: Record<string, string>;
};

export type MixQueueStatus = "queued" | "running" | "success" | "failed";

export type MixQueueItem = {
  id: number;
  createdAt: string;
  outputPath: string;
  inputs: MixTrackInput[];
  status: MixQueueStatus;
  message?: string;
};
