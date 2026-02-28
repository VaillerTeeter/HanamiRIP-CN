<script setup lang="ts">
/** 文件：TrackingVariantsPage.vue | 用途：展示追踪页多种布局方案的预览页面 | 关键对象：props, tabItems, mockItems */
// 本行目的：引入 computed，用于声明依赖语言的派生展示数据。
import { computed } from "vue";
// 本行目的：引入国际化函数与语言键类型。
import { t, type LocaleKey } from "../../../shared/i18n/messages";

// 变量：props | 含义：父级传入的页面语言上下文 | 类型：由 defineProps 推断的对象 | 作用域：TrackingVariantsPage 组件内
/** 函数：defineProps | 输入：locale | 输出：组件 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：声明当前组件仅依赖 locale 进行文案渲染。
const props = defineProps<{
    // 本行目的：声明当前页面使用的语言键。
    locale: LocaleKey;
}>();

// 变量：tabItems | 含义：预览页顶部标签页配置 | 类型：ComputedRef<Array<{key:string;label:string}>> | 作用域：TrackingVariantsPage 组件内
/** 函数：tabItems | 输入：无（读取 locale） | 输出：四个方案标签项数组 | 可能失败：无（纯派生） */
// 本行目的：根据当前语言生成 A/B/C/D 四个方案标签。
const tabItems = computed(() => [
    // 本行目的：定义方案 A 标签项。
    { key: "planA", label: t("tracking.preview.planA", props.locale) },
    // 本行目的：定义方案 B 标签项。
    { key: "planB", label: t("tracking.preview.planB", props.locale) },
    // 本行目的：定义方案 C 标签项。
    { key: "planC", label: t("tracking.preview.planC", props.locale) },
    // 本行目的：定义方案 D 标签项。
    { key: "planD", label: t("tracking.preview.planD", props.locale) },
]);

// 变量：mockItems | 含义：预览用示例条目数据 | 类型：ComputedRef<Array<{id:number;title:string;subtitle:string;rating:string;weekday:string}>> | 作用域：TrackingVariantsPage 组件内
/** 函数：mockItems | 输入：无（读取 locale） | 输出：本地化后的示例条目数组 | 可能失败：无（纯派生） */
// 本行目的：生成多方案共用的示例数据，避免依赖真实后端数据。
const mockItems = computed(() => [
    {
        // 本行目的：定义示例条目唯一ID。
        id: 1,
        // 本行目的：定义示例条目主标题。
        title: t("tracking.preview.item1", props.locale),
        // 本行目的：定义示例条目副标题。
        subtitle: t("tracking.preview.item1.subtitle", props.locale),
        // 本行目的：定义示例评分文本。
        rating: "8.6",
        // 本行目的：定义示例放送星期文案。
        weekday: t("tracking.preview.weekday.mon", props.locale),
    },

    {
        // 本行目的：定义第二条示例条目唯一ID。
        id: 2,
        // 本行目的：定义第二条示例主标题。
        title: t("tracking.preview.item2", props.locale),
        // 本行目的：定义第二条示例副标题。
        subtitle: t("tracking.preview.item2.subtitle", props.locale),
        // 本行目的：定义第二条示例评分文本。
        rating: "7.2",
        // 本行目的：定义第二条示例星期文案。
        weekday: t("tracking.preview.weekday.wed", props.locale),
    },

    {
        // 本行目的：定义第三条示例条目唯一ID。
        id: 3,
        // 本行目的：定义第三条示例主标题。
        title: t("tracking.preview.item3", props.locale),
        // 本行目的：定义第三条示例副标题。
        subtitle: t("tracking.preview.item3.subtitle", props.locale),
        // 本行目的：定义第三条示例评分文本。
        rating: "9.1",
        // 本行目的：定义第三条示例星期文案。
        weekday: t("tracking.preview.weekday.sat", props.locale),
    },
]);
</script>

<template>
    <!-- 本行目的：页面主容器，承载整块预览内容。 -->
    <div class="app-body tracking-preview">
        <!-- 本行目的：使用卡片包裹追踪方案预览。 -->
        <a-card class="tracking-preview__card" :bordered="true">
            <!-- 本行目的：纵向排列标题、说明和方案内容。 -->
            <a-space direction="vertical" :size="16" fill>
                <!-- 本行目的：显示预览页主标题。 -->
                <a-typography-title :heading="4" class="tracking-preview__title">
                    {{ t("tracking.preview.title", props.locale) }}
                </a-typography-title>
                <!-- 本行目的：显示预览页副标题说明。 -->
                <a-typography-text type="secondary">
                    {{ t("tracking.preview.subtitle", props.locale) }}
                </a-typography-text>
                <!-- 本行目的：标题区与内容区之间的分隔线。 -->
                <a-divider />
                <!-- 本行目的：按 A/B/C/D 四种方案展示标签页。 -->
                <a-tabs type="line" default-active-key="planA">
                    <!-- 本行目的：遍历标签项并渲染对应的标签面板。 -->
                    <a-tab-pane v-for="tab in tabItems" :key="tab.key" :title="tab.label">
                        <!-- 本行目的：每个方案的内容面板容器。 -->
                        <div class="tracking-preview__panel">
                            <!-- 本行目的：方案 A：卡片网格 + 快捷操作按钮。 -->
                            <template v-if="tab.key === 'planA'">
                                <a-space direction="vertical" :size="16" fill>
                                    <!-- 本行目的：显示方案 A 描述文案。 -->
                                    <a-typography-text class="tracking-preview__label">
                                        {{ t("tracking.preview.planA.desc", props.locale) }}
                                    </a-typography-text>
                                    <!-- 本行目的：网格布局展示示例条目。 -->
                                    <div class="tracking-preview__grid">
                                        <!-- 本行目的：遍历 mockItems 渲染方案 A 的条目卡片。 -->
                                        <a-card v-for="item in mockItems" :key="item.id" :bordered="true" class="tracking-preview__item">
                                            <a-space direction="vertical" :size="8">
                                                <!-- 本行目的：显示条目标题。 -->
                                                <a-typography-text strong>{{ item.title }}</a-typography-text>
                                                <!-- 本行目的：显示条目副标题。 -->
                                                <a-typography-text type="secondary">{{ item.subtitle }}</a-typography-text>
                                                <!-- 本行目的：显示星期与评分标签。 -->
                                                <a-space :size="8">
                                                    <a-tag color="arcoblue">{{ item.weekday }}</a-tag>
                                                    <a-tag color="green">{{ t("tracking.preview.rating", props.locale) }} {{ item.rating }}</a-tag>
                                                </a-space>
                                                <!-- 本行目的：展示三种状态动作按钮示意。 -->
                                                <a-space :size="8">
                                                    <a-button type="primary" size="mini">{{
                                                        t("tracking.preview.action.keep", props.locale)
                                                    }}</a-button>
                                                    <a-button type="outline" size="mini">{{
                                                        t("tracking.preview.action.backlog", props.locale)
                                                    }}</a-button>
                                                    <a-button type="outline" size="mini">{{
                                                        t("tracking.preview.action.finish", props.locale)
                                                    }}</a-button>
                                                </a-space>
                                            </a-space>
                                        </a-card>
                                    </div>
                                </a-space>
                            </template>
                            <!-- 本行目的：方案 B：看板双列布局（今日/即将）。 -->
                            <template v-else-if="tab.key === 'planB'">
                                <a-space direction="vertical" :size="16" fill>
                                    <!-- 本行目的：显示方案 B 描述文案。 -->
                                    <a-typography-text class="tracking-preview__label">
                                        {{ t("tracking.preview.planB.desc", props.locale) }}
                                    </a-typography-text>
                                    <!-- 本行目的：看板容器，包含两列。 -->
                                    <div class="tracking-preview__kanban">
                                        <!-- 本行目的：左列展示“今日”分组。 -->
                                        <div class="tracking-preview__column">
                                            <div class="tracking-preview__column-title">{{ t("tracking.preview.column.today", props.locale) }}</div>
                                            <!-- 本行目的：遍历示例条目渲染今日列卡片。 -->
                                            <a-card v-for="item in mockItems" :key="item.id" class="tracking-preview__item">
                                                <a-typography-text strong>{{ item.title }}</a-typography-text>
                                                <a-typography-text type="secondary">{{ item.subtitle }}</a-typography-text>
                                            </a-card>
                                        </div>
                                        <!-- 本行目的：右列展示“即将”分组。 -->
                                        <div class="tracking-preview__column">
                                            <div class="tracking-preview__column-title">
                                                {{ t("tracking.preview.column.upcoming", props.locale) }}
                                            </div>
                                            <!-- 本行目的：遍历示例条目渲染即将列卡片。 -->
                                            <a-card v-for="item in mockItems" :key="`up-${item.id}`" class="tracking-preview__item">
                                                <a-typography-text strong>{{ item.title }}</a-typography-text>
                                                <a-typography-text type="secondary">{{ item.subtitle }}</a-typography-text>
                                            </a-card>
                                        </div>
                                    </div>
                                </a-space>
                            </template>
                            <!-- 本行目的：方案 C：按时间轴展示条目。 -->
                            <template v-else-if="tab.key === 'planC'">
                                <a-space direction="vertical" :size="16" fill>
                                    <!-- 本行目的：显示方案 C 描述文案。 -->
                                    <a-typography-text class="tracking-preview__label">
                                        {{ t("tracking.preview.planC.desc", props.locale) }}
                                    </a-typography-text>
                                    <!-- 本行目的：时间轴容器。 -->
                                    <a-timeline>
                                        <!-- 本行目的：遍历示例条目渲染时间轴节点。 -->
                                        <a-timeline-item v-for="item in mockItems" :key="`time-${item.id}`">
                                            <a-space direction="vertical" :size="8">
                                                <!-- 本行目的：显示节点对应星期。 -->
                                                <a-typography-text strong>{{ item.weekday }}</a-typography-text>
                                                <!-- 本行目的：节点内卡片展示标题和副标题。 -->
                                                <a-card class="tracking-preview__item">
                                                    <a-typography-text strong>{{ item.title }}</a-typography-text>
                                                    <a-typography-text type="secondary">{{ item.subtitle }}</a-typography-text>
                                                </a-card>
                                            </a-space>
                                        </a-timeline-item>
                                    </a-timeline>
                                </a-space>
                            </template>
                            <!-- 本行目的：方案 D：表格展示条目核心字段。 -->
                            <template v-else>
                                <a-space direction="vertical" :size="16" fill>
                                    <!-- 本行目的：显示方案 D 描述文案。 -->
                                    <a-typography-text class="tracking-preview__label">
                                        {{ t("tracking.preview.planD.desc", props.locale) }}
                                    </a-typography-text>
                                    <!-- 本行目的：渲染无分页表格，列配置与数据均来自本地派生值，表格数据源使用 mockItems。 -->
                                    <a-table
                                        :pagination="false"
                                        :columns="[
                                            // 本行目的：定义标题列。
                                            { title: t('tracking.preview.table.title', props.locale), dataIndex: 'title' },
                                            // 本行目的：定义星期列。
                                            { title: t('tracking.preview.table.weekday', props.locale), dataIndex: 'weekday' },
                                            // 本行目的：定义评分列。
                                            { title: t('tracking.preview.table.rating', props.locale), dataIndex: 'rating' },
                                        ]"
                                        :data="mockItems"
                                    />
                                </a-space>
                            </template>
                        </div>
                    </a-tab-pane>
                </a-tabs>
            </a-space>
        </a-card>
    </div>
</template>
