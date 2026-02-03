import { computed, h, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NTag } from "naive-ui";
import type { MonthAnime, SeasonResponse, StaffGroup, CharacterLink } from "../types/anime";
import type { ItemStatus } from "../types/tracking";

const currentMonth = new Date().getMonth() + 1;
const getSeasonStartMonth = (value: number) => {
  if (value >= 1 && value <= 3) return 1;
  if (value >= 4 && value <= 6) return 4;
  if (value >= 7 && value <= 9) return 7;
  return 10;
};

const currentYear = new Date().getFullYear();
const minYear = 2000;
const yearOptions = Array.from({ length: currentYear - minYear + 1 }, (_, i) => {
  const value = currentYear - i;
  return { label: `${value}年`, value };
});

const seasonOptionsAll = [
  { label: "冬季", value: 1 },
  { label: "春季", value: 4 },
  { label: "夏季", value: 7 },
  { label: "秋季", value: 10 },
];

const seasonKeyFromMonth = (value: number | null) => {
  if (!value) return "";
  if (value === 1) return "winter";
  if (value === 4) return "spring";
  if (value === 7) return "summer";
  if (value === 10) return "autumn";
  return "";
};

const seasonLabelFromMonth = (value: number | null) => {
  if (!value) return "";
  if (value === 1) return "冬季";
  if (value === 4) return "春季";
  if (value === 7) return "夏季";
  if (value === 10) return "秋季";
  return "";
};

const preloadImages = (items: MonthAnime[], timeoutMs = 5000) => {
  const tasks = items.map(
    (item) =>
      new Promise<void>((resolve) => {
        const img = new Image();
        img.onload = () => resolve();
        img.onerror = () => resolve();
        img.src = item.image;
      })
  );
  return Promise.race([
    Promise.all(tasks).then(() => undefined),
    new Promise<void>((resolve) => setTimeout(resolve, timeoutMs)),
  ]);
};

const INACTIVITY_SCROLL_DELAY_MS = 5000;
const MIN_PROGRESS_DURATION_MS = 100_000;
const CATCHUP_DURATION_MS = 5_000;
const MAX_PROGRESS_BEFORE_FINISH = 99;

const OFFICIAL_TYPE_OPTIONS = [
  "科幻",
  "喜剧",
  "同人",
  "百合",
  "校园",
  "惊悚",
  "后宫",
  "机战",
  "悬疑",
  "恋爱",
  "奇幻",
  "推理",
  "运动",
  "耽美",
  "音乐",
  "战斗",
  "冒险",
  "萌系",
  "穿越",
  "玄幻",
  "乙女",
  "恐怖",
  "历史",
  "日常",
  "剧情",
  "武侠",
  "美食",
  "职场",
];

const OFFICIAL_REGION_OPTIONS = [
  "日本",
  "欧美",
  "中国",
  "美国",
  "韩国",
  "法国",
  "中国香港",
  "英国",
  "俄罗斯",
  "苏联",
  "捷克",
  "中国台湾",
  "马来西亚",
];

const OFFICIAL_AUDIENCE_OPTIONS = [
  "BL",
  "GL",
  "子供向",
  "女性向",
  "少女向",
  "少年向",
  "青年向",
];

const allFilterValue = "全部";

export const useQueryPage = (options: { ensureStatus: (id: number) => ItemStatus }) => {
  const year = ref<number | null>(new Date().getFullYear());
  const month = ref<number | null>(getSeasonStartMonth(currentMonth));

  const seasonOptions = computed(() => {
    if (!year.value) return seasonOptionsAll;
    if (year.value === currentYear) {
      return seasonOptionsAll.filter((option) => option.value <= currentMonth);
    }
    return seasonOptionsAll;
  });

  watch([year, seasonOptions], () => {
    const optionsList = seasonOptions.value;
    if (!optionsList.length) {
      month.value = null;
      return;
    }
    if (!optionsList.some((option) => option.value === month.value)) {
      month.value = optionsList[optionsList.length - 1].value;
    }
  });

  const loading = ref(false);
  const results = ref<MonthAnime[]>([]);
  const resultUrl = ref("");
  const resultFetchedAt = ref("");
  const errorMessage = ref("");
  const hasQueried = ref(false);
  const progress = ref(0);
  const progressStartAt = ref<number | null>(null);
  const selected = ref<MonthAnime | null>(null);
  const originLoadingId = ref<number | null>(null);
  const originError = ref("");
  const airedLoadingId = ref<number | null>(null);
  const airedError = ref("");
  const showStaffModal = ref(false);
  const queryPanelRef = ref<HTMLElement | null>(null);
  const resultListRef = ref<HTMLElement | null>(null);
  const listItemRefs = ref<Record<number, HTMLElement | null>>({});
  const staffLoadingId = ref<number | null>(null);
  const staffError = ref("");
  const staffCache = ref<Record<number, StaffGroup[]>>({});
  const characterLoadingId = ref<number | null>(null);
  const characterError = ref("");
  const characterCache = ref<Record<number, CharacterLink[]>>({});
  const summaryLoadingId = ref<number | null>(null);
  const summaryError = ref("");

  const selectedStaff = computed(() => {
    const id = selected.value?.id;
    if (!id) return [] as StaffGroup[];
    return staffCache.value[id] || [];
  });

  const selectedCharacters = computed(() => {
    const id = selected.value?.id;
    if (!id) return [] as CharacterLink[];
    return characterCache.value[id] || [];
  });

  const showResults = computed(() => hasQueried.value && !loading.value && progress.value >= 100);
  const detailVisible = computed(() => showResults.value && !!selected.value);

  let progressTimer: number | undefined;
  let catchupTimer: number | undefined;
  let listMouseLeaveTimer: number | undefined;
  const dataCache = new Map<string, SeasonResponse>();
  const queryToken = ref(0);

  const setListItemRef = (item: MonthAnime, el: HTMLElement | null) => {
    if (!item.id) return;
    if (el) {
      listItemRefs.value[item.id] = el;
    } else {
      delete listItemRefs.value[item.id];
    }
  };

  const clearListMouseLeaveTimer = () => {
    if (listMouseLeaveTimer) {
      window.clearTimeout(listMouseLeaveTimer);
      listMouseLeaveTimer = undefined;
    }
  };

  const findScrollParent = (el: HTMLElement | null): HTMLElement | null => {
    let node: HTMLElement | null = el?.parentElement || null;
    while (node) {
      const style = window.getComputedStyle(node);
      const overflowY = style.overflowY;
      if (overflowY === "auto" || overflowY === "scroll") {
        return node;
      }
      node = node.parentElement;
    }
    return null;
  };

  const scrollToSelectedItem = async (behavior: ScrollBehavior = "smooth") => {
    if (!detailVisible.value) return;
    const id = selected.value?.id;
    if (!id) return;
    await nextTick();
    const target = listItemRefs.value[id];
    if (!target) return;

    const container = findScrollParent(target) || resultListRef.value;
    if (container) {
      const containerRect = container.getBoundingClientRect();
      const targetRect = target.getBoundingClientRect();
      const targetCenter = targetRect.top - containerRect.top + container.scrollTop + targetRect.height / 2;
      const nextScrollTop = targetCenter - container.clientHeight / 2;
      const maxScrollTop = container.scrollHeight - container.clientHeight;
      const clamped = Math.max(0, Math.min(nextScrollTop, maxScrollTop));
      container.scrollTo({ top: clamped, behavior });
      return;
    }

    target.scrollIntoView({ behavior, block: "center", inline: "center" });
  };

  const handleListMouseEnter = () => {
    clearListMouseLeaveTimer();
  };

  const handleListMouseLeave = () => {
    if (!detailVisible.value) return;
    clearListMouseLeaveTimer();
    listMouseLeaveTimer = window.setTimeout(() => {
      void scrollToSelectedItem();
    }, INACTIVITY_SCROLL_DELAY_MS);
  };

  onBeforeUnmount(() => {
    clearListMouseLeaveTimer();
    if (progressTimer) window.clearInterval(progressTimer);
    if (catchupTimer) window.clearInterval(catchupTimer);
  });

  const monthFilter = ref<Array<number | string>>([]);
  const typeFilter = ref<string[]>([]);
  const regionFilter = ref<string[]>([]);
  const audienceFilter = ref<string[]>([]);
  const filterLoading = ref(false);
  const filtersInitialized = ref(false);

  const monthFilterOptions = computed(() => {
    const values = new Set<number>();
    results.value.forEach((item) => {
      if (typeof item.month === "number") values.add(item.month);
    });
    const options = Array.from(values)
      .sort((a, b) => a - b)
      .map((value) => ({ label: `${value}月`, value }));
    return [{ label: allFilterValue, value: allFilterValue }, ...options];
  });

  const typeOptions = computed(() => [
    { label: allFilterValue, value: allFilterValue },
    ...OFFICIAL_TYPE_OPTIONS.map((value) => ({ label: value, value })),
  ]);
  const regionOptions = computed(() => [
    { label: allFilterValue, value: allFilterValue },
    ...OFFICIAL_REGION_OPTIONS.map((value) => ({ label: value, value })),
  ]);
  const audienceOptions = computed(() => [
    { label: allFilterValue, value: allFilterValue },
    ...OFFICIAL_AUDIENCE_OPTIONS.map((value) => ({ label: value, value })),
  ]);

  const applySelectAllBehavior = <T extends string | number>(
    next: T[],
    optionsList: { value: T }[],
    prev: T[]
  ): T[] => {
    const real = optionsList
      .map((o) => o.value)
      .filter((v) => v !== (allFilterValue as unknown as T));
    if (!real.length) return [] as T[];

    const prevHasAll = prev.includes(allFilterValue as unknown as T);
    const nextHasAll = next.includes(allFilterValue as unknown as T);

    if (prevHasAll !== nextHasAll) {
      return nextHasAll ? [allFilterValue as unknown as T, ...real] : [];
    }

    const nextReal = next.filter((v) => v !== (allFilterValue as unknown as T));
    const allRealSelected = nextReal.length === real.length;

    return allRealSelected ? [allFilterValue as unknown as T, ...real] : nextReal;
  };

  const matchesTextFilter = (values: string[] | undefined, selectedList: string[], total: number) => {
    if (!selectedList.length || selectedList.includes(allFilterValue) || selectedList.length >= total) return true;
    if (!values || !values.length) return false;
    return values.some((value) => selectedList.includes(value));
  };

  const filteredResults = computed(() => {
    return results.value.filter((item) => {
      const monthAll =
        !monthFilter.value.length ||
        monthFilter.value.includes(allFilterValue) ||
        monthFilter.value.length >= monthFilterOptions.value.length - 1;
      const monthMatch = monthAll || (item.month && monthFilter.value.includes(item.month));
      if (!monthMatch) return false;
      if (!matchesTextFilter(item.types, typeFilter.value, OFFICIAL_TYPE_OPTIONS.length)) return false;
      if (!matchesTextFilter(item.regions, regionFilter.value, OFFICIAL_REGION_OPTIONS.length)) return false;
      if (!matchesTextFilter(item.audiences, audienceFilter.value, OFFICIAL_AUDIENCE_OPTIONS.length)) return false;
      return true;
    });
  });

  const resultCount = computed(() => filteredResults.value.length);

  const updateDefaultFilters = () => {
    if (filtersInitialized.value) return;
    monthFilter.value = [];
    typeFilter.value = [];
    regionFilter.value = [];
    audienceFilter.value = [];
    filtersInitialized.value = true;
  };

  watch(monthFilterOptions, () => {
    updateDefaultFilters();
  });

  const getFilterLabel = (selectedCount: number, totalCount: number) => {
    if (selectedCount === 0) return "未筛选";
    if (selectedCount >= totalCount) return "全部";
    return "已筛选";
  };

  const monthFilterLabel = computed(() =>
    getFilterLabel(
      monthFilter.value.filter((v) => v !== allFilterValue).length,
      Math.max(monthFilterOptions.value.length - 1, 0)
    )
  );
  const typeFilterLabel = computed(() =>
    getFilterLabel(typeFilter.value.filter((v) => v !== allFilterValue).length, OFFICIAL_TYPE_OPTIONS.length)
  );
  const regionFilterLabel = computed(() =>
    getFilterLabel(regionFilter.value.filter((v) => v !== allFilterValue).length, OFFICIAL_REGION_OPTIONS.length)
  );
  const audienceFilterLabel = computed(() =>
    getFilterLabel(audienceFilter.value.filter((v) => v !== allFilterValue).length, OFFICIAL_AUDIENCE_OPTIONS.length)
  );

  const handleMonthFilterChange = (value: Array<number | string>) => {
    const prev = monthFilter.value;
    const next = applySelectAllBehavior(value, monthFilterOptions.value, prev);
    monthFilter.value = next;
    selected.value = null;
  };

  const handleTypeFilterChange = (value: string[]) => {
    const prev = typeFilter.value;
    const next = applySelectAllBehavior(value, typeOptions.value, prev);
    typeFilter.value = next;
    selected.value = null;
  };

  const handleRegionFilterChange = (value: string[]) => {
    const prev = regionFilter.value;
    const next = applySelectAllBehavior(value, regionOptions.value, prev);
    regionFilter.value = next;
    selected.value = null;
  };

  const handleAudienceFilterChange = (value: string[]) => {
    const prev = audienceFilter.value;
    const next = applySelectAllBehavior(value, audienceOptions.value, prev);
    audienceFilter.value = next;
    selected.value = null;
  };

  const renderFilterTag = (label: string) => h(NTag, { size: "small", type: "info" }, { default: () => label });

  const loadSeasonData = async (yearValue: number, seasonMonth: number) => {
    const seasonKey = seasonKeyFromMonth(seasonMonth);
    if (!seasonKey) throw new Error("季节选择无效");
    const cacheKey = `${yearValue}-${seasonKey}`;
    if (dataCache.has(cacheKey)) return dataCache.get(cacheKey)!;
    const payload = await invoke<SeasonResponse>("get_season_subjects", {
      year: yearValue,
      season: seasonKey,
    });
    if (!payload || !Array.isArray(payload.months)) {
      throw new Error("季度数据格式不正确");
    }
    dataCache.set(cacheKey, payload);
    return payload;
  };

  const clearProgressTimers = () => {
    if (progressTimer) {
      window.clearInterval(progressTimer);
      progressTimer = undefined;
    }
    if (catchupTimer) {
      window.clearInterval(catchupTimer);
      catchupTimer = undefined;
    }
  };

  const cancelActiveQuery = () => {
    queryToken.value += 1;
    loading.value = false;
    hasQueried.value = false;
    clearProgressTimers();
    progress.value = 0;
  };

  const startProgress = () => {
    clearProgressTimers();
    progress.value = 0;
    progressStartAt.value = Date.now();
    progressTimer = window.setInterval(() => {
      const start = progressStartAt.value;
      if (!start) return;
      const elapsed = Date.now() - start;
      const target = Math.min(
        MAX_PROGRESS_BEFORE_FINISH,
        Math.floor((elapsed / MIN_PROGRESS_DURATION_MS) * MAX_PROGRESS_BEFORE_FINISH)
      );
      if (target > progress.value) {
        progress.value = target;
      }
      if (target >= MAX_PROGRESS_BEFORE_FINISH) {
        clearProgressTimers();
        progressTimer = window.setInterval(() => {
          progress.value = MAX_PROGRESS_BEFORE_FINISH;
        }, 1000);
      }
    }, 120);
  };

  const finishProgress = async () => {
    const startedAt = progressStartAt.value;
    clearProgressTimers();
    if (!startedAt) {
      progress.value = 100;
      return;
    }
    const elapsed = Date.now() - startedAt;

    if (elapsed >= MIN_PROGRESS_DURATION_MS) {
      progress.value = 100;
      return;
    }

    const startValue = progress.value;
    const sprintStart = Date.now();
    catchupTimer = window.setInterval(() => {
      const ratio = Math.min(1, (Date.now() - sprintStart) / CATCHUP_DURATION_MS);
      const target = Math.round(startValue + (100 - startValue) * ratio);
      progress.value = target;
      if (ratio >= 1) {
        clearProgressTimers();
      }
    }, 50);
  };

  const handleQuery = async () => {
    if (!year.value || !month.value) return;
    const token = ++queryToken.value;
    loading.value = true;
    errorMessage.value = "";
    hasQueried.value = true;
    filtersInitialized.value = false;
    monthFilter.value = [];
    typeFilter.value = [];
    regionFilter.value = [];
    audienceFilter.value = [];
    startProgress();
    try {
      const data = await loadSeasonData(year.value, month.value);
      if (token !== queryToken.value) return;
      const merged = data.months
        .flatMap((item) => item.list.map((entry) => ({ ...entry, month: item.month })))
        .sort((a, b) => {
          const monthDiff = (a.month ?? 0) - (b.month ?? 0);
          if (monthDiff !== 0) return monthDiff;
          return (a.date || "").localeCompare(b.date || "");
        });
      if (token !== queryToken.value) return;
      results.value = merged;
      resultUrl.value = data.source;
      resultFetchedAt.value = data.fetchedAt;
      selected.value = null;
      monthFilter.value = monthFilterOptions.value.map((option) => option.value);
      if (merged.length > 0) {
        await preloadImages(merged);
      }
      if (merged.length > 0) {
        await loadFiltersForResults(merged);
      }
    } catch (error) {
      if (token === queryToken.value) {
        errorMessage.value = String(error);
      }
    } finally {
      if (token === queryToken.value) {
        loading.value = false;
        await finishProgress();
      }
    }
  };

  watch([year, month], () => {
    if (loading.value) {
      cancelActiveQuery();
    }
  });

  const loadFiltersForResults = async (items: MonthAnime[]) => {
    filterLoading.value = true;
    const queue = items.slice();
    const worker = async () => {
      while (queue.length) {
        const item = queue.shift();
        if (!item?.id) continue;
        try {
          const payload = await invoke<{
            id: number;
            types: string[];
            regions: string[];
            audiences: string[];
          }>("get_subject_filters", { id: item.id });
          item.types = payload.types || [];
          item.regions = payload.regions || [];
          item.audiences = payload.audiences || [];
        } catch (_) {
          item.types = item.types || [];
          item.regions = item.regions || [];
          item.audiences = item.audiences || [];
        }
      }
    };
    const concurrency = 6;
    await Promise.all(Array.from({ length: concurrency }, () => worker()));
    filterLoading.value = false;
    updateDefaultFilters();
  };

  const handleSelect = (item: MonthAnime) => {
    selected.value = item;
    if (item.id) options.ensureStatus(item.id);
    originError.value = "";
    airedError.value = "";
    staffError.value = "";
    characterError.value = "";
    summaryError.value = "";
    void loadOrigin(item);
    void loadAiredCount(item);
    void loadStaff(item);
    void loadCharacters(item);
    void loadSummaryCn(item);
  };

  const handleStaffOpen = async () => {
    if (!selected.value) return;
    showStaffModal.value = true;
    await loadStaff(selected.value);
  };

  const loadOrigin = async (item: MonthAnime) => {
    if (!item?.id) return;
    if (item.origin !== undefined) return;
    originLoadingId.value = item.id;
    originError.value = "";
    try {
      const payload = await invoke<{ id: number; origin?: string | null }>("get_subject_origin", {
        id: item.id,
      });
      item.origin = payload.origin ?? "";
    } catch (error) {
      originError.value = String(error);
      item.origin = "";
    } finally {
      if (originLoadingId.value === item.id) {
        originLoadingId.value = null;
      }
    }
  };

  const loadAiredCount = async (item: MonthAnime) => {
    if (!item?.id) return;
    if (item.airedCount !== undefined) return;
    airedLoadingId.value = item.id;
    airedError.value = "";
    try {
      const payload = await invoke<{ id: number; airedCount?: number | null; totalCount?: number | null }>(
        "get_subject_aired_count",
        {
          id: item.id,
        }
      );
      item.airedCount = payload.airedCount ?? 0;
      item.totalCount = payload.totalCount ?? 0;
    } catch (error) {
      airedError.value = String(error);
      item.airedCount = 0;
      item.totalCount = 0;
    } finally {
      if (airedLoadingId.value === item.id) {
        airedLoadingId.value = null;
      }
    }
  };

  const loadStaff = async (item: MonthAnime) => {
    if (!item?.id) return;
    if (staffCache.value[item.id]) return;
    staffLoadingId.value = item.id;
    staffError.value = "";
    try {
      const payload = await invoke<{ id: number; groups: StaffGroup[] }>("get_subject_staff", {
        id: item.id,
      });
      staffCache.value = { ...staffCache.value, [item.id]: payload.groups || [] };
    } catch (error) {
      staffError.value = String(error);
      staffCache.value = { ...staffCache.value, [item.id]: [] };
    } finally {
      if (staffLoadingId.value === item.id) {
        staffLoadingId.value = null;
      }
    }
  };

  const loadCharacters = async (item: MonthAnime) => {
    if (!item?.id) return;
    if (characterCache.value[item.id]) return;
    characterLoadingId.value = item.id;
    characterError.value = "";
    try {
      const payload = await invoke<{ id: number; characters: CharacterLink[] }>(
        "get_subject_characters",
        { id: item.id }
      );
      characterCache.value = { ...characterCache.value, [item.id]: payload.characters || [] };
    } catch (error) {
      characterError.value = String(error);
      characterCache.value = { ...characterCache.value, [item.id]: [] };
    } finally {
      if (characterLoadingId.value === item.id) {
        characterLoadingId.value = null;
      }
    }
  };

  const loadSummaryCn = async (item: MonthAnime) => {
    if (!item?.id) return;
    const shouldRefresh =
      item.summaryTranslateFailed ||
      (item.summaryCn !== undefined &&
        !item.summaryTranslated &&
        !item.summaryTranslateFailed &&
        (item.summaryCn || "") === (item.summary || ""));
    if (item.summaryCn !== undefined && !shouldRefresh) return;
    summaryLoadingId.value = item.id;
    summaryError.value = "";
    try {
      const payload = await invoke<{ id: number; summary: string; translated: boolean; error?: string | null }>(
        "get_subject_summary_cn",
        {
          id: item.id,
          summary: item.summary || "",
        }
      );
      item.summaryCn = payload.summary;
      item.summaryTranslated = payload.translated;
      item.summaryTranslateFailed = false;
      if (payload.error) {
        summaryError.value = payload.error;
        item.summaryTranslateFailed = true;
      }
    } catch (error) {
      summaryError.value = String(error);
      item.summaryCn = "";
      item.summaryTranslated = false;
      item.summaryTranslateFailed = true;
    } finally {
      if (summaryLoadingId.value === item.id) {
        summaryLoadingId.value = null;
      }
    }
  };

  const setQueryPanelHeight = async () => {
    await nextTick();
    const height = queryPanelRef.value?.getBoundingClientRect().height;
    if (height) {
      document.documentElement.style.setProperty("--query-panel-height", `${height}px`);
    }
  };

  return {
    year,
    month,
    yearOptions,
    seasonOptions,
    seasonLabelFromMonth,
    loading,
    results,
    resultUrl,
    resultFetchedAt,
    errorMessage,
    hasQueried,
    progress,
    selected,
    originLoadingId,
    originError,
    airedLoadingId,
    airedError,
    showStaffModal,
    queryPanelRef,
    resultListRef,
    listItemRefs,
    staffLoadingId,
    staffError,
    staffCache,
    characterLoadingId,
    characterError,
    characterCache,
    summaryLoadingId,
    summaryError,
    selectedStaff,
    selectedCharacters,
    showResults,
    detailVisible,
    setListItemRef,
    handleListMouseEnter,
    handleListMouseLeave,
    scrollToSelectedItem,
    monthFilter,
    typeFilter,
    regionFilter,
    audienceFilter,
    filterLoading,
    monthFilterOptions,
    typeOptions,
    regionOptions,
    audienceOptions,
    monthFilterLabel,
    typeFilterLabel,
    regionFilterLabel,
    audienceFilterLabel,
    handleMonthFilterChange,
    handleTypeFilterChange,
    handleRegionFilterChange,
    handleAudienceFilterChange,
    renderFilterTag,
    filteredResults,
    resultCount,
    handleQuery,
    handleSelect,
    handleStaffOpen,
    setQueryPanelHeight,
  };
};

export type UseQueryPageReturn = ReturnType<typeof useQueryPage>;
