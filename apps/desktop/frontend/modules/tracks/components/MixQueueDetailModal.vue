<script setup lang="ts">
import { NModal } from "naive-ui";
import type { MixQueueItem } from "../types/tracks";

const props = defineProps<{
  show: boolean;
  selectedMixTask: MixQueueItem | null;
}>();

const emit = defineEmits<{ (e: "update:show", value: boolean): void }>();
</script>

<template>
  <NModal
    :show="props.show"
    preset="card"
    title="混流任务详情"
    style="width: min(720px, 92vw)"
    @update:show="(value: boolean) => emit('update:show', value)"
  >
    <div v-if="props.selectedMixTask" class="mix-queue-detail">
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
