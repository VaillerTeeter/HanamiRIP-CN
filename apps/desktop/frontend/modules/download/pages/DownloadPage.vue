<script setup lang="ts">
/** 文件：DownloadPage.vue | 用途：展示下载列表、恢复任务与下载控制操作 | 关键对象：props, statusLabel, displayTitle, progressPercent */
// 本行目的：引入 Vue 的 computed，用于派生响应式只读状态。
import { computed } from "vue";
// 本行目的：引入国际化函数与语言类型，用于根据当前语言输出文案。
import { t, type LocaleKey } from "../../../shared/i18n/messages";
// 本行目的：引入下载页面组合式函数返回类型，约束组件接收的下载能力集合。
import type { UseDownloadPageReturn } from "../composables/useDownloadPage";
// 本行目的：引入下载条目类型，约束下载项字段访问。
import type { DownloadItem } from "../types/download";

// 变量：props | 含义：组件入参，包含当前语言与下载页状态操作集合 | 类型：{ locale: LocaleKey; downloads: UseDownloadPageReturn } | 作用域：组件内
// 本行目的：声明并读取组件 props 类型定义。
const props = defineProps<{
    // 本行目的：声明当前界面语言键。
    locale: LocaleKey;
    // 本行目的：声明下载页面组合逻辑对象。
    downloads: UseDownloadPageReturn;
    // 本行目的：结束 props 类型定义。
}>();

// 变量：statusLabel | 含义：把内部下载状态映射成可本地化显示文本 | 类型：(status: DownloadItem["status"]) => string | 作用域：组件内
/** 函数：statusLabel | 输入：下载状态值 | 输出：对应语言的状态文案 | 可能失败：i18n 键缺失会回退到默认行为 */
// 本行目的：定义状态文案转换函数。
const statusLabel = (status: DownloadItem["status"]) => {
    // 本行目的：完成态时返回“已完成”文案。
    if (status === "completed") return t("download.status.completed", props.locale);
    // 本行目的：失败态时返回“失败”文案。
    if (status === "failed") return t("download.status.failed", props.locale);
    // 本行目的：暂停态时返回“暂停”文案。
    if (status === "paused") return t("download.status.paused", props.locale);
    // 本行目的：其余状态统一按“下载中”文案处理。
    return t("download.status.started", props.locale);
    // 本行目的：结束 statusLabel 函数体。
};

// 变量：displayTitle | 含义：根据下载状态生成展示标题（带后缀） | 类型：(item: DownloadItem) => string | 作用域：组件内
/** 函数：displayTitle | 输入：下载条目对象 | 输出：用于列表展示的标题字符串 | 可能失败：无（纯字符串拼接） */
// 本行目的：定义下载标题展示规则。
const displayTitle = (item: DownloadItem) => {
    // 本行目的：完成态显示原始标题。
    if (item.status === "completed") return item.title;
    // 本行目的：暂停态追加 .paused 后缀。
    if (item.status === "paused") return `${item.title}.paused`;
    // 本行目的：失败态追加 .failed 后缀。
    if (item.status === "failed") return `${item.title}.failed`;
    // 本行目的：其余状态追加 .downloading 后缀。
    return `${item.title}.downloading`;
    // 本行目的：结束 displayTitle 函数体。
};

// 变量：resumePromptVisible | 含义：恢复任务弹窗是否可见 | 类型：ComputedRef<boolean> | 作用域：组件内
/** 函数：resumePromptVisible | 输入：无 | 输出：恢复弹窗显示状态 | 可能失败：无（纯派生） */
// 本行目的：从下载组合状态中派生恢复弹窗可见性。
const resumePromptVisible = computed(() => props.downloads.resumePromptVisible.value);
// 变量：pendingTasks | 含义：待恢复下载任务列表 | 类型：ComputedRef<PendingDownloadTask[]>（由推断得到） | 作用域：组件内
/** 函数：pendingTasks | 输入：无 | 输出：待恢复任务数组 | 可能失败：无（纯派生） */
// 本行目的：从下载组合状态中派生待恢复任务列表。
const pendingTasks = computed(() => props.downloads.pendingTasks.value);

// 变量：handleResumeTask | 含义：触发单个待恢复任务的恢复动作 | 类型：(task: (typeof pendingTasks.value)[number]) => void | 作用域：组件内
/** 函数：handleResumeTask | 输入：单个待恢复任务 | 输出：无（触发副作用） | 可能失败：底层恢复命令可能失败 */
// 本行目的：定义恢复任务按钮点击处理函数。
const handleResumeTask = (task: (typeof pendingTasks.value)[number]) => {
    // 本行目的：调用下载组合逻辑恢复任务，并显式忽略 Promise 返回值。
    void props.downloads.handleResumePending(task);
    // 本行目的：结束 handleResumeTask 函数体。
};

// 变量：handleDiscardTask | 含义：触发单个待恢复任务的丢弃动作 | 类型：(task: (typeof pendingTasks.value)[number]) => void | 作用域：组件内
/** 函数：handleDiscardTask | 输入：单个待恢复任务 | 输出：无（触发副作用） | 可能失败：底层丢弃命令可能失败 */
// 本行目的：定义丢弃任务按钮点击处理函数。
const handleDiscardTask = (task: (typeof pendingTasks.value)[number]) => {
    // 本行目的：调用下载组合逻辑丢弃任务，并显式忽略 Promise 返回值。
    void props.downloads.handleDiscardPending(task);
    // 本行目的：结束 handleDiscardTask 函数体。
};

// 变量：progressBytes | 含义：计算安全的已下载字节数（不超过总字节） | 类型：(item: DownloadItem) => number | 作用域：组件内
/** 函数：progressBytes | 输入：下载条目对象 | 输出：钳制后的已下载字节数 | 可能失败：无（纯数值计算） */
// 本行目的：定义下载进度字节数的安全计算函数。
const progressBytes = (item: DownloadItem) => Math.min(item.progressBytes ?? 0, item.totalBytes ?? 0);

// 变量：progressPercent | 含义：计算进度条百分比（0~1）并避免 100% 假完成 | 类型：(item: DownloadItem) => number | 作用域：组件内
/** 函数：progressPercent | 输入：下载条目对象 | 输出：进度百分比（0 到 1） | 可能失败：无（纯数值计算） */
// 本行目的：定义进度百分比计算函数。
const progressPercent = (item: DownloadItem) => {
    // 本行目的：无总大小时返回 0，避免除零。
    if (!item.totalBytes) return 0;
    // 本行目的：已完成时返回 1，保证进度条满格。
    if (item.status === "completed") return 1;
    // 本行目的：按已下载字节除以总字节得到进度比例。
    const fraction = progressBytes(item) / item.totalBytes;
    // 本行目的：将进度限制在 [0, 0.99]，避免未完成但显示 100%。
    return Math.min(0.99, Math.max(0, Number.isFinite(fraction) ? fraction : 0));
    // 本行目的：结束 progressPercent 函数体。
};
</script>

<template>
    <!-- 本行目的：下载页面最外层容器，复用应用通用 body 布局。 -->
    <div class="app-body download-view">
        <!-- 本行目的：使用卡片组件承载下载区域主内容与标题。 -->
        <a-card class="download-card" :title="t('download.title', props.locale)" size="small">
            <!-- 本行目的：工具栏容器，放置总速率与批量操作按钮。 -->
            <div class="download-toolbar">
                <!-- 本行目的：仅在存在下载项时显示总下载/上传速度。 -->
                <div v-if="props.downloads.downloads.value.length" class="download-total-speed">
                    <!-- 本行目的：显示总速率标签文案。 -->
                    <span class="download-total-label">{{ t("download.total.label", props.locale) }}</span>
                    <!-- 本行目的：显示总下载速度。 -->
                    <span class="pill">↓ {{ props.downloads.totalDownloadSpeedLabel.value }}</span>
                    <!-- 本行目的：显示总上传速度。 -->
                    <span class="pill">↑ {{ props.downloads.totalUploadSpeedLabel.value }}</span>
                </div>
                <!-- 本行目的：工具栏右侧按钮区域。 -->
                <div class="download-toolbar-actions">
                    <!-- 本行目的：批量暂停按钮，存在活动下载时可点击。 -->
                    <a-button
                        size="mini"
                        type="outline"
                        class="icon-button"
                        :disabled="!props.downloads.hasActiveDownloads.value"
                        :aria-label="t('download.action.pauseAll', props.locale)"
                        :title="t('download.action.pauseAll', props.locale)"
                        @click="props.downloads.handlePauseAllDownloads"
                    >
                        <!-- 本行目的：暂停图标（双竖线）。 -->
                        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                            <!-- 本行目的：绘制左侧竖线。 -->
                            <line x1="8" y1="6" x2="8" y2="18" />
                            <!-- 本行目的：绘制右侧竖线。 -->
                            <line x1="16" y1="6" x2="16" y2="18" />
                        </svg>
                    </a-button>
                    <!-- 本行目的：批量恢复按钮，存在暂停下载时可点击。 -->
                    <a-button
                        size="mini"
                        type="outline"
                        class="icon-button"
                        :disabled="!props.downloads.hasPausedDownloads.value"
                        :aria-label="t('download.action.resumeAll', props.locale)"
                        :title="t('download.action.resumeAll', props.locale)"
                        @click="props.downloads.handleResumeAllDownloads"
                    >
                        <!-- 本行目的：恢复图标（三角播放）。 -->
                        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                            <!-- 本行目的：绘制播放三角形。 -->
                            <polygon points="9 7 19 12 9 17" />
                        </svg>
                    </a-button>
                </div>
            </div>
            <!-- 本行目的：存在待恢复任务时展示恢复选择弹窗。 -->
            <a-modal v-if="resumePromptVisible" :visible="true" :title="t('download.resume.title', props.locale)" :footer="false">
                <!-- 本行目的：垂直布局容器，组织弹窗内部说明与列表。 -->
                <a-space direction="vertical" size="12" fill>
                    <!-- 本行目的：展示恢复弹窗副标题说明。 -->
                    <a-typography-text>
                        {{ t("download.resume.subtitle", props.locale) }}
                    </a-typography-text>
                    <!-- 本行目的：展示待恢复任务列表。 -->
                    <a-list :bordered="false" size="small">
                        <!-- 本行目的：遍历每个待恢复任务并渲染列表项。 -->
                        <a-list-item v-for="task in pendingTasks" :key="task.taskId">
                            <!-- 本行目的：单行任务容器，左信息右操作。 -->
                            <div class="download-resume-row">
                                <!-- 本行目的：任务主要信息区域。 -->
                                <div class="download-resume-info">
                                    <!-- 本行目的：显示任务名称（优先后端名称，兜底路径名）。 -->
                                    <div class="download-resume-title">
                                        {{ props.downloads.displayPendingName(task) }}
                                    </div>
                                    <!-- 本行目的：展示任务附加信息（路径、大小）。 -->
                                    <div class="download-resume-meta">
                                        <!-- 本行目的：显示恢复目标路径。 -->
                                        <span class="pill">{{ task.outputPath }}</span>
                                        <!-- 本行目的：总大小存在时显示格式化大小。 -->
                                        <span v-if="task.totalBytes" class="pill">
                                            {{ props.downloads.formatBytes(task.totalBytes) }}
                                        </span>
                                    </div>
                                </div>
                                <!-- 本行目的：恢复与丢弃按钮区域。 -->
                                <div class="download-resume-actions">
                                    <!-- 本行目的：点击后恢复该待恢复任务。 -->
                                    <a-button size="mini" type="primary" @click="handleResumeTask(task)">
                                        {{ t("download.resume.action.resume", props.locale) }}
                                    </a-button>
                                    <!-- 本行目的：点击后丢弃该待恢复任务。 -->
                                    <a-button size="mini" type="outline" @click="handleDiscardTask(task)">
                                        {{ t("download.resume.action.discard", props.locale) }}
                                    </a-button>
                                </div>
                            </div>
                        </a-list-item>
                    </a-list>
                </a-space>
            </a-modal>
            <!-- 本行目的：存在下载数据时渲染下载列表。 -->
            <div v-if="props.downloads.downloads.value.length" class="download-list">
                <!-- 本行目的：遍历每一条下载记录。 -->
                <div v-for="item in props.downloads.downloads.value" :key="item.id" class="download-row">
                    <!-- 本行目的：下载主信息区域（标题、标签、进度、错误）。 -->
                    <div class="download-main">
                        <!-- 本行目的：展示按状态加工后的下载标题。 -->
                        <div class="download-title">{{ displayTitle(item) }}</div>
                        <!-- 本行目的：下载元信息标签区域。 -->
                        <div class="download-meta">
                            <!-- 本行目的：显示下载类型（磁力或种子）。 -->
                            <span class="pill">
                                {{ item.kind === "magnet" ? t("download.kind.magnet", props.locale) : t("download.kind.torrent", props.locale) }}
                            </span>
                            <!-- 本行目的：显示统一状态文案。 -->
                            <span class="pill">{{ statusLabel(item.status) }}</span>
                            <!-- 本行目的：后端状态存在时显示原始状态字符串。 -->
                            <span v-if="item.state" class="pill">{{ item.state }}</span>
                            <!-- 本行目的：下载速度存在时显示下载速率。 -->
                            <span v-if="item.downloadSpeed" class="pill">↓ {{ item.downloadSpeed }}</span>
                            <!-- 本行目的：上传速度存在时显示上传速率。 -->
                            <span v-if="item.uploadSpeed" class="pill">↑ {{ item.uploadSpeed }}</span>
                            <!-- 本行目的：剩余时间存在时显示预计剩余。 -->
                            <span v-if="item.timeRemaining" class="pill"> {{ t("download.remaining", props.locale) }} {{ item.timeRemaining }} </span>
                            <!-- 本行目的：显示下载开始时间。 -->
                            <span class="pill">{{ item.startedAt }}</span>
                            <!-- 本行目的：路径存在时显示下载目标路径，并提供悬浮完整提示。 -->
                            <span v-if="item.path" class="pill path-pill" :title="item.path">{{ item.path }}</span>
                        </div>
                        <!-- 本行目的：总大小存在时显示进度条与字节进度文本。 -->
                        <div v-if="item.totalBytes" class="download-progress">
                            <!-- 本行目的：渲染线性进度条，百分比由 progressPercent 计算。 -->
                            <a-progress type="line" :percent="progressPercent(item)" :show-text="false" :stroke-width="8" />
                            <!-- 本行目的：显示“已下载 / 总大小”文本。 -->
                            <div class="progress-text">
                                {{ props.downloads.formatBytes(progressBytes(item)) }} / {{ props.downloads.formatBytes(item.totalBytes) }}
                            </div>
                        </div>
                        <!-- 本行目的：存在错误信息时展示错误文本。 -->
                        <div v-if="item.error" class="download-error">{{ item.error }}</div>
                    </div>
                    <!-- 本行目的：单条下载操作按钮区域。 -->
                    <div class="download-actions">
                        <!-- 本行目的：暂停按钮，满足条件时可点击。 -->
                        <a-button
                            size="mini"
                            type="text"
                            class="icon-button"
                            :disabled="item.torrentId == null || props.downloads.isDownloadTerminal(item) || props.downloads.isDownloadPaused(item)"
                            :aria-label="t('download.action.pause', props.locale)"
                            :title="t('download.action.pause', props.locale)"
                            @click="props.downloads.handlePauseDownload(item)"
                        >
                            <!-- 本行目的：暂停图标容器。 -->
                            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                                <!-- 本行目的：绘制左侧暂停竖线。 -->
                                <line x1="8" y1="6" x2="8" y2="18" />
                                <!-- 本行目的：绘制右侧暂停竖线。 -->
                                <line x1="16" y1="6" x2="16" y2="18" />
                            </svg>
                        </a-button>
                        <!-- 本行目的：恢复按钮，仅暂停状态可点击。 -->
                        <a-button
                            size="mini"
                            type="text"
                            class="icon-button"
                            :disabled="item.torrentId == null || !props.downloads.isDownloadPaused(item)"
                            :aria-label="t('download.action.resume', props.locale)"
                            :title="t('download.action.resume', props.locale)"
                            @click="props.downloads.handleResumeDownload(item)"
                        >
                            <!-- 本行目的：恢复图标容器。 -->
                            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                                <!-- 本行目的：绘制播放三角形。 -->
                                <polygon points="9 7 19 12 9 17" />
                            </svg>
                        </a-button>
                        <!-- 本行目的：删除按钮，触发删除下载任务。 -->
                        <a-button
                            size="mini"
                            type="text"
                            class="icon-button"
                            status="danger"
                            :aria-label="t('download.action.delete', props.locale)"
                            :title="t('download.action.delete', props.locale)"
                            @click="props.downloads.handleDeleteDownload(item)"
                        >
                            <!-- 本行目的：删除图标容器（垃圾桶）。 -->
                            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                                <!-- 本行目的：绘制垃圾桶上沿。 -->
                                <path d="M4 7h16" />
                                <!-- 本行目的：绘制左侧桶内竖线。 -->
                                <path d="M9 7v12" />
                                <!-- 本行目的：绘制右侧桶内竖线。 -->
                                <path d="M15 7v12" />
                                <!-- 本行目的：绘制桶身轮廓。 -->
                                <path d="M6 7l1 13h10l1-13" />
                                <!-- 本行目的：绘制桶盖形状。 -->
                                <path d="M9 4h6l1 3H8l1-3Z" />
                            </svg>
                        </a-button>
                    </div>
                </div>
            </div>
            <!-- 本行目的：无下载数据时显示空状态文案。 -->
            <p v-else class="download-empty">{{ t("download.empty", props.locale) }}</p>
        </a-card>
    </div>
</template>
