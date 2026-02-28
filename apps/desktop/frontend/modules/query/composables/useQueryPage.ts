/** 文件：useQueryPage.ts | 用途：查询页核心状态、筛选、加载与详情联动逻辑 | 关键对象：useQueryPage, loadSeasonData, handleQuery */
// 本行目的：引入 Vue 的响应式与生命周期工具，驱动查询页面状态流转。
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
// 本行目的：引入 Tauri invoke，用于调用后端查询与详情接口。
import { invoke } from "@tauri-apps/api/core";
// 本行目的：引入查询页需要的业务类型，约束数据字段访问与返回结构。
import type { MonthAnime, SeasonResponse, StaffGroup, CharacterLink } from "../../tracking/types/anime";
// 本行目的：引入条目状态类型，用于触发状态同步。
import type { ItemStatus } from "../../tracking/types/tracking";

// 变量：currentMonth | 含义：当前自然月（1~12） | 类型：number | 作用域：模块级
// 本行目的：记录当前月份，用于限制当年可选季度。
const currentMonth = new Date().getMonth() + 1;

// 变量：getSeasonStartMonth | 含义：把任意月份映射到季度起始月 | 类型：(value: number) => number | 作用域：模块级
/** 函数：getSeasonStartMonth | 输入：月份数字（1~12） | 输出：季度起始月（1/4/7/10） | 可能失败：无（纯判断） */
// 本行目的：定义月份到季度起始月的映射函数。
const getSeasonStartMonth = (value: number) => {
    // 本行目的：1~3 月归入冬季（1 月）。
    if (value >= 1 && value <= 3) return 1;
    // 本行目的：4~6 月归入春季（4 月）。
    if (value >= 4 && value <= 6) return 4;
    // 本行目的：7~9 月归入夏季（7 月）。
    if (value >= 7 && value <= 9) return 7;
    // 本行目的：其余归入秋季（10 月）。
    return 10;
    // 本行目的：结束 getSeasonStartMonth 函数体。
};

// 变量：currentYear | 含义：当前自然年 | 类型：number | 作用域：模块级
// 本行目的：记录当前年份，用于年份选项生成。
const currentYear = new Date().getFullYear();
// 变量：minYear | 含义：查询允许的最小年份 | 类型：number | 作用域：模块级
// 本行目的：定义年份下限，避免生成过早年份选项。
const minYear = 2000;

// 变量：yearOptions | 含义：年份下拉选项列表 | 类型：Array<{ label: string; value: number }> | 作用域：模块级
// 本行目的：生成从当前年到最小年的递减选项。
const yearOptions = Array.from({ length: currentYear - minYear + 1 }, (_, i) => {
    // 本行目的：按索引递减计算年份值。
    const value = currentYear - i;
    // 本行目的：返回单个年份选项对象。
    return { label: `${value}年`, value };
    // 本行目的：结束 yearOptions 构造回调。
});

// 变量：seasonOptionsAll | 含义：全年固定季度选项 | 类型：Array<{ label: string; value: number }> | 作用域：模块级
// 本行目的：定义四季固定选项（值使用季度起始月）。
const seasonOptionsAll = [
    // 本行目的：冬季选项（1 月）。
    { label: "冬季", value: 1 },
    // 本行目的：春季选项（4 月）。
    { label: "春季", value: 4 },
    // 本行目的：夏季选项（7 月）。
    { label: "夏季", value: 7 },
    // 本行目的：秋季选项（10 月）。
    { label: "秋季", value: 10 },
    // 本行目的：结束季度选项数组。
];

// 变量：seasonKeyFromMonth | 含义：季度起始月到后端 season key 的映射 | 类型：(value: number | null) => string | 作用域：模块级
/** 函数：seasonKeyFromMonth | 输入：季度起始月（可空） | 输出：后端季节键（winter/spring/summer/autumn） | 可能失败：无（纯映射） */
// 本行目的：定义季度值到 API 参数的映射函数。
const seasonKeyFromMonth = (value: number | null) => {
    // 本行目的：空值时返回空字符串，表示无效季节。
    if (!value) return "";
    // 本行目的：1 映射 winter。
    if (value === 1) return "winter";
    // 本行目的：4 映射 spring。
    if (value === 4) return "spring";
    // 本行目的：7 映射 summer。
    if (value === 7) return "summer";
    // 本行目的：10 映射 autumn。
    if (value === 10) return "autumn";
    // 本行目的：其它值视为无效。
    return "";
    // 本行目的：结束 seasonKeyFromMonth 函数体。
};

// 变量：seasonLabelFromMonth | 含义：季度起始月到中文季节名的映射 | 类型：(value: number | null) => string | 作用域：模块级
/** 函数：seasonLabelFromMonth | 输入：季度起始月（可空） | 输出：中文季节标签 | 可能失败：无（纯映射） */
// 本行目的：定义季度值到界面中文文案的映射函数。
const seasonLabelFromMonth = (value: number | null) => {
    // 本行目的：空值返回空字符串。
    if (!value) return "";
    // 本行目的：1 映射冬季。
    if (value === 1) return "冬季";
    // 本行目的：4 映射春季。
    if (value === 4) return "春季";
    // 本行目的：7 映射夏季。
    if (value === 7) return "夏季";
    // 本行目的：10 映射秋季。
    if (value === 10) return "秋季";
    // 本行目的：其它值返回空字符串。
    return "";
    // 本行目的：结束 seasonLabelFromMonth 函数体。
};

// 变量：preloadImages | 含义：预加载条目封面图，减少列表渲染时闪烁 | 类型：(items: MonthAnime[], timeoutMs?: number) => Promise<void> | 作用域：模块级
/** 函数：preloadImages | 输入：条目数组与超时时间 | 输出：预加载完成或超时后结束 | 可能失败：图片加载失败（已吞掉错误） */
// 本行目的：定义图片预加载函数并增加超时保护。
const preloadImages = (items: MonthAnime[], timeoutMs = 5000) => {
    // 本行目的：为每个条目创建独立加载任务，成功失败都算完成。
    const tasks = items.map(
        (item) =>
            new Promise<void>((resolve) => {
                // 本行目的：创建浏览器 Image 对象触发资源加载。
                const img = new Image();
                // 本行目的：加载成功即结束该任务。
                img.onload = () => resolve();
                // 本行目的：加载失败也结束该任务，避免整体阻塞。
                img.onerror = () => resolve();
                // 本行目的：写入图片地址开始加载。
                img.src = item.image;
            })
    );
    // 本行目的：任务全部完成或达到超时任一条件即返回。
    return Promise.race([Promise.all(tasks).then(() => undefined), new Promise<void>((resolve) => setTimeout(resolve, timeoutMs))]);
};

// 变量：INACTIVITY_SCROLL_DELAY_MS | 含义：列表无操作后自动回滚到选中项的延迟毫秒数 | 类型：number | 作用域：模块级
// 本行目的：定义鼠标离开列表后触发自动滚动的延迟时间。
const INACTIVITY_SCROLL_DELAY_MS = 5000;
// 变量：MIN_PROGRESS_DURATION_MS | 含义：最小进度展示时长 | 类型：number | 作用域：模块级
// 本行目的：保证进度动画不会过快结束，提升体感稳定性。
const MIN_PROGRESS_DURATION_MS = 100_000;
// 变量：CATCHUP_DURATION_MS | 含义：最终追赶到100%的过渡时长 | 类型：number | 作用域：模块级
// 本行目的：定义完成阶段进度条平滑补齐时长。
const CATCHUP_DURATION_MS = 5_000;
// 变量：MAX_PROGRESS_BEFORE_FINISH | 含义：加载中进度上限（未完成前） | 类型：number | 作用域：模块级
// 本行目的：在真正完成前把进度上限锁在 99%。
const MAX_PROGRESS_BEFORE_FINISH = 99;

// 变量：OFFICIAL_TYPE_OPTIONS | 含义：官方预定义题材筛选项 | 类型：string[] | 作用域：模块级
// 本行目的：定义类型筛选可选值（固定字典）。
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

// 变量：OFFICIAL_REGION_OPTIONS | 含义：官方预定义地区筛选项 | 类型：string[] | 作用域：模块级
// 本行目的：定义地区筛选可选值（固定字典）。
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

// 变量：OFFICIAL_AUDIENCE_OPTIONS | 含义：官方预定义受众筛选项 | 类型：string[] | 作用域：模块级
// 本行目的：定义受众筛选可选值（固定字典）。
const OFFICIAL_AUDIENCE_OPTIONS = ["BL", "GL", "子供向", "女性向", "少女向", "少年向", "青年向"];

// 变量：allFilterValue | 含义：“全选”选项的内部标记值 | 类型：string | 作用域：模块级
// 本行目的：定义筛选器通用“全部”值，统一多选逻辑。
const allFilterValue = "__all__";

// 变量：useQueryPage | 含义：查询页组合式函数入口 | 类型：(options: { ensureStatus: (id: number) => ItemStatus; allFilterLabel: () => string }) => UseQueryPageReturn | 作用域：模块级
/** 函数：useQueryPage | 输入：状态确保函数与“全部”文案函数 | 输出：查询页所需状态与行为集合 | 可能失败：后端调用失败、网络失败、数据格式异常 */
// 本行目的：导出查询页面核心组合逻辑。
export const useQueryPage = (options: { ensureStatus: (id: number) => ItemStatus; allFilterLabel: () => string }) => {
    // 变量：year | 含义：当前选择年份 | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    // 本行目的：初始化年份选择为当前年。
    const year = ref<number | null>(new Date().getFullYear());
    // 变量：month | 含义：当前选择季度起始月 | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    // 本行目的：初始化季度选择为当前月对应季度。
    const month = ref<number | null>(getSeasonStartMonth(currentMonth));

    // 变量：seasonOptions | 含义：按年份动态裁剪的季度选项 | 类型：ComputedRef<Array<{label:string;value:number}>> | 作用域：useQueryPage 内部
    /** 函数：seasonOptions | 输入：无 | 输出：当前可用季度选项列表 | 可能失败：无（纯计算） */
    // 本行目的：计算年份对应可选季度（当年只到当前月所在季度）。
    const seasonOptions = computed(() => {
        // 本行目的：年份空值时返回全年四季。
        if (!year.value) return seasonOptionsAll;
        // 本行目的：当年时过滤未来季度。
        if (year.value === currentYear) {
            return seasonOptionsAll.filter((option) => option.value <= currentMonth);
        }
        // 本行目的：历史年份返回全年四季。
        return seasonOptionsAll;
    });

    // 本行目的：监听年份和可选季度变化，修正 month 到合法值。
    watch([year, seasonOptions], () => {
        // 变量：optionsList | 含义：当前有效季度列表 | 类型：Array<{label:string;value:number}> | 作用域：watch 回调内
        // 本行目的：读取当前季度选项列表。
        const optionsList = seasonOptions.value;

        // 本行目的：无可选项时清空月份并结束。
        if (!optionsList.length) {
            month.value = null;
            return;
        }

        // 本行目的：若当前月份不在可选项中，回退到最后一个可选季度。
        if (!optionsList.some((option) => option.value === month.value)) {
            month.value = optionsList[optionsList.length - 1].value;
        }
    });

    // 变量：loading | 含义：主查询加载状态 | 类型：Ref<boolean> | 作用域：useQueryPage 内部
    const loading = ref(false);
    // 变量：results | 含义：查询结果列表 | 类型：Ref<MonthAnime[]> | 作用域：useQueryPage 内部
    const results = ref<MonthAnime[]>([]);
    // 变量：resultUrl | 含义：数据来源地址 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const resultUrl = ref("");
    // 变量：resultFetchedAt | 含义：数据抓取时间 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const resultFetchedAt = ref("");
    // 变量：errorMessage | 含义：主查询错误信息 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const errorMessage = ref("");
    // 变量：hasQueried | 含义：是否已经发起过查询 | 类型：Ref<boolean> | 作用域：useQueryPage 内部
    const hasQueried = ref(false);
    // 变量：progress | 含义：查询进度百分比（0~100） | 类型：Ref<number> | 作用域：useQueryPage 内部
    const progress = ref(0);
    // 变量：progressStartAt | 含义：进度开始时间戳 | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    const progressStartAt = ref<number | null>(null);
    // 变量：selected | 含义：当前选中条目 | 类型：Ref<MonthAnime | null> | 作用域：useQueryPage 内部
    const selected = ref<MonthAnime | null>(null);
    // 变量：originLoadingId | 含义：原作信息加载中的条目ID | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    const originLoadingId = ref<number | null>(null);
    // 变量：originError | 含义：原作信息加载错误 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const originError = ref("");
    // 变量：airedLoadingId | 含义：已播信息加载中的条目ID | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    const airedLoadingId = ref<number | null>(null);
    // 变量：airedError | 含义：已播信息加载错误 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const airedError = ref("");
    // 变量：showStaffModal | 含义：staff 弹窗显示状态 | 类型：Ref<boolean> | 作用域：useQueryPage 内部
    const showStaffModal = ref(false);
    // 变量：queryPanelRef | 含义：查询区容器引用 | 类型：Ref<HTMLElement | null> | 作用域：useQueryPage 内部
    const queryPanelRef = ref<HTMLElement | null>(null);
    // 变量：summaryPanelRef | 含义：摘要区容器引用 | 类型：Ref<HTMLElement | null> | 作用域：useQueryPage 内部
    const summaryPanelRef = ref<HTMLElement | null>(null);
    // 变量：filterPanelRef | 含义：筛选区容器引用 | 类型：Ref<HTMLElement | null> | 作用域：useQueryPage 内部
    const filterPanelRef = ref<HTMLElement | null>(null);
    // 变量：resultListRef | 含义：结果列表容器引用 | 类型：Ref<HTMLElement | null> | 作用域：useQueryPage 内部
    const resultListRef = ref<HTMLElement | null>(null);
    // 变量：listItemRefs | 含义：列表项 DOM 引用表（按 id 索引） | 类型：Ref<Record<number, HTMLElement | null>> | 作用域：useQueryPage 内部
    const listItemRefs = ref<Record<number, HTMLElement | null>>({});
    // 变量：staffLoadingId | 含义：staff 加载中的条目ID | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    const staffLoadingId = ref<number | null>(null);
    // 变量：staffError | 含义：staff 加载错误 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const staffError = ref("");
    // 变量：staffCache | 含义：staff 缓存（按条目 id） | 类型：Ref<Record<number, StaffGroup[]>> | 作用域：useQueryPage 内部
    const staffCache = ref<Record<number, StaffGroup[]>>({});
    // 变量：characterLoadingId | 含义：角色加载中的条目ID | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    const characterLoadingId = ref<number | null>(null);
    // 变量：characterError | 含义：角色加载错误 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const characterError = ref("");
    // 变量：characterCache | 含义：角色缓存（按条目 id） | 类型：Ref<Record<number, CharacterLink[]>> | 作用域：useQueryPage 内部
    const characterCache = ref<Record<number, CharacterLink[]>>({});
    // 变量：summaryLoadingId | 含义：简介翻译加载中的条目ID | 类型：Ref<number | null> | 作用域：useQueryPage 内部
    const summaryLoadingId = ref<number | null>(null);
    // 变量：summaryError | 含义：简介翻译错误 | 类型：Ref<string> | 作用域：useQueryPage 内部
    const summaryError = ref("");

    // 变量：selectedStaff | 含义：当前选中条目的 staff 分组数据 | 类型：ComputedRef<StaffGroup[]> | 作用域：useQueryPage 内部
    /** 函数：selectedStaff | 输入：无 | 输出：当前条目的 staff 列表 | 可能失败：无（纯派生） */
    // 本行目的：从缓存中读取当前选中项的 staff 数据。
    const selectedStaff = computed(() => {
        // 变量：id | 含义：当前选中条目的 id | 类型：number | undefined | 作用域：computed 回调内
        // 本行目的：读取选中条目 id。
        const id = selected.value?.id;
        // 本行目的：无 id 时返回空数组。
        if (!id) return [] as StaffGroup[];
        // 本行目的：有缓存则返回缓存，没有则返回空数组。
        return staffCache.value[id] || [];
    });

    // 变量：selectedCharacters | 含义：当前选中条目的角色关联数据 | 类型：ComputedRef<CharacterLink[]> | 作用域：useQueryPage 内部
    /** 函数：selectedCharacters | 输入：无 | 输出：当前条目的角色列表 | 可能失败：无（纯派生） */
    // 本行目的：从缓存中读取当前选中项的角色数据。
    const selectedCharacters = computed(() => {
        // 本行目的：读取选中条目 id。
        const id = selected.value?.id;
        // 本行目的：无 id 时返回空数组。
        if (!id) return [] as CharacterLink[];
        // 本行目的：有缓存则返回缓存，没有则返回空数组。
        return characterCache.value[id] || [];
    });

    // 变量：showResults | 含义：结果区域是否可展示 | 类型：ComputedRef<boolean> | 作用域：useQueryPage 内部
    /** 函数：showResults | 输入：无 | 输出：是否展示结果区 | 可能失败：无（纯派生） */
    // 本行目的：只有查询完成且进度100后才显示结果。
    const showResults = computed(() => hasQueried.value && !loading.value && progress.value >= 100);
    // 变量：detailVisible | 含义：详情区域是否可见 | 类型：ComputedRef<boolean> | 作用域：useQueryPage 内部
    /** 函数：detailVisible | 输入：无 | 输出：是否展示详情 | 可能失败：无（纯派生） */
    // 本行目的：结果可见且有选中项时展示详情。
    const detailVisible = computed(() => showResults.value && !!selected.value);

    // 变量：progressTimer | 含义：主进度定时器句柄 | 类型：number | undefined | 作用域：useQueryPage 内部
    let progressTimer: number | undefined;
    // 变量：catchupTimer | 含义：冲刺到100%的定时器句柄 | 类型：number | undefined | 作用域：useQueryPage 内部
    let catchupTimer: number | undefined;
    // 变量：listMouseLeaveTimer | 含义：鼠标离开后自动滚动延时器句柄 | 类型：number | undefined | 作用域：useQueryPage 内部
    let listMouseLeaveTimer: number | undefined;

    // 变量：dataCache | 含义：季度数据缓存（按 年份-季节键） | 类型：Map<string, SeasonResponse> | 作用域：useQueryPage 内部
    // 本行目的：缓存季度结果避免重复请求。
    const dataCache = new Map<string, SeasonResponse>();
    // 变量：queryToken | 含义：查询请求版本号，用于并发请求防串扰 | 类型：Ref<number> | 作用域：useQueryPage 内部
    // 本行目的：通过递增 token 丢弃过期查询结果。
    const queryToken = ref(0);

    // 变量：setListItemRef | 含义：维护列表项 DOM 引用表 | 类型：(item: MonthAnime, el: HTMLElement | null) => void | 作用域：useQueryPage 内部
    /** 函数：setListItemRef | 输入：条目与对应元素 | 输出：无（更新引用表） | 可能失败：无（纯内存操作） */
    // 本行目的：定义列表项 DOM 引用登记函数。
    const setListItemRef = (item: MonthAnime, el: HTMLElement | null) => {
        // 本行目的：无 id 条目不参与引用管理。
        if (!item.id) return;

        // 本行目的：元素存在则写入引用表。
        if (el) {
            listItemRefs.value[item.id] = el;
        } else {
            // 本行目的：元素卸载时从引用表删除。
            delete listItemRefs.value[item.id];
        }
    };

    // 变量：clearListMouseLeaveTimer | 含义：清理列表离开延时器 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：clearListMouseLeaveTimer | 输入：无 | 输出：无（清理定时器） | 可能失败：无（浏览器 API） */
    // 本行目的：统一清理鼠标离开触发的定时器。
    const clearListMouseLeaveTimer = () => {
        // 本行目的：仅在定时器存在时执行清理。
        if (listMouseLeaveTimer) {
            // 本行目的：清除延时器。
            window.clearTimeout(listMouseLeaveTimer);
            // 本行目的：重置句柄为空。
            listMouseLeaveTimer = undefined;
        }
    };

    // 变量：findScrollParent | 含义：查找目标元素最近的可滚动父容器 | 类型：(el: HTMLElement | null) => HTMLElement | null | 作用域：useQueryPage 内部
    /** 函数：findScrollParent | 输入：目标元素 | 输出：可滚动父元素或空 | 可能失败：无（DOM 查询） */
    // 本行目的：定义可滚动父容器查找函数。
    const findScrollParent = (el: HTMLElement | null): HTMLElement | null => {
        // 变量：node | 含义：向上遍历的当前节点 | 类型：HTMLElement | null | 作用域：findScrollParent 内部
        // 本行目的：从父元素开始逐级向上查找。
        let node: HTMLElement | null = el?.parentElement || null;

        while (node) {
            // 本行目的：读取当前节点计算样式。
            const style = window.getComputedStyle(node);
            // 本行目的：提取垂直滚动策略。
            const overflowY = style.overflowY;

            // 本行目的：命中可滚动容器时直接返回。
            if (overflowY === "auto" || overflowY === "scroll") {
                return node;
            }

            // 本行目的：继续向上层父节点遍历。
            node = node.parentElement;
        }

        // 本行目的：未找到时返回空。
        return null;
    };

    // 变量：scrollToSelectedItem | 含义：把列表滚动到当前选中条目 | 类型：(behavior?: ScrollBehavior) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：scrollToSelectedItem | 输入：滚动行为（smooth/auto） | 输出：无（执行滚动） | 可能失败：DOM 未就绪或目标不存在 */
    // 本行目的：定义选中项居中滚动函数。
    const scrollToSelectedItem = async (behavior: ScrollBehavior = "smooth") => {
        // 本行目的：详情不可见时不执行滚动。
        if (!detailVisible.value) return;

        // 变量：id | 含义：当前选中条目 id | 类型：number | undefined | 作用域：scrollToSelectedItem 内部
        // 本行目的：读取当前选中条目 id。
        const id = selected.value?.id;
        // 本行目的：无选中 id 时结束。
        if (!id) return;

        // 本行目的：等待下一帧 DOM 更新后再定位。
        await nextTick();

        // 变量：target | 含义：目标条目元素 | 类型：HTMLElement | null | 作用域：scrollToSelectedItem 内部
        // 本行目的：从引用表取出目标元素。
        const target = listItemRefs.value[id];
        // 本行目的：目标不存在时结束。
        if (!target) return;

        // 变量：container | 含义：目标滚动容器（优先最近可滚动父级） | 类型：HTMLElement | null | 作用域：scrollToSelectedItem 内部
        // 本行目的：确定滚动容器。
        const container = findScrollParent(target) || resultListRef.value;
        if (container) {
            // 本行目的：读取容器位置矩形。
            const containerRect = container.getBoundingClientRect();
            // 本行目的：读取目标元素位置矩形。
            const targetRect = target.getBoundingClientRect();
            // 本行目的：计算目标中心点在容器滚动坐标系中的位置。
            const targetCenter = targetRect.top - containerRect.top + container.scrollTop + targetRect.height / 2;
            // 本行目的：计算让目标居中的 scrollTop 值。
            const nextScrollTop = targetCenter - container.clientHeight / 2;
            // 本行目的：计算容器最大可滚动距离。
            const maxScrollTop = container.scrollHeight - container.clientHeight;
            // 本行目的：把滚动值限制到合法区间。
            const clamped = Math.max(0, Math.min(nextScrollTop, maxScrollTop));
            // 本行目的：执行容器滚动。
            container.scrollTo({ top: clamped, behavior });
            return;
        }

        // 本行目的：无容器时回退到原生 scrollIntoView。
        target.scrollIntoView({ behavior, block: "center", inline: "center" });
    };

    // 变量：handleListMouseEnter | 含义：列表鼠标进入事件处理 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：handleListMouseEnter | 输入：无 | 输出：无（清理定时器） | 可能失败：无 */
    // 本行目的：鼠标进入时清理离开延时滚动。
    const handleListMouseEnter = () => {
        clearListMouseLeaveTimer();
    };

    // 变量：handleListMouseLeave | 含义：列表鼠标离开事件处理 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：handleListMouseLeave | 输入：无 | 输出：无（延时触发滚动） | 可能失败：无 */
    // 本行目的：鼠标离开后延迟把列表滚回选中项。
    const handleListMouseLeave = () => {
        // 本行目的：详情不可见时不触发自动回滚。
        if (!detailVisible.value) return;
        // 本行目的：先清理旧定时器避免重复。
        clearListMouseLeaveTimer();
        // 本行目的：创建新的延时回滚任务。
        listMouseLeaveTimer = window.setTimeout(() => {
            // 本行目的：延时执行滚动到选中项。
            void scrollToSelectedItem();
        }, INACTIVITY_SCROLL_DELAY_MS);
    };

    // 本行目的：组件卸载前清理所有定时器，防止内存泄漏。
    onBeforeUnmount(() => {
        // 本行目的：卸载时清理离开延时器。
        clearListMouseLeaveTimer();
        // 本行目的：卸载时清理主进度定时器。
        if (progressTimer) window.clearInterval(progressTimer);
        // 本行目的：卸载时清理追赶定时器。
        if (catchupTimer) window.clearInterval(catchupTimer);
    });

    // 变量：monthFilter | 含义：月份筛选多选值 | 类型：Ref<Array<number | string>> | 作用域：useQueryPage 内部
    const monthFilter = ref<Array<number | string>>([]);
    // 变量：typeFilter | 含义：题材筛选多选值 | 类型：Ref<string[]> | 作用域：useQueryPage 内部
    const typeFilter = ref<string[]>([]);
    // 变量：regionFilter | 含义：地区筛选多选值 | 类型：Ref<string[]> | 作用域：useQueryPage 内部
    const regionFilter = ref<string[]>([]);
    // 变量：audienceFilter | 含义：受众筛选多选值 | 类型：Ref<string[]> | 作用域：useQueryPage 内部
    const audienceFilter = ref<string[]>([]);
    // 变量：filterLoading | 含义：筛选元数据加载状态 | 类型：Ref<boolean> | 作用域：useQueryPage 内部
    const filterLoading = ref(false);
    // 变量：filtersInitialized | 含义：默认筛选是否已初始化 | 类型：Ref<boolean> | 作用域：useQueryPage 内部
    const filtersInitialized = ref(false);

    // 变量：monthFilterOptions | 含义：月份筛选选项（含“全部”） | 类型：ComputedRef<Array<{label:string;value:number|string}>> | 作用域：useQueryPage 内部
    /** 函数：monthFilterOptions | 输入：无 | 输出：月份筛选选项列表 | 可能失败：无（纯计算） */
    // 本行目的：根据结果动态生成月份筛选项。
    const monthFilterOptions = computed(() => {
        // 变量：values | 含义：去重后的月份集合 | 类型：Set<number> | 作用域：computed 回调内
        // 本行目的：创建月份去重集合。
        const values = new Set<number>();

        // 本行目的：遍历结果收集有效月份。
        results.value.forEach((item) => {
            if (typeof item.month === "number") values.add(item.month);
        });

        // 变量：monthOptions | 含义：排序后的月份选项 | 类型：Array<{label:string;value:number}> | 作用域：computed 回调内
        // 本行目的：按升序转成“x月”选项。
        const monthOptions = Array.from(values)
            .sort((a, b) => a - b)
            .map((value) => ({ label: `${value}月`, value }));

        // 本行目的：在首位插入“全部”选项并返回。
        return [{ label: options.allFilterLabel(), value: allFilterValue }, ...monthOptions];
    });

    // 变量：typeOptions | 含义：题材筛选选项（含“全部”） | 类型：ComputedRef<Array<{label:string;value:string}>> | 作用域：useQueryPage 内部
    /** 函数：typeOptions | 输入：无 | 输出：题材筛选选项 | 可能失败：无（纯计算） */
    const typeOptions = computed(() => [
        { label: options.allFilterLabel(), value: allFilterValue },
        ...OFFICIAL_TYPE_OPTIONS.map((value) => ({ label: value, value })),
    ]);

    // 变量：regionOptions | 含义：地区筛选选项（含“全部”） | 类型：ComputedRef<Array<{label:string;value:string}>> | 作用域：useQueryPage 内部
    /** 函数：regionOptions | 输入：无 | 输出：地区筛选选项 | 可能失败：无（纯计算） */
    const regionOptions = computed(() => [
        { label: options.allFilterLabel(), value: allFilterValue },
        ...OFFICIAL_REGION_OPTIONS.map((value) => ({ label: value, value })),
    ]);

    // 变量：audienceOptions | 含义：受众筛选选项（含“全部”） | 类型：ComputedRef<Array<{label:string;value:string}>> | 作用域：useQueryPage 内部
    /** 函数：audienceOptions | 输入：无 | 输出：受众筛选选项 | 可能失败：无（纯计算） */
    const audienceOptions = computed(() => [
        { label: options.allFilterLabel(), value: allFilterValue },
        ...OFFICIAL_AUDIENCE_OPTIONS.map((value) => ({ label: value, value })),
    ]);

    // 变量：applySelectAllBehavior | 含义：统一处理多选筛选中的“全选”行为 | 类型：<T extends string | number>(next: T[], optionsList: { value: T }[], prev: T[]) => T[] | 作用域：useQueryPage 内部
    /** 函数：applySelectAllBehavior | 输入：新值、选项列表、旧值 | 输出：规范化后的选择结果 | 可能失败：无（纯数组逻辑） */
    // 本行目的：定义“全部 + 单项”联动规则，避免选择状态冲突。
    const applySelectAllBehavior = <T extends string | number>(next: T[], optionsList: { value: T }[], prev: T[]): T[] => {
        // 变量：allValue | 含义：当前泛型下的“全部”值 | 类型：T | 作用域：函数内
        // 本行目的：把 allFilterValue 转成泛型 T 参与比较。
        const allValue = allFilterValue as unknown as T;
        // 变量：real | 含义：除“全部”外的真实可选值列表 | 类型：T[] | 作用域：函数内
        // 本行目的：提取真实选项集合。
        const real = optionsList.map((o) => o.value).filter((v) => v !== allValue);

        if (!real.length) return [] as T[];

        // 本行目的：构造新旧选择集合便于高效判断。
        const nextSet = new Set(next);
        const prevSet = new Set(prev);
        // 本行目的：判断新旧值是否包含“全部”。
        const hasAll = nextSet.has(allValue);
        const prevHasAll = prevSet.has(allValue);
        // 本行目的：提取新选择中真实选中的值。
        const nextReal = real.filter((value) => nextSet.has(value));
        // 本行目的：判断是否已选中全部真实值。
        const allRealSelected = nextReal.length === real.length;

        // 本行目的：从“全部”切换到非全选时，清空筛选。
        if (prevHasAll && !hasAll) {
            return [] as T[];
        }

        // 本行目的：已全选后取消部分项时，仅保留剩余真实项。
        if (prevHasAll && !allRealSelected) {
            return nextReal;
        }

        // 本行目的：显式点“全部”或真实项全选时，返回“全部+所有真实项”。
        if (hasAll || allRealSelected) {
            return [allValue, ...real];
        }

        // 本行目的：默认返回当前真实选项。
        return nextReal;
    };

    // 变量：matchesTextFilter | 含义：判断文本数组字段是否命中筛选器 | 类型：(values: string[] | undefined, selectedList: string[], total: number) => boolean | 作用域：useQueryPage 内部
    /** 函数：matchesTextFilter | 输入：条目值数组、已选列表、总可选数 | 输出：是否匹配 | 可能失败：无（纯判断） */
    // 本行目的：封装类型/地区/受众的通用匹配逻辑。
    const matchesTextFilter = (values: string[] | undefined, selectedList: string[], total: number) => {
        // 本行目的：未筛选、全选或全覆盖时直接匹配通过。
        if (!selectedList.length || selectedList.includes(allFilterValue) || selectedList.length >= total) return true;
        // 本行目的：条目无值则不匹配。
        if (!values || !values.length) return false;
        // 本行目的：任一值命中即通过。
        return values.some((value) => selectedList.includes(value));
    };

    // 变量：filteredResults | 含义：按多维筛选后的结果列表 | 类型：ComputedRef<MonthAnime[]> | 作用域：useQueryPage 内部
    /** 函数：filteredResults | 输入：无 | 输出：筛选后的结果数组 | 可能失败：无（纯计算） */
    // 本行目的：执行月份、题材、地区、受众四维过滤。
    const filteredResults = computed(() => {
        return results.value.filter((item) => {
            // 本行目的：判断月份筛选是否处于“全匹配”状态。
            const monthAll =
                !monthFilter.value.length ||
                monthFilter.value.includes(allFilterValue) ||
                monthFilter.value.length >= monthFilterOptions.value.length - 1;

            // 本行目的：计算当前条目月份是否命中。
            const monthMatch = monthAll || (item.month && monthFilter.value.includes(item.month));

            // 本行目的：月份不匹配直接过滤掉。
            if (!monthMatch) return false;
            // 本行目的：题材不匹配时过滤掉。
            if (!matchesTextFilter(item.types, typeFilter.value, OFFICIAL_TYPE_OPTIONS.length)) return false;
            // 本行目的：地区不匹配时过滤掉。
            if (!matchesTextFilter(item.regions, regionFilter.value, OFFICIAL_REGION_OPTIONS.length)) return false;
            // 本行目的：受众不匹配时过滤掉。
            if (!matchesTextFilter(item.audiences, audienceFilter.value, OFFICIAL_AUDIENCE_OPTIONS.length)) return false;

            // 本行目的：通过全部过滤条件时保留条目。
            return true;
        });
    });

    // 变量：resultCount | 含义：筛选后结果数量 | 类型：ComputedRef<number> | 作用域：useQueryPage 内部
    /** 函数：resultCount | 输入：无 | 输出：筛选结果数量 | 可能失败：无（纯派生） */
    const resultCount = computed(() => filteredResults.value.length);

    // 变量：updateDefaultFilters | 含义：初始化默认筛选状态 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：updateDefaultFilters | 输入：无 | 输出：无（设置筛选初值） | 可能失败：无 */
    // 本行目的：首次可用时重置筛选并标记已初始化。
    const updateDefaultFilters = () => {
        // 本行目的：已初始化过时直接返回。
        if (filtersInitialized.value) return;
        // 本行目的：初始化月份筛选为空（表示不过滤）。
        monthFilter.value = [];
        // 本行目的：初始化题材筛选为空。
        typeFilter.value = [];
        // 本行目的：初始化地区筛选为空。
        regionFilter.value = [];
        // 本行目的：初始化受众筛选为空。
        audienceFilter.value = [];
        // 本行目的：标记筛选已完成初始化。
        filtersInitialized.value = true;
    };

    // 本行目的：当月份选项变化时触发默认筛选初始化。
    watch(monthFilterOptions, () => {
        // 本行目的：执行默认筛选初始化。
        updateDefaultFilters();
    });

    // 变量：getFilterLabel | 含义：根据选中数量返回筛选标签键 | 类型：(selectedCount: number, totalCount: number) => string | 作用域：useQueryPage 内部
    /** 函数：getFilterLabel | 输入：已选数量与总数量 | 输出：i18n 标签 key | 可能失败：无（纯判断） */
    // 本行目的：统一筛选按钮“未选/全选/部分”的文案键。
    const getFilterLabel = (selectedCount: number, totalCount: number) => {
        // 本行目的：0个选中返回“未选择”标签。
        if (selectedCount === 0) return "query.filter.label.none";
        // 本行目的：达到总数返回“全部”标签。
        if (selectedCount >= totalCount) return "query.filter.label.all";
        // 本行目的：其余返回“部分”标签。
        return "query.filter.label.partial";
    };

    // 变量：monthFilterLabel | 含义：月份筛选标签键 | 类型：ComputedRef<string> | 作用域：useQueryPage 内部
    const monthFilterLabel = computed(() =>
        getFilterLabel(monthFilter.value.filter((v) => v !== allFilterValue).length, Math.max(monthFilterOptions.value.length - 1, 0))
    );

    // 变量：typeFilterLabel | 含义：题材筛选标签键 | 类型：ComputedRef<string> | 作用域：useQueryPage 内部
    const typeFilterLabel = computed(() => getFilterLabel(typeFilter.value.filter((v) => v !== allFilterValue).length, OFFICIAL_TYPE_OPTIONS.length));

    // 变量：regionFilterLabel | 含义：地区筛选标签键 | 类型：ComputedRef<string> | 作用域：useQueryPage 内部
    const regionFilterLabel = computed(() =>
        getFilterLabel(regionFilter.value.filter((v) => v !== allFilterValue).length, OFFICIAL_REGION_OPTIONS.length)
    );

    // 变量：audienceFilterLabel | 含义：受众筛选标签键 | 类型：ComputedRef<string> | 作用域：useQueryPage 内部
    const audienceFilterLabel = computed(() =>
        getFilterLabel(audienceFilter.value.filter((v) => v !== allFilterValue).length, OFFICIAL_AUDIENCE_OPTIONS.length)
    );

    // 变量：handleMonthFilterChange | 含义：处理月份筛选变更 | 类型：(value: Array<number | string>) => void | 作用域：useQueryPage 内部
    /** 函数：handleMonthFilterChange | 输入：新筛选值数组 | 输出：无（更新筛选与选中项） | 可能失败：无 */
    const handleMonthFilterChange = (value: Array<number | string>) => {
        // 本行目的：读取旧值用于全选行为计算。
        const prev = monthFilter.value;
        // 本行目的：计算规范化后的新值。
        const next = applySelectAllBehavior(value, monthFilterOptions.value, prev);
        // 本行目的：写回月份筛选值。
        monthFilter.value = next;
        // 本行目的：筛选变化后清空当前选中项。
        selected.value = null;
    };

    // 变量：handleTypeFilterChange | 含义：处理题材筛选变更 | 类型：(value: string[]) => void | 作用域：useQueryPage 内部
    /** 函数：handleTypeFilterChange | 输入：新筛选值数组 | 输出：无 | 可能失败：无 */
    const handleTypeFilterChange = (value: string[]) => {
        // 本行目的：读取旧值用于全选行为计算。
        const prev = typeFilter.value;
        // 本行目的：计算规范化后的新值。
        const next = applySelectAllBehavior(value, typeOptions.value, prev);
        // 本行目的：写回题材筛选值。
        typeFilter.value = next;
        // 本行目的：筛选变化后清空当前选中项。
        selected.value = null;
    };

    // 变量：handleRegionFilterChange | 含义：处理地区筛选变更 | 类型：(value: string[]) => void | 作用域：useQueryPage 内部
    /** 函数：handleRegionFilterChange | 输入：新筛选值数组 | 输出：无 | 可能失败：无 */
    const handleRegionFilterChange = (value: string[]) => {
        // 本行目的：读取旧值用于全选行为计算。
        const prev = regionFilter.value;
        // 本行目的：计算规范化后的新值。
        const next = applySelectAllBehavior(value, regionOptions.value, prev);
        // 本行目的：写回地区筛选值。
        regionFilter.value = next;
        // 本行目的：筛选变化后清空当前选中项。
        selected.value = null;
    };

    // 变量：handleAudienceFilterChange | 含义：处理受众筛选变更 | 类型：(value: string[]) => void | 作用域：useQueryPage 内部
    /** 函数：handleAudienceFilterChange | 输入：新筛选值数组 | 输出：无 | 可能失败：无 */
    const handleAudienceFilterChange = (value: string[]) => {
        // 本行目的：读取旧值用于全选行为计算。
        const prev = audienceFilter.value;
        // 本行目的：计算规范化后的新值。
        const next = applySelectAllBehavior(value, audienceOptions.value, prev);
        // 本行目的：写回受众筛选值。
        audienceFilter.value = next;
        // 本行目的：筛选变化后清空当前选中项。
        selected.value = null;
    };

    // 变量：loadSeasonData | 含义：加载季度数据并进行缓存 | 类型：(yearValue: number, seasonMonth: number) => Promise<SeasonResponse> | 作用域：useQueryPage 内部
    /** 函数：loadSeasonData | 输入：年份与季度起始月 | 输出：季度响应数据 | 可能失败：季节无效、接口失败、返回格式错误 */
    // 本行目的：定义季度数据加载函数，优先读取缓存。
    const loadSeasonData = async (yearValue: number, seasonMonth: number) => {
        // 本行目的：将月份转换为后端季节键。
        const seasonKey = seasonKeyFromMonth(seasonMonth);
        if (!seasonKey) throw new Error("季节选择无效");

        // 变量：cacheKey | 含义：季度缓存键 | 类型：string | 作用域：loadSeasonData 内部
        // 本行目的：拼接缓存键。
        const cacheKey = `${yearValue}-${seasonKey}`;
        // 本行目的：命中缓存时直接返回。
        if (dataCache.has(cacheKey)) return dataCache.get(cacheKey)!;

        // 本行目的：调用后端获取季度数据。
        const payload = await invoke<SeasonResponse>("get_season_subjects", {
            // 本行目的：传入查询年份。
            year: yearValue,
            // 本行目的：传入查询季节键。
            season: seasonKey,
        });

        // 本行目的：校验返回结构，防止后续处理报错。
        if (!payload || !Array.isArray(payload.months)) {
            throw new Error("季度数据格式不正确");
        }

        // 本行目的：写入缓存供后续复用。
        dataCache.set(cacheKey, payload);

        // 本行目的：返回季度数据。
        return payload;
    };

    // 变量：clearProgressTimers | 含义：清理进度相关定时器 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：clearProgressTimers | 输入：无 | 输出：无（清理定时器） | 可能失败：无 */
    const clearProgressTimers = () => {
        // 本行目的：存在主定时器时清理。
        if (progressTimer) {
            // 本行目的：停止主进度定时器。
            window.clearInterval(progressTimer);
            // 本行目的：重置主定时器句柄。
            progressTimer = undefined;
        }

        // 本行目的：存在追赶定时器时清理。
        if (catchupTimer) {
            // 本行目的：停止追赶定时器。
            window.clearInterval(catchupTimer);
            // 本行目的：重置追赶定时器句柄。
            catchupTimer = undefined;
        }
    };

    // 变量：cancelActiveQuery | 含义：取消当前查询并重置状态 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：cancelActiveQuery | 输入：无 | 输出：无（重置查询状态） | 可能失败：无 */
    const cancelActiveQuery = () => {
        // 本行目的：递增 token 使旧请求失效。
        queryToken.value += 1;
        // 本行目的：关闭加载状态。
        loading.value = false;
        // 本行目的：标记当前无有效查询结果。
        hasQueried.value = false;
        // 本行目的：清理进度相关定时器。
        clearProgressTimers();
        // 本行目的：重置进度值。
        progress.value = 0;
    };

    // 变量：startProgress | 含义：启动查询进度动画 | 类型：() => void | 作用域：useQueryPage 内部
    /** 函数：startProgress | 输入：无 | 输出：无（更新进度） | 可能失败：无（定时器逻辑） */
    const startProgress = () => {
        // 本行目的：启动前先清理旧定时器。
        clearProgressTimers();

        // 本行目的：重置进度到 0。
        progress.value = 0;
        // 本行目的：记录进度起始时间。
        progressStartAt.value = Date.now();

        // 本行目的：创建主进度更新定时器。
        progressTimer = window.setInterval(() => {
            // 本行目的：读取起始时间快照。
            const start = progressStartAt.value;
            // 本行目的：无起始时间则跳过本次更新。
            if (!start) return;

            // 本行目的：计算已耗时毫秒。
            const elapsed = Date.now() - start;
            // 本行目的：按耗时计算目标进度并限制到 99。
            const target = Math.min(MAX_PROGRESS_BEFORE_FINISH, Math.floor((elapsed / MIN_PROGRESS_DURATION_MS) * MAX_PROGRESS_BEFORE_FINISH));

            // 本行目的：仅向前推进进度，避免回退。
            if (target > progress.value) {
                // 本行目的：写入新的进度值。
                progress.value = target;
            }

            // 本行目的：到达上限后切换为“保活”定时器。
            if (target >= MAX_PROGRESS_BEFORE_FINISH) {
                // 本行目的：先清理当前定时器。
                clearProgressTimers();
                // 本行目的：创建保活定时器维持99%。
                progressTimer = window.setInterval(() => {
                    // 本行目的：持续保持进度为99%。
                    progress.value = MAX_PROGRESS_BEFORE_FINISH;
                }, 1000);
            }
        }, 120);
    };

    // 变量：finishProgress | 含义：在查询结束时平滑收敛到100% | 类型：() => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：finishProgress | 输入：无 | 输出：无（更新进度到100） | 可能失败：无（定时器逻辑） */
    const finishProgress = async () => {
        // 本行目的：读取本次进度启动时间。
        const startedAt = progressStartAt.value;

        // 本行目的：结束前先清理所有进度定时器。
        clearProgressTimers();

        // 本行目的：若无起始时间则直接置100并返回。
        if (!startedAt) {
            progress.value = 100;
            return;
        }

        // 本行目的：计算已耗时。
        const elapsed = Date.now() - startedAt;

        // 本行目的：已超过最短展示时长则直接完成。
        if (elapsed >= MIN_PROGRESS_DURATION_MS) {
            progress.value = 100;
            return;
        }

        // 本行目的：记录当前进度作为冲刺起点。
        const startValue = progress.value;
        // 本行目的：记录冲刺开始时间。
        const sprintStart = Date.now();

        // 本行目的：创建冲刺定时器平滑到100%。
        catchupTimer = window.setInterval(() => {
            // 本行目的：计算冲刺完成比例。
            const ratio = Math.min(1, (Date.now() - sprintStart) / CATCHUP_DURATION_MS);
            // 本行目的：按比例计算目标进度。
            const target = Math.round(startValue + (100 - startValue) * ratio);

            // 本行目的：写入冲刺进度。
            progress.value = target;

            // 本行目的：冲刺完成后清理定时器。
            if (ratio >= 1) {
                clearProgressTimers();
            }
        }, 50);
    };

    // 变量：handleQuery | 含义：执行季度查询主流程 | 类型：() => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：handleQuery | 输入：无（读取 year/month） | 输出：无（更新查询状态与结果） | 可能失败：请求失败、数据处理失败 */
    const handleQuery = async () => {
        // 本行目的：年份或季度未选择时不发起查询。
        if (!year.value || !month.value) return;

        // 变量：token | 含义：本次查询版本号快照 | 类型：number | 作用域：handleQuery 内部
        // 本行目的：递增并锁定本次请求 token。
        const token = ++queryToken.value;

        // 本行目的：重置并初始化查询态。
        loading.value = true;
        errorMessage.value = "";
        // 本行目的：标记已发起过查询。
        hasQueried.value = true;
        // 本行目的：重置筛选初始化标记。
        filtersInitialized.value = false;
        // 本行目的：重置月份筛选。
        monthFilter.value = [];
        // 本行目的：重置题材筛选。
        typeFilter.value = [];
        // 本行目的：重置地区筛选。
        regionFilter.value = [];
        // 本行目的：重置受众筛选。
        audienceFilter.value = [];

        // 本行目的：启动进度条动画。
        startProgress();

        try {
            // 本行目的：加载季度数据（含缓存）。
            const data = await loadSeasonData(year.value, month.value);

            // 本行目的：若请求已过期则直接退出。
            if (token !== queryToken.value) return;

            // 本行目的：扁平化并排序月份数据，生成展示列表。
            const merged = data.months
                .flatMap((item) => item.list.map((entry) => ({ ...entry, month: item.month })))
                .sort((a, b) => {
                    // 本行目的：先按月份排序。
                    const monthDiff = (a.month ?? 0) - (b.month ?? 0);
                    // 本行目的：月份不同时直接返回月份差。
                    if (monthDiff !== 0) return monthDiff;
                    // 本行目的：月份相同按日期字符串排序。
                    return (a.date || "").localeCompare(b.date || "");
                });

            // 本行目的：再次校验 token，防止异步过程中状态串扰。
            if (token !== queryToken.value) return;

            // 本行目的：写入查询结果与来源信息。
            results.value = merged;
            resultUrl.value = data.source;
            // 本行目的：记录结果抓取时间。
            resultFetchedAt.value = data.fetchedAt;
            // 本行目的：每次新查询后清空选中项。
            selected.value = null;
            // 本行目的：默认月份筛选为全选。
            monthFilter.value = monthFilterOptions.value.map((option) => option.value);

            // 本行目的：有结果时预加载封面图，优化首次浏览体验。
            if (merged.length > 0) {
                await preloadImages(merged);
            }

            // 本行目的：有结果时并行加载筛选元数据。
            if (merged.length > 0) {
                await loadFiltersForResults(merged);
            }
        } catch (error) {
            // 本行目的：仅在当前 token 有效时写入错误，避免覆盖新请求状态。
            if (token === queryToken.value) {
                errorMessage.value = String(error);
            }
        } finally {
            // 本行目的：仅在当前 token 有效时收尾并完成进度。
            if (token === queryToken.value) {
                loading.value = false;
                // 本行目的：等待进度条自然完成到100%。
                await finishProgress();
            }
        }
    };

    // 本行目的：监听年份/季度变更，若正在请求则取消旧请求。
    watch([year, month], () => {
        if (loading.value) {
            // 本行目的：参数变化时取消当前进行中的请求。
            cancelActiveQuery();
        }
    });

    // 变量：loadFiltersForResults | 含义：为结果项加载类型/地区/受众筛选元数据 | 类型：(items: MonthAnime[]) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：loadFiltersForResults | 输入：结果条目数组 | 输出：无（回填条目筛选字段） | 可能失败：单项接口失败（已兜底为空数组） */
    const loadFiltersForResults = async (items: MonthAnime[]) => {
        // 本行目的：开始筛选元数据加载。
        filterLoading.value = true;

        // 变量：queue | 含义：待处理队列副本 | 类型：MonthAnime[] | 作用域：loadFiltersForResults 内部
        const queue = items.slice();

        // 变量：worker | 含义：并发 worker，循环消费队列并回填字段 | 类型：() => Promise<void> | 作用域：loadFiltersForResults 内部
        const worker = async () => {
            // 本行目的：持续消费队列直到为空。
            while (queue.length) {
                // 本行目的：弹出一个待处理条目。
                const item = queue.shift();

                // 本行目的：无效条目直接跳过。
                if (!item?.id) continue;

                try {
                    // 本行目的：获取单条目的筛选元数据。
                    const payload = await invoke<{
                        id: number;
                        types: string[];
                        regions: string[];
                        audiences: string[];
                    }>("get_subject_filters", { id: item.id });

                    // 本行目的：回填题材字段。
                    item.types = payload.types || [];
                    // 本行目的：回填地区字段。
                    item.regions = payload.regions || [];
                    // 本行目的：回填受众字段。
                    item.audiences = payload.audiences || [];
                } catch (_) {
                    // 本行目的：失败时兜底为空数组，保证后续筛选逻辑安全。
                    item.types = item.types || [];
                    item.regions = item.regions || [];
                    item.audiences = item.audiences || [];
                }
            }
        };

        // 变量：concurrency | 含义：并发 worker 数量 | 类型：number | 作用域：loadFiltersForResults 内部
        const concurrency = 6;

        // 本行目的：启动固定并发 worker 处理队列。
        await Promise.all(Array.from({ length: concurrency }, () => worker()));

        // 本行目的：结束筛选加载状态并初始化默认筛选。
        filterLoading.value = false;

        updateDefaultFilters();
    };

    // 变量：handleSelect | 含义：选择条目并触发详情加载 | 类型：(item: MonthAnime) => void | 作用域：useQueryPage 内部
    /** 函数：handleSelect | 输入：选中条目 | 输出：无（更新选中态并触发详情请求） | 可能失败：详情接口失败（各自错误状态承接） */
    const handleSelect = (item: MonthAnime) => {
        // 本行目的：写入当前选中条目。
        selected.value = item;

        // 本行目的：有 id 时触发状态确保逻辑。
        if (item.id) options.ensureStatus(item.id);

        // 本行目的：清空原作错误。
        originError.value = "";
        // 本行目的：清空已播错误。
        airedError.value = "";
        // 本行目的：清空 staff 错误。
        staffError.value = "";
        // 本行目的：清空角色错误。
        characterError.value = "";
        // 本行目的：清空简介错误。
        summaryError.value = "";

        // 本行目的：并发触发各类详情加载（不阻塞 UI）。
        void loadOrigin(item);
        void loadAiredCount(item);
        // 本行目的：并发触发 staff 加载。
        void loadStaff(item);
        // 本行目的：并发触发角色加载。
        void loadCharacters(item);
        // 本行目的：并发触发中文简介加载。
        void loadSummaryCn(item);
    };

    // 变量：handleStaffOpen | 含义：打开 staff 弹窗并确保数据已加载 | 类型：() => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：handleStaffOpen | 输入：无（依赖 selected） | 输出：无（显示弹窗） | 可能失败：staff 接口失败 */
    const handleStaffOpen = async () => {
        // 本行目的：无选中项时不打开 staff。
        if (!selected.value) return;
        // 本行目的：打开 staff 弹窗。
        showStaffModal.value = true;
        // 本行目的：确保当前选中项 staff 已加载。
        await loadStaff(selected.value);
    };

    // 变量：loadOrigin | 含义：加载条目原作信息 | 类型：(item: MonthAnime) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：loadOrigin | 输入：条目对象 | 输出：无（回填 origin） | 可能失败：接口失败 */
    const loadOrigin = async (item: MonthAnime) => {
        // 本行目的：条目无 id 时跳过。
        if (!item?.id) return;
        // 本行目的：已加载过 origin 时跳过。
        if (item.origin !== undefined) return;

        // 本行目的：标记当前条目为 origin 加载中。
        originLoadingId.value = item.id;
        // 本行目的：清空 origin 错误。
        originError.value = "";

        try {
            // 本行目的：请求原作信息。
            const payload = await invoke<{ id: number; origin?: string | null }>("get_subject_origin", {
                id: item.id,
            });
            // 本行目的：写回 origin 字段。
            item.origin = payload.origin ?? "";
        } catch (error) {
            // 本行目的：记录错误文本。
            originError.value = String(error);
            // 本行目的：失败时兜底为空字符串。
            item.origin = "";
        } finally {
            // 本行目的：仅清理当前条目的 loading 标记。
            if (originLoadingId.value === item.id) {
                originLoadingId.value = null;
            }
        }
    };

    // 变量：loadAiredCount | 含义：加载已播集数和总集数 | 类型：(item: MonthAnime) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：loadAiredCount | 输入：条目对象 | 输出：无（回填 airedCount/totalCount） | 可能失败：接口失败 */
    const loadAiredCount = async (item: MonthAnime) => {
        // 本行目的：条目无 id 时跳过。
        if (!item?.id) return;
        // 本行目的：已加载过 airedCount 时跳过。
        if (item.airedCount !== undefined) return;

        // 本行目的：标记当前条目为已播信息加载中。
        airedLoadingId.value = item.id;
        // 本行目的：清空已播错误。
        airedError.value = "";

        try {
            // 本行目的：请求已播信息。
            const payload = await invoke<{ id: number; airedCount?: number | null; totalCount?: number | null }>("get_subject_aired_count", {
                id: item.id,
            });
            // 本行目的：写回已播集数。
            item.airedCount = payload.airedCount ?? 0;
            // 本行目的：写回总集数。
            item.totalCount = payload.totalCount ?? 0;
        } catch (error) {
            // 本行目的：记录错误文本。
            airedError.value = String(error);
            // 本行目的：失败兜底已播集数为0。
            item.airedCount = 0;
            // 本行目的：失败兜底总集数为0。
            item.totalCount = 0;
        } finally {
            // 本行目的：仅清理当前条目的 loading 标记。
            if (airedLoadingId.value === item.id) {
                airedLoadingId.value = null;
            }
        }
    };

    // 变量：loadStaff | 含义：加载 staff 分组并写入缓存 | 类型：(item: MonthAnime) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：loadStaff | 输入：条目对象 | 输出：无（回填 staffCache） | 可能失败：接口失败 */
    const loadStaff = async (item: MonthAnime) => {
        // 本行目的：条目无 id 时跳过。
        if (!item?.id) return;
        // 本行目的：已存在缓存时跳过请求。
        if (staffCache.value[item.id]) return;

        // 本行目的：标记当前条目 staff 加载中。
        staffLoadingId.value = item.id;
        // 本行目的：清空 staff 错误。
        staffError.value = "";

        try {
            // 本行目的：请求 staff 分组数据。
            const payload = await invoke<{ id: number; groups: StaffGroup[] }>("get_subject_staff", {
                id: item.id,
            });
            // 本行目的：写入 staff 缓存。
            staffCache.value = { ...staffCache.value, [item.id]: payload.groups || [] };
        } catch (error) {
            // 本行目的：记录错误文本。
            staffError.value = String(error);
            // 本行目的：失败兜底为该条目空 staff。
            staffCache.value = { ...staffCache.value, [item.id]: [] };
        } finally {
            // 本行目的：仅清理当前条目的 loading 标记。
            if (staffLoadingId.value === item.id) {
                staffLoadingId.value = null;
            }
        }
    };

    // 变量：loadCharacters | 含义：加载角色关联信息并写入缓存 | 类型：(item: MonthAnime) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：loadCharacters | 输入：条目对象 | 输出：无（回填 characterCache） | 可能失败：接口失败 */
    const loadCharacters = async (item: MonthAnime) => {
        // 本行目的：条目无 id 时跳过。
        if (!item?.id) return;
        // 本行目的：已存在缓存时跳过请求。
        if (characterCache.value[item.id]) return;

        // 本行目的：标记当前条目角色加载中。
        characterLoadingId.value = item.id;
        // 本行目的：清空角色错误。
        characterError.value = "";

        try {
            // 本行目的：请求角色关联信息。
            const payload = await invoke<{ id: number; characters: CharacterLink[] }>("get_subject_characters", { id: item.id });
            // 本行目的：写入角色缓存。
            characterCache.value = { ...characterCache.value, [item.id]: payload.characters || [] };
        } catch (error) {
            // 本行目的：记录错误文本。
            characterError.value = String(error);
            // 本行目的：失败兜底为该条目空角色列表。
            characterCache.value = { ...characterCache.value, [item.id]: [] };
        } finally {
            // 本行目的：仅清理当前条目的 loading 标记。
            if (characterLoadingId.value === item.id) {
                characterLoadingId.value = null;
            }
        }
    };

    // 变量：loadSummaryCn | 含义：加载/翻译中文简介并处理失败状态 | 类型：(item: MonthAnime) => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：loadSummaryCn | 输入：条目对象 | 输出：无（回填 summaryCn 与翻译标记） | 可能失败：接口失败、翻译失败 */
    const loadSummaryCn = async (item: MonthAnime) => {
        // 本行目的：条目无 id 时跳过。
        if (!item?.id) return;

        // 变量：shouldRefresh | 含义：是否需要重新翻译摘要 | 类型：boolean | 作用域：loadSummaryCn 内部
        // 本行目的：根据失败标记与旧内容一致性决定是否强制刷新。
        const shouldRefresh =
            // 本行目的：历史翻译失败时强制刷新。
            item.summaryTranslateFailed ||
            (item.summaryCn !== undefined &&
                // 本行目的：尚未翻译成功时继续判断。
                !item.summaryTranslated &&
                // 本行目的：且尚未标记失败时继续判断内容一致性。
                !item.summaryTranslateFailed &&
                // 本行目的：中文摘要与原文一致时视为需刷新。
                (item.summaryCn || "") === (item.summary || ""));

        // 本行目的：已有摘要且无需刷新时直接返回。
        if (item.summaryCn !== undefined && !shouldRefresh) return;

        // 本行目的：标记当前条目简介加载中。
        summaryLoadingId.value = item.id;
        // 本行目的：清空简介错误。
        summaryError.value = "";

        try {
            // 本行目的：请求中文摘要（必要时触发翻译）。
            const payload = await invoke<{ id: number; summary: string; translated: boolean; error?: string | null }>("get_subject_summary_cn", {
                id: item.id,
                // 本行目的：传入原始摘要供后端翻译使用。
                summary: item.summary || "",
            });

            // 本行目的：写回中文摘要文本。
            item.summaryCn = payload.summary;
            // 本行目的：写回是否翻译成功标记。
            item.summaryTranslated = payload.translated;
            // 本行目的：先重置翻译失败标记。
            item.summaryTranslateFailed = false;

            // 本行目的：后端返回 error 时记录并标记失败。
            if (payload.error) {
                summaryError.value = payload.error;
                // 本行目的：出现后端错误时标记翻译失败。
                item.summaryTranslateFailed = true;
            }
        } catch (error) {
            // 本行目的：记录错误文本。
            summaryError.value = String(error);
            // 本行目的：失败时清空中文摘要。
            item.summaryCn = "";
            // 本行目的：失败时标记未翻译。
            item.summaryTranslated = false;
            // 本行目的：失败时标记翻译失败。
            item.summaryTranslateFailed = true;
        } finally {
            // 本行目的：仅清理当前条目的 loading 标记。
            if (summaryLoadingId.value === item.id) {
                summaryLoadingId.value = null;
            }
        }
    };

    // 变量：setQueryPanelHeight | 含义：同步三栏面板高度到 CSS 变量 | 类型：() => Promise<void> | 作用域：useQueryPage 内部
    /** 函数：setQueryPanelHeight | 输入：无 | 输出：无（更新 CSS 变量） | 可能失败：DOM 未挂载 */
    const setQueryPanelHeight = async () => {
        // 本行目的：等待 DOM 更新完成。
        await nextTick();
        // 本行目的：在下一帧读取真实布局高度。
        requestAnimationFrame(() => {
            // 变量：heights | 含义：三个面板的高度集合 | 类型：number[] | 作用域：requestAnimationFrame 回调内
            const heights = [
                queryPanelRef.value?.getBoundingClientRect().height ?? 0,
                summaryPanelRef.value?.getBoundingClientRect().height ?? 0,
                filterPanelRef.value?.getBoundingClientRect().height ?? 0,
            ];
            // 变量：maxHeight | 含义：三栏中的最大高度 | 类型：number | 作用域：requestAnimationFrame 回调内
            const maxHeight = Math.max(...heights);
            // 本行目的：仅在存在有效高度时写入 CSS 变量。
            if (maxHeight > 0) {
                document.documentElement.style.setProperty("--query-panel-height", `${maxHeight}px`);
            }
        });
    };

    // 本行目的：返回查询页组件所需的全部状态与方法。
    return {
        // 本行目的：导出年份筛选状态。
        year,
        // 本行目的：导出季度筛选状态。
        month,
        // 本行目的：导出年份选项。
        yearOptions,
        // 本行目的：导出季度选项。
        seasonOptions,
        // 本行目的：导出季度标签转换函数。
        seasonLabelFromMonth,
        // 本行目的：导出加载状态。
        loading,
        // 本行目的：导出原始结果列表。
        results,
        // 本行目的：导出来源地址。
        resultUrl,
        // 本行目的：导出抓取时间。
        resultFetchedAt,
        // 本行目的：导出错误信息。
        errorMessage,
        // 本行目的：导出是否已查询标记。
        hasQueried,
        // 本行目的：导出进度值。
        progress,
        // 本行目的：导出当前选中条目。
        selected,
        // 本行目的：导出原作加载ID。
        originLoadingId,
        // 本行目的：导出原作错误。
        originError,
        // 本行目的：导出已播加载ID。
        airedLoadingId,
        // 本行目的：导出已播错误。
        airedError,
        // 本行目的：导出 staff 弹窗显示状态。
        showStaffModal,
        // 本行目的：导出查询区引用。
        queryPanelRef,
        // 本行目的：导出结果列表引用。
        resultListRef,
        // 本行目的：导出摘要区引用。
        summaryPanelRef,
        // 本行目的：导出筛选区引用。
        filterPanelRef,
        // 本行目的：导出列表项引用表。
        listItemRefs,
        // 本行目的：导出 staff 加载ID。
        staffLoadingId,
        // 本行目的：导出 staff 错误。
        staffError,
        // 本行目的：导出 staff 缓存。
        staffCache,
        // 本行目的：导出角色加载ID。
        characterLoadingId,
        // 本行目的：导出角色错误。
        characterError,
        // 本行目的：导出角色缓存。
        characterCache,
        // 本行目的：导出简介加载ID。
        summaryLoadingId,
        // 本行目的：导出简介错误。
        summaryError,
        // 本行目的：导出选中条目的 staff 数据。
        selectedStaff,
        // 本行目的：导出选中条目的角色数据。
        selectedCharacters,
        // 本行目的：导出结果展示开关。
        showResults,
        // 本行目的：导出详情展示开关。
        detailVisible,
        // 本行目的：导出列表项引用设置函数。
        setListItemRef,
        // 本行目的：导出列表鼠标进入处理函数。
        handleListMouseEnter,
        // 本行目的：导出列表鼠标离开处理函数。
        handleListMouseLeave,
        // 本行目的：导出滚动到选中项函数。
        scrollToSelectedItem,
        // 本行目的：导出月份筛选值。
        monthFilter,
        // 本行目的：导出题材筛选值。
        typeFilter,
        // 本行目的：导出地区筛选值。
        regionFilter,
        // 本行目的：导出受众筛选值。
        audienceFilter,
        // 本行目的：导出筛选加载状态。
        filterLoading,
        // 本行目的：导出月份筛选选项。
        monthFilterOptions,
        // 本行目的：导出题材筛选选项。
        typeOptions,
        // 本行目的：导出地区筛选选项。
        regionOptions,
        // 本行目的：导出受众筛选选项。
        audienceOptions,
        // 本行目的：导出月份筛选标签键。
        monthFilterLabel,
        // 本行目的：导出题材筛选标签键。
        typeFilterLabel,
        // 本行目的：导出地区筛选标签键。
        regionFilterLabel,
        // 本行目的：导出受众筛选标签键。
        audienceFilterLabel,
        // 本行目的：导出月份筛选变更处理函数。
        handleMonthFilterChange,
        // 本行目的：导出题材筛选变更处理函数。
        handleTypeFilterChange,
        // 本行目的：导出地区筛选变更处理函数。
        handleRegionFilterChange,
        // 本行目的：导出受众筛选变更处理函数。
        handleAudienceFilterChange,
        // 本行目的：导出过滤结果。
        filteredResults,
        // 本行目的：导出结果数量。
        resultCount,
        // 本行目的：导出主查询函数。
        handleQuery,
        // 本行目的：导出条目选择函数。
        handleSelect,
        // 本行目的：导出 staff 弹窗打开函数。
        handleStaffOpen,
        // 本行目的：导出面板高度同步函数。
        setQueryPanelHeight,
    };
};

/** 类型：UseQueryPageReturn | 用途：导出 useQueryPage 返回值类型 | 字段：由 ReturnType 自动推断 */
// 本行目的：导出组合式函数返回类型，便于页面组件复用类型约束。
export type UseQueryPageReturn = ReturnType<typeof useQueryPage>;
