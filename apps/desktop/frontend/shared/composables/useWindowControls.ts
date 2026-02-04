import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "../utils/tauri";

export const useWindowControls = () => {
  if (!isTauri()) {
    const noop = async () => undefined;
    return { handleMinimize: noop, handleClose: noop };
  }

  let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
  try {
    appWindow = getCurrentWindow();
  } catch (err) {
    console.warn("getCurrentWindow failed", err);
    const noop = async () => undefined;
    return { handleMinimize: noop, handleClose: noop };
  }

  const handleMinimize = async () => {
    try {
      await appWindow?.minimize();
    } catch (err) {
      console.error("minimize failed", err);
    }
  };

  const handleClose = async () => {
    try {
      await appWindow?.close();
    } catch (err) {
      console.error("close failed", err);
    }
  };

  return { handleMinimize, handleClose };
};
