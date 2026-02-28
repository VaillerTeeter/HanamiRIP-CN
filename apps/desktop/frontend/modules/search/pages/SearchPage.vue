<script setup lang="ts">
/** 文件：SearchPage.vue | 用途：渲染搜索页的关键词构建、结果展示与下载入口界面 | 关键对象：props, hasInlineResults, trackedOptionsView, AliasModal */
// 本行目的：引入 computed，用于从现有状态派生模板展示数据。
import { computed } from "vue";
// 本行目的：引入国际化函数和语言键类型，用于多语言文案渲染。
import { t, type LocaleKey } from "../../../shared/i18n/messages";
// 本行目的：引入别名选择弹窗组件，用于确认追踪条目的别名关键词。
import AliasModal from "../components/AliasModal.vue";
// 本行目的：引入搜索组合式返回类型，约束父层传入的 search 对象结构。
import type { UseSearchPageReturn } from "../composables/useSearchPage";
// 本行目的：引入搜索结果和逻辑运算符类型，确保事件参数类型安全。
import type { SearchResult, LogicOp } from "../types/search";

// 变量：props | 含义：父组件传入的页面上下文和回调能力集合 | 类型：由 defineProps 推断的对象类型 | 作用域：SearchPage 组件内
/** 函数：defineProps | 输入：locale、search、外链打开回调、下载回调 | 输出：只读 props 对象 | 可能失败：无（编译期类型约束） */
// 本行目的：声明组件入参，集中接收语言、搜索状态与对外操作方法。
const props = defineProps<{
    // 本行目的：声明当前界面使用的语言键。
    locale: LocaleKey;
    // 本行目的：声明搜索页面状态与行为集合。
    search: UseSearchPageReturn;
    // 本行目的：声明打开外部链接的统一回调。
    openExternalLink: (url?: string | null) => void | Promise<void>;
    // 本行目的：声明把搜索结果提交到下载模块的统一回调。
    handleDownloadClick: (item: SearchResult, kind: "magnet" | "torrent", link?: string) => void | Promise<void>;
}>();

// 变量：updateAliasModalVisible | 含义：同步别名弹窗显示状态到搜索状态对象 | 类型：(value: boolean) => void | 作用域：SearchPage 组件内
/** 函数：updateAliasModalVisible | 输入：弹窗显示布尔值 | 输出：无（写入响应式状态） | 可能失败：无 */
// 本行目的：封装子组件 show 双向绑定的更新逻辑。
const updateAliasModalVisible = (value: boolean) => {
    // 本行目的：把子组件更新值写回组合式状态。
    props.search.aliasModalVisible.value = value;
};

// 变量：updateAliasSelected | 含义：同步别名弹窗勾选结果到搜索状态对象 | 类型：(value: string[]) => void | 作用域：SearchPage 组件内
/** 函数：updateAliasSelected | 输入：用户勾选的别名数组 | 输出：无（写入响应式状态） | 可能失败：无 */
// 本行目的：封装子组件 aliasSelected 双向绑定的更新逻辑。
const updateAliasSelected = (value: string[]) => {
    // 本行目的：把子组件勾选结果写回组合式状态。
    props.search.aliasSelected.value = value;
};

// 变量：opLabel | 含义：把逻辑运算符值转换为可展示文案 | 类型：(op: LogicOp) => string | 作用域：SearchPage 组件内
/** 函数：opLabel | 输入：逻辑运算符 and/or/not | 输出：当前语言下的逻辑标签文案 | 可能失败：无（i18n key 固定） */
// 本行目的：统一生成标签中显示的逻辑前缀文本。
const opLabel = (op: LogicOp) => {
    // 本行目的：处理 and 逻辑标签。
    if (op === "and") return t("search.logic.and", props.locale);
    // 本行目的：处理 or 逻辑标签。
    if (op === "or") return t("search.logic.or", props.locale);
    // 本行目的：其余情况返回 not 逻辑标签。
    return t("search.logic.not", props.locale);
};

// 变量：hasInlineResults | 含义：控制是否显示内联结果区域 | 类型：ComputedRef<boolean> | 作用域：SearchPage 组件内
/** 函数：hasInlineResults | 输入：无（读取 search 多个状态） | 输出：是否展示结果区域的布尔值 | 可能失败：无（纯计算） */
// 本行目的：根据加载、错误、结果和 HTML 任一命中来决定显示结果区。
const hasInlineResults = computed(() => {
    // 变量：search | 含义：props.search 的本地引用，减少重复访问链 | 类型：UseSearchPageReturn | 作用域：computed 回调内
    // 本行目的：缓存搜索状态对象，提升表达式可读性。
    const search = props.search;
    // 本行目的：任一状态存在时返回 true，触发结果区域渲染。
    return search.searchLoading.value || search.searchError.value || search.searchResults.value.length > 0 || Boolean(search.searchHtml.value);
});

// 变量：trackedOptionsView | 含义：供下拉框渲染的本地化分组选项 | 类型：ComputedRef<Array<{label: string; options: {label: string; value: number}[]}>> | 作用域：SearchPage 组件内
/** 函数：trackedOptionsView | 输入：无（读取 trackedOptions 与 locale） | 输出：过滤空组后的本地化选项列表 | 可能失败：无（纯计算） */
// 本行目的：把组合式中的 labelKey 转成当前语言文案，并过滤空分组。
const trackedOptionsView = computed(() =>
    // 本行目的：读取追踪分组选项并逐组转换显示结构。
    props.search.trackedOptions.value
        // 本行目的：把每组的 labelKey 翻译为当前语言文本。
        .map((group) => ({
            // 本行目的：生成当前分组显示标题。
            label: t(group.labelKey, props.locale),
            // 本行目的：保留原有选项数据。
            options: group.options,
        }))
        // 本行目的：过滤掉没有选项的空分组，避免渲染无意义标题。
        .filter((group) => group.options.length)
);
</script>

<template>
    <!-- 本行目的：定义搜索页面主体容器，承接页面布局样式。 -->
    <div class="app-body search-view">
        <!-- 本行目的：使用卡片承载搜索配置与结果内容。 -->
        <a-card class="search-panel" size="small">
            <!-- 本行目的：自定义卡片标题区域。 -->
            <template #title>
                <!-- 本行目的：标题栏内展示主标题和辅助提示。 -->
                <div class="search-card-header">
                    <!-- 本行目的：显示搜索页主标题。 -->
                    <span>{{ t("search.title", props.locale) }}</span>
                    <!-- 本行目的：显示标题旁的使用提示文案。 -->
                    <span class="search-hint header-hint">
                        <!-- 本行目的：渲染多语言提示文本。 -->
                        {{ t("search.hint", props.locale) }}
                    </span>
                </div>
            </template>
            <!-- 本行目的：包裹全部搜索配置区与结果区。 -->
            <div class="search-controls">
                <!-- 本行目的：逻辑运算符选择行。 -->
                <div class="search-row">
                    <!-- 本行目的：显示逻辑选择标签文本。 -->
                    <span class="search-label">{{ t("search.logic.label", props.locale) }}</span>
                    <!-- 本行目的：容纳 and/or/not 切换按钮组。 -->
                    <div class="search-button-group logic-group">
                        <!-- 本行目的：遍历逻辑选项并渲染按钮（键/尺寸/激活样式/点击切换逻辑）。 -->
                        <a-button
                            v-for="logic in props.search.logicOptions"
                            :key="logic.value"
                            size="mini"
                            :type="props.search.activeLogic.value === logic.value ? 'primary' : 'outline'"
                            @click="props.search.activeLogic.value = logic.value"
                        >
                            <!-- 本行目的：显示逻辑选项的本地化文案。 -->
                            {{ t(logic.labelKey, props.locale) }}
                        </a-button>
                    </div>
                </div>
                <!-- 本行目的：预置关键词快捷添加行。 -->
                <div class="search-row">
                    <!-- 本行目的：显示预置关键词标签文本。 -->
                    <span class="search-label">{{ t("search.preset.label", props.locale) }}</span>
                    <!-- 本行目的：容纳预置关键词按钮组。 -->
                    <div class="search-button-group">
                        <!-- 本行目的：遍历预置词并渲染快速添加按钮（键/尺寸/点击添加）。 -->
                        <a-button
                            v-for="phrase in props.search.presetPhrases"
                            :key="phrase"
                            size="mini"
                            @click="props.search.handleAddPreset(phrase)"
                        >
                            <!-- 本行目的：展示预置词文本。 -->
                            {{ phrase }}
                        </a-button>
                    </div>
                </div>
                <!-- 本行目的：追踪条目选择 + 自定义关键词输入行。 -->
                <div class="search-row compact">
                    <!-- 本行目的：显示追踪条目选择标签。 -->
                    <span class="search-label">{{ t("search.anime.label", props.locale) }}</span>
                    <!-- 本行目的：渲染追踪条目分组下拉框。 -->
                    <!-- 本行目的：下拉框属性包含当前值、占位、允许清空与更新回调。 -->
                    <a-select
                        :model-value="props.search.trackedSelection.value"
                        :placeholder="t('search.anime.placeholder', props.locale)"
                        allow-clear
                        @update:modelValue="
                            (value: number | null) => {
                                // 本行目的：立即同步下拉选择到搜索状态。
                                props.search.trackedSelection.value = value as number | null;
                                // 本行目的：触发追踪选择处理（可能打开别名弹窗）。
                                props.search.handleSelectTracked(value as number | null);
                            }
                        "
                    >
                        <!-- 本行目的：按分组渲染追踪下拉选项组。 -->
                        <a-optgroup v-for="group in trackedOptionsView" :key="group.label" :label="group.label">
                            <!-- 本行目的：渲染分组内每个具体选项。 -->
                            <a-option v-for="option in group.options" :key="option.value" :value="option.value">
                                <!-- 本行目的：显示条目名称文本。 -->
                                {{ option.label }}
                            </a-option>
                        </a-optgroup>
                    </a-select>
                    <!-- 本行目的：渲染自定义关键词输入框。 -->
                    <!-- 本行目的：输入框属性包含样式类、双向值、占位、清空和回车提交。 -->
                    <a-input
                        class="search-input-flex"
                        :model-value="props.search.customSearchInput.value"
                        :placeholder="t('search.custom.placeholder', props.locale)"
                        allow-clear
                        @update:modelValue="(value: string) => (props.search.customSearchInput.value = value)"
                        @press-enter="props.search.handleAddCustom"
                    />
                    <!-- 本行目的：点击提交当前输入关键词。 -->
                    <a-button type="primary" @click="props.search.handleAddCustom">
                        <!-- 本行目的：显示添加按钮文案。 -->
                        {{ t("search.action.add", props.locale) }}
                    </a-button>
                </div>
                <!-- 本行目的：仅在有关键词时显示已选关键词标签列表。 -->
                <div v-if="props.search.searchTerms.value.length" class="search-row search-tags">
                    <!-- 本行目的：显示已选关键词区域标签。 -->
                    <span class="search-label">{{ t("search.selected.label", props.locale) }}</span>
                    <!-- 本行目的：包裹关键词标签集合。 -->
                    <div class="search-tag-list">
                        <!-- 本行目的：遍历关键词并渲染可关闭标签（键/尺寸/关闭回调）。 -->
                        <a-tag
                            v-for="(term, idx) in props.search.searchTerms.value"
                            :key="term.id ?? `term-${idx}`"
                            size="small"
                            closable
                            @close="props.search.removeSearchTerm(term.id ?? idx, term.id == null)"
                        >
                            <!-- 本行目的：显示关键词逻辑前缀。 -->
                            <span class="term-op">{{ opLabel(term.op) }}</span>
                            <!-- 本行目的：显示关键词文本内容。 -->
                            <span class="term-value">{{ term.value }}</span>
                        </a-tag>
                    </div>
                </div>
                <!-- 本行目的：查询预览与执行搜索按钮行。 -->
                <div class="search-row">
                    <!-- 本行目的：显示查询预览标签。 -->
                    <span class="search-label">{{ t("search.query.label", props.locale) }}</span>
                    <!-- 本行目的：包裹查询预览文本容器。 -->
                    <div class="search-preview">
                        <!-- 本行目的：显示当前拼接后的查询字符串。 -->
                        <div class="search-query">
                            <!-- 本行目的：无查询时显示空提示文案。 -->
                            {{ props.search.searchQuery.value || t("search.query.empty", props.locale) }}
                        </div>
                    </div>
                    <!-- 本行目的：查询非空时允许触发搜索请求。 -->
                    <a-button type="primary" :disabled="!props.search.searchQuery.value" @click="props.search.openSearch">
                        <!-- 本行目的：显示执行搜索按钮文案。 -->
                        {{ t("search.action.open", props.locale) }}
                    </a-button>
                </div>
                <!-- 本行目的：仅在存在加载/错误/结果/HTML时显示结果区域。 -->
                <div v-if="hasInlineResults" class="search-inline-results">
                    <!-- 本行目的：结果区头部，展示标题与折叠按钮。 -->
                    <div class="search-result-header">
                        <!-- 本行目的：显示结果区标题文案。 -->
                        <span>{{ t("search.results.title", props.locale) }}</span>
                        <!-- 本行目的：点击后清空结果并折叠结果区。 -->
                        <a-button size="mini" @click="props.search.clearSearchResults">
                            <!-- 本行目的：显示折叠按钮文案。 -->
                            {{ t("search.results.collapse", props.locale) }}
                        </a-button>
                    </div>
                    <!-- 本行目的：结果内容主体容器。 -->
                    <div class="search-open-modal">
                        <!-- 本行目的：展示当前搜索 URL，便于跳转外部查看。 -->
                        <p class="search-modal-row">
                            <!-- 本行目的：显示 URL 标签名称。 -->
                            <span class="search-modal-label">{{ t("search.url.label", props.locale) }}：</span>
                            <!-- 本行目的：渲染可点击的搜索 URL 链接。 -->
                            <!-- 本行目的：链接属性包含 href、新窗口打开、安全 rel 与统一外链回调。 -->
                            <a
                                :href="props.search.searchUrl.value"
                                target="_blank"
                                rel="noreferrer"
                                @click.prevent="props.openExternalLink(props.search.searchUrl.value)"
                            >
                                <!-- 本行目的：显示当前搜索 URL 文本。 -->
                                {{ props.search.searchUrl.value }}
                            </a>
                        </p>
                        <!-- 本行目的：有解析结果时优先显示结构化结果列表。 -->
                        <div v-if="props.search.searchResults.value.length" class="search-result-list">
                            <!-- 本行目的：遍历每条搜索结果并渲染行内容。 -->
                            <div v-for="item in props.search.searchResults.value" :key="item.detailUrl || item.title" class="search-result-row">
                                <!-- 本行目的：展示条目名称与元信息区域。 -->
                                <div class="sr-name">
                                    <!-- 本行目的：条目标题点击后打开详情或可用下载链接。 -->
                                    <!-- 本行目的：标题链接属性包含目标地址、新窗口、安全 rel 与统一外链回调。 -->
                                    <a
                                        :href="item.detailUrl || item.magnet || item.download"
                                        target="_blank"
                                        rel="noreferrer"
                                        @click.prevent="props.openExternalLink(item.detailUrl || item.magnet || item.download)"
                                    >
                                        <!-- 本行目的：显示结果标题。 -->
                                        {{ item.title }}
                                    </a>
                                    <!-- 本行目的：存在大小或日期时显示元信息。 -->
                                    <div v-if="item.size || item.date" class="sr-meta">
                                        <!-- 本行目的：有大小时显示文件大小。 -->
                                        <span v-if="item.size">{{ item.size }}</span>
                                        <!-- 本行目的：有日期时显示发布时间。 -->
                                        <span v-if="item.date">{{ item.date }}</span>
                                    </div>
                                </div>
                                <!-- 本行目的：展示可执行下载动作按钮区域。 -->
                                <div class="sr-links">
                                    <!-- 本行目的：有磁力链接时显示磁力下载按钮。 -->
                                    <!-- 本行目的：按钮属性包含显示条件、文本样式、尺寸与点击下载回调。 -->
                                    <a-button
                                        v-if="item.magnet"
                                        type="text"
                                        size="mini"
                                        @click="props.handleDownloadClick(item, 'magnet', item.magnet)"
                                    >
                                        <!-- 本行目的：显示磁力按钮文案。 -->
                                        {{ t("search.result.magnet", props.locale) }}
                                    </a-button>
                                    <!-- 本行目的：有 torrent 链接时显示 torrent 下载按钮。 -->
                                    <!-- 本行目的：按钮属性包含显示条件、文本样式、尺寸与点击下载回调。 -->
                                    <a-button
                                        v-if="item.download"
                                        type="text"
                                        size="mini"
                                        @click="props.handleDownloadClick(item, 'torrent', item.download)"
                                    >
                                        <!-- 本行目的：显示 torrent 按钮文案。 -->
                                        {{ t("search.result.torrent", props.locale) }}
                                    </a-button>
                                </div>
                            </div>
                        </div>
                        <!-- 本行目的：无结构化结果且无加载/错误时，回退展示原始 HTML 预览。 -->
                        <!-- 本行目的：iframe 属性包含显示条件、样式、srcdoc、sandbox 与可访问标题。 -->
                        <iframe
                            v-else-if="!props.search.searchLoading.value && !props.search.searchError.value"
                            class="search-preview-frame"
                            :srcdoc="props.search.searchHtml.value"
                            sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
                            :title="t('search.title', props.locale)"
                        />
                        <!-- 本行目的：加载中时展示加载提示。 -->
                        <div v-if="props.search.searchLoading.value" class="search-loading">
                            <!-- 本行目的：显示加载状态文案。 -->
                            {{ t("search.loading", props.locale) }}
                        </div>
                        <!-- 本行目的：加载失败时展示错误信息。 -->
                        <div v-else-if="props.search.searchError.value" class="search-error">
                            <!-- 本行目的：显示当前错误文本。 -->
                            {{ props.search.searchError.value }}
                        </div>
                    </div>
                </div>
            </div>
        </a-card>
        <!-- 本行目的：渲染别名选择弹窗，并与搜索状态双向联动。 -->
        <!-- 本行目的：传入弹窗所需状态、确认/取消回调以及两个更新事件。 -->
        <AliasModal
            :locale="props.locale"
            :show="props.search.aliasModalVisible.value"
            :pending-tracked-name="props.search.pendingTrackedName.value"
            :alias-loading="props.search.aliasLoading.value"
            :alias-options="props.search.aliasOptions.value"
            :alias-selected="props.search.aliasSelected.value"
            :on-cancel="props.search.cancelAliasSelection"
            :on-confirm="props.search.confirmAliasSelection"
            @update:show="updateAliasModalVisible"
            @update:aliasSelected="updateAliasSelected"
        />
    </div>
</template>
