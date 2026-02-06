<script setup lang="ts">
/**
 * 季度查询页面：
 * - 提供“年份/季度”选择与查询入口
 * - 查询完成后展示筛选器、进度条、结果列表与详情
 * - 详情区包含原作/放送/角色/简介等扩展信息
 */
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

/**
 * query：查询页逻辑（状态 + 方法）。
 *   - 包含查询条件、结果列表、详情加载状态等。
 * tracking：追番相关状态与操作（用于详情按钮）。
 * openExternalLink：外链打开方法（桌面端优先走 Tauri）。
 */
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

// 初次挂载后计算查询面板高度，用于 CSS 变量布局（避免列表顶住）。
onMounted(() => {
  void props.query.setQueryPanelHeight();
});
</script>

<template>
  <!-- 结果页整体容器：查询后加 results-view 类（用于切换布局） -->
  <div class="app-body" :class="{ 'results-view': query.hasQueried }">
    <section class="query-summary-row">
      <!-- 查询条件面板：选择年份/季度并触发查询 -->
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
      <!-- 查询概览（来源/季度/条目数）：仅在有结果时显示 -->
      <section v-if="query.showResults.value" class="summary-panel">
        <NCard size="small" class="result-summary">
          <NSpace vertical size="small">
            <span>来源：{{ query.resultUrl }}</span>
            <span>季度：{{ query.year.value }}年{{ query.seasonLabelFromMonth(query.month.value) }}</span>
            <span>条目数量：{{ query.resultCount.value }}</span>
          </NSpace>
        </NCard>
      </section>
      <!-- 筛选器面板：月份/类型/地区/受众 -->
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
                <!-- 用 Tag 展示“筛选状态”，不直接显示选中项 -->
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
                <!-- 同上：只展示筛选状态文案 -->
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
                <!-- 同上：只展示筛选状态文案 -->
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
                <!-- 同上：只展示筛选状态文案 -->
                <div class="filter-select-chip">
                  <NTag size="small" type="info">{{ query.audienceFilterLabel.value }}</NTag>
                </div>
              </div>
            </div>
          </div>
          <!-- 当筛选数据仍在加载时，给出提示 -->
          <span v-if="query.filterLoading.value" class="filter-loading">筛选信息加载中...</span>
        </NCard>
      </section>
    </section>

    <!-- 查询进度条：伪进度 + 实际加载完成后补齐到 100% -->
    <section v-if="query.hasQueried.value" class="progress-panel">
      <NProgress type="line" :percentage="query.progress.value" color="#18a058" :show-indicator="true" />
    </section>

    <!-- 结果区：列表 + 详情 -->
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
              <!-- 列表项：可点击/键盘操作，触发选中并加载详情 -->
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
          <!-- 详情面板：仅在选中条目时显示 -->
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
                <!-- 标题：点击外链打开条目页面 -->
                <a
                  class="detail-title detail-title-link"
                  :href="query.selected.value?.url"
                  target="_blank"
                  rel="noreferrer"
                  @click.prevent="openExternalLink(query.selected.value?.url)"
                >
                  {{ query.selected.value?.nameCn || query.selected.value?.name }}
                </a>
                <!-- 追番状态操作区：三态互斥 -->
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
                  <!-- 原名 -->
                  <div class="detail-info-row">
                    <span class="detail-label">原名</span>
                    <span class="detail-value">{{ query.selected.value?.name }}</span>
                  </div>
                  <!-- 评分：星级 + 数值 -->
                  <div class="detail-info-row">
                    <span class="detail-label">评分</span>
                    <span class="detail-value detail-rating">
                      <span class="detail-stars">{{ formatStars(query.selected.value?.rating) }}</span>
                      <span class="detail-score">{{ formatRating(query.selected.value?.rating) }}</span>
                    </span>
                  </div>
                  <!-- 放送：日期 + 已播出集数（异步加载） -->
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
                  <!-- 原作：来自 infobox，可能为空 -->
                  <div class="detail-info-row">
                    <span class="detail-label">原作</span>
                    <span class="detail-value">
                      <template v-if="query.originLoadingId.value === query.selected.value?.id">加载中...</template>
                      <template v-else-if="query.originError.value && !query.selected.value?.origin">获取失败</template>
                      <template v-else>{{ query.selected.value?.origin || "未知" }}</template>
                    </span>
                  </div>
                  <!-- 工作人员弹窗入口 -->
                  <div class="detail-info-row">
                    <span class="detail-label">演员相关</span>
                    <span class="detail-value">
                      <NButton size="tiny" type="primary" secondary @click="query.handleStaffOpen">
                        查看工作人员
                      </NButton>
                    </span>
                  </div>
                  <!-- 角色列表：链接到外部页面 -->
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
                  <!-- 简介：优先展示中文简介，必要时标注翻译 -->
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
