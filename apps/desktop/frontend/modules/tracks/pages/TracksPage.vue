<script setup lang="ts">
/** 文件：TracksPage.vue | 用途：提供轨道文件选择、轨道解析、混流任务入队与队列管理界面 | 关键对象：props, statusLabel, yesNoLabel, MixQueueDetailModal */
// 本行目的：引入国际化函数与语言键类型。
import { t, type LocaleKey } from "../../../shared/i18n/messages";
// 本行目的：引入 tracks 页面组合式返回类型。
import type { UseTracksPageReturn } from "../composables/useTracksPage";
// 本行目的：引入混流队列状态类型。
import type { MixQueueStatus } from "../types/tracks";
// 本行目的：引入混流任务详情弹窗组件。
import MixQueueDetailModal from "../components/MixQueueDetailModal.vue";

// 变量：props | 含义：父组件传入的语言和 tracks 页面状态集合 | 类型：由 defineProps 推断的对象 | 作用域：TracksPage 组件内
/** 函数：defineProps | 输入：locale、tracks | 输出：组件 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：声明页面渲染依赖的入参。
const props = defineProps<{
    // 本行目的：声明当前语言键。
    locale: LocaleKey;
    // 本行目的：声明 tracks 页面状态与操作对象。
    tracks: UseTracksPageReturn;
}>();

// 变量：statusLabel | 含义：把队列状态值转为本地化文本 | 类型：(status: MixQueueStatus) => string | 作用域：TracksPage 组件内
/** 函数：statusLabel | 输入：混流队列状态 | 输出：状态对应文案 | 可能失败：未知状态回退 failed 文案 */
// 本行目的：统一生成队列状态展示文本。
const statusLabel = (status: MixQueueStatus) => {
    // 本行目的：queued 状态文案。
    if (status === "queued") return t("tracks.mix.status.queued", props.locale);
    // 本行目的：running 状态文案。
    if (status === "running") return t("tracks.mix.status.running", props.locale);
    // 本行目的：success 状态文案。
    if (status === "success") return t("tracks.mix.status.success", props.locale);
    // 本行目的：其余状态回退 failed 文案。
    return t("tracks.mix.status.failed", props.locale);
};

// 变量：yesNoLabel | 含义：把布尔值映射为是/否/空文本 | 类型：(value?: boolean) => string | 作用域：TracksPage 组件内
/** 函数：yesNoLabel | 输入：可选布尔值 | 输出：yes/no/empty 文案 | 可能失败：无 */
// 本行目的：统一处理轨道字段中的布尔展示。
const yesNoLabel = (value?: boolean) => {
    // 本行目的：true 映射 yes 文案。
    if (value === true) return t("tracks.value.yes", props.locale);
    // 本行目的：false 映射 no 文案。
    if (value === false) return t("tracks.value.no", props.locale);
    // 本行目的：undefined 映射 empty 文案。
    return t("tracks.value.empty", props.locale);
};
</script>

<template>
    <!-- 本行目的：页面根容器。 -->
    <div class="app-body download-view">
        <!-- 本行目的：主卡片容器，承载 tracks 全部操作区。 -->
        <a-card :title="t('tracks.title', props.locale)" size="small" class="download-card">
            <!-- 本行目的：混流入队操作条。 -->
            <div class="tracks-mix-bar">
                <!-- 本行目的：点击后把当前选择轨道组合加入混流队列。 -->
                <a-button type="primary" size="small" :loading="tracks.trackMixLoading.value" @click="tracks.enqueueMixTask">
                    {{ t("tracks.mix.enqueue", props.locale) }}
                </a-button>
                <!-- 本行目的：显示混流成功或提示消息。 -->
                <span v-if="tracks.trackMixResult.value" class="tracks-mix-success">{{ tracks.trackMixResult.value }}</span>
                <!-- 本行目的：显示混流错误消息。 -->
                <span v-if="tracks.trackMixError.value" class="tracks-mix-error">{{ tracks.trackMixError.value }}</span>
            </div>
            <!-- 本行目的：默认轨道语言配置区域。 -->
            <div class="tracks-mix-lang">
                <!-- 本行目的：语言配置标题。 -->
                <span class="tracks-mix-lang-label">{{ t("tracks.lang.default", props.locale) }}</span>
                <!-- 本行目的：视频默认语言配置。 -->
                <div class="tracks-mix-lang-item">
                    <span>{{ t("tracks.type.video", props.locale) }}</span>
                    <a-select
                        v-model="tracks.trackLangDefaults.video"
                        size="small"
                        :options="tracks.trackLanguageOptions"
                        allow-search
                        allow-create
                        :placeholder="t('tracks.lang.placeholder', props.locale)"
                    />
                </div>
                <!-- 本行目的：音频默认语言配置。 -->
                <div class="tracks-mix-lang-item">
                    <span>{{ t("tracks.type.audio", props.locale) }}</span>
                    <a-select
                        v-model="tracks.trackLangDefaults.audio"
                        size="small"
                        :options="tracks.trackLanguageOptions"
                        allow-search
                        allow-create
                        :placeholder="t('tracks.lang.placeholder', props.locale)"
                    />
                </div>
                <!-- 本行目的：字幕默认语言配置。 -->
                <div class="tracks-mix-lang-item">
                    <span>{{ t("tracks.type.subtitle", props.locale) }}</span>
                    <a-select
                        v-model="tracks.trackLangDefaults.subtitle"
                        size="small"
                        :options="tracks.trackLanguageOptions"
                        allow-search
                        allow-create
                        :placeholder="t('tracks.lang.placeholder', props.locale)"
                    />
                </div>
            </div>
            <!-- 本行目的：三类轨道与队列列表总容器。 -->
            <div class="tracks-list">
                <!-- 本行目的：视频轨道区域。 -->
                <div class="tracks-section">
                    <div class="tracks-header">
                        <span class="tracks-title">{{ t("tracks.type.video", props.locale) }}</span>
                        <div class="tracks-actions">
                            <!-- 本行目的：选择视频文件。 -->
                            <a-button size="small" type="outline" @click="tracks.addTrackFile('video')">
                                {{ t("tracks.action.addFile", props.locale) }}
                            </a-button>
                            <!-- 本行目的：解析视频轨道信息。 -->
                            <a-button
                                size="small"
                                type="primary"
                                :loading="tracks.trackLoading.value.video"
                                :disabled="!tracks.trackFiles.value.video.length"
                                @click="tracks.detectTracks('video')"
                            >
                                {{ t("tracks.action.detect", props.locale) }}
                            </a-button>
                        </div>
                    </div>
                    <div class="tracks-body">
                        <!-- 本行目的：有文件时显示视频文件列表。 -->
                        <div class="tracks-files" v-if="tracks.trackFiles.value.video.length">
                            <div v-for="file in tracks.trackFiles.value.video" :key="file.id" class="tracks-file">
                                <span class="tracks-file-name">{{ file.name }}</span>
                                <span class="tracks-file-size">{{ file.fileSize || t("tracks.value.empty", props.locale) }}</span>
                                <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
                            </div>
                        </div>
                        <!-- 本行目的：无文件时显示空提示。 -->
                        <p v-else class="download-empty">{{ t("tracks.file.empty.video", props.locale) }}</p>
                        <!-- 本行目的：显示视频解析错误。 -->
                        <div v-if="tracks.trackErrors.value.video" class="tracks-error">{{ tracks.trackErrors.value.video }}</div>
                        <!-- 本行目的：显示视频解析进度条。 -->
                        <div v-if="tracks.trackLoading.value.video" class="tracks-progress">
                            <a-progress type="line" :percent="tracks.trackProgress.value.video" :show-text="true" :stroke-width="8" />
                        </div>
                        <!-- 本行目的：显示视频轨道解析结果。 -->
                        <div v-if="tracks.trackInfos.value.video.length" class="tracks-info">
                            <div v-for="group in tracks.trackInfos.value.video" :key="group.file.id" class="tracks-info-group">
                                <div class="tracks-info-file">{{ group.file.name }}</div>
                                <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                                    <!-- 本行目的：勾选是否参与混流。 -->
                                    <a-checkbox v-model="info.selected" size="small" />
                                    <span class="tracks-info-name"
                                        >{{ t("tracks.track.id", props.locale) }} {{ info.trackId || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.codec", props.locale) }} {{ info.codec || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.lang", props.locale) }} {{ info.lang || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta">
                                        {{ t("tracks.track.langName", props.locale) }}
                                        {{ info.languageName || t("tracks.value.empty", props.locale) }}
                                    </span>
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.name", props.locale) }}
                                        {{ info.trackName || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.default", props.locale) }} {{ yesNoLabel(info.isDefault) }}</span
                                    >
                                    <span class="tracks-info-meta">{{ t("tracks.track.forced", props.locale) }} {{ yesNoLabel(info.isForced) }}</span>
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.charset", props.locale) }}
                                        {{ info.charset || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.attributes", props.locale) }}
                                        {{ info.attributes || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.container", props.locale) }}
                                        {{ info.container || t("tracks.value.empty", props.locale) }}</span
                                    >
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <!-- 本行目的：音频轨道区域。 -->
                <div class="tracks-section">
                    <div class="tracks-header">
                        <span class="tracks-title">{{ t("tracks.type.audio", props.locale) }}</span>
                        <div class="tracks-actions">
                            <!-- 本行目的：选择音频文件。 -->
                            <a-button size="small" type="outline" @click="tracks.addTrackFile('audio')">
                                {{ t("tracks.action.addFile", props.locale) }}
                            </a-button>
                            <!-- 本行目的：解析音频轨道信息。 -->
                            <a-button
                                size="small"
                                type="primary"
                                :loading="tracks.trackLoading.value.audio"
                                :disabled="!tracks.trackFiles.value.audio.length"
                                @click="tracks.detectTracks('audio')"
                            >
                                {{ t("tracks.action.detect", props.locale) }}
                            </a-button>
                        </div>
                    </div>
                    <div class="tracks-body">
                        <!-- 本行目的：有文件时显示音频文件列表。 -->
                        <div class="tracks-files" v-if="tracks.trackFiles.value.audio.length">
                            <div v-for="file in tracks.trackFiles.value.audio" :key="file.id" class="tracks-file">
                                <span class="tracks-file-name">{{ file.name }}</span>
                                <span class="tracks-file-size">{{ file.fileSize || t("tracks.value.empty", props.locale) }}</span>
                                <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
                            </div>
                        </div>
                        <!-- 本行目的：无音频文件时显示空提示。 -->
                        <p v-else class="download-empty">{{ t("tracks.file.empty.audio", props.locale) }}</p>
                        <!-- 本行目的：显示音频解析错误。 -->
                        <div v-if="tracks.trackErrors.value.audio" class="tracks-error">{{ tracks.trackErrors.value.audio }}</div>
                        <!-- 本行目的：显示音频解析进度。 -->
                        <div v-if="tracks.trackLoading.value.audio" class="tracks-progress">
                            <a-progress type="line" :percent="tracks.trackProgress.value.audio" :show-text="true" :stroke-width="8" />
                        </div>
                        <!-- 本行目的：展示音频轨道解析详情。 -->
                        <div v-if="tracks.trackInfos.value.audio.length" class="tracks-info">
                            <div v-for="group in tracks.trackInfos.value.audio" :key="group.file.id" class="tracks-info-group">
                                <div class="tracks-info-file">{{ group.file.name }}</div>
                                <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                                    <!-- 本行目的：勾选该音频轨是否参与混流。 -->
                                    <a-checkbox v-model="info.selected" size="small" />
                                    <span class="tracks-info-name"
                                        >{{ t("tracks.track.id", props.locale) }} {{ info.trackId || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.codec", props.locale) }} {{ info.codec || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.lang", props.locale) }} {{ info.lang || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta">
                                        {{ t("tracks.track.langName", props.locale) }}
                                        {{ info.languageName || t("tracks.value.empty", props.locale) }}
                                    </span>
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.name", props.locale) }}
                                        {{ info.trackName || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.default", props.locale) }} {{ yesNoLabel(info.isDefault) }}</span
                                    >
                                    <span class="tracks-info-meta">{{ t("tracks.track.forced", props.locale) }} {{ yesNoLabel(info.isForced) }}</span>
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.charset", props.locale) }}
                                        {{ info.charset || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.attributes", props.locale) }}
                                        {{ info.attributes || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.container", props.locale) }}
                                        {{ info.container || t("tracks.value.empty", props.locale) }}</span
                                    >
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <!-- 本行目的：字幕轨道区域。 -->
                <div class="tracks-section">
                    <div class="tracks-header">
                        <span class="tracks-title">{{ t("tracks.type.subtitle", props.locale) }}</span>
                        <div class="tracks-actions">
                            <!-- 本行目的：选择字幕文件。 -->
                            <a-button size="small" type="outline" @click="tracks.addTrackFile('subtitle')">
                                {{ t("tracks.action.addFile", props.locale) }}
                            </a-button>
                            <!-- 本行目的：解析字幕轨道信息。 -->
                            <a-button
                                size="small"
                                type="primary"
                                :loading="tracks.trackLoading.value.subtitle"
                                :disabled="!tracks.trackFiles.value.subtitle.length"
                                @click="tracks.detectTracks('subtitle')"
                            >
                                {{ t("tracks.action.detect", props.locale) }}
                            </a-button>
                        </div>
                    </div>
                    <div class="tracks-body">
                        <!-- 本行目的：有文件时显示字幕文件列表。 -->
                        <div class="tracks-files" v-if="tracks.trackFiles.value.subtitle.length">
                            <div v-for="file in tracks.trackFiles.value.subtitle" :key="file.id" class="tracks-file">
                                <span class="tracks-file-name">{{ file.name }}</span>
                                <span class="tracks-file-size">{{ file.fileSize || t("tracks.value.empty", props.locale) }}</span>
                                <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
                            </div>
                        </div>
                        <!-- 本行目的：无字幕文件时显示空提示。 -->
                        <p v-else class="download-empty">{{ t("tracks.file.empty.subtitle", props.locale) }}</p>
                        <!-- 本行目的：显示字幕解析错误。 -->
                        <div v-if="tracks.trackErrors.value.subtitle" class="tracks-error">{{ tracks.trackErrors.value.subtitle }}</div>
                        <!-- 本行目的：显示字幕解析进度。 -->
                        <div v-if="tracks.trackLoading.value.subtitle" class="tracks-progress">
                            <a-progress type="line" :percent="tracks.trackProgress.value.subtitle" :show-text="true" :stroke-width="8" />
                        </div>
                        <!-- 本行目的：展示字幕轨道解析详情。 -->
                        <div v-if="tracks.trackInfos.value.subtitle.length" class="tracks-info">
                            <div v-for="group in tracks.trackInfos.value.subtitle" :key="group.file.id" class="tracks-info-group">
                                <div class="tracks-info-file">{{ group.file.name }}</div>
                                <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                                    <!-- 本行目的：勾选该字幕轨是否参与混流。 -->
                                    <a-checkbox v-model="info.selected" size="small" />
                                    <span class="tracks-info-name"
                                        >{{ t("tracks.track.id", props.locale) }} {{ info.trackId || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.codec", props.locale) }} {{ info.codec || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.lang", props.locale) }} {{ info.lang || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta">
                                        {{ t("tracks.track.langName", props.locale) }}
                                        {{ info.languageName || t("tracks.value.empty", props.locale) }}
                                    </span>
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.name", props.locale) }}
                                        {{ info.trackName || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.default", props.locale) }} {{ yesNoLabel(info.isDefault) }}</span
                                    >
                                    <span class="tracks-info-meta">{{ t("tracks.track.forced", props.locale) }} {{ yesNoLabel(info.isForced) }}</span>
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.charset", props.locale) }}
                                        {{ info.charset || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.attributes", props.locale) }}
                                        {{ info.attributes || t("tracks.value.empty", props.locale) }}</span
                                    >
                                    <span class="tracks-info-meta"
                                        >{{ t("tracks.track.container", props.locale) }}
                                        {{ info.container || t("tracks.value.empty", props.locale) }}</span
                                    >
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <!-- 本行目的：混流任务队列区域。 -->
                <div class="tracks-section">
                    <div class="tracks-header">
                        <span class="tracks-title">{{ t("tracks.mix.queue.title", props.locale) }}</span>
                        <div class="tracks-actions">
                            <!-- 本行目的：启动队列执行（运行中禁用）。 -->
                            <a-button size="small" type="outline" :disabled="tracks.mixQueueRunning.value" @click="tracks.startMixQueue">
                                {{ t("tracks.mix.queue.start", props.locale) }}
                            </a-button>
                            <!-- 本行目的：清空队列（运行中或空队列时禁用）。 -->
                            <a-button
                                size="small"
                                type="outline"
                                :disabled="tracks.mixQueueRunning.value || !tracks.mixQueue.value.length"
                                @click="tracks.clearMixQueue"
                            >
                                {{ t("tracks.mix.queue.clear", props.locale) }}
                            </a-button>
                        </div>
                    </div>
                    <div class="tracks-body">
                        <!-- 本行目的：队列为空时显示空提示。 -->
                        <p v-if="!tracks.mixQueue.value.length" class="download-empty">
                            {{ t("tracks.mix.queue.empty", props.locale) }}
                        </p>
                        <!-- 本行目的：队列有数据时展示任务列表。 -->
                        <div v-else class="mix-queue-list">
                            <!-- 本行目的：遍历任务并渲染任务行，支持点击查看详情。 -->
                            <div
                                v-for="item in tracks.mixQueue.value"
                                :key="item.id"
                                class="mix-queue-row"
                                @click.stop="tracks.openMixTaskDetail(item)"
                            >
                                <!-- 本行目的：显示任务编号。 -->
                                <span class="mix-queue-id">#{{ item.id }}</span>
                                <!-- 本行目的：显示任务创建时间。 -->
                                <span class="mix-queue-time">{{ item.createdAt }}</span>
                                <!-- 本行目的：显示输出路径，超长时可悬浮查看完整值。 -->
                                <span class="mix-queue-output" :title="item.outputPath">{{ item.outputPath }}</span>
                                <!-- 本行目的：显示当前任务状态文本。 -->
                                <span class="mix-queue-status" :data-status="item.status">
                                    {{ statusLabel(item.status) }}
                                </span>
                                <!-- 本行目的：存在执行消息时显示消息行。 -->
                                <div v-if="item.message" class="mix-queue-message-row" :title="item.message">
                                    {{ item.message }}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </a-card>
        <!-- 本行目的：渲染混流任务详情弹窗并与页面状态联动。 -->
        <MixQueueDetailModal
            :locale="props.locale"
            :show="tracks.mixQueueDetailVisible.value"
            :selected-mix-task="tracks.selectedMixTask.value"
            @update:show="(value) => (tracks.mixQueueDetailVisible.value = value)"
        />
    </div>
</template>
