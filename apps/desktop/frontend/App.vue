<script setup lang="ts">
/** 文件：App.vue | 用途：应用壳组件，负责页面路由切换、多模块组合与全局语言上下文 | 关键对象：locale、activePage、switchPage、tracking、query、search、downloads、tracks */
// 引入 Vue 的响应式基础能力，用于管理全局页面状态与派生状态。
import { computed, ref, watch } from "vue";
// 引入 Arco 中文语言包，用于组件库文案本地化。
import zhCN from "@arco-design/web-vue/es/locale/lang/zh-cn";
// 引入 Arco 英文语言包，用于切换到英文时的组件库文案。
import enUS from "@arco-design/web-vue/es/locale/lang/en-us";
// 引入顶部标题栏组件，提供导航与窗口按钮。
import AppTitlebar from "./shared/components/AppTitlebar.vue";
// 引入外链打开能力，统一由封装层处理平台差异。
import { useExternalLink } from "./shared/composables/useExternalLink";
// 引入窗口控制能力，统一处理最小化/关闭动作。
import { useWindowControls } from "./shared/composables/useWindowControls";
// 引入翻译函数与语言键类型，确保文案读取类型安全。
import { t, type LocaleKey } from "./shared/i18n/messages";
// 引入页面键类型，限定可切换页面集合。
import type { PageKey } from "./shared/types/page";
// 引入查询页组件。
import QueryPage from "./modules/query/pages/QueryPage.vue";
// 引入在看页组件。
import WatchingPage from "./modules/tracking/pages/WatchingPage.vue";
// 引入积压页组件。
import BacklogPage from "./modules/tracking/pages/BacklogPage.vue";
// 引入看完页组件。
import FinishedPage from "./modules/tracking/pages/FinishedPage.vue";
// 引入搜索页组件。
import SearchPage from "./modules/search/pages/SearchPage.vue";
// 引入下载页组件。
import DownloadPage from "./modules/download/pages/DownloadPage.vue";
// 引入轨道处理页组件。
import TracksPage from "./modules/tracks/pages/TracksPage.vue";
// 引入查询页组合式逻辑。
import { useQueryPage } from "./modules/query/composables/useQueryPage";
// 引入追番模块组合式逻辑。
import { useTracking } from "./modules/tracking/composables/useTracking";
// 引入搜索模块组合式逻辑。
import { useSearchPage } from "./modules/search/composables/useSearchPage";
// 引入下载模块组合式逻辑。
import { useDownloadPage } from "./modules/download/composables/useDownloadPage";
// 引入轨道模块组合式逻辑。
import { useTracksPage } from "./modules/tracks/composables/useTracksPage";
// 引入搜索结果类型，用于下载入口函数参数约束。
import type { SearchResult } from "./modules/search/types/search";
// 引入追番状态类型，约束状态映射与按钮文案逻辑。
import type { ItemStatus, StatusKey } from "./modules/tracking/types/tracking";

// 变量：locale | 含义：当前 UI 使用的语言代码 | 类型：Ref<LocaleKey> | 作用域：App 根组件
const locale = ref<LocaleKey>("zh-CN");
// 变量：localeObject | 含义：把当前语言键映射为 Arco 所需的 locale 对象 | 类型：ComputedRef<any> | 作用域：App 根组件
const localeObject = computed(() => (locale.value === "en-US" ? enUS : zhCN));
// 变量：activePage | 含义：当前激活的主页面标识 | 类型：Ref<PageKey> | 作用域：App 根组件
const activePage = ref<PageKey>("query");

/** 函数：switchPage | 输入：page（目标页面键） | 输出：无（更新 activePage 并按需刷新在看排序） | 可能失败：tracking.refreshWeekdayOrder 异步内部失败 */
const switchPage = (page: PageKey) => {
    // 当切换到在看页时，先刷新星期分组顺序，确保界面数据新鲜。
    if (page === "watching") {
        // 异步刷新周几分组顺序，使用 void 忽略返回 Promise 以避免未处理警告。
        void tracking.refreshWeekdayOrder();
        // 递增随机种子，触发依赖该种子的子组件重新计算展示顺序。
        watchingOrderSeed.value += 1;
    }

    // 最后统一更新当前页，驱动模板中的 v-if 分支切换。
    activePage.value = page;
};

// 变量：handleMinimize/handleClose | 含义：窗口最小化与关闭动作处理器 | 类型：函数 | 作用域：App 根组件
const { handleMinimize, handleClose } = useWindowControls();
// 变量：openExternalLink | 含义：统一打开外部链接的方法 | 类型：函数 | 作用域：App 根组件
const { openExternalLink } = useExternalLink();
// 变量：tracking | 含义：追番领域状态与操作集合 | 类型：组合式返回对象 | 作用域：App 根组件
const tracking = useTracking();

// 变量：search | 含义：搜索页业务状态与动作集合 | 类型：组合式返回对象 | 作用域：App 根组件
const search = useSearchPage({
    // 向搜索模块注入追番条目引用，用于搜索结果联动追番状态。
    trackedItems: tracking.trackedItems,
    // 注入统一错误文案回退函数，跟随当前语言变化。
    errorFallback: () => t("search.error.fallback", locale.value),
});

// 变量：downloads | 含义：下载页业务状态与动作集合 | 类型：组合式返回对象 | 作用域：App 根组件
const downloads = useDownloadPage({
    // 根据下载类型返回不同对话框标题。
    dialogTitleForKind: (kind) => (kind === "magnet" ? t("download.dialog.magnet", locale.value) : t("download.dialog.torrent", locale.value)),
    // 下载通用错误回退文案。
    errorFallback: () => t("download.error.fallback", locale.value),
    // 非法磁力链接提示文案。
    errorInvalidMagnet: () => t("download.error.invalidMagnet", locale.value),
    // 非法种子文件提示文案。
    errorInvalidTorrent: () => t("download.error.invalidTorrent", locale.value),
});

// 变量：trackLanguageOptions | 含义：轨道混流语言下拉项，随语言切换实时翻译 | 类型：ComputedRef<Array<{label:string;value:string}>> | 作用域：App 根组件
const trackLanguageOptions = computed(() => [
    // 自动识别语言。
    { label: t("tracks.lang.auto", locale.value), value: "" },
    // 日语。
    { label: t("tracks.lang.ja", locale.value), value: "ja" },
    // 英语。
    { label: t("tracks.lang.en", locale.value), value: "en" },
    // 简体中文（脚本标签）。
    { label: t("tracks.lang.zhHans", locale.value), value: "zh-Hans" },
    // 繁体中文（脚本标签）。
    { label: t("tracks.lang.zhHant", locale.value), value: "zh-Hant" },
    // 中文（通用标签）。
    { label: t("tracks.lang.zh", locale.value), value: "zh" },
    // 韩语。
    { label: t("tracks.lang.ko", locale.value), value: "ko" },
    // 法语。
    { label: t("tracks.lang.fr", locale.value), value: "fr" },
    // 德语。
    { label: t("tracks.lang.de", locale.value), value: "de" },
    // 西班牙语。
    { label: t("tracks.lang.es", locale.value), value: "es" },
    // 未定义语言代码。
    { label: t("tracks.lang.und", locale.value), value: "und" },
]);

// 变量：tracks | 含义：音视频轨道解析与混流业务对象 | 类型：组合式返回对象 | 作用域：App 根组件
const tracks = useTracksPage({
    // 轨道类型标签翻译函数。
    labelForType: (type) => t(`tracks.type.${type}`, locale.value),
    // 选择输入文件对话框标题。
    selectFileTitle: (type) => t(`tracks.dialog.${type}.title`, locale.value),
    // 选择输入文件过滤器名称。
    selectFileFilterName: (type) => t(`tracks.dialog.${type}.filter`, locale.value),
    // 缺失输入文件时的提示文案。
    missingFileError: (type) => t(`tracks.error.missing.${type}`, locale.value),
    // 轨道解析失败时的兜底文案。
    parseErrorFallback: () => t("tracks.error.parse", locale.value),
    // 输出路径选择对话框标题。
    outputDialogTitle: () => t("tracks.mix.output.title", locale.value),
    // 输出路径文件过滤器名称。
    outputDialogFilterName: () => t("tracks.mix.output.filter", locale.value),
    // 输出文件名后缀。
    outputSuffix: () => t("tracks.mix.output.suffix", locale.value),
    // 未选择视频轨道时的错误文案。
    mixVideoRequiredError: () => t("tracks.error.videoRequired", locale.value),
    // 入队成功提示文案。
    mixQueuedMessage: () => t("tracks.mix.enqueued", locale.value),
    // 队列无结果文案。
    mixQueueEmptyMessage: () => t("tracks.mix.queue.emptyResult", locale.value),
    // 混流失败文案。
    mixFailedMessage: () => t("tracks.mix.failed", locale.value),
    // 语言下拉选项在创建时注入一次。
    trackLanguageOptions: trackLanguageOptions.value,
});

// 变量：watchingOrderSeed | 含义：在看页排序刷新触发种子 | 类型：Ref<number> | 作用域：App 根组件
const watchingOrderSeed = ref(0);

/** 函数：currentStatusKey | 输入：status（三态布尔对象） | 输出：当前激活状态键或 null | 可能失败：无（纯同步映射） */
const currentStatusKey = (status: ItemStatus): StatusKey => {
    // 优先判定在看状态。
    if (status.watching) return "watching";
    // 其次判定想看状态。
    if (status.backlog) return "backlog";
    // 再判定看完状态。
    if (status.watched) return "watched";
    // 如果三者都未激活，则返回空状态。
    return null;
};

/** 函数：labelForAction | 输入：active（当前状态）、target（目标状态） | 输出：按钮文案 | 可能失败：无（纯同步翻译映射） */
const labelForAction = (active: StatusKey, target: StatusKey) => {
    // 当前状态与目标一致时，动作含义是“移除该状态”。
    if (active === target) return t("tracking.action.remove", locale.value);
    // 已有状态且目标为在看时，文案为“切换到在看”。
    if (active && target === "watching") return t("tracking.action.switch.watching", locale.value);
    // 已有状态且目标为想看时，文案为“切换到想看”。
    if (active && target === "backlog") return t("tracking.action.switch.backlog", locale.value);
    // 已有状态且目标为看完时，文案为“切换到看完”。
    if (active && target === "watched") return t("tracking.action.switch.watched", locale.value);
    // 无当前状态且目标为在看时，文案为“设为在看”。
    if (target === "watching") return t("tracking.action.set.watching", locale.value);
    // 无当前状态且目标为想看时，文案为“设为想看”。
    if (target === "backlog") return t("tracking.action.set.backlog", locale.value);
    // 无当前状态且目标为看完时，文案为“设为看完”。
    if (target === "watched") return t("tracking.action.set.watched", locale.value);
    // 兜底返回通用文案，防止异常输入导致空文本。
    return t("tracking.action.default", locale.value);
};

// 变量：query | 含义：查询页业务状态与动作集合 | 类型：组合式返回对象 | 作用域：App 根组件
const query = useQueryPage({
    // 注入状态确保函数，供查询页筛选/标记时读写状态。
    ensureStatus: tracking.ensureStatus,
    // 注入“全部”筛选项文案工厂。
    allFilterLabel: () => t("query.filter.all", locale.value),
});

// 变量：selectedStatus | 含义：当前查询选中条目的追番状态快照 | 类型：ComputedRef<ItemStatus> | 作用域：App 根组件
const selectedStatus = computed<ItemStatus>(() => {
    // 读取当前选中条目 ID。
    const id = query.selected.value?.id;
    // 如果没有选中条目，返回全 false 的默认状态对象。
    if (!id) return { watching: false, backlog: false, watched: false };
    // 有选中条目时返回状态仓库中的真实状态。
    return tracking.ensureStatus(id);
});

// 监听页面切换事件，用于处理跨页副作用（滚动恢复与在看排序刷新）。
watch(activePage, (next, prev) => {
    // 从其他页面返回查询页时，自动滚动到已选中项提升连续操作体验。
    if (next === "query" && prev && prev !== "query") {
        // 异步滚动恢复，不阻塞页面切换。
        void query.scrollToSelectedItem("auto");
    }

    // 切到在看页时刷新周几分组顺序，保证展示即时性。
    if (next === "watching") {
        // 忽略 Promise 返回值，避免未处理警告。
        void tracking.refreshWeekdayOrder();
    }
});

/** 函数：handleDownloadClick | 输入：item（搜索结果）、kind（下载类型）、link（可选链接） | 输出：无（委托下载模块处理） | 可能失败：下载模块内部校验或 I/O 失败 */
const handleDownloadClick = (item: SearchResult, kind: "magnet" | "torrent", link?: string) => {
    // 把下载动作统一委托给下载组合式，保持页面层职责简单。
    void downloads.handleDownloadClick(item, kind, link);
};
</script>

<template>
    <!-- 使用 Arco 配置提供器注入当前组件库语言对象，影响全局 Arco 子组件文案 -->
    <a-config-provider :locale="localeObject">
        <!-- 应用外壳容器：顶部标题栏 + 页面主体 -->
        <div class="app-shell">
            <!-- 顶部标题栏：接收当前页与语言，并透传窗口控制函数 -->
            <AppTitlebar :active-page="activePage" :locale="locale" :on-minimize="handleMinimize" :on-close="handleClose" @switch="switchPage" />
            <!-- 查询页：当 activePage 为 query 时展示 -->
            <QueryPage
                v-if="activePage === 'query'"
                :locale="locale"
                :query="query"
                :open-external-link="openExternalLink"
                :tracking="{
                    selectedStatus,
                    currentStatusKey,
                    labelForAction,
                    setExclusiveStatus: tracking.setExclusiveStatus,
                }"
            />
            <!-- 在看页容器：使用 tracking-body 控制滚动与拉伸 -->
            <div v-else-if="activePage === 'watching'" class="app-body tracking-body">
                <!-- 在看页：接收周几分组、刷新函数与外链能力 -->
                <WatchingPage
                    :locale="locale"
                    :watching-by-weekday="tracking.watchingByWeekday.value"
                    :set-exclusive-status="tracking.setExclusiveStatus"
                    :refresh-watching-details="tracking.refreshWatchingDetails"
                    :refresh-weekday-order="tracking.refreshWeekdayOrder"
                    :weekday-order-seed="watchingOrderSeed"
                    :open-external-link="openExternalLink"
                />
            </div>
            <!-- 积压页容器 -->
            <div v-else-if="activePage === 'backlog'" class="app-body tracking-body">
                <!-- 积压页：接收积压列表与状态切换能力 -->
                <BacklogPage
                    :locale="locale"
                    :backlog-list="tracking.backlogList.value"
                    :set-exclusive-status="tracking.setExclusiveStatus"
                    :open-external-link="openExternalLink"
                />
            </div>
            <!-- 看完页容器 -->
            <div v-else-if="activePage === 'finished'" class="app-body tracking-body">
                <!-- 看完页：接收已完成列表与状态切换能力 -->
                <FinishedPage
                    :locale="locale"
                    :finished-list="tracking.finishedList.value"
                    :set-exclusive-status="tracking.setExclusiveStatus"
                    :open-external-link="openExternalLink"
                />
            </div>
            <!-- 搜索页：提供搜索状态、外链与下载委托函数 -->
            <SearchPage
                v-else-if="activePage === 'search'"
                :locale="locale"
                :search="search"
                :open-external-link="openExternalLink"
                :handle-download-click="handleDownloadClick"
            />
            <!-- 下载页：提供下载模块状态与动作 -->
            <DownloadPage v-else-if="activePage === 'download'" :locale="locale" :downloads="downloads" />
            <!-- 轨道页：提供轨道解析与混流模块状态 -->
            <TracksPage v-else-if="activePage === 'tracks'" :locale="locale" :tracks="tracks" />
            <!-- 未知页面兜底：避免异常 page key 导致空白 -->
            <main v-else class="app-body">
                <!-- 占位卡片用于提示当前状态并显示 activePage 原始值 -->
                <a-card class="app-placeholder" :bordered="true">
                    <!-- 垂直布局容器：展示标题、副标题与当前页信息 -->
                    <a-space direction="vertical" size="16" fill>
                        <!-- 占位区主标题 -->
                        <a-typography-title :heading="4">
                            {{ t("shell.placeholder.title", locale) }}
                        </a-typography-title>
                        <!-- 占位区副标题 -->
                        <a-typography-text type="secondary">
                            {{ t("shell.placeholder.subtitle", locale) }}
                        </a-typography-text>
                        <!-- 显示当前未知页面键，便于排查路由状态 -->
                        <a-typography-text> {{ t("shell.placeholder.active", locale) }}：{{ activePage }} </a-typography-text>
                    </a-space>
                </a-card>
            </main>
        </div>
    </a-config-provider>
</template>
