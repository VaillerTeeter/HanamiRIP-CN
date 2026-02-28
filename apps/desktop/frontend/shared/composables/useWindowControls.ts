/** 文件：useWindowControls.ts | 用途：封装窗口最小化与关闭控制逻辑 | 关键对象：useWindowControls, handleMinimize, handleClose */
// 本行目的：引入当前窗口对象获取函数。
import { getCurrentWindow } from "@tauri-apps/api/window";

// 变量：useWindowControls | 含义：窗口控制组合式函数入口 | 类型：() => { handleMinimize: () => Promise<void>; handleClose: () => Promise<void> } | 作用域：模块级
/** 函数：useWindowControls | 输入：无 | 输出：窗口控制方法集合 | 可能失败：获取窗口失败时返回 noop 方法 */
// 本行目的：导出窗口控制能力。
export const useWindowControls = () => {
    // 变量：noop | 含义：失败场景下的空异步函数 | 类型：() => Promise<undefined> | 作用域：useWindowControls 内部
    // 本行目的：定义兜底空操作函数。
    const noop = async () => undefined;

    // 变量：appWindow | 含义：当前应用窗口对象 | 类型：ReturnType<typeof getCurrentWindow> | null | 作用域：useWindowControls 内部
    // 本行目的：初始化窗口对象引用。
    let appWindow: ReturnType<typeof getCurrentWindow> | null = null;

    try {
        // 本行目的：尝试获取当前窗口实例。
        appWindow = getCurrentWindow();
    } catch (error) {
        // 本行目的：记录获取窗口失败日志。
        console.warn("getCurrentWindow failed", error);
        // 本行目的：返回空操作方法，避免调用方报错。
        return { handleMinimize: noop, handleClose: noop };
    }

    // 变量：handleMinimize | 含义：最小化窗口方法 | 类型：() => Promise<void> | 作用域：useWindowControls 内部
    /** 函数：handleMinimize | 输入：无 | 输出：无 | 可能失败：窗口最小化失败时记录错误 */
    // 本行目的：定义窗口最小化行为。
    const handleMinimize = async () => {
        try {
            // 本行目的：调用窗口最小化 API。
            await appWindow?.minimize();
        } catch (error) {
            // 本行目的：记录最小化失败日志。
            console.error("minimize failed", error);
        }
    };

    // 变量：handleClose | 含义：关闭窗口方法 | 类型：() => Promise<void> | 作用域：useWindowControls 内部
    /** 函数：handleClose | 输入：无 | 输出：无 | 可能失败：窗口关闭失败时记录错误 */
    // 本行目的：定义窗口关闭行为。
    const handleClose = async () => {
        try {
            // 本行目的：调用窗口关闭 API。
            await appWindow?.close();
        } catch (error) {
            // 本行目的：记录关闭失败日志。
            console.error("close failed", error);
        }
    };

    // 本行目的：返回窗口控制方法集合。
    return { handleMinimize, handleClose };
};
