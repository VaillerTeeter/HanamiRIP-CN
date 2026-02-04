import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Ref } from "vue";
import type { SearchResult, SearchTerm, LogicOp } from "../types/search";
import type { TrackedItem } from "../../tracking/types/tracking";

const NYAA_BASE = "https://nyaa.vaciller.top/?f=0&c=0_0&q=";
const presetPhrases = ["SubsPlease", "LoliHouse", "内封", "外挂", "480", "720", "1080"];
const logicOptions: { label: string; value: LogicOp }[] = [
  { label: "与", value: "and" },
  { label: "或", value: "or" },
  { label: "非", value: "not" },
];

export const useSearchPage = (options: { trackedItems: Ref<TrackedItem[]> }) => {
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
    options.trackedItems.value.map((item) => ({
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
    const target = options.trackedItems.value.find((item) => item.id === value);
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
      // ignore
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
    trackedSelection.value = null;
  };

  const confirmAliasSelection = () => {
    if (!pendingTrackedId.value) {
      cancelAliasSelection();
      return;
    }
    if (!aliasSelected.value.length) {
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
          const anchorCandidates = nameCell ? Array.from(nameCell.querySelectorAll("a")) : [];
          const nameLink = anchorCandidates.filter((a) => a.getAttribute("href")?.includes("/view/")).pop();
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

  return {
    presetPhrases,
    logicOptions,
    activeLogic,
    searchTerms,
    customSearchInput,
    trackedSelection,
    trackedOptions,
    searchLoading,
    searchError,
    searchHtml,
    searchResults,
    aliasModalVisible,
    aliasLoading,
    aliasOptions,
    aliasSelected,
    pendingTrackedName,
    searchQuery,
    searchUrl,
    handleAddPreset,
    handleAddCustom,
    handleSelectTracked,
    removeSearchTerm,
    openSearch,
    clearSearchResults,
    cancelAliasSelection,
    confirmAliasSelection,
  };
};

export type UseSearchPageReturn = ReturnType<typeof useSearchPage>;
