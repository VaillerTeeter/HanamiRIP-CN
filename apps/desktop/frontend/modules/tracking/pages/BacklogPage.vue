<script setup lang="ts">
/** 文件：BacklogPage.vue | 用途：展示“想看”追踪列表并提供状态切换操作 | 关键对象：props, starTone */
// 本行目的：引入月番类型，约束状态切换回调参数。
import type { MonthAnime } from "../types/anime";
// 本行目的：引入追踪条目类型，约束列表渲染数据结构。
import type { TrackedItem } from "../types/tracking";
// 本行目的：引入日期/评分格式化工具，用于界面展示友好文本。
import { formatAirDate, formatRating, formatStars } from "../../../shared/utils/format";
// 本行目的：引入国际化函数与语言键类型，渲染多语言文案。
import { t, type LocaleKey } from "../../../shared/i18n/messages";

// 变量：props | 含义：父层传入的语言、想看列表与交互回调集合 | 类型：由 defineProps 推断的只读对象 | 作用域：BacklogPage 组件内
/** 函数：defineProps | 输入：locale、backlogList、setExclusiveStatus、openExternalLink | 输出：组件 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：定义页面所需输入数据与行为回调。
const props = defineProps<{
    // 本行目的：声明当前页面使用的语言键。
    locale: LocaleKey;
    // 本行目的：声明“想看”条目列表数据。
    backlogList: TrackedItem[];
    // 本行目的：声明条目状态互斥切换回调。
    setExclusiveStatus: (item: MonthAnime, target: "watching" | "backlog" | "watched" | null) => void | Promise<void>;
    // 本行目的：声明统一外链打开回调。
    openExternalLink: (url?: string | null) => void | Promise<void>;
}>();

// 变量：starTone | 含义：根据评分返回星级文本样式类名 | 类型：(value?: number | null) => string | 作用域：BacklogPage 组件内
/** 函数：starTone | 输入：可选评分值 | 输出：rating-neutral/high/medium/low 样式类 | 可能失败：无 */
// 本行目的：定义评分颜色分级规则，统一星级视觉反馈。
const starTone = (value?: number | null) => {
    // 本行目的：无评分时返回中性样式。
    if (value == null) return "rating-neutral";
    // 本行目的：8 分及以上返回高分样式。
    if (value >= 8) return "rating-high";
    // 本行目的：6 分及以上返回中分样式。
    if (value >= 6) return "rating-medium";
    // 本行目的：其余情况返回低分样式。
    return "rating-low";
};
</script>

<template>
    <!-- 本行目的：定义“想看”页主容器。 -->
    <main class="tracking-backlog">
        <!-- 本行目的：有数据时渲染网格列表。 -->
        <div v-if="props.backlogList.length" class="watchlist-grid">
            <!-- 本行目的：遍历想看条目并渲染卡片。 -->
            <div v-for="item in props.backlogList" :key="item.id" class="watchlist-item">
                <!-- 本行目的：展示条目封面区域。 -->
                <div class="watchlist-thumb">
                    <!-- 本行目的：渲染封面图片并提供替代文本。 -->
                    <img :src="item.image" :alt="item.name" />
                </div>
                <!-- 本行目的：展示条目文本信息与操作按钮。 -->
                <div class="watchlist-body">
                    <!-- 本行目的：渲染条目标题链接，点击走统一外链处理。 -->
                    <a
                        class="watchlist-title"
                        :href="item.url || `https://bgm.tv/subject/${item.id}`"
                        target="_blank"
                        rel="noreferrer"
                        @click.prevent="props.openExternalLink(item.url)"
                    >
                        <!-- 本行目的：优先显示中文名，缺失时显示原名。 -->
                        {{ item.nameCn || item.name }}
                    </a>
                    <!-- 本行目的：展示放送日期与评分元信息。 -->
                    <div class="watchlist-meta">
                        <!-- 本行目的：显示本地化后的放送日期。 -->
                        <span class="meta-row"> {{ t("tracking.backlog.airDate", locale) }}：{{ formatAirDate(item.date) }} </span>
                        <!-- 本行目的：显示评分文本与星级符号。 -->
                        <span class="meta-row">
                            {{ t("tracking.backlog.rating", locale) }}：{{ formatRating(item.rating) }}
                            <!-- 本行目的：根据评分动态应用星级颜色样式。 -->
                            <span class="watchlist-stars" :class="starTone(item.rating)">
                                <!-- 本行目的：显示评分星级字符表示。 -->
                                {{ formatStars(item.rating) }}
                            </span>
                        </span>
                    </div>
                    <!-- 本行目的：展示状态切换操作按钮组。 -->
                    <div class="watchlist-actions">
                        <!-- 本行目的：当前页对应 backlog 状态按钮置灰不可点。 -->
                        <a-button size="mini" disabled>
                            {{ t("nav.backlog", locale) }}
                        </a-button>
                        <!-- 本行目的：点击切换条目状态为 watching。 -->
                        <a-button size="mini" @click="props.setExclusiveStatus(item, 'watching')">
                            {{ t("tracking.action.switch.watching", locale) }}
                        </a-button>
                        <!-- 本行目的：点击切换条目状态为 watched。 -->
                        <a-button size="mini" type="primary" @click="props.setExclusiveStatus(item, 'watched')">
                            {{ t("tracking.action.switch.watched", locale) }}
                        </a-button>
                    </div>
                </div>
            </div>
        </div>
        <!-- 本行目的：无数据时展示空状态。 -->
        <div v-else class="tracking-backlog__empty">
            <!-- 本行目的：使用统一空状态组件承载提示文案。 -->
            <a-empty>
                <!-- 本行目的：自定义空状态描述内容。 -->
                <template #description>
                    <!-- 本行目的：包裹空状态标题与副标题。 -->
                    <div class="tracking-backlog__empty-text">
                        <!-- 本行目的：显示空状态主标题。 -->
                        <div class="tracking-backlog__empty-title">
                            {{ t("tracking.backlog.empty.title", locale) }}
                        </div>
                        <!-- 本行目的：显示空状态补充说明。 -->
                        <div class="tracking-backlog__empty-subtitle">
                            {{ t("tracking.backlog.empty.subtitle", locale) }}
                        </div>
                    </div>
                </template>
            </a-empty>
        </div>
    </main>
</template>
