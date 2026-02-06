<script setup lang="ts">
/**
 * 已完番剧页面：展示标记为 watched 的条目列表。
 */
import { NButton, NCard } from "naive-ui";
import type { TrackedItem } from "../types/tracking";
import type { MonthAnime } from "../types/anime";
import { formatAirDate, formatRating } from "../../../shared/utils/format";

/**
 * finishedList：已完条目列表
 * setExclusiveStatus：切换追番状态
 * openExternalLink：打开条目外链
 */
const props = defineProps<{
  finishedList: TrackedItem[];
  setExclusiveStatus: (item: MonthAnime, target: "watching" | "backlog" | "watched" | null) => void | Promise<void>;
  openExternalLink: (url?: string | null) => void | Promise<void>;
}>();
</script>

<template>
  <!-- 已完番剧列表 -->
  <div class="app-body list-view">
    <NCard title="已完番剧" size="small" class="watchlist-card">
      <div v-if="props.finishedList.length" class="watchlist-grid">
        <div v-for="item in props.finishedList" :key="item.id" class="watchlist-item">
          <div class="watchlist-thumb">
            <img :src="item.image" :alt="item.name" />
          </div>
          <div class="watchlist-body">
            <a
              class="watchlist-title"
              :href="item.url"
              target="_blank"
              rel="noreferrer"
              @click.prevent="props.openExternalLink(item.url)"
            >
              {{ item.nameCn || item.name }}
            </a>
            <div class="watchlist-meta">
              <span class="meta-row">放送：{{ formatAirDate(item.date) }}</span>
              <span class="meta-row">评分：{{ formatRating(item.rating) }}</span>
            </div>
            <div class="watchlist-actions">
              <NButton size="tiny" secondary disabled>
                转为正在追番
              </NButton>
              <NButton size="tiny" secondary disabled>
                转为补番计划
              </NButton>
              <NButton size="tiny" type="primary" secondary @click="props.setExclusiveStatus(item, null)">
                变为未观看
              </NButton>
            </div>
          </div>
        </div>
      </div>
      <p v-else class="watchlist-empty">还没有标记已看的番剧。</p>
    </NCard>
  </div>
</template>
