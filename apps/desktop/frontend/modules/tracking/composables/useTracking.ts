/** 文件：useTracking.ts | 用途：维护追踪条目状态、分组列表与状态持久化逻辑 | 关键对象：useTracking, watchingByWeekday, setExclusiveStatus, refreshWatchingDetails */
// 本行目的：引入 Vue 响应式与生命周期能力，构建追踪模块状态。
import { computed, onMounted, ref } from "vue";
// 本行目的：引入 Tauri invoke，用于调用后端命令读写追踪数据。
import { invoke } from "@tauri-apps/api/core";
// 本行目的：引入月番条目类型，作为状态切换与持久化输入。
import type { MonthAnime } from "../types/anime";
// 本行目的：引入追踪状态与条目类型，约束本模块核心数据结构。
import type { ItemStatus, StatusKey, TrackedItem } from "../types/tracking";

// 变量：useTracking | 含义：追踪模块组合式函数入口 | 类型：() => UseTrackingReturn | 作用域：模块级
/** 函数：useTracking | 输入：无 | 输出：追踪页所需状态与方法集合 | 可能失败：内部 invoke 调用可能失败（已在局部捕获） */
// 本行目的：导出追踪模块的统一状态管理入口。
export const useTracking = () => {
    // 变量：statuses | 含义：按条目ID索引的状态字典 | 类型：Ref<Record<number, ItemStatus>> | 作用域：useTracking 内部
    // 本行目的：初始化状态映射表，便于 O(1) 读取单条状态。
    const statuses = ref<Record<number, ItemStatus>>({});

    // 变量：trackedItems | 含义：已追踪条目数组 | 类型：Ref<TrackedItem[]> | 作用域：useTracking 内部
    // 本行目的：初始化追踪条目列表。
    const trackedItems = ref<TrackedItem[]>([]);

    // 变量：parseDateValue | 含义：把日期字符串转换为可比较时间戳 | 类型：(date?: string) => number | 作用域：useTracking 内部
    /** 函数：parseDateValue | 输入：可选日期字符串（yyyy-mm-dd） | 输出：时间戳或正无穷 | 可能失败：日期非法时返回兜底值 */
    // 本行目的：定义日期比较值转换函数，供排序逻辑复用。
    const parseDateValue = (date?: string) => {
        // 本行目的：无日期时返回正无穷，使其在升序排序中靠后。
        if (!date) return Number.POSITIVE_INFINITY;
        // 本行目的：构造本地日期对象，固定时间为当天零点。
        const parsed = new Date(`${date}T00:00:00`);
        // 本行目的：无效日期返回正无穷，有效日期返回时间戳。
        return Number.isNaN(parsed.getTime()) ? Number.POSITIVE_INFINITY : parsed.getTime();
    };

    // 变量：sortByDate | 含义：按日期升序排序追踪条目 | 类型：(list: TrackedItem[]) => TrackedItem[] | 作用域：useTracking 内部
    /** 函数：sortByDate | 输入：追踪条目数组 | 输出：日期升序的新数组 | 可能失败：无（纯计算） */
    // 本行目的：定义日期排序函数，复制后排序避免修改原数组。
    const sortByDate = (list: TrackedItem[]) => list.slice().sort((a, b) => parseDateValue(a.date) - parseDateValue(b.date));

    // 变量：sortByName | 含义：按中文优先名称排序追踪条目 | 类型：(list: TrackedItem[]) => TrackedItem[] | 作用域：useTracking 内部
    /** 函数：sortByName | 输入：追踪条目数组 | 输出：名称升序的新数组 | 可能失败：无（纯计算） */
    // 本行目的：定义名称排序函数，使用中文区域规则比较。
    const sortByName = (list: TrackedItem[]) => list.slice().sort((a, b) => (a.nameCn || a.name).localeCompare(b.nameCn || b.name, "zh-Hans-CN"));

    // 变量：watchingList | 含义：在看列表（按日期排序） | 类型：ComputedRef<TrackedItem[]> | 作用域：useTracking 内部
    /** 函数：watchingList | 输入：无（读取 trackedItems） | 输出：在看条目数组 | 可能失败：无（纯派生） */
    // 本行目的：筛选 watching 条目并按日期排序。
    const watchingList = computed(() => sortByDate(trackedItems.value.filter((item) => item.watching)));
    // 变量：backlogList | 含义：想看列表（按名称排序） | 类型：ComputedRef<TrackedItem[]> | 作用域：useTracking 内部
    /** 函数：backlogList | 输入：无（读取 trackedItems） | 输出：想看条目数组 | 可能失败：无（纯派生） */
    // 本行目的：筛选 backlog 条目并按名称排序。
    const backlogList = computed(() => sortByName(trackedItems.value.filter((item) => item.backlog)));
    // 变量：finishedList | 含义：看完列表（按名称排序） | 类型：ComputedRef<TrackedItem[]> | 作用域：useTracking 内部
    /** 函数：finishedList | 输入：无（读取 trackedItems） | 输出：看完条目数组 | 可能失败：无（纯派生） */
    // 本行目的：筛选 watched 条目并按名称排序。
    const finishedList = computed(() => sortByName(trackedItems.value.filter((item) => item.watched)));
    // 变量：todayIndex | 含义：今天星期索引（0-6） | 类型：Ref<number> | 作用域：useTracking 内部
    // 本行目的：初始化今天星期值，用于分组排序起点。
    const todayIndex = ref(new Date().getDay());
    // 变量：weekdayOrderSeed | 含义：星期顺序刷新触发器 | 类型：Ref<number> | 作用域：useTracking 内部
    // 本行目的：通过自增值显式触发依赖该值的 computed 重新计算。
    const weekdayOrderSeed = ref(0);

    // 变量：refreshWeekdayOrder | 含义：刷新本地星期并触发分组顺序重算 | 类型：() => Promise<void> | 作用域：useTracking 内部
    /** 函数：refreshWeekdayOrder | 输入：无 | 输出：无（更新 todayIndex 与 seed） | 可能失败：后端命令失败时回退本地时间 */
    // 本行目的：定义刷新星期顺序的异步方法。
    const refreshWeekdayOrder = async () => {
        try {
            // 本行目的：调用后端获取本地星期，避免时区偏差。
            const weekday = await invoke<number>("get_local_weekday");

            // 本行目的：仅在返回值是有限数字时才更新 todayIndex。
            if (Number.isFinite(weekday)) {
                // 本行目的：写入最新星期索引。
                todayIndex.value = weekday;
            }
        } catch (_) {
            // 本行目的：后端失败时回退到浏览器本地星期值。
            todayIndex.value = new Date().getDay();
        } finally {
            // 本行目的：无论成功失败都触发依赖方重算。
            weekdayOrderSeed.value += 1;
        }
    };

    // 变量：watchingByWeekday | 含义：按“从今天开始”顺序分组的在看列表 | 类型：ComputedRef<Array<{key:number|null;items:TrackedItem[]}>> | 作用域：useTracking 内部
    /** 函数：watchingByWeekday | 输入：无（读取 watchingList/todayIndex） | 输出：按星期分组且过滤空组的列表 | 可能失败：无（异常日期已兜底到 null 组） */
    // 本行目的：按星期组织在看条目，供页面按周视图展示。
    const watchingByWeekday = computed(() => {
        // 本行目的：建立对 seed 的依赖，确保 refreshWeekdayOrder 后触发重算。
        void weekdayOrderSeed.value;

        // 变量：groups | 含义：按星期键聚合条目（null 表示无效或缺失日期） | 类型：Map<number | null, TrackedItem[]> | 作用域：watchingByWeekday 内部
        // 本行目的：初始化分组容器。
        const groups = new Map<number | null, TrackedItem[]>();

        // 本行目的：遍历在看条目并写入对应星期分组。
        watchingList.value.forEach((item) => {
            // 变量：date | 含义：当前条目的日期字符串 | 类型：string | undefined | 作用域：forEach 回调内
            // 本行目的：读取条目日期字段。
            const date = item.date;

            // 本行目的：无日期条目归入 null 分组。
            if (!date) {
                // 本行目的：向 null 分组追加当前条目。
                groups.set(null, [...(groups.get(null) || []), item]);
                // 本行目的：结束本次条目处理。
                return;
            }

            // 本行目的：把日期字符串解析为 Date 对象。
            const parsed = new Date(`${date}T00:00:00`);

            // 本行目的：无效日期同样归入 null 分组。
            if (Number.isNaN(parsed.getTime())) {
                // 本行目的：向 null 分组追加当前条目。
                groups.set(null, [...(groups.get(null) || []), item]);
                // 本行目的：结束本次条目处理。
                return;
            }

            // 变量：day | 含义：解析后的星期索引（0-6） | 类型：number | 作用域：forEach 回调内
            // 本行目的：提取日期对应星期。
            const day = parsed.getDay();

            // 本行目的：向对应星期分组追加当前条目。
            groups.set(day, [...(groups.get(day) || []), item]);
        });

        // 变量：weekdayOrder | 含义：从今天开始的 7 天星期顺序 | 类型：number[] | 作用域：watchingByWeekday 内部
        // 本行目的：生成“今天 -> 明天 -> ...”的星期索引序列。
        const weekdayOrder = Array.from({ length: 7 }, (_, i) => (todayIndex.value + i) % 7);
        // 变量：order | 含义：最终分组顺序（末尾追加 null 组） | 类型：Array<number | null> | 作用域：watchingByWeekday 内部
        // 本行目的：把 null 分组放在最后展示。
        const order: Array<number | null> = [...weekdayOrder, null];

        // 变量：sortByName | 含义：分组内条目名称排序函数 | 类型：(list: TrackedItem[]) => TrackedItem[] | 作用域：watchingByWeekday 内部
        // 本行目的：定义分组内部按中文优先名称排序逻辑。
        const sortByName = (list: TrackedItem[]) => list.slice().sort((a, b) => (a.nameCn || a.name).localeCompare(b.nameCn || b.name, "zh-Hans-CN"));

        // 本行目的：按既定顺序构建分组对象并过滤空分组。
        return order
            // 本行目的：把每个分组键映射为 { key, items } 结构。
            .map((key) => ({
                // 本行目的：保留当前分组键。
                key,
                // 本行目的：读取分组条目并按名称排序。
                items: sortByName(groups.get(key) || []),
            }))
            // 本行目的：去掉没有条目的分组，避免界面空占位。
            .filter((group) => group.items.length);
    });

    // 变量：normalizeStatus | 含义：把任意状态输入规范为互斥状态对象 | 类型：(state?: ItemStatus) => ItemStatus | 作用域：useTracking 内部
    /** 函数：normalizeStatus | 输入：可选状态对象 | 输出：仅一个状态为 true 的规范状态 | 可能失败：无（无输入时返回全 false） */
    // 本行目的：定义状态互斥规范化函数，避免同条目多个状态同时为 true。
    const normalizeStatus = (state?: ItemStatus): ItemStatus => {
        // 本行目的：无状态输入时返回全 false 的默认状态。
        if (!state) return { watching: false, backlog: false, watched: false };

        // 变量：keys | 含义：状态字段优先级遍历数组 | 类型：Array<keyof ItemStatus> | 作用域：normalizeStatus 内部
        // 本行目的：定义互斥判断顺序（watched -> watching -> backlog）。
        const keys: Array<keyof ItemStatus> = ["watched", "watching", "backlog"];
        // 变量：active | 含义：命中的第一个激活状态键 | 类型：keyof ItemStatus | undefined | 作用域：normalizeStatus 内部
        // 本行目的：查找当前输入中第一个 true 的状态字段。
        const active = keys.find((key) => state[key]);

        // 本行目的：按命中键返回互斥后的规范状态对象。
        return {
            // 本行目的：仅当 active 为 watching 时置 true。
            watching: active === "watching",
            // 本行目的：仅当 active 为 backlog 时置 true。
            backlog: active === "backlog",
            // 本行目的：仅当 active 为 watched 时置 true。
            watched: active === "watched",
        };
    };

    // 变量：ensureStatus | 含义：确保指定条目存在规范状态对象 | 类型：(id: number) => ItemStatus | 作用域：useTracking 内部
    /** 函数：ensureStatus | 输入：条目ID | 输出：该 ID 对应的规范状态对象 | 可能失败：无 */
    // 本行目的：读取或初始化某个条目的状态，并确保其互斥规范。
    const ensureStatus = (id: number): ItemStatus => {
        // 本行目的：不存在状态时写入默认全 false。
        if (!statuses.value[id]) {
            statuses.value[id] = { watching: false, backlog: false, watched: false };
        } else {
            // 本行目的：已存在状态时执行规范化，防止脏数据。
            statuses.value[id] = normalizeStatus(statuses.value[id]);
        }

        // 本行目的：返回最终可用状态对象。
        return statuses.value[id];
    };

    // 变量：syncStatusesFromTracked | 含义：根据追踪条目数组重建状态映射并同步列表 | 类型：(items: TrackedItem[]) => void | 作用域：useTracking 内部
    /** 函数：syncStatusesFromTracked | 输入：追踪条目数组 | 输出：无（更新 statuses 与 trackedItems） | 可能失败：无 */
    // 本行目的：统一从后端返回数据同步前端状态。
    const syncStatusesFromTracked = (items: TrackedItem[]) => {
        // 变量：map | 含义：重建后的状态字典 | 类型：Record<number, ItemStatus> | 作用域：syncStatusesFromTracked 内部
        // 本行目的：初始化临时状态映射。
        const map: Record<number, ItemStatus> = {};

        // 本行目的：遍历条目并写入规范化状态。
        items.forEach((item) => {
            // 本行目的：按条目 ID 保存规范状态对象。
            map[item.id] = normalizeStatus({
                // 本行目的：把 watching 转为明确布尔值。
                watching: !!item.watching,
                // 本行目的：把 backlog 转为明确布尔值。
                backlog: !!item.backlog,
                // 本行目的：把 watched 转为明确布尔值。
                watched: !!item.watched,
            });
        });

        // 本行目的：整体替换状态映射，触发响应式更新。
        statuses.value = map;
        // 本行目的：同步保存最新追踪条目数组。
        trackedItems.value = items;
    };

    // 变量：persistStatusToDb | 含义：把条目状态持久化到数据库并回写前端状态 | 类型：(item: MonthAnime, nextStatus: ItemStatus) => Promise<void> | 作用域：useTracking 内部
    /** 函数：persistStatusToDb | 输入：月番条目与目标状态 | 输出：无（保存后同步状态） | 可能失败：后端保存命令失败 */
    // 本行目的：定义状态保存到后端的统一入口。
    const persistStatusToDb = async (item: MonthAnime, nextStatus: ItemStatus) => {
        // 本行目的：缺少 ID 时无法持久化，直接返回。
        if (!item.id) return;

        // 变量：payload | 含义：后端保存接口需要的条目快照 | 类型：对象字面量（subject payload） | 作用域：persistStatusToDb 内部
        // 本行目的：组装后端所需完整字段，缺省值在前端补齐。
        const payload = {
            // 本行目的：写入条目 ID。
            id: item.id,
            // 本行目的：写入原始名称。
            name: item.name,
            // 本行目的：写入中文名称，缺省为空串。
            nameCn: item.nameCn || "",
            // 本行目的：写入封面图地址。
            image: item.image,
            // 本行目的：写入条目链接，缺省使用 bgm subject 地址。
            url: item.url || `https://bgm.tv/subject/${item.id}`,
            // 本行目的：写入日期，缺省为空串。
            date: item.date || "",
            // 本行目的：写入评分，缺省为 null。
            rating: item.rating ?? null,
            // 本行目的：写入简介，缺省为空串。
            summary: item.summary || "",
            // 本行目的：写入已放送集数，缺省为 0。
            airedCount: item.airedCount ?? 0,
            // 本行目的：写入总集数，缺省为 0。
            totalCount: item.totalCount ?? 0,
            // 本行目的：写入 watching 目标状态。
            watching: nextStatus.watching,
            // 本行目的：写入 backlog 目标状态。
            backlog: nextStatus.backlog,
            // 本行目的：写入 watched 目标状态。
            watched: nextStatus.watched,
        };

        // 本行目的：调用后端保存条目并获取最新追踪列表。
        const saved = await invoke<TrackedItem[]>("save_tracked_subject", { subject: payload });

        // 本行目的：用后端返回结果同步本地状态，保证前后端一致。
        syncStatusesFromTracked(saved);
    };

    // 变量：setExclusiveStatus | 含义：切换条目到目标状态（互斥）或取消当前状态 | 类型：(item: MonthAnime, target: StatusKey) => Promise<void> | 作用域：useTracking 内部
    /** 函数：setExclusiveStatus | 输入：月番条目与目标状态键 | 输出：无（保存并同步） | 可能失败：后端持久化失败 */
    // 本行目的：定义状态切换核心逻辑，保证三态互斥。
    const setExclusiveStatus = async (item: MonthAnime, target: StatusKey) => {
        // 本行目的：无 ID 或无目标状态时直接退出。
        if (!item.id || !target) return;

        // 变量：base | 含义：全 false 的基础状态模板 | 类型：ItemStatus | 作用域：setExclusiveStatus 内部
        // 本行目的：初始化一个“未选择任何状态”的模板。
        const base: ItemStatus = { watching: false, backlog: false, watched: false };
        // 变量：current | 含义：当前条目已存在的规范状态 | 类型：ItemStatus | 作用域：setExclusiveStatus 内部
        // 本行目的：读取并确保当前状态合法。
        const current = ensureStatus(item.id);

        // 本行目的：若点击的是当前已激活状态，则执行取消（保存全 false）。
        if (current[target]) {
            await persistStatusToDb(item, base);
            // 本行目的：已完成取消保存后结束当前切换流程。
            return;
        }

        // 本行目的：目标键合法时把目标状态置为 true。
        if (Object.prototype.hasOwnProperty.call(base, target)) {
            (base as ItemStatus)[target] = true;
        }

        // 本行目的：持久化新的互斥状态。
        await persistStatusToDb(item, base);
    };

    // 变量：updateTrackedItem | 含义：按 ID 局部更新追踪条目字段 | 类型：(id: number, patch: Partial<TrackedItem>) => void | 作用域：useTracking 内部
    /** 函数：updateTrackedItem | 输入：条目ID与补丁字段 | 输出：无（更新 trackedItems） | 可能失败：无 */
    // 本行目的：在不刷新整表的情况下更新单条追踪数据。
    const updateTrackedItem = (id: number, patch: Partial<TrackedItem>) => {
        // 本行目的：映射替换目标条目，其余条目保持不变。
        trackedItems.value = trackedItems.value.map((item) => (item.id === id ? { ...item, ...patch } : item));
    };

    // 变量：CONCURRENT_REFRESH_LIMIT | 含义：刷新详情并发上限 | 类型：number | 作用域：useTracking 内部常量
    // 本行目的：限制并发请求数，避免瞬时请求过高。
    const CONCURRENT_REFRESH_LIMIT = 6;

    // 变量：refreshWatchingDetails | 含义：刷新在看条目的详情与集数信息，可选持久化 | 类型：(options?: { persist?: boolean }) => Promise<void> | 作用域：useTracking 内部
    /** 函数：refreshWatchingDetails | 输入：可选 persist 开关 | 输出：无（更新本地条目并可选回写后端） | 可能失败：单条请求失败会被捕获跳过 */
    // 本行目的：定义批量刷新在看条目详情的异步流程。
    const refreshWatchingDetails = async (options: { persist?: boolean } = {}) => {
        // 变量：queue | 含义：待刷新条目队列 | 类型：TrackedItem[] | 作用域：refreshWatchingDetails 内部
        // 本行目的：复制在看列表作为可变队列。
        const queue = [...watchingList.value];
        // 本行目的：无待处理条目时直接结束。
        if (!queue.length) return;

        // 变量：updated | 含义：成功刷新后的条目集合 | 类型：TrackedItem[] | 作用域：refreshWatchingDetails 内部
        // 本行目的：收集成功更新的条目，供后续可选持久化。
        const updated: TrackedItem[] = [];

        // 变量：worker | 含义：消费队列并刷新单条详情的并发工作函数 | 类型：() => Promise<void> | 作用域：refreshWatchingDetails 内部
        /** 函数：worker | 输入：无（共享 queue） | 输出：无（更新本地并写入 updated） | 可能失败：单条 invoke 失败时捕获并跳过 */
        // 本行目的：定义单个并发工作协程，循环消费队列。
        const worker = async () => {
            // 本行目的：队列非空时持续处理。
            while (queue.length) {
                // 本行目的：从队首取一个条目。
                const item = queue.shift();
                // 本行目的：空值保护，防止并发竞争下空读取。
                if (!item) continue;

                try {
                    // 本行目的：请求条目基础详情。
                    const payload = await invoke<{
                        // 本行目的：声明返回条目ID字段类型。
                        id: number;
                        // 本行目的：声明返回原始名称字段类型。
                        name: string;
                        // 本行目的：声明返回中文名称字段类型。
                        nameCn: string;
                        // 本行目的：声明返回封面图字段类型。
                        image: string;
                        // 本行目的：声明返回日期字段类型。
                        date: string;
                        // 本行目的：声明返回评分字段类型（可为空）。
                        rating: number | null;
                        // 本行目的：声明返回简介字段类型。
                        summary: string;
                        // 本行目的：声明返回链接字段类型。
                        url: string;
                    }>("get_subject_brief", { id: item.id });

                    // 本行目的：请求已放送/总集数信息。
                    const count = await invoke<{ id: number; airedCount?: number | null; totalCount?: number | null }>("get_subject_aired_count", {
                        id: item.id,
                    });

                    // 变量：patch | 含义：本次需要合并到条目的更新字段 | 类型：Partial<TrackedItem> | 作用域：worker 内部
                    // 本行目的：组装详情和集数的更新补丁。
                    const patch: Partial<TrackedItem> = {
                        // 本行目的：写入最新原始名称。
                        name: payload.name,
                        // 本行目的：写入最新中文名称。
                        nameCn: payload.nameCn,
                        // 本行目的：写入最新封面图。
                        image: payload.image,
                        // 本行目的：写入最新日期。
                        date: payload.date,
                        // 本行目的：写入最新评分。
                        rating: payload.rating,
                        // 本行目的：写入最新简介。
                        summary: payload.summary,
                        // 本行目的：写入最新详情链接。
                        url: payload.url,
                        // 本行目的：写入最新已放送集数（缺失时回退旧值或0）。
                        airedCount: count.airedCount ?? item.airedCount ?? 0,
                        // 本行目的：写入最新总集数（缺失时回退旧值或0）。
                        totalCount: count.totalCount ?? item.totalCount ?? 0,
                    };

                    // 本行目的：先更新前端本地列表，提升界面响应速度。
                    updateTrackedItem(item.id, patch);

                    // 变量：merged | 含义：原条目与补丁合并后的完整对象 | 类型：TrackedItem | 作用域：worker 内部
                    // 本行目的：生成最终条目快照用于后续持久化队列。
                    const merged = { ...item, ...patch } as TrackedItem;
                    // 本行目的：记录成功刷新的条目。
                    updated.push(merged);
                } catch (_) {}
            }
        };

        // 变量：concurrency | 含义：实际并发工作数 | 类型：number | 作用域：refreshWatchingDetails 内部
        // 本行目的：按队列长度与并发上限计算并发数。
        const concurrency = Math.min(CONCURRENT_REFRESH_LIMIT, queue.length);
        // 本行目的：并发执行 worker，直到队列清空。
        await Promise.all(Array.from({ length: concurrency }, () => worker()));

        // 本行目的：开启持久化且存在更新条目时，批量回写数据库。
        if (options.persist && updated.length) {
            // 变量：persistQueue | 含义：待持久化条目队列 | 类型：TrackedItem[] | 作用域：refreshWatchingDetails 内部
            // 本行目的：复制更新结果作为持久化工作队列。
            const persistQueue = [...updated];

            // 变量：persistWorker | 含义：消费持久化队列并保存条目的工作函数 | 类型：() => Promise<void> | 作用域：refreshWatchingDetails 内部
            /** 函数：persistWorker | 输入：无（共享 persistQueue） | 输出：无 | 可能失败：单条保存失败会被捕获跳过 */
            // 本行目的：定义持久化并发工作协程。
            const persistWorker = async () => {
                // 本行目的：队列非空时持续保存。
                while (persistQueue.length) {
                    // 本行目的：从队首取一个待持久化条目。
                    const item = persistQueue.shift();
                    // 本行目的：空值保护，避免并发竞争下空读取。
                    if (!item) continue;

                    try {
                        // 变量：latestStatus | 含义：条目最新状态快照（优先 statuses） | 类型：ItemStatus | 作用域：persistWorker 内部
                        // 本行目的：读取最新状态，避免覆盖用户刚修改的状态。
                        const latestStatus = statuses.value[item.id] ?? normalizeStatus(item);

                        // 变量：subject | 含义：后端保存所需完整条目对象 | 类型：TrackedItem 兼容对象 | 作用域：persistWorker 内部
                        // 本行目的：合并条目详情与最新状态后再提交。
                        const subject = {
                            ...item,
                            // 本行目的：写入当前最新 watching 状态。
                            watching: latestStatus.watching,
                            // 本行目的：写入当前最新 backlog 状态。
                            backlog: latestStatus.backlog,
                            // 本行目的：写入当前最新 watched 状态。
                            watched: latestStatus.watched,
                        };

                        // 本行目的：调用后端保存当前条目。
                        await invoke<TrackedItem[]>("save_tracked_subject", { subject });
                    } catch (_) {}
                }
            };
            // 本行目的：以最多 3 并发执行持久化任务，降低写入压力。
            await Promise.all(Array.from({ length: Math.min(3, persistQueue.length || 1) }, () => persistWorker()));
        }
    };

    // 变量：currentStatusKey | 含义：从状态对象推导当前激活状态键 | 类型：(status: ItemStatus) => StatusKey | 作用域：useTracking 内部
    /** 函数：currentStatusKey | 输入：状态对象 | 输出：watched/watching/backlog/null | 可能失败：无 */
    // 本行目的：把布尔状态对象转换为单一状态键，便于界面判断。
    const currentStatusKey = (status: ItemStatus): StatusKey => {
        // 本行目的：优先判断 watched。
        if (status.watched) return "watched";
        // 本行目的：其次判断 watching。
        if (status.watching) return "watching";
        // 本行目的：再次判断 backlog。
        if (status.backlog) return "backlog";
        // 本行目的：都不命中时返回 null。
        return null;
    };

    /** 函数：onMounted 回调 | 输入：无 | 输出：无（初始化追踪列表） | 可能失败：列表查询失败时回退为空状态 */
    // 本行目的：组件挂载后加载已保存追踪数据并触发详情刷新。
    onMounted(async () => {
        try {
            // 本行目的：读取后端已保存的追踪条目。
            const saved = await invoke<TrackedItem[]>("list_tracked_subjects");
            // 本行目的：同步状态映射与条目列表。
            syncStatusesFromTracked(saved);
            // 本行目的：异步刷新在看条目详情并回写数据库（不阻塞当前流程）。
            void refreshWatchingDetails({ persist: true });
        } catch (_) {
            // 本行目的：加载失败时回退为空状态，避免界面持有脏数据。
            statuses.value = {};
            // 本行目的：清空追踪条目列表。
            trackedItems.value = [];
        }
    });

    // 本行目的：返回追踪模块对外可用的状态与方法。
    return {
        // 本行目的：导出状态映射字典。
        statuses,
        // 本行目的：导出追踪条目列表。
        trackedItems,
        // 本行目的：导出在看列表。
        watchingList,
        // 本行目的：导出想看列表。
        backlogList,
        // 本行目的：导出看完列表。
        finishedList,
        // 本行目的：导出按星期分组后的在看列表。
        watchingByWeekday,
        // 本行目的：导出星期顺序刷新方法。
        refreshWeekdayOrder,
        // 本行目的：导出状态确保方法。
        ensureStatus,
        // 本行目的：导出互斥状态切换方法。
        setExclusiveStatus,
        // 本行目的：导出详情刷新方法。
        refreshWatchingDetails,
        // 本行目的：导出当前状态键推导方法。
        currentStatusKey,
    };
};

/** 类型：UseTrackingReturn | 用途：导出 useTracking 返回值类型 | 字段：由 ReturnType 自动推断 */
// 本行目的：导出组合式返回类型供页面与组件复用。
export type UseTrackingReturn = ReturnType<typeof useTracking>;
