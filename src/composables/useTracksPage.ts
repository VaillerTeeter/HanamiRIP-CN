import { reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import type { MixQueueItem, MixTrackInput, TrackFileResult, TrackInfo, TrackItem, TrackType } from "../types/tracks";

const trackLabelMap: Record<TrackType, string> = {
  video: "视频",
  audio: "音频",
  subtitle: "字幕",
};

export const useTracksPage = () => {
  const trackFiles = ref<Record<TrackType, TrackItem[]>>({
    video: [],
    audio: [],
    subtitle: [],
  });
  const trackInfos = ref<Record<TrackType, TrackFileResult[]>>({
    video: [],
    audio: [],
    subtitle: [],
  });
  const trackLoading = ref<Record<TrackType, boolean>>({
    video: false,
    audio: false,
    subtitle: false,
  });
  const trackProgress = ref<Record<TrackType, number>>({
    video: 0,
    audio: 0,
    subtitle: 0,
  });
  const trackErrors = ref<Record<TrackType, string>>({
    video: "",
    audio: "",
    subtitle: "",
  });
  const trackMixLoading = ref(false);
  const trackMixError = ref("");
  const trackMixResult = ref("");
  const mixQueue = ref<MixQueueItem[]>([]);
  const mixQueueRunning = ref(false);
  const mixQueueDetailVisible = ref(false);
  const selectedMixTask = ref<MixQueueItem | null>(null);
  let trackSeq = 1;
  let mixQueueSeq = 1;

  const trackLangDefaults = reactive<Record<TrackType, string>>({
    video: "ja",
    audio: "ja",
    subtitle: "zh-Hans",
  });

  const trackLanguageOptions = [
    { label: "自动", value: "" },
    { label: "日语 (ja)", value: "ja" },
    { label: "英语 (en)", value: "en" },
    { label: "简体中文 (zh-Hans)", value: "zh-Hans" },
    { label: "繁体中文 (zh-Hant)", value: "zh-Hant" },
    { label: "中文 (zh)", value: "zh" },
    { label: "韩语 (ko)", value: "ko" },
    { label: "法语 (fr)", value: "fr" },
    { label: "德语 (de)", value: "de" },
    { label: "西班牙语 (es)", value: "es" },
    { label: "未知 (und)", value: "und" },
  ];

  const addTrackFile = async (type: TrackType) => {
    try {
      const videoExt = ["mkv", "mp4", "avi", "mov", "ts", "m2ts", "webm", "mpg", "mpeg"];
      const subtitleExt = ["srt", "ass", "ssa", "vtt", "sup", "sub"];
      const extMap: Record<TrackType, string[]> = {
        video: videoExt,
        audio: videoExt,
        subtitle: [...videoExt, ...subtitleExt],
      };
      const result = await openDialog({
        title: `选择${trackLabelMap[type]}文件`,
        directory: false,
        multiple: false,
        filters: [{ name: `${trackLabelMap[type]}文件`, extensions: extMap[type] }],
      });
      if (!result || Array.isArray(result)) return;
      const file = String(result);
      let fileSize: string | undefined;
      try {
        const size = await invoke<string | null>("get_media_file_size", { path: file });
        fileSize = size ?? undefined;
      } catch (err) {
        console.error("get_media_file_size failed", err);
        fileSize = undefined;
      }
      trackFiles.value[type] = [
        {
          id: trackSeq++,
          name: file.split(/[\\/]/).filter(Boolean).pop() || file,
          path: file,
          fileSize,
        },
      ];
      trackInfos.value[type] = [];
      trackErrors.value[type] = "";
      trackProgress.value[type] = 0;
    } catch (err) {
      console.error("openDialog failed", err);
    }
  };

  const detectTracks = async (type: TrackType) => {
    if (trackLoading.value[type]) return;
    if (!trackFiles.value[type].length) {
      trackErrors.value[type] = `请先添加${trackLabelMap[type]}文件`;
      trackInfos.value[type] = [];
      trackProgress.value[type] = 0;
      return;
    }
    trackLoading.value[type] = true;
    trackProgress.value[type] = 0;
    trackInfos.value[type] = [];
    trackErrors.value[type] = "";

    const files = trackFiles.value[type];
    const totalFiles = files.length;
    const results: TrackFileResult[] = [];
    for (let i = 0; i < files.length; i += 1) {
      const file = files[i];
      try {
        const response = await invoke<{ tracks: TrackInfo[] }>("parse_media_tracks", {
          path: file.path,
          kind: type,
        });
        const tracks = (response.tracks || []).map((track) => ({
          ...track,
          selected: track.selected ?? true,
          langOverride: track.lang || trackLangDefaults[type],
        }));
        results.push({ file, tracks });
      } catch (err: any) {
        trackErrors.value[type] = typeof err === "string" ? err : err?.message || "解析失败";
        results.push({ file, tracks: [] });
      } finally {
        trackProgress.value[type] = Math.round(((i + 1) / totalFiles) * 100);
      }
    }
    trackInfos.value[type] = results;
    trackLoading.value[type] = false;
  };

  const pickOutputPath = async (baseFile?: TrackItem) => {
    const baseName = baseFile?.name ? baseFile.name.replace(/\.[^/.\\]+$/, "") : "mixed";
    const dir = baseFile?.path ? baseFile.path.replace(/[\\/][^\\/]+$/, "") : "";
    const defaultPath = dir ? `${dir}\\${baseName}_mixed.mkv` : `${baseName}_mixed.mkv`;
    const result = await saveDialog({
      title: "保存混合后的视频",
      defaultPath,
      filters: [{ name: "MKV", extensions: ["mkv"] }],
    });
    if (!result) return null;
    return result.endsWith(".mkv") ? result : `${result}.mkv`;
  };

  const collectMixInput = (type: TrackType): MixTrackInput | null => {
    const files = trackFiles.value[type];
    if (!files.length) return null;
    const file = files[0];
    const group = trackInfos.value[type].find((item) => item.file.id === file.id);
    if (!group || !group.tracks.length) return null;
    const selected = group.tracks.filter((track) => track.selected !== false).map((track) => track.trackId);
    if (!selected.length) return null;
    const trackLangs: Record<string, string> = {};
    group.tracks.forEach((track) => {
      if (track.selected === false) return;
      const lang = trackLangDefaults[type];
      if (lang) {
        trackLangs[track.trackId] = lang;
      }
    });
    return { path: file.path, kind: type, trackIds: selected, trackLangs };
  };

  const enqueueMixTask = async () => {
    if (trackMixLoading.value) return;
    trackMixError.value = "";
    trackMixResult.value = "";

    const videoInput = collectMixInput("video");
    const audioInput = collectMixInput("audio");
    const subtitleInput = collectMixInput("subtitle");

    if (!videoInput) {
      trackMixError.value = "请先检测并选择至少一个视频轨道";
      return;
    }

    trackMixLoading.value = true;
    try {
      const outputPath = await pickOutputPath(trackFiles.value.video[0]);
      if (!outputPath) return;

      const inputs: MixTrackInput[] = [videoInput];
      if (audioInput) inputs.push(audioInput);
      if (subtitleInput) inputs.push(subtitleInput);

      mixQueue.value.push({
        id: mixQueueSeq++,
        createdAt: new Date().toLocaleString(),
        outputPath,
        inputs,
        status: "queued",
      });
      trackFiles.value = { video: [], audio: [], subtitle: [] };
      trackInfos.value = { video: [], audio: [], subtitle: [] };
      trackErrors.value = { video: "", audio: "", subtitle: "" };
      trackProgress.value = { video: 0, audio: 0, subtitle: 0 };
      trackMixResult.value = "已添加到混流任务队列";
    } catch (err: any) {
      trackMixError.value = typeof err === "string" ? err : err?.message || "添加任务失败";
    } finally {
      trackMixLoading.value = false;
    }
  };

  const startMixQueue = async () => {
    if (mixQueueRunning.value) return;
    trackMixError.value = "";
    trackMixResult.value = "";
    const pending = mixQueue.value.some((item) => item.status === "queued");
    if (!pending) {
      trackMixResult.value = "当前没有待处理的混流任务";
      return;
    }

    mixQueueRunning.value = true;
    for (const item of mixQueue.value) {
      if (item.status !== "queued") continue;
      item.status = "running";
      item.message = undefined;
      try {
        const output = await invoke<string>("mix_media_tracks", {
          inputs: item.inputs.map((input) => ({
            path: input.path,
            kind: input.kind,
            trackIds: input.trackIds,
            trackLangs: input.trackLangs,
          })),
          outputPath: item.outputPath,
        });
        item.status = "success";
        item.message = output;
      } catch (err: any) {
        item.status = "failed";
        item.message = typeof err === "string" ? err : err?.message || "合成失败";
      }
    }
    mixQueueRunning.value = false;
  };

  const clearMixQueue = () => {
    if (mixQueueRunning.value) return;
    mixQueue.value = [];
    trackMixResult.value = "";
    trackMixError.value = "";
  };

  const openMixTaskDetail = (item: MixQueueItem) => {
    selectedMixTask.value = item;
    mixQueueDetailVisible.value = true;
  };

  return {
    trackFiles,
    trackInfos,
    trackLoading,
    trackProgress,
    trackErrors,
    trackMixLoading,
    trackMixError,
    trackMixResult,
    mixQueue,
    mixQueueRunning,
    mixQueueDetailVisible,
    selectedMixTask,
    trackLangDefaults,
    trackLanguageOptions,
    addTrackFile,
    detectTracks,
    enqueueMixTask,
    startMixQueue,
    clearMixQueue,
    openMixTaskDetail,
  };
};

export type UseTracksPageReturn = ReturnType<typeof useTracksPage>;
