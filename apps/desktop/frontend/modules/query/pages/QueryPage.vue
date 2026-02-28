<script setup lang="ts">
/** 文件：QueryPage.vue | 用途：渲染季度查询页主界面与详情面板 | 关键对象：props, progressPercent, normalizeRating, ratingTone */
// 本行目的：引入 Vue 组合式 API，用于计算属性、挂载钩子和侦听。
import { computed, onMounted, watch } from "vue";
// 本行目的：引入条目类型，约束 tracking 操作函数的参数。
import type { MonthAnime } from "../../tracking/types/anime";
// 本行目的：引入追番状态类型，约束状态按钮逻辑。
import type { ItemStatus, StatusKey } from "../../tracking/types/tracking";
// 本行目的：引入查询页组合逻辑返回类型。
import type { UseQueryPageReturn } from "../composables/useQueryPage";
// 本行目的：引入日期与评分格式化工具。
import { formatAirDate, formatRating, formatStars } from "../../../shared/utils/format";
// 本行目的：引入国际化函数与语言键类型。
import { t, type LocaleKey } from "../../../shared/i18n/messages";
// 本行目的：引入 staff 弹窗组件。
import StaffModal from "../components/StaffModal.vue";

// 变量：props | 含义：页面组件输入参数集合 | 类型：defineProps 返回对象 | 作用域：组件内
// 本行目的：声明组件 props 类型。
const props = defineProps<{
    // 本行目的：当前语言键。
    locale: LocaleKey;
    // 本行目的：查询页组合状态与方法集合。
    query: UseQueryPageReturn;
    // 本行目的：追番状态操作集合。
    tracking: {
        // 本行目的：当前已选状态集合。
        selectedStatus: ItemStatus;
        // 本行目的：把状态对象映射为状态键。
        currentStatusKey: (status: ItemStatus) => StatusKey;
        // 本行目的：生成状态按钮文案。
        labelForAction: (active: StatusKey, target: StatusKey) => string;
        // 本行目的：设置互斥状态（三选一）。
        setExclusiveStatus: (item: MonthAnime, target: StatusKey) => void | Promise<void>;
    };
    // 本行目的：打开外部链接函数。
    openExternalLink: (url?: string | null) => void | Promise<void>;
    // 本行目的：结束 props 类型定义。
}>();

/** 函数：onMounted回调 | 输入：无 | 输出：无（触发副作用） | 可能失败：DOM 未稳定时高度计算可能无效 */
// 本行目的：组件初次挂载后同步三栏高度。
onMounted(() => {
    // 本行目的：调用查询组合函数刷新面板高度。
    void props.query.setQueryPanelHeight();
});

/** 函数：watch(showResults) | 输入：结果区可见状态 | 输出：无（触发副作用） | 可能失败：无 */
// 本行目的：结果区显示时重新计算面板高度。
watch(
    // 本行目的：侦听结果可见状态。
    () => props.query.showResults.value,
    // 本行目的：可见性变化回调。
    (visible) => {
        // 本行目的：仅在可见时执行高度同步。
        if (visible) {
            // 本行目的：调用高度同步函数。
            void props.query.setQueryPanelHeight();
        }
    }
);

/** 函数：watch(filterLoading) | 输入：筛选加载状态 | 输出：无（触发副作用） | 可能失败：无 */
// 本行目的：筛选数据加载结束后重新计算面板高度。
watch(
    // 本行目的：侦听筛选加载状态。
    () => props.query.filterLoading.value,
    // 本行目的：加载状态变化回调。
    (loading) => {
        // 本行目的：仅在加载结束且结果区可见时同步高度。
        if (!loading && props.query.showResults.value) {
            // 本行目的：调用高度同步函数。
            void props.query.setQueryPanelHeight();
        }
    }
);

// 变量：progressPercent | 含义：进度条组件使用的 0~1 比例值 | 类型：ComputedRef<number> | 作用域：组件内
/** 函数：progressPercent | 输入：无 | 输出：归一化进度比例 | 可能失败：无（纯计算） */
// 本行目的：把查询进度（0~100）转换为进度条百分比（0~1）。
const progressPercent = computed(() => Math.min(100, props.query.progress.value) / 100);

// 变量：normalizeRating | 含义：把评分值规范为有效数字或空 | 类型：(value?: number | null) => number | null | 作用域：组件内
/** 函数：normalizeRating | 输入：评分原值 | 输出：有效数字或 null | 可能失败：无（纯转换） */
// 本行目的：统一处理评分空值与非法数字。
const normalizeRating = (value?: number | null) => {
    // 本行目的：空值直接返回 null。
    if (value == null) return null;
    // 本行目的：尝试转成 number。
    const num = Number(value);
    // 本行目的：仅返回有限数字，否则返回 null。
    return Number.isFinite(num) ? num : null;
};

// 变量：ratingTone | 含义：根据评分返回样式语义类名 | 类型：(value?: number | null) => string | 作用域：组件内
/** 函数：ratingTone | 输入：评分值 | 输出：评分色调类名 | 可能失败：无（纯判断） */
// 本行目的：定义评分等级到 CSS 类名的映射。
const ratingTone = (value?: number | null) => {
    // 本行目的：先标准化评分值。
    const normalized = normalizeRating(value);
    // 本行目的：无评分返回中性样式。
    if (normalized == null) return "rating-neutral";
    // 本行目的：高分返回高亮样式。
    if (normalized >= 8) return "rating-high";
    // 本行目的：中等分返回中等级样式。
    if (normalized >= 6) return "rating-medium";
    // 本行目的：低分返回低分样式。
    return "rating-low";
};

// 变量：ratingColor | 含义：根据评分返回对应颜色值 | 类型：(value?: number | null) => string | 作用域：组件内
/** 函数：ratingColor | 输入：评分值 | 输出：颜色字符串 | 可能失败：无（纯判断） */
// 本行目的：定义评分等级到颜色值的映射。
const ratingColor = (value?: number | null) => {
    // 本行目的：先标准化评分值。
    const normalized = normalizeRating(value);
    // 本行目的：无评分返回中性灰。
    if (normalized == null) return "#6b7280";
    // 本行目的：高分返回绿色。
    if (normalized >= 8) return "#16a34a";
    // 本行目的：中等分返回橙色。
    if (normalized >= 6) return "#f59e0b";
    // 本行目的：低分返回红色。
    return "#ef4444";
};

// 变量：linkColor | 含义：链接文本统一强调色 | 类型：string | 作用域：组件内
// 本行目的：定义可点击链接颜色。
const linkColor = "#16a34a";

// 变量：ratingColorStyle | 含义：评分颜色内联样式生成函数 | 类型：(value?: number | null) => string | 作用域：组件内
/** 函数：ratingColorStyle | 输入：评分值 | 输出：CSS 样式字符串 | 可能失败：无（纯拼接） */
// 本行目的：生成带 !important 的评分色样式。
const ratingColorStyle = (value?: number | null) => {
    // 本行目的：计算评分对应颜色。
    const color = ratingColor(value);
    // 本行目的：返回兼容 WebKit 的文本颜色样式。
    return `color: ${color} !important; -webkit-text-fill-color: ${color} !important;`;
};

// 变量：linkColorStyle | 含义：详情链接统一内联颜色样式 | 类型：string | 作用域：组件内
// 本行目的：生成链接样式字符串。
const linkColorStyle = `color: ${linkColor} !important; -webkit-text-fill-color: ${linkColor} !important;`;
// 变量：titleColorStyle | 含义：详情标题固定颜色样式 | 类型：string | 作用域：组件内
// 本行目的：生成标题样式字符串。
const titleColorStyle = `color: #111111 !important; -webkit-text-fill-color: #111111 !important;`;
</script>

<template>
    <!-- 本行目的：页面根容器，根据是否查询过切换布局模式。 -->
    <div class="app-body" :class="{ 'results-view': query.hasQueried.value, 'query-centered': !query.hasQueried.value }">
        <!-- 本行目的：顶部三栏区域（查询、摘要、筛选）。 -->
        <section class="query-summary-row">
            <!-- 本行目的：查询输入面板。 -->
            <section :ref="query.queryPanelRef" class="query-panel">
                <!-- 本行目的：查询面板卡片。 -->
                <a-card :title="t('query.panel.title', locale)" size="small">
                    <!-- 本行目的：查询控件横向布局容器。 -->
                    <a-space class="query-actions" align="center" :size="12">
                        <!-- 本行目的：年份选择器。 -->
                        <a-select
                            v-model="query.year.value"
                            :options="query.yearOptions"
                            :placeholder="t('query.year.placeholder', locale)"
                            allow-clear
                        />
                        <!-- 本行目的：季度选择器。 -->
                        <a-select
                            v-model="query.month.value"
                            :options="query.seasonOptions.value"
                            :placeholder="t('query.season.placeholder', locale)"
                            allow-clear
                        />
                        <!-- 本行目的：触发查询按钮，加载中显示 loading。 -->
                        <a-button type="primary" :loading="query.loading.value" @click="query.handleQuery">
                            {{ t("query.action.search", locale) }}
                        </a-button>
                    </a-space>
                </a-card>
            </section>
            <!-- 本行目的：查询完成后显示摘要信息面板。 -->
            <section v-if="query.showResults.value" :ref="query.summaryPanelRef" class="summary-panel">
                <!-- 本行目的：结果摘要卡片。 -->
                <a-card size="small" class="result-summary">
                    <!-- 本行目的：摘要字段纵向排列。 -->
                    <a-space direction="vertical" :size="8">
                        <!-- 本行目的：展示数据来源地址。 -->
                        <span>{{ t("query.summary.source", locale) }}：{{ query.resultUrl }}</span>
                        <!-- 本行目的：展示当前查询季节信息。 -->
                        <span>
                            {{ t("query.summary.season", locale) }}：{{ query.year.value }}{{ t("query.summary.year", locale) }}
                            {{ query.seasonLabelFromMonth(query.month.value) }}
                        </span>
                        <!-- 本行目的：展示筛选后的条目数量。 -->
                        <span>{{ t("query.summary.count", locale) }}：{{ query.resultCount.value }}</span>
                    </a-space>
                </a-card>
            </section>
            <!-- 本行目的：查询完成后显示筛选面板。 -->
            <section v-if="query.showResults.value" :ref="query.filterPanelRef" class="filter-panel">
                <!-- 本行目的：筛选卡片容器。 -->
                <a-card size="small">
                    <!-- 本行目的：筛选项网格布局。 -->
                    <div class="filter-grid">
                        <!-- 本行目的：月份筛选字段。 -->
                        <div class="filter-field">
                            <!-- 本行目的：月份筛选标签。 -->
                            <span class="filter-label">{{ t("query.filter.month", locale) }}</span>
                            <!-- 本行目的：月份筛选控件与状态标签容器。 -->
                            <div class="filter-select">
                                <!-- 本行目的：月份多选下拉。 -->
                                <a-select
                                    :model-value="query.monthFilter.value"
                                    :options="query.monthFilterOptions.value"
                                    multiple
                                    placeholder=""
                                    :disabled="query.filterLoading.value"
                                    :max-tag-count="0"
                                    @update:model-value="query.handleMonthFilterChange"
                                />
                                <!-- 本行目的：月份筛选状态标签。 -->
                                <div class="filter-select-chip">
                                    <a-tag size="small" color="arcoblue">{{ t(query.monthFilterLabel.value, locale) }}</a-tag>
                                </div>
                            </div>
                        </div>
                        <!-- 本行目的：题材筛选字段。 -->
                        <div class="filter-field">
                            <span class="filter-label">{{ t("query.filter.type", locale) }}</span>
                            <div class="filter-select">
                                <a-select
                                    :model-value="query.typeFilter.value"
                                    :options="query.typeOptions.value"
                                    multiple
                                    placeholder=""
                                    :disabled="query.filterLoading.value"
                                    :max-tag-count="0"
                                    @update:model-value="query.handleTypeFilterChange"
                                />
                                <!-- 本行目的：题材筛选状态标签。 -->
                                <div class="filter-select-chip">
                                    <a-tag size="small" color="arcoblue">{{ t(query.typeFilterLabel.value, locale) }}</a-tag>
                                </div>
                            </div>
                        </div>
                        <!-- 本行目的：地区筛选字段。 -->
                        <div class="filter-field">
                            <span class="filter-label">{{ t("query.filter.region", locale) }}</span>
                            <div class="filter-select">
                                <a-select
                                    :model-value="query.regionFilter.value"
                                    :options="query.regionOptions.value"
                                    multiple
                                    placeholder=""
                                    :disabled="query.filterLoading.value"
                                    :max-tag-count="0"
                                    @update:model-value="query.handleRegionFilterChange"
                                />
                                <!-- 本行目的：地区筛选状态标签。 -->
                                <div class="filter-select-chip">
                                    <a-tag size="small" color="arcoblue">{{ t(query.regionFilterLabel.value, locale) }}</a-tag>
                                </div>
                            </div>
                        </div>
                        <!-- 本行目的：受众筛选字段。 -->
                        <div class="filter-field">
                            <span class="filter-label">{{ t("query.filter.audience", locale) }}</span>
                            <div class="filter-select">
                                <a-select
                                    :model-value="query.audienceFilter.value"
                                    :options="query.audienceOptions.value"
                                    multiple
                                    placeholder=""
                                    :disabled="query.filterLoading.value"
                                    :max-tag-count="0"
                                    @update:model-value="query.handleAudienceFilterChange"
                                />
                                <!-- 本行目的：受众筛选状态标签。 -->
                                <div class="filter-select-chip">
                                    <a-tag size="small" color="arcoblue">{{ t(query.audienceFilterLabel.value, locale) }}</a-tag>
                                </div>
                            </div>
                        </div>
                    </div>
                    <!-- 本行目的：筛选元数据加载时显示提示。 -->
                    <span v-if="query.filterLoading.value" class="filter-loading">
                        {{ t("query.filter.loading", locale) }}
                    </span>
                </a-card>
            </section>
        </section>
        <!-- 本行目的：查询发起后显示全局进度条。 -->
        <section v-if="query.hasQueried.value" class="progress-panel">
            <a-progress :percent="progressPercent" :format="(value: number) => `${Math.round(value * 100)}%`" status="normal" />
        </section>
        <!-- 本行目的：结果展示主区域。 -->
        <section v-if="query.showResults.value" class="result-panel">
            <div class="result-content">
                <!-- 本行目的：有结果时展示列表+详情双栏布局。 -->
                <div v-if="query.filteredResults.value.length" class="result-layout">
                    <!-- 本行目的：结果列表卡片。 -->
                    <a-card :title="t('query.list.title', locale)" size="small" class="result-list">
                        <!-- 本行目的：列表滚动容器，挂载鼠标进出行为。 -->
                        <div
                            :ref="query.resultListRef"
                            class="result-list-grid"
                            @mouseenter="query.handleListMouseEnter"
                            @mouseleave="query.handleListMouseLeave"
                        >
                            <!-- 本行目的：遍历筛选后的条目列表。 -->
                            <div
                                v-for="item in query.filteredResults.value"
                                :key="item.id ?? item.name"
                                class="result-list-item"
                                role="button"
                                tabindex="0"
                                :ref="(el: unknown) => query.setListItemRef(item, el as HTMLElement | null)"
                                @click="query.handleSelect(item)"
                                @keydown.enter.prevent="query.handleSelect(item)"
                                @keydown.space.prevent="query.handleSelect(item)"
                            >
                                <!-- 本行目的：条目缩略图区域。 -->
                                <div class="result-list-thumb">
                                    <img :src="item.image" :alt="item.name" />
                                </div>
                                <!-- 本行目的：条目文本信息区域。 -->
                                <div class="result-list-info">
                                    <div class="result-list-title">{{ item.nameCn || item.name }}</div>
                                </div>
                            </div>
                        </div>
                    </a-card>
                    <!-- 本行目的：有选中条目时展示详情面板。 -->
                    <a-card v-if="query.selected.value" :title="t('query.detail.title', locale)" size="small" class="detail-panel">
                        <!-- 本行目的：详情面板主体布局容器。 -->
                        <div class="detail-panel-body">
                            <!-- 本行目的：详情海报区域。 -->
                            <div class="detail-media">
                                <img class="detail-image" :src="query.selected.value?.image" :alt="query.selected.value?.name" />
                            </div>
                            <!-- 本行目的：详情信息区。 -->
                            <div class="detail-info">
                                <!-- 本行目的：可点击标题，跳转到外部详情页。 -->
                                <span
                                    class="detail-title detail-title-link detail-link-text"
                                    role="link"
                                    tabindex="0"
                                    :style="titleColorStyle"
                                    @click="openExternalLink(query.selected.value?.url)"
                                    @keydown.enter.prevent="openExternalLink(query.selected.value?.url)"
                                >
                                    {{ query.selected.value?.nameCn || query.selected.value?.name }}
                                </span>
                                <!-- 本行目的：追番状态切换按钮组。 -->
                                <div class="detail-actions">
                                    <!-- 本行目的：切换为在看状态。 -->
                                    <a-button
                                        size="mini"
                                        :type="tracking.selectedStatus.watching ? 'primary' : 'outline'"
                                        @click="query.selected.value && tracking.setExclusiveStatus(query.selected.value, 'watching')"
                                    >
                                        {{ tracking.labelForAction(tracking.currentStatusKey(tracking.selectedStatus), "watching") }}
                                    </a-button>
                                    <!-- 本行目的：切换为想看状态。 -->
                                    <a-button
                                        size="mini"
                                        :type="tracking.selectedStatus.backlog ? 'primary' : 'outline'"
                                        @click="query.selected.value && tracking.setExclusiveStatus(query.selected.value, 'backlog')"
                                    >
                                        {{ tracking.labelForAction(tracking.currentStatusKey(tracking.selectedStatus), "backlog") }}
                                    </a-button>
                                    <!-- 本行目的：切换为看过状态。 -->
                                    <a-button
                                        size="mini"
                                        :type="tracking.selectedStatus.watched ? 'primary' : 'outline'"
                                        @click="query.selected.value && tracking.setExclusiveStatus(query.selected.value, 'watched')"
                                    >
                                        {{ tracking.labelForAction(tracking.currentStatusKey(tracking.selectedStatus), "watched") }}
                                    </a-button>
                                </div>
                                <!-- 本行目的：详情字段列表。 -->
                                <div class="detail-info-list">
                                    <!-- 本行目的：原名字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.originName", locale) }}</span>
                                        <span class="detail-value">{{ query.selected.value?.name }}</span>
                                    </div>
                                    <!-- 本行目的：评分字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.rating", locale) }}</span>
                                        <span class="detail-value detail-rating">
                                            <!-- 本行目的：星级展示（含颜色和等级样式）。 -->
                                            <span
                                                class="detail-stars"
                                                :class="ratingTone(query.selected.value?.rating)"
                                                :style="ratingColorStyle(query.selected.value?.rating)"
                                            >
                                                {{ formatStars(normalizeRating(query.selected.value?.rating)) }}
                                            </span>
                                            <!-- 本行目的：数字评分展示。 -->
                                            <span class="detail-score">
                                                {{ formatRating(normalizeRating(query.selected.value?.rating)) }}
                                            </span>
                                        </span>
                                    </div>
                                    <!-- 本行目的：首播日期与已播集数字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.airDate", locale) }}</span>
                                        <span class="detail-value">
                                            {{ formatAirDate(query.selected.value?.date) }}
                                            <span class="detail-divider">·</span>
                                            <!-- 本行目的：已播数据加载中时显示 loading 文案。 -->
                                            <template v-if="query.airedLoadingId.value === query.selected.value?.id">
                                                {{ t("query.detail.aired.loading", locale) }}
                                            </template>
                                            <!-- 本行目的：已播数据失败且无历史值时显示错误文案。 -->
                                            <template v-else-if="query.airedError.value && query.selected.value?.airedCount == null">
                                                {{ t("query.detail.aired.error", locale) }}
                                            </template>
                                            <!-- 本行目的：正常显示已播集数。 -->
                                            <template v-else>
                                                {{ t("query.detail.aired.count", locale) }} {{ query.selected.value?.airedCount ?? 0 }}
                                                {{ t("query.detail.aired.unit", locale) }}
                                            </template>
                                        </span>
                                    </div>
                                    <!-- 本行目的：原作来源字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.origin", locale) }}</span>
                                        <span class="detail-value">
                                            <!-- 本行目的：原作数据加载中时显示 loading。 -->
                                            <template v-if="query.originLoadingId.value === query.selected.value?.id">
                                                {{ t("query.detail.origin.loading", locale) }}
                                            </template>
                                            <!-- 本行目的：原作加载失败且无值时显示错误。 -->
                                            <template v-else-if="query.originError.value && !query.selected.value?.origin">
                                                {{ t("query.detail.origin.error", locale) }}
                                            </template>
                                            <!-- 本行目的：显示原作值或未知文案。 -->
                                            <template v-else>
                                                {{ query.selected.value?.origin || t("query.detail.origin.unknown", locale) }}
                                            </template>
                                        </span>
                                    </div>
                                    <!-- 本行目的：staff 查看入口字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.staff", locale) }}</span>
                                        <span class="detail-value">
                                            <a-button size="mini" type="primary" @click="query.handleStaffOpen">
                                                {{ t("query.detail.staff.action", locale) }}
                                            </a-button>
                                        </span>
                                    </div>
                                    <!-- 本行目的：角色字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.characters", locale) }}</span>
                                        <span class="detail-value">
                                            <!-- 本行目的：角色加载中状态。 -->
                                            <template v-if="query.characterLoadingId.value === query.selected.value?.id">
                                                {{ t("query.detail.characters.loading", locale) }}
                                            </template>
                                            <!-- 本行目的：角色加载失败且无数据。 -->
                                            <template v-else-if="query.characterError.value && !query.selectedCharacters.value.length">
                                                {{ t("query.detail.characters.error", locale) }}
                                            </template>
                                            <!-- 本行目的：角色为空状态。 -->
                                            <template v-else-if="!query.selectedCharacters.value.length">
                                                {{ t("query.detail.characters.empty", locale) }}
                                            </template>
                                            <!-- 本行目的：角色列表展示。 -->
                                            <span v-else class="detail-link-list">
                                                <!-- 本行目的：遍历角色并渲染可点击链接。 -->
                                                <span
                                                    v-for="item in query.selectedCharacters.value"
                                                    :key="item.id"
                                                    class="detail-link detail-link-text"
                                                    role="link"
                                                    tabindex="0"
                                                    :style="linkColorStyle"
                                                    @click="openExternalLink(item.url)"
                                                    @keydown.enter.prevent="openExternalLink(item.url)"
                                                >
                                                    {{ item.name }}
                                                </span>
                                            </span>
                                        </span>
                                    </div>
                                    <!-- 本行目的：简介字段。 -->
                                    <div class="detail-info-row">
                                        <span class="detail-label">{{ t("query.detail.summary", locale) }}</span>
                                        <span class="detail-value detail-summary">
                                            <!-- 本行目的：简介加载中状态。 -->
                                            <template v-if="query.summaryLoadingId.value === query.selected.value?.id">
                                                {{ t("query.detail.summary.loading", locale) }}
                                            </template>
                                            <!-- 本行目的：简介失败且无内容时显示错误。 -->
                                            <template v-else-if="query.summaryError.value && !query.selected.value?.summaryCn">
                                                {{ t("query.detail.summary.error", locale) }}：{{ query.summaryError.value }}
                                            </template>
                                            <!-- 本行目的：简介为空状态。 -->
                                            <template v-else-if="!query.selected.value?.summaryCn">
                                                {{ t("query.detail.summary.empty", locale) }}
                                            </template>
                                            <!-- 本行目的：简介正常展示并显示翻译状态标签。 -->
                                            <template v-else>
                                                {{ query.selected.value?.summaryCn }}
                                                <!-- 本行目的：翻译成功标签。 -->
                                                <span v-if="query.selected.value?.summaryTranslated" class="detail-translate-tag">
                                                    {{ t("query.detail.summary.translated", locale) }}
                                                </span>
                                                <!-- 本行目的：翻译失败时附加错误提示。 -->
                                                <span v-else-if="query.summaryError.value" class="detail-translate-tag">
                                                    （{{ query.summaryError.value }}）
                                                </span>
                                            </template>
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </a-card>
                </div>
                <!-- 本行目的：无筛选结果时显示空结果组件。 -->
                <a-result v-else status="info" :title="t('query.empty.title', locale)" :subtitle="t('query.empty.subtitle', locale)" />
            </div>
        </section>
        <!-- 本行目的：staff 弹窗组件，承接 show/update 双向可见性。 -->
        <StaffModal
            :show="query.showStaffModal.value"
            :locale="locale"
            :selected="query.selected.value"
            :staff-loading-id="query.staffLoadingId.value"
            :staff-error="query.staffError.value"
            :selected-staff="query.selectedStaff.value"
            :open-external-link="openExternalLink"
            @update:show="(value: boolean) => (query.showStaffModal.value = value)"
        />
    </div>
</template>
