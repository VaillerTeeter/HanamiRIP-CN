<script setup lang="ts">
import { NButton, NCard, NInput, NSelect, NTag } from "naive-ui";
import type { UseSearchPageReturn } from "../composables/useSearchPage";
import type { SearchResult } from "../types/search";
import type { DownloadItem } from "../types/download";

const props = defineProps<{
  search: UseSearchPageReturn;
  openExternalLink: (url?: string | null) => void | Promise<void>;
  handleDownloadClick: (item: SearchResult, kind: DownloadItem["kind"], link?: string) => void | Promise<void>;
}>();
</script>

<template>
  <div class="app-body search-view">
    <NCard size="small" class="search-panel">
      <template #header>
        <div class="search-card-header">
          <span>搜索资源</span>
          <span class="search-hint header-hint">先选逻辑，再添加短语或番剧；逻辑只对后续新增项生效</span>
        </div>
      </template>
      <div class="search-controls">
        <div class="search-row">
          <span class="search-label">逻辑</span>
          <div class="search-button-group logic-group">
            <NButton
              v-for="logic in search.logicOptions"
              :key="logic.value"
              secondary
              size="small"
              :type="search.activeLogic.value === logic.value ? 'primary' : 'default'"
              @click="search.activeLogic.value = logic.value"
            >
              {{ logic.label }}
            </NButton>
          </div>
        </div>

        <div class="search-row">
          <span class="search-label">预设</span>
          <div class="search-button-group">
            <NButton
              v-for="phrase in search.presetPhrases"
              :key="phrase"
              size="small"
              secondary
              @click="search.handleAddPreset(phrase)"
            >
              {{ phrase }}
            </NButton>
          </div>
        </div>

        <div class="search-row compact">
          <span class="search-label">番剧</span>
          <NSelect
            :value="search.trackedSelection.value"
            :options="search.trackedOptions.value"
            placeholder="选择正在追番/补番/已完番剧"
            clearable
            @update:value="(value) => {
              search.trackedSelection.value = value as number | null;
              search.handleSelectTracked(value as number | null);
            }"
          />
          <NInput
            :value="search.customSearchInput.value"
            placeholder="输入自定义精确短语"
            clearable
            @update:value="(value) => (search.customSearchInput.value = value)"
            @keyup.enter="search.handleAddCustom"
            class="search-input-flex"
          />
          <NButton type="primary" @click="search.handleAddCustom">添加</NButton>
        </div>

        <div class="search-row search-tags" v-if="search.searchTerms.value.length">
          <span class="search-label">已选</span>
          <div class="search-tag-list">
            <NTag
              v-for="(term, idx) in search.searchTerms.value"
              :key="idx"
              size="small"
              closable
              @close="search.removeSearchTerm(Number(idx))"
            >
              <span class="term-op">{{ term.op === 'and' ? '与' : term.op === 'or' ? '或' : '非' }}</span>
              <span class="term-value">{{ term.value }}</span>
            </NTag>
          </div>
        </div>

        <div class="search-row">
          <span class="search-label">拼接结果</span>
          <div class="search-preview">
            <div class="search-query">{{ search.searchQuery.value || '（尚未添加关键词）' }}</div>
          </div>
          <NButton type="primary" :disabled="!search.searchQuery.value" @click="search.openSearch">打开搜索</NButton>
        </div>

        <div
          v-if="search.searchLoading.value || search.searchError.value || search.searchResults.value.length || search.searchHtml.value"
          class="search-inline-results"
        >
          <div class="search-result-header">
            <span>搜索结果</span>
            <NButton size="tiny" secondary @click="search.clearSearchResults">收起</NButton>
          </div>
          <div class="search-open-modal">
            <p class="search-modal-row">
              <span class="search-modal-label">URL：</span>
              <a
                :href="search.searchUrl.value"
                target="_blank"
                rel="noreferrer"
                @click.prevent="props.openExternalLink(search.searchUrl.value)"
              >
                {{ search.searchUrl.value }}
              </a>
            </p>
            <div v-if="search.searchResults.value.length" class="search-result-list">
              <div class="search-result-row" v-for="item in search.searchResults.value" :key="item.detailUrl || item.title">
                <div class="sr-name">
                  <a
                    :href="item.detailUrl || item.magnet || item.download"
                    target="_blank"
                    rel="noreferrer"
                    @click.prevent="props.openExternalLink(item.detailUrl || item.magnet || item.download)"
                  >
                    {{ item.title }}
                  </a>
                  <div class="sr-meta" v-if="item.size || item.date">
                    <span v-if="item.size">{{ item.size }}</span>
                    <span v-if="item.date">{{ item.date }}</span>
                  </div>
                </div>
                <div class="sr-links">
                  <NButton
                    v-if="item.magnet"
                    text
                    type="primary"
                    size="small"
                    @click="props.handleDownloadClick(item, 'magnet', item.magnet)"
                  >
                    磁链
                  </NButton>
                  <NButton
                    v-if="item.download"
                    text
                    type="primary"
                    size="small"
                    @click="props.handleDownloadClick(item, 'torrent', item.download)"
                  >
                    种子
                  </NButton>
                </div>
              </div>
            </div>
            <iframe
              v-else-if="!search.searchLoading.value && !search.searchError.value"
              class="search-preview-frame"
              :srcdoc="search.searchHtml.value"
              sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
              title="搜索页面"
            />
            <div v-if="search.searchLoading.value" class="search-loading">正在加载...</div>
            <div v-else-if="search.searchError.value" class="search-error">{{ search.searchError.value }}</div>
          </div>
        </div>
      </div>
    </NCard>
  </div>
</template>
