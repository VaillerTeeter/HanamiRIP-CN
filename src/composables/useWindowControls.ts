import { getCurrentWindow } from "@tauri-apps/api/window";

export const useWindowControls = () => {
  const appWindow = getCurrentWindow();

  const handleMinimize = async () => {
    await appWindow.minimize();
  };

  const handleClose = async () => {
    await appWindow.close();
  };

  return { handleMinimize, handleClose };
};
