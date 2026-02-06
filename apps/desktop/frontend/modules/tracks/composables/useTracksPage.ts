/*
  轨道解析与混流页面逻辑：
  - 选择媒体文件并解析轨道
  - 选择轨道与默认语言
  - 生成混流任务队列并执行
*/
import { reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import type { MixQueueItem, MixTrackInput, TrackFileResult, TrackInfo, TrackItem, TrackType } from "../types/tracks";

// 轨道类型 -> 中文显示名称。
const trackLabelMap: Record<TrackType, string> = {
  video: "视频",
  audio: "音频",
  subtitle: "字幕",
};

/**
 * 轨道页业务入口：返回轨道状态与操作函数。
 */
export const useTracksPage = () => {
  // 用户选择的文件列表（按轨道类型分组）。
  const trackFiles = ref<Record<TrackType, TrackItem[]>>({
    video: [],
    audio: [],
    subtitle: [],
  });
  // 解析后的轨道信息（按类型分组）。
  const trackInfos = ref<Record<TrackType, TrackFileResult[]>>({
    video: [],
    audio: [],
    subtitle: [],
  });
  // 解析过程中的 loading、进度与错误。
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
  // 混流队列相关状态。
  const trackMixLoading = ref(false);
  const trackMixError = ref("");
  const trackMixResult = ref("");
  const mixQueue = ref<MixQueueItem[]>([]);
  const mixQueueRunning = ref(false);
  const mixQueueDetailVisible = ref(false);
  const selectedMixTask = ref<MixQueueItem | null>(null);
  // 前端自增 ID（文件/队列用）。
  let trackSeq = 1;
  let mixQueueSeq = 1;

  // 各类型轨道的默认语言（用于写入 mkvmerge 参数）。
  const trackLangDefaults = reactive<Record<TrackType, string>>({
    video: "ja",
    audio: "ja",
    subtitle: "zh-Hans",
  });

  // 轨道语言下拉选项。
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

  // 选择文件并写入 trackFiles。
  const addTrackFile = async (type: TrackType) => {
    try {
      const videoExt = ["mkv", "mp4", "avi", "mov", "ts", "m2ts", "webm", "mpg", "mpeg"];
      const subtitleExt = ["srt", "ass", "ssa", "vtt", "sup", "sub"];
      const extMap: Record<TrackType, string[]> = {
        video: videoExt,
        audio: videoExt,
        subtitle: [...videoExt, ...subtitleExt],
      };
      // 打开文件选择对话框。
      const result = await openDialog({
        title: `选择${trackLabelMap[type]}文件`,
        directory: false,
        multiple: false,
        filters: [{ name: `${trackLabelMap[type]}文件`, extensions: extMap[type] }],
      });
      if (!result || Array.isArray(result)) return;
      const file = String(result);
      // 可选读取文件大小（失败不阻塞）。
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

  // 调用后端解析轨道信息。
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
        // 调用后端解析轨道。
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
        // 解析失败时写入错误提示。
        trackErrors.value[type] = typeof err === "string" ? err : err?.message || "解析失败";
        results.push({ file, tracks: [] });
      } finally {
        trackProgress.value[type] = Math.round(((i + 1) / totalFiles) * 100);
      }
    }
    trackInfos.value[type] = results;
    trackLoading.value[type] = false;
  };

  // 选择混流后输出文件路径。
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

  // 从解析结果中收集“需要混流的轨道”。
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

  // 把混流任务加入队列（不立即执行）。
  const enqueueMixTask = async () => {
    if (trackMixLoading.value) return;
    trackMixError.value = "";
    trackMixResult.value = "";

    const videoInput = collectMixInput("video");
    const audioInput = collectMixInput("audio");
    const subtitleInput = collectMixInput("subtitle");

    // 视频轨道是必须的。
    if (!videoInput) {
      trackMixError.value = "请先检测并选择至少一个视频轨道";
      return;
    }

    trackMixLoading.value = true;
    try {
      // 选择输出路径。
      const outputPath = await pickOutputPath(trackFiles.value.video[0]);
      if (!outputPath) return;

      const inputs: MixTrackInput[] = [videoInput];
      if (audioInput) inputs.push(audioInput);
      if (subtitleInput) inputs.push(subtitleInput);

      // 生成队列项。
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

  // 顺序执行队列中的混流任务。
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
        // 调用后端执行混流。
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
        // 捕获错误并写入提示。
        item.status = "failed";
        item.message = typeof err === "string" ? err : err?.message || "合成失败";
      }
    }
    mixQueueRunning.value = false;
  };

  // 清空队列（执行中则拒绝）。
  const clearMixQueue = () => {
    if (mixQueueRunning.value) return;
    mixQueue.value = [];
    trackMixResult.value = "";
    trackMixError.value = "";
  };

  // 打开队列详情弹窗。
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
