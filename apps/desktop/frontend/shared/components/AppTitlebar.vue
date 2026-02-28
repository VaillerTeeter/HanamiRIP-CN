<script setup lang="ts">
/** 文件：AppTitlebar.vue | 用途：提供应用顶部标题栏、页面导航与窗口控制按钮 | 关键对象：props, emit, navItems, isActive, switchPage */
// 本行目的：引入 computed，用于派生导航项数组。
import { computed } from "vue";
// 本行目的：引入窗口控制图标。
import { IconClose, IconMinus } from "@arco-design/web-vue/es/icon";
// 本行目的：引入国际化函数与语言键类型。
import { t, type LocaleKey } from "../i18n/messages";
// 本行目的：引入页面键类型，约束导航切换参数。
import type { PageKey } from "../types/page";

// 变量：props | 含义：父组件传入的激活页、语言与窗口操作回调 | 类型：由 defineProps 推断的对象 | 作用域：AppTitlebar 组件内
/** 函数：defineProps | 输入：activePage、locale、onMinimize、onClose | 输出：组件 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：声明标题栏组件所需入参。
const props = defineProps<{
    // 本行目的：声明当前激活页面键。
    activePage: PageKey;
    // 本行目的：声明当前语言键。
    locale: LocaleKey;
    // 本行目的：声明最小化窗口回调。
    onMinimize: () => void | Promise<void>;
    // 本行目的：声明关闭窗口回调。
    onClose: () => void | Promise<void>;
}>();

// 变量：emit | 含义：组件事件派发函数 | 类型：由 defineEmits 推断 | 作用域：AppTitlebar 组件内
/** 函数：defineEmits | 输入：switch 事件与目标 PageKey | 输出：emit 函数 | 可能失败：无（类型不匹配编译期报错） */
// 本行目的：声明页面切换事件。
const emit = defineEmits<{ (event: "switch", page: PageKey): void }>();

// 变量：navItems | 含义：标题栏导航按钮配置列表 | 类型：ComputedRef<Array<{key:PageKey;label:string}>> | 作用域：AppTitlebar 组件内
/** 函数：navItems | 输入：无（读取 locale） | 输出：导航项数组 | 可能失败：无（纯派生） */
// 本行目的：生成本地化导航项列表。
const navItems = computed(() => [
    // 本行目的：查询页面导航项。
    { key: "query", label: t("nav.query", props.locale) },
    // 本行目的：在看页面导航项。
    { key: "watching", label: t("nav.watching", props.locale) },
    // 本行目的：补番页面导航项。
    { key: "backlog", label: t("nav.backlog", props.locale) },
    // 本行目的：已完页面导航项。
    { key: "finished", label: t("nav.finished", props.locale) },
    // 本行目的：搜索页面导航项。
    { key: "search", label: t("nav.search", props.locale) },
    // 本行目的：下载页面导航项。
    { key: "download", label: t("nav.download", props.locale) },
    // 本行目的：轨道页面导航项。
    { key: "tracks", label: t("nav.tracks", props.locale) },
]);

// 变量：isActive | 含义：判断某页面是否为当前激活页 | 类型：(page: PageKey) => boolean | 作用域：AppTitlebar 组件内
/** 函数：isActive | 输入：页面键 | 输出：是否激活 | 可能失败：无 */
// 本行目的：提供导航按钮激活态判断。
const isActive = (page: PageKey) => props.activePage === page;

// 变量：switchPage | 含义：派发页面切换事件 | 类型：(page: PageKey) => void | 作用域：AppTitlebar 组件内
/** 函数：switchPage | 输入：目标页面键 | 输出：无（触发 switch 事件） | 可能失败：无 */
// 本行目的：封装页面切换事件派发。
const switchPage = (page: PageKey) => {
    emit("switch", page);
};
</script>

<template>
    <!-- 本行目的：标题栏根节点，启用 Tauri 拖拽区域。 -->
    <header class="app-titlebar" data-tauri-drag-region>
        <!-- 本行目的：标题栏主体拖拽区域。 -->
        <div class="titlebar-drag-region" data-tauri-drag-region>
            <!-- 本行目的：左侧应用标题与副标题区域。 -->
            <div class="titlebar-left" data-tauri-drag-region>
                <!-- 本行目的：显示应用标题。 -->
                <a-typography-text strong class="app-title">
                    {{ t("titlebar.appTitle", props.locale) }}
                </a-typography-text>
                <!-- 本行目的：显示应用副标题。 -->
                <a-typography-text type="secondary" class="app-subtitle">
                    {{ t("titlebar.subtitle", props.locale) }}
                </a-typography-text>
            </div>
            <!-- 本行目的：中部导航按钮组。 -->
            <a-space class="titlebar-nav" :size="8" data-tauri-drag-region>
                <!-- 本行目的：遍历导航项渲染页面切换按钮。 -->
                <a-button
                    v-for="item in navItems"
                    :key="item.key"
                    size="small"
                    :type="isActive(item.key as PageKey) ? 'primary' : 'outline'"
                    :data-tauri-drag-region="false"
                    @click="switchPage(item.key as PageKey)"
                >
                    <!-- 本行目的：显示导航按钮文案。 -->
                    {{ item.label }}
                </a-button>
            </a-space>
        </div>
        <!-- 本行目的：右侧窗口控制按钮区域（不参与拖拽）。 -->
        <div class="titlebar-actions" aria-label="window actions" data-tauri-drag-region="false">
            <!-- 本行目的：最小化窗口按钮。 -->
            <a-button
                class="titlebar-control"
                type="text"
                size="small"
                :aria-label="t('window.minimize', props.locale)"
                :data-tauri-drag-region="false"
                @click="props.onMinimize"
            >
                <!-- 本行目的：渲染最小化图标。 -->
                <IconMinus class="titlebar-icon" />
            </a-button>
            <!-- 本行目的：关闭窗口按钮。 -->
            <a-button
                class="titlebar-control titlebar-close"
                type="text"
                size="small"
                :aria-label="t('window.close', props.locale)"
                :data-tauri-drag-region="false"
                @click="props.onClose"
            >
                <!-- 本行目的：渲染关闭图标。 -->
                <IconClose class="titlebar-icon" />
            </a-button>
        </div>
    </header>
</template>
