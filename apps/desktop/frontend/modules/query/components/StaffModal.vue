<script setup lang="ts">
/**
 * 制作人员弹窗：
 * - 展示选中条目的 staff 分组
 * - 支持点击外链跳转
 */
import { NModal } from "naive-ui";
import type { StaffGroup } from "../../tracking/types/anime";
import type { MonthAnime } from "../../tracking/types/anime";

/**
 * props：外部传入的显示状态与数据。
 */
const props = defineProps<{
  show: boolean;
  selected: MonthAnime | null;
  staffLoadingId: number | null;
  staffError: string;
  selectedStaff: StaffGroup[];
  openExternalLink: (url?: string | null) => void | Promise<void>;
}>();

/**
 * emit：用于控制弹窗开关。
 */
const emit = defineEmits<{ (e: "update:show", value: boolean): void }>();
</script>

<template>
  <!-- 使用 NaiveUI Modal 作为弹窗容器 -->
  <NModal
    :show="props.show"
    preset="card"
    title="工作人员"
    size="small"
    @update:show="(value: boolean) => emit('update:show', value)"
  >
    <div class="staff-modal-body">
      <!-- 当前选中条目标题 -->
      <p class="staff-modal-title">当前条目：{{ props.selected?.nameCn || props.selected?.name || "" }}</p>
      <p v-if="props.staffLoadingId === props.selected?.id">正在加载工作人员信息...</p>
      <p v-else-if="props.staffError">{{ props.staffError }}</p>
      <div v-else>
        <p v-if="!props.selectedStaff.length">暂无工作人员信息。</p>
        <div v-else class="staff-group-list">
          <div v-for="group in props.selectedStaff" :key="group.role" class="staff-group">
            <div class="staff-role">{{ group.role }}</div>
            <div class="staff-people">
              <a
                v-for="person in group.people"
                :key="person.id"
                class="staff-link"
                :href="person.url"
                target="_blank"
                rel="noreferrer"
                @click.prevent="props.openExternalLink(person.url)"
              >
                {{ person.name }}
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </NModal>
</template>
