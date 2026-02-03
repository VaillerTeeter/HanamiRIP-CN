<script setup lang="ts">
import { NButton, NCheckbox, NCheckboxGroup, NModal } from "naive-ui";

const props = defineProps<{
  show: boolean;
  pendingTrackedName: string;
  aliasLoading: boolean;
  aliasOptions: string[];
  aliasSelected: string[];
  onCancel: () => void;
  onConfirm: () => void;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "update:aliasSelected", value: string[]): void;
}>();
</script>

<template>
  <NModal
    :show="props.show"
    preset="card"
    title="选择名称 / 别名"
    size="small"
    :style="{ width: 'auto', maxWidth: '520px' }"
    @update:show="(value) => emit('update:show', value)"
  >
    <div class="alias-modal-body">
      <p class="alias-title">为 {{ props.pendingTrackedName || "该番剧" }} 选择要加入的名称</p>
      <p v-if="props.aliasLoading" class="alias-hint">正在获取别名...</p>
      <template v-else>
        <NCheckboxGroup
          :value="props.aliasSelected"
          @update:value="(value) => emit('update:aliasSelected', value as string[])"
        >
          <div class="alias-list">
            <NCheckbox v-for="name in props.aliasOptions" :key="name" :value="name" class="alias-item">
              {{ name }}
            </NCheckbox>
          </div>
        </NCheckboxGroup>
      </template>
      <div class="alias-actions">
        <NButton size="small" @click="props.onCancel">取消</NButton>
        <NButton size="small" type="primary" :disabled="props.aliasLoading || !props.aliasSelected.length" @click="props.onConfirm">
          确定
        </NButton>
      </div>
    </div>
  </NModal>
</template>
