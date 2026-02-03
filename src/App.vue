<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NConfigProvider, darkTheme } from "naive-ui";
import AppTitlebar from "./components/AppTitlebar.vue";
import QueryPage from "./pages/QueryPage.vue";
import WatchingPage from "./pages/WatchingPage.vue";
import BacklogPage from "./pages/BacklogPage.vue";
import FinishedPage from "./pages/FinishedPage.vue";
import SearchPage from "./pages/SearchPage.vue";
import DownloadPage from "./pages/DownloadPage.vue";
import TracksPage from "./pages/TracksPage.vue";
import StaffModal from "./components/StaffModal.vue";
import AliasModal from "./components/AliasModal.vue";
import MixQueueDetailModal from "./components/MixQueueDetailModal.vue";
import { useWindowControls } from "./composables/useWindowControls";
import { useExternalLink } from "./composables/useExternalLink";
import { useTracking } from "./composables/useTracking";
import { useQueryPage } from "./composables/useQueryPage";
import { useSearchPage } from "./composables/useSearchPage";
import { useDownloadPage } from "./composables/useDownloadPage";
import { useTracksPage } from "./composables/useTracksPage";
import type { PageKey } from "./types/page";
import type { ItemStatus } from "./types/tracking";

const isDark = ref(false);
const theme = computed(() => (isDark.value ? darkTheme : null));

const activePage = ref<PageKey>("query");
const switchPage = (page: PageKey) => {
  activePage.value = page;
};

const isQueryPage = computed(() => activePage.value === "query");
const isWatchingPage = computed(() => activePage.value === "watching");
const isBacklogPage = computed(() => activePage.value === "backlog");
const isFinishedPage = computed(() => activePage.value === "finished");
const isSearchPage = computed(() => activePage.value === "search");
const isDownloadPage = computed(() => activePage.value === "download");
const isTracksPage = computed(() => activePage.value === "tracks");

const { handleMinimize, handleClose } = useWindowControls();
const { openExternalLink } = useExternalLink();
const tracking = useTracking();
const query = useQueryPage({ ensureStatus: tracking.ensureStatus });
const search = useSearchPage({ trackedItems: tracking.trackedItems });
const downloads = useDownloadPage();
const tracks = useTracksPage();

const selectedStatus = computed<ItemStatus>(() => {
  const id = query.selected.value?.id;
  if (!id) return { watching: false, backlog: false, watched: false };
  return tracking.ensureStatus(id);
});

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

watch(isWatchingPage, (active) => {
  if (active) {
    void tracking.refreshWatchingDetails();
  }
});

watch(activePage, (next, prev) => {
  if (next === "query" && prev && prev !== "query") {
    void query.scrollToSelectedItem();
  }
});
</script>

<template>
  <NConfigProvider :theme="theme">
    <div class="app-shell">
      <AppTitlebar
        :active-page="activePage"
        :on-minimize="handleMinimize"
        :on-close="handleClose"
        @switch="switchPage"
      />

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

      <WatchingPage
        v-else-if="isWatchingPage"
        :watching-by-weekday="tracking.watchingByWeekday.value"
        :set-exclusive-status="tracking.setExclusiveStatus"
        :open-external-link="openExternalLink"
      />

      <BacklogPage
        v-else-if="isBacklogPage"
        :backlog-list="tracking.backlogList.value"
        :set-exclusive-status="tracking.setExclusiveStatus"
        :open-external-link="openExternalLink"
      />

      <FinishedPage
        v-else-if="isFinishedPage"
        :finished-list="tracking.finishedList.value"
        :set-exclusive-status="tracking.setExclusiveStatus"
        :open-external-link="openExternalLink"
      />

      <SearchPage
        v-else-if="isSearchPage"
        :search="search"
        :open-external-link="openExternalLink"
        :handle-download-click="downloads.handleDownloadClick"
      />

      <DownloadPage v-else-if="isDownloadPage" :downloads="downloads" />

      <TracksPage v-else-if="isTracksPage" :tracks="tracks" />
    </div>

    <StaffModal
      :show="query.showStaffModal.value"
      :selected="query.selected.value"
      :staff-loading-id="query.staffLoadingId.value"
      :staff-error="query.staffError.value"
      :selected-staff="query.selectedStaff.value"
      :open-external-link="openExternalLink"
      @update:show="updateStaffModalVisible"
    />

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

    <MixQueueDetailModal
      :show="tracks.mixQueueDetailVisible.value"
      :selected-mix-task="tracks.selectedMixTask.value"
      @update:show="updateMixQueueDetailVisible"
    />
  </NConfigProvider>
</template>
