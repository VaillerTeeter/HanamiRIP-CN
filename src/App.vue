<script setup lang="ts">
import { computed, h, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  NButton,
  NCard,
  NCheckbox,
  NCheckboxGroup,
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
  totalCount?: number;
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

type PageKey = "query" | "watching" | "backlog" | "finished" | "search" | "download" | "tracks";
const activePage = ref<PageKey>("query");
const isQueryPage = computed(() => activePage.value === "query");
const isWatchingPage = computed(() => activePage.value === "watching");
const isBacklogPage = computed(() => activePage.value === "backlog");
const isFinishedPage = computed(() => activePage.value === "finished");
const isSearchPage = computed(() => activePage.value === "search");
const isDownloadPage = computed(() => activePage.value === "download");
const isTracksPage = computed(() => activePage.value === "tracks");
const switchPage = (page: PageKey) => {
  activePage.value = page;
};
const downloads = ref<DownloadItem[]>([]);
let downloadSeq = 1;

const appWindow = getCurrentWindow();
let downloadPoller: number | null = null;

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
const detailVisible = computed(() => showResults.value && !!selected.value);
const INACTIVITY_SCROLL_DELAY_MS = 5000;
let listMouseLeaveTimer: number | undefined;

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

  // 兜底：若未找到容器引用，退回浏览器默认行为
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
});
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

// --- 搜索资源 ---
type LogicOp = "and" | "or" | "not";
type SearchTerm = { value: string; op: LogicOp; source: "preset" | "custom" | "tracked" };
type SearchResult = {
  title: string;
  detailUrl?: string;
  magnet?: string;
  download?: string;
  size?: string;
  date?: string;
};
type DownloadItem = {
  id: number;
  title: string;
  link: string;
  kind: "magnet" | "torrent";
  startedAt: string;
  status: "started" | "failed" | "completed" | "paused";
  path?: string;
  tempPath?: string;
  finalPath?: string;
  finalized?: boolean;
  torrentId?: number;
  infoHash?: string;
  progressBytes?: number;
  totalBytes?: number;
  uploadedBytes?: number;
  state?: string;
  error?: string;
  downloadSpeed?: string;
  uploadSpeed?: string;
  timeRemaining?: string;
};
const NYAA_BASE = "https://nyaa.vaciller.top/?f=0&c=0_0&q=";
const presetPhrases = ["SubsPlease", "LoliHouse", "内封", "外挂", "480", "720", "1080"];
const logicOptions: { label: string; value: LogicOp }[] = [
  { label: "与", value: "and" },
  { label: "或", value: "or" },
  { label: "非", value: "not" },
];
const activeLogic = ref<LogicOp>("and");
const searchTerms = ref<SearchTerm[]>([]);
const customSearchInput = ref("");
const trackedSelection = ref<number | null>(null);
const searchLoading = ref(false);
const searchError = ref("");
const searchHtml = ref("");
const searchResults = ref<SearchResult[]>([]);
const aliasModalVisible = ref(false);
const aliasLoading = ref(false);
const aliasOptions = ref<string[]>([]);
const aliasSelected = ref<string[]>([]);
const pendingTrackedId = ref<number | null>(null);
const pendingTrackedName = ref("");

const trackedOptions = computed(() =>
  trackedItems.value.map((item) => ({
    label: item.nameCn || item.name,
    value: item.id,
  }))
);

const addSearchTerm = (value: string, source: SearchTerm["source"], op?: LogicOp) => {
  const trimmed = value.trim();
  if (!trimmed) return;
  searchTerms.value = [...searchTerms.value, { value: trimmed, op: op ?? activeLogic.value, source }];
};

const handleAddPreset = (value: string) => {
  addSearchTerm(value, "preset");
};

const handleAddCustom = () => {
  addSearchTerm(customSearchInput.value, "custom");
  customSearchInput.value = "";
};

const removeTrackedTerms = () => {
  searchTerms.value = searchTerms.value.filter((t) => t.source !== "tracked");
};

const handleSelectTracked = async (value: number | null) => {
  if (value == null) {
    trackedSelection.value = null;
    removeTrackedTerms();
    aliasModalVisible.value = false;
    return;
  }
  const target = trackedItems.value.find((item) => item.id === value);
  if (!target) {
    trackedSelection.value = null;
    removeTrackedTerms();
    aliasModalVisible.value = false;
    return;
  }

  pendingTrackedId.value = value;
  pendingTrackedName.value = target.nameCn || target.name || "";
  aliasModalVisible.value = true;
  aliasLoading.value = true;
  aliasOptions.value = [];
  aliasSelected.value = [];

  const aliasSet = new Set<string>();
  const baseName = (pendingTrackedName.value || "").trim();
  if (baseName) aliasSet.add(baseName);
  try {
    const payload = await invoke<{ id: number; aliases: string[] }>("get_subject_aliases", { id: value });
    (payload.aliases || []).forEach((a) => {
      const t = (a || "").trim();
      if (t && !aliasSet.has(t)) aliasSet.add(t);
    });
  } catch (_) {
    // 保留已有别名集合
  } finally {
    const list = Array.from(aliasSet);
    const base = baseName || "";
    const ordered = base ? [base, ...list.filter((v) => v !== base)] : list;
    aliasOptions.value = ordered;
    aliasSelected.value = [];
    aliasLoading.value = false;
  }
};

const cancelAliasSelection = () => {
  aliasModalVisible.value = false;
  aliasLoading.value = false;
  pendingTrackedId.value = null;
  pendingTrackedName.value = "";
  aliasOptions.value = [];
  aliasSelected.value = [];
  // 清空下拉选中，避免误认为已加入
  trackedSelection.value = null;
};

const confirmAliasSelection = () => {
  if (!pendingTrackedId.value) {
    cancelAliasSelection();
    return;
  }
  if (!aliasSelected.value.length) {
    // 没选名称则不添加
    cancelAliasSelection();
    return;
  }
  removeTrackedTerms();
  trackedSelection.value = pendingTrackedId.value;
  const seen = new Set<string>();
  aliasSelected.value.forEach((term) => {
    const t = term.trim();
    if (t && !seen.has(t)) {
      seen.add(t);
      addSearchTerm(t, "tracked", "and");
    }
  });
  cancelAliasSelection();
};


const removeSearchTerm = (index: number) => {
  searchTerms.value = searchTerms.value.filter((_, i) => i !== index);
};

const queryParts = computed(() => {
  const andTerms = searchTerms.value.filter((t) => t.op === "and").map((t) => `"${t.value}"`);
  const notTerms = searchTerms.value.filter((t) => t.op === "not").map((t) => `-"${t.value}"`);
  const orTermsRaw = searchTerms.value.filter((t) => t.op === "or").map((t) => `"${t.value}"`);
  // 用管道符将 OR 词汇连接，不包裹括号
  const orTerm = orTermsRaw.length ? orTermsRaw.join("|") : "";
  const parts = [...andTerms, ...(orTerm ? [orTerm] : []), ...notTerms];
  return parts.filter(Boolean);
});

const searchQuery = computed(() => queryParts.value.join(" "));
const searchUrl = computed(() => `${NYAA_BASE}${encodeURIComponent(searchQuery.value)}`);

const injectBaseTag = (html: string, url: string) => {
  const baseHref = url.split("?")[0];
  const baseTag = `<base href="${baseHref}" />`;
  const headMatch = html.match(/<head[^>]*>/i);
  if (headMatch && headMatch.index !== undefined) {
    const idx = headMatch.index + headMatch[0].length;
    return `${html.slice(0, idx)}${baseTag}${html.slice(idx)}`;
  }
  return `<head>${baseTag}</head>${html}`;
};

const parseSearchResults = (html: string): SearchResult[] => {
  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    const rows = Array.from(doc.querySelectorAll("table tbody tr"));
    const toAbs = (href?: string | null) => {
      if (!href) return undefined;
      try {
        return new URL(href, searchUrl.value).toString();
      } catch (_) {
        return href || undefined;
      }
    };

    return rows
      .map((tr) => {
        const nameCell = tr.querySelector("td:nth-child(2)");
        const anchorCandidates = nameCell
          ? Array.from(nameCell.querySelectorAll("a"))
          : [];
        const nameLink = anchorCandidates
          .filter((a) => a.getAttribute("href")?.includes("/view/"))
          .pop();
        if (!nameLink) return null;
        const title = nameLink.textContent?.trim() || "";
        const detailUrl = toAbs(nameLink.getAttribute("href"));
        const magnet = toAbs(tr.querySelector("a[href^='magnet:']")?.getAttribute("href"));
        const download = toAbs(tr.querySelector("a[href$='.torrent']")?.getAttribute("href"));
        const size = tr.querySelector("td:nth-child(5)")?.textContent?.trim() || undefined;
        const date = tr.querySelector("td:nth-child(6)")?.textContent?.trim() || undefined;
        if (!title) return null;
        return { title, detailUrl, magnet, download, size, date } as SearchResult;
      })
      .filter((v): v is SearchResult => Boolean(v));
  } catch (e) {
    console.error("parseSearchResults error", e);
    return [];
  }
};

const openSearch = async () => {
  if (!searchQuery.value) return;
  searchLoading.value = true;
  searchError.value = "";
  searchResults.value = [];
  searchHtml.value = "";
  try {
    const html = await invoke<string>("fetch_search_html", { url: searchUrl.value });
    searchHtml.value = injectBaseTag(html, searchUrl.value);
    searchResults.value = parseSearchResults(html);
  } catch (err: any) {
    searchError.value = typeof err === "string" ? err : err?.message || "获取搜索结果失败";
  } finally {
    searchLoading.value = false;
  }
};
const clearSearchResults = () => {
  searchResults.value = [];
  searchHtml.value = "";
  searchError.value = "";
};

const addDownload = (
  title: string,
  link: string,
  kind: DownloadItem["kind"],
  path?: string,
  payload?: Partial<DownloadItem>
) => {
  const startedAt = new Date().toISOString();
  downloads.value = [
    {
      id: downloadSeq++,
      title,
      link,
      kind,
      startedAt,
      status: "started",
      path,
      ...payload,
    },
    ...downloads.value,
  ];
};

const formatBytes = (value?: number) => {
  if (value == null || Number.isNaN(value)) return "-";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = value;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
};

const parseSpeedToBps = (value?: string) => {
  if (!value) return 0;
  const match = value.match(/([\d.]+)\s*([a-zA-Z/]+)?/);
  if (!match) return 0;
  const num = Number.parseFloat(match[1]);
  if (!Number.isFinite(num)) return 0;
  const unitRaw = (match[2] || "B").replace(/\s*/g, "").replace(/\/s/i, "").toUpperCase();
  const unit = unitRaw.endsWith("/S") ? unitRaw.slice(0, -2) : unitRaw;
  const factorMap: Record<string, number> = {
    B: 1,
    KB: 1024,
    KIB: 1024,
    MB: 1024 ** 2,
    MIB: 1024 ** 2,
    GB: 1024 ** 3,
    GIB: 1024 ** 3,
    TB: 1024 ** 4,
    TIB: 1024 ** 4,
  };
  const factor = factorMap[unit] ?? 1;
  return num * factor;
};

const formatSpeed = (bps: number) => {
  if (!Number.isFinite(bps) || bps <= 0) return "0 B/s";
  const units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
  let size = bps;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
};

const formatDownloadStatus = (status: DownloadItem["status"]) => {
  switch (status) {
    case "completed":
      return "已完成";
    case "failed":
      return "失败";
    case "paused":
      return "已暂停";
    default:
      return "下载中";
  }
};

const downloadDisplayTitle = (item: DownloadItem) => {
  if (item.status === "completed") return item.title;
  if (item.status === "paused") return `${item.title}.paused`;
  if (item.status === "failed") return `${item.title}.failed`;
  return `${item.title}.downloading`;
};

type TrackType = "video" | "audio" | "subtitle";
type TrackItem = {
  id: number;
  name: string;
  path: string;
  fileSize?: string;
};
type TrackInfo = {
  trackId: string;
  codec: string;
  lang?: string;
  languageName?: string;
  trackName?: string;
  isDefault?: boolean;
  isForced?: boolean;
  charset?: string;
  attributes?: string;
  container?: string;
  fileSize?: string;
  selected?: boolean;
  langOverride?: string;
};
type TrackFileResult = {
  file: TrackItem;
  tracks: TrackInfo[];
};
type MixTrackInput = {
  path: string;
  kind: TrackType;
  trackIds: string[];
  trackLangs?: Record<string, string>;
};
type MixQueueStatus = "queued" | "running" | "success" | "failed";
type MixQueueItem = {
  id: number;
  createdAt: string;
  outputPath: string;
  inputs: MixTrackInput[];
  status: MixQueueStatus;
  message?: string;
};

const trackFiles = ref<Record<TrackType, TrackItem[]>>({
  video: [],
  audio: [],
  subtitle: [],
});
const trackInfos = ref<Record<TrackType, TrackFileResult[]>>({
  video: [],
  audio: [],
  subtitle: [],
});
const trackLoading = ref<Record<TrackType, boolean>>({
  video: false,
  audio: false,
  subtitle: false,
});
const trackProgress = ref<Record<TrackType, number>>({
  video: 0,
  audio: 0,
  subtitle: 0,
});
const trackErrors = ref<Record<TrackType, string>>({
  video: "",
  audio: "",
  subtitle: "",
});
const trackMixLoading = ref(false);
const trackMixError = ref("");
const trackMixResult = ref("");
const mixQueue = ref<MixQueueItem[]>([]);
const mixQueueRunning = ref(false);
const mixQueueDetailVisible = ref(false);
const selectedMixTask = ref<MixQueueItem | null>(null);
let trackSeq = 1;
let mixQueueSeq = 1;

const trackLabelMap: Record<TrackType, string> = {
  video: "视频",
  audio: "音频",
  subtitle: "字幕",
};

const trackLangDefaults = reactive<Record<TrackType, string>>({
  video: "ja",
  audio: "ja",
  subtitle: "zh-Hans",
});

const trackLanguageOptions = [
  { label: "自动", value: "" },
  { label: "日语 (ja)", value: "ja" },
  { label: "英语 (en)", value: "en" },
  { label: "简体中文 (zh-Hans)", value: "zh-Hans" },
  { label: "繁体中文 (zh-Hant)", value: "zh-Hant" },
  { label: "中文 (zh)", value: "zh" },
  { label: "韩语 (ko)", value: "ko" },
  { label: "法语 (fr)", value: "fr" },
  { label: "德语 (de)", value: "de" },
  { label: "西班牙语 (es)", value: "es" },
  { label: "未知 (und)", value: "und" },
];

const addTrackFile = async (type: TrackType) => {
  try {
    const videoExt = ["mkv", "mp4", "avi", "mov", "ts", "m2ts", "webm", "mpg", "mpeg"];
    const subtitleExt = ["srt", "ass", "ssa", "vtt", "sup", "sub"];
    const extMap: Record<TrackType, string[]> = {
      video: videoExt,
      audio: videoExt,
      subtitle: [...videoExt, ...subtitleExt],
    };
    const result = await openDialog({
      title: `选择${trackLabelMap[type]}文件`,
      directory: false,
      multiple: false,
      filters: [{ name: `${trackLabelMap[type]}文件`, extensions: extMap[type] }],
    });
    if (!result || Array.isArray(result)) return;
    const file = String(result);
    let fileSize: string | undefined;
    try {
      const size = await invoke<string | null>("get_media_file_size", { path: file });
      fileSize = size ?? undefined;
    } catch (err) {
      console.error("get_media_file_size failed", err);
      fileSize = undefined;
    }
    trackFiles.value[type] = [
      {
        id: trackSeq++,
        name: file.split(/[\\/]/).filter(Boolean).pop() || file,
        path: file,
        fileSize,
      },
    ];
    trackInfos.value[type] = [];
    trackErrors.value[type] = "";
    trackProgress.value[type] = 0;
  } catch (err) {
    console.error("openDialog failed", err);
  }
};

const detectTracks = async (type: TrackType) => {
  if (trackLoading.value[type]) return;
  if (!trackFiles.value[type].length) {
    trackErrors.value[type] = `请先添加${trackLabelMap[type]}文件`;
    trackInfos.value[type] = [];
    trackProgress.value[type] = 0;
    return;
  }
  trackLoading.value[type] = true;
  trackProgress.value[type] = 0;
  trackInfos.value[type] = [];
  trackErrors.value[type] = "";

  const files = trackFiles.value[type];
  const totalFiles = files.length;
  const results: TrackFileResult[] = [];
  for (let i = 0; i < files.length; i += 1) {
    const file = files[i];
    try {
      const response = await invoke<{ tracks: TrackInfo[] }>("parse_media_tracks", {
        path: file.path,
        kind: type,
      });
      const tracks = (response.tracks || []).map((track) => ({
        ...track,
        selected: track.selected ?? true,
        langOverride: track.lang || trackLangDefaults[type],
      }));
      results.push({ file, tracks });
    } catch (err: any) {
      trackErrors.value[type] = typeof err === "string" ? err : err?.message || "解析失败";
      results.push({ file, tracks: [] });
    } finally {
      trackProgress.value[type] = Math.round(((i + 1) / totalFiles) * 100);
    }
  }
  trackInfos.value[type] = results;
  trackLoading.value[type] = false;
};

const pickOutputPath = async (baseFile?: TrackItem) => {
  const baseName = baseFile?.name ? baseFile.name.replace(/\.[^/.\\]+$/, "") : "mixed";
  const dir = baseFile?.path ? baseFile.path.replace(/[\\/][^\\/]+$/, "") : "";
  const defaultPath = dir ? `${dir}\\${baseName}_mixed.mkv` : `${baseName}_mixed.mkv`;
  const result = await saveDialog({
    title: "保存混合后的视频",
    defaultPath,
    filters: [{ name: "MKV", extensions: ["mkv"] }],
  });
  if (!result) return null;
  return result.endsWith(".mkv") ? result : `${result}.mkv`;
};

const collectMixInput = (type: TrackType): MixTrackInput | null => {
  const files = trackFiles.value[type];
  if (!files.length) return null;
  const file = files[0];
  const group = trackInfos.value[type].find((item) => item.file.id === file.id);
  if (!group || !group.tracks.length) return null;
  const selected = group.tracks.filter((track) => track.selected !== false).map((track) => track.trackId);
  if (!selected.length) return null;
  const trackLangs: Record<string, string> = {};
  group.tracks.forEach((track) => {
    if (track.selected === false) return;
    const lang = trackLangDefaults[type];
    if (lang) {
      trackLangs[track.trackId] = lang;
    }
  });
  return { path: file.path, kind: type, trackIds: selected, trackLangs };
};

const enqueueMixTask = async () => {
  if (trackMixLoading.value) return;
  trackMixError.value = "";
  trackMixResult.value = "";

  const videoInput = collectMixInput("video");
  const audioInput = collectMixInput("audio");
  const subtitleInput = collectMixInput("subtitle");

  if (!videoInput) {
    trackMixError.value = "请先检测并选择至少一个视频轨道";
    return;
  }

  trackMixLoading.value = true;
  try {
    const outputPath = await pickOutputPath(trackFiles.value.video[0]);
    if (!outputPath) return;

    const inputs: MixTrackInput[] = [videoInput];
    if (audioInput) inputs.push(audioInput);
    if (subtitleInput) inputs.push(subtitleInput);

    mixQueue.value.push({
      id: mixQueueSeq++,
      createdAt: new Date().toLocaleString(),
      outputPath,
      inputs,
      status: "queued",
    });
    trackFiles.value = { video: [], audio: [], subtitle: [] };
    trackInfos.value = { video: [], audio: [], subtitle: [] };
    trackErrors.value = { video: "", audio: "", subtitle: "" };
    trackProgress.value = { video: 0, audio: 0, subtitle: 0 };
    trackMixResult.value = "已添加到混流任务队列";
  } catch (err: any) {
    trackMixError.value = typeof err === "string" ? err : err?.message || "添加任务失败";
  } finally {
    trackMixLoading.value = false;
  }
};

const startMixQueue = async () => {
  if (mixQueueRunning.value) return;
  trackMixError.value = "";
  trackMixResult.value = "";
  const pending = mixQueue.value.some((item) => item.status === "queued");
  if (!pending) {
    trackMixResult.value = "当前没有待处理的混流任务";
    return;
  }

  mixQueueRunning.value = true;
  for (const item of mixQueue.value) {
    if (item.status !== "queued") continue;
    item.status = "running";
    item.message = undefined;
    try {
      const output = await invoke<string>("mix_media_tracks", {
        inputs: item.inputs.map((input) => ({
          path: input.path,
          kind: input.kind,
          trackIds: input.trackIds,
          trackLangs: input.trackLangs,
        })),
        outputPath: item.outputPath,
      });
      item.status = "success";
      item.message = output;
    } catch (err: any) {
      item.status = "failed";
      item.message = typeof err === "string" ? err : err?.message || "合成失败";
    }
  }
  mixQueueRunning.value = false;
};

const clearMixQueue = () => {
  if (mixQueueRunning.value) return;
  mixQueue.value = [];
  trackMixResult.value = "";
  trackMixError.value = "";
};

const openMixTaskDetail = (item: MixQueueItem) => {
  selectedMixTask.value = item;
  mixQueueDetailVisible.value = true;
};

const isDownloadPaused = (item: DownloadItem) => {
  const state = item.state?.toLowerCase() ?? "";
  return item.status === "paused" || state.includes("paused") || state.includes("stopped");
};

const isDownloadTerminal = (item: DownloadItem) => item.status === "completed" || item.status === "failed";

const handlePauseDownload = async (item: DownloadItem) => {
  if (item.torrentId == null || isDownloadTerminal(item) || isDownloadPaused(item)) return;
  try {
    await invoke("pause_torrent", { id: item.torrentId });
    item.status = "paused";
  } catch (err) {
    console.error("pause_torrent failed", err);
  }
};

const handleResumeDownload = async (item: DownloadItem) => {
  if (item.torrentId == null || !isDownloadPaused(item)) return;
  try {
    await invoke("resume_torrent", { id: item.torrentId });
    item.status = "started";
  } catch (err) {
    console.error("resume_torrent failed", err);
  }
};

const handleDeleteDownload = async (item: DownloadItem) => {
  if (item.torrentId != null) {
    try {
      await invoke("delete_torrent", { id: item.torrentId });
    } catch (err) {
      console.error("delete_torrent failed", err);
      return;
    }
  }
  downloads.value = downloads.value.filter((row) => row.id !== item.id);
};

const hasActiveDownloads = computed(() =>
  downloads.value.some((item) => item.torrentId != null && !isDownloadTerminal(item) && !isDownloadPaused(item))
);
const hasPausedDownloads = computed(() =>
  downloads.value.some((item) => item.torrentId != null && isDownloadPaused(item))
);
const totalDownloadBps = computed(() =>
  downloads.value.reduce((sum, item) => sum + parseSpeedToBps(item.downloadSpeed), 0)
);
const totalUploadBps = computed(() =>
  downloads.value.reduce((sum, item) => sum + parseSpeedToBps(item.uploadSpeed), 0)
);
const totalDownloadSpeedLabel = computed(() => formatSpeed(totalDownloadBps.value));
const totalUploadSpeedLabel = computed(() => formatSpeed(totalUploadBps.value));

const handlePauseAllDownloads = async () => {
  const active = downloads.value.filter((item) => item.status === "started" && item.torrentId != null);
  await Promise.all(active.map((item) => handlePauseDownload(item)));
};

const handleResumeAllDownloads = async () => {
  const paused = downloads.value.filter((item) => item.status === "paused" && item.torrentId != null);
  await Promise.all(paused.map((item) => handleResumeDownload(item)));
};

const openExternalLink = async (url?: string | null) => {
  if (!url) return;
  try {
    await invoke("open_external_link", { url });
  } catch (err) {
    console.error("openExternalLink failed", err);
    window.open(url, "_blank");
  }
};

const refreshDownloadStatuses = async () => {
  const active = downloads.value.filter((item) => item.torrentId != null);
  if (!active.length) return;

  const updates = await Promise.all(
    active.map(async (item) => {
      try {
        const stats = await invoke<{
          id: number;
          state: string;
          progressBytes: number;
          totalBytes: number;
          uploadedBytes: number;
          finished: boolean;
          error?: string | null;
          downloadSpeed?: string | null;
          uploadSpeed?: string | null;
          timeRemaining?: string | null;
        }>("get_torrent_status", { id: item.torrentId });
        return { id: item.id, stats };
      } catch (err) {
        console.error("get_torrent_status failed", err);
        return { id: item.id, stats: null };
      }
    })
  );

  let shouldFinalize = false;
  const updated = downloads.value.map((item) => {
    const update = updates.find((u) => u.id === item.id)?.stats;
    if (!update) return item;
    const completed = update.finished;
    const failed = Boolean(update.error);
    const stateLower = update.state?.toLowerCase() ?? "";
    const paused = stateLower.includes("paused") || stateLower.includes("stopped");
    const next: DownloadItem = {
      ...item,
      status: completed ? "completed" : failed ? "failed" : paused ? "paused" : "started",
      state: update.state,
      progressBytes: update.progressBytes,
      totalBytes: update.totalBytes,
      uploadedBytes: update.uploadedBytes,
      error: update.error ?? undefined,
      downloadSpeed: update.downloadSpeed ?? undefined,
      uploadSpeed: update.uploadSpeed ?? undefined,
      timeRemaining: update.timeRemaining ?? undefined,
    };
    if (completed && !item.finalized && item.tempPath && item.finalPath) {
      shouldFinalize = true;
    }
    return next;
  });
  downloads.value = updated;

  if (shouldFinalize) {
    await Promise.all(
      downloads.value.map(async (item) => {
        if (item.status !== "completed" || item.finalized || !item.tempPath || !item.finalPath) return;
        try {
          await invoke("finalize_torrent_download", {
            tempFolder: item.tempPath,
            finalFolder: item.finalPath,
          });
          item.finalized = true;
        } catch (err) {
          console.error("finalize_torrent_download failed", err);
        }
      })
    );
  }
};

const handleDownloadClick = async (item: SearchResult, kind: DownloadItem["kind"], link?: string) => {
  if (!link) return;

  let path: string | undefined;
  try {
    const result = await openDialog({
      title: kind === "magnet" ? "选择磁链下载目录" : "选择种子保存目录",
      directory: true,
      multiple: false,
    });
    if (!result || Array.isArray(result)) return; // 用户取消或选中了多个目录
    path = String(result);
  } catch (err) {
    console.error("openDialog failed", err);
    return;
  }

  try {
    const started = await invoke<{
      id: number;
      infoHash: string;
      name?: string | null;
      outputFolder: string;
      finalFolder: string;
    }>("start_torrent_download", { url: link, outputDir: path });
    addDownload(item.title, link, kind, path, {
      torrentId: started.id,
      infoHash: started.infoHash,
      tempPath: started.outputFolder,
      finalPath: started.finalFolder,
    });
  } catch (err) {
    console.error("start_torrent_download failed", err);
    addDownload(item.title, link, kind, path, { status: "failed" });
  }
};

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
  if (downloadPoller == null) {
    downloadPoller = window.setInterval(refreshDownloadStatuses, 1500);
  }
  try {
    const saved = await invoke<TrackedItem[]>("list_tracked_subjects");
    syncStatusesFromTracked(saved);
    // 启动时后台刷新正在追番的条目信息与集数，并落盘缓存
    void refreshWatchingDetails({ persist: true });
  } catch (_) {
    statuses.value = {};
    trackedItems.value = [];
  }
});

onBeforeUnmount(() => {
  if (downloadPoller != null) {
    window.clearInterval(downloadPoller);
    downloadPoller = null;
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
    airedCount: item.airedCount ?? 0,
    totalCount: item.totalCount ?? 0,
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

const CONCURRENT_REFRESH_LIMIT = 6;

const refreshWatchingDetails = async (options: { persist?: boolean } = {}) => {
  const queue = [...watchingList.value];
  if (!queue.length) return;
  const updated: TrackedItem[] = [];

  const worker = async () => {
    while (queue.length) {
      const item = queue.shift();
      if (!item) continue;
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

        const count = await invoke<{ id: number; airedCount?: number | null; totalCount?: number | null }>(
          "get_subject_aired_count",
          { id: item.id }
        );

        const patch: Partial<TrackedItem> = {
          name: payload.name,
          nameCn: payload.nameCn,
          image: payload.image,
          date: payload.date,
          rating: payload.rating,
          summary: payload.summary,
          url: payload.url,
          airedCount: count.airedCount ?? item.airedCount ?? 0,
          totalCount: count.totalCount ?? item.totalCount ?? 0,
        };

        updateTrackedItem(item.id, patch);
        const merged = { ...item, ...patch } as TrackedItem;
        updated.push(merged);
      } catch (_) {
        // 静默失败，保持本地快照
      }
    }
  };

  const concurrency = Math.min(CONCURRENT_REFRESH_LIMIT, queue.length);
  await Promise.all(Array.from({ length: concurrency }, () => worker()));

  if (options.persist && updated.length) {
    // 持久化最新的集数快照，防止下次启动重复等待
    const persistQueue = [...updated];
    const persistWorker = async () => {
      while (persistQueue.length) {
        const item = persistQueue.shift();
        if (!item) continue;
        try {
          await invoke<TrackedItem[]>("save_tracked_subject", { subject: item });
        } catch (_) {
          // 忽略写入失败，保持内存快照
        }
      }
    };
    await Promise.all(Array.from({ length: Math.min(3, persistQueue.length || 1) }, () => persistWorker()));
  }
};

watch(isWatchingPage, (active) => {
  if (active) {
    void refreshWatchingDetails();
  }
});

watch(activePage, (next, prev) => {
  if (next === "query" && prev && prev !== "query") {
    void scrollToSelectedItem();
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
            <NButton
              secondary
              :type="isDownloadPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('download')"
            >
              下载
            </NButton>
            <NButton
              secondary
              :type="isTracksPage ? 'primary' : 'default'"
              :data-tauri-drag-region="false"
              @click="switchPage('tracks')"
            >
              轨道工坊
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
                <div
                  ref="resultListRef"
                  class="result-list-grid"
                  @mouseenter="handleListMouseEnter"
                  @mouseleave="handleListMouseLeave"
                >
                  <div
                    v-for="item in filteredResults"
                    :key="item.id ?? item.name"
                    class="result-list-item"
                    role="button"
                    tabindex="0"
                    :ref="(el) => setListItemRef(item, el as HTMLElement | null)"
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
                      @click.prevent="openExternalLink(selected?.url)"
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
                              @click.prevent="openExternalLink(item.url)"
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
                    <a
                      class="watchlist-title"
                      :href="item.url"
                      target="_blank"
                      rel="noreferrer"
                      @click.prevent="openExternalLink(item.url)"
                    >
                      {{ item.nameCn || item.name }}
                    </a>
                    <div class="watchlist-meta">
                      <span class="meta-row">放送：{{ formatAirDate(item.date) }}</span>
                      <span class="meta-row">评分：{{ formatRating(item.rating) }}</span>
                    </div>
                    <div v-if="item.totalCount" class="episode-strip">
                      <div class="episode-strip-header">
                        <span class="episode-strip-label">章节列表</span>
                        <span class="episode-strip-summary">
                          共 {{ item.totalCount }} 集 · 已播 {{ item.airedCount ?? 0 }} 集
                        </span>
                      </div>
                      <div class="episode-strip-grid">
                        <span
                          v-for="n in item.totalCount"
                          :key="n"
                          class="episode-pill"
                          :class="{ active: (item.airedCount ?? 0) >= n }"
                        >
                          {{ String(n).padStart(2, '0') }}
                        </span>
                      </div>
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
                  <a
                    class="watchlist-title"
                    :href="item.url"
                    target="_blank"
                    rel="noreferrer"
                    @click.prevent="openExternalLink(item.url)"
                  >
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
                <a
                  class="watchlist-title"
                  :href="item.url"
                  target="_blank"
                  rel="noreferrer"
                  @click.prevent="openExternalLink(item.url)"
                >
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

      <div v-else-if="isDownloadPage" class="app-body download-view">
        <NCard title="下载" size="small" class="download-card">
          <div class="download-toolbar">
            <div v-if="downloads.length" class="download-total-speed">
              <span class="download-total-label">总速率</span>
              <span class="pill">↓ {{ totalDownloadSpeedLabel }}</span>
              <span class="pill">↑ {{ totalUploadSpeedLabel }}</span>
            </div>
            <div class="download-toolbar-actions">
              <NButton
                size="small"
                secondary
                class="icon-button"
                :disabled="!hasActiveDownloads"
                aria-label="全部暂停"
                title="全部暂停"
                @click="handlePauseAllDownloads"
              >
                <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                  <line x1="8" y1="6" x2="8" y2="18" />
                  <line x1="16" y1="6" x2="16" y2="18" />
                </svg>
              </NButton>
              <NButton
                size="small"
                secondary
                class="icon-button"
                :disabled="!hasPausedDownloads"
                aria-label="全部继续"
                title="全部继续"
                @click="handleResumeAllDownloads"
              >
                <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                  <polygon points="9 7 19 12 9 17" />
                </svg>
              </NButton>
            </div>
          </div>
          <div v-if="downloads.length" class="download-list">
            <div v-for="item in downloads" :key="item.id" class="download-row">
              <div class="download-main">
                <div class="download-title">{{ downloadDisplayTitle(item) }}</div>
                <div class="download-meta">
                  <span class="pill">{{ item.kind === 'magnet' ? '磁链' : '种子' }}</span>
                  <span class="pill">{{ formatDownloadStatus(item.status) }}</span>
                  <span v-if="item.state" class="pill">{{ item.state }}</span>
                  <span v-if="item.downloadSpeed" class="pill">↓ {{ item.downloadSpeed }}</span>
                  <span v-if="item.uploadSpeed" class="pill">↑ {{ item.uploadSpeed }}</span>
                  <span v-if="item.timeRemaining" class="pill">剩余 {{ item.timeRemaining }}</span>
                  <span class="pill">{{ item.startedAt }}</span>
                  <span v-if="item.path" class="pill path-pill" :title="item.path">{{ item.path }}</span>
                </div>
                <div v-if="item.totalBytes" class="download-progress">
                  <NProgress
                    type="line"
                    :percentage="Math.min(100, Math.round(((item.progressBytes ?? 0) / item.totalBytes) * 100))"
                    :show-indicator="false"
                    :height="8"
                  />
                  <div class="progress-text">
                    {{ formatBytes(item.progressBytes) }} / {{ formatBytes(item.totalBytes) }}
                  </div>
                </div>
                <div v-if="item.error" class="download-error">{{ item.error }}</div>
              </div>
              <div class="download-actions">
                <NButton
                  size="tiny"
                  text
                  class="icon-button"
                  :disabled="item.torrentId == null || isDownloadTerminal(item) || isDownloadPaused(item)"
                  aria-label="暂停"
                  title="暂停"
                  @click="handlePauseDownload(item)"
                >
                  <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                    <line x1="8" y1="6" x2="8" y2="18" />
                    <line x1="16" y1="6" x2="16" y2="18" />
                  </svg>
                </NButton>
                <NButton
                  size="tiny"
                  text
                  class="icon-button"
                  :disabled="item.torrentId == null || !isDownloadPaused(item)"
                  aria-label="继续"
                  title="继续"
                  @click="handleResumeDownload(item)"
                >
                  <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                    <polygon points="9 7 19 12 9 17" />
                  </svg>
                </NButton>
                <NButton
                  size="tiny"
                  text
                  type="error"
                  class="icon-button"
                  aria-label="删除"
                  title="删除"
                  @click="handleDeleteDownload(item)"
                >
                  <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M4 7h16" />
                    <path d="M9 7v12" />
                    <path d="M15 7v12" />
                    <path d="M6 7l1 13h10l1-13" />
                    <path d="M9 4h6l1 3H8l1-3Z" />
                  </svg>
                </NButton>
              </div>
            </div>
          </div>
          <p v-else class="download-empty">暂无下载记录。</p>
        </NCard>
      </div>

      <div v-else-if="isTracksPage" class="app-body download-view">
        <NCard title="轨道工坊" size="small" class="download-card">
          <div class="tracks-mix-bar">
            <NButton type="primary" size="small" :loading="trackMixLoading" @click="enqueueMixTask">
              添加到混流任务队列
            </NButton>
            <span v-if="trackMixResult" class="tracks-mix-success">{{ trackMixResult }}</span>
            <span v-if="trackMixError" class="tracks-mix-error">{{ trackMixError }}</span>
          </div>
          <div class="tracks-mix-lang">
            <span class="tracks-mix-lang-label">默认语言</span>
            <div class="tracks-mix-lang-item">
              <span>视频</span>
              <NSelect
                v-model:value="trackLangDefaults.video"
                size="small"
                :options="trackLanguageOptions"
                filterable
                tag
                placeholder="语言"
              />
            </div>
            <div class="tracks-mix-lang-item">
              <span>音频</span>
              <NSelect
                v-model:value="trackLangDefaults.audio"
                size="small"
                :options="trackLanguageOptions"
                filterable
                tag
                placeholder="语言"
              />
            </div>
            <div class="tracks-mix-lang-item">
              <span>字幕</span>
              <NSelect
                v-model:value="trackLangDefaults.subtitle"
                size="small"
                :options="trackLanguageOptions"
                filterable
                tag
                placeholder="语言"
              />
            </div>
          </div>
          <div class="tracks-list">
            <div class="tracks-section">
              <div class="tracks-header">
                <span class="tracks-title">视频</span>
                <div class="tracks-actions">
                  <NButton size="small" secondary @click="addTrackFile('video')">添加文件</NButton>
                  <NButton
                    size="small"
                    type="primary"
                    :loading="trackLoading.video"
                    :disabled="!trackFiles.video.length"
                    @click="detectTracks('video')"
                  >
                    检测
                  </NButton>
                </div>
              </div>
              <div class="tracks-body">
                <div class="tracks-files" v-if="trackFiles.video.length">
                  <div v-for="file in trackFiles.video" :key="file.id" class="tracks-file">
                    <span class="tracks-file-name">{{ file.name }}</span>
                    <span class="tracks-file-size">{{ file.fileSize || '-' }}</span>
                    <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
                  </div>
                </div>
                <p v-else class="download-empty">尚未添加视频文件。</p>
                <div v-if="trackErrors.video" class="tracks-error">{{ trackErrors.video }}</div>
                <div v-if="trackLoading.video" class="tracks-progress">
                  <NProgress type="line" :percentage="trackProgress.video" :show-indicator="true" :height="8" />
                </div>
                <div v-if="trackInfos.video.length" class="tracks-info">
                  <div v-for="group in trackInfos.video" :key="group.file.id" class="tracks-info-group">
                    <div class="tracks-info-file">{{ group.file.name }}</div>
                    <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                      <NCheckbox v-model:checked="info.selected" size="small" />
                      <span class="tracks-info-name">轨道 {{ info.trackId || '-' }}</span>
                      <span class="tracks-info-meta">编码 {{ info.codec || '-' }}</span>
                      <span class="tracks-info-meta">语言 {{ info.lang || '-' }}</span>
                      <span class="tracks-info-meta">语言名称 {{ info.languageName || '-' }}</span>
                      <span class="tracks-info-meta">名称 {{ info.trackName || '-' }}</span>
                      <span class="tracks-info-meta">
                        默认 {{ info.isDefault === true ? '是' : info.isDefault === false ? '否' : '-' }}
                      </span>
                      <span class="tracks-info-meta">
                        强制 {{ info.isForced === true ? '是' : info.isForced === false ? '否' : '-' }}
                      </span>
                      <span class="tracks-info-meta">字符集 {{ info.charset || '-' }}</span>
                      <span class="tracks-info-meta">属性 {{ info.attributes || '-' }}</span>
                      <span class="tracks-info-meta">容器 {{ info.container || '-' }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="tracks-section">
              <div class="tracks-header">
                <span class="tracks-title">音频</span>
                <div class="tracks-actions">
                  <NButton size="small" secondary @click="addTrackFile('audio')">添加文件</NButton>
                  <NButton
                    size="small"
                    type="primary"
                    :loading="trackLoading.audio"
                    :disabled="!trackFiles.audio.length"
                    @click="detectTracks('audio')"
                  >
                    检测
                  </NButton>
                </div>
              </div>
              <div class="tracks-body">
                <div class="tracks-files" v-if="trackFiles.audio.length">
                  <div v-for="file in trackFiles.audio" :key="file.id" class="tracks-file">
                    <span class="tracks-file-name">{{ file.name }}</span>
                    <span class="tracks-file-size">{{ file.fileSize || '-' }}</span>
                    <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
                  </div>
                </div>
                <p v-else class="download-empty">尚未添加音频文件。</p>
                <div v-if="trackErrors.audio" class="tracks-error">{{ trackErrors.audio }}</div>
                <div v-if="trackLoading.audio" class="tracks-progress">
                  <NProgress type="line" :percentage="trackProgress.audio" :show-indicator="true" :height="8" />
                </div>
                <div v-if="trackInfos.audio.length" class="tracks-info">
                  <div v-for="group in trackInfos.audio" :key="group.file.id" class="tracks-info-group">
                    <div class="tracks-info-file">{{ group.file.name }}</div>
                    <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                      <NCheckbox v-model:checked="info.selected" size="small" />
                      <span class="tracks-info-name">轨道 {{ info.trackId || '-' }}</span>
                      <span class="tracks-info-meta">编码 {{ info.codec || '-' }}</span>
                      <span class="tracks-info-meta">语言 {{ info.lang || '-' }}</span>
                      <span class="tracks-info-meta">语言名称 {{ info.languageName || '-' }}</span>
                      <span class="tracks-info-meta">名称 {{ info.trackName || '-' }}</span>
                      <span class="tracks-info-meta">
                        默认 {{ info.isDefault === true ? '是' : info.isDefault === false ? '否' : '-' }}
                      </span>
                      <span class="tracks-info-meta">
                        强制 {{ info.isForced === true ? '是' : info.isForced === false ? '否' : '-' }}
                      </span>
                      <span class="tracks-info-meta">字符集 {{ info.charset || '-' }}</span>
                      <span class="tracks-info-meta">属性 {{ info.attributes || '-' }}</span>
                      <span class="tracks-info-meta">容器 {{ info.container || '-' }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="tracks-section">
              <div class="tracks-header">
                <span class="tracks-title">字幕</span>
                <div class="tracks-actions">
                  <NButton size="small" secondary @click="addTrackFile('subtitle')">添加文件</NButton>
                  <NButton
                    size="small"
                    type="primary"
                    :loading="trackLoading.subtitle"
                    :disabled="!trackFiles.subtitle.length"
                    @click="detectTracks('subtitle')"
                  >
                    检测
                  </NButton>
                </div>
              </div>
              <div class="tracks-body">
                <div class="tracks-files" v-if="trackFiles.subtitle.length">
                  <div v-for="file in trackFiles.subtitle" :key="file.id" class="tracks-file">
                    <span class="tracks-file-name">{{ file.name }}</span>
                    <span class="tracks-file-size">{{ file.fileSize || '-' }}</span>
                    <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
                  </div>
                </div>
                <p v-else class="download-empty">尚未添加字幕文件。</p>
                <div v-if="trackErrors.subtitle" class="tracks-error">{{ trackErrors.subtitle }}</div>
                <div v-if="trackLoading.subtitle" class="tracks-progress">
                  <NProgress type="line" :percentage="trackProgress.subtitle" :show-indicator="true" :height="8" />
                </div>
                <div v-if="trackInfos.subtitle.length" class="tracks-info">
                  <div v-for="group in trackInfos.subtitle" :key="group.file.id" class="tracks-info-group">
                    <div class="tracks-info-file">{{ group.file.name }}</div>
                    <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                      <NCheckbox v-model:checked="info.selected" size="small" />
                      <span class="tracks-info-name">轨道 {{ info.trackId || '-' }}</span>
                      <span class="tracks-info-meta">编码 {{ info.codec || '-' }}</span>
                      <span class="tracks-info-meta">语言 {{ info.lang || '-' }}</span>
                      <span class="tracks-info-meta">语言名称 {{ info.languageName || '-' }}</span>
                      <span class="tracks-info-meta">名称 {{ info.trackName || '-' }}</span>
                      <span class="tracks-info-meta">
                        默认 {{ info.isDefault === true ? '是' : info.isDefault === false ? '否' : '-' }}
                      </span>
                      <span class="tracks-info-meta">
                        强制 {{ info.isForced === true ? '是' : info.isForced === false ? '否' : '-' }}
                      </span>
                      <span class="tracks-info-meta">字符集 {{ info.charset || '-' }}</span>
                      <span class="tracks-info-meta">属性 {{ info.attributes || '-' }}</span>
                      <span class="tracks-info-meta">容器 {{ info.container || '-' }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="tracks-section">
              <div class="tracks-header">
                <span class="tracks-title">混流任务队列</span>
                <div class="tracks-actions">
                  <NButton size="small" secondary :disabled="mixQueueRunning" @click="startMixQueue">
                    开始任务队列
                  </NButton>
                  <NButton size="small" secondary :disabled="mixQueueRunning || !mixQueue.length" @click="clearMixQueue">
                    清除所有任务
                  </NButton>
                </div>
              </div>
              <div class="tracks-body">
                <p v-if="!mixQueue.length" class="download-empty">暂无混流任务。</p>
                <div v-else class="mix-queue-list">
                  <div v-for="item in mixQueue" :key="item.id" class="mix-queue-row" @click.stop="openMixTaskDetail(item)">
                    <span class="mix-queue-id">#{{ item.id }}</span>
                    <span class="mix-queue-time">{{ item.createdAt }}</span>
                    <span class="mix-queue-output" :title="item.outputPath">{{ item.outputPath }}</span>
                    <span class="mix-queue-status" :data-status="item.status">
                      {{
                        item.status === 'queued'
                          ? '排队中'
                          : item.status === 'running'
                            ? '处理中'
                            : item.status === 'success'
                              ? '完成'
                              : '失败'
                      }}
                    </span>
                    <div v-if="item.message" class="mix-queue-message-row" :title="item.message">
                      {{ item.message }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </NCard>
      </div>

      <div v-else class="app-body search-view">
        <NCard size="small" class="search-panel">
          <template #header>
            <div class="search-card-header">
              <span>搜索资源</span>
              <span class="search-hint header-hint">先选逻辑，再添加短语或番剧；逻辑只对后续新增项生效</span>
            </div>
          </template>
          <div class="search-controls">
            <div class="search-row">
              <span class="search-label">逻辑</span>
              <div class="search-button-group logic-group">
                <NButton
                  v-for="logic in logicOptions"
                  :key="logic.value"
                  secondary
                  size="small"
                  :type="activeLogic === logic.value ? 'primary' : 'default'"
                  @click="activeLogic = logic.value"
                >
                  {{ logic.label }}
                </NButton>
              </div>
            </div>

            <div class="search-row">
              <span class="search-label">预设</span>
              <div class="search-button-group">
                <NButton
                  v-for="phrase in presetPhrases"
                  :key="phrase"
                  size="small"
                  secondary
                  @click="handleAddPreset(phrase)"
                >
                  {{ phrase }}
                </NButton>
              </div>
            </div>

            <div class="search-row compact">
              <span class="search-label">番剧</span>
              <NSelect
                v-model:value="trackedSelection"
                :options="trackedOptions"
                placeholder="选择正在追番/补番/已完番剧"
                clearable
                @update:value="handleSelectTracked"
              />
              <NInput
                v-model:value="customSearchInput"
                placeholder="输入自定义精确短语"
                clearable
                @keyup.enter="handleAddCustom"
                class="search-input-flex"
              />
              <NButton type="primary" @click="handleAddCustom">添加</NButton>
            </div>

            <div class="search-row search-tags" v-if="searchTerms.length">
              <span class="search-label">已选</span>
              <div class="search-tag-list">
                <NTag
                  v-for="(term, idx) in searchTerms"
                  :key="idx"
                  size="small"
                  closable
                  @close="removeSearchTerm(idx)"
                >
                  <span class="term-op">{{ term.op === 'and' ? '与' : term.op === 'or' ? '或' : '非' }}</span>
                  <span class="term-value">{{ term.value }}</span>
                </NTag>
              </div>
            </div>

            <div class="search-row">
              <span class="search-label">拼接结果</span>
              <div class="search-preview">
                <div class="search-query">{{ searchQuery || '（尚未添加关键词）' }}</div>
              </div>
              <NButton type="primary" :disabled="!searchQuery" @click="openSearch">打开搜索</NButton>
            </div>

            <div
              v-if="searchLoading || searchError || searchResults.length || searchHtml"
              class="search-inline-results"
            >
              <div class="search-result-header">
                <span>搜索结果</span>
                <NButton size="tiny" secondary @click="clearSearchResults">收起</NButton>
              </div>
              <div class="search-open-modal">
                <p class="search-modal-row">
                  <span class="search-modal-label">URL：</span>
                  <a
                    :href="searchUrl"
                    target="_blank"
                    rel="noreferrer"
                    @click.prevent="openExternalLink(searchUrl)"
                  >
                    {{ searchUrl }}
                  </a>
                </p>
                <div v-if="searchResults.length" class="search-result-list">
                  <div class="search-result-row" v-for="item in searchResults" :key="item.detailUrl || item.title">
                    <div class="sr-name">
                      <a
                        :href="item.detailUrl || item.magnet || item.download"
                        target="_blank"
                        rel="noreferrer"
                        @click.prevent="openExternalLink(item.detailUrl || item.magnet || item.download)"
                      >
                        {{ item.title }}
                      </a>
                      <div class="sr-meta" v-if="item.size || item.date">
                        <span v-if="item.size">{{ item.size }}</span>
                        <span v-if="item.date">{{ item.date }}</span>
                      </div>
                    </div>
                    <div class="sr-links">
                      <NButton
                        v-if="item.magnet"
                        text
                        type="primary"
                        size="small"
                        @click="handleDownloadClick(item, 'magnet', item.magnet)"
                      >
                        磁链
                      </NButton>
                      <NButton
                        v-if="item.download"
                        text
                        type="primary"
                        size="small"
                        @click="handleDownloadClick(item, 'torrent', item.download)"
                      >
                        种子
                      </NButton>
                    </div>
                  </div>
                </div>
                <iframe
                  v-else-if="!searchLoading && !searchError"
                  class="search-preview-frame"
                  :srcdoc="searchHtml"
                  sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
                  title="搜索页面"
                />
                <div v-if="searchLoading" class="search-loading">正在加载...</div>
                <div v-else-if="searchError" class="search-error">{{ searchError }}</div>
              </div>
            </div>
          </div>
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
                        @click.prevent="openExternalLink(person.url)"
                >
                  {{ person.name }}
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </NModal>

    <NModal
      v-model:show="aliasModalVisible"
      preset="card"
      title="选择名称 / 别名"
      size="small"
      :style="{ width: 'auto', maxWidth: '520px' }"
    >
      <div class="alias-modal-body">
        <p class="alias-title">为 {{ pendingTrackedName || "该番剧" }} 选择要加入的名称</p>
        <p v-if="aliasLoading" class="alias-hint">正在获取别名...</p>
        <template v-else>
          <NCheckboxGroup v-model:value="aliasSelected">
            <div class="alias-list">
              <NCheckbox v-for="name in aliasOptions" :key="name" :value="name" class="alias-item">
                {{ name }}
              </NCheckbox>
            </div>
          </NCheckboxGroup>
        </template>
        <div class="alias-actions">
          <NButton size="small" @click="cancelAliasSelection">取消</NButton>
          <NButton size="small" type="primary" :disabled="aliasLoading || !aliasSelected.length" @click="confirmAliasSelection">
            确定
          </NButton>
        </div>
      </div>
    </NModal>

    <NModal v-model:show="mixQueueDetailVisible" preset="card" title="混流任务详情" style="width: min(720px, 92vw)">
      <div v-if="selectedMixTask" class="mix-queue-detail">
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">任务 ID</span>
          <span>#{{ selectedMixTask.id }}</span>
        </div>
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">创建时间</span>
          <span>{{ selectedMixTask.createdAt }}</span>
        </div>
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">输出路径</span>
          <span class="mix-queue-detail-value" :title="selectedMixTask.outputPath">
            {{ selectedMixTask.outputPath }}
          </span>
        </div>
        <div class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">状态</span>
          <span>{{
            selectedMixTask.status === 'queued'
              ? '排队中'
              : selectedMixTask.status === 'running'
                ? '处理中'
                : selectedMixTask.status === 'success'
                  ? '完成'
                  : '失败'
          }}</span>
        </div>
        <div v-if="selectedMixTask.message" class="mix-queue-detail-row">
          <span class="mix-queue-detail-label">消息</span>
          <span class="mix-queue-detail-value" :title="selectedMixTask.message">
            {{ selectedMixTask.message }}
          </span>
        </div>

        <div class="mix-queue-detail-section">轨道输入</div>
        <div v-for="(input, index) in selectedMixTask.inputs" :key="`${input.kind}-${index}`" class="mix-queue-detail-block">
          <div class="mix-queue-detail-row">
            <span class="mix-queue-detail-label">类型</span>
            <span>{{ input.kind === 'video' ? '视频' : input.kind === 'audio' ? '音频' : '字幕' }}</span>
          </div>
          <div class="mix-queue-detail-row">
            <span class="mix-queue-detail-label">来源文件</span>
            <span class="mix-queue-detail-value" :title="input.path">{{ input.path }}</span>
          </div>
          <div class="mix-queue-detail-row">
            <span class="mix-queue-detail-label">轨道 ID</span>
            <span>{{ input.trackIds.join(', ') || '-' }}</span>
          </div>
          <div class="mix-queue-detail-row">
            <span class="mix-queue-detail-label">语言设置</span>
            <span class="mix-queue-detail-value">
              {{
                input.trackIds
                  .map((id) => `${id}:${input.trackLangs?.[id] || '-'}`)
                  .join('、') || '-'
              }}
            </span>
          </div>
        </div>
      </div>
    </NModal>
  </NConfigProvider>
</template>
