<script setup lang="ts">
/** 文件：WatchingPage.vue | 用途：按星期时间线展示“在看”条目并提供状态切换与外链打开 | 关键对象：props, orderedGroups, weekdayLabel, latestEpisode */
// 本行目的：引入 computed/onMounted/ref/watch，用于派生数据、初始化与响应同步。
import { computed, onMounted, ref, watch } from "vue";
// 本行目的：引入 invoke，用于调用后端获取本地星期索引。
import { invoke } from "@tauri-apps/api/core";
// 本行目的：引入 MonthAnime 类型，约束状态切换函数参数。
import type { MonthAnime } from "../types/anime";
// 本行目的：引入状态键与追踪条目类型。
import type { StatusKey, TrackedItem } from "../types/tracking";
// 本行目的：引入日期/评分/星级格式化工具。
import { formatAirDate, formatRating, formatStars } from "../../../shared/utils/format";
// 本行目的：引入国际化函数与语言类型。
import { t, type LocaleKey } from "../../../shared/i18n/messages";

// 变量：props | 含义：父组件传入的在看页面数据与行为集合 | 类型：由 defineProps 推断的对象 | 作用域：WatchingPage 组件内
/** 函数：defineProps | 输入：locale、watchingByWeekday、状态切换/刷新/外链回调等 | 输出：组件 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：声明页面依赖的输入数据和方法。
const props = defineProps<{
    // 本行目的：声明当前页面语言。
    locale: LocaleKey;
    // 本行目的：声明按星期分组的在看列表。
    watchingByWeekday: Array<{ key: number | null; items: TrackedItem[] }>;
    // 本行目的：声明互斥状态切换方法。
    setExclusiveStatus: (item: MonthAnime, target: StatusKey) => Promise<void> | void;
    // 本行目的：声明刷新在看详情的方法。
    refreshWatchingDetails: (options?: { persist?: boolean }) => Promise<void> | void;
    // 本行目的：声明刷新星期起点的方法。
    refreshWeekdayOrder: () => void;
    // 本行目的：声明星期顺序更新种子值（用于触发 watch）。
    weekdayOrderSeed: number;
    // 本行目的：声明统一外链打开方法。
    openExternalLink: (url: string) => void;
}>();

// 变量：weekdayStartIndex | 含义：时间线显示顺序的起始星期（0-6） | 类型：Ref<number> | 作用域：WatchingPage 组件内
// 本行目的：初始化星期起始值为当前本地星期。
const weekdayStartIndex = ref(new Date().getDay());

// 变量：syncWeekdayStart | 含义：同步本地星期起始索引 | 类型：() => Promise<void> | 作用域：WatchingPage 组件内
/** 函数：syncWeekdayStart | 输入：无 | 输出：无（更新 weekdayStartIndex） | 可能失败：后端命令失败时回退浏览器日期 */
// 本行目的：定义星期起始同步逻辑。
const syncWeekdayStart = async () => {
    try {
        // 本行目的：调用后端获取更准确的本地星期。
        const weekday = await invoke<number>("get_local_weekday");

        // 本行目的：返回值有效时更新起始星期并提前结束。
        if (Number.isFinite(weekday)) {
            weekdayStartIndex.value = weekday;
            return;
        }
    } catch (_) {}

    // 本行目的：后端失败或返回无效值时回退到浏览器本地星期。
    weekdayStartIndex.value = new Date().getDay();
};

/** 函数：watch 回调 | 输入：weekdayOrderSeed 变化 | 输出：无（触发同步） | 可能失败：syncWeekdayStart 内部已兜底 */
// 本行目的：监听父级种子变化，重新同步星期起点。
watch(
    // 本行目的：声明被监听的源值。
    () => props.weekdayOrderSeed,
    // 本行目的：种子变化时触发同步。
    () => {
        void syncWeekdayStart();
    },
    // 本行目的：组件初始化时立即执行一次。
    { immediate: true }
);

// 变量：orderedGroups | 含义：按起始星期重排后的分组列表 | 类型：ComputedRef<Array<{key:number|null;items:TrackedItem[]}>> | 作用域：WatchingPage 组件内
/** 函数：orderedGroups | 输入：无（读取 props.watchingByWeekday 与 weekdayStartIndex） | 输出：按显示顺序且过滤空组的分组数组 | 可能失败：无（纯派生） */
// 本行目的：根据起始星期对分组做稳定重排。
const orderedGroups = computed(() => {
    // 变量：map | 含义：分组键到分组对象的映射表 | 类型：Map<number | null, { key: number | null; items: TrackedItem[] }> | 作用域：orderedGroups 内部
    // 本行目的：创建分组查找映射，便于按顺序取值。
    const map = new Map<number | null, { key: number | null; items: TrackedItem[] }>();

    // 本行目的：把原始分组数组写入映射表。
    props.watchingByWeekday.forEach((group) => {
        map.set(group.key, group);
    });

    // 变量：weekdayOrder | 含义：从起始星期开始的7天顺序数组 | 类型：number[] | 作用域：orderedGroups 内部
    // 本行目的：生成循环星期顺序。
    const weekdayOrder = Array.from({ length: 7 }, (_, i) => (weekdayStartIndex.value + i) % 7);
    // 变量：order | 含义：最终顺序（星期 + unknown/null） | 类型：Array<number | null> | 作用域：orderedGroups 内部
    // 本行目的：将 unknown 分组放在最后。
    const order: Array<number | null> = [...weekdayOrder, null];

    // 本行目的：按顺序取分组并过滤空分组。
    return order
        // 本行目的：把顺序键映射为具体分组对象。
        .map((key) => map.get(key))
        // 本行目的：保留存在且条目数大于0的分组。
        .filter((group): group is { key: number | null; items: TrackedItem[] } => !!group && group.items.length > 0);
});

// 变量：hasGroups | 含义：当前是否存在可展示分组 | 类型：ComputedRef<boolean> | 作用域：WatchingPage 组件内
/** 函数：hasGroups | 输入：无（读取 orderedGroups） | 输出：是否有分组可展示 | 可能失败：无（纯派生） */
// 本行目的：用于模板分支判断空状态或时间线。
const hasGroups = computed(() => orderedGroups.value.length > 0);

// 变量：weekdayLabel | 含义：把星期键转换为本地化文案 | 类型：(key: number | null) => string | 作用域：WatchingPage 组件内
/** 函数：weekdayLabel | 输入：星期键（0-6/null） | 输出：对应本地化星期文本 | 可能失败：未知键时回退 unknown */
// 本行目的：统一生成时间线左侧的星期标签。
const weekdayLabel = (key: number | null) => {
    // 本行目的：null 组显示“未知星期”。
    if (key === null) return t("tracking.weekday.unknown", props.locale);

    // 变量：map | 含义：星期索引到 i18n key 的映射 | 类型：Record<number, string> | 作用域：weekdayLabel 内部
    // 本行目的：定义固定星期文案映射。
    const map: Record<number, string> = {
        0: "tracking.weekday.sun",
        1: "tracking.weekday.mon",
        2: "tracking.weekday.tue",
        3: "tracking.weekday.wed",
        4: "tracking.weekday.thu",
        5: "tracking.weekday.fri",
        6: "tracking.weekday.sat",
    };

    // 本行目的：返回对应星期文案，异常键回退 unknown。
    return t(map[key] ?? "tracking.weekday.unknown", props.locale);
};

// 变量：currentStatusKey | 含义：从条目字段推导当前状态键 | 类型：(item: TrackedItem) => StatusKey | 作用域：WatchingPage 组件内
/** 函数：currentStatusKey | 输入：追踪条目 | 输出：watched/watching/backlog/null | 可能失败：无 */
// 本行目的：统一状态推导逻辑，便于后续扩展与调试。
const currentStatusKey = (item: TrackedItem): StatusKey => {
    // 本行目的：优先判断 watched。
    if (item.watched) return "watched";
    // 本行目的：其次判断 watching。
    if (item.watching) return "watching";
    // 本行目的：再次判断 backlog。
    if (item.backlog) return "backlog";
    // 本行目的：都未命中时返回 null。
    return null;
};

// 变量：normalizeRating | 含义：把评分转换为可计算数字或 null | 类型：(value?: number | null) => number | null | 作用域：WatchingPage 组件内
/** 函数：normalizeRating | 输入：可选评分值 | 输出：有效数字或 null | 可能失败：非法数字字符串时回退 null */
// 本行目的：规范化评分输入，避免 NaN 参与后续判断。
const normalizeRating = (value?: number | null) => {
    // 本行目的：空值直接返回 null。
    if (value == null) return null;
    // 本行目的：尝试转换成数字。
    const num = Number(value);
    // 本行目的：有限数字返回本身，否则回退 null。
    return Number.isFinite(num) ? num : null;
};

// 变量：ratingTone | 含义：根据评分返回语义等级 | 类型：(value?: number | null) => "rating-neutral" | "rating-high" | "rating-medium" | "rating-low" | 作用域：WatchingPage 组件内
/** 函数：ratingTone | 输入：可选评分值 | 输出：评分语义等级 | 可能失败：无（normalizeRating 已兜底） */
// 本行目的：把评分映射为高/中/低/中性等级。
const ratingTone = (value?: number | null) => {
    // 变量：normalized | 含义：规范化后的评分数字 | 类型：number | null | 作用域：ratingTone 内部
    // 本行目的：先进行评分规范化。
    const normalized = normalizeRating(value);

    // 本行目的：无评分时返回中性等级。
    if (normalized == null) return "rating-neutral";
    // 本行目的：8 分及以上返回高等级。
    if (normalized >= 8) return "rating-high";
    // 本行目的：6 分及以上返回中等级。
    if (normalized >= 6) return "rating-medium";

    // 本行目的：其余返回低等级。
    return "rating-low";
};

// 变量：starColor | 含义：根据评分语义返回星级颜色 | 类型：(value?: number | null) => string | 作用域：WatchingPage 组件内
/** 函数：starColor | 输入：可选评分值 | 输出：CSS 颜色值 | 可能失败：无 */
// 本行目的：把评分等级转换为具体颜色用于模板样式。
const starColor = (value?: number | null) => {
    // 变量：tone | 含义：评分语义等级 | 类型：string | 作用域：starColor 内部
    // 本行目的：先计算评分等级。
    const tone = ratingTone(value);

    // 本行目的：高分返回绿色。
    if (tone === "rating-high") return "#00b42a";
    // 本行目的：中分返回黄色。
    if (tone === "rating-medium") return "#f7ba1e";
    // 本行目的：低分返回红色。
    if (tone === "rating-low") return "#f53f3f";

    // 本行目的：无评分返回中性色。
    return "var(--color-text-3)";
};

// 变量：latestEpisode | 含义：计算当前条目最新高亮集数 | 类型：(item: TrackedItem) => number | 作用域：WatchingPage 组件内
/** 函数：latestEpisode | 输入：追踪条目 | 输出：最新集数（无效时为0） | 可能失败：无 */
// 本行目的：计算并限制最新集数不超过总集数。
const latestEpisode = (item: TrackedItem) => {
    // 变量：aired | 含义：已放送集数（缺省为0） | 类型：number | 作用域：latestEpisode 内部
    // 本行目的：读取已放送集数字段。
    const aired = item.airedCount ?? 0;
    // 变量：total | 含义：总集数（缺省为0） | 类型：number | 作用域：latestEpisode 内部
    // 本行目的：读取总集数字段。
    const total = item.totalCount ?? 0;

    // 本行目的：任一为0时不高亮具体集数。
    if (!aired || !total) return 0;

    // 本行目的：返回已放送与总集数中的较小值。
    return Math.min(aired, total);
};

/** 函数：onMounted 回调 | 输入：无 | 输出：无（触发刷新） | 可能失败：内部方法自行处理异常 */
// 本行目的：组件挂载后主动刷新星期顺序与在看详情。
onMounted(() => {
    // 本行目的：触发父层刷新星期顺序。
    props.refreshWeekdayOrder();
    // 本行目的：触发父层刷新详情并请求持久化。
    void props.refreshWatchingDetails({ persist: true });
});

// 变量：handleOpenLink | 含义：统一处理条目外链打开 | 类型：(item: TrackedItem) => void | 作用域：WatchingPage 组件内
/** 函数：handleOpenLink | 输入：追踪条目 | 输出：无（调用 openExternalLink） | 可能失败：无 */
// 本行目的：构建兜底 URL 并调用外链打开回调。
const handleOpenLink = (item: TrackedItem) => {
    // 变量：url | 含义：条目详情链接（缺失时回退 bgm 地址） | 类型：string | 作用域：handleOpenLink 内部
    // 本行目的：计算最终打开链接。
    const url = item.url || `https://bgm.tv/subject/${item.id}`;
    // 本行目的：调用父层外链打开方法。
    props.openExternalLink(url);
};
</script>

<template>
    <!-- 本行目的：定义“在看”页面主容器。 -->
    <main class="tracking-watching">
        <!-- 本行目的：无分组数据时显示空状态。 -->
        <div v-if="!hasGroups" class="tracking-watching__empty">
            <!-- 本行目的：使用空状态组件展示提示信息。 -->
            <a-empty>
                <!-- 本行目的：自定义空状态描述区域。 -->
                <template #description>
                    <!-- 本行目的：空状态文案容器。 -->
                    <div class="tracking-watching__empty-text">
                        <!-- 本行目的：显示空状态标题。 -->
                        <div class="tracking-watching__empty-title">
                            {{ t("tracking.watching.empty.title", locale) }}
                        </div>
                        <!-- 本行目的：显示空状态副标题。 -->
                        <div class="tracking-watching__empty-subtitle">
                            {{ t("tracking.watching.empty.subtitle", locale) }}
                        </div>
                    </div>
                </template>
            </a-empty>
        </div>
        <!-- 本行目的：有分组数据时显示时间线。 -->
        <div v-else class="tracking-watching__timeline">
            <!-- 本行目的：遍历重排后的分组并渲染每个星期段落。 -->
            <section v-for="group in orderedGroups" :key="String(group.key)" class="tracking-watching__group">
                <!-- 本行目的：时间线左侧标记区域（圆点+星期名）。 -->
                <div class="tracking-watching__marker">
                    <!-- 本行目的：时间线节点圆点。 -->
                    <span class="tracking-watching__dot" />
                    <!-- 本行目的：显示分组星期文本并附加动态样式类。 -->
                    <span class="tracking-watching__weekday" :class="`tracking-watching__weekday--${group.key ?? 'unknown'}`">
                        {{ weekdayLabel(group.key) }}
                    </span>
                </div>
                <!-- 本行目的：时间线右侧内容区域。 -->
                <div class="tracking-watching__content">
                    <!-- 本行目的：使用网格布局展示当前分组条目。 -->
                    <div class="watchlist-grid">
                        <!-- 本行目的：遍历分组条目渲染卡片。 -->
                        <div v-for="item in group.items" :key="item.id" class="watchlist-item">
                            <!-- 本行目的：封面图片区。 -->
                            <div class="watchlist-thumb">
                                <!-- 本行目的：渲染封面图片并设置替代文本。 -->
                                <img :src="item.image" :alt="item.name" />
                            </div>
                            <!-- 本行目的：条目主体信息区。 -->
                            <div class="watchlist-body">
                                <!-- 本行目的：标题链接，点击时走统一外链处理。 -->
                                <a
                                    class="watchlist-title"
                                    :href="item.url || `https://bgm.tv/subject/${item.id}`"
                                    target="_blank"
                                    rel="noreferrer"
                                    @click.prevent="handleOpenLink(item)"
                                >
                                    <!-- 本行目的：优先显示中文名，缺失时显示原名。 -->
                                    {{ item.nameCn || item.name }}
                                </a>
                                <!-- 本行目的：显示日期与评分元信息。 -->
                                <div class="watchlist-meta">
                                    <!-- 本行目的：展示本地化放送日期。 -->
                                    <span class="meta-row"> {{ t("tracking.watching.airDate", locale) }}：{{ formatAirDate(item.date) }} </span>
                                    <!-- 本行目的：展示评分文本与星级符号。 -->
                                    <span class="meta-row">
                                        {{ t("tracking.watching.rating", locale) }}：{{ formatRating(item.rating) }}
                                        <!-- 本行目的：根据评分动态设置星级颜色。 -->
                                        <span class="watchlist-stars" :style="{ color: starColor(item.rating) }">
                                            <!-- 本行目的：输出星级符号字符串。 -->
                                            {{ formatStars(item.rating) }}
                                        </span>
                                    </span>
                                </div>
                                <!-- 本行目的：有总集数时显示集数进度条区域。 -->
                                <div v-if="item.totalCount" class="episode-strip">
                                    <!-- 本行目的：集数条头部（标题+统计摘要）。 -->
                                    <div class="episode-strip-header">
                                        <!-- 本行目的：显示集数列表标签文案。 -->
                                        <span class="episode-strip-label">
                                            {{ t("tracking.watching.episode.list", locale) }}
                                        </span>
                                        <!-- 本行目的：显示总集数与已放送汇总信息。 -->
                                        <span class="episode-strip-summary">
                                            {{ t("tracking.watching.episode.totalPrefix", locale) }}
                                            {{ item.totalCount }}
                                            {{ t("tracking.watching.episode.unit", locale) }}
                                            {{ t("tracking.watching.episode.separator", locale) }}
                                            {{ t("tracking.watching.episode.airedPrefix", locale) }}
                                            {{ item.airedCount ?? 0 }}
                                            {{ t("tracking.watching.episode.unit", locale) }}
                                        </span>
                                    </div>
                                    <!-- 本行目的：集数胶囊网格，逐集展示放送状态。 -->
                                    <div class="episode-strip-grid">
                                        <!-- 本行目的：按总集数循环渲染每一集胶囊。 -->
                                        <span
                                            v-for="n in item.totalCount"
                                            :key="n"
                                            class="episode-pill"
                                            :class="{
                                                active: (item.airedCount ?? 0) >= n,
                                                latest: latestEpisode(item) === n,
                                            }"
                                        >
                                            <!-- 本行目的：集数统一补零显示为两位。 -->
                                            {{ String(n).padStart(2, "0") }}
                                        </span>
                                    </div>
                                </div>
                                <!-- 本行目的：状态切换按钮区域。 -->
                                <div class="watchlist-actions">
                                    <!-- 本行目的：当前页面为在看页，Watching 按钮高亮并禁用。 -->
                                    <a-button size="mini" type="primary" disabled>
                                        {{ t("nav.watching", locale) }}
                                    </a-button>
                                    <!-- 本行目的：点击切换条目到 backlog 状态。 -->
                                    <a-button size="mini" @click="setExclusiveStatus(item, 'backlog')">
                                        {{ t("tracking.action.switch.backlog", locale) }}
                                    </a-button>
                                    <!-- 本行目的：点击切换条目到 watched 状态。 -->
                                    <a-button size="mini" @click="setExclusiveStatus(item, 'watched')">
                                        {{ t("tracking.action.switch.watched", locale) }}
                                    </a-button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </main>
</template>
