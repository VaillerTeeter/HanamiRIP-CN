/*
  追番状态管理：
  - 读取/保存追番列表
  - 维护 watching/backlog/watched 状态
  - 定时刷新条目信息
*/
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { MonthAnime } from "../types/anime";
import type { ItemStatus, StatusKey, TrackedItem } from "../types/tracking";

/**
 * 追番页业务入口：返回状态列表与操作函数。
 */
export const useTracking = () => {
  // 每个条目的状态（key=条目 id）。
  const statuses = ref<Record<number, ItemStatus>>({});
  // 追番列表本体。
  const trackedItems = ref<TrackedItem[]>([]);

  // 把日期字符串转换成可排序的时间戳（无效则排到最后）。
  const parseDateValue = (date?: string) => {
    if (!date) return Number.POSITIVE_INFINITY;
    const parsed = new Date(`${date}T00:00:00`);
    return Number.isNaN(parsed.getTime()) ? Number.POSITIVE_INFINITY : parsed.getTime();
  };

  // 按日期升序排序（不改变原数组）。
  const sortByDate = (list: TrackedItem[]) =>
    list
      .slice()
      .sort((a, b) => parseDateValue(a.date) - parseDateValue(b.date));

  // 不同状态的列表拆分。
  const watchingList = computed(() => sortByDate(trackedItems.value.filter((item) => item.watching)));
  const backlogList = computed(() => sortByDate(trackedItems.value.filter((item) => item.backlog)));
  const finishedList = computed(() => sortByDate(trackedItems.value.filter((item) => item.watched)));

  // 星期文案，用于按周几分组。
  const weekdayLabels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  // 把“正在追番”按星期分组，便于日历式展示。
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

  // 规范化状态：确保三者互斥（只保留一个 true）。
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

  // 确保某个条目有状态记录（无则初始化）。
  const ensureStatus = (id: number): ItemStatus => {
    if (!statuses.value[id]) {
      statuses.value[id] = { watching: false, backlog: false, watched: false };
    } else {
      statuses.value[id] = normalizeStatus(statuses.value[id]);
    }
    return statuses.value[id];
  };

  // 用数据库结果同步本地状态。
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

  // 把条目状态写入后端数据库。
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

  // 将条目设置为某个单一状态（互斥）。
  const setExclusiveStatus = async (item: MonthAnime, target: StatusKey) => {
    if (!item.id) return;
    const base: ItemStatus = { watching: false, backlog: false, watched: false };
    if (target && Object.prototype.hasOwnProperty.call(base, target)) {
      (base as any)[target] = true;
    }
    await persistStatusToDb(item, base);
  };

  // 更新单个条目字段（不可变更新）。
  const updateTrackedItem = (id: number, patch: Partial<TrackedItem>) => {
    trackedItems.value = trackedItems.value.map((item) => (item.id === id ? { ...item, ...patch } : item));
  };

  // 同时刷新条目数量上限，避免请求过多。
  const CONCURRENT_REFRESH_LIMIT = 6;

  // 刷新“正在追番”的详细信息（可选持久化）。
  const refreshWatchingDetails = async (options: { persist?: boolean } = {}) => {
    const queue = [...watchingList.value];
    if (!queue.length) return;
    const updated: TrackedItem[] = [];

    // worker 从队列中逐个取任务，直到队列为空。
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

    // 启动有限数量的并发 worker。
    const concurrency = Math.min(CONCURRENT_REFRESH_LIMIT, queue.length);
    await Promise.all(Array.from({ length: concurrency }, () => worker()));

    // 需要持久化时，再写回数据库。
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

  // 把三态转换成单个 key 便于 UI 展示。
  const currentStatusKey = (status: ItemStatus): StatusKey => {
    if (status.watched) return "watched";
    if (status.watching) return "watching";
    if (status.backlog) return "backlog";
    return null;
  };

  // 根据当前状态与目标状态生成按钮文案。
  const labelForAction = (active: StatusKey, target: StatusKey) => {
    if (target === "watching") return active === "watching" ? "正在追番" : active ? "转为正在追番" : "加入正在追番";
    if (target === "backlog") return active === "backlog" ? "补番计划" : active ? "转为补番计划" : "加入补番计划";
    if (target === "watched") return active === "watched" ? "已完番剧" : active ? "转为已完番剧" : "标记已完番剧";
    return "";
  };

  // 首次加载：读取追番列表并刷新详细信息。
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
