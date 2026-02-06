<script setup lang="ts">
/**
 * 混流任务详情弹窗：
 * - 展示任务基本信息（ID/时间/输出路径/状态）
 * - 展示每个输入轨道的明细（类型/来源/轨道 ID/语言）
 */
import { NModal } from "naive-ui";
import type { MixQueueItem } from "../types/tracks";

/**
 * show：是否显示弹窗
 * selectedMixTask：当前选中的任务（为空则不渲染内容）
 */
const props = defineProps<{
  show: boolean;
  selectedMixTask: MixQueueItem | null;
}>();

/**
 * emit：控制弹窗显示/隐藏。
 */
const emit = defineEmits<{ (e: "update:show", value: boolean): void }>();
</script>

<template>
  <!-- 任务详情弹窗容器 -->
  <NModal
    :show="props.show"
    preset="card"
    title="混流任务详情"
    style="width: min(720px, 92vw)"
    @update:show="(value: boolean) => emit('update:show', value)"
  >
    <!-- 仅在有选中任务时显示内容 -->
    <div v-if="props.selectedMixTask" class="mix-queue-detail">
      <!-- 基础信息：任务 ID/创建时间/输出路径/状态 -->
      <div class="mix-queue-detail-row">
        <span class="mix-queue-detail-label">任务 ID</span>
        <span>#{{ props.selectedMixTask.id }}</span>
      </div>
      <div class="mix-queue-detail-row">
        <span class="mix-queue-detail-label">创建时间</span>
        <span>{{ props.selectedMixTask.createdAt }}</span>
      </div>
      <div class="mix-queue-detail-row">
        <span class="mix-queue-detail-label">输出路径</span>
        <span class="mix-queue-detail-value" :title="props.selectedMixTask.outputPath">
          {{ props.selectedMixTask.outputPath }}
        </span>
      </div>
      <div class="mix-queue-detail-row">
        <span class="mix-queue-detail-label">状态</span>
        <span>{{
          props.selectedMixTask.status === 'queued'
            ? '排队中'
            : props.selectedMixTask.status === 'running'
              ? '处理中'
              : props.selectedMixTask.status === 'success'
                ? '完成'
                : '失败'
        }}</span>
      </div>
      <div v-if="props.selectedMixTask.message" class="mix-queue-detail-row">
        <span class="mix-queue-detail-label">消息</span>
        <span class="mix-queue-detail-value" :title="props.selectedMixTask.message">
          {{ props.selectedMixTask.message }}
        </span>
      </div>

      <!-- 轨道输入列表：逐个展示输入源信息 -->
      <div class="mix-queue-detail-section">轨道输入</div>
      <div v-for="(input, index) in props.selectedMixTask.inputs" :key="`${input.kind}-${index}`" class="mix-queue-detail-block">
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">类型</span>
          <span>{{ input.kind === 'video' ? '视频' : input.kind === 'audio' ? '音频' : '字幕' }}</span>
        </div>
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">来源文件</span>
          <span class="mix-queue-detail-value" :title="input.path">{{ input.path }}</span>
        </div>
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">轨道 ID</span>
          <span>{{ input.trackIds.join(', ') || '-' }}</span>
        </div>
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">语言设置</span>
          <span class="mix-queue-detail-value">
            {{
              input.trackIds
                .map((id) => `${id}:${input.trackLangs?.[id] || '-'}`)
                .join('、') || '-'
            }}
          </span>
        </div>
      </div>
    </div>
  </NModal>
</template>
