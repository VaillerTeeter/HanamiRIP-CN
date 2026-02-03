import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { DownloadItem } from "../types/download";
import { formatBytes, formatSpeed, parseSpeedToBps } from "../utils/format";
import type { SearchResult } from "../types/search";

export const useDownloadPage = () => {
  const downloads = ref<DownloadItem[]>([]);
  let downloadSeq = 1;
  let downloadPoller: number | null = null;

  const addDownload = (
    title: string,
    link: string,
    kind: DownloadItem["kind"],
    path?: string,
    payload?: Partial<DownloadItem>
  ) => {
    const startedAt = new Date().toISOString();
    downloads.value = [
      {
        id: downloadSeq++,
        title,
        link,
        kind,
        startedAt,
        status: "started",
        path,
        ...payload,
      },
      ...downloads.value,
    ];
  };

  const formatDownloadStatus = (status: DownloadItem["status"]) => {
    switch (status) {
      case "completed":
        return "已完成";
      case "failed":
        return "失败";
      case "paused":
        return "已暂停";
      default:
        return "下载中";
    }
  };

  const downloadDisplayTitle = (item: DownloadItem) => {
    if (item.status === "completed") return item.title;
    if (item.status === "paused") return `${item.title}.paused`;
    if (item.status === "failed") return `${item.title}.failed`;
    return `${item.title}.downloading`;
  };

  const isDownloadPaused = (item: DownloadItem) => {
    const state = item.state?.toLowerCase() ?? "";
    return item.status === "paused" || state.includes("paused") || state.includes("stopped");
  };

  const isDownloadTerminal = (item: DownloadItem) => item.status === "completed" || item.status === "failed";

  const handlePauseDownload = async (item: DownloadItem) => {
    if (item.torrentId == null || isDownloadTerminal(item) || isDownloadPaused(item)) return;
    try {
      await invoke("pause_torrent", { id: item.torrentId });
      item.status = "paused";
    } catch (err) {
      console.error("pause_torrent failed", err);
    }
  };

  const handleResumeDownload = async (item: DownloadItem) => {
    if (item.torrentId == null || !isDownloadPaused(item)) return;
    try {
      await invoke("resume_torrent", { id: item.torrentId });
      item.status = "started";
    } catch (err) {
      console.error("resume_torrent failed", err);
    }
  };

  const handleDeleteDownload = async (item: DownloadItem) => {
    if (item.torrentId != null) {
      try {
        await invoke("delete_torrent", { id: item.torrentId });
      } catch (err) {
        console.error("delete_torrent failed", err);
        return;
      }
    }
    downloads.value = downloads.value.filter((row) => row.id !== item.id);
  };

  const hasActiveDownloads = computed(() =>
    downloads.value.some((item) => item.torrentId != null && !isDownloadTerminal(item) && !isDownloadPaused(item))
  );
  const hasPausedDownloads = computed(() =>
    downloads.value.some((item) => item.torrentId != null && isDownloadPaused(item))
  );
  const totalDownloadBps = computed(() => downloads.value.reduce((sum, item) => sum + parseSpeedToBps(item.downloadSpeed), 0));
  const totalUploadBps = computed(() => downloads.value.reduce((sum, item) => sum + parseSpeedToBps(item.uploadSpeed), 0));
  const totalDownloadSpeedLabel = computed(() => formatSpeed(totalDownloadBps.value));
  const totalUploadSpeedLabel = computed(() => formatSpeed(totalUploadBps.value));

  const handlePauseAllDownloads = async () => {
    const active = downloads.value.filter((item) => item.status === "started" && item.torrentId != null);
    await Promise.all(active.map((item) => handlePauseDownload(item)));
  };

  const handleResumeAllDownloads = async () => {
    const paused = downloads.value.filter((item) => item.status === "paused" && item.torrentId != null);
    await Promise.all(paused.map((item) => handleResumeDownload(item)));
  };

  const refreshDownloadStatuses = async () => {
    const active = downloads.value.filter((item) => item.torrentId != null);
    if (!active.length) return;

    const updates = await Promise.all(
      active.map(async (item) => {
        try {
          const stats = await invoke<{
            id: number;
            state: string;
            progressBytes: number;
            totalBytes: number;
            uploadedBytes: number;
            finished: boolean;
            error?: string | null;
            downloadSpeed?: string | null;
            uploadSpeed?: string | null;
            timeRemaining?: string | null;
          }>("get_torrent_status", { id: item.torrentId });
          return { id: item.id, stats };
        } catch (err) {
          console.error("get_torrent_status failed", err);
          return { id: item.id, stats: null };
        }
      })
    );

    let shouldFinalize = false;
    const updated = downloads.value.map((item) => {
      const update = updates.find((u) => u.id === item.id)?.stats;
      if (!update) return item;
      const completed = update.finished;
      const failed = Boolean(update.error);
      const stateLower = update.state?.toLowerCase() ?? "";
      const paused = stateLower.includes("paused") || stateLower.includes("stopped");
      const next: DownloadItem = {
        ...item,
        status: completed ? "completed" : failed ? "failed" : paused ? "paused" : "started",
        state: update.state,
        progressBytes: update.progressBytes,
        totalBytes: update.totalBytes,
        uploadedBytes: update.uploadedBytes,
        error: update.error ?? undefined,
        downloadSpeed: update.downloadSpeed ?? undefined,
        uploadSpeed: update.uploadSpeed ?? undefined,
        timeRemaining: update.timeRemaining ?? undefined,
      };
      if (completed && !item.finalized && item.tempPath && item.finalPath) {
        shouldFinalize = true;
      }
      return next;
    });
    downloads.value = updated;

    if (shouldFinalize) {
      await Promise.all(
        downloads.value.map(async (item) => {
          if (item.status !== "completed" || item.finalized || !item.tempPath || !item.finalPath) return;
          try {
            await invoke("finalize_torrent_download", {
              tempFolder: item.tempPath,
              finalFolder: item.finalPath,
            });
            item.finalized = true;
          } catch (err) {
            console.error("finalize_torrent_download failed", err);
          }
        })
      );
    }
  };

  const handleDownloadClick = async (item: SearchResult, kind: DownloadItem["kind"], link?: string) => {
    if (!link) return;

    let path: string | undefined;
    try {
      const result = await openDialog({
        title: kind === "magnet" ? "选择磁链下载目录" : "选择种子保存目录",
        directory: true,
        multiple: false,
      });
      if (!result || Array.isArray(result)) return;
      path = String(result);
    } catch (err) {
      console.error("openDialog failed", err);
      return;
    }

    try {
      const started = await invoke<{
        id: number;
        infoHash: string;
        name?: string | null;
        outputFolder: string;
        finalFolder: string;
      }>("start_torrent_download", { url: link, outputDir: path });
      addDownload(item.title, link, kind, path, {
        torrentId: started.id,
        infoHash: started.infoHash,
        tempPath: started.outputFolder,
        finalPath: started.finalFolder,
      });
    } catch (err) {
      console.error("start_torrent_download failed", err);
      addDownload(item.title, link, kind, path, { status: "failed" });
    }
  };

  onMounted(() => {
    if (downloadPoller == null) {
      downloadPoller = window.setInterval(refreshDownloadStatuses, 1500);
    }
  });

  onBeforeUnmount(() => {
    if (downloadPoller != null) {
      window.clearInterval(downloadPoller);
      downloadPoller = null;
    }
  });

  return {
    downloads,
    formatDownloadStatus,
    downloadDisplayTitle,
    formatBytes,
    handlePauseDownload,
    handleResumeDownload,
    handleDeleteDownload,
    handlePauseAllDownloads,
    handleResumeAllDownloads,
    handleDownloadClick,
    isDownloadPaused,
    isDownloadTerminal,
    hasActiveDownloads,
    hasPausedDownloads,
    totalDownloadSpeedLabel,
    totalUploadSpeedLabel,
  };
};

export type UseDownloadPageReturn = ReturnType<typeof useDownloadPage>;
