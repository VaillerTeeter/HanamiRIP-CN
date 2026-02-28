/** 文件：useTracksPage.ts | 用途：封装音视频轨道解析、混流任务入队与执行的页面状态逻辑 | 关键对象：useTracksPage, addTrackFile, detectTracks, enqueueMixTask, startMixQueue */
// 本行目的：引入 Vue 响应式能力，用于管理页面状态。
import { reactive, ref } from "vue";
// 本行目的：引入 Tauri invoke，用于调用后端解析轨道与执行混流。
import { invoke } from "@tauri-apps/api/core";
// 本行目的：引入系统文件打开/保存对话框能力。
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
// 本行目的：引入混流与轨道相关类型，确保状态结构类型安全。
import type { MixQueueItem, MixTrackInput, TrackFileResult, TrackInfo, TrackItem, TrackType } from "../types/tracks";

/** 类型：TracksPageOptions | 用途：集中注入 tracks 页面文案与选项构造函数 | 字段：文案函数与语言选项列表 */
// 本行目的：声明组合式函数外部依赖配置类型。
type TracksPageOptions = {
    // 本行目的：根据轨道类型返回显示标签。
    labelForType: (type: TrackType) => string;
    // 本行目的：返回文件选择弹窗标题。
    selectFileTitle: (type: TrackType) => string;
    // 本行目的：返回文件过滤器名称。
    selectFileFilterName: (type: TrackType) => string;
    // 本行目的：返回缺少文件时的错误文案。
    missingFileError: (type: TrackType) => string;
    // 本行目的：返回解析失败兜底文案。
    parseErrorFallback: () => string;
    // 本行目的：返回输出文件保存弹窗标题。
    outputDialogTitle: () => string;
    // 本行目的：返回输出文件过滤器名称。
    outputDialogFilterName: () => string;
    // 本行目的：返回输出文件后缀。
    outputSuffix: () => string;
    // 本行目的：返回视频轨缺失错误文案。
    mixVideoRequiredError: () => string;
    // 本行目的：返回入队成功文案。
    mixQueuedMessage: () => string;
    // 本行目的：返回队列为空文案。
    mixQueueEmptyMessage: () => string;
    // 本行目的：返回混流失败文案。
    mixFailedMessage: () => string;
    // 本行目的：返回可选轨道语言列表。
    trackLanguageOptions: Array<{ label: string; value: string }>;
};

// 变量：useTracksPage | 含义：tracks 页面组合式入口函数 | 类型：(options: TracksPageOptions) => UseTracksPageReturn | 作用域：模块级
/** 函数：useTracksPage | 输入：页面文案与语言选项配置 | 输出：tracks 页状态和操作方法集合 | 可能失败：内部 invoke/dialog 调用可能失败（局部已处理） */
// 本行目的：导出 tracks 页面核心状态管理逻辑。
export const useTracksPage = (options: TracksPageOptions) => {
    // 变量：trackFiles | 含义：按轨道类型维护当前选中文件列表 | 类型：Ref<Record<TrackType, TrackItem[]>> | 作用域：useTracksPage 内部
    // 本行目的：初始化每类轨道对应的文件列表。
    const trackFiles = ref<Record<TrackType, TrackItem[]>>({
        video: [],
        audio: [],
        subtitle: [],
    });

    // 变量：trackInfos | 含义：按轨道类型维护解析后的轨道信息 | 类型：Ref<Record<TrackType, TrackFileResult[]>> | 作用域：useTracksPage 内部
    // 本行目的：初始化每类轨道的解析结果容器。
    const trackInfos = ref<Record<TrackType, TrackFileResult[]>>({
        video: [],
        audio: [],
        subtitle: [],
    });

    // 变量：trackLoading | 含义：按轨道类型记录解析进行中状态 | 类型：Ref<Record<TrackType, boolean>> | 作用域：useTracksPage 内部
    // 本行目的：初始化每类轨道的加载状态。
    const trackLoading = ref<Record<TrackType, boolean>>({
        video: false,
        audio: false,
        subtitle: false,
    });

    // 变量：trackProgress | 含义：按轨道类型记录解析进度百分比 | 类型：Ref<Record<TrackType, number>> | 作用域：useTracksPage 内部
    // 本行目的：初始化每类轨道进度为0。
    const trackProgress = ref<Record<TrackType, number>>({
        video: 0,
        audio: 0,
        subtitle: 0,
    });

    // 变量：trackErrors | 含义：按轨道类型记录错误文案 | 类型：Ref<Record<TrackType, string>> | 作用域：useTracksPage 内部
    // 本行目的：初始化每类轨道错误信息为空。
    const trackErrors = ref<Record<TrackType, string>>({
        video: "",
        audio: "",
        subtitle: "",
    });

    // 变量：trackMixLoading | 含义：混流入队流程是否进行中 | 类型：Ref<boolean> | 作用域：useTracksPage 内部
    // 本行目的：初始化混流入队加载状态。
    const trackMixLoading = ref(false);
    // 变量：trackMixError | 含义：混流相关错误文案 | 类型：Ref<string> | 作用域：useTracksPage 内部
    // 本行目的：初始化混流错误为空。
    const trackMixError = ref("");
    // 变量：trackMixResult | 含义：混流操作结果提示文案 | 类型：Ref<string> | 作用域：useTracksPage 内部
    // 本行目的：初始化混流结果提示为空。
    const trackMixResult = ref("");
    // 变量：mixQueue | 含义：混流任务队列 | 类型：Ref<MixQueueItem[]> | 作用域：useTracksPage 内部
    // 本行目的：初始化空的混流队列。
    const mixQueue = ref<MixQueueItem[]>([]);
    // 变量：mixQueueRunning | 含义：队列执行是否进行中 | 类型：Ref<boolean> | 作用域：useTracksPage 内部
    // 本行目的：初始化队列执行状态。
    const mixQueueRunning = ref(false);
    // 变量：mixQueueDetailVisible | 含义：任务详情弹窗显示状态 | 类型：Ref<boolean> | 作用域：useTracksPage 内部
    // 本行目的：初始化详情弹窗为隐藏。
    const mixQueueDetailVisible = ref(false);
    // 变量：selectedMixTask | 含义：当前选中的任务详情对象 | 类型：Ref<MixQueueItem | null> | 作用域：useTracksPage 内部
    // 本行目的：初始化选中任务为空。
    const selectedMixTask = ref<MixQueueItem | null>(null);

    // 变量：trackSeq | 含义：轨道文件本地自增ID计数器 | 类型：number | 作用域：useTracksPage 内部
    // 本行目的：初始化轨道文件计数器。
    let trackSeq = 1;
    // 变量：mixQueueSeq | 含义：混流队列任务自增ID计数器 | 类型：number | 作用域：useTracksPage 内部
    // 本行目的：初始化混流任务计数器。
    let mixQueueSeq = 1;

    // 变量：trackLangDefaults | 含义：各轨道类型默认语言码 | 类型：Record<TrackType, string>（reactive） | 作用域：useTracksPage 内部
    // 本行目的：初始化轨道默认语言配置。
    const trackLangDefaults = reactive<Record<TrackType, string>>({
        video: "ja",
        audio: "ja",
        subtitle: "zh-Hans",
    });

    // 变量：trackLanguageOptions | 含义：轨道语言可选项列表 | 类型：Array<{label:string;value:string}> | 作用域：useTracksPage 内部
    // 本行目的：从外部配置读取语言选项。
    const trackLanguageOptions = options.trackLanguageOptions;

    // 变量：addTrackFile | 含义：按轨道类型选择单个媒体文件并写入状态 | 类型：(type: TrackType) => Promise<void> | 作用域：useTracksPage 内部
    /** 函数：addTrackFile | 输入：轨道类型（video/audio/subtitle） | 输出：无（更新 trackFiles 等状态） | 可能失败：文件对话框/文件大小读取失败 */
    // 本行目的：定义文件选择与基础信息初始化流程。
    const addTrackFile = async (type: TrackType) => {
        try {
            // 变量：videoExt | 含义：视频及音频可选扩展名集合 | 类型：string[] | 作用域：addTrackFile 内部
            // 本行目的：定义视频类文件扩展名白名单。
            const videoExt = ["mkv", "mp4", "avi", "mov", "ts", "m2ts", "webm", "mpg", "mpeg"];
            // 变量：subtitleExt | 含义：字幕可选扩展名集合 | 类型：string[] | 作用域：addTrackFile 内部
            // 本行目的：定义字幕类文件扩展名白名单。
            const subtitleExt = ["srt", "ass", "ssa", "vtt", "sup", "sub"];

            // 变量：extMap | 含义：轨道类型到扩展名列表映射 | 类型：Record<TrackType, string[]> | 作用域：addTrackFile 内部
            // 本行目的：按轨道类型选择允许的文件扩展名。
            const extMap: Record<TrackType, string[]> = {
                video: videoExt,
                audio: videoExt,
                subtitle: [...videoExt, ...subtitleExt],
            };

            // 本行目的：打开文件选择对话框。
            const result = await openDialog({
                title: options.selectFileTitle(type),
                directory: false,
                multiple: false,
                filters: [{ name: options.selectFileFilterName(type), extensions: extMap[type] }],
            });

            // 本行目的：未选择或返回多选数组时直接结束。
            if (!result || Array.isArray(result)) return;

            // 变量：file | 含义：用户选择的文件绝对路径 | 类型：string | 作用域：addTrackFile 内部
            // 本行目的：把返回值转成字符串路径。
            const file = String(result);

            // 变量：fileSize | 含义：文件大小文本（可选） | 类型：string | undefined | 作用域：addTrackFile 内部
            // 本行目的：初始化文件大小字段。
            let fileSize: string | undefined;

            try {
                // 本行目的：调用后端读取媒体文件大小。
                const size = await invoke<string | null>("get_media_file_size", { path: file });
                // 本行目的：把 null 归一化为 undefined。
                fileSize = size ?? undefined;
            } catch (err) {
                // 本行目的：记录读取文件大小失败日志。
                console.error("get_media_file_size failed", err);
                // 本行目的：读取失败时保持文件大小为空。
                fileSize = undefined;
            }

            // 本行目的：写入当前类型的单文件列表（覆盖旧值）。
            trackFiles.value[type] = [
                {
                    // 本行目的：分配本地自增ID。
                    id: trackSeq++,
                    // 本行目的：提取文件名，失败时回退完整路径。
                    name: file.split(/[\\/]/).filter(Boolean).pop() || file,
                    // 本行目的：保存文件绝对路径。
                    path: file,
                    // 本行目的：保存文件大小文本。
                    fileSize,
                },
            ];

            // 本行目的：选择新文件后清空该类型旧解析结果。
            trackInfos.value[type] = [];
            // 本行目的：清空该类型错误状态。
            trackErrors.value[type] = "";
            // 本行目的：重置该类型进度为0。
            trackProgress.value[type] = 0;
        } catch (err) {
            // 本行目的：记录打开文件对话框失败日志。
            console.error("openDialog failed", err);
        }
    };

    // 变量：detectTracks | 含义：解析指定类型文件中的轨道信息 | 类型：(type: TrackType) => Promise<void> | 作用域：useTracksPage 内部
    /** 函数：detectTracks | 输入：轨道类型 | 输出：无（更新解析结果/错误/进度） | 可能失败：后端解析失败或文件缺失 */
    // 本行目的：定义轨道解析主流程。
    const detectTracks = async (type: TrackType) => {
        // 本行目的：同类型正在解析时直接跳过，防止重复请求。
        if (trackLoading.value[type]) return;

        // 本行目的：未选择文件时写入错误并结束。
        if (!trackFiles.value[type].length) {
            trackErrors.value[type] = options.missingFileError(type);
            trackInfos.value[type] = [];
            trackProgress.value[type] = 0;
            return;
        }

        // 本行目的：初始化本次解析状态。
        trackLoading.value[type] = true;
        trackProgress.value[type] = 0;
        trackInfos.value[type] = [];
        trackErrors.value[type] = "";

        // 变量：files | 含义：当前类型待解析文件列表 | 类型：TrackItem[] | 作用域：detectTracks 内部
        // 本行目的：读取当前类型文件列表快照。
        const files = trackFiles.value[type];
        // 变量：totalFiles | 含义：待解析文件总数 | 类型：number | 作用域：detectTracks 内部
        // 本行目的：记录总数用于计算进度。
        const totalFiles = files.length;
        // 变量：results | 含义：解析结果收集数组 | 类型：TrackFileResult[] | 作用域：detectTracks 内部
        // 本行目的：初始化解析结果容器。
        const results: TrackFileResult[] = [];

        // 本行目的：逐文件顺序解析轨道信息。
        for (let i = 0; i < files.length; i += 1) {
            // 变量：file | 含义：当前迭代文件对象 | 类型：TrackItem | 作用域：for 循环内
            // 本行目的：读取当前处理文件。
            const file = files[i];

            try {
                // 本行目的：调用后端解析媒体轨道。
                const response = await invoke<{ tracks: TrackInfo[] }>("parse_media_tracks", {
                    path: file.path,
                    kind: type,
                });

                // 变量：tracks | 含义：增强后的轨道列表（含默认选中与语言覆盖） | 类型：Array<TrackInfo & {selected:boolean;langOverride:string}> | 作用域：detectTracks 内部
                // 本行目的：为轨道补充前端选择状态与默认语言。
                const tracks = (response.tracks || []).map((track, index) => ({
                    ...track,
                    selected: index === 0,
                    langOverride: track.lang || trackLangDefaults[type],
                }));

                // 本行目的：把当前文件解析结果加入结果数组。
                results.push({ file, tracks });
            } catch (err: any) {
                // 本行目的：写入可读错误信息。
                trackErrors.value[type] = typeof err === "string" ? err : err?.message || options.parseErrorFallback();
                // 本行目的：解析失败时仍保留该文件占位结果。
                results.push({ file, tracks: [] });
            } finally {
                // 本行目的：按完成比例更新解析进度。
                trackProgress.value[type] = Math.round(((i + 1) / totalFiles) * 100);
            }
        }

        // 本行目的：写回解析结果并关闭加载状态。
        trackInfos.value[type] = results;
        trackLoading.value[type] = false;
    };

    // 变量：pickOutputPath | 含义：弹出保存对话框并返回最终输出路径 | 类型：(baseFile?: TrackItem) => Promise<string | null> | 作用域：useTracksPage 内部
    /** 函数：pickOutputPath | 输入：可选基础文件对象 | 输出：输出路径或 null | 可能失败：保存对话框失败/用户取消 */
    // 本行目的：定义混流输出路径选择逻辑。
    const pickOutputPath = async (baseFile?: TrackItem) => {
        // 变量：baseName | 含义：输出文件基名（去扩展名） | 类型：string | 作用域：pickOutputPath 内部
        // 本行目的：从基础文件名推导输出文件基名。
        const baseName = baseFile?.name ? baseFile.name.replace(/\.[^/.\\]+$/, "") : "mixed";
        // 变量：suffix | 含义：输出文件后缀文本（来自配置） | 类型：string | 作用域：pickOutputPath 内部
        // 本行目的：读取输出后缀配置。
        const suffix = options.outputSuffix();
        // 变量：dir | 含义：基础文件目录路径 | 类型：string | 作用域：pickOutputPath 内部
        // 本行目的：提取基础文件所在目录。
        const dir = baseFile?.path ? baseFile.path.replace(/[\\/][^\\/]+$/, "") : "";
        // 变量：defaultPath | 含义：保存对话框默认输出路径 | 类型：string | 作用域：pickOutputPath 内部
        // 本行目的：拼接默认输出路径。
        const defaultPath = dir ? `${dir}\\${baseName}${suffix}.mkv` : `${baseName}${suffix}.mkv`;

        // 本行目的：打开保存对话框让用户确认输出路径。
        const result = await saveDialog({
            title: options.outputDialogTitle(),
            defaultPath,
            filters: [{ name: options.outputDialogFilterName(), extensions: ["mkv"] }],
        });

        // 本行目的：用户取消时返回 null。
        if (!result) return null;

        // 本行目的：确保输出路径以 .mkv 结尾。
        return result.endsWith(".mkv") ? result : `${result}.mkv`;
    };

    // 变量：collectMixInput | 含义：收集某一类型轨道的混流输入配置 | 类型：(type: TrackType) => MixTrackInput | null | 作用域：useTracksPage 内部
    /** 函数：collectMixInput | 输入：轨道类型 | 输出：混流输入对象或 null | 可能失败：无可用文件/轨道未选中时返回 null */
    // 本行目的：从当前选择状态提取后端混流所需输入参数。
    const collectMixInput = (type: TrackType): MixTrackInput | null => {
        // 变量：files | 含义：当前类型文件列表 | 类型：TrackItem[] | 作用域：collectMixInput 内部
        // 本行目的：读取指定类型的文件列表。
        const files = trackFiles.value[type];
        // 本行目的：无文件时无法构建输入。
        if (!files.length) return null;

        // 变量：file | 含义：当前类型首个文件 | 类型：TrackItem | 作用域：collectMixInput 内部
        // 本行目的：当前逻辑只使用该类型第一个文件。
        const file = files[0];

        // 变量：group | 含义：该文件对应的解析结果组 | 类型：TrackFileResult | undefined | 作用域：collectMixInput 内部
        // 本行目的：查找该文件对应的轨道信息组。
        const group = trackInfos.value[type].find((item) => item.file.id === file.id);
        // 本行目的：无解析结果或轨道为空时返回 null。
        if (!group || !group.tracks.length) return null;

        // 变量：selected | 含义：被选中的轨道ID数组 | 类型：string[] | 作用域：collectMixInput 内部
        // 本行目的：收集未被显式取消的轨道ID。
        const selected = group.tracks.filter((track) => track.selected !== false).map((track) => track.trackId);
        // 本行目的：无选中轨道时返回 null。
        if (!selected.length) return null;

        // 变量：trackLangs | 含义：轨道ID到语言码映射 | 类型：Record<string, string> | 作用域：collectMixInput 内部
        // 本行目的：初始化轨道语言映射对象。
        const trackLangs: Record<string, string> = {};

        // 本行目的：遍历轨道并为已选轨道写入默认语言。
        group.tracks.forEach((track) => {
            // 本行目的：跳过未选中的轨道。
            if (track.selected === false) return;

            // 变量：lang | 含义：当前类型默认语言码 | 类型：string | 作用域：forEach 回调内
            // 本行目的：读取该类型默认语言。
            const lang = trackLangDefaults[type];
            if (lang) {
                // 本行目的：记录轨道ID对应语言。
                trackLangs[track.trackId] = lang;
            }
        });

        // 本行目的：返回混流输入对象。
        return { path: file.path, kind: type, trackIds: selected, trackLangs };
    };

    // 变量：enqueueMixTask | 含义：构建并入队一个混流任务 | 类型：() => Promise<void> | 作用域：useTracksPage 内部
    /** 函数：enqueueMixTask | 输入：无（读取当前页面状态） | 输出：无（更新队列与提示） | 可能失败：输出路径选择失败/输入校验失败/异常抛出 */
    // 本行目的：定义混流任务入队流程。
    const enqueueMixTask = async () => {
        // 本行目的：入队处理中时阻止重复提交。
        if (trackMixLoading.value) return;

        // 本行目的：清空历史错误与结果提示。
        trackMixError.value = "";
        trackMixResult.value = "";

        // 本行目的：收集视频输入。
        const videoInput = collectMixInput("video");
        // 本行目的：收集音频输入。
        const audioInput = collectMixInput("audio");
        // 本行目的：收集字幕输入。
        const subtitleInput = collectMixInput("subtitle");

        // 本行目的：视频输入缺失时给出错误并终止。
        if (!videoInput) {
            trackMixError.value = options.mixVideoRequiredError();
            return;
        }

        // 本行目的：开启入队加载状态。
        trackMixLoading.value = true;

        try {
            // 本行目的：选择输出文件路径。
            const outputPath = await pickOutputPath(trackFiles.value.video[0]);
            // 本行目的：用户取消输出路径时直接结束。
            if (!outputPath) return;

            // 变量：inputs | 含义：本次混流输入数组（至少含视频） | 类型：MixTrackInput[] | 作用域：enqueueMixTask 内部
            // 本行目的：初始化输入数组并加入视频输入。
            const inputs: MixTrackInput[] = [videoInput];

            // 本行目的：有音频输入时追加。
            if (audioInput) inputs.push(audioInput);
            // 本行目的：有字幕输入时追加。
            if (subtitleInput) inputs.push(subtitleInput);

            // 本行目的：向队列追加一个新的 queued 任务。
            mixQueue.value.push({
                id: mixQueueSeq++,
                createdAt: new Date().toLocaleString(),
                outputPath,
                inputs,
                status: "queued",
            });

            // 本行目的：入队后清空文件与解析状态，准备下一次选择。
            trackFiles.value = { video: [], audio: [], subtitle: [] };
            trackInfos.value = { video: [], audio: [], subtitle: [] };
            trackErrors.value = { video: "", audio: "", subtitle: "" };
            trackProgress.value = { video: 0, audio: 0, subtitle: 0 };
            // 本行目的：显示入队成功提示。
            trackMixResult.value = options.mixQueuedMessage();
        } catch (err: any) {
            // 本行目的：写入可读错误信息。
            trackMixError.value = typeof err === "string" ? err : err?.message || options.mixFailedMessage();
        } finally {
            // 本行目的：关闭入队加载状态。
            trackMixLoading.value = false;
        }
    };

    // 变量：startMixQueue | 含义：按队列顺序执行所有待处理混流任务 | 类型：() => Promise<void> | 作用域：useTracksPage 内部
    /** 函数：startMixQueue | 输入：无（读取 mixQueue） | 输出：无（更新任务状态与提示） | 可能失败：后端混流命令失败时标记单任务 failed */
    // 本行目的：定义混流队列执行流程。
    const startMixQueue = async () => {
        // 本行目的：队列已在运行时避免重复启动。
        if (mixQueueRunning.value) return;

        // 本行目的：清空历史错误和结果提示。
        trackMixError.value = "";
        trackMixResult.value = "";

        // 变量：pending | 含义：是否存在 queued 状态任务 | 类型：boolean | 作用域：startMixQueue 内部
        // 本行目的：检查队列中是否有待执行任务。
        const pending = mixQueue.value.some((item) => item.status === "queued");

        // 本行目的：没有待执行任务时给出提示并结束。
        if (!pending) {
            trackMixResult.value = options.mixQueueEmptyMessage();
            return;
        }

        // 本行目的：标记队列进入运行状态。
        mixQueueRunning.value = true;

        // 本行目的：按顺序遍历任务队列执行混流。
        for (const item of mixQueue.value) {
            // 本行目的：仅处理 queued 状态任务。
            if (item.status !== "queued") continue;

            // 本行目的：切换任务状态为运行中。
            item.status = "running";
            // 本行目的：清空旧消息。
            item.message = undefined;

            try {
                // 本行目的：调用后端执行轨道混流。
                const output = await invoke<string>("mix_media_tracks", {
                    // 本行目的：转换输入结构为后端期望格式。
                    inputs: item.inputs.map((input) => ({
                        path: input.path,
                        kind: input.kind,
                        trackIds: input.trackIds,
                        trackLangs: input.trackLangs,
                    })),
                    // 本行目的：传入输出路径。
                    outputPath: item.outputPath,
                });

                // 本行目的：执行成功后更新任务状态与消息。
                item.status = "success";
                item.message = output;
            } catch (err: any) {
                // 本行目的：执行失败后更新任务状态与错误消息。
                item.status = "failed";
                item.message = typeof err === "string" ? err : err?.message || options.mixFailedMessage();
            }
        }
        // 本行目的：全部任务处理结束，关闭运行状态。
        mixQueueRunning.value = false;
    };

    // 变量：clearMixQueue | 含义：清空混流队列和结果提示 | 类型：() => void | 作用域：useTracksPage 内部
    /** 函数：clearMixQueue | 输入：无 | 输出：无（清空队列状态） | 可能失败：无 */
    // 本行目的：定义队列清空逻辑。
    const clearMixQueue = () => {
        // 本行目的：队列运行中禁止清空，避免状态冲突。
        if (mixQueueRunning.value) return;

        // 本行目的：清空任务队列。
        mixQueue.value = [];
        // 本行目的：清空结果提示。
        trackMixResult.value = "";
        // 本行目的：清空错误提示。
        trackMixError.value = "";
    };

    // 变量：openMixTaskDetail | 含义：打开指定任务的详情弹窗 | 类型：(item: MixQueueItem) => void | 作用域：useTracksPage 内部
    /** 函数：openMixTaskDetail | 输入：队列任务对象 | 输出：无（更新详情弹窗状态） | 可能失败：无 */
    // 本行目的：定义详情弹窗打开行为。
    const openMixTaskDetail = (item: MixQueueItem) => {
        // 本行目的：设置当前选中任务。
        selectedMixTask.value = item;
        // 本行目的：显示详情弹窗。
        mixQueueDetailVisible.value = true;
    };

    // 本行目的：返回 tracks 页面需要的全部状态和操作方法。
    return {
        // 本行目的：导出文件列表状态。
        trackFiles,
        // 本行目的：导出轨道解析结果。
        trackInfos,
        // 本行目的：导出解析加载状态。
        trackLoading,
        // 本行目的：导出解析进度。
        trackProgress,
        // 本行目的：导出解析错误信息。
        trackErrors,
        // 本行目的：导出入队加载状态。
        trackMixLoading,
        // 本行目的：导出混流错误信息。
        trackMixError,
        // 本行目的：导出混流结果提示。
        trackMixResult,
        // 本行目的：导出混流任务队列。
        mixQueue,
        // 本行目的：导出队列运行状态。
        mixQueueRunning,
        // 本行目的：导出详情弹窗显示状态。
        mixQueueDetailVisible,
        // 本行目的：导出当前选中任务。
        selectedMixTask,
        // 本行目的：导出默认语言配置。
        trackLangDefaults,
        // 本行目的：导出语言选项列表。
        trackLanguageOptions,
        // 本行目的：导出添加文件方法。
        addTrackFile,
        // 本行目的：导出轨道解析方法。
        detectTracks,
        // 本行目的：导出入队方法。
        enqueueMixTask,
        // 本行目的：导出队列执行方法。
        startMixQueue,
        // 本行目的：导出队列清空方法。
        clearMixQueue,
        // 本行目的：导出详情弹窗打开方法。
        openMixTaskDetail,
        // 本行目的：导出类型标签生成函数。
        labelForType: options.labelForType,
    };
};

/** 类型：UseTracksPageReturn | 用途：导出 useTracksPage 返回值类型 | 字段：由 ReturnType 自动推断 */
// 本行目的：导出组合式返回类型供页面与组件复用。
export type UseTracksPageReturn = ReturnType<typeof useTracksPage>;
