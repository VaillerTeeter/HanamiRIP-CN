<script setup lang="ts">
import { NButton, NCard } from "naive-ui";
import type { TrackedItem } from "../types/tracking";
import type { MonthAnime } from "../types/anime";
import { formatAirDate, formatRating } from "../utils/format";

const props = defineProps<{
  watchingByWeekday: Array<{ key: number | null; label: string; items: TrackedItem[] }>;
  setExclusiveStatus: (item: MonthAnime, target: "watching" | "backlog" | "watched" | null) => void | Promise<void>;
  openExternalLink: (url?: string | null) => void | Promise<void>;
}>();
</script>

<template>
  <div class="app-body list-view">
    <NCard title="正在追番" size="small" class="watchlist-card">
      <div v-if="props.watchingByWeekday.length" class="watchlist-section-list">
        <div v-for="group in props.watchingByWeekday" :key="group.label" class="watchlist-section">
          <div class="watchlist-section-title">{{ group.label }}</div>
          <div class="watchlist-grid">
            <div v-for="item in group.items" :key="item.id" class="watchlist-item">
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
                <div v-if="item.totalCount" class="episode-strip">
                  <div class="episode-strip-header">
                    <span class="episode-strip-label">章节列表</span>
                    <span class="episode-strip-summary">
                      共 {{ item.totalCount }} 集 · 已播 {{ item.airedCount ?? 0 }} 集
                    </span>
                  </div>
                  <div class="episode-strip-grid">
                    <span
                      v-for="n in item.totalCount"
                      :key="n"
                      class="episode-pill"
                      :class="{ active: (item.airedCount ?? 0) >= n }"
                    >
                      {{ String(n).padStart(2, '0') }}
                    </span>
                  </div>
                </div>
                <div class="watchlist-actions">
                  <NButton size="tiny" type="primary" secondary disabled>
                    正在追番
                  </NButton>
                  <NButton size="tiny" secondary @click="props.setExclusiveStatus(item, 'backlog')">
                    转为补番计划
                  </NButton>
                  <NButton size="tiny" secondary @click="props.setExclusiveStatus(item, 'watched')">
                    转为已完番剧
                  </NButton>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <p v-else class="watchlist-empty">还没有正在追的番剧。</p>
    </NCard>
  </div>
</template>
