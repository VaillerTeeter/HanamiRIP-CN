/** 文件：useSearchPage.ts | 用途：搜索页关键词组合、别名选择与结果抓取解析逻辑 | 关键对象：useSearchPage, openSearch, parseSearchResults */
// 本行目的：引入 Vue 响应式能力，用于维护搜索页状态。
import { computed, ref } from "vue";
// 本行目的：引入 Tauri invoke，用于调用后端抓取与别名接口。
import { invoke } from "@tauri-apps/api/core";
// 本行目的：引入 Ref 类型，仅用于类型注解。
import type { Ref } from "vue";
// 本行目的：引入搜索模块类型定义。
import type { SearchResult, SearchTerm, LogicOp, LogicOption } from "../types/search";
// 本行目的：引入追踪条目类型，用于从追踪列表生成候选。
import type { TrackedItem } from "../../tracking/types/tracking";

// 变量：NYAA_BASE | 含义：搜索站基础查询前缀 | 类型：string | 作用域：模块级
// 本行目的：定义拼接最终搜索 URL 的固定前缀。
const NYAA_BASE = "https://nyaa.si/?f=0&c=0_0&q=";
// 变量：presetPhrases | 含义：预置关键词列表 | 类型：string[] | 作用域：模块级
// 本行目的：定义常用搜索词快捷按钮集合。
const presetPhrases = ["SubsPlease", "LoliHouse", "内封", "外挂", "480", "720", "1080"];

// 变量：logicOptions | 含义：关键词逻辑运算符候选 | 类型：LogicOption[] | 作用域：模块级
// 本行目的：定义 and/or/not 的下拉选项。
const logicOptions: LogicOption[] = [
    // 本行目的：逻辑与选项。
    { labelKey: "search.logic.and", value: "and" },
    // 本行目的：逻辑或选项。
    { labelKey: "search.logic.or", value: "or" },
    // 本行目的：逻辑非选项。
    { labelKey: "search.logic.not", value: "not" },
];

// 变量：useSearchPage | 含义：搜索页组合式函数入口 | 类型：(options: { trackedItems: Ref<TrackedItem[]>; errorFallback?: () => string }) => UseSearchPageReturn | 作用域：模块级
/** 函数：useSearchPage | 输入：追踪条目引用与可选错误兜底函数 | 输出：搜索页状态与操作集合 | 可能失败：后端调用失败、HTML解析失败 */
// 本行目的：导出搜索页核心状态与行为。
export const useSearchPage = (options: { trackedItems: Ref<TrackedItem[]>; errorFallback?: () => string }) => {
    // 变量：activeLogic | 含义：当前新增关键词默认逻辑操作符 | 类型：Ref<LogicOp> | 作用域：useSearchPage 内部
    // 本行目的：初始化逻辑操作符为 and。
    const activeLogic = ref<LogicOp>("and");
    // 变量：searchTerms | 含义：当前搜索关键词列表 | 类型：Ref<SearchTerm[]> | 作用域：useSearchPage 内部
    // 本行目的：初始化关键词数组。
    const searchTerms = ref<SearchTerm[]>([]);

    // 变量：termSeq | 含义：关键词自增ID计数器 | 类型：number | 作用域：useSearchPage 内部
    // 本行目的：初始化关键词 ID 计数。
    let termSeq = 1;

    // 变量：customSearchInput | 含义：自定义输入框文本 | 类型：Ref<string> | 作用域：useSearchPage 内部
    const customSearchInput = ref("");
    // 变量：trackedSelection | 含义：当前选中的追踪条目ID | 类型：Ref<number | null> | 作用域：useSearchPage 内部
    const trackedSelection = ref<number | null>(null);
    // 变量：searchLoading | 含义：搜索请求加载状态 | 类型：Ref<boolean> | 作用域：useSearchPage 内部
    const searchLoading = ref(false);
    // 变量：searchError | 含义：搜索错误信息 | 类型：Ref<string> | 作用域：useSearchPage 内部
    const searchError = ref("");
    // 变量：searchHtml | 含义：搜索页原始HTML（含base注入） | 类型：Ref<string> | 作用域：useSearchPage 内部
    const searchHtml = ref("");
    // 变量：searchResults | 含义：解析后的搜索结果列表 | 类型：Ref<SearchResult[]> | 作用域：useSearchPage 内部
    const searchResults = ref<SearchResult[]>([]);
    // 变量：aliasModalVisible | 含义：别名选择弹窗显示状态 | 类型：Ref<boolean> | 作用域：useSearchPage 内部
    const aliasModalVisible = ref(false);
    // 变量：aliasLoading | 含义：别名加载状态 | 类型：Ref<boolean> | 作用域：useSearchPage 内部
    const aliasLoading = ref(false);
    // 变量：aliasOptions | 含义：别名候选列表 | 类型：Ref<string[]> | 作用域：useSearchPage 内部
    const aliasOptions = ref<string[]>([]);
    // 变量：aliasSelected | 含义：用户选中的别名列表 | 类型：Ref<string[]> | 作用域：useSearchPage 内部
    const aliasSelected = ref<string[]>([]);
    // 变量：pendingTrackedId | 含义：等待确认别名的条目ID | 类型：Ref<number | null> | 作用域：useSearchPage 内部
    const pendingTrackedId = ref<number | null>(null);
    // 变量：pendingTrackedName | 含义：等待确认别名的条目名称 | 类型：Ref<string> | 作用域：useSearchPage 内部
    const pendingTrackedName = ref("");

    // 变量：trackedOptions | 含义：追踪条目分组选项（在看/想看） | 类型：ComputedRef<Array<{labelKey:string;options:Array<{label:string;value:number}>}>> | 作用域：useSearchPage 内部
    /** 函数：trackedOptions | 输入：无 | 输出：按状态分组的条目选项 | 可能失败：无（纯计算） */
    // 本行目的：从追踪列表计算下拉分组选项。
    const trackedOptions = computed(() => {
        // 变量：byName | 含义：按名称排序比较函数 | 类型：(a: TrackedItem, b: TrackedItem) => number | 作用域：computed 回调内
        // 本行目的：定义按中文优先名称排序规则。
        const byName = (a: TrackedItem, b: TrackedItem) => (a.nameCn || a.name).localeCompare(b.nameCn || b.name, "zh-Hans-CN");

        // 本行目的：提取“在看”分组并排序映射。
        const watching = options.trackedItems.value
            .filter((item) => item.watching)
            // 本行目的：复制数组避免原数组被原地排序影响。
            .slice()
            // 本行目的：按名称规则进行稳定排序。
            .sort(byName)
            // 本行目的：映射成下拉选项结构。
            .map((item) => ({ label: item.nameCn || item.name, value: item.id }));

        // 本行目的：提取“想看”分组并排序映射。
        const backlog = options.trackedItems.value
            .filter((item) => item.backlog)
            // 本行目的：复制数组避免原数组被原地排序影响。
            .slice()
            // 本行目的：按名称规则进行稳定排序。
            .sort(byName)
            // 本行目的：映射成下拉选项结构。
            .map((item) => ({ label: item.nameCn || item.name, value: item.id }));

        // 本行目的：返回分组结构供下拉组件使用。
        return [
            { labelKey: "search.anime.group.watching", options: watching },
            { labelKey: "search.anime.group.backlog", options: backlog },
        ];
    });

    // 变量：addSearchTerm | 含义：新增关键词到搜索词列表 | 类型：(value: string, source: SearchTerm["source"], op?: LogicOp) => void | 作用域：useSearchPage 内部
    /** 函数：addSearchTerm | 输入：词值、来源、可选逻辑操作符 | 输出：无（更新关键词列表） | 可能失败：无（纯状态更新） */
    // 本行目的：定义关键词新增逻辑。
    const addSearchTerm = (value: string, source: SearchTerm["source"], op?: LogicOp) => {
        // 本行目的：去除前后空白。
        const trimmed = value.trim();
        // 本行目的：空字符串不加入。
        if (!trimmed) return;
        // 本行目的：追加新关键词并分配唯一ID。
        searchTerms.value = [...searchTerms.value, { id: termSeq++, value: trimmed, op: op ?? activeLogic.value, source }];
    };

    // 变量：handleAddPreset | 含义：添加预置关键词 | 类型：(value: string) => void | 作用域：useSearchPage 内部
    /** 函数：handleAddPreset | 输入：预置词 | 输出：无（更新关键词列表） | 可能失败：无 */
    // 本行目的：把预置词加入关键词列表。
    const handleAddPreset = (value: string) => {
        addSearchTerm(value, "preset");
    };

    // 变量：handleAddCustom | 含义：添加自定义关键词并清空输入 | 类型：() => void | 作用域：useSearchPage 内部
    /** 函数：handleAddCustom | 输入：无（读取 customSearchInput） | 输出：无（更新关键词列表） | 可能失败：无 */
    // 本行目的：提交自定义关键词。
    const handleAddCustom = () => {
        // 本行目的：添加输入框中的关键词。
        addSearchTerm(customSearchInput.value, "custom");
        // 本行目的：清空输入框。
        customSearchInput.value = "";
    };

    // 变量：removeTrackedTerms | 含义：移除所有来源为 tracked 的关键词 | 类型：() => void | 作用域：useSearchPage 内部
    /** 函数：removeTrackedTerms | 输入：无 | 输出：无（更新关键词列表） | 可能失败：无 */
    // 本行目的：清除旧的追踪别名关键词。
    const removeTrackedTerms = () => {
        searchTerms.value = searchTerms.value.filter((t) => t.source !== "tracked");
    };

    // 变量：handleSelectTracked | 含义：处理追踪条目选择并拉取别名候选 | 类型：(value: number | null) => Promise<void> | 作用域：useSearchPage 内部
    /** 函数：handleSelectTracked | 输入：追踪条目ID或空 | 输出：无（更新别名弹窗与关键词状态） | 可能失败：别名接口调用失败 */
    // 本行目的：定义追踪条目选择逻辑。
    const handleSelectTracked = async (value: number | null) => {
        // 本行目的：选择为空时重置追踪相关状态。
        if (value == null) {
            // 本行目的：清空当前追踪选择。
            trackedSelection.value = null;
            // 本行目的：移除旧的追踪关键词。
            removeTrackedTerms();
            // 本行目的：关闭别名弹窗。
            aliasModalVisible.value = false;
            // 本行目的：结束当前处理流程。
            return;
        }

        // 本行目的：在追踪列表中查找对应条目。
        const target = options.trackedItems.value.find((item) => item.id === value);
        // 本行目的：目标不存在时回退并清理状态。
        if (!target) {
            // 本行目的：清空当前追踪选择。
            trackedSelection.value = null;
            // 本行目的：移除旧的追踪关键词。
            removeTrackedTerms();
            // 本行目的：关闭别名弹窗。
            aliasModalVisible.value = false;
            // 本行目的：结束当前处理流程。
            return;
        }

        // 本行目的：设置待确认条目信息并打开别名弹窗。
        pendingTrackedId.value = value;
        // 本行目的：缓存当前条目名称用于弹窗显示。
        pendingTrackedName.value = target.nameCn || target.name || "";
        // 本行目的：打开别名选择弹窗。
        aliasModalVisible.value = true;
        // 本行目的：进入别名加载状态。
        aliasLoading.value = true;
        // 本行目的：重置别名候选列表。
        aliasOptions.value = [];
        // 本行目的：重置已选别名列表。
        aliasSelected.value = [];

        // 变量：aliasSet | 含义：去重后的别名集合 | 类型：Set<string> | 作用域：handleSelectTracked 内部
        // 本行目的：创建别名去重集合。
        const aliasSet = new Set<string>();

        // 变量：baseName | 含义：条目基础名称（中文优先） | 类型：string | 作用域：handleSelectTracked 内部
        // 本行目的：读取基础名称并去空白。
        const baseName = (pendingTrackedName.value || "").trim();
        // 本行目的：基础名称有效时加入候选集合。
        if (baseName) aliasSet.add(baseName);

        try {
            // 本行目的：调用后端获取别名列表。
            const payload = await invoke<{ id: number; aliases: string[] }>("get_subject_aliases", { id: value });
            // 本行目的：遍历别名并去重后加入集合。
            (payload.aliases || []).forEach((alias) => {
                // 本行目的：清理单个别名两端空白。
                const trimmed = (alias || "").trim();
                // 本行目的：只添加非空且未重复的别名。
                if (trimmed && !aliasSet.has(trimmed)) aliasSet.add(trimmed);
            });
        } catch (_) {
            // 本行目的：获取别名失败时静默处理，至少保留基础名称。
        } finally {
            // 变量：list | 含义：集合转数组后的候选列表 | 类型：string[] | 作用域：finally 内
            // 本行目的：把别名集合转换为数组。
            const list = Array.from(aliasSet);
            // 变量：base | 含义：基础名称快照 | 类型：string | 作用域：finally 内
            // 本行目的：读取基础名称快照，便于后续排序。
            const base = baseName || "";
            // 变量：ordered | 含义：基础名称优先的最终候选列表 | 类型：string[] | 作用域：finally 内
            // 本行目的：让基础名称排在候选列表首位。
            const ordered = base ? [base, ...list.filter((v) => v !== base)] : list;

            // 本行目的：写入别名候选与默认选中状态。
            aliasOptions.value = ordered;
            // 本行目的：默认不预选任何别名，等待用户勾选。
            aliasSelected.value = [];
            // 本行目的：结束别名加载状态。
            aliasLoading.value = false;
        }
    };

    // 变量：cancelAliasSelection | 含义：取消别名选择并重置相关状态 | 类型：() => void | 作用域：useSearchPage 内部
    /** 函数：cancelAliasSelection | 输入：无 | 输出：无（重置状态） | 可能失败：无 */
    // 本行目的：定义取消别名弹窗后的状态清理逻辑。
    const cancelAliasSelection = () => {
        // 本行目的：关闭别名弹窗。
        aliasModalVisible.value = false;
        // 本行目的：结束别名加载状态。
        aliasLoading.value = false;
        // 本行目的：清空待确认条目ID。
        pendingTrackedId.value = null;
        // 本行目的：清空待确认条目名称。
        pendingTrackedName.value = "";
        // 本行目的：清空别名候选。
        aliasOptions.value = [];
        // 本行目的：清空别名勾选结果。
        aliasSelected.value = [];
        // 本行目的：重置追踪下拉选择。
        trackedSelection.value = null;
    };

    // 变量：confirmAliasSelection | 含义：确认别名并写入 tracked 关键词 | 类型：() => void | 作用域：useSearchPage 内部
    /** 函数：confirmAliasSelection | 输入：无（读取待确认与已选别名） | 输出：无（更新关键词列表） | 可能失败：无（纯状态更新） */
    // 本行目的：定义确认别名后的关键词写入逻辑。
    const confirmAliasSelection = () => {
        // 本行目的：无待确认ID时直接取消。
        if (!pendingTrackedId.value) {
            // 本行目的：执行取消并重置临时状态。
            cancelAliasSelection();
            // 本行目的：结束当前确认流程。
            return;
        }

        // 本行目的：未选择任何别名时直接取消。
        if (!aliasSelected.value.length) {
            // 本行目的：执行取消并重置临时状态。
            cancelAliasSelection();
            // 本行目的：结束当前确认流程。
            return;
        }

        // 本行目的：清除旧的 tracked 关键词。
        removeTrackedTerms();

        // 本行目的：确认当前追踪选择。
        trackedSelection.value = pendingTrackedId.value;

        // 变量：seen | 含义：已加入关键词的去重集合 | 类型：Set<string> | 作用域：confirmAliasSelection 内部
        // 本行目的：创建去重集合避免重复关键词。
        const seen = new Set<string>();

        // 本行目的：遍历已选别名并按 and 逻辑加入关键词。
        aliasSelected.value.forEach((term) => {
            // 本行目的：去除别名单词两端空白。
            const trimmed = term.trim();
            if (trimmed && !seen.has(trimmed)) {
                // 本行目的：记录该别名已处理，防止重复添加。
                seen.add(trimmed);
                // 本行目的：以 tracked 来源和 and 逻辑加入关键词。
                addSearchTerm(trimmed, "tracked", "and");
            }
        });

        // 本行目的：确认后关闭弹窗并清理临时状态。
        cancelAliasSelection();
    };

    // 变量：removeSearchTerm | 含义：移除指定关键词（按id或索引） | 类型：(idOrIndex: number, useIndex?: boolean) => void | 作用域：useSearchPage 内部
    /** 函数：removeSearchTerm | 输入：id或索引、是否按索引 | 输出：无（更新关键词列表） | 可能失败：无 */
    // 本行目的：定义关键词删除逻辑。
    const removeSearchTerm = (idOrIndex: number, useIndex = false) => {
        // 本行目的：按索引删除模式。
        if (useIndex) {
            // 本行目的：过滤并移除目标索引对应的关键词。
            searchTerms.value = searchTerms.value.filter((_, index) => index !== idOrIndex);
            // 本行目的：按索引删除完成后结束函数。
            return;
        }
        // 本行目的：按关键词ID删除模式。
        searchTerms.value = searchTerms.value.filter((term) => term.id !== idOrIndex);
    };

    // 变量：queryParts | 含义：按逻辑运算拼装后的查询片段数组 | 类型：ComputedRef<string[]> | 作用域：useSearchPage 内部
    /** 函数：queryParts | 输入：无 | 输出：查询片段数组 | 可能失败：无（纯计算） */
    // 本行目的：把关键词按 and/or/not 规则转换成站点查询表达式。
    const queryParts = computed(() => {
        // 本行目的：收集 and 关键词并加引号。
        const andTerms = searchTerms.value.filter((t) => t.op === "and").map((t) => `"${t.value}"`);
        // 本行目的：收集 not 关键词并加负号。
        const notTerms = searchTerms.value.filter((t) => t.op === "not").map((t) => `-"${t.value}"`);
        // 本行目的：收集 or 关键词原始片段。
        const orTermsRaw = searchTerms.value.filter((t) => t.op === "or").map((t) => `"${t.value}"`);
        // 本行目的：把 or 关键词用 | 连接。
        const orTerm = orTermsRaw.length ? orTermsRaw.join("|") : "";
        // 本行目的：按顺序组合全部片段。
        const parts = [...andTerms, ...(orTerm ? [orTerm] : []), ...notTerms];
        // 本行目的：过滤空片段并返回。
        return parts.filter(Boolean);
    });

    // 变量：searchQuery | 含义：最终查询字符串 | 类型：ComputedRef<string> | 作用域：useSearchPage 内部
    /** 函数：searchQuery | 输入：无 | 输出：完整查询文本 | 可能失败：无（纯派生） */
    // 本行目的：把查询片段数组拼成最终查询字符串。
    const searchQuery = computed(() => queryParts.value.join(" "));
    // 变量：searchUrl | 含义：最终搜索URL | 类型：ComputedRef<string> | 作用域：useSearchPage 内部
    /** 函数：searchUrl | 输入：无 | 输出：编码后的搜索URL | 可能失败：无（纯派生） */
    // 本行目的：把查询字符串编码后拼接成完整搜索URL。
    const searchUrl = computed(() => `${NYAA_BASE}${encodeURIComponent(searchQuery.value)}`);

    // 变量：injectBaseTag | 含义：为HTML注入base标签以修正相对链接 | 类型：(html: string, url: string) => string | 作用域：useSearchPage 内部
    /** 函数：injectBaseTag | 输入：原始HTML和当前URL | 输出：注入base后的HTML | 可能失败：无（纯字符串处理） */
    // 本行目的：定义 HTML base 标签注入逻辑。
    const injectBaseTag = (html: string, url: string) => {
        // 本行目的：提取基础地址（去查询参数）。
        const baseHref = url.split("?")[0];
        // 本行目的：构建 base 标签字符串。
        const baseTag = `<base href="${baseHref}" />`;
        // 本行目的：查找 head 起始标签。
        const headMatch = html.match(/<head[^>]*>/i);

        // 本行目的：命中 head 时插入 base 标签。
        if (headMatch && headMatch.index !== undefined) {
            // 本行目的：计算插入 base 标签的位置。
            const idx = headMatch.index + headMatch[0].length;
            // 本行目的：返回插入 base 标签后的 HTML。
            return `${html.slice(0, idx)}${baseTag}${html.slice(idx)}`;
        }

        // 本行目的：未命中 head 时在前面补一个 head 包含 base。
        return `<head>${baseTag}</head>${html}`;
    };

    // 变量：parseSearchResults | 含义：从搜索HTML中提取结构化结果 | 类型：(html: string) => SearchResult[] | 作用域：useSearchPage 内部
    /** 函数：parseSearchResults | 输入：搜索结果HTML | 输出：解析后的结果数组 | 可能失败：DOM解析异常、节点缺失 */
    // 本行目的：定义搜索结果解析器。
    const parseSearchResults = (html: string): SearchResult[] => {
        try {
            // 本行目的：创建 DOM 解析器。
            const parser = new DOMParser();
            // 本行目的：把HTML字符串转成文档对象。
            const doc = parser.parseFromString(html, "text/html");
            // 本行目的：定位结果表格的行节点。
            const rows = Array.from(doc.querySelectorAll("table tbody tr"));
            // 变量：toAbs | 含义：把相对链接转成绝对链接 | 类型：(href?: string | null) => string | undefined | 作用域：parseSearchResults 内部
            // 本行目的：定义链接归一化函数。
            const toAbs = (href?: string | null) => {
                // 本行目的：空链接返回 undefined。
                if (!href) return undefined;

                try {
                    // 本行目的：优先使用 URL 构造器转绝对地址。
                    return new URL(href, searchUrl.value).toString();
                } catch (_) {
                    // 本行目的：转换失败时返回原链接兜底。
                    return href || undefined;
                }
            };

            // 本行目的：遍历每一行并转换为结构化搜索结果。
            return rows
                // 本行目的：逐行提取标题、链接与元数据字段。
                .map((tr) => {
                    // 本行目的：定位名称列单元格。
                    const nameCell = tr.querySelector("td:nth-child(2)");
                    // 本行目的：收集名称列中的所有链接。
                    const anchorCandidates = nameCell ? Array.from(nameCell.querySelectorAll("a")) : [];
                    // 本行目的：提取详情页链接（/view/）。
                    const nameLink = anchorCandidates.filter((a) => a.getAttribute("href")?.includes("/view/")).pop();

                    // 本行目的：无详情链接则忽略该行。
                    if (!nameLink) return null;

                    // 本行目的：提取标题文本。
                    const title = nameLink.textContent?.trim() || "";
                    // 本行目的：提取详情URL并转绝对地址。
                    const detailUrl = toAbs(nameLink.getAttribute("href"));
                    // 本行目的：提取磁力链接。
                    const magnet = toAbs(tr.querySelector("a[href^='magnet:']")?.getAttribute("href"));
                    // 本行目的：提取 torrent 下载链接。
                    const download = toAbs(tr.querySelector("a[href$='.torrent']")?.getAttribute("href"));
                    // 本行目的：提取文件大小文本。
                    const size = tr.querySelector("td:nth-child(5)")?.textContent?.trim() || undefined;
                    // 本行目的：提取发布时间文本。
                    const date = tr.querySelector("td:nth-child(6)")?.textContent?.trim() || undefined;

                    // 本行目的：无标题视为无效行。
                    if (!title) return null;

                    // 本行目的：构造结构化结果对象。
                    return { title, detailUrl, magnet, download, size, date } as SearchResult;
                })
                // 本行目的：过滤掉空结果并保留有效类型。
                .filter((v): v is SearchResult => Boolean(v));
        } catch (error) {
            // 本行目的：记录解析错误。
            console.error("parseSearchResults error", error);
            // 本行目的：解析失败时返回空数组。
            return [];
        }
    };

    // 变量：openSearch | 含义：执行搜索请求并回填 HTML 与结果列表 | 类型：() => Promise<void> | 作用域：useSearchPage 内部
    /** 函数：openSearch | 输入：无（读取 searchQuery/searchUrl） | 输出：无（更新搜索状态） | 可能失败：后端抓取失败 */
    // 本行目的：定义搜索执行主流程。
    const openSearch = async () => {
        // 本行目的：无查询词时不发起请求。
        if (!searchQuery.value) return;

        // 本行目的：初始化本次搜索状态。
        searchLoading.value = true;
        // 本行目的：清空历史错误信息。
        searchError.value = "";
        // 本行目的：清空历史结果列表。
        searchResults.value = [];
        // 本行目的：清空历史HTML预览。
        searchHtml.value = "";

        try {
            // 本行目的：调用后端获取搜索页HTML。
            const html = await invoke<string>("fetch_search_html", { url: searchUrl.value });
            // 本行目的：注入 base 标签后保存到预览HTML。
            searchHtml.value = injectBaseTag(html, searchUrl.value);
            // 本行目的：解析结构化结果列表。
            searchResults.value = parseSearchResults(html);
        } catch (err: any) {
            // 本行目的：提取可读错误消息并写入状态。
            searchError.value = typeof err === "string" ? err : err?.message || options.errorFallback?.() || "获取搜索结果失败";
        } finally {
            // 本行目的：关闭加载状态。
            searchLoading.value = false;
        }
    };

    // 变量：clearSearchResults | 含义：清空搜索结果与错误信息 | 类型：() => void | 作用域：useSearchPage 内部
    /** 函数：clearSearchResults | 输入：无 | 输出：无（重置结果状态） | 可能失败：无 */
    // 本行目的：定义搜索结果清空逻辑。
    const clearSearchResults = () => {
        // 本行目的：清空结果数组。
        searchResults.value = [];
        // 本行目的：清空HTML预览。
        searchHtml.value = "";
        // 本行目的：清空错误消息。
        searchError.value = "";
    };

    // 本行目的：返回搜索页所需状态与行为集合。
    return {
        // 本行目的：导出预置短语。
        presetPhrases,
        // 本行目的：导出逻辑选项。
        logicOptions,
        // 本行目的：导出当前逻辑操作符。
        activeLogic,
        // 本行目的：导出关键词数组。
        searchTerms,
        // 本行目的：导出自定义输入值。
        customSearchInput,
        // 本行目的：导出追踪选择值。
        trackedSelection,
        // 本行目的：导出追踪分组选项。
        trackedOptions,
        // 本行目的：导出搜索加载状态。
        searchLoading,
        // 本行目的：导出搜索错误。
        searchError,
        // 本行目的：导出搜索HTML。
        searchHtml,
        // 本行目的：导出搜索结果。
        searchResults,
        // 本行目的：导出别名弹窗可见状态。
        aliasModalVisible,
        // 本行目的：导出别名加载状态。
        aliasLoading,
        // 本行目的：导出别名候选。
        aliasOptions,
        // 本行目的：导出别名已选值。
        aliasSelected,
        // 本行目的：导出待确认条目名称。
        pendingTrackedName,
        // 本行目的：导出最终查询文本。
        searchQuery,
        // 本行目的：导出最终搜索URL。
        searchUrl,
        // 本行目的：导出添加预置词函数。
        handleAddPreset,
        // 本行目的：导出添加自定义词函数。
        handleAddCustom,
        // 本行目的：导出追踪选择处理函数。
        handleSelectTracked,
        // 本行目的：导出关键词删除函数。
        removeSearchTerm,
        // 本行目的：导出搜索执行函数。
        openSearch,
        // 本行目的：导出结果清空函数。
        clearSearchResults,
        // 本行目的：导出别名取消函数。
        cancelAliasSelection,
        // 本行目的：导出别名确认函数。
        confirmAliasSelection,
    };
};

/** 类型：UseSearchPageReturn | 用途：导出 useSearchPage 返回值类型 | 字段：由 ReturnType 自动推断 */
// 本行目的：导出组合式函数返回类型供页面组件复用。
export type UseSearchPageReturn = ReturnType<typeof useSearchPage>;
