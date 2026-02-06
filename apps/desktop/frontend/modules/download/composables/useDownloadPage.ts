/*
  下载页面的组合式逻辑：
  - 维护下载列表状态
  - 调用后端下载/暂停/恢复/删除接口
  - 定时轮询下载状态并更新 UI
*/
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { DownloadItem } from "../types/download";
import { formatBytes, formatSpeed, parseSpeedToBps } from "../../../shared/utils/format";
import type { SearchResult } from "../../search/types/search";

/**
 * 下载页业务入口：返回给组件使用的状态与操作函数。
 */
export const useDownloadPage = () => {
  // 所有下载任务的响应式数组（会驱动 UI 列表渲染）。
  const downloads = ref<DownloadItem[]>([]);
  // 本地自增 ID，用于在前端区分条目。
  let downloadSeq = 1;
  // 轮询定时器句柄，避免重复创建。
  let downloadPoller: number | null = null;

  // 新增一个下载记录（立即显示在列表顶端）。
  const addDownload = (
    title: string,
    link: string,
    kind: DownloadItem["kind"],
    path?: string,
    payload?: Partial<DownloadItem>
  ) => {
    // 记录开始时间（ISO 字符串便于存储和排序）。
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

  // 将内部状态转成用户可读的中文文本。
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

  // 根据状态给文件名加后缀，提示用户当前状态。
  const downloadDisplayTitle = (item: DownloadItem) => {
    if (item.status === "completed") return item.title;
    if (item.status === "paused") return `${item.title}.paused`;
    if (item.status === "failed") return `${item.title}.failed`;
    return `${item.title}.downloading`;
  };

  // 判断“是否处于暂停态”（包括后端返回的 paused/stopped）。
  const isDownloadPaused = (item: DownloadItem) => {
    const state = item.state?.toLowerCase() ?? "";
    return item.status === "paused" || state.includes("paused") || state.includes("stopped");
  };

  // 终态：已完成或失败（终态不再轮询/操作）。
  const isDownloadTerminal = (item: DownloadItem) => item.status === "completed" || item.status === "failed";

  // 暂停单个下载（需要有 torrentId 且不是终态）。
  const handlePauseDownload = async (item: DownloadItem) => {
    if (item.torrentId == null || isDownloadTerminal(item) || isDownloadPaused(item)) return;
    try {
      await invoke("pause_torrent", { id: item.torrentId });
      item.status = "paused";
    } catch (err) {
      console.error("pause_torrent failed", err);
    }
  };

  // 恢复单个下载（必须处于暂停态）。
  const handleResumeDownload = async (item: DownloadItem) => {
    if (item.torrentId == null || !isDownloadPaused(item)) return;
    try {
      await invoke("resume_torrent", { id: item.torrentId });
      item.status = "started";
    } catch (err) {
      console.error("resume_torrent failed", err);
    }
  };

  // 删除下载：先通知后端删除任务，再从前端列表移除。
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

  // 是否存在“正在下载”的任务（用于 UI 显示全局按钮）。
  const hasActiveDownloads = computed(() =>
    downloads.value.some((item) => item.torrentId != null && !isDownloadTerminal(item) && !isDownloadPaused(item))
  );
  // 是否存在“已暂停”的任务。
  const hasPausedDownloads = computed(() =>
    downloads.value.some((item) => item.torrentId != null && isDownloadPaused(item))
  );
  // 合计速度（字符串转为 B/s 再求和）。
  const totalDownloadBps = computed(() => downloads.value.reduce((sum, item) => sum + parseSpeedToBps(item.downloadSpeed), 0));
  const totalUploadBps = computed(() => downloads.value.reduce((sum, item) => sum + parseSpeedToBps(item.uploadSpeed), 0));
  // 把合计速度格式化成可读文本。
  const totalDownloadSpeedLabel = computed(() => formatSpeed(totalDownloadBps.value));
  const totalUploadSpeedLabel = computed(() => formatSpeed(totalUploadBps.value));

  // 批量暂停：对所有“正在下载”的任务并发调用暂停。
  const handlePauseAllDownloads = async () => {
    const active = downloads.value.filter((item) => item.status === "started" && item.torrentId != null);
    await Promise.all(active.map((item) => handlePauseDownload(item)));
  };

  // 批量恢复：对所有“已暂停”的任务并发调用恢复。
  const handleResumeAllDownloads = async () => {
    const paused = downloads.value.filter((item) => item.status === "paused" && item.torrentId != null);
    await Promise.all(paused.map((item) => handleResumeDownload(item)));
  };

  // 轮询后端状态并更新本地列表。
  // 说明：这里会并发请求所有活动任务的状态。
  const refreshDownloadStatuses = async () => {
    const active = downloads.value.filter((item) => item.torrentId != null);
    if (!active.length) return;

    // 并发拉取每个任务的状态。
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

    // 是否需要触发“完成后移动文件”的收尾动作。
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
      // 完成但还没收尾时，标记需要执行 finalize。
      if (completed && !item.finalized && item.tempPath && item.finalPath) {
        shouldFinalize = true;
      }
      return next;
    });
    downloads.value = updated;

    // 对已完成但未收尾的任务，调用后端移动文件。
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

  // 点击“下载”时的主流程：
  // 1) 选择目录
  // 2) 调用后端开始下载
  // 3) 写入本地列表
  const handleDownloadClick = async (item: SearchResult, kind: DownloadItem["kind"], link?: string) => {
    if (!link) return;

    let path: string | undefined;
    try {
      // 弹出系统目录选择框。
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
      // 启动下载任务并拿到后端返回的任务信息。
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

  // 组件挂载时开启定时轮询。
  onMounted(() => {
    if (downloadPoller == null) {
      downloadPoller = window.setInterval(refreshDownloadStatuses, 1500);
    }
  });

  // 组件卸载时清理定时器，避免内存泄漏。
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
