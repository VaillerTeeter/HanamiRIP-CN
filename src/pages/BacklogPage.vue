<script setup lang="ts">
import { NButton, NCard } from "naive-ui";
import type { TrackedItem } from "../types/tracking";
import type { MonthAnime } from "../types/anime";
import { formatAirDate, formatRating } from "../utils/format";

const props = defineProps<{
  backlogList: TrackedItem[];
  setExclusiveStatus: (item: MonthAnime, target: "watching" | "backlog" | "watched" | null) => void | Promise<void>;
  openExternalLink: (url?: string | null) => void | Promise<void>;
}>();
</script>

<template>
  <div class="app-body list-view">
    <NCard title="补番计划" size="small" class="watchlist-card">
      <div v-if="props.backlogList.length" class="watchlist-grid">
        <div v-for="item in props.backlogList" :key="item.id" class="watchlist-item">
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
                补番计划
              </NButton>
              <NButton size="tiny" secondary disabled>
                转为正在追番
              </NButton>
              <NButton size="tiny" type="primary" secondary @click="props.setExclusiveStatus(item, 'watched')">
                转为已完番剧
              </NButton>
            </div>
          </div>
        </div>
      </div>
      <p v-else class="watchlist-empty">补番计划为空。</p>
    </NCard>
  </div>
</template>
