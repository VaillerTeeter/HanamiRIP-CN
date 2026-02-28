/** 文件：useDownloadPage.ts | 用途：下载页面核心状态与操作逻辑 | 关键对象：useDownloadPage, PendingDownloadTask */
// 本行目的：引入 Vue 的响应式与生命周期工具以管理页面状态。
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
// 本行目的：引入 Tauri invoke 用于调用后端命令。
import { invoke } from "@tauri-apps/api/core";
// 本行目的：引入路径工具以构造默认保存路径。
import { downloadDir, join } from "@tauri-apps/api/path";
// 本行目的：引入保存对话框能力以让用户选择保存位置。
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
// 本行目的：引入文件系统能力以读写下载设置。
import { BaseDirectory, mkdir, readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
// 本行目的：引入下载条目类型以保证字段一致性。
import type { DownloadItem } from "../types/download";
// 本行目的：引入格式化工具以展示速度与字节大小。
import { formatBytes, formatSpeed, parseSpeedToBps } from "../../../shared/utils/format";
// 本行目的：引入搜索结果类型以限制下载入口的输入。
import type { SearchResult } from "../../search/types/search";

/** 类型：PendingDownloadTask | 用途：描述可恢复的下载任务快照 | 字段：任务标识、路径与大小信息 */
// 本行目的：声明可恢复下载任务的数据结构。
type PendingDownloadTask = {
    // 本行目的：标识任务的唯一 ID，用于恢复或丢弃。
    taskId: string;
    // 本行目的：保存原始下载 URL，用于再次启动。
    url: string;
    // 本行目的：可选名称，若缺失则回退为路径解析的名称。
    name?: string | null;
    // 本行目的：输出目录路径，用于恢复时定位最终位置。
    outputDir: string;
    // 本行目的：输出完整路径，用于恢复时保持用户选择。
    outputPath: string;
    // 本行目的：临时目录路径，用于断点下载的中间文件。
    tempFolder: string;
    // 本行目的：占位文件路径，用于保持目标位置占用。
    placeholderPath: string;
    // 本行目的：可选总大小，便于恢复时展示进度。
    totalBytes?: number | null;
    // 本行目的：创建时间字符串，便于排序或展示。
    createdAt: string;
};

// 变量：useDownloadPage | 含义：下载页面组合式函数入口 | 类型：(options: { dialogTitleForKind: (kind: DownloadItem["kind"]) => string; errorFallback?: () => string; errorInvalidMagnet?: () => string; errorInvalidTorrent?: () => string; }) => UseDownloadPageReturn | 作用域：模块级
/** 函数：useDownloadPage | 输入：下载对话框标题与错误文案提供者 | 输出：下载页面所需状态与操作集合 | 可能失败：文件读写、后端命令调用失败 */
export const useDownloadPage = (options: {
    // 本行目的：提供不同下载类型的对话框标题生成函数。
    dialogTitleForKind: (kind: DownloadItem["kind"]) => string;
    // 本行目的：提供通用错误兜底文案生成函数。
    errorFallback?: () => string;
    // 本行目的：提供磁力链接无效时的错误文案生成函数。
    errorInvalidMagnet?: () => string;
    // 本行目的：提供种子链接无效时的错误文案生成函数。
    errorInvalidTorrent?: () => string;
    // 本行目的：结束 options 类型定义。
}) => {
    // 变量：downloads | 含义：当前页面展示的下载条目列表 | 类型：Ref<DownloadItem[]> | 作用域：useDownloadPage 内部
    // 本行目的：初始化下载列表的响应式状态。
    const downloads = ref<DownloadItem[]>([]);

    // 变量：downloadSeq | 含义：用于生成本地下载条目自增 ID | 类型：number | 作用域：useDownloadPage 内部
    // 本行目的：初始化下载条目的自增序号。
    let downloadSeq = 1;
    // 变量：downloadPoller | 含义：下载状态轮询定时器 ID | 类型：number | null | 作用域：useDownloadPage 内部
    // 本行目的：初始化轮询定时器占位。
    let downloadPoller: number | null = null;

    // 变量：settingsPath | 含义：下载设置在 AppData 下的存储路径 | 类型：string | 作用域：useDownloadPage 内部
    // 本行目的：定义下载设置文件路径常量。
    const settingsPath = "hanamirip-cn/download-settings.json";
    // 变量：settingsStorageKey | 含义：本地缓存上次目录的 localStorage key | 类型：string | 作用域：useDownloadPage 内部
    // 本行目的：定义缓存下载目录的键名。
    const settingsStorageKey = "download.lastDir";

    // 变量：settingsLoaded | 含义：标记下载设置是否已加载 | 类型：boolean | 作用域：useDownloadPage 内部
    // 本行目的：初始化设置加载标记。
    let settingsLoaded = false;
    // 变量：lastDownloadDir | 含义：上次下载目录缓存 | 类型：string | null | 作用域：useDownloadPage 内部
    // 本行目的：初始化上次下载目录缓存值。
    let lastDownloadDir: string | null = null;

    // 变量：pendingTasks | 含义：待恢复的下载任务列表 | 类型：Ref<PendingDownloadTask[]> | 作用域：useDownloadPage 内部
    // 本行目的：初始化待恢复任务的响应式列表。
    const pendingTasks = ref<PendingDownloadTask[]>([]);
    // 变量：resumePromptVisible | 含义：是否显示“恢复下载”提示 | 类型：Ref<boolean> | 作用域：useDownloadPage 内部
    // 本行目的：初始化恢复提示的显示状态。
    const resumePromptVisible = ref(false);

    // 变量：addDownload | 含义：新增下载条目的辅助函数 | 类型：(title: string, link: string, kind: DownloadItem["kind"], path?: string, payload?: Partial<DownloadItem>) => void | 作用域：useDownloadPage 内部
    /** 函数：addDownload | 输入：标题、链接、类型与可选路径/扩展字段 | 输出：无（更新状态） | 可能失败：无（纯前端状态更新） */
    const addDownload = (title: string, link: string, kind: DownloadItem["kind"], path?: string, payload?: Partial<DownloadItem>) => {
        // 本行目的：记录开始时间用于展示与排序。
        const startedAt = new Date().toISOString();

        // 本行目的：将新条目插入列表头部以优先展示。
        downloads.value = [
            // 本行目的：构造新的下载条目对象。
            {
                // 本行目的：生成本地唯一 ID，便于前端追踪。
                id: downloadSeq++,
                // 本行目的：保存展示标题。
                title,
                // 本行目的：保存原始链接。
                link,
                // 本行目的：保存下载类型。
                kind,
                // 本行目的：保存开始时间。
                startedAt,
                // 本行目的：初始化状态为 started。
                status: "started",
                // 本行目的：保存用户选择的路径。
                path,
                // 本行目的：合并外部传入的补充字段。
                ...payload,
                // 本行目的：结束对象字面量。
            },
            // 本行目的：保留原有下载列表内容。
            ...downloads.value,
            // 本行目的：结束数组字面量。
        ];
        // 本行目的：结束 addDownload 函数体。
    };

    // 变量：sanitizeFileName | 含义：清理文件名的工具函数 | 类型：(value: string) => string | 作用域：useDownloadPage 内部
    /** 函数：sanitizeFileName | 输入：原始文件名字符串 | 输出：清理后的安全文件名 | 可能失败：无（纯字符串处理） */
    const sanitizeFileName = (value: string) => {
        // 本行目的：定义非法字符正则，用于替换为下划线。
        const invalid = /[<>:"/\\|?*]/g;
        // 本行目的：去除首尾空白并替换非法字符。
        const trimmed = value.trim().replace(invalid, "_");
        // 本行目的：去掉尾部点与空白，避免系统拒绝文件名。
        return trimmed.replace(/[.\s]+$/, "") || "download";
        // 本行目的：结束 sanitizeFileName 函数体。
    };

    // 变量：ensureSettingsLoaded | 含义：加载下载设置的异步函数 | 类型：() => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：ensureSettingsLoaded | 输入：无 | 输出：无（更新缓存） | 可能失败：读取设置文件失败 */
    const ensureSettingsLoaded = async () => {
        // 本行目的：若已加载过则直接返回，避免重复 IO。
        if (settingsLoaded) return;

        // 本行目的：标记为已加载，防止并发重复读取。
        settingsLoaded = true;

        // 本行目的：尝试从 localStorage 读取上次下载目录。
        const cached = localStorage.getItem(settingsStorageKey);

        // 本行目的：若缓存存在则直接使用并结束。
        if (cached) {
            // 本行目的：记录缓存目录到内存。
            lastDownloadDir = cached;
            // 本行目的：结束函数，避免继续读取文件。
            return;
        }

        // 本行目的：尝试读取磁盘上的设置文件。
        try {
            // 本行目的：读取设置文件内容字符串。
            const content = await readTextFile(settingsPath, { baseDir: BaseDirectory.AppData });
            // 本行目的：解析 JSON 内容为对象。
            const parsed = JSON.parse(content) as { lastDownloadDir?: string };

            // 本行目的：若存在目录字段则缓存。
            if (parsed.lastDownloadDir) {
                // 本行目的：记录目录到内存。
                lastDownloadDir = parsed.lastDownloadDir;
                // 本行目的：写入 localStorage 以加速后续读取。
                localStorage.setItem(settingsStorageKey, parsed.lastDownloadDir);
            }
        } catch (err) {
            // 本行目的：记录失败日志但不中断流程。
            console.debug("load download settings failed", err);
        }
        // 本行目的：结束 ensureSettingsLoaded 函数体。
    };

    // 变量：persistLastDownloadDir | 含义：持久化下载目录的异步函数 | 类型：(dir: string) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：persistLastDownloadDir | 输入：目录路径 | 输出：无（持久化缓存） | 可能失败：写文件失败 */
    const persistLastDownloadDir = async (dir: string) => {
        // 本行目的：更新内存中的上次目录。
        lastDownloadDir = dir;
        // 本行目的：同步写入 localStorage。
        localStorage.setItem(settingsStorageKey, dir);

        // 本行目的：尝试写入磁盘设置文件。
        try {
            // 本行目的：确保 AppData 目录存在。
            await mkdir("hanamirip-cn", { baseDir: BaseDirectory.AppData, recursive: true });
            // 本行目的：写入 JSON 设置文件。
            await writeTextFile(settingsPath, JSON.stringify({ lastDownloadDir: dir }), {
                // 本行目的：指定写入的基础目录。
                baseDir: BaseDirectory.AppData,
                // 本行目的：结束写入参数对象。
            });
        } catch (err) {
            // 本行目的：记录失败日志但不阻塞主流程。
            console.debug("save download settings failed", err);
        }
        // 本行目的：结束 persistLastDownloadDir 函数体。
    };

    // 变量：resolveOutputDir | 含义：从完整路径中解析目录的工具函数 | 类型：(selectedPath: string) => string | 作用域：useDownloadPage 内部
    /** 函数：resolveOutputDir | 输入：用户选择的完整路径 | 输出：目录路径 | 可能失败：无（字符串处理） */
    const resolveOutputDir = (selectedPath: string) => {
        // 本行目的：根据路径内容选择正确的分隔符。
        const separator = selectedPath.includes("\\") ? "\\" : "/";
        // 本行目的：找到最后一个分隔符位置。
        const lastIndex = selectedPath.lastIndexOf(separator);

        // 本行目的：若未找到有效分隔符则直接返回原路径。
        if (lastIndex <= 0) return selectedPath;

        // 本行目的：截取并返回目录部分。
        return selectedPath.slice(0, lastIndex);
        // 本行目的：结束 resolveOutputDir 函数体。
    };

    // 变量：displayPendingName | 含义：展示待恢复任务名称的工具函数 | 类型：(task: PendingDownloadTask) => string | 作用域：useDownloadPage 内部
    /** 函数：displayPendingName | 输入：待恢复任务 | 输出：用于显示的名称 | 可能失败：无（字符串处理） */
    const displayPendingName = (task: PendingDownloadTask) => {
        // 本行目的：若任务自带名称则直接返回。
        if (task.name) return task.name;

        // 本行目的：根据路径内容选择正确的分隔符。
        const separator = task.outputPath.includes("\\") ? "\\" : "/";
        // 本行目的：定位文件名在路径中的起点。
        const lastIndex = task.outputPath.lastIndexOf(separator);

        // 本行目的：返回路径末尾的文件名部分。
        return lastIndex >= 0 ? task.outputPath.slice(lastIndex + 1) : task.outputPath;
        // 本行目的：结束 displayPendingName 函数体。
    };

    // 变量：loadPendingTasks | 含义：加载待恢复任务列表的异步函数 | 类型：() => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：loadPendingTasks | 输入：无 | 输出：无（更新状态） | 可能失败：后端命令调用失败 */
    const loadPendingTasks = async () => {
        // 本行目的：尝试从后端获取待恢复任务。
        try {
            // 本行目的：调用后端命令并获取任务数组。
            const tasks = await invoke<PendingDownloadTask[]>("list_pending_downloads");

            // 本行目的：更新响应式任务列表。
            pendingTasks.value = tasks;

            // 本行目的：若存在待恢复任务则显示提示。
            if (tasks.length) {
                // 本行目的：打开恢复提示。
                resumePromptVisible.value = true;
            }
        } catch (err) {
            // 本行目的：记录后端调用失败日志。
            console.error("list_pending_downloads failed", err);
        }
        // 本行目的：结束 loadPendingTasks 函数体。
    };

    // 变量：removePendingTask | 含义：移除待恢复任务并更新提示 | 类型：(taskId: string) => void | 作用域：useDownloadPage 内部
    /** 函数：removePendingTask | 输入：任务 ID | 输出：无（更新状态） | 可能失败：无（纯前端状态更新） */
    const removePendingTask = (taskId: string) => {
        // 本行目的：过滤掉已处理的任务。
        pendingTasks.value = pendingTasks.value.filter((task) => task.taskId !== taskId);

        // 本行目的：若列表清空则关闭提示。
        if (!pendingTasks.value.length) {
            // 本行目的：隐藏恢复提示。
            resumePromptVisible.value = false;
        }
        // 本行目的：结束 removePendingTask 函数体。
    };

    // 变量：handleResumePending | 含义：恢复指定待恢复任务 | 类型：(task: PendingDownloadTask) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handleResumePending | 输入：待恢复任务 | 输出：无（更新状态并启动下载） | 可能失败：后端命令调用失败 */
    const handleResumePending = async (task: PendingDownloadTask) => {
        // 本行目的：尝试调用后端恢复下载。
        try {
            // 本行目的：调用后端并获取恢复后的下载信息。
            const started = await invoke<{
                // 本行目的：返回下载记录 ID。
                id: number;
                // 本行目的：返回任务 ID。
                taskId: string;
                // 本行目的：返回 torrent infoHash。
                infoHash: string;
                // 本行目的：返回可选名称。
                name?: string | null;
                // 本行目的：返回临时文件夹。
                outputFolder: string;
                // 本行目的：返回最终文件夹。
                finalFolder: string;
                // 本行目的：返回占位文件路径。
                placeholderPath: string;
                // 本行目的：结束响应类型定义。
            }>("resume_torrent_download", { taskId: task.taskId });
            // 本行目的：根据 URL 判断是磁力还是种子。
            const kind = task.url.startsWith("magnet:") ? "magnet" : "torrent";

            // 本行目的：将恢复任务加入下载列表。
            addDownload(task.name || displayPendingName(task), task.url, kind, task.outputPath, {
                // 本行目的：绑定后端下载 ID。
                torrentId: started.id,
                // 本行目的：绑定任务 ID。
                taskId: started.taskId,
                // 本行目的：绑定 infoHash。
                infoHash: started.infoHash,
                // 本行目的：保存临时目录路径。
                tempPath: started.outputFolder,
                // 本行目的：保存最终目录路径。
                finalPath: started.finalFolder,
                // 本行目的：保存占位文件路径。
                placeholderPath: started.placeholderPath,
                // 本行目的：保留任务总大小（若存在）。
                totalBytes: task.totalBytes ?? undefined,
                // 本行目的：结束补充字段对象。
            });

            // 本行目的：从待恢复列表移除已处理任务。
            removePendingTask(task.taskId);
        } catch (err) {
            // 本行目的：记录恢复失败日志。
            console.error("resume_torrent_download failed", err);
        }
        // 本行目的：结束 handleResumePending 函数体。
    };

    // 变量：handleDiscardPending | 含义：丢弃待恢复任务 | 类型：(task: PendingDownloadTask) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handleDiscardPending | 输入：待恢复任务 | 输出：无（更新状态并清理任务） | 可能失败：后端命令调用失败 */
    const handleDiscardPending = async (task: PendingDownloadTask) => {
        // 本行目的：尝试调用后端丢弃任务。
        try {
            // 本行目的：通知后端删除任务记录。
            await invoke("discard_torrent_download", { taskId: task.taskId });
            // 本行目的：从待恢复列表移除任务。
            removePendingTask(task.taskId);
        } catch (err) {
            // 本行目的：记录丢弃失败日志。
            console.error("discard_torrent_download failed", err);
        }
        // 本行目的：结束 handleDiscardPending 函数体。
    };

    // 变量：isDownloadPaused | 含义：判断下载是否处于暂停状态 | 类型：(item: DownloadItem) => boolean | 作用域：useDownloadPage 内部
    /** 函数：isDownloadPaused | 输入：下载条目 | 输出：是否暂停 | 可能失败：无（纯判断） */
    const isDownloadPaused = (item: DownloadItem) => {
        // 本行目的：将状态字符串转为小写，便于匹配。
        const state = item.state?.toLowerCase() ?? "";
        // 本行目的：根据前端状态与后端状态判断是否暂停。
        return item.status === "paused" || state.includes("paused") || state.includes("stopped");
        // 本行目的：结束 isDownloadPaused 函数体。
    };

    // 变量：isDownloadTerminal | 含义：判断下载是否终态（完成或失败） | 类型：(item: DownloadItem) => boolean | 作用域：useDownloadPage 内部
    /** 函数：isDownloadTerminal | 输入：下载条目 | 输出：是否终态 | 可能失败：无（纯判断） */
    const isDownloadTerminal = (item: DownloadItem) => item.status === "completed" || item.status === "failed";

    // 变量：isSpeedActive | 含义：判断速度是否应计入统计 | 类型：(item: DownloadItem) => boolean | 作用域：useDownloadPage 内部
    /** 函数：isSpeedActive | 输入：下载条目 | 输出：是否计入速度 | 可能失败：无（纯判断） */
    const isSpeedActive = (item: DownloadItem) => item.torrentId != null && !isDownloadTerminal(item) && !isDownloadPaused(item);

    // 变量：handlePauseDownload | 含义：暂停指定下载 | 类型：(item: DownloadItem) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handlePauseDownload | 输入：下载条目 | 输出：无（更新状态） | 可能失败：后端命令调用失败 */
    const handlePauseDownload = async (item: DownloadItem) => {
        // 本行目的：过滤无效状态，避免重复暂停或终态操作。
        if (item.torrentId == null || isDownloadTerminal(item) || isDownloadPaused(item)) return;

        // 本行目的：尝试调用后端暂停下载。
        try {
            // 本行目的：通知后端暂停指定任务。
            await invoke("pause_torrent", { id: item.torrentId });
            // 本行目的：更新前端状态为暂停。
            item.status = "paused";
        } catch (err) {
            // 本行目的：记录暂停失败日志。
            console.error("pause_torrent failed", err);
        }
        // 本行目的：结束 handlePauseDownload 函数体。
    };

    // 变量：handleResumeDownload | 含义：恢复指定下载 | 类型：(item: DownloadItem) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handleResumeDownload | 输入：下载条目 | 输出：无（更新状态） | 可能失败：后端命令调用失败 */
    const handleResumeDownload = async (item: DownloadItem) => {
        // 本行目的：只有暂停状态才允许恢复。
        if (item.torrentId == null || !isDownloadPaused(item)) return;

        // 本行目的：尝试调用后端恢复下载。
        try {
            // 本行目的：通知后端恢复指定任务。
            await invoke("resume_torrent", { id: item.torrentId });
            // 本行目的：更新前端状态为 started。
            item.status = "started";
        } catch (err) {
            // 本行目的：记录恢复失败日志。
            console.error("resume_torrent failed", err);
        }
        // 本行目的：结束 handleResumeDownload 函数体。
    };

    // 变量：handleDeleteDownload | 含义：删除指定下载及其任务 | 类型：(item: DownloadItem) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handleDeleteDownload | 输入：下载条目 | 输出：无（更新状态） | 可能失败：后端命令调用失败 */
    const handleDeleteDownload = async (item: DownloadItem) => {
        // 本行目的：若存在后端下载 ID，则先请求删除。
        if (item.torrentId != null) {
            // 本行目的：尝试调用后端删除下载。
            try {
                // 本行目的：通知后端删除指定任务。
                await invoke("delete_torrent", { id: item.torrentId });
            } catch (err) {
                // 本行目的：记录删除失败并终止后续流程。
                console.error("delete_torrent failed", err);
                // 本行目的：返回以避免前端状态与后端不一致。
                return;
            }
        }

        // 本行目的：若存在待恢复任务 ID，则请求丢弃。
        if (item.taskId) {
            // 本行目的：尝试调用后端丢弃任务。
            try {
                // 本行目的：通知后端删除待恢复任务记录。
                await invoke("discard_torrent_download", { taskId: item.taskId });
            } catch (err) {
                // 本行目的：记录丢弃失败日志。
                console.error("discard_torrent_download failed", err);
            }
        }

        // 本行目的：从前端列表中移除下载条目。
        downloads.value = downloads.value.filter((row) => row.id !== item.id);
        // 本行目的：结束 handleDeleteDownload 函数体。
    };

    // 变量：hasActiveDownloads | 含义：是否存在正在计速的下载 | 类型：ComputedRef<boolean> | 作用域：useDownloadPage 内部
    /** 函数：hasActiveDownloads | 输入：无 | 输出：是否存在活动下载 | 可能失败：无（纯计算） */
    const hasActiveDownloads = computed(() => downloads.value.some((item) => isSpeedActive(item)));

    // 变量：hasPausedDownloads | 含义：是否存在暂停下载 | 类型：ComputedRef<boolean> | 作用域：useDownloadPage 内部
    /** 函数：hasPausedDownloads | 输入：无 | 输出：是否存在暂停下载 | 可能失败：无（纯计算） */
    const hasPausedDownloads = computed(() => downloads.value.some((item) => item.torrentId != null && isDownloadPaused(item)));

    // 变量：totalDownloadBps | 含义：当前总下载速度（bps） | 类型：ComputedRef<number> | 作用域：useDownloadPage 内部
    /** 函数：totalDownloadBps | 输入：无 | 输出：总下载速度（bps） | 可能失败：无（纯计算） */
    const totalDownloadBps = computed(() =>
        // 本行目的：筛选活跃条目并累加速度（字符串转 bps）。
        downloads.value.filter((item) => isSpeedActive(item)).reduce((sum, item) => sum + parseSpeedToBps(item.downloadSpeed), 0)
        // 本行目的：结束 computed 回调。
    );

    // 变量：totalUploadBps | 含义：当前总上传速度（bps） | 类型：ComputedRef<number> | 作用域：useDownloadPage 内部
    /** 函数：totalUploadBps | 输入：无 | 输出：总上传速度（bps） | 可能失败：无（纯计算） */
    const totalUploadBps = computed(() =>
        // 本行目的：筛选活跃条目并累加速度（字符串转 bps）。
        downloads.value.filter((item) => isSpeedActive(item)).reduce((sum, item) => sum + parseSpeedToBps(item.uploadSpeed), 0)
        // 本行目的：结束 computed 回调。
    );

    // 变量：totalDownloadSpeedLabel | 含义：总下载速度的格式化文本 | 类型：ComputedRef<string> | 作用域：useDownloadPage 内部
    /** 函数：totalDownloadSpeedLabel | 输入：无 | 输出：格式化下载速度 | 可能失败：无（纯计算） */
    const totalDownloadSpeedLabel = computed(() => formatSpeed(totalDownloadBps.value));

    // 变量：totalUploadSpeedLabel | 含义：总上传速度的格式化文本 | 类型：ComputedRef<string> | 作用域：useDownloadPage 内部
    /** 函数：totalUploadSpeedLabel | 输入：无 | 输出：格式化上传速度 | 可能失败：无（纯计算） */
    const totalUploadSpeedLabel = computed(() => formatSpeed(totalUploadBps.value));

    // 变量：handlePauseAllDownloads | 含义：暂停所有可暂停的下载 | 类型：() => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handlePauseAllDownloads | 输入：无 | 输出：无（批量暂停） | 可能失败：后端命令调用失败 */
    const handlePauseAllDownloads = async () => {
        // 本行目的：筛选处于 started 的条目。
        const active = downloads.value.filter((item) => item.status === "started" && item.torrentId != null);
        // 本行目的：并行暂停所有活动下载。
        await Promise.all(active.map((item) => handlePauseDownload(item)));
        // 本行目的：结束 handlePauseAllDownloads 函数体。
    };

    // 变量：handleResumeAllDownloads | 含义：恢复所有可恢复的下载 | 类型：() => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handleResumeAllDownloads | 输入：无 | 输出：无（批量恢复） | 可能失败：后端命令调用失败 */
    const handleResumeAllDownloads = async () => {
        // 本行目的：筛选处于 paused 的条目。
        const paused = downloads.value.filter((item) => item.status === "paused" && item.torrentId != null);
        // 本行目的：并行恢复所有暂停下载。
        await Promise.all(paused.map((item) => handleResumeDownload(item)));
        // 本行目的：结束 handleResumeAllDownloads 函数体。
    };

    // 变量：refreshDownloadStatuses | 含义：轮询并刷新所有下载状态 | 类型：() => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：refreshDownloadStatuses | 输入：无 | 输出：无（更新状态） | 可能失败：后端命令调用失败 */
    const refreshDownloadStatuses = async () => {
        // 本行目的：筛选拥有 torrentId 的条目进行状态更新。
        const active = downloads.value.filter((item) => item.torrentId != null);

        // 本行目的：若无活跃条目则直接返回。
        if (!active.length) return;

        // 本行目的：并行请求所有下载状态。
        const updates = await Promise.all(
            // 本行目的：为每个条目构造状态查询任务。
            active.map(async (item) => {
                // 本行目的：尝试调用后端查询状态。
                try {
                    // 本行目的：请求并获取下载统计信息。
                    const stats = await invoke<{
                        // 本行目的：返回下载记录 ID。
                        id: number;
                        // 本行目的：返回状态字符串。
                        state: string;
                        // 本行目的：返回已完成字节数。
                        progressBytes: number;
                        // 本行目的：返回总字节数。
                        totalBytes: number;
                        // 本行目的：返回已上传字节数。
                        uploadedBytes: number;
                        // 本行目的：返回是否完成标志。
                        finished: boolean;
                        // 本行目的：返回错误信息（可选）。
                        error?: string | null;
                        // 本行目的：返回下载速度（可选）。
                        downloadSpeed?: string | null;
                        // 本行目的：返回上传速度（可选）。
                        uploadSpeed?: string | null;
                        // 本行目的：返回剩余时间（可选）。
                        timeRemaining?: string | null;
                        // 本行目的：结束响应类型定义。
                    }>("get_torrent_status", { id: item.torrentId });
                    // 本行目的：返回条目 ID 与状态结果。
                    return { id: item.id, stats };
                } catch (err) {
                    // 本行目的：记录状态查询失败日志。
                    console.error("get_torrent_status failed", err);
                    // 本行目的：返回空状态以保持流程继续。
                    return { id: item.id, stats: null };
                }
            })
            // 本行目的：结束 Promise.all 参数。
        );

        // 本行目的：根据后端状态构造新的下载列表。
        const updated = downloads.value.map((item) => {
            // 本行目的：找到对应条目的状态更新数据。
            const update = updates.find((u) => u.id === item.id)?.stats;

            // 本行目的：若无更新数据则保留原条目。
            if (!update) return item;

            // 本行目的：总大小取本地与后端较大值以避免回退。
            const totalBytes = item.totalBytes && item.totalBytes > 0 ? Math.max(item.totalBytes, update.totalBytes) : update.totalBytes;

            // 本行目的：判断是否完成（完成标记且进度达到总大小）。
            const completed = update.finished && totalBytes > 0 && update.progressBytes >= totalBytes;

            // 本行目的：根据错误字段判断失败状态。
            const failed = Boolean(update.error);
            // 本行目的：规范化状态字符串用于判断暂停。
            const stateLower = update.state?.toLowerCase() ?? "";
            // 本行目的：判断是否为暂停或停止。
            const paused = stateLower.includes("paused") || stateLower.includes("stopped");

            // 本行目的：完成时强制进度等于总大小。
            const progressBytes = completed ? totalBytes : update.progressBytes;

            // 本行目的：构造新的下载条目快照。
            const next: DownloadItem = {
                // 本行目的：保留原条目其他字段。
                ...item,
                // 本行目的：根据完成/失败/暂停/进行中更新状态。
                status: completed ? "completed" : failed ? "failed" : paused ? "paused" : "started",
                // 本行目的：保存后端状态字符串。
                state: update.state,
                // 本行目的：保存更新后的进度字节。
                progressBytes,
                // 本行目的：保存更新后的总大小。
                totalBytes,
                // 本行目的：保存上传字节数。
                uploadedBytes: update.uploadedBytes,
                // 本行目的：保存错误信息（无则置空）。
                error: update.error ?? undefined,
                // 本行目的：非活跃状态时清空下载速度。
                downloadSpeed: completed || failed || paused ? undefined : (update.downloadSpeed ?? undefined),
                // 本行目的：非活跃状态时清空上传速度。
                uploadSpeed: completed || failed || paused ? undefined : (update.uploadSpeed ?? undefined),
                // 本行目的：非活跃状态时清空剩余时间。
                timeRemaining: completed || failed || paused ? undefined : (update.timeRemaining ?? undefined),
                // 本行目的：结束条目对象。
            };

            // 本行目的：返回更新后的条目。
            return next;
        });

        // 本行目的：将更新后的列表写回响应式状态。
        downloads.value = updated;

        // 本行目的：遍历完成的下载以执行最终归档。
        for (const item of downloads.value) {
            // 本行目的：若未满足最终归档条件则跳过。
            if (item.status !== "completed" || item.finalized || !item.tempPath || !item.finalPath || !item.placeholderPath) continue;

            // 本行目的：尝试调用后端完成下载文件归档。
            try {
                // 本行目的：通知后端将临时文件移到最终路径。
                await invoke("finalize_torrent_download", {
                    // 本行目的：传递临时文件夹路径。
                    tempFolder: item.tempPath,
                    // 本行目的：传递最终目标路径。
                    finalPath: item.finalPath,
                    // 本行目的：传递占位文件路径。
                    placeholderPath: item.placeholderPath,
                    // 本行目的：结束参数对象。
                });
                // 本行目的：标记该条目已完成最终归档。
                item.finalized = true;
            } catch (err) {
                // 本行目的：记录最终归档失败日志。
                console.error("finalize_torrent_download failed", err);
            }
        }
        // 本行目的：结束 refreshDownloadStatuses 函数体。
    };

    // 变量：handleDownloadClick | 含义：处理用户点击下载入口 | 类型：(item: SearchResult, kind: DownloadItem["kind"], link?: string) => Promise<void> | 作用域：useDownloadPage 内部
    /** 函数：handleDownloadClick | 输入：搜索结果、下载类型与链接 | 输出：无（更新状态并启动下载） | 可能失败：对话框、后端命令调用失败 */
    const handleDownloadClick = async (item: SearchResult, kind: DownloadItem["kind"], link?: string) => {
        // 本行目的：若没有链接则直接返回。
        if (!link) return;

        // 本行目的：去除前后空白，避免格式判断失败。
        const normalizedLink = link.trim();

        // 本行目的：磁力链接必须以 magnet: 开头，否则直接失败。
        if (kind === "magnet" && !/^magnet:/i.test(normalizedLink)) {
            // 本行目的：新增失败条目，提示磁力链接无效。
            addDownload(item.title, normalizedLink, kind, undefined, {
                // 本行目的：标记下载为失败。
                status: "failed",
                // 本行目的：使用可配置文案拼接错误说明。
                error: `${options.errorInvalidMagnet?.() || "磁链格式无效"}：${normalizedLink}`,
                // 本行目的：结束失败条目补充字段。
            });
            // 本行目的：终止流程。
            return;
        }

        // 本行目的：种子类型但链接是磁力链接时直接失败。
        if (kind === "torrent" && /^magnet:/i.test(normalizedLink)) {
            // 本行目的：新增失败条目，提示种子链接无效。
            addDownload(item.title, normalizedLink, kind, undefined, {
                // 本行目的：标记下载为失败。
                status: "failed",
                // 本行目的：使用可配置文案拼接错误说明。
                error: `${options.errorInvalidTorrent?.() || "种子链接无效"}：${normalizedLink}`,
                // 本行目的：结束失败条目补充字段。
            });
            // 本行目的：终止流程。
            return;
        }

        // 本行目的：种子链接必须以 .torrent 结尾或带参数。
        if (kind === "torrent" && !/\.torrent(\?|#|$)/i.test(normalizedLink)) {
            // 本行目的：新增失败条目，提示种子链接无效。
            addDownload(item.title, normalizedLink, kind, undefined, {
                // 本行目的：标记下载为失败。
                status: "failed",
                // 本行目的：使用可配置文案拼接错误说明。
                error: `${options.errorInvalidTorrent?.() || "种子链接无效"}：${normalizedLink}`,
                // 本行目的：结束失败条目补充字段。
            });
            // 本行目的：终止流程。
            return;
        }

        // 变量：path | 含义：用户选择的下载路径 | 类型：string | undefined | 作用域：handleDownloadClick 内部
        // 本行目的：初始化下载路径占位。
        let path: string | undefined;
        // 变量：outputDir | 含义：下载输出目录 | 类型：string | undefined | 作用域：handleDownloadClick 内部
        // 本行目的：初始化输出目录占位。
        let outputDir: string | undefined;
        // 变量：totalBytes | 含义：下载总大小（若可获取） | 类型：number | undefined | 作用域：handleDownloadClick 内部
        // 本行目的：初始化总大小占位。
        let totalBytes: number | undefined;
        // 变量：defaultName | 含义：默认文件名候选 | 类型：string | 作用域：handleDownloadClick 内部
        // 本行目的：使用搜索结果标题作为初始名称。
        let defaultName = item.title;

        // 本行目的：尝试准备路径与元数据。
        try {
            // 本行目的：确保已加载下载目录设置。
            await ensureSettingsLoaded();

            // 本行目的：尝试从后端读取 torrent 元数据（不强制）。
            try {
                // 本行目的：调用后端获取 infoHash、名称与总大小。
                const metadata = await invoke<{
                    // 本行目的：返回 infoHash。
                    infoHash: string;
                    // 本行目的：返回可选名称。
                    name?: string | null;
                    // 本行目的：返回总大小。
                    totalBytes: number;
                    // 本行目的：结束响应类型定义。
                }>("get_torrent_metadata", { url: normalizedLink });

                // 本行目的：若总大小有效则记录。
                if (metadata.totalBytes > 0) {
                    // 本行目的：保存总大小用于进度显示。
                    totalBytes = metadata.totalBytes;
                }

                // 本行目的：若后端提供名称则覆盖默认名称。
                if (metadata.name) {
                    // 本行目的：更新默认名称。
                    defaultName = metadata.name;
                }
            } catch (err) {
                // 本行目的：记录元数据获取失败但继续流程。
                console.debug("get_torrent_metadata failed", err);
            }

            // 本行目的：将名称清理为安全文件名。
            const safeName = sanitizeFileName(defaultName);

            // 本行目的：对种子类型确保扩展名为 .torrent。
            const defaultFileName = kind === "torrent" && !safeName.endsWith(".torrent") ? `${safeName}.torrent` : safeName;

            // 本行目的：选择默认目录，优先使用上次目录。
            const defaultDir = lastDownloadDir ?? (await downloadDir());
            // 本行目的：拼接默认保存路径。
            const defaultPath = await join(defaultDir, defaultFileName);

            // 本行目的：弹出保存对话框让用户选择路径。
            const result = await saveDialog({
                // 本行目的：设置对话框标题。
                title: options.dialogTitleForKind(kind),
                // 本行目的：设置对话框默认路径。
                defaultPath,
                // 本行目的：结束对话框参数对象。
            });

            // 本行目的：若用户取消则终止流程。
            if (!result) return;

            // 本行目的：保存用户选择的路径。
            path = String(result);
            // 本行目的：解析并保存输出目录。
            outputDir = resolveOutputDir(path);

            // 本行目的：持久化用户选择的目录。
            await persistLastDownloadDir(outputDir);
        } catch (err) {
            // 本行目的：记录对话框或路径准备失败日志。
            console.error("saveDialog failed", err);
            // 本行目的：终止流程，避免继续启动下载。
            return;
        }

        // 本行目的：尝试启动下载任务。
        try {
            // 本行目的：调用后端启动下载并获取任务信息。
            const started = await invoke<{
                // 本行目的：返回下载记录 ID。
                id: number;
                // 本行目的：返回任务 ID。
                taskId: string;
                // 本行目的：返回 infoHash。
                infoHash: string;
                // 本行目的：返回可选名称。
                name?: string | null;
                // 本行目的：返回临时文件夹。
                outputFolder: string;
                // 本行目的：返回最终文件夹。
                finalFolder: string;
                // 本行目的：返回占位文件路径。
                placeholderPath: string;
                // 本行目的：结束响应类型定义。
            }>("start_torrent_download", {
                // 本行目的：传入下载链接。
                url: normalizedLink,
                // 本行目的：传入输出目录（兜底为路径所在目录）。
                outputDir: outputDir ?? path,
                // 本行目的：传入输出完整路径。
                outputPath: path,
                // 本行目的：传入总大小（若可用）。
                totalBytes,
                // 本行目的：结束启动参数对象。
            });

            // 本行目的：将新下载任务加入列表。
            addDownload(item.title, normalizedLink, kind, path, {
                // 本行目的：绑定后端下载 ID。
                torrentId: started.id,
                // 本行目的：绑定任务 ID。
                taskId: started.taskId,
                // 本行目的：绑定 infoHash。
                infoHash: started.infoHash,
                // 本行目的：保存临时目录路径。
                tempPath: started.outputFolder,
                // 本行目的：保存最终目录路径。
                finalPath: started.finalFolder,
                // 本行目的：保存占位文件路径。
                placeholderPath: started.placeholderPath,
                // 本行目的：保存总大小用于进度显示。
                totalBytes,
                // 本行目的：结束补充字段对象。
            });
        } catch (err: any) {
            // 本行目的：记录启动失败日志。
            console.error("start_torrent_download failed", err);

            // 本行目的：生成可读错误信息（优先使用后端消息）。
            const message = typeof err === "string" ? err : err?.message || err?.toString?.() || options.errorFallback?.() || "下载启动失败";

            // 本行目的：新增失败条目用于展示错误。
            addDownload(item.title, normalizedLink, kind, path, {
                // 本行目的：标记下载为失败。
                status: "failed",
                // 本行目的：拼接错误信息与链接。
                error: `${message} (${normalizedLink})`,
                // 本行目的：结束失败条目补充字段。
            });
        }
        // 本行目的：结束 handleDownloadClick 函数体。
    };

    // 本行目的：注册组件挂载后的逻辑。
    onMounted(() => {
        // 本行目的：若未启动轮询则创建定时器。
        if (downloadPoller == null) {
            // 本行目的：每 1500ms 刷新下载状态。
            downloadPoller = window.setInterval(refreshDownloadStatuses, 1500);
        }
        // 本行目的：加载待恢复任务（忽略返回值）。
        void loadPendingTasks();
        // 本行目的：结束 onMounted 回调。
    });

    // 本行目的：注册组件卸载前的清理逻辑。
    onBeforeUnmount(() => {
        // 本行目的：若轮询存在则清理定时器。
        if (downloadPoller != null) {
            // 本行目的：停止轮询以避免内存泄漏。
            window.clearInterval(downloadPoller);
            // 本行目的：清空轮询句柄。
            downloadPoller = null;
        }
        // 本行目的：结束 onBeforeUnmount 回调。
    });

    // 本行目的：返回下载页面所需的状态与操作集合。
    return {
        // 本行目的：暴露下载列表给 UI 使用。
        downloads,
        // 本行目的：暴露字节格式化工具。
        formatBytes,
        // 本行目的：暴露单个暂停操作。
        handlePauseDownload,
        // 本行目的：暴露单个恢复操作。
        handleResumeDownload,
        // 本行目的：暴露删除操作。
        handleDeleteDownload,
        // 本行目的：暴露批量暂停操作。
        handlePauseAllDownloads,
        // 本行目的：暴露批量恢复操作。
        handleResumeAllDownloads,
        // 本行目的：暴露点击下载入口。
        handleDownloadClick,
        // 本行目的：暴露暂停状态判断。
        isDownloadPaused,
        // 本行目的：暴露终态判断。
        isDownloadTerminal,
        // 本行目的：暴露是否存在活跃下载。
        hasActiveDownloads,
        // 本行目的：暴露是否存在暂停下载。
        hasPausedDownloads,
        // 本行目的：暴露格式化后的总下载速度。
        totalDownloadSpeedLabel,
        // 本行目的：暴露格式化后的总上传速度。
        totalUploadSpeedLabel,
        // 本行目的：暴露待恢复任务列表。
        pendingTasks,
        // 本行目的：暴露恢复提示显示状态。
        resumePromptVisible,
        // 本行目的：暴露待恢复任务名称展示函数。
        displayPendingName,
        // 本行目的：暴露恢复任务操作。
        handleResumePending,
        // 本行目的：暴露丢弃任务操作。
        handleDiscardPending,
        // 本行目的：结束返回对象。
    };
    // 本行目的：结束 useDownloadPage 函数体。
};

/** 类型：UseDownloadPageReturn | 用途：导出 useDownloadPage 的返回类型 | 字段：由 ReturnType 推断 */
// 本行目的：导出组合式函数返回类型，便于类型复用。
export type UseDownloadPageReturn = ReturnType<typeof useDownloadPage>;
