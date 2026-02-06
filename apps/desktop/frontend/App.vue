<script setup lang="ts">
/**
 * 应用根组件：
 * - 初始化全局状态与各模块 composables
 * - 控制页面切换与弹窗显示
 */
import { computed, ref, watch } from "vue";
import { NConfigProvider, darkTheme } from "naive-ui";
import AppTitlebar from "./shared/components/AppTitlebar.vue";
import StaffModal from "./modules/query/components/StaffModal.vue";
import AliasModal from "./modules/search/components/AliasModal.vue";
import MixQueueDetailModal from "./modules/tracks/components/MixQueueDetailModal.vue";
import { useWindowControls } from "./shared/composables/useWindowControls";
import { useExternalLink } from "./shared/composables/useExternalLink";
import { useTracking } from "./modules/tracking/composables/useTracking";
import { useQueryPage } from "./modules/query/composables/useQueryPage";
import { useSearchPage } from "./modules/search/composables/useSearchPage";
import { useDownloadPage } from "./modules/download/composables/useDownloadPage";
import { useTracksPage } from "./modules/tracks/composables/useTracksPage";
import QueryPage from "./modules/query/pages/QueryPage.vue";
import WatchingPage from "./modules/tracking/pages/WatchingPage.vue";
import BacklogPage from "./modules/tracking/pages/BacklogPage.vue";
import FinishedPage from "./modules/tracking/pages/FinishedPage.vue";
import SearchPage from "./modules/search/pages/SearchPage.vue";
import DownloadPage from "./modules/download/pages/DownloadPage.vue";
import TracksPage from "./modules/tracks/pages/TracksPage.vue";
import type { PageKey } from "./shared/types/page";
import type { ItemStatus } from "./modules/tracking/types/tracking";

// 是否启用暗色主题。
const isDark = ref(false);
// 当前主题对象（Naive UI）。
const theme = computed(() => (isDark.value ? darkTheme : null));

// 当前激活页面。
const activePage = ref<PageKey>("query");
// 页面切换函数（由标题栏触发）。
const switchPage = (page: PageKey) => {
  activePage.value = page;
};

// 便捷判断当前页面，用于模板条件渲染。
const isQueryPage = computed(() => activePage.value === "query");
const isWatchingPage = computed(() => activePage.value === "watching");
const isBacklogPage = computed(() => activePage.value === "backlog");
const isFinishedPage = computed(() => activePage.value === "finished");
const isSearchPage = computed(() => activePage.value === "search");
const isDownloadPage = computed(() => activePage.value === "download");
const isTracksPage = computed(() => activePage.value === "tracks");

// 窗口控制与外链打开。
const { handleMinimize, handleClose } = useWindowControls();
const { openExternalLink } = useExternalLink();
// 各功能模块的组合式逻辑。
const tracking = useTracking();
const query = useQueryPage({ ensureStatus: tracking.ensureStatus });
const search = useSearchPage({ trackedItems: tracking.trackedItems });
const downloads = useDownloadPage();
const tracks = useTracksPage();

// 当前选中条目的追番状态（供详情按钮使用）。
const selectedStatus = computed<ItemStatus>(() => {
  const id = query.selected.value?.id;
  if (!id) return { watching: false, backlog: false, watched: false };
  return tracking.ensureStatus(id);
});

// 弹窗开关的更新方法（用于 v-model）。
const updateStaffModalVisible = (value: boolean) => {
  query.showStaffModal.value = value;
};

const updateAliasModalVisible = (value: boolean) => {
  search.aliasModalVisible.value = value;
};

const updateAliasSelected = (value: string[]) => {
  search.aliasSelected.value = value;
};

const updateMixQueueDetailVisible = (value: boolean) => {
  tracks.mixQueueDetailVisible.value = value;
};

// 当切到“正在追番”页时刷新详细信息。
watch(isWatchingPage, (active: boolean) => {
  if (active) {
    void tracking.refreshWatchingDetails();
  }
});

// 从其他页回到查询页时，自动滚动到已选条目。
watch(activePage, (next: PageKey, prev: PageKey | undefined) => {
  if (next === "query" && prev && prev !== "query") {
    void query.scrollToSelectedItem();
  }
});
</script>

<template>
  <!-- Naive UI 主题提供器 -->
  <NConfigProvider :theme="theme">
    <!-- 应用整体布局容器 -->
    <div class="app-shell">
      <AppTitlebar
        :active-page="activePage"
        :on-minimize="handleMinimize"
        :on-close="handleClose"
        @switch="switchPage"
      />

      <!-- 季度查询页 -->
      <QueryPage
        v-if="isQueryPage"
        :query="query"
        :open-external-link="openExternalLink"
        :tracking="{
          selectedStatus,
          currentStatusKey: tracking.currentStatusKey,
          labelForAction: tracking.labelForAction,
          setExclusiveStatus: tracking.setExclusiveStatus,
        }"
      />

      <!-- 正在追番页 -->
      <WatchingPage
        v-else-if="isWatchingPage"
        :watching-by-weekday="tracking.watchingByWeekday.value"
        :set-exclusive-status="tracking.setExclusiveStatus"
        :open-external-link="openExternalLink"
      />

      <!-- 补番计划页 -->
      <BacklogPage
        v-else-if="isBacklogPage"
        :backlog-list="tracking.backlogList.value"
        :set-exclusive-status="tracking.setExclusiveStatus"
        :open-external-link="openExternalLink"
      />

      <!-- 已完番剧页 -->
      <FinishedPage
        v-else-if="isFinishedPage"
        :finished-list="tracking.finishedList.value"
        :set-exclusive-status="tracking.setExclusiveStatus"
        :open-external-link="openExternalLink"
      />

      <!-- 搜索资源页 -->
      <SearchPage
        v-else-if="isSearchPage"
        :search="search"
        :open-external-link="openExternalLink"
        :handle-download-click="downloads.handleDownloadClick"
      />

      <!-- 下载页 -->
      <DownloadPage v-else-if="isDownloadPage" :downloads="downloads" />

      <!-- 轨道工坊页 -->
      <TracksPage v-else-if="isTracksPage" :tracks="tracks" />
    </div>

    <!-- 工作人员弹窗 -->
    <StaffModal
      :show="query.showStaffModal.value"
      :selected="query.selected.value"
      :staff-loading-id="query.staffLoadingId.value"
      :staff-error="query.staffError.value"
      :selected-staff="query.selectedStaff.value"
      :open-external-link="openExternalLink"
      @update:show="updateStaffModalVisible"
    />

    <!-- 别名选择弹窗 -->
    <AliasModal
      :show="search.aliasModalVisible.value"
      :pending-tracked-name="search.pendingTrackedName.value"
      :alias-loading="search.aliasLoading.value"
      :alias-options="search.aliasOptions.value"
      :alias-selected="search.aliasSelected.value"
      :on-cancel="search.cancelAliasSelection"
      :on-confirm="search.confirmAliasSelection"
      @update:show="updateAliasModalVisible"
      @update:aliasSelected="updateAliasSelected"
    />

    <!-- 混流任务详情弹窗 -->
    <MixQueueDetailModal
      :show="tracks.mixQueueDetailVisible.value"
      :selected-mix-task="tracks.selectedMixTask.value"
      @update:show="updateMixQueueDetailVisible"
    />
  </NConfigProvider>
</template>
