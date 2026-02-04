<script setup lang="ts">
import { onMounted } from "vue";
import {
  NButton,
  NCard,
  NProgress,
  NSelect,
  NSpace,
  NTag,
} from "naive-ui";
import type { MonthAnime } from "../../tracking/types/anime";
import type { ItemStatus, StatusKey } from "../../tracking/types/tracking";
import type { UseQueryPageReturn } from "../composables/useQueryPage";
import { formatAirDate, formatRating, formatStars } from "../../../shared/utils/format";

const props = defineProps<{
  query: UseQueryPageReturn;
  tracking: {
    selectedStatus: ItemStatus;
    currentStatusKey: (status: ItemStatus) => StatusKey;
    labelForAction: (active: StatusKey, target: StatusKey) => string;
    setExclusiveStatus: (item: MonthAnime, target: StatusKey) => void | Promise<void>;
  };
  openExternalLink: (url?: string | null) => void | Promise<void>;
}>();

onMounted(() => {
  void props.query.setQueryPanelHeight();
});
</script>

<template>
  <div class="app-body" :class="{ 'results-view': query.hasQueried }">
    <section class="query-summary-row">
      <section :ref="query.queryPanelRef" class="query-panel">
        <NCard title="选择查询条件" size="small">
          <NSpace align="center" size="medium" class="query-actions">
            <NSelect
              :value="query.year.value"
              :options="query.yearOptions"
              placeholder="选择年份"
              @update:value="(value: number | null) => (query.year.value = value as number | null)"
            />
            <NSelect
              :value="query.month.value"
              :options="query.seasonOptions.value"
              placeholder="选择季度"
              @update:value="(value: number | null) => (query.month.value = value as number | null)"
            />
            <NButton type="primary" :loading="query.loading.value" @click="query.handleQuery">
              开始查询季度番剧
            </NButton>
          </NSpace>
        </NCard>
      </section>
      <section v-if="query.showResults.value" class="summary-panel">
        <NCard size="small" class="result-summary">
          <NSpace vertical size="small">
            <span>来源：{{ query.resultUrl }}</span>
            <span>季度：{{ query.year.value }}年{{ query.seasonLabelFromMonth(query.month.value) }}</span>
            <span>条目数量：{{ query.resultCount.value }}</span>
          </NSpace>
        </NCard>
      </section>
      <section v-if="query.showResults.value" class="filter-panel">
        <NCard size="small">
          <div class="filter-grid">
            <div class="filter-field">
              <span class="filter-label">月份</span>
              <div class="filter-select">
                <NSelect
                  :value="query.monthFilter.value"
                  :options="query.monthFilterOptions.value"
                  multiple
                  placeholder="选择月份"
                  :disabled="query.filterLoading.value"
                  :max-tag-count="0"
                  :max-tag-placeholder="() => ''"
                  :render-tag="() => null"
                  @update:value="query.handleMonthFilterChange"
                />
                <div class="filter-select-chip">
                  <NTag size="small" type="info">{{ query.monthFilterLabel.value }}</NTag>
                </div>
              </div>
            </div>
            <div class="filter-field">
              <span class="filter-label">类型</span>
              <div class="filter-select">
                <NSelect
                  :value="query.typeFilter.value"
                  :options="query.typeOptions.value"
                  multiple
                  placeholder="选择类型"
                  :disabled="query.filterLoading.value"
                  :max-tag-count="0"
                  :max-tag-placeholder="() => ''"
                  :render-tag="() => null"
                  @update:value="query.handleTypeFilterChange"
                />
                <div class="filter-select-chip">
                  <NTag size="small" type="info">{{ query.typeFilterLabel.value }}</NTag>
                </div>
              </div>
            </div>
            <div class="filter-field">
              <span class="filter-label">地区</span>
              <div class="filter-select">
                <NSelect
                  :value="query.regionFilter.value"
                  :options="query.regionOptions.value"
                  multiple
                  placeholder="选择地区"
                  :disabled="query.filterLoading.value"
                  :max-tag-count="0"
                  :max-tag-placeholder="() => ''"
                  :render-tag="() => null"
                  @update:value="query.handleRegionFilterChange"
                />
                <div class="filter-select-chip">
                  <NTag size="small" type="info">{{ query.regionFilterLabel.value }}</NTag>
                </div>
              </div>
            </div>
            <div class="filter-field">
              <span class="filter-label">受众</span>
              <div class="filter-select">
                <NSelect
                  :value="query.audienceFilter.value"
                  :options="query.audienceOptions.value"
                  multiple
                  placeholder="选择受众"
                  :disabled="query.filterLoading.value"
                  :max-tag-count="0"
                  :max-tag-placeholder="() => ''"
                  :render-tag="() => null"
                  @update:value="query.handleAudienceFilterChange"
                />
                <div class="filter-select-chip">
                  <NTag size="small" type="info">{{ query.audienceFilterLabel.value }}</NTag>
                </div>
              </div>
            </div>
          </div>
          <span v-if="query.filterLoading.value" class="filter-loading">筛选信息加载中...</span>
        </NCard>
      </section>
    </section>

    <section v-if="query.hasQueried.value" class="progress-panel">
      <NProgress type="line" :percentage="query.progress.value" color="#18a058" :show-indicator="true" />
    </section>

    <section v-if="query.showResults.value" class="result-panel">
      <div class="result-content">
        <div class="result-layout" v-if="query.filteredResults.value.length">
          <NCard title="条目列表" size="small" class="result-list">
            <div
              :ref="query.resultListRef"
              class="result-list-grid"
              @mouseenter="query.handleListMouseEnter"
              @mouseleave="query.handleListMouseLeave"
            >
              <div
                v-for="item in query.filteredResults.value"
                :key="item.id ?? item.name"
                class="result-list-item"
                role="button"
                tabindex="0"
                :ref="(el: any) => query.setListItemRef(item, el as HTMLElement | null)"
                @click="query.handleSelect(item)"
                @keydown.enter.prevent="query.handleSelect(item)"
                @keydown.space.prevent="query.handleSelect(item)"
              >
                <div class="result-list-thumb">
                  <img :src="item.image" :alt="item.name" />
                </div>
                <div class="result-list-info">
                  <div class="result-list-title">{{ item.nameCn || item.name }}</div>
                </div>
              </div>
            </div>
          </NCard>
          <NCard v-if="query.selected.value" title="条目详情" size="small" class="detail-panel">
            <div class="detail-panel-body">
              <div class="detail-media">
                <img
                  class="detail-image"
                  :src="query.selected.value?.image"
                  :alt="query.selected.value?.name"
                />
              </div>
              <div class="detail-info">
                <a
                  class="detail-title detail-title-link"
                  :href="query.selected.value?.url"
                  target="_blank"
                  rel="noreferrer"
                  @click.prevent="openExternalLink(query.selected.value?.url)"
                >
                  {{ query.selected.value?.nameCn || query.selected.value?.name }}
                </a>
                <div class="detail-actions">
                  <NButton
                    size="tiny"
                    :type="tracking.selectedStatus.watching ? 'primary' : 'default'"
                    secondary
                    @click="query.selected.value && tracking.setExclusiveStatus(query.selected.value, 'watching')"
                  >
                    {{ tracking.labelForAction(tracking.currentStatusKey(tracking.selectedStatus), 'watching') }}
                  </NButton>
                  <NButton
                    size="tiny"
                    :type="tracking.selectedStatus.backlog ? 'primary' : 'default'"
                    secondary
                    @click="query.selected.value && tracking.setExclusiveStatus(query.selected.value, 'backlog')"
                  >
                    {{ tracking.labelForAction(tracking.currentStatusKey(tracking.selectedStatus), 'backlog') }}
                  </NButton>
                  <NButton
                    size="tiny"
                    :type="tracking.selectedStatus.watched ? 'primary' : 'default'"
                    secondary
                    @click="query.selected.value && tracking.setExclusiveStatus(query.selected.value, 'watched')"
                  >
                    {{ tracking.labelForAction(tracking.currentStatusKey(tracking.selectedStatus), 'watched') }}
                  </NButton>
                </div>
                <div class="detail-info-list">
                  <div class="detail-info-row">
                    <span class="detail-label">原名</span>
                    <span class="detail-value">{{ query.selected.value?.name }}</span>
                  </div>
                  <div class="detail-info-row">
                    <span class="detail-label">评分</span>
                    <span class="detail-value detail-rating">
                      <span class="detail-stars">{{ formatStars(query.selected.value?.rating) }}</span>
                      <span class="detail-score">{{ formatRating(query.selected.value?.rating) }}</span>
                    </span>
                  </div>
                  <div class="detail-info-row">
                    <span class="detail-label">放送</span>
                    <span class="detail-value">
                      {{ formatAirDate(query.selected.value?.date) }}
                      <span class="detail-divider">·</span>
                      <template v-if="query.airedLoadingId.value === query.selected.value?.id">已播出计算中...</template>
                      <template v-else-if="query.airedError.value && query.selected.value?.airedCount == null">已播出获取失败</template>
                      <template v-else>已播出 {{ query.selected.value?.airedCount ?? 0 }} 集</template>
                    </span>
                  </div>
                  <div class="detail-info-row">
                    <span class="detail-label">原作</span>
                    <span class="detail-value">
                      <template v-if="query.originLoadingId.value === query.selected.value?.id">加载中...</template>
                      <template v-else-if="query.originError.value && !query.selected.value?.origin">获取失败</template>
                      <template v-else>{{ query.selected.value?.origin || "未知" }}</template>
                    </span>
                  </div>
                  <div class="detail-info-row">
                    <span class="detail-label">演员相关</span>
                    <span class="detail-value">
                      <NButton size="tiny" type="primary" secondary @click="query.handleStaffOpen">
                        查看工作人员
                      </NButton>
                    </span>
                  </div>
                  <div class="detail-info-row">
                    <span class="detail-label">角色</span>
                    <span class="detail-value">
                      <template v-if="query.characterLoadingId.value === query.selected.value?.id">加载中...</template>
                      <template v-else-if="query.characterError.value && !query.selectedCharacters.value.length">获取失败</template>
                      <template v-else-if="!query.selectedCharacters.value.length">暂无</template>
                      <span v-else class="detail-link-list">
                        <a
                          v-for="item in query.selectedCharacters.value"
                          :key="item.id"
                          class="detail-link"
                          :href="item.url"
                          target="_blank"
                          rel="noreferrer"
                          @click.prevent="openExternalLink(item.url)"
                        >
                          {{ item.name }}
                        </a>
                      </span>
                    </span>
                  </div>
                  <div class="detail-info-row">
                    <span class="detail-label">简介</span>
                    <span class="detail-value detail-summary">
                      <template v-if="query.summaryLoadingId.value === query.selected.value?.id">加载中...</template>
                      <template v-else-if="query.summaryError.value && !query.selected.value?.summaryCn">获取失败：{{ query.summaryError.value }}</template>
                      <template v-else-if="!query.selected.value?.summaryCn">暂无</template>
                      <template v-else>
                        {{ query.selected.value?.summaryCn }}
                        <span v-if="query.selected.value?.summaryTranslated" class="detail-translate-tag">（翻译）</span>
                        <span v-else-if="query.summaryError.value" class="detail-translate-tag">
                          （{{ query.summaryError.value }}）
                        </span>
                      </template>
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </NCard>
        </div>
      </div>
    </section>
  </div>
</template>
