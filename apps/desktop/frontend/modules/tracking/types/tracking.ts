import type { MonthAnime } from "./anime";

/**
 * 追番状态：三者互斥（只会有一个 true）。
 */
export type ItemStatus = { watching: boolean; backlog: boolean; watched: boolean };

/**
 * 已追番条目 = 基础条目 + 状态字段。
 */
export type TrackedItem = MonthAnime & ItemStatus;

/**
 * 用于 UI 的状态枚举（null 表示未选择）。
 */
export type StatusKey = "watching" | "backlog" | "watched" | null;
