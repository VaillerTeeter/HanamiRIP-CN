/** 文件：page.ts | 用途：定义应用可切换页面键的统一类型 | 关键对象：PageKey */
/** 类型：PageKey | 用途：约束标题栏和路由状态可用页面键 | 取值：query/watching/backlog/finished/search/download/tracks */
// 本行目的：导出页面键联合字面量类型。
export type PageKey = "query" | "watching" | "backlog" | "finished" | "search" | "download" | "tracks";
