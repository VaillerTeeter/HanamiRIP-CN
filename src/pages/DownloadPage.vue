<script setup lang="ts">
import { NButton, NCard, NProgress } from "naive-ui";
import type { UseDownloadPageReturn } from "../composables/useDownloadPage";

const props = defineProps<{
  downloads: UseDownloadPageReturn;
}>();
</script>

<template>
  <div class="app-body download-view">
    <NCard title="下载" size="small" class="download-card">
      <div class="download-toolbar">
        <div v-if="downloads.downloads.value.length" class="download-total-speed">
          <span class="download-total-label">总速率</span>
          <span class="pill">↓ {{ downloads.totalDownloadSpeedLabel.value }}</span>
          <span class="pill">↑ {{ downloads.totalUploadSpeedLabel.value }}</span>
        </div>
        <div class="download-toolbar-actions">
          <NButton
            size="small"
            secondary
            class="icon-button"
            :disabled="!downloads.hasActiveDownloads.value"
            aria-label="全部暂停"
            title="全部暂停"
            @click="downloads.handlePauseAllDownloads"
          >
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
              <line x1="8" y1="6" x2="8" y2="18" />
              <line x1="16" y1="6" x2="16" y2="18" />
            </svg>
          </NButton>
          <NButton
            size="small"
            secondary
            class="icon-button"
            :disabled="!downloads.hasPausedDownloads.value"
            aria-label="全部继续"
            title="全部继续"
            @click="downloads.handleResumeAllDownloads"
          >
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
              <polygon points="9 7 19 12 9 17" />
            </svg>
          </NButton>
        </div>
      </div>
      <div v-if="downloads.downloads.value.length" class="download-list">
        <div v-for="item in downloads.downloads.value" :key="item.id" class="download-row">
          <div class="download-main">
            <div class="download-title">{{ downloads.downloadDisplayTitle(item) }}</div>
            <div class="download-meta">
              <span class="pill">{{ item.kind === 'magnet' ? '磁链' : '种子' }}</span>
              <span class="pill">{{ downloads.formatDownloadStatus(item.status) }}</span>
              <span v-if="item.state" class="pill">{{ item.state }}</span>
              <span v-if="item.downloadSpeed" class="pill">↓ {{ item.downloadSpeed }}</span>
              <span v-if="item.uploadSpeed" class="pill">↑ {{ item.uploadSpeed }}</span>
              <span v-if="item.timeRemaining" class="pill">剩余 {{ item.timeRemaining }}</span>
              <span class="pill">{{ item.startedAt }}</span>
              <span v-if="item.path" class="pill path-pill" :title="item.path">{{ item.path }}</span>
            </div>
            <div v-if="item.totalBytes" class="download-progress">
              <NProgress
                type="line"
                :percentage="Math.min(100, Math.round(((item.progressBytes ?? 0) / item.totalBytes) * 100))"
                :show-indicator="false"
                :height="8"
              />
              <div class="progress-text">
                {{ downloads.formatBytes(item.progressBytes) }} / {{ downloads.formatBytes(item.totalBytes) }}
              </div>
            </div>
            <div v-if="item.error" class="download-error">{{ item.error }}</div>
          </div>
          <div class="download-actions">
            <NButton
              size="tiny"
              text
              class="icon-button"
              :disabled="item.torrentId == null || downloads.isDownloadTerminal(item) || downloads.isDownloadPaused(item)"
              aria-label="暂停"
              title="暂停"
              @click="downloads.handlePauseDownload(item)"
            >
              <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                <line x1="8" y1="6" x2="8" y2="18" />
                <line x1="16" y1="6" x2="16" y2="18" />
              </svg>
            </NButton>
            <NButton
              size="tiny"
              text
              class="icon-button"
              :disabled="item.torrentId == null || !downloads.isDownloadPaused(item)"
              aria-label="继续"
              title="继续"
              @click="downloads.handleResumeDownload(item)"
            >
              <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                <polygon points="9 7 19 12 9 17" />
              </svg>
            </NButton>
            <NButton
              size="tiny"
              text
              type="error"
              class="icon-button"
              aria-label="删除"
              title="删除"
              @click="downloads.handleDeleteDownload(item)"
            >
              <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 7h16" />
                <path d="M9 7v12" />
                <path d="M15 7v12" />
                <path d="M6 7l1 13h10l1-13" />
                <path d="M9 4h6l1 3H8l1-3Z" />
              </svg>
            </NButton>
          </div>
        </div>
      </div>
      <p v-else class="download-empty">暂无下载记录。</p>
    </NCard>
  </div>
</template>
