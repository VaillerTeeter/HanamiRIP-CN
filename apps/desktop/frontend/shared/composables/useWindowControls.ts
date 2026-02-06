/*
  窗口控制工具：提供最小化/关闭按钮逻辑。
  在非 Tauri 环境下会返回空操作，避免浏览器报错。
*/
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "../utils/tauri";

/**
 * 获取窗口控制方法（最小化/关闭）。
 */
export const useWindowControls = () => {
  // 非 Tauri 环境下直接返回 noop。
  if (!isTauri()) {
    const noop = async () => undefined;
    return { handleMinimize: noop, handleClose: noop };
  }

  // 获取当前窗口实例（可能抛错）。
  let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
  try {
    appWindow = getCurrentWindow();
  } catch (err) {
    // 如果获取失败，返回 noop，避免 UI 崩溃。
    console.warn("getCurrentWindow failed", err);
    const noop = async () => undefined;
    return { handleMinimize: noop, handleClose: noop };
  }

  // 最小化窗口。
  const handleMinimize = async () => {
    try {
      await appWindow?.minimize();
    } catch (err) {
      console.error("minimize failed", err);
    }
  };

  // 关闭窗口。
  const handleClose = async () => {
    try {
      await appWindow?.close();
    } catch (err) {
      console.error("close failed", err);
    }
  };

  return { handleMinimize, handleClose };
};
