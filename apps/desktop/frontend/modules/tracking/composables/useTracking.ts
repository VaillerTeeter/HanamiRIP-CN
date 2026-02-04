import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { MonthAnime } from "../types/anime";
import type { ItemStatus, StatusKey, TrackedItem } from "../types/tracking";

export const useTracking = () => {
  const statuses = ref<Record<number, ItemStatus>>({});
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

  const weekdayLabels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
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

  const normalizeStatus = (state?: ItemStatus): ItemStatus => {
    if (!state) return { watching: false, backlog: false, watched: false };
    const keys: Array<keyof ItemStatus> = ["watched", "watching", "backlog"];
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
    if (target && Object.prototype.hasOwnProperty.call(base, target)) {
      (base as any)[target] = true;
    }
    await persistStatusToDb(item, base);
  };

  const updateTrackedItem = (id: number, patch: Partial<TrackedItem>) => {
    trackedItems.value = trackedItems.value.map((item) => (item.id === id ? { ...item, ...patch } : item));
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
          // ignore
        }
      }
    };

    const concurrency = Math.min(CONCURRENT_REFRESH_LIMIT, queue.length);
    await Promise.all(Array.from({ length: concurrency }, () => worker()));

    if (options.persist && updated.length) {
      const persistQueue = [...updated];
      const persistWorker = async () => {
        while (persistQueue.length) {
          const item = persistQueue.shift();
          if (!item) continue;
          try {
            await invoke<TrackedItem[]>("save_tracked_subject", { subject: item });
          } catch (_) {
            // ignore
          }
        }
      };
      await Promise.all(Array.from({ length: Math.min(3, persistQueue.length || 1) }, () => persistWorker()));
    }
  };

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

  onMounted(async () => {
    try {
      const saved = await invoke<TrackedItem[]>("list_tracked_subjects");
      syncStatusesFromTracked(saved);
      void refreshWatchingDetails({ persist: true });
    } catch (_) {
      statuses.value = {};
      trackedItems.value = [];
    }
  });

  return {
    statuses,
    trackedItems,
    watchingList,
    backlogList,
    finishedList,
    watchingByWeekday,
    ensureStatus,
    setExclusiveStatus,
    refreshWatchingDetails,
    currentStatusKey,
    labelForAction,
  };
};

export type UseTrackingReturn = ReturnType<typeof useTracking>;
