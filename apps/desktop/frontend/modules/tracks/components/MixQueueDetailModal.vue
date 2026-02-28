<script setup lang="ts">
/** 文件：MixQueueDetailModal.vue | 用途：展示混流队列任务的详细信息弹窗 | 关键对象：props, emit, statusLabelMap, kindLabel */
// 本行目的：引入国际化函数与语言键类型。
import { t, type LocaleKey } from "../../../shared/i18n/messages";
// 本行目的：引入混流队列条目类型，约束详情数据结构。
import type { MixQueueItem } from "../types/tracks";

// 变量：props | 含义：父组件传入的弹窗状态与当前任务数据 | 类型：由 defineProps 推断的对象 | 作用域：MixQueueDetailModal 组件内
/** 函数：defineProps | 输入：locale、show、selectedMixTask | 输出：组件 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：声明弹窗渲染所需输入参数。
const props = defineProps<{
    // 本行目的：声明当前语言键。
    locale: LocaleKey;
    // 本行目的：声明弹窗显示状态。
    show: boolean;
    // 本行目的：声明当前选中的混流任务（可为空）。
    selectedMixTask: MixQueueItem | null;
}>();

// 变量：emit | 含义：组件事件派发函数 | 类型：由 defineEmits 推断 | 作用域：MixQueueDetailModal 组件内
/** 函数：defineEmits | 输入：update:show 事件和布尔值 | 输出：emit 函数 | 可能失败：无（类型不匹配会在编译期报错） */
// 本行目的：声明支持的事件，供弹窗显隐双向绑定。
const emit = defineEmits<{ (e: "update:show", value: boolean): void }>();

// 变量：statusLabelMap | 含义：任务状态到本地化文案函数的映射 | 类型：Record<string, () => string> | 作用域：MixQueueDetailModal 组件内
// 本行目的：集中维护状态文案映射，避免模板内重复判断。
const statusLabelMap = {
    // 本行目的：排队中状态文案。
    queued: () => t("tracks.mix.status.queued", props.locale),
    // 本行目的：执行中状态文案。
    running: () => t("tracks.mix.status.running", props.locale),
    // 本行目的：成功状态文案。
    success: () => t("tracks.mix.status.success", props.locale),
    // 本行目的：失败状态文案。
    failed: () => t("tracks.mix.status.failed", props.locale),
};

// 变量：kindLabel | 含义：把输入类型映射为本地化类型文案 | 类型：(kind: string) => string | 作用域：MixQueueDetailModal 组件内
/** 函数：kindLabel | 输入：输入类型字符串 | 输出：video/audio/subtitle 对应文案 | 可能失败：未知类型回退 subtitle 文案 */
// 本行目的：统一处理输入类型显示文本。
const kindLabel = (kind: string) => {
    // 本行目的：视频类型映射。
    if (kind === "video") return t("tracks.type.video", props.locale);
    // 本行目的：音频类型映射。
    if (kind === "audio") return t("tracks.type.audio", props.locale);
    // 本行目的：其余类型回退字幕文案。
    return t("tracks.type.subtitle", props.locale);
};
</script>

<template>
    <!-- 本行目的：渲染详情弹窗容器。 -->
    <a-modal
        :visible="props.show"
        :title="t('tracks.mix.detail.title', props.locale)"
        :width="720"
        @update:visible="(value: boolean) => emit('update:show', value)"
    >
        <!-- 本行目的：仅在存在选中任务时渲染详情内容。 -->
        <div v-if="props.selectedMixTask" class="mix-queue-detail">
            <!-- 本行目的：展示任务ID。 -->
            <div class="mix-queue-detail-row">
                <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.id", props.locale) }}</span>
                <span>#{{ props.selectedMixTask.id }}</span>
            </div>
            <!-- 本行目的：展示任务创建时间。 -->
            <div class="mix-queue-detail-row">
                <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.createdAt", props.locale) }}</span>
                <span>{{ props.selectedMixTask.createdAt }}</span>
            </div>
            <!-- 本行目的：展示输出文件路径。 -->
            <div class="mix-queue-detail-row">
                <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.output", props.locale) }}</span>
                <span class="mix-queue-detail-value" :title="props.selectedMixTask.outputPath">
                    {{ props.selectedMixTask.outputPath }}
                </span>
            </div>
            <!-- 本行目的：展示任务当前状态。 -->
            <div class="mix-queue-detail-row">
                <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.status", props.locale) }}</span>
                <span>{{ statusLabelMap[props.selectedMixTask.status]() }}</span>
            </div>
            <!-- 本行目的：有附加消息时展示消息文本。 -->
            <div v-if="props.selectedMixTask.message" class="mix-queue-detail-row">
                <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.message", props.locale) }}</span>
                <span class="mix-queue-detail-value" :title="props.selectedMixTask.message">
                    {{ props.selectedMixTask.message }}
                </span>
            </div>
            <!-- 本行目的：输入列表分节标题。 -->
            <div class="mix-queue-detail-section">{{ t("tracks.mix.detail.inputs", props.locale) }}</div>
            <!-- 本行目的：遍历输入源并渲染每个输入块。 -->
            <div v-for="(input, index) in props.selectedMixTask.inputs" :key="`${input.kind}-${index}`" class="mix-queue-detail-block">
                <!-- 本行目的：展示输入类型。 -->
                <div class="mix-queue-detail-row">
                    <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.kind", props.locale) }}</span>
                    <span>{{ kindLabel(input.kind) }}</span>
                </div>
                <!-- 本行目的：展示输入源文件路径。 -->
                <div class="mix-queue-detail-row">
                    <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.source", props.locale) }}</span>
                    <span class="mix-queue-detail-value" :title="input.path">{{ input.path }}</span>
                </div>
                <!-- 本行目的：展示输入的轨道ID列表。 -->
                <div class="mix-queue-detail-row">
                    <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.trackIds", props.locale) }}</span>
                    <span>{{ input.trackIds.join(", ") || "-" }}</span>
                </div>
                <!-- 本行目的：展示轨道ID与语言映射信息。 -->
                <div class="mix-queue-detail-row">
                    <span class="mix-queue-detail-label">{{ t("tracks.mix.detail.trackLangs", props.locale) }}</span>
                    <span class="mix-queue-detail-value">
                        {{ input.trackIds.map((id) => `${id}:${input.trackLangs?.[id] || "-"}`).join("、") || "-" }}
                    </span>
                </div>
            </div>
        </div>
    </a-modal>
</template>
