/** 文件：async-state.ts | 用途：定义异步状态组件通用阶段与载荷类型 | 关键对象：AsyncPhase, AsyncStatePayload */
/** 类型：AsyncPhase | 用途：约束异步流程阶段枚举 | 取值：idle、loading、empty、error、ready */
// 本行目的：导出异步阶段联合字面量类型。
export type AsyncPhase = "idle" | "loading" | "empty" | "error" | "ready";

/** 类型：AsyncStatePayload | 用途：描述异步状态传输载荷 | 字段：phase、emptyMessage、errorMessage */
// 本行目的：导出异步状态载荷接口。
export interface AsyncStatePayload {
    // 本行目的：定义当前阶段。
    phase: AsyncPhase;
    // 本行目的：定义空状态文案（可选）。
    emptyMessage?: string;
    // 本行目的：定义错误状态文案（可选）。
    errorMessage?: string;
}
