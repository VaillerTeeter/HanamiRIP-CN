<script setup lang="ts">
import { computed, h, nextTick, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  NButton,
  NCard,
  NConfigProvider,
  NInput,
  NModal,
  NProgress,
  NSelect,
  NSpace,
  NTag,
  darkTheme,
} from "naive-ui";
interface MonthAnime {
  id: number;
  name: string;
  nameCn?: string;
  alias?: string;
  origin?: string;
  airedCount?: number;
  summaryCn?: string;
  summaryTranslated?: boolean;
  summaryTranslateFailed?: boolean;
  types?: string[];
  regions?: string[];
  audiences?: string[];
  image: string;
  date?: string;
  rating?: number | null;
  summary?: string;
  url?: string;
  month?: number;
}

interface StaffPerson {
  id: number;
  name: string;
  url: string;
}

interface StaffGroup {
  role: string;
  people: StaffPerson[];
}

interface CharacterLink {
  id: number;
  name: string;
  url: string;
  relation?: string;
}

interface SeasonMonthData {
  year: number;
  month: number;
  count: number;
  list: MonthAnime[];
}

interface SeasonResponse {
  year: number;
  season: string;
  fetchedAt: string;
  source: string;
  months: SeasonMonthData[];
}

const isDark = ref(false);
const theme = computed(() => (isDark.value ? darkTheme : null));

type PageKey = "query" | "watching" | "backlog" | "finished" | "search";
const activePage = ref<PageKey>("query");
const isQueryPage = computed(() => activePage.value === "query");
const isWatchingPage = computed(() => activePage.value === "watching");
const isBacklogPage = computed(() => activePage.value === "backlog");
const isFinishedPage = computed(() => activePage.value === "finished");
const isSearchPage = computed(() => activePage.value === "search");
const switchPage = (page: PageKey) => {
  activePage.value = page;
};

const appWindow = getCurrentWindow();

const handleMinimize = async () => {
  await appWindow.minimize();
};

const handleClose = async () => {
  await appWindow.close();
};

const currentMonth = new Date().getMonth() + 1;
const getSeasonStartMonth = (value: number) => {
  if (value >= 1 && value <= 3) return 1;
  if (value >= 4 && value <= 6) return 4;
  if (value >= 7 && value <= 9) return 7;
  return 10;
};
const year = ref<number | null>(new Date().getFullYear());
const month = ref<number | null>(getSeasonStartMonth(currentMonth));

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

const seasonOptions = computed(() => {
  if (!year.value) return seasonOptionsAll;
  if (year.value === currentYear) {
    return seasonOptionsAll.filter((option) => option.value <= currentMonth);
  }
  return seasonOptionsAll;
});

watch([year, seasonOptions], () => {
  const options = seasonOptions.value;
  if (!options.length) {
    month.value = null;
    return;
  }
  if (!options.some((option) => option.value === month.value)) {
    month.value = options[options.length - 1].value;
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
const staffLoadingId = ref<number | null>(null);
const staffError = ref("");
const staffCache = ref<Record<number, StaffGroup[]>>({});
const characterLoadingId = ref<number | null>(null);
const characterError = ref("");
const characterCache = ref<Record<number, CharacterLink[]>>({});
const summaryLoadingId = ref<number | null>(null);
const summaryError = ref("");
const formatRating = (value?: number | null) => (typeof value === "number" ? value.toFixed(1) : "0.0");
const formatStars = (value?: number | null) => {
  if (typeof value !== "number") return "☆☆☆☆☆";
  const normalized = Math.max(0, Math.min(10, value));
  const filled = Math.round(normalized / 2);
  return "★".repeat(filled) + "☆".repeat(5 - filled);
};
const weekdayLabels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
const formatAirDate = (value?: string) => {
  if (!value) return "未知";
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return `${value}（${weekdayLabels[parsed.getDay()]}）`;
};
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
const showResults = computed(() => hasQueried.value && !loading.value && progress.value >= 100);
let progressTimer: number | undefined;
let catchupTimer: number | undefined;
const dataCache = new Map<string, SeasonResponse>();
const queryToken = ref(0);
type ItemStatus = { watching: boolean; backlog: boolean; watched: boolean };
const statuses = ref<Record<number, ItemStatus>>({});
type TrackedItem = MonthAnime & ItemStatus;
const trackedItems = ref<TrackedItem[]>([]);
const parseDateValue = (date?: string) => {
  if (!date) return Number.POSITIVE_INFINITY;
  const parsed = new Date(`${date}T00:00:00`);
  return Number.isNaN(parsed.getTime()) ? Number.POSITIVE_INFINITY : parsed.getTime();
};
const sortByDate = (list: TrackedItem[]) =>
  list
    .slice()
    .sort((a, b) => parseDateValue(a.date) - parseDateValue(b.date));

const watchingList = computed(() => sortByDate(trackedItems.value.filter((item) => item.watching)));
const backlogList = computed(() => sortByDate(trackedItems.value.filter((item) => item.backlog)));
const finishedList = computed(() => sortByDate(trackedItems.value.filter((item) => item.watched)));
const watchingByWeekday = computed(() => {
  const groups = new Map<number | null, TrackedItem[]>();
  watchingList.value.forEach((item) => {
    const date = item.date;
    if (!date) {
      groups.set(null, [...(groups.get(null) || []), item]);
      return;
    }
    const parsed = new Date(`${date}T00:00:00`);
    if (Number.isNaN(parsed.getTime())) {
      groups.set(null, [...(groups.get(null) || []), item]);
      return;
    }
    const day = parsed.getDay();
    groups.set(day, [...(groups.get(day) || []), item]);
  });
  const order: Array<number | null> = [1, 2, 3, 4, 5, 6, 0, null];
  return order
    .map((key) => ({
      key,
      label: key === null ? "未知日期" : weekdayLabels[key],
      items: groups.get(key) || [],
    }))
    .filter((group) => group.items.length);
});
type StatusKey = "watching" | "backlog" | "watched" | null;
const formatMonth = (value: number) => String(value).padStart(2, "0");

const monthFilter = ref<Array<number | string>>([]);
const typeFilter = ref<string[]>([]);
const regionFilter = ref<string[]>([]);
const audienceFilter = ref<string[]>([]);
const filterLoading = ref(false);
const filtersInitialized = ref(false);
const allFilterValue = "全部";

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

/**
 * 统一处理四个筛选器的「全部」行为
 * 规则：
 * 1. 点亮「全部」→ 所有子项被勾选，且「全部」点亮
 * 2. 熄灭「全部」→ 所有子项被取消，且「全部」熄灭
 * 3. 点击任意子项后，若子项全部一致则把「全部」同步成一致；否则「全部」熄灭
 * 4. 全选状态下，熄灭任一子项，都会将全选熄灭
 * 5. 子项全部勾选，自动点亮全选
 * 6. 主动点亮"全部"选项：判断子项是否点亮，没有点亮的操作子项点亮
 * 7. 主动熄灭"全部"选项：判断子项是否熄灭，没有熄灭的操作子项熄灭
 */
const applySelectAllBehavior = <T extends string | number>(
  next: T[],               // 用户刚刚点完之后的原始数组
  options: { value: T }[], // 下拉框里所有选项（含“全部”）
  prev: T[]                // 点击前的旧数组
): T[] => {
  const REAL = options
    .map(o => o.value)
    .filter(v => v !== (allFilterValue as unknown as T));
  if (!REAL.length) return [] as T[];

  const prevHasAll = prev.includes(allFilterValue as unknown as T);
  const nextHasAll = next.includes(allFilterValue as unknown as T);

  /* ---------- 1. 主动操作了「全部」 ---------- */
  if (prevHasAll !== nextHasAll) {
    // 点亮/熄灭「全部」→ 所有子项强制同步成与它相同的状态
    return nextHasAll
      ? [allFilterValue as unknown as T, ...REAL]
      : [];
  }

  /* ---------- 2. 只动了子项 ---------- */
  const nextReal = next.filter(v => v !== (allFilterValue as unknown as T));
  const allRealSelected = nextReal.length === REAL.length;

  // 子项全部一致 → 把「全部」同步成一致；否则「全部」熄灭
  return allRealSelected
    ? [allFilterValue as unknown as T, ...REAL]
    : nextReal;
};

const matchesTextFilter = (values: string[] | undefined, selected: string[], total: number) => {
  if (!selected.length || selected.includes(allFilterValue) || selected.length >= total) return true;
  if (!values || !values.length) return false;
  return values.some((value) => selected.includes(value));
};

const filteredResults = computed(() => {
  return results.value.filter((item) => {
    const monthAll =
      !monthFilter.value.length || monthFilter.value.includes(allFilterValue) ||
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

onMounted(async () => {
  await nextTick();
  const height = queryPanelRef.value?.getBoundingClientRect().height;
  if (height) {
    document.documentElement.style.setProperty("--query-panel-height", `${height}px`);
  }
  try {
    const saved = await invoke<TrackedItem[]>("list_tracked_subjects");
    syncStatusesFromTracked(saved);
  } catch (_) {
    statuses.value = {};
    trackedItems.value = [];
  }
});

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
  getFilterLabel(
    typeFilter.value.filter((v) => v !== allFilterValue).length,
    OFFICIAL_TYPE_OPTIONS.length
  )
);
const regionFilterLabel = computed(() =>
  getFilterLabel(
    regionFilter.value.filter((v) => v !== allFilterValue).length,
    OFFICIAL_REGION_OPTIONS.length
  )
);
const audienceFilterLabel = computed(() =>
  getFilterLabel(
    audienceFilter.value.filter((v) => v !== allFilterValue).length,
    OFFICIAL_AUDIENCE_OPTIONS.length
  )
);

const handleMonthFilterChange = (value: Array<number | string>) => {
  const prev = monthFilter.value;
  const next = applySelectAllBehavior(value, monthFilterOptions.value, prev);
  monthFilter.value = next;
  selected.value = null; // 筛选变动后清空详情
};

const handleTypeFilterChange = (value: string[]) => {
  const prev = typeFilter.value;
  const next = applySelectAllBehavior(value, typeOptions.value, prev);
  typeFilter.value = next;
  selected.value = null; // 筛选变动后清空详情
};

const handleRegionFilterChange = (value: string[]) => {
  const prev = regionFilter.value;
  const next = applySelectAllBehavior(value, regionOptions.value, prev);
  regionFilter.value = next;
  selected.value = null; // 筛选变动后清空详情
};

const handleAudienceFilterChange = (value: string[]) => {
  const prev = audienceFilter.value;
  const next = applySelectAllBehavior(value, audienceOptions.value, prev);
  audienceFilter.value = next;
  selected.value = null; // 筛选变动后清空详情
};

const renderFilterTag = (label: string) =>
  h(NTag, { size: "small", type: "info" }, { default: () => label });

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

const MIN_PROGRESS_DURATION_MS = 100_000; // 至少 100 秒匀速到 99%
const CATCHUP_DURATION_MS = 5_000; // 提前完成时，用 5 秒冲到 100%
const MAX_PROGRESS_BEFORE_FINISH = 99;

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

const normalizeStatus = (state?: ItemStatus): ItemStatus => {
  if (!state) return { watching: false, backlog: false, watched: false };
  const keys: Array<keyof ItemStatus> = ["watched", "watching", "backlog"]; // 优先级：已看 > 追番 > 补番
  const active = keys.find((key) => state[key]);
  return {
    watching: active === "watching",
    backlog: active === "backlog",
    watched: active === "watched",
  };
};

const ensureStatus = (id: number): ItemStatus => {
  if (!statuses.value[id]) {
    statuses.value[id] = { watching: false, backlog: false, watched: false };
  } else {
    statuses.value[id] = normalizeStatus(statuses.value[id]);
  }
  return statuses.value[id];
};

const syncStatusesFromTracked = (items: TrackedItem[]) => {
  const map: Record<number, ItemStatus> = {};
  items.forEach((item) => {
    map[item.id] = normalizeStatus({
      watching: !!item.watching,
      backlog: !!item.backlog,
      watched: !!item.watched,
    });
  });
  statuses.value = map;
  trackedItems.value = items;
};

const persistStatusToDb = async (item: MonthAnime, nextStatus: ItemStatus) => {
  if (!item.id) return;
  const payload = {
    id: item.id,
    name: item.name,
    nameCn: item.nameCn || "",
    image: item.image,
    url: item.url || `https://bgm.tv/subject/${item.id}`,
    date: item.date || "",
    rating: item.rating ?? null,
    summary: item.summary || "",
    watching: nextStatus.watching,
    backlog: nextStatus.backlog,
    watched: nextStatus.watched,
  };
  const saved = await invoke<TrackedItem[]>("save_tracked_subject", { subject: payload });
  syncStatusesFromTracked(saved);
};
const setExclusiveStatus = async (item: MonthAnime, target: StatusKey) => {
  if (!item.id) return;
  const base: ItemStatus = { watching: false, backlog: false, watched: false };
  if (target && base.hasOwnProperty(target)) {
    (base as any)[target] = true;
  }
  await persistStatusToDb(item, base);
};

const cancelActiveQuery = () => {
  // 失效当前查询 token，后续异步结果将被丢弃
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
  // 匀速推进到 99%，用时间计算避免累加误差
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
    // 到达 99% 后维持在 99%，不用继续跑定时器
    if (target >= MAX_PROGRESS_BEFORE_FINISH) {
      clearProgressTimers();
      progressTimer = window.setInterval(() => {
        progress.value = MAX_PROGRESS_BEFORE_FINISH; // 维持 99%，直到查询完成
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

  // 还没到 100s，启动 5s 的冲刺动画
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

const normalizeText = (value: string) =>
  value
    .toLowerCase()
    .replace(/[^a-z0-9\s]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();

const splitAliases = (text: string) => {
  if (!text) return [];
  return text
    .split(/[、,，/\|]+/)
    .map((value) => value.trim())
    .filter(Boolean);
};

const buildSearchKeywords = (item: MonthAnime | null) => {
  if (!item) return [];
  const raw = [item.name, item.nameCn, ...splitAliases(item.alias || "")].filter(
    (value): value is string => Boolean(value)
  );
  const seen = new Set<string>();
  return raw
    .map((value) => normalizeText(value))
    .filter((value) => value.length > 1 && !seen.has(value) && (seen.add(value), true));
};

const parseQueryKeywords = (value: string) => {
  if (!value) return [];
  return value
    .split(/[|/、,，]+/)
    .map((part) => normalizeText(part))
    .filter((part) => part.length > 1);
};

const handleSelect = (item: MonthAnime) => {
  selected.value = item;
  if (item.id) ensureStatus(item.id);
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
    const payload = await invoke<{ id: number; airedCount?: number | null }>("get_subject_aired_count", {
      id: item.id,
    });
    item.airedCount = payload.airedCount ?? 0;
  } catch (error) {
    airedError.value = String(error);
    item.airedCount = 0;
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

const selectedStatus = computed(() => {
  const id = selected.value?.id;
  if (!id) return { watching: false, backlog: false, watched: false };
  return ensureStatus(id);
});

const currentStatusKey = (status: ItemStatus): StatusKey => {
  if (status.watched) return "watched";
  if (status.watching) return "watching";
  if (status.backlog) return "backlog";
  return null;
};

const labelForAction = (active: StatusKey, target: StatusKey) => {
  if (target === "watching") return active === "watching" ? "正在追番" : active ? "转为正在追番" : "加入正在追番";
  if (target === "backlog") return active === "backlog" ? "补番计划" : active ? "转为补番计划" : "加入补番计划";
  if (target === "watched") return active === "watched" ? "已完番剧" : active ? "转为已完番剧" : "标记已完番剧";
  return "";
};

const updateTrackedItem = (id: number, patch: Partial<TrackedItem>) => {
  trackedItems.value = trackedItems.value.map((item) =>
    item.id === id ? { ...item, ...patch } : item
  );
};

const refreshWatchingDetails = async () => {
  if (!watchingList.value.length) return;
  for (const item of watchingList.value) {
    try {
      const payload = await invoke<{
        id: number;
        name: string;
        nameCn: string;
        image: string;
        date: string;
        rating: number | null;
        summary: string;
        url: string;
      }>("get_subject_brief", { id: item.id });
      updateTrackedItem(item.id, {
        name: payload.name,
        nameCn: payload.nameCn,
        image: payload.image,
        date: payload.date,
        rating: payload.rating,
        summary: payload.summary,
        url: payload.url,
      });
    } catch (_) {
      // 静默失败，保持本地快照
    }
  }
};

watch(isWatchingPage, (active) => {
  if (active) {
    void refreshWatchingDetails();
  }
});
</script>

<template>
  <NConfigProvider :theme="theme">
    <div class="app-shell">
      <header class="app-titlebar" data-tauri-drag-region>
        <div class="titlebar-drag-region" data-tauri-drag-region>
          <div class="titlebar-left" data-tauri-drag-region>
            <span class="app-title">HanamiRIP</span>
            <span class="app-subtitle">番剧助手</span>
          </div>
          <div class="titlebar-nav" data-tauri-drag-region>
            <NButton
              secondary
              :type="isQueryPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('query')"
            >
              季度查询
            </NButton>
            <NButton
              secondary
              :type="isWatchingPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('watching')"
            >
              正在追番
            </NButton>
            <NButton
              secondary
              :type="isBacklogPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('backlog')"
            >
              补番计划
            </NButton>
            <NButton
              secondary
              :type="isFinishedPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('finished')"
            >
              已完番剧
            </NButton>
            <NButton
              secondary
              :type="isSearchPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('search')"
            >
              搜索资源
            </NButton>
          </div>
        </div>
        <div class="titlebar-actions" aria-label="window actions" data-tauri-drag-region="false">
          <button
            class="titlebar-control"
            type="button"
            aria-label="最小化"
            data-tauri-drag-region="false"
            @click="handleMinimize"
          >
            <span class="titlebar-icon">−</span>
          </button>
          <button
            class="titlebar-control titlebar-close"
            type="button"
            aria-label="关闭"
            data-tauri-drag-region="false"
            @click="handleClose"
          >
            <span class="titlebar-icon">×</span>
          </button>
        </div>
      </header>

      <div v-if="isQueryPage" class="app-body" :class="{ 'results-view': hasQueried }">
        <section class="query-summary-row">
          <section ref="queryPanelRef" class="query-panel">
            <NCard title="选择查询条件" size="small">
              <NSpace align="center" size="medium" class="query-actions">
                <NSelect v-model:value="year" :options="yearOptions" placeholder="选择年份" />
                <NSelect v-model:value="month" :options="seasonOptions" placeholder="选择季度" />
                <NButton type="primary" :loading="loading" @click="handleQuery">
                  开始查询季度番剧
                </NButton>
              </NSpace>
            </NCard>
          </section>
          <section v-if="showResults" class="summary-panel">
            <NCard size="small" class="result-summary">
              <NSpace vertical size="small">
                <span>来源：{{ resultUrl }}</span>
                <span>季度：{{ year }}年{{ seasonLabelFromMonth(month) }}</span>
                <span>条目数量：{{ resultCount }}</span>
              </NSpace>
            </NCard>
          </section>
          <section v-if="showResults" class="filter-panel">
            <NCard size="small">
              <div class="filter-grid">
                <div class="filter-field">
                  <span class="filter-label">月份</span>
                  <div class="filter-select">
                    <NSelect
                      :value="monthFilter"
                      :options="monthFilterOptions"
                      multiple
                      placeholder="选择月份"
                      :disabled="filterLoading"
                      :max-tag-count="0"
                      :max-tag-placeholder="() => ''"
                      :render-tag="() => null"
                      @update:value="handleMonthFilterChange"
                    />
                    <div class="filter-select-chip">
                      <NTag size="small" type="info">{{ monthFilterLabel }}</NTag>
                    </div>
                  </div>
                </div>
                <div class="filter-field">
                  <span class="filter-label">类型</span>
                  <div class="filter-select">
                    <NSelect
                      :value="typeFilter"
                      :options="typeOptions"
                      multiple
                      placeholder="选择类型"
                      :disabled="filterLoading"
                      :max-tag-count="0"
                      :max-tag-placeholder="() => ''"
                      :render-tag="() => null"
                      @update:value="handleTypeFilterChange"
                    />
                    <div class="filter-select-chip">
                      <NTag size="small" type="info">{{ typeFilterLabel }}</NTag>
                    </div>
                  </div>
                </div>
                <div class="filter-field">
                  <span class="filter-label">地区</span>
                  <div class="filter-select">
                    <NSelect
                      :value="regionFilter"
                      :options="regionOptions"
                      multiple
                      placeholder="选择地区"
                      :disabled="filterLoading"
                      :max-tag-count="0"
                      :max-tag-placeholder="() => ''"
                      :render-tag="() => null"
                      @update:value="handleRegionFilterChange"
                    />
                    <div class="filter-select-chip">
                      <NTag size="small" type="info">{{ regionFilterLabel }}</NTag>
                    </div>
                  </div>
                </div>
                <div class="filter-field">
                  <span class="filter-label">受众</span>
                  <div class="filter-select">
                    <NSelect
                      :value="audienceFilter"
                      :options="audienceOptions"
                      multiple
                      placeholder="选择受众"
                      :disabled="filterLoading"
                      :max-tag-count="0"
                      :max-tag-placeholder="() => ''"
                      :render-tag="() => null"
                      @update:value="handleAudienceFilterChange"
                    />
                    <div class="filter-select-chip">
                      <NTag size="small" type="info">{{ audienceFilterLabel }}</NTag>
                    </div>
                  </div>
                </div>
              </div>
              <span v-if="filterLoading" class="filter-loading">筛选信息加载中...</span>
            </NCard>
          </section>
        </section>

        <section v-if="hasQueried" class="progress-panel">
          <NProgress
            type="line"
            :percentage="progress"
            color="#18a058"
            :show-indicator="true"
          />
        </section>

        <section v-if="showResults" class="result-panel">
          <div class="result-content">
            <div class="result-layout" v-if="filteredResults.length">
              <NCard title="条目列表" size="small" class="result-list">
                <div class="result-list-grid">
                  <div
                    v-for="item in filteredResults"
                    :key="item.id ?? item.name"
                    class="result-list-item"
                    role="button"
                    tabindex="0"
                    @click="handleSelect(item)"
                    @keydown.enter.prevent="handleSelect(item)"
                    @keydown.space.prevent="handleSelect(item)"
                  >
                    <div class="result-list-thumb">
                      <img :src="item.image" :alt="item.name" />
                    </div>
                    <div class="result-list-info">
                      <div class="result-list-title">{{ item.nameCn || item.name }}</div>
                    </div>
                  </div>
                </div>
              </NCard>
              <NCard v-if="selected" title="条目详情" size="small" class="detail-panel">
                <div class="detail-panel-body">
                  <div class="detail-media">
                    <img class="detail-image" :src="selected.image" :alt="selected.name" />
                  </div>
                  <div class="detail-info">
                    <a
                      class="detail-title detail-title-link"
                      :href="selected.url"
                      target="_blank"
                      rel="noreferrer"
                    >
                      {{ selected.nameCn || selected.name }}
                    </a>
                    <div class="detail-actions">
                      <NButton
                        size="tiny"
                        :type="selectedStatus.watching ? 'primary' : 'default'"
                        secondary
                        @click="selected && setExclusiveStatus(selected, 'watching')"
                      >
                        {{ labelForAction(currentStatusKey(selectedStatus), 'watching') }}
                      </NButton>
                      <NButton
                        size="tiny"
                        :type="selectedStatus.backlog ? 'primary' : 'default'"
                        secondary
                        @click="selected && setExclusiveStatus(selected, 'backlog')"
                      >
                        {{ labelForAction(currentStatusKey(selectedStatus), 'backlog') }}
                      </NButton>
                      <NButton
                        size="tiny"
                        :type="selectedStatus.watched ? 'primary' : 'default'"
                        secondary
                        @click="selected && setExclusiveStatus(selected, 'watched')"
                      >
                        {{ labelForAction(currentStatusKey(selectedStatus), 'watched') }}
                      </NButton>
                    </div>
                    <div class="detail-info-list">
                      <div class="detail-info-row">
                        <span class="detail-label">原名</span>
                        <span class="detail-value">{{ selected.name }}</span>
                      </div>
                      <div class="detail-info-row">
                        <span class="detail-label">评分</span>
                        <span class="detail-value detail-rating">
                          <span class="detail-stars">{{ formatStars(selected.rating) }}</span>
                          <span class="detail-score">{{ formatRating(selected.rating) }}</span>
                        </span>
                      </div>
                      <div class="detail-info-row">
                        <span class="detail-label">放送</span>
                        <span class="detail-value">
                          {{ formatAirDate(selected.date) }}
                          <span class="detail-divider">·</span>
                          <template v-if="airedLoadingId === selected.id">已播出计算中...</template>
                          <template v-else-if="airedError && selected.airedCount == null">已播出获取失败</template>
                          <template v-else>已播出 {{ selected.airedCount ?? 0 }} 集</template>
                        </span>
                      </div>
                      <div class="detail-info-row">
                        <span class="detail-label">原作</span>
                        <span class="detail-value">
                          <template v-if="originLoadingId === selected.id">加载中...</template>
                          <template v-else-if="originError && !selected.origin">获取失败</template>
                          <template v-else>{{ selected.origin || "未知" }}</template>
                        </span>
                      </div>
                      <div class="detail-info-row">
                        <span class="detail-label">演员相关</span>
                        <span class="detail-value">
                          <NButton size="tiny" type="primary" secondary @click="handleStaffOpen">
                            查看工作人员
                          </NButton>
                        </span>
                      </div>
                      <div class="detail-info-row">
                        <span class="detail-label">角色</span>
                        <span class="detail-value">
                          <template v-if="characterLoadingId === selected.id">加载中...</template>
                          <template v-else-if="characterError && !selectedCharacters.length">获取失败</template>
                          <template v-else-if="!selectedCharacters.length">暂无</template>
                          <span v-else class="detail-link-list">
                            <a
                              v-for="item in selectedCharacters"
                              :key="item.id"
                              class="detail-link"
                              :href="item.url"
                              target="_blank"
                              rel="noreferrer"
                            >
                              {{ item.name }}
                            </a>
                          </span>
                        </span>
                      </div>
                      <div class="detail-info-row">
                        <span class="detail-label">简介</span>
                        <span class="detail-value detail-summary">
                          <template v-if="summaryLoadingId === selected.id">加载中...</template>
                          <template v-else-if="summaryError && !selected.summaryCn">获取失败：{{ summaryError }}</template>
                          <template v-else-if="!selected.summaryCn">暂无</template>
                          <template v-else>
                            {{ selected.summaryCn }}
                            <span v-if="selected.summaryTranslated" class="detail-translate-tag">
                              （翻译）
                            </span>
                            <span v-else-if="summaryError" class="detail-translate-tag">
                              （{{ summaryError }}）
                            </span>
                          </template>
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </NCard>
            </div>
          </div>
        </section>
      </div>

      <div v-else-if="isWatchingPage" class="app-body list-view">
        <NCard title="正在追番" size="small" class="watchlist-card">
          <div v-if="watchingByWeekday.length" class="watchlist-section-list">
            <div v-for="group in watchingByWeekday" :key="group.label" class="watchlist-section">
              <div class="watchlist-section-title">{{ group.label }}</div>
              <div class="watchlist-grid">
                <div v-for="item in group.items" :key="item.id" class="watchlist-item">
                  <div class="watchlist-thumb">
                    <img :src="item.image" :alt="item.name" />
                  </div>
                  <div class="watchlist-body">
                    <a class="watchlist-title" :href="item.url" target="_blank" rel="noreferrer">
                      {{ item.nameCn || item.name }}
                    </a>
                    <div class="watchlist-meta">
                      <span class="meta-row">放送：{{ formatAirDate(item.date) }}</span>
                      <span class="meta-row">评分：{{ formatRating(item.rating) }}</span>
                    </div>
                    <div class="watchlist-actions">
                      <NButton size="tiny" type="primary" secondary disabled>
                        正在追番
                      </NButton>
                      <NButton size="tiny" secondary @click="setExclusiveStatus(item, 'backlog')">
                        转为补番计划
                      </NButton>
                      <NButton size="tiny" secondary @click="setExclusiveStatus(item, 'watched')">
                        转为已完番剧
                      </NButton>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <p v-else class="watchlist-empty">还没有正在追的番剧。</p>
        </NCard>
      </div>

      <div v-else-if="isBacklogPage" class="app-body list-view">
        <NCard title="补番计划" size="small" class="watchlist-card">
          <div v-if="backlogList.length" class="watchlist-grid">
            <div v-for="item in backlogList" :key="item.id" class="watchlist-item">
              <div class="watchlist-thumb">
                <img :src="item.image" :alt="item.name" />
              </div>
              <div class="watchlist-body">
                <a class="watchlist-title" :href="item.url" target="_blank" rel="noreferrer">
                  {{ item.nameCn || item.name }}
                </a>
                <div class="watchlist-meta">
                  <span class="meta-row">放送：{{ formatAirDate(item.date) }}</span>
                  <span class="meta-row">评分：{{ formatRating(item.rating) }}</span>
                </div>
                <div class="watchlist-actions">
                  <NButton size="tiny" secondary disabled>
                    补番计划
                  </NButton>
                  <NButton size="tiny" secondary disabled>
                    转为正在追番
                  </NButton>
                  <NButton size="tiny" type="primary" secondary @click="setExclusiveStatus(item, 'watched')">
                    转为已完番剧
                  </NButton>
                </div>
              </div>
            </div>
          </div>
          <p v-else class="watchlist-empty">补番计划为空。</p>
        </NCard>
      </div>

      <div v-else-if="isFinishedPage" class="app-body list-view">
        <NCard title="已完番剧" size="small" class="watchlist-card">
          <div v-if="finishedList.length" class="watchlist-grid">
            <div v-for="item in finishedList" :key="item.id" class="watchlist-item">
              <div class="watchlist-thumb">
                <img :src="item.image" :alt="item.name" />
              </div>
              <div class="watchlist-body">
                <a class="watchlist-title" :href="item.url" target="_blank" rel="noreferrer">
                  {{ item.nameCn || item.name }}
                </a>
                <div class="watchlist-meta">
                  <span class="meta-row">放送：{{ formatAirDate(item.date) }}</span>
                  <span class="meta-row">评分：{{ formatRating(item.rating) }}</span>
                </div>
                <div class="watchlist-actions">
                  <NButton size="tiny" secondary disabled>
                    转为正在追番
                  </NButton>
                  <NButton size="tiny" secondary disabled>
                    转为补番计划
                  </NButton>
                  <NButton size="tiny" type="primary" secondary @click="setExclusiveStatus(item, null)">
                    变为未观看
                  </NButton>
                </div>
              </div>
            </div>
          </div>
          <p v-else class="watchlist-empty">还没有标记已看的番剧。</p>
        </NCard>
      </div>

      <div v-else class="app-body search-view">
        <NCard title="搜索资源" size="small" class="search-placeholder">
          <p>搜索资源页面正在建设中，敬请期待。</p>
        </NCard>
      </div>
    </div>
    <NModal v-model:show="showStaffModal" preset="card" title="工作人员" size="small">
      <div class="staff-modal-body">
        <p class="staff-modal-title">当前条目：{{ selected?.nameCn || selected?.name || "" }}</p>
        <p v-if="staffLoadingId === selected?.id">正在加载工作人员信息...</p>
        <p v-else-if="staffError">{{ staffError }}</p>
        <div v-else>
          <p v-if="!selectedStaff.length">暂无工作人员信息。</p>
          <div v-else class="staff-group-list">
            <div v-for="group in selectedStaff" :key="group.role" class="staff-group">
              <div class="staff-role">{{ group.role }}</div>
              <div class="staff-people">
                <a
                  v-for="person in group.people"
                  :key="person.id"
                  class="staff-link"
                  :href="person.url"
                  target="_blank"
                  rel="noreferrer"
                >
                  {{ person.name }}
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </NModal>
  </NConfigProvider>
</template>
