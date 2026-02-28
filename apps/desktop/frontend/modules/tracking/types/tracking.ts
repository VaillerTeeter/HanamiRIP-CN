/** 文件：tracking.ts | 用途：定义追踪状态与追踪条目的核心类型别名 | 关键对象：ItemStatus, TrackedItem, StatusKey */
// 本行目的：引入 MonthAnime 类型，作为追踪条目的基础信息结构。
import type { MonthAnime } from "./anime";

/** 类型：ItemStatus | 用途：描述追踪模块的三态互斥状态对象 | 字段：watching、backlog、watched */
// 本行目的：导出条目状态对象类型。
export type ItemStatus = {
	// 本行目的：表示条目是否处于“在看”状态。
	watching: boolean;
	// 本行目的：表示条目是否处于“想看”状态。
	backlog: boolean;
	// 本行目的：表示条目是否处于“看完”状态。
	watched: boolean;
};

/** 类型：TrackedItem | 用途：把动画基础信息与追踪状态合并为完整条目类型 | 结构：MonthAnime + ItemStatus */
// 本行目的：导出追踪条目类型（交叉类型合并）。
export type TrackedItem = MonthAnime & ItemStatus;

/** 类型：StatusKey | 用途：约束可切换的状态键与空状态 | 取值：watching、backlog、watched、null */
// 本行目的：导出状态键联合类型。
export type StatusKey = "watching" | "backlog" | "watched" | null;
