<script setup lang="ts">
/** 文件：FinishedPage.vue | 用途：展示“看完”追踪列表并提供状态重置操作 | 关键对象：props, starColor */
// 本行目的：引入月番条目类型，约束状态切换回调参数。
import type { MonthAnime } from "../types/anime";
// 本行目的：引入追踪条目类型，约束看完列表的数据结构。
import type { TrackedItem } from "../types/tracking";
// 本行目的：引入日期与评分格式化工具，统一展示格式。
import { formatAirDate, formatRating, formatStars } from "../../../shared/utils/format";
// 本行目的：引入国际化函数和语言键类型，支持多语言文案。
import { t, type LocaleKey } from "../../../shared/i18n/messages";

// 变量：props | 含义：父组件传入的语言、看完列表与交互回调 | 类型：由 defineProps 推断的只读对象 | 作用域：FinishedPage 组件内
/** 函数：defineProps | 输入：locale、finishedList、setExclusiveStatus、openExternalLink | 输出：组件 props 对象 | 可能失败：无（编译期类型校验） */
// 本行目的：声明页面依赖的数据与操作能力。
const props = defineProps<{
    // 本行目的：声明当前界面使用的语言键。
    locale: LocaleKey;
    // 本行目的：声明“看完”条目列表数据源。
    finishedList: TrackedItem[];
    // 本行目的：声明互斥状态切换回调。
    setExclusiveStatus: (item: MonthAnime, target: "watching" | "backlog" | "watched" | null) => void | Promise<void>;
    // 本行目的：声明统一外链打开回调。
    openExternalLink: (url?: string | null) => void | Promise<void>;
}>();

// 变量：starColor | 含义：按评分返回星级颜色值 | 类型：(value?: number | null) => string | 作用域：FinishedPage 组件内
/** 函数：starColor | 输入：可选评分值 | 输出：对应颜色字符串 | 可能失败：无 */
// 本行目的：根据评分区间决定星级文本颜色。
const starColor = (value?: number | null) => {
    // 本行目的：无评分时返回中性色。
    if (value == null) return "var(--color-text-3)";
    // 本行目的：8 分及以上返回高分绿色。
    if (value >= 8) return "#00b42a";
    // 本行目的：6 分及以上返回中分黄色。
    if (value >= 6) return "#f7ba1e";
    // 本行目的：其余分数返回低分红色。
    return "#f53f3f";
};
</script>

<template>
    <!-- 本行目的：定义“看完”页主容器。 -->
    <main class="tracking-finished">
        <!-- 本行目的：有看完条目时渲染网格列表。 -->
        <div v-if="props.finishedList.length" class="watchlist-grid">
            <!-- 本行目的：遍历看完列表并渲染每个条目卡片。 -->
            <div v-for="item in props.finishedList" :key="item.id" class="watchlist-item">
                <!-- 本行目的：展示条目封面图区域。 -->
                <div class="watchlist-thumb">
                    <!-- 本行目的：渲染封面图片并设置替代文本。 -->
                    <img :src="item.image" :alt="item.name" />
                </div>
                <!-- 本行目的：展示标题、元信息与动作按钮区域。 -->
                <div class="watchlist-body">
                    <!-- 本行目的：渲染标题外链，点击时走统一外链打开逻辑。 -->
                    <a
                        class="watchlist-title"
                        :href="item.url || `https://bgm.tv/subject/${item.id}`"
                        target="_blank"
                        rel="noreferrer"
                        @click.prevent="props.openExternalLink(item.url)"
                    >
                        <!-- 本行目的：优先显示中文名，缺失时回退原名。 -->
                        {{ item.nameCn || item.name }}
                    </a>
                    <!-- 本行目的：展示日期与评分的元信息块。 -->
                    <div class="watchlist-meta">
                        <!-- 本行目的：显示本地化放送日期。 -->
                        <span class="meta-row"> {{ t("tracking.finished.airDate", props.locale) }}：{{ formatAirDate(item.date) }} </span>
                        <!-- 本行目的：显示评分文本与星级符号。 -->
                        <span class="meta-row">
                            {{ t("tracking.finished.rating", props.locale) }}：{{ formatRating(item.rating) }}
                            <!-- 本行目的：根据评分动态设置星级颜色。 -->
                            <span class="watchlist-stars" :style="{ color: starColor(item.rating) }">
                                <!-- 本行目的：显示星级符号表示。 -->
                                {{ formatStars(item.rating) }}
                            </span>
                        </span>
                    </div>
                    <!-- 本行目的：展示状态操作按钮组。 -->
                    <div class="watchlist-actions">
                        <!-- 本行目的：Watching 按钮在本页面仅用于提示，禁用交互。 -->
                        <a-button size="mini" disabled>
                            {{ t("nav.watching", props.locale) }}
                        </a-button>
                        <!-- 本行目的：Backlog 按钮在本页面仅用于提示，禁用交互。 -->
                        <a-button size="mini" disabled>
                            {{ t("nav.backlog", props.locale) }}
                        </a-button>
                        <!-- 本行目的：点击后调用互斥状态设置，保持 watched 状态。 -->
                        <a-button size="mini" type="primary" @click="props.setExclusiveStatus(item, 'watched')">
                            {{ t("tracking.finished.action.reset", props.locale) }}
                        </a-button>
                    </div>
                </div>
            </div>
        </div>
        <!-- 本行目的：无看完条目时渲染空状态。 -->
        <div v-else class="tracking-finished__empty">
            <!-- 本行目的：使用空状态组件承载说明信息。 -->
            <a-empty>
                <!-- 本行目的：自定义空状态描述区域。 -->
                <template #description>
                    <!-- 本行目的：包裹空状态文案容器。 -->
                    <div class="tracking-finished__empty-text">
                        <!-- 本行目的：展示空状态标题文案。 -->
                        <div class="tracking-finished__empty-title">
                            {{ t("tracking.finished.empty.title", props.locale) }}
                        </div>
                        <!-- 本行目的：展示空状态副标题文案。 -->
                        <div class="tracking-finished__empty-subtitle">
                            {{ t("tracking.finished.empty.subtitle", props.locale) }}
                        </div>
                    </div>
                </template>
            </a-empty>
        </div>
    </main>
</template>
